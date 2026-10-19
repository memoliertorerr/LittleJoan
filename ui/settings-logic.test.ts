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
  trayLabelFor,
  trayText,
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
  developerMode: false,
  disabledExtensions: ["hello"],
  siteModes: { "127.0.0.1": "desktop" },
  showTrayIcon: true,
};

const same = () => toInput(saved.shortcut, saved.searchEngine, saved.translateTo, saved.mobileSites, saved.rememberHistory);

describe("settings logic", () => {
  it("counts an edited theme as a change, and an equal one as none", () => {
    const theme = { ...saved.theme, colors: [...saved.theme.colors] };
    const withTheme = (t: typeof theme) => toInput(saved.shortcut, saved.searchEngine, saved.translateTo, saved.mobileSites, saved.rememberHistory, t);
    expect(isChanged(saved, withTheme(theme))).toBe(false);
    expect(isChanged(saved, withTheme({ ...theme, mode: "dark" }))).toBe(true);
    expect(isChanged(saved, withTheme({ ...theme, colors: [...theme.colors, "#123456"] }))).toBe(true);
    expect(isChanged(saved, withTheme({ ...theme, grain: 0.9 }))).toBe(true);
    expect(withTheme(theme).theme).toEqual(theme);
    expect("theme" in same()).toBe(false);
  });

  it("counts the Extensions tab's switches as changes, in any order", () => {
    const withExtensions = (disabledExtensions: string[], developerMode: boolean) =>
      toInput(saved.shortcut, saved.searchEngine, saved.translateTo, saved.mobileSites, saved.rememberHistory, undefined, {
        disabledExtensions,
        developerMode,
      });
    expect(isChanged(saved, withExtensions(["hello"], false))).toBe(false);
    expect(isChanged(saved, withExtensions([], false))).toBe(true);
    expect(isChanged(saved, withExtensions(["hello", "builtin:calc"], false))).toBe(true);
    expect(isChanged(saved, withExtensions(["hello"], true))).toBe(true);
    expect(isChanged({ ...saved, disabledExtensions: ["b", "a"] }, withExtensions(["a", "b"], false))).toBe(false);
    expect(withExtensions([], true)).toMatchObject({ disabledExtensions: [], developerMode: true });
    expect("developerMode" in same()).toBe(false);
  });

  it("counts the per-site list as a change only when it differs, and sends it only then", () => {
    const withSites = (siteModes: Settings["siteModes"]) =>
      toInput(saved.shortcut, saved.searchEngine, saved.translateTo, saved.mobileSites, saved.rememberHistory, undefined, undefined, siteModes);
    expect(isChanged(saved, withSites({ "127.0.0.1": "desktop" }))).toBe(false);
    expect(isChanged(saved, withSites({}))).toBe(true);
    expect(isChanged(saved, withSites({ "127.0.0.1": "phone" }))).toBe(true);
    expect(isChanged(saved, withSites({ "127.0.0.1": "desktop", "youtube.com": "auto" }))).toBe(true);
    expect(withSites({}).siteModes).toEqual({});
    expect("siteModes" in same()).toBe(false);
  });

  it("counts the icon switch as a change only when it differs", () => {
    const withIcon = (showTrayIcon: boolean) =>
      toInput(saved.shortcut, saved.searchEngine, saved.translateTo, saved.mobileSites, saved.rememberHistory, undefined, undefined, undefined, showTrayIcon);
    expect(isChanged(saved, withIcon(true))).toBe(false);
    expect(isChanged(saved, withIcon(false))).toBe(true);
    expect(isChanged({ ...saved, showTrayIcon: false }, withIcon(false))).toBe(false);
    expect(withIcon(false).showTrayIcon).toBe(false);
    expect("showTrayIcon" in same()).toBe(false);
    expect(hasChanges(changes(saved, true, withIcon(false), true))).toBe(true);
  });

  it("names the menu bar on macOS and the system tray elsewhere, and the shortcut in the note", () => {
    expect(trayLabelFor("macos")).toBe("Show the icon in the menu bar");
    expect(trayLabelFor("windows")).toBe("Show the icon in the system tray");
    expect(trayLabelFor("linux")).toBe("Show the icon in the system tray");
    expect(trayText("⌘⇧Space")).toBe(
      "Without it, press ⌘⇧Space or open Little Joan again from your apps. The icon comes back by itself whenever the shortcut can't work.",
    );
    expect(trayText("")).toContain("press your shortcut");
  });

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
