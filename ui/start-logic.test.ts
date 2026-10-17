import { describe, expect, it } from "vitest";
import { onKey, prefillFrom } from "./start-logic.ts";

describe("onKey", () => {
  it("opens the trimmed input on Enter", () => {
    expect(onKey("Enter", "  example.com ", "", false)).toEqual({ kind: "open", input: "example.com" });
  });

  it("ignores Enter on an empty field and while composing", () => {
    expect(onKey("Enter", "   ", "", false)).toEqual({ kind: "none" });
    expect(onKey("Enter", "日本", "", true)).toEqual({ kind: "none" });
  });

  it("closes on Esc when the start page is fresh", () => {
    expect(onKey("Escape", "", "", false)).toEqual({ kind: "hibernate" });
  });

  it("goes back on Esc when opened over a page", () => {
    expect(onKey("Escape", "https://example.com/", "?u=https%3A%2F%2Fexample.com%2F", false)).toEqual({
      kind: "back",
    });
  });

  it("ignores other keys", () => {
    expect(onKey("a", "a", "", false)).toEqual({ kind: "none" });
  });
});

describe("prefillFrom", () => {
  it("reads and decodes u", () => {
    expect(prefillFrom("?u=https%3A%2F%2Fexample.com%2Fa%3Fb%3D1")).toBe("https://example.com/a?b=1");
  });

  it("is empty without u", () => {
    expect(prefillFrom("")).toBe("");
    expect(prefillFrom("?x=1")).toBe("");
  });
});
