import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import {
  PAGE_HEIGHT,
  RESIZE_SETTINGS,
  RESIZE_STEPS,
  burst,
  parseViewport,
  resizeFailures,
  resizeSummary,
  standing,
  stepResult,
  type ResizeStep,
} from "./smoke-resize.mts";

const PAGE = "http://127.0.0.1:5000/smoke.html";
const log = (rows: string[]) => parseLog(rows.map((row, i) => `${1000 + i} ${row}`).join("\n") + "\n");
const viewport = (width: number, ua: string) => `title LJ-SMOKE viewport=${width}x${PAGE_HEIGHT} ua=${ua} dpr=1`;

/** A step's log as a good run writes it: layouts, then (with a switch) one reload, then the report. */
function goodStep(step: ResizeStep): string[] {
  const width = step.widths.at(-1) ?? 0;
  const layouts = step.widths.map(() => "bounds-set 2");
  const before = step.ua === "phone" ? "desktop" : "phone";
  const switched = step.switches
    ? [viewport(width, before), `user-agent ${step.ua}`, `page-started ${PAGE}`, "title LJ-SMOKE ready h264=mse vp9=mse av1=mse"]
    : [];
  return [...layouts, ...switched, viewport(width, step.ua)];
}

const results = (steps: readonly ResizeStep[], rows: (step: ResizeStep) => string[]) =>
  steps.map((step) => stepResult(step, log(rows(step)), PAGE));

describe("the run's shape", () => {
  it("bursts 12 steps from 380 to 820 and back", () => {
    expect(burst(380, 820)).toEqual([380, 420, 460, 500, 540, 580, 620, 660, 700, 740, 780, 820]);
    expect(burst(820, 380)).toEqual([...burst(380, 820)].reverse());
    const up = RESIZE_STEPS.find((step) => step.name === "burst up");
    expect(up?.widths).toHaveLength(12);
    expect(up?.shot).toBe("resize-820.png");
    expect(RESIZE_STEPS.find((step) => step.name === "burst down")?.shot).toBe("resize-380.png");
  });

  it("runs dark, with the window created at the widest width", () => {
    expect(RESIZE_SETTINGS.theme.mode).toBe("dark");
    expect(RESIZE_SETTINGS.windowWidth).toBe(Math.max(...RESIZE_STEPS.flatMap((step) => step.widths)));
  });

  it("checks the hysteresis both ways", () => {
    const names = RESIZE_STEPS.map((step) => `${step.name}:${step.ua}:${step.switches}`);
    expect(names).toContain("620 from phone:phone:0");
    expect(names).toContain("660:desktop:1");
    expect(names).toContain("620 from desktop:desktop:0");
  });
});

describe("viewport reports", () => {
  it("parse", () => {
    expect(parseViewport("LJ-SMOKE viewport=380x560 ua=phone dpr=2")).toEqual({ width: 380, height: 560, ua: "phone" });
    expect(parseViewport("LJ-SMOKE ready h264=mse")).toBeUndefined();
    expect(parseViewport("LJ-FS ready width=390 ua=phone")).toBeUndefined();
  });

  it("stand only after the page's last load", () => {
    const events = log([viewport(380, "desktop"), "user-agent phone", `page-started ${PAGE}`]);
    expect(standing(events, PAGE)).toBeUndefined();
    const reported = log([viewport(380, "desktop"), `page-started ${PAGE}`, viewport(380, "phone")]);
    expect(standing(reported, PAGE)).toEqual({ width: 380, height: PAGE_HEIGHT, ua: "phone" });
    // Another page's load doesn't count.
    expect(standing(log([viewport(620, "phone"), "page-started https://example.com/"]), PAGE)?.width).toBe(620);
  });
});

