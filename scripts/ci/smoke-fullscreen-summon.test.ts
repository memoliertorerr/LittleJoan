import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { enteredFullscreen, fsSummonFailures, fsSummonSummary, summonDuringFullscreenFailures, type FsSummon } from "./smoke-fullscreen-summon.mts";

describe("summonDuringFullscreenFailures", () => {
  const during = (lines: string[], os = "darwin") =>
    summonDuringFullscreenFailures(parseLog([...(os === "darwin" ? ["wk-fullscreen entering", "wk-fullscreen in"] : ["element-fullscreen true"]), ...lines].map((line, i) => `${i + 1} ${line}`).join("\n") + "\n"), os);
  it("pass when the summon with a URL left the page's fullscreen first (LJ19.5), on every OS", () => {
    expect(during(["summon-fullscreen leaving", "wk-fullscreen exiting", "wk-fullscreen not", "summon-fullscreen left 820", "page-started x"])).toEqual([]);
    expect(during(["summon-fullscreen leaving", "element-fullscreen false", "summon-fullscreen left 60"], "win32")).toEqual([]);
    expect(during(["summon-fullscreen leaving", "element-fullscreen false", "summon-fullscreen left 75"], "linux")).toEqual([]);
  });
  it("fail when it only activated the app (LJ19.4-fix4) or gave up", () => {
    expect(during(["summon-fullscreen activated", "wk-fullscreen exiting", "wk-fullscreen not"])).toEqual([
      "fullscreen: the summon with a URL during the page's fullscreen didn't leave it first (summon-fullscreen activated)",
    ]);
    expect(during(["summon-fullscreen leaving", "summon-fullscreen gave-up 4000"], "linux")[0]).toContain("summon-fullscreen leaving, gave-up 4000");
    expect(during([], "win32")[0]).toContain("no summon-fullscreen line");
  });
  it("ask nothing when the page never entered the fullscreen", () => {
    expect(summonDuringFullscreenFailures(parseLog("1 wk-fullscreen not\n"), "darwin")).toEqual([]);
    expect(summonDuringFullscreenFailures(parseLog("1 fullscreen-off 2.50.4\n"), "linux")).toEqual([]);
  });
});

