import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import {
  BAR_MARKS,
  CEILINGS,
  DEFAULT_CEILINGS,
  NATIVE_MARKS,
  ceilingsFor,
  median,
  parseTiming,
  readyMs,
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
  cookiesRestored: 141,
  agentSet: 145,
  created: 150,
  historyLoaded: 45,
  barLoaded: 300,
  barScript: 310,
  barReady: 330,
  barPainted: 325,
  commandBar: 360,
};

function line(over: Partial<SummonTiming> = {}): string {
  return JSON.stringify({ source: "cli", url: false, at: 1_000_500, partial: false, marks: MARKS, ...over });
}

function measured(over: Partial<Measured> = {}, timing: Partial<SummonTiming> = {}): Measured {
  return { kind: "warm", trigger: "cli", evicted: "none", gapMs: 3000, sentAt: 1_000_000, timing: parseTiming(line(timing)) ?? null, ...over };
}

describe("summon-timing lines (LJ19.1)", () => {
  it("parse the app's JSON, keeping only numeric marks", () => {
    const t = parseTiming(JSON.stringify({ source: "shortcut", url: true, at: 5, nap: "active=1 lowpri=1 timer=5 disk=1", partial: true, marks: { shown: 40, bad: "x", pageStarted: 90 } }));
    expect(t).toEqual({ source: "shortcut", url: true, at: 5, nap: "active=1 lowpri=1 timer=5 disk=1", partial: true, marks: { shown: 40, pageStarted: 90 } });
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
    const url = parseTiming(line({ url: true, marks: { ...MARKS, pageStarted: 400 } })) as SummonTiming;
    expect(readyMs(url)).toBe(400);
    const early = parseTiming(line({ url: true, marks: { ...MARKS, pageStarted: 200 } })) as SummonTiming;
    expect(readyMs(early)).toBe(325);
    const missing = parseTiming(line({ url: true })) as SummonTiming;
    expect(readyMs(missing)).toBeUndefined();
  });
});

describe("the timing run's rules", () => {
  it("pass complete lines under the ceilings", () => {
    const run = [measured({ kind: "first" }), measured(), measured({ kind: "url" }, { url: true, marks: { ...MARKS, pageStarted: 200 } })];
    expect(timingFailures(run, "linux")).toEqual([]);
  });

  it("need a warm summon, and a line for every CLI summon (a shortcut that didn't arrive is fine)", () => {
    expect(timingFailures([measured({ kind: "first" })], "linux")).toEqual(["summon timing: the run measured no warm summon"]);
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

  it("refuse native marks out of order (broken instrumentation)", () => {
    const failures = timingFailures([measured({}, { marks: { ...MARKS, shown: 10 } })], "linux");
    expect(failures[0]).toMatch(/out of order/);
    // History loads on its own thread: any time.
    expect(timingFailures([measured({}, { marks: { ...MARKS, historyLoaded: 900 } })], "linux")).toEqual([]);
  });

  it("refuse a summon over its ceiling, per kind and OS", () => {
    const slow = { ...MARKS, commandBar: CEILINGS["linux"]!.warm + 1 };
    expect(timingFailures([measured({}, { marks: slow })], "linux")[0]).toMatch(/over the 2000 ms ceiling/);
    // The same time is fine for a cold summon, and for a warm one on macOS.
    expect(timingFailures([measured(), measured({ kind: "cold" }, { marks: slow })], "linux")).toEqual([]);
    expect(timingFailures([measured({}, { marks: slow })], "darwin")).toEqual([]);
    // An OS without its own ceilings gets the loosest.
    expect(ceilingsFor("freebsd")).toEqual(DEFAULT_CEILINGS);
    for (const os of ["linux", "darwin", "win32"]) {
      const c = ceilingsFor(os);
      expect(c.warm).toBeLessThanOrEqual(c.first);
      expect(c.first).toBeLessThanOrEqual(c.cold);
    }
  });

  it("list every mark the app logs", () => {
    expect([...NATIVE_MARKS, ...BAR_MARKS, "commandBar"].sort()).toEqual(Object.keys(MARKS).sort());
  });
});

describe("the timing run's summary", () => {
  it("has the trigger's latency and the ready time", () => {
    expect(summaryRow(measured())).toMatchObject({ kind: "warm", reached: true, sentToRequestMs: 500, readyMs: 360 });
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
    expect(table).toContain("| warm | cli | 500 | 20 | 120 | 140 | 145 | 45 | – | 300 | – | 330 | 325 | – | 360 | – | 360 |");
    expect(table).toContain("| cold (purge) |");
    expect(table).toContain("not reached");
    expect(table).toContain("- cold: active=1 lowpri=1 timer=5 disk=1");
    expect(timingTable([measured()], "linux")).not.toContain("App Nap");
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