describe("the rules", () => {
  it("pass a good run", () => {
    expect(resizeFailures(RESIZE_STEPS, results(RESIZE_STEPS, goodStep))).toEqual([]);
  });

  it("fail a wrong final viewport", () => {
    const got = results(RESIZE_STEPS, (step) => (step.name === "burst up" ? [...goodStep(step), viewport(780, "desktop")] : goodStep(step)));
    expect(resizeFailures(RESIZE_STEPS, got)).toEqual(["resize: burst up: the page ended at 780x560 ua=desktop, not 820x560 ua=desktop"]);
    const missing = results(RESIZE_STEPS, (step) => (step.name === "narrow" ? ["bounds-set 2"] : goodStep(step)));
    expect(resizeFailures(RESIZE_STEPS, missing)[0]).toContain("no report");
  });

  it("fail a flip-flop: two switches and two reloads for one crossing", () => {
    const flip = (step: ResizeStep) =>
      step.name === "burst down"
        ? ["bounds-set 2", "user-agent phone", `page-started ${PAGE}`, "user-agent desktop", "user-agent phone", `page-started ${PAGE}`, viewport(380, "phone")]
        : goodStep(step);
    expect(resizeFailures(RESIZE_STEPS, results(RESIZE_STEPS, flip))).toEqual([
      "resize: burst down: 1 switch to phone expected, saw [phone, desktop, phone]",
      "resize: burst down: 1 reload expected, saw 2",
    ]);
  });

  it("fail a switch inside the hysteresis band", () => {
    const band = (step: ResizeStep) =>
      step.name === "620 from phone" ? ["bounds-set 2", "user-agent desktop", `page-started ${PAGE}`, viewport(620, "desktop")] : goodStep(step);
    const failures = resizeFailures(RESIZE_STEPS, results(RESIZE_STEPS, band));
    expect(failures).toContain("resize: 620 from phone: the page ended at 620x560 ua=desktop, not 620x560 ua=phone");
    expect(failures).toContain("resize: 620 from phone: 0 switch to phone expected, saw [desktop]");
    expect(failures).toContain("resize: 620 from phone: 0 reload expected, saw 1");
  });

  it("fail more than two bounds per layout, no layout at all, and a resize that shows the page", () => {
    const bad = (step: ResizeStep) => {
      if (step.name === "narrow") {
        return goodStep(step).map((row) => (row === "bounds-set 2" ? "bounds-set 3" : row));
      }
      if (step.name === "660") {
        return goodStep(step).filter((row) => !row.startsWith("bounds-set"));
      }
      if (step.name === "620 from desktop") {
        return ["bounds-set 2", "page-visible true", viewport(620, "desktop")];
      }
      return goodStep(step);
    };
    expect(resizeFailures(RESIZE_STEPS, results(RESIZE_STEPS, bad))).toEqual([
      "resize: narrow: a layout set more than two webviews' bounds (3)",
      "resize: 660: no layout ran (no bounds-set line)",
      "resize: 620 from desktop: a resize showed or hid the page (page-visible true)",
    ]);
  });

  it("fail a step that never ran", () => {
    expect(resizeFailures(RESIZE_STEPS, [])).toHaveLength(RESIZE_STEPS.length);
  });

  it("allow layouts that set nothing (a Resized at the size the last layout already had)", () => {
    const quiet = (step: ResizeStep) => goodStep(step).map((row) => (row === "bounds-set 2" ? "bounds-set 0" : row));
    expect(resizeFailures(RESIZE_STEPS, results(RESIZE_STEPS, quiet))).toEqual([]);
  });
});

describe("the summary", () => {
  it("counts the layouts by how many bounds they set", () => {
    const step = RESIZE_STEPS[1] as ResizeStep;
    const [summary] = resizeSummary([stepResult(step, log(["bounds-set 2", "bounds-set 2", "bounds-set 1", viewport(820, "desktop")]), PAGE)]);
    expect(summary?.boundsSet).toEqual({ "2": 2, "1": 1 });
    expect(summary?.layouts).toBe(3);
  });
});