describe("the summon during the page's fullscreen (LJ19.5)", () => {
  /** A summon: lines before the first press, after it, and after the second (millis 1000 + index). */
  function summon(os: string, over: { before?: string[]; first?: string[]; second?: string[]; pings?: number[]; entered?: boolean } = {}): FsSummon {
    const mac = os === "darwin";
    const before = over.before ?? (mac ? ["wk-fullscreen entering", "wk-fullscreen in"] : ["element-fullscreen true", "fullscreen true"]);
    const first =
      over.first ??
      (mac
        ? ["summon-fullscreen leaving", "wk-fullscreen exiting", "wk-fullscreen not", "summon-fullscreen left 640", "overlay true", "page-visible false", "suggestions 4 typed=1"]
        : ["summon-fullscreen leaving", "element-fullscreen false", "fullscreen false", "summon-fullscreen left 120", "overlay true", "suggestions 4 typed=1"]);
    const second = over.second ?? ["closing ready", "hibernated ready"];
    const events = parseLog([...before, ...first, ...second].map((line, i) => `${1000 + i} ${line}`).join("\n") + "\n");
    return {
      entered: over.entered ?? true,
      events,
      first: before.length,
      second: before.length + first.length,
      firstAt: 10_000,
      secondAt: 20_000,
      pings: over.pings ?? [9_800, 10_500, 11_250, 12_000, 19_900],
    };
  }

  it("sees the page enter element fullscreen per engine", () => {
    expect(enteredFullscreen(parseLog("1 wk-fullscreen in\n"), "darwin")).toBe(true);
    expect(enteredFullscreen(parseLog("1 element-fullscreen true\n"), "darwin")).toBe(false);
    expect(enteredFullscreen(parseLog("1 element-fullscreen true\n"), "win32")).toBe(true);
    expect(enteredFullscreen(parseLog("1 element-fullscreen false\n"), "linux")).toBe(false);
  });

  it("pass when the press left the fullscreen, the command bar opened and the second press closed", () => {
    for (const os of ["darwin", "win32", "linux"]) {
      expect(fsSummonFailures(summon(os), os)).toEqual([]);
    }
    expect(fsSummonFailures(undefined, "linux")).toEqual([]);
  });

  it("fail when the first press closed the window (what Windows and Linux did before LJ19.5)", () => {
    const failures = fsSummonFailures(summon("win32", { first: ["closing ready", "element-fullscreen false", "hibernated ready"], second: ["window-created ready", "overlay true"], pings: [9_900] }), "win32");
    expect(failures.join("\n")).toContain("the first press closed the window");
    expect(failures.join("\n")).toContain("no summon-fullscreen left line");
    expect(failures.join("\n")).toContain("the command bar didn't open");
    expect(failures.join("\n")).toContain("stopped pinging");
    expect(failures.join("\n")).toContain("didn't reach the command bar's field");
    expect(failures.join("\n")).toContain("the second press didn't close the window");
  });

  it("fail when macOS only activated the app (LJ19.4-fix4's way, before LJ19.5)", () => {
    const failures = fsSummonFailures(summon("darwin", { first: ["summon-fullscreen activated"], second: ["summon-fullscreen activated"] }), "darwin");
    expect(failures).toEqual([
      "fullscreen summon: the app didn't leave the page's fullscreen (no summon-fullscreen left line)",
      "fullscreen summon: the command bar didn't open after the fullscreen ended (no overlay true)",
      "fullscreen summon: the second press didn't close the window",
    ]);
  });

  it("fail when the app gave up, or said left before WebKit gave the page back", () => {
    expect(fsSummonFailures(summon("linux", { first: ["summon-fullscreen leaving", "summon-fullscreen gave-up 3000", "suggestions 4 typed=1"] }), "linux").join()).toContain("summon-fullscreen gave-up 3000");
    const early = summon("darwin", { first: ["summon-fullscreen leaving", "summon-fullscreen left 10", "wk-fullscreen not", "overlay true"] });
    expect(fsSummonFailures(early, "darwin")).toEqual(["fullscreen summon: summon-fullscreen left came before WebKit gave the page back (wk-fullscreen not)"]);
    const never = summon("win32", { first: ["summon-fullscreen left 10", "overlay true", "suggestions 4 typed=1"] });
    expect(fsSummonFailures(never, "win32")).toEqual(["fullscreen summon: summon-fullscreen left came without the page's fullscreen ending"]);
  });

  it("fail when the command bar opened before the fullscreen ended", () => {
    const failures = fsSummonFailures(summon("linux", { first: ["overlay true", "summon-fullscreen leaving", "element-fullscreen false", "summon-fullscreen left 90", "suggestions 4 typed=1"] }), "linux");
    expect(failures).toEqual(["fullscreen summon: the command bar didn't open after the fullscreen ended (no overlay true)"]);
  });

  it("fail when the page was reloaded or the letter went elsewhere; macOS records the letter only", () => {
    const reloaded = summon("win32", { first: ["summon-fullscreen leaving", "element-fullscreen false", "summon-fullscreen left 80", "overlay true", "page-started http://127.0.0.1/smoke.html"] });
    expect(fsSummonFailures(reloaded, "win32")).toEqual([
      "fullscreen summon: the page was loaded again (page-started after the press)",
      "fullscreen summon: the letter typed after the press didn't reach the command bar's field",
    ]);
    const untyped = summon("darwin", { first: ["summon-fullscreen leaving", "wk-fullscreen not", "summon-fullscreen left 400", "overlay true"] });
    expect(fsSummonFailures(untyped, "darwin")).toEqual([]);
  });

  it("ask nothing on macOS when the click didn't reach the page; elsewhere it must have", () => {
    expect(fsSummonFailures(summon("darwin", { entered: false }), "darwin")).toEqual([]);
    expect(fsSummonFailures(summon("linux", { entered: false }), "linux")).toEqual(["fullscreen summon: the page never entered element fullscreen after its click"]);
  });

  it("summarizes both presses and the pings", () => {
    const summary = fsSummonSummary(summon("win32"), "win32");
    expect(summary?.firstPress).toEqual(["2 summon-fullscreen leaving", "3 element-fullscreen false", "4 fullscreen false", "5 summon-fullscreen left 120", "6 overlay true", "7 suggestions 4 typed=1"]);
    expect(summary?.secondPress).toEqual(["8 closing ready", "9 hibernated ready"]);
    expect(summary?.pingsAfterFirstPress).toBe(4);
    expect(summary?.pingsAfterSecondPress).toBe(0);
    expect(summary?.rules).toEqual([]);
    expect(fsSummonSummary(undefined, "win32")).toBeNull();
  });
});
