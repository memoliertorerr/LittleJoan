import { describe, expect, it } from "vitest";
import { parseLog, type LogEvent } from "./smoke-log.mts";
import { PINGS_STOP_MS, TYPED_KEYS, restFailures, restTable, sample, typedCounts, type RestRun } from "./smoke-rest.mts";

const timing = (kind: string) =>
  `summon-timing {"source":"cli","kind":"${kind}","mode":"ready","url":false,"at":1,"nap":null,"partial":false,"marks":{"dispatched":0,"commandBar":90}}`;

function events(text: string): LogEvent[] {
  return parseLog(text);
}

/** A ready run that passes every rule; each test breaks one thing. */
function readyRun(over: Partial<RestRun> = {}): RestRun {
  return {
    mode: "ready",
    events: events(
      [
        "1000 started",
        "4000 prewarmed ready",
        "9000 window-created ready",
        "12000 closing ready",
        "12400 hibernated ready",
        "20000 window-created ready",
        `20100 suggestions 3 first=- typed=${TYPED_KEYS.length}`,
        "30000 bar-gone RenderProcessExited",
        "31000 released bar-gone",
        "31500 window-created cold",
        `32000 ${timing("cold")}`,
      ].join("\n"),
    ),
    samples: [
      { at: "start", web: 1, engine: 3, memoryMb: 90, ownMb: 40 },
      { at: "page", web: 2, engine: 4, memoryMb: 150, ownMb: 70 },
      { at: "after-close", web: 1, engine: 3, memoryMb: 95, ownMb: 42 },
    ],
    close: { closingAt: 12000, lastPing: 12100, pingsBefore: 12, pingsAfter: 0, pageWeb: [77], left: { "2s": 1, "10s": 0, "30s": 0 } },
    typed: { sent: true, counts: [1, 3, TYPED_KEYS.length] },
    killed: { pids: 1, after: events(["31000 released bar-gone", "31500 window-created cold", `32000 ${timing("cold")}`].join("\n")) },
    ...over,
  };
}

describe("the rest runs' measurements (LJ19.4)", () => {
  it("sum the app's and its engine's memory, and count web processes", () => {
    const engine = [
      { pid: 1, name: "WebKitWebProcess", web: true, rssKb: 30 * 1024, ownKb: 12 * 1024 },
      { pid: 2, name: "WebKitNetworkProcess", web: false, rssKb: 10 * 1024 + 512, ownKb: 3 * 1024 },
    ];
    expect(sample("start", 40 * 1024, engine, 5 * 1024)).toEqual({ at: "start", web: 1, engine: 2, memoryMb: 80.5, ownMb: 20 });
    // A part that couldn't be read leaves the own figure out rather than short.
    expect(sample("start", 40 * 1024, engine).ownMb).toBeNull();
  });

  it("read the typed counts of the command bar's suggestions", () => {
    expect(typedCounts(events("1 suggestions 8 first=a typed=0\n2 suggestions 5 first=b typed=6\n3 other typed=9"))).toEqual([0, 6]);
  });
});

