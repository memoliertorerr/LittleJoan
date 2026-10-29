import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import {
  BAR_MARKS,
  CEILINGS,
  DEFAULT_CEILINGS,
  NATIVE_MARKS,
  PAGE_MARKS,
  READY_BAR_MARKS,
  READY_NATIVE_MARKS,
  ceilingsFor,
  focusedMs,
  median,
  parseTiming,
  readyMs,
  timingSummary,
  wantedMarks,
  seedHistory,
  summaryRow,
  summonTimings,
  timingFailures,
  timingTable,
  type Measured,
  type SummonTiming,
} from "./smoke-timing.mts";

const MARKS = {
  dispatched: 1,
  windowBuilt: 20,
  webviewsAdded: 120,
  shown: 140,
  created: 150,
  historyLoaded: 45,
  pageBuilt: 400,
  cookiesRestored: 401,
  agentSet: 405,
  barLoaded: 300,
  barScript: 310,
  barReady: 330,
  barPainted: 325,
  barFocused: 340,
  commandBar: 360,
};

/** A summon from ready (LJ19.4): no window, bar or early theme to build. */
const READY_MARKS = {
  dispatched: 1,
  shown: 12,
  created: 14,
  historyLoaded: 40,
  pageBuilt: 90,
  cookiesRestored: 90,
  agentSet: 91,
  barPainted: 30,
  barFocused: 25,
  commandBar: 70,
};

function line(over: Partial<SummonTiming> = {}): string {
  return JSON.stringify({ source: "cli", kind: "cold", mode: "hibernate", url: false, at: 1_000_500, partial: false, marks: MARKS, ...over });
}

function measured(over: Partial<Measured> = {}, timing: Partial<SummonTiming> = {}): Measured {
  return { run: "hibernate", kind: "warm", trigger: "cli", evicted: "none", gapMs: 3000, sentAt: 1_000_000, timing: parseTiming(line(timing)) ?? null, ...over };
}

function readyMeasured(over: Partial<Measured> = {}, timing: Partial<SummonTiming> = {}): Measured {
  return measured({ run: "ready", ...over }, { kind: "ready", mode: "ready", marks: READY_MARKS, ...timing });
}

describe("summon-timing lines (LJ19.1)", () => {
  it("parse the app's JSON, keeping only numeric marks", () => {
    const t = parseTiming(JSON.stringify({ source: "shortcut", kind: "ready", mode: "ready15", url: true, at: 5, nap: "active=1 lowpri=1 timer=5 disk=1", partial: true, marks: { shown: 40, bad: "x", pageStarted: 90 } }));
    expect(t).toEqual({ source: "shortcut", kind: "ready", mode: "ready15", url: true, at: 5, nap: "active=1 lowpri=1 timer=5 disk=1", partial: true, marks: { shown: 40, pageStarted: 90 } });
    // A line from before LJ19.4 is a cold summon.
    expect(parseTiming(JSON.stringify({ source: "cli", url: false, marks: {} }))).toMatchObject({ kind: "cold", mode: "hibernate" });
    expect(parseTiming(line())?.nap).toBeNull();
    for (const bad of ["", "{", "[]", JSON.stringify({ source: 1, url: false, marks: {} }), JSON.stringify({ source: "cli", url: false })]) {
      expect(parseTiming(bad)).toBeUndefined();
    }
  });

  it("are read from the log in order", () => {
    const events = parseLog(`1 window-created\n2 summon-timing ${line()}\n3 hibernated\n4 summon-timing ${line({ source: "shortcut" })}\n5 summon-timing nonsense\n`);
    expect(summonTimings(events).map((t) => t.source)).toEqual(["cli", "shortcut"]);
  });

  it("are ready at the command bar's rows, or with a URL at the later of the frame and the page", () => {
    const plain = parseTiming(line()) as SummonTiming;
    expect(readyMs(plain)).toBe(360);
    expect(focusedMs(plain)).toBe(340);
    const url = parseTiming(line({ url: true, marks: { ...MARKS, pageStarted: 400 } })) as SummonTiming;
    expect(readyMs(url)).toBe(400);
    const early = parseTiming(line({ url: true, marks: { ...MARKS, pageStarted: 200 } })) as SummonTiming;
    expect(readyMs(early)).toBe(325);
    const missing = parseTiming(line({ url: true })) as SummonTiming;
    expect(readyMs(missing)).toBeUndefined();
    expect(focusedMs(url)).toBeUndefined();
  });

  it("want the marks of their kind", () => {
    expect(wantedMarks({ kind: "cold", url: false }).native).toEqual(NATIVE_MARKS);
    expect(wantedMarks({ kind: "ready", url: false }).native).toEqual(READY_NATIVE_MARKS);
    expect(wantedMarks({ kind: "ready", url: false }).other).toEqual([...READY_BAR_MARKS, "commandBar", "barFocused"]);
    expect(wantedMarks({ kind: "cold", url: true }).other).toEqual([...BAR_MARKS, "pageStarted"]);
    expect(wantedMarks({ kind: "ready", url: true }).page).toEqual(PAGE_MARKS);
  });
});

