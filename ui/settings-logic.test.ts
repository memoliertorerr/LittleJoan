import { describe, expect, it } from "vitest";
import { changes, hasChanges, isChanged, toInput, type Settings } from "./settings-logic.ts";

const saved: Settings = {
  version: 1,
  shortcut: "CommandOrControl+Shift+Space",
  searchEngine: "google",
  translateTo: "en",
  windowWidth: 480,
  windowHeight: 640,
  mobileSites: true,
};

describe("settings logic", () => {
  it("sends only the edited fields", () => {
    expect(toInput("Alt+KeyJ", "brave", "tr", false)).toEqual({
      shortcut: "Alt+KeyJ",
      searchEngine: "brave",
      translateTo: "tr",
      mobileSites: false,
    });
  });

  it("detects changes field by field", () => {
    expect(isChanged(saved, toInput(saved.shortcut, saved.searchEngine, saved.translateTo, true))).toBe(false);
    expect(isChanged(saved, toInput("Alt+KeyJ", "google", "en", true))).toBe(true);
    expect(isChanged(saved, toInput(saved.shortcut, "bing", "en", true))).toBe(true);
    expect(isChanged(saved, toInput(saved.shortcut, "google", "de", true))).toBe(true);
    expect(isChanged(saved, toInput(saved.shortcut, "google", "en", false))).toBe(true);
  });

  it("includes the start-at-login switch, applied on its own", () => {
    const same = toInput(saved.shortcut, saved.searchEngine, saved.translateTo, saved.mobileSites);
    expect(changes(saved, false, same, false)).toEqual({ settings: false, startAtLogin: false });
    expect(hasChanges(changes(saved, false, same, false))).toBe(false);
    expect(changes(saved, false, same, true)).toEqual({ settings: false, startAtLogin: true });
    expect(changes(saved, true, same, false)).toEqual({ settings: false, startAtLogin: true });
    expect(hasChanges(changes(saved, true, same, false))).toBe(true);
    const other = toInput("Alt+KeyJ", saved.searchEngine, saved.translateTo, saved.mobileSites);
    expect(changes(saved, true, other, false)).toEqual({ settings: true, startAtLogin: true });
    expect(changes(saved, true, other, true)).toEqual({ settings: true, startAtLogin: false });
  });

  it("never changes a login entry that couldn't be read", () => {
    const same = toInput(saved.shortcut, saved.searchEngine, saved.translateTo, saved.mobileSites);
    expect(changes(saved, null, same, true)).toEqual({ settings: false, startAtLogin: false });
    expect(hasChanges(changes(saved, null, same, false))).toBe(false);
  });
});
