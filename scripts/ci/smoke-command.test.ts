import { describe, expect, it } from "vitest";
import {
  commandBarFailures,
  historyViewFailures,
  parseHistoryList,
  parseSuggestions,
  sawHistoryView,
  sawSettings,
  sawSmokeTopSite,
  sawTypedMatch,
  settingsFailures,
  settingsTabs,
} from "./smoke-command.mts";
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

describe("the history view check", () => {
  const OPENED = `1 window-created
2 suggestions 2 first=127.0.0.1:5000 typed=3
3 history-view
4 history-list 3
5 hibernated
`;

  it("parses the listed count", () => {
    const [event] = parseLog("1 history-list 12\n");
    expect(event && parseHistoryList(event)).toBe(12);
    const [bad] = parseLog("1 history-list many\n");
    expect(bad && parseHistoryList(bad)).toBeUndefined();
    const [other] = parseLog("1 title history-list 3\n");
    expect(other && parseHistoryList(other)).toBeUndefined();
  });

  it("wants Ctrl/Cmd+Y to open the view and the view to list pages", () => {
    expect(sawHistoryView(parseLog(OPENED))).toBe(true);
    expect(historyViewFailures(parseLog(OPENED), { driven: true })).toEqual([]);
    const empty = parseLog(OPENED.replace("4 history-list 3", "4 history-list 0"));
    expect(sawHistoryView(empty)).toBe(false);
    expect(historyViewFailures(empty, { driven: true })).toEqual([expect.stringContaining("Ctrl/Cmd+Y")]);
    // A list from the command bar's History row alone isn't the key.
    const rowOnly = parseLog(OPENED.replace("3 history-view\n", ""));
    expect(sawHistoryView(rowOnly)).toBe(false);
    const before = parseLog("1 history-list 3\n2 history-view\n");
    expect(sawHistoryView(before)).toBe(false);
  });

  it("is best effort where the keys aren't driven", () => {
    expect(historyViewFailures(parseLog("1 window-created\n"), { driven: false })).toEqual([]);
    expect(historyViewFailures(parseLog("1 window-created\n"), { driven: true })).toHaveLength(1);
  });
});

describe("the settings check", () => {
  const GOOD_SETTINGS = `1 window-created
2 page-loaded http://127.0.0.1:5000/smoke.html
3 hibernated
4 settings-window general
5 settings-loaded
6 settings-window privacy
7 settings-window appearance
8 quit
`;

  it("lists the tabs and sees the page load after General", () => {
    const events = parseLog(GOOD_SETTINGS);
    expect(settingsTabs(events)).toEqual(["general", "privacy", "appearance"]);
    expect(sawSettings(events, "general")).toBe(true);
    expect(sawSettings(parseLog("1 settings-loaded\n2 settings-window general\n"), "general")).toBe(false);
  });

  it("passes General, then Privacy, then Appearance, with no page window", () => {
    expect(settingsFailures(parseLog(GOOD_SETTINGS))).toEqual([]);
  });

  it("wants the page to load, Privacy after General, Appearance after Privacy, and no page window", () => {
    const unloaded = parseLog(GOOD_SETTINGS.replace("5 settings-loaded\n", ""));
    expect(settingsFailures(unloaded)).toEqual([expect.stringContaining("on General")]);
    const noPrivacy = parseLog(GOOD_SETTINGS.replace("6 settings-window privacy\n", ""));
    expect(settingsFailures(noPrivacy)).toEqual([expect.stringContaining("to Privacy")]);
    const noAppearance = parseLog(GOOD_SETTINGS.replace("7 settings-window appearance\n", ""));
    expect(settingsFailures(noAppearance)).toEqual([expect.stringContaining("to Appearance")]);
    const opened = parseLog(GOOD_SETTINGS.replace("8 quit", "8 window-created"));
    expect(settingsFailures(opened)).toEqual([expect.stringContaining("page window")]);
    expect(settingsFailures(parseLog("1 started\n"))).toHaveLength(3);
  });
});
