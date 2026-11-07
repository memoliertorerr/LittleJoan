import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { pinnedAddresses, pinnedFailures, pinnedSummary, pinnedUrls, type PinnedRun } from "./smoke-pinned.mts";

const log = (rows: string[]) => parseLog(rows.map((row, i) => `${1000 + i} ${row}`).join("\n") + "\n");
const PAGE = "http://127.0.0.1:5000/smoke.html";
const { pinned, other } = pinnedUrls(PAGE);

function run(over: Partial<PinnedRun> = {}): PinnedRun {
  return {
    events: log(["started", "suggestions 1 first=127.0.0.1:5000 typed=0", "overlay true", "suggestions 2 first=google.com typed=3", "quick-command setPinned true"]),
    pinned,
    other,
    typed: true,
    settings: { pinnedSites: [{ url: pinned, title: "LJ pinned" }, { url: other, title: "LJ-SMOKE ready" }] },
    ...over,
  };
}

describe("the pinned run's rules (LJ50)", () => {
  it("passes when the first window leads with the pinned site and pin adds the other page", () => {
    expect(pinnedFailures(run(), "win32")).toEqual([]);
    expect(pinnedFailures(run(), "linux")).toEqual([]);
    expect(pinnedFailures(run(), "darwin")).toEqual([]);
  });

  it("wants the very first suggestions to lead with the pinned site", () => {
    const starters = run({ events: log(["started", "suggestions 4 first=wikipedia.org typed=0", "quick-command setPinned true"]) });
    expect(pinnedFailures(starters, "linux").join()).toContain("not the pinned 127.0.0.1:5000");
    // A later line doesn't make up for it.
    const late = run({ events: log(["suggestions 4 first=wikipedia.org typed=0", "suggestions 1 first=127.0.0.1:5000 typed=0", "quick-command setPinned true"]) });
    expect(pinnedFailures(late, "win32").join()).toContain("led with");
    expect(pinnedFailures(run({ events: log(["started"]) }), "darwin").join()).toContain("never asked for suggestions");
  });

  it("wants pin to run where the typing is driven", () => {
    expect(pinnedFailures(run({ typed: false }), "win32").join()).toContain('never showed the typed "pin"');
    expect(pinnedFailures(run({ typed: false, settings: null }), "darwin")).toEqual([]);
    const notRun = run({ events: log(["suggestions 1 first=127.0.0.1:5000 typed=0", "quick-command setMobileSites false"]) });
    expect(pinnedFailures(notRun, "linux").join()).toContain('not "quick-command setPinned true"');
  });

  it("wants settings.json to pin the first page and then the other", () => {
    const unsaved = run({ settings: { pinnedSites: [{ url: pinned, title: "LJ pinned" }] } });
    expect(pinnedFailures(unsaved, "win32").join()).toContain("settings.json pins");
    const swapped = run({ settings: { pinnedSites: [{ url: other }, { url: pinned }] } });
    expect(pinnedFailures(swapped, "linux").join()).toContain("settings.json pins");
  });

  it("reads settings.json's addresses tolerantly", () => {
    expect(pinnedAddresses({ pinnedSites: [{ url: "a" }, { title: "no url" }, "b", { url: 3 }] })).toEqual(["a"]);
    expect(pinnedAddresses({ pinnedSites: "no" })).toEqual([]);
    expect(pinnedAddresses(null)).toEqual([]);
  });

  it("summarises the run", () => {
    const summary = pinnedSummary(run(), "win32");
    expect(summary.firstSuggestions).toBe("1 first=127.0.0.1:5000 typed=0");
    expect(summary.pinnedSites).toEqual([pinned, other]);
    expect(summary.rules).toEqual([]);
  });
});
