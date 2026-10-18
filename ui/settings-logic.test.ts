import { describe, expect, it } from "vitest";
import { DEFAULT_THEME } from "./lib/theme.ts";
import {
  CLEAR_START,
  TABS,
  changes,
  clearQuestion,
  clearStep,
  hasChanges,
  isChanged,
  tabForKey,
  tabFrom,
  toInput,
  type Settings,
} from "./settings-logic.ts";

const saved: Settings = {
  version: 1,
  shortcut: "CommandOrControl+Shift+Space",
  searchEngine: "google",
  translateTo: "en",
  windowWidth: 480,
  windowHeight: 640,
  mobileSites: true,
  rememberHistory: true,
  theme: DEFAULT_THEME,
};

const same = () => toInput(saved.shortcut, saved.searchEngine, saved.translateTo, saved.mobileSites, saved.rememberHistory);

describe("settings logic", () => {
  it("sends only the edited fields", () => {
    expect(toInput("Alt+KeyJ", "brave", "tr", false, false)).toEqual({
      shortcut: "Alt+KeyJ",
      searchEngine: "brave",
      translateTo: "tr",
      mobileSites: false,
      rememberHistory: false,
    });
  });

  it("detects changes field by field", () => {
    expect(isChanged(saved, same())).toBe(false);
    expect(isChanged(saved, toInput("Alt+KeyJ", "google", "en", true, true))).toBe(true);
    expect(isChanged(saved, toInput(saved.shortcut, "bing", "en", true, true))).toBe(true);
    expect(isChanged(saved, toInput(saved.shortcut, "google", "de", true, true))).toBe(true);
    expect(isChanged(saved, toInput(saved.shortcut, "google", "en", false, true))).toBe(true);
    expect(isChanged(saved, toInput(saved.shortcut, "google", "en", true, false))).toBe(true);
  });

  it("includes the start-at-login switch, applied on its own", () => {
    expect(changes(saved, false, same(), false)).toEqual({ settings: false, startAtLogin: false });
    expect(hasChanges(changes(saved, false, same(), false))).toBe(false);
    expect(changes(saved, false, same(), true)).toEqual({ settings: false, startAtLogin: true });
    expect(changes(saved, true, same(), false)).toEqual({ settings: false, startAtLogin: true });
    expect(hasChanges(changes(saved, true, same(), false))).toBe(true);
    const other = toInput("Alt+KeyJ", saved.searchEngine, saved.translateTo, saved.mobileSites, saved.rememberHistory);
    expect(changes(saved, true, other, false)).toEqual({ settings: true, startAtLogin: true });
    expect(changes(saved, true, other, true)).toEqual({ settings: true, startAtLogin: false });
  });

  it("never changes a login entry that couldn't be read", () => {
    expect(changes(saved, null, same(), true)).toEqual({ settings: false, startAtLogin: false });
    expect(hasChanges(changes(saved, null, same(), false))).toBe(false);
  });

  it("sees changes on every tab at once, and an undone change as none", () => {
    // General (the engine) and Privacy (remember history) edited: one Save covers both.
    const both = toInput(saved.shortcut, "duckduckgo", saved.translateTo, saved.mobileSites, false);
    expect(changes(saved, false, both, false)).toEqual({ settings: true, startAtLogin: false });
    // Privacy alone.
    const privacy = toInput(saved.shortcut, saved.searchEngine, saved.translateTo, saved.mobileSites, false);
    expect(hasChanges(changes(saved, false, privacy, false))).toBe(true);
    // Switched back on: nothing to save.
    expect(hasChanges(changes(saved, false, same(), false))).toBe(false);
  });
});

describe("tabs", () => {
  it("are General, Appearance, Extensions, Privacy and About", () => {
    expect(TABS.map((tab) => tab.label)).toEqual(["General", "Appearance", "Extensions", "Privacy", "About"]);
  });

  it("open by name, General for anything else", () => {
    expect(tabFrom("privacy")).toBe("privacy");
    expect(tabFrom(" Extensions ")).toBe("extensions");
    expect(tabFrom("about")).toBe("about");
    expect(tabFrom("advanced")).toBe("general");
    expect(tabFrom(undefined)).toBe("general");
    expect(tabFrom(42)).toBe("general");
  });

  it("move with ↑/↓, Home and End, without wrapping", () => {
    expect(tabForKey("general", "ArrowDown")).toBe("appearance");
    expect(tabForKey("about", "ArrowDown")).toBe("about");
    expect(tabForKey("privacy", "ArrowUp")).toBe("extensions");
    expect(tabForKey("general", "ArrowUp")).toBe("general");
    expect(tabForKey("privacy", "Home")).toBe("general");
    expect(tabForKey("general", "End")).toBe("about");
    expect(tabForKey("general", "Enter")).toBeNull();
  });
});

describe("the Privacy tab's clear action", () => {
  it("starts on the last hour, asks before clearing, and says what went", () => {
    expect(CLEAR_START).toEqual({ range: "hour", asking: false, busy: false, message: "" });
    let state = clearStep(CLEAR_START, { kind: "pick", range: "week" });
    expect(state.range).toBe("week");
    // Confirming without the question does nothing.
    expect(clearStep(state, { kind: "confirm" })).toEqual(state);
    state = clearStep(state, { kind: "ask" });
    expect(state.asking).toBe(true);
    expect(clearQuestion(state.range)).toBe("Remove the history of the last 7 days?");
    state = clearStep(state, { kind: "confirm" });
    expect(state.busy).toBe(true);
    // While Rust works, nothing else changes it.
    expect(clearStep(state, { kind: "pick", range: "all" })).toEqual(state);
    expect(clearStep(state, { kind: "confirm" })).toEqual(state);
    state = clearStep(state, { kind: "done", removed: 4 });
    expect(state).toEqual({ range: "week", asking: false, busy: false, message: "Removed 4 pages from the last 7 days." });
  });

  it("can be cancelled, and reports a failure", () => {
    const asking = clearStep(clearStep(CLEAR_START, { kind: "pick", range: "all" }), { kind: "ask" });
    expect(clearQuestion("all")).toBe("Remove all of your history?");
    expect(clearStep(asking, { kind: "cancel" })).toEqual({ range: "all", asking: false, busy: false, message: "" });
    const failed = clearStep(clearStep(asking, { kind: "confirm" }), { kind: "failed", error: "Couldn't save the history: disk full" });
    expect(failed).toEqual({ range: "all", asking: false, busy: false, message: "Couldn't save the history: disk full" });
    expect(clearStep(clearStep(asking, { kind: "confirm" }), { kind: "done", removed: 9 }).message).toBe("History cleared.");
  });

  it("picking another range drops a question that was showing", () => {
    const asking = clearStep(CLEAR_START, { kind: "ask" });
    expect(clearStep(asking, { kind: "pick", range: "day" })).toEqual({ range: "day", asking: false, busy: false, message: "" });
  });
});
