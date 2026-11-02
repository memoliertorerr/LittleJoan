import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import {
  GLASS_RUNS,
  MAC_BACKDROP,
  glassOffFailures,
  glassRunFailures,
  glassTimingFailures,
  parseGlassSupport,
  parseGlassView,
  versionHasGlass,
  wantedView,
  type GlassRun,
} from "./smoke-glass.mts";
import type { Measured, RunMode, SummonTiming } from "./smoke-timing.mts";

const log = (lines: string[]) => parseLog(lines.map((line, i) => `${i + 1} ${line}`).join("\n") + "\n");

describe("the glass-support line", () => {
  it("parses what the system has", () => {
    expect(parseGlassSupport(log(["glass-support available=true os=26 reduce=false contrast=true"]))).toEqual({
      available: true,
      os: 26,
      reduce: false,
      contrast: true,
    });
    expect(parseGlassSupport(log(["glass-support available=false os=- reduce=false contrast=false"]))?.os).toBeNull();
    expect(parseGlassSupport(log(["glass-support yes"]))).toBeUndefined();
    expect(parseGlassSupport(log(["started"]))).toBeUndefined();
  });

  it("matches settings/glass.rs: macOS 26 and later (16 is Tahoe's compat number)", () => {
    for (const old of [12, 13, 14, 15, 17, 25]) {
      expect(versionHasGlass(old)).toBe(false);
    }
    for (const yes of [16, 26, 27]) {
      expect(versionHasGlass(yes)).toBe(true);
    }
  });
});

describe("the main run, glass Off", () => {
  const plain = ["started", "theme-early light colors=1", "theme light colors=1", "theme-summon light colors=1"];

  it("passes when the OS says what it has and nothing of glass shows up", () => {
    expect(glassOffFailures(log(["glass-support available=true os=26 reduce=false contrast=false", ...plain]), "darwin")).toEqual([]);
    expect(glassOffFailures(log(["glass-support available=false os=15 reduce=false contrast=false", ...plain]), "darwin")).toEqual([]);
    for (const os of ["win32", "linux"]) {
      expect(glassOffFailures(log(["glass-support available=false os=- reduce=false contrast=false", ...plain]), os)).toEqual([]);
    }
  });

  it("fails when the OS is misread, or Off touched the window or the bar", () => {
    expect(glassOffFailures(log(plain), "darwin")).toEqual(["glass: no glass-support line at launch"]);
    expect(glassOffFailures(log(["glass-support available=false os=26 reduce=false contrast=false"]), "darwin")[0]).toMatch(/available=false, not true/);
    expect(glassOffFailures(log(["glass-support available=true os=15 reduce=false contrast=false"]), "darwin")[0]).toMatch(/not false/);
    expect(glassOffFailures(log(["glass-support available=true os=- reduce=false contrast=false"]), "linux")[0]).toMatch(/linux reports glass/);
    const touched = glassOffFailures(
      log([
        "glass-support available=true os=26 reduce=false contrast=false",
        "glass-view clear back=true",
        "bar-surface clear html=rgba(0, 0, 0, 0)",
        "theme light colors=1 glass=clear",
      ]),
      "darwin",
    );
    expect(touched).toHaveLength(3);
  });
});

describe("glass-view lines", () => {
  it("parse the style and the fields", () => {
    expect(parseGlassView("regular back=true fills=true corner=16 tint=0.65 dark=true opaque=false window-alpha=0.00 views=3")).toEqual({
      style: "regular",
      fields: { back: "true", fills: "true", corner: "16", tint: "0.65", dark: "true", opaque: "false", "window-alpha": "0.00", views: "3" },
    });
    expect(parseGlassView("none opaque=true window-alpha=1.00 views=2")?.style).toBe("none");
    expect(parseGlassView("frosted back=true")).toBeUndefined();
  });
});

const tinted = GLASS_RUNS.find((run) => run.glass === "tinted") as GlassRun;
const clear = GLASS_RUNS.find((run) => run.glass === "clear") as GlassRun;
const SURFACE = (glass: string) => `bar-surface ${glass} html=rgba(0, 0, 0, 0) body=rgba(0, 0, 0, 0) image=none`;
const view = (style: string, run: GlassRun) =>
  `glass-view ${style} ${Object.entries(wantedView(run))
    .map(([key, value]) => `${key}=${value}`)
    .join(" ")} corner=16 views=3`;
const base = (run: GlassRun, style: string) => [
  "glass-support available=true os=26 reduce=false contrast=false",
  `theme-early ${run.theme.mode} colors=2 glass=${run.glass}`,
  view(style, run),
  SURFACE(run.glass),
  `theme ${run.theme.mode} colors=2 glass=${run.glass}`,
  "title LJ-SMOKE ready",
  "suggestions 1 first=127.0.0.1:5000 typed=0",
  `theme-summon ${run.theme.mode} colors=2 glass=${run.glass}`,
];
const OFF_VIEW = "glass-view none opaque=true window-alpha=1.00 views=2";

