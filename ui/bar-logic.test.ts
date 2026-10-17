import { describe, expect, it } from "vitest";
import { mayReplace, textFor } from "./bar-logic.ts";

const PAGE = "https://example.com/a";

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
