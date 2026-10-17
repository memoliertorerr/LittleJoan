import { describe, expect, it } from "vitest";
import { barKey, mayReplace, textFor } from "./bar-logic.ts";

const PAGE = "https://example.com/a";

describe("barKey", () => {
  it("opens the trimmed text on Enter", () => {
    expect(barKey("Enter", "  rust lang ", PAGE, false)).toEqual({ kind: "open", input: "rust lang" });
    expect(barKey("Enter", PAGE, PAGE, false)).toEqual({ kind: "open", input: PAGE });
  });

  it("ignores Enter on an empty field and while composing", () => {
    expect(barKey("Enter", "   ", PAGE, false)).toEqual({ kind: "none" });
    expect(barKey("Enter", "日本", "", true)).toEqual({ kind: "none" });
  });

  it("reverts an edit on Esc", () => {
    expect(barKey("Escape", "https://exam", PAGE, false)).toEqual({ kind: "revert" });
    expect(barKey("Escape", "typed", "", false)).toEqual({ kind: "revert" });
  });

  it("with nothing to revert, Esc returns to the page — or closes on the start page", () => {
    expect(barKey("Escape", PAGE, PAGE, false)).toEqual({ kind: "focusPage" });
    expect(barKey("Escape", "", "", false)).toEqual({ kind: "hibernate" });
  });

  it("ignores other keys", () => {
    expect(barKey("a", "a", PAGE, false)).toEqual({ kind: "none" });
  });
});

describe("mayReplace", () => {
  it("replaces freely while the user is elsewhere", () => {
    expect(mayReplace("anything", PAGE, false)).toBe(true);
  });

  it("never clobbers an edit in progress", () => {
    expect(mayReplace("https://new", PAGE, true)).toBe(false);
    expect(mayReplace(PAGE, PAGE, true)).toBe(true);
  });
});

describe("textFor", () => {
  it("shows nothing for the start page", () => {
    expect(textFor(null)).toBe("");
    expect(textFor(PAGE)).toBe(PAGE);
  });
});