describe("the timing run's rules", () => {
  it("pass complete lines under the ceilings", () => {
    const run = [measured({ kind: "first" }), measured(), measured({ kind: "url" }, { url: true, marks: { ...MARKS, pageStarted: 200 } })];
    expect(timingFailures(run, "linux")).toEqual([]);
    const ready = [readyMeasured({ kind: "first" }), readyMeasured(), readyMeasured({ kind: "url" }, { url: true, marks: { ...READY_MARKS, pageStarted: 120 } })];
    expect(timingFailures([...ready, ...run], "linux")).toEqual([]);
  });

  it("want a summon from ready in Stay ready's run, and a cold one in Hibernate's", () => {
    expect(timingFailures([readyMeasured({}, { kind: "cold", marks: MARKS })], "linux")).toEqual(["summon timing (ready warm): a cold summon, not ready"]);
    expect(timingFailures([measured({}, { kind: "ready", marks: MARKS })], "linux")[0]).toBe("summon timing (warm): a ready summon, not cold");
  });

  it("need the field to take the keyboard without a URL, and the page webview built", () => {
    const { barFocused: _focused, ...unfocused } = READY_MARKS;
    void _focused;
    expect(timingFailures([readyMeasured({}, { marks: unfocused })], "win32")).toEqual(["summon timing (ready warm): missing barFocused"]);
    const { pageBuilt: _built, ...noPage } = MARKS;
    void _built;
    expect(timingFailures([measured({}, { marks: noPage })], "win32")).toEqual(["summon timing (warm): missing pageBuilt"]);
    expect(timingFailures([measured({}, { marks: { ...MARKS, agentSet: 10 } })], "linux")[0]).toMatch(/page marks are out of order/);
  });

  it("hold a summon from ready to its own, tighter ceiling — the rows and the keyboard", () => {
    const slowRows = { ...READY_MARKS, commandBar: CEILINGS["linux"]!.ready + 1 };
    expect(timingFailures([readyMeasured({}, { marks: slowRows })], "linux")[0]).toMatch(/ready after \d+ ms, over the 1500 ms ceiling/);
    const slowKeys = { ...READY_MARKS, barFocused: CEILINGS["linux"]!.ready + 1 };
    expect(timingFailures([readyMeasured({}, { marks: slowKeys })], "linux")[0]).toMatch(/keyboard after \d+ ms, over the 1500 ms ceiling/);
  });

  it("need a warm summon, and a line for every CLI summon (a shortcut that didn't arrive is fine)", () => {
    expect(timingFailures([measured({ kind: "first" })], "linux")).toEqual(["summon timing: the hibernate run measured no warm summon"]);
    const failures = timingFailures([measured(), measured({ kind: "cold", timing: null }), measured({ kind: "shortcut", trigger: "shortcut", timing: null })], "darwin");
    expect(failures).toEqual(["summon timing (cold): no summon-timing line"]);
  });

  it("refuse partial lines and missing marks", () => {
    expect(timingFailures([measured({}, { partial: true })], "win32")[0]).toMatch(/partial/);
    const { barPainted: _painted, ...withoutPaint } = MARKS;
    void _painted;
    expect(timingFailures([measured({}, { marks: withoutPaint })], "win32")).toEqual(["summon timing (warm): missing barPainted"]);
    expect(timingFailures([measured({}, { url: true })], "win32")).toEqual(["summon timing (warm): missing pageStarted"]);
  });

  it("accept the bar's marks in any order — on Windows they all come before the webviews are added", () => {
    // CI's Windows run (LJ19.1): WebView2 loads, runs and paints the bar inside add_child.
    const windows = {
      dispatched: 0,
      windowBuilt: 3,
      historyLoaded: 51,
      barLoaded: 480,
      barScript: 495,
      barReady: 505,
      barPainted: 470,
      barFocused: 520,
      commandBar: 560,
      webviewsAdded: 695,
      shown: 700,
      created: 744,
      pageBuilt: 900,
      cookiesRestored: 901,
      agentSet: 942,
    };
    expect(timingFailures([measured({}, { marks: windows })], "win32")).toEqual([]);
    expect(readyMs(parseTiming(line({ marks: windows })) as SummonTiming)).toBe(560);
  });

  it("refuse native marks out of order (broken instrumentation)", () => {
    const failures = timingFailures([measured({}, { marks: { ...MARKS, shown: 10 } })], "linux");
    expect(failures[0]).toMatch(/out of order/);
    // History loads on its own thread: any time.
    expect(timingFailures([measured({}, { marks: { ...MARKS, historyLoaded: 900 } })], "linux")).toEqual([]);
  });

  it("refuse a summon over its ceiling, per kind and OS", () => {
    const slow = { ...MARKS, commandBar: CEILINGS["linux"]!.warm + 1, barFocused: 100 };
    expect(timingFailures([measured({}, { marks: slow })], "linux")[0]).toMatch(/over the 2000 ms ceiling/);
    // The same time is fine for a cold summon, and for a warm one on macOS.
    expect(timingFailures([measured(), measured({ kind: "cold" }, { marks: slow })], "linux")).toEqual([]);
    expect(timingFailures([measured({}, { marks: slow })], "darwin")).toEqual([]);
    // An OS without its own ceilings gets the loosest.
    expect(ceilingsFor("freebsd")).toEqual(DEFAULT_CEILINGS);
    for (const os of ["linux", "darwin", "win32"]) {
      const c = ceilingsFor(os);
      expect(c.ready).toBeLessThanOrEqual(c.warm);
      expect(c.warm).toBeLessThanOrEqual(c.first);
      expect(c.first).toBeLessThanOrEqual(c.cold);
    }
  });

  it("list every mark the app logs", () => {
    expect([...NATIVE_MARKS, ...PAGE_MARKS, ...BAR_MARKS, "commandBar", "barFocused"].sort()).toEqual(Object.keys(MARKS).sort());
    expect([...READY_NATIVE_MARKS, ...PAGE_MARKS, ...READY_BAR_MARKS, "commandBar", "barFocused"].sort()).toEqual(Object.keys(READY_MARKS).sort());
  });
});

