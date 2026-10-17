import { describe, expect, it } from "vitest";
import { commandBarFailures, parseSuggestions, sawSmokeTopSite, sawTypedMatch } from "./smoke-command.mts";
import { parseLog } from "./smoke-log.mts";

const GOOD = `1 window-created
2 suggestions 4 first=wikipedia.org typed=0
3 page-loaded http://127.0.0.1:5000/smoke.html
4 hibernated
5 window-created
6 suggestions 1 first=127.0.0.1:5000 typed=0
7 suggestions 2 first=127.0.0.1:5000 typed=3
8 hibernated
`;

describe("suggestions events", () => {
  it("parse counts, the first site and the typed length", () => {
    const [event] = parseLog("1 suggestions 3 first=127.0.0.1:5000 typed=2\n");
    expect(event && parseSuggestions(event)).toEqual({ rows: 3, first: "127.0.0.1:5000", typed: 2 });
    const [empty] = parseLog("1 suggestions 0 first=- typed=0\n");
    expect(empty && parseSuggestions(empty)).toEqual({ rows: 0, first: "-", typed: 0 });
    const [other] = parseLog("1 title suggestions 3 first=x typed=1\n");
    expect(other && parseSuggestions(other)).toBeUndefined();
    const [bad] = parseLog("1 suggestions three\n");
    expect(bad && parseSuggestions(bad)).toBeUndefined();
  });

  it("find the smoke site as a top site and as a typed match", () => {
    const events = parseLog(GOOD);
    expect(sawSmokeTopSite(events)).toBe(true);
    expect(sawTypedMatch(events)).toBe(true);
    expect(sawSmokeTopSite(parseLog("1 suggestions 1 first=127.0.0.1:5000 typed=3\n"))).toBe(false);
    expect(sawTypedMatch(parseLog("1 suggestions 1 first=google.com typed=3\n"))).toBe(false);
  });
});

describe("the command bar check", () => {
  it("passes a run with starters first, then the smoke site, then the typed match", () => {
    expect(commandBarFailures(parseLog(GOOD), { typing: true })).toEqual([]);
  });

  it("wants the starters while there is no history", () => {
    const log = parseLog(GOOD.replace("2 suggestions 4 first=wikipedia.org typed=0", "2 suggestions 1 first=example.com typed=0"));
    expect(commandBarFailures(log, { typing: true })).toEqual([expect.stringContaining("not the starters (wikipedia.org)")]);
    const silent = parseLog(GOOD.replace("2 suggestions 4 first=wikipedia.org typed=0\n", ""));
    expect(commandBarFailures(silent, { typing: true })).toEqual([expect.stringContaining("never asked for suggestions")]);
  });

  it("wants the smoke site in a new window after the visit", () => {
    const sameWindow = parseLog(GOOD.replace("5 window-created\n", ""));
    expect(commandBarFailures(sameWindow, { typing: false })).toEqual([expect.stringContaining("as a top site")]);
    const never = parseLog(GOOD.replace("6 suggestions 1 first=127.0.0.1:5000 typed=0", "6 suggestions 4 first=wikipedia.org typed=0"));
    expect(commandBarFailures(never, { typing: false })).toEqual([expect.stringContaining("as a top site")]);
  });

  it("wants the typed match only where typing is driven", () => {
    const untyped = parseLog(GOOD.replace("7 suggestions 2 first=127.0.0.1:5000 typed=3\n", ""));
    expect(commandBarFailures(untyped, { typing: false })).toEqual([]);
    expect(commandBarFailures(untyped, { typing: true })).toEqual([expect.stringContaining('typing "127"')]);
  });
});