describe("the rest runs' rules", () => {
  it("pass a ready run that did everything", () => {
    expect(restFailures(readyRun(), "linux")).toEqual([]);
  });

  it("refuse pings after the close, and the page's web processes left behind", () => {
    const late = readyRun({ close: { ...readyRun().close!, pingsAfter: 3, lastPing: 12000 + PINGS_STOP_MS + 700 } });
    expect(restFailures(late, "linux")[0]).toMatch(/kept running after the close — 3 pings/);
    const left = readyRun({ close: { ...readyRun().close!, left: { "2s": 1, "10s": 1, "30s": 1 } } });
    expect(restFailures(left, "darwin")[0]).toMatch(/1 of the page's 1 web processes were still there 30 s after/);
  });

  it("allow only the bar's web processes at rest after a close", () => {
    const more = readyRun({ samples: [...readyRun().samples.slice(0, 2), { at: "after-close", web: 3, engine: 5, memoryMb: 120, ownMb: 60 }] });
    expect(restFailures(more, "win32")).toEqual(["rest ready: 3 web processes at rest after the close, more than the bar's 1"]);
  });

  it("refuse lost keys on Windows and Linux, and take macOS's as information", () => {
    const lost = readyRun({ typed: { sent: true, counts: [3, 5] } });
    expect(restFailures(lost, "linux")[0]).toMatch(/keys typed 150 ms after the shortcut were lost — the field saw typed=3,5/);
    expect(restFailures(lost, "darwin")).toEqual([]);
  });

  it("need the gone bar rebuilt by the next summon", () => {
    const notRebuilt = readyRun({ killed: { pids: 1, after: events("31500 window-created ready") } });
    expect(restFailures(notRebuilt, "linux")).toEqual([
      "rest ready: the summon after the kill didn't release the gone bar's window",
      'rest ready: the summon after the kill wasn\'t built anew (no "window-created cold")',
      "rest ready: the summon after the kill never drew the command bar's rows (no complete summon-timing)",
    ]);
    const unnoticed = readyRun({ events: readyRun().events.filter((e) => e.event !== "bar-gone") });
    expect(restFailures(unnoticed, "win32")).toEqual(['rest ready: the engine never said the bar\'s web process ended ("bar-gone")']);
  });

  it("allow nothing of ours at rest in Hibernate, and no prewarm", () => {
    const hibernate: RestRun = {
      mode: "hibernate",
      events: events("1000 started\n9000 window-created cold\n12000 closing hibernate\n12500 hibernated"),
      samples: [
        { at: "start", web: 0, engine: 0, memoryMb: 30, ownMb: 10 },
        // The engine's network process may stay (WebKitGTK's web context, WebKit's services).
        { at: "after-close", web: 0, engine: 1, memoryMb: 34, ownMb: 12 },
      ],
      close: { closingAt: 12000, lastPing: 12050, pingsBefore: 8, pingsAfter: 0, pageWeb: [5, 6], left: { "2s": 2, "10s": 0, "30s": 0 } },
      typed: null,
      killed: null,
    };
    expect(restFailures(hibernate, "linux")).toEqual([]);
    const busy = {
      ...hibernate,
      events: events("1000 started\n4000 prewarmed hibernate"),
      samples: [{ at: "after-close", web: 1, engine: 2, memoryMb: 80, ownMb: 30 }],
    };
    expect(restFailures(busy, "linux")).toEqual([
      "rest hibernate: it built the ready window although the setting says Hibernate",
      "rest hibernate: 1 web processes of ours at rest after the close (none allowed)",
    ]);
  });

  it("want Stay ready for 15 minutes to go by itself twice, the summon between cold", () => {
    const ready15: RestRun = {
      mode: "ready15",
      events: events(
        "1000 started\n4000 prewarmed ready15\n10000 released timer\n14000 window-created cold\n16000 closing ready15\n16100 hibernated ready15\n22100 released timer",
      ),
      samples: [{ at: "start", web: 1, engine: 3, memoryMb: 90, ownMb: 40 }],
      close: { closingAt: 16000, lastPing: 16010, pingsBefore: 6, pingsAfter: 0, pageWeb: [], left: { "30s": 0 } },
      typed: null,
      killed: null,
    };
    expect(restFailures(ready15, "darwin")).toEqual([]);
    const once = { ...ready15, events: ready15.events.filter((e) => e.millis !== 22100) };
    expect(restFailures(once, "darwin")).toEqual(["rest ready15: the ready window went by itself 1 times, not twice (after the launch, after the close)"]);
  });

  it("draw a table per mode", () => {
    const table = restTable([readyRun()], "linux");
    expect(table).toContain('### At rest per "After closing" mode (linux)');
    expect(table).toContain("| ready | after-close | 1 | 3 | 95 | 42 |");
    expect(table).toContain("- ready: 12 pings before the close, 0 more than 2000 ms after");
  });
});
