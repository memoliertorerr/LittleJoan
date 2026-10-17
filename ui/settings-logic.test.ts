import { describe, expect, it } from "vitest";
import { isChanged, toInput, type Settings } from "./settings-logic.ts";

const saved: Settings = {
  version: 1,
  shortcut: "CommandOrControl+Shift+Space",
  searchEngine: "google",
  translateTo: "en",
  windowWidth: 480,
  windowHeight: 640,
};

describe("settings logic", () => {
  it("sends only the edited fields", () => {
    expect(toInput("Alt+KeyJ", "brave", "tr")).toEqual({ shortcut: "Alt+KeyJ", searchEngine: "brave", translateTo: "tr" });
  });

  it("detects changes field by field", () => {
    expect(isChanged(saved, toInput(saved.shortcut, saved.searchEngine, saved.translateTo))).toBe(false);
    expect(isChanged(saved, toInput("Alt+KeyJ", "google", "en"))).toBe(true);
    expect(isChanged(saved, toInput(saved.shortcut, "bing", "en"))).toBe(true);
    expect(isChanged(saved, toInput(saved.shortcut, "google", "de"))).toBe(true);
  });
});
