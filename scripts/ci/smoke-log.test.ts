import { describe, expect, it } from "vitest";
import { count, evaluate, lastTitle, parseCodecs, parseLog, type LogEvent } from "./smoke-log.mts";

const GOOD = `1 started
2 shortcut-registered CommandOrControl+Shift+Space
3 bar-loaded
3 window-created
4 title Little Joan
5 bar-url http://127.0.0.1:5000/smoke.html
5 title LJ-SMOKE ready h264=mse vp9=mse av1=no
6 page-loaded http://127.0.0.1:5000/smoke.html
7 fullscreen true
8 title LJ-SMOKE fullscreen=on
9 hibernated
10 window-created
11 hibernated
`;

const NONE = { helpersBefore: 0, helpersAfter: 0 };

describe("parseLog", () => {
  it("reads events with and without details, skipping junk", () => {
    const events = parseLog("12 started\n13 title A  b\n\nnot a line\n14 hibernated\r\n");
    expect(events).toEqual<LogEvent[]>([
      { millis: 12, event: "started", detail: "" },
      { millis: 13, event: "title", detail: "A  b" },
      { millis: 14, event: "hibernated", detail: "" },
    ]);
  });
});

describe("queries", () => {
  const events = parseLog(GOOD);

  it("counts and finds titles", () => {
    expect(count(events, "window-created")).toBe(2);
    expect(count(events, "fullscreen", "true")).toBe(1);
    expect(lastTitle(events, "LJ-SMOKE ready")).toBe("LJ-SMOKE ready h264=mse vp9=mse av1=no");
    expect(lastTitle(events, "nope")).toBeUndefined();
  });

  it("parses codecs from the ready title", () => {
    expect(parseCodecs(lastTitle(events, "LJ-SMOKE ready"))).toEqual({ h264: "mse", vp9: "mse", av1: "no" });
    expect(parseCodecs(undefined)).toEqual({});
  });
});

describe("evaluate", () => {
  it("passes a complete run", () => {
    expect(evaluate(parseLog(GOOD), { windowFullscreen: true, pageFullscreen: true }, NONE)).toEqual([]);
  });

  it("reports missing steps and a second instance", () => {
    const failures = evaluate(
      parseLog("1 started\n2 started\n3 window-created\n"),
      { windowFullscreen: false, pageFullscreen: false },
      NONE,
    );
    expect(failures.join("\n")).toContain('missing "shortcut-registered"');
    expect(failures.join("\n")).toContain('exactly one "started"');
    expect(failures.join("\n")).toContain("second window");
  });

  it("requires fullscreen only where expected", () => {
    const noFullscreen = parseLog(GOOD.replace("7 fullscreen true\n", "").replace("8 title LJ-SMOKE fullscreen=on\n", ""));
    expect(evaluate(noFullscreen, { windowFullscreen: false, pageFullscreen: false }, NONE)).toEqual([]);
    expect(evaluate(noFullscreen, { windowFullscreen: true, pageFullscreen: false }, NONE)).toHaveLength(1);
    expect(evaluate(noFullscreen, { windowFullscreen: false, pageFullscreen: true }, NONE)).toHaveLength(1);
  });

  it("requires the URL bar to load and to show the page's address", () => {
    const noBar = parseLog(GOOD.replace("3 bar-loaded\n", "").replace("5 bar-url http://127.0.0.1:5000/smoke.html\n", ""));
    const failures = evaluate(noBar, { windowFullscreen: false, pageFullscreen: false }, NONE);
    expect(failures).toEqual([
      expect.stringContaining("bar never loaded"),
      expect.stringContaining("smoke page's address"),
    ]);
  });

  it("fails when webview helpers outlive the window", () => {
    const failures = evaluate(parseLog(GOOD), { windowFullscreen: false, pageFullscreen: false }, {
      helpersBefore: 0,
      helpersAfter: 3,
    });
    expect(failures).toEqual([expect.stringContaining("helper processes left")]);
  });
});
