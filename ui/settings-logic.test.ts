import { describe, expect, it } from "vitest";
import { isChanged, toInput, type Settings } from "./settings-logic.ts";

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
});