describe("the timing run's summary", () => {
  it("has the trigger's latency and the ready time", () => {
    expect(summaryRow(measured())).toMatchObject({ run: "hibernate", kind: "warm", reached: true, sentToRequestMs: 500, readyMs: 360, focusedMs: 340, summon: "cold" });
    expect(summaryRow(readyMeasured())).toMatchObject({ run: "ready", readyMs: 70, focusedMs: 25, summon: "ready" });
    expect(summaryRow(measured({ kind: "shortcut", trigger: "shortcut", timing: null, sentAt: null }))).toMatchObject({ reached: false, sentToRequestMs: null, readyMs: null });
  });

  it("draws a Markdown table, with App Nap's state under it", () => {
    const table = timingTable(
      [
        measured(),
        measured({ kind: "cold", evicted: "purge" }, { nap: "active=1 lowpri=1 timer=5 disk=1" }),
        measured({ kind: "shortcut", trigger: "shortcut", timing: null }),
      ],
      "darwin",
    );
    expect(table).toContain("### Summon timing (darwin)");
    expect(table).toContain("| hibernate | warm | cli | 500 | 20 | 120 | 140 | 45 | – | 300 | – | 330 | 325 | 340 | – | 360 | 400 | 405 | – | 360 |");
    expect(table).toContain("| hibernate | cold (purge) |");
    expect(table).toContain("not reached");
    expect(table).toContain("- hibernate cold: active=1 lowpri=1 timer=5 disk=1");
    expect(timingTable([measured()], "linux")).not.toContain("App Nap");
  });

  it("sums up each mode on its own", () => {
    const summary = timingSummary([readyMeasured({ kind: "first" }), readyMeasured(), measured({ kind: "first" }), measured()], "linux");
    expect(summary.ready).toMatchObject({ firstMs: 70, warmMedianMs: 70, warmFocusedMedianMs: 25 });
    expect(summary.hibernate).toMatchObject({ firstMs: 360, warmMedianMs: 360, warmFocusedMedianMs: 340 });
    expect(summary.rules).toEqual([]);
  });

  it("takes the median (the lower middle for an even count)", () => {
    expect(median([300, 100, 200])).toBe(200);
    expect(median([4, 1, 3, 2])).toBe(2);
    expect(median([])).toBeUndefined();
  });
});

describe("the seeded history", () => {
  it("is a valid history.json of heavy use on hosts that never resolve", () => {
    const now = Date.UTC(2026, 9, 22);
    const file = JSON.parse(seedHistory(now, 600)) as { version: number; entries: { url: string; host: string; visits: { at: number; kind: string }[] }[] };
    expect(file.version).toBe(1);
    expect(file.entries).toHaveLength(600);
    expect(new Set(file.entries.map((e) => e.url)).size).toBe(600);
    expect(new Set(file.entries.map((e) => e.host)).size).toBe(300);
    for (const entry of file.entries) {
      expect(entry.host.endsWith(".lj-smoke.test")).toBe(true);
      expect(entry.visits).toHaveLength(10);
      const times = entry.visits.map((v) => v.at);
      expect([...times].sort((a, b) => a - b)).toEqual(times);
      expect(Math.min(...times)).toBeGreaterThan(now - 61 * 86_400_000);
    }
  });
});
