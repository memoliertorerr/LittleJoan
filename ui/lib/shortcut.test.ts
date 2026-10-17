import { describe, expect, it } from "vitest";
import { displayShortcut, keyName } from "./shortcut.ts";

describe("displayShortcut", () => {
  it("uses symbols on macOS", () => {
    expect(displayShortcut("CommandOrControl+Shift+Space", true)).toBe("⌘⇧Space");
    expect(displayShortcut("Control+Alt+KeyJ", true)).toBe("⌃⌥J");
  });

  it("uses names elsewhere", () => {
    expect(displayShortcut("CommandOrControl+Shift+Space", false)).toBe("Ctrl+Shift+Space");
    expect(displayShortcut("Alt+Shift+Digit1", false)).toBe("Alt+Shift+1");
    expect(displayShortcut("Super+F5", false)).toBe("Super+F5");
  });

  it("handles empty input", () => {
    expect(displayShortcut("", false)).toBe("");
  });
});

describe("keyName", () => {
  it("shortens letter and digit codes only", () => {
    expect(keyName("KeyA")).toBe("A");
    expect(keyName("Digit0")).toBe("0");
    expect(keyName("Space")).toBe("Space");
    expect(keyName("Keyboard")).toBe("Keyboard");
  });
});