describe("glass runs", () => {
  it("are Clear in light and Tinted in dark, the tinted one switching accessibility on and off", () => {
    expect(GLASS_RUNS.map((run) => [run.glass, run.theme.mode])).toEqual([
      ["clear", "light"],
      ["tinted", "dark"],
    ]);
    expect(tinted.accessibility.map((step) => `${step.write}→${step.shows}`)).toEqual(["reduce→off", "none→tinted", "contrast→off", "none→tinted"]);
    expect(wantedView(clear).tint).toBe("0.25");
    expect(wantedView(tinted)).toMatchObject({ tint: "0.65", dark: "true", opaque: "false", "window-alpha": "0.00" });
  });

  it("pass with the glass behind the bar, the bar clear, and every switch followed", () => {
    expect(glassRunFailures(clear, log(base(clear, "clear")))).toEqual([]);
    const switched = [
      ...base(tinted, "regular"),
      "glass-accessibility reduce=true contrast=false",
      "glass off",
      OFF_VIEW,
      "glass-accessibility reduce=false contrast=false",
      "glass tinted",
      view("regular", tinted),
      "glass-accessibility reduce=false contrast=true",
      "glass off",
      OFF_VIEW,
      "glass-accessibility reduce=false contrast=false",
      "glass tinted",
      view("regular", tinted),
    ];
    expect(glassRunFailures(tinted, log(switched))).toEqual([]);
  });

  it("fail without glass, with the wrong style or a solid window, a painted bar, or a missed switch", () => {
    expect(glassRunFailures(clear, log(["glass-support available=false os=15 reduce=false contrast=false"]))[0]).toMatch(/no Liquid Glass on this runner/);
    const wrongStyle = base(clear, "regular");
    expect(glassRunFailures(clear, log(wrongStyle)).join("\n")).toMatch(/style regular, not clear/);
    const opaque = base(clear, "clear").map((line) => line.replace("opaque=false", "opaque=true"));
    expect(glassRunFailures(clear, log(opaque)).join("\n")).toMatch(/opaque=true, not false/);
    const painted = base(clear, "clear").map((line) => (line.startsWith("bar-surface") ? "bar-surface clear html=rgb(1, 2, 3) body=rgba(0, 0, 0, 0) image=set" : line));
    expect(glassRunFailures(clear, log(painted)).join("\n")).toMatch(/draws under itself/);
    const noGlassTheme = base(clear, "clear").map((line) => line.replace(" glass=clear", ""));
    expect(glassRunFailures(clear, log(noGlassTheme)).join("\n")).toMatch(/not glass=clear/);
    const missed = [...base(tinted, "regular"), "glass-accessibility reduce=true contrast=false", "glass off", view("regular", tinted)];
    const failures = glassRunFailures(tinted, log(missed)).join("\n");
    expect(failures).toMatch(/showed \[off\], not \[off, tinted, off, tinted\]/);
    expect(failures).toMatch(/after "reduce" — style regular, not none/);
  });
});

function measured(run: RunMode, kind: Measured["kind"], ready: number): Measured {
  const timing: SummonTiming = {
    source: "cli",
    kind: run.startsWith("ready") ? "ready" : "cold",
    mode: "ready",
    url: false,
    at: 0,
    nap: null,
    partial: false,
    marks: { commandBar: ready },
  };
  return { run, kind, trigger: "cli", evicted: "none", gapMs: 3000, sentAt: null, timing };
}

describe("glass timing", () => {
  const off = [measured("ready", "warm", 100), measured("ready", "warm", 120), measured("ready", "warm", 90), measured("hibernate", "warm", 2500), measured("hibernate", "warm", 2700), measured("hibernate", "warm", 2600)];

  it("passes when glass costs no more than 1.5× + the slack, and without glass runs at all", () => {
    expect(glassTimingFailures(off)).toEqual([]);
    const glass = [...off, measured("ready-glass", "warm", 130), measured("ready-glass", "warm", 140), measured("ready-glass", "warm", 300), measured("hibernate-glass", "warm", 3100), measured("hibernate-glass", "warm", 2900), measured("hibernate-glass", "warm", 3000)];
    expect(glassTimingFailures(glass)).toEqual([]);
  });

  it("fails a much slower summon with glass, or one that measured nothing", () => {
    const slow = [...off, measured("ready-glass", "warm", 400), measured("ready-glass", "warm", 450), measured("ready-glass", "warm", 500)];
    expect(glassTimingFailures(slow)[0]).toMatch(/ready-glass\): warm median 450 ms with glass, 100 ms without/);
    const none = [...off, measured("hibernate-glass", "first", 3000)];
    expect(glassTimingFailures(none)[0]).toMatch(/no warm median/);
  });
});

describe("the backdrop", () => {
  it("is a Swift script with a status file and a time limit, behind the app and never activating", () => {
    expect(MAC_BACKDROP).toContain("app.setActivationPolicy(.accessory)");
    expect(MAC_BACKDROP).toContain("window.ignoresMouseEvents = true");
    expect(MAC_BACKDROP).toContain("orderFrontRegardless()");
    expect(MAC_BACKDROP).toContain('"shown\\n"');
    expect(MAC_BACKDROP).toContain('statusPath + ".stop"');
    expect(MAC_BACKDROP).not.toContain("activate(");
  });
});
