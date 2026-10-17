import { describe, expect, it } from "vitest";
import { acceleratorFromEvent, displayShortcut, keyName, type KeyPress } from "./shortcut.ts";

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

describe("acceleratorFromEvent", () => {
  const press = (code: string, mods: Partial<Omit<KeyPress, "code">> = {}): KeyPress => ({
    code,
    ctrlKey: false,
    shiftKey: false,
    altKey: false,
    metaKey: false,
    ...mods,
  });

  it("stores the primary modifier as CommandOrControl on every platform", () => {
    expect(acceleratorFromEvent(press("Space", { ctrlKey: true, shiftKey: true }), false)).toBe(
      "CommandOrControl+Shift+Space",
    );
    expect(acceleratorFromEvent(press("Space", { metaKey: true, shiftKey: true }), true)).toBe(
      "CommandOrControl+Shift+Space",
    );
  });

  it("keeps the secondary modifier distinct", () => {
    expect(acceleratorFromEvent(press("KeyJ", { ctrlKey: true, altKey: true }), true)).toBe("Control+Alt+KeyJ");
    expect(acceleratorFromEvent(press("KeyJ", { metaKey: true }), false)).toBe("Super+KeyJ");
  });

  it("rejects a modifier alone, a bare key, and Shift as the only modifier", () => {
    expect(acceleratorFromEvent(press("ShiftLeft", { shiftKey: true }), false)).toBeNull();
    expect(acceleratorFromEvent(press("ControlLeft", { ctrlKey: true }), false)).toBeNull();
    expect(acceleratorFromEvent(press("KeyA"), false)).toBeNull();
    expect(acceleratorFromEvent(press("KeyA", { shiftKey: true }), false)).toBeNull();
    expect(acceleratorFromEvent(press("F5"), false)).toBeNull();
  });

  it("allows F13–F24 without a modifier", () => {
    expect(acceleratorFromEvent(press("F13"), false)).toBe("F13");
    expect(acceleratorFromEvent(press("F24"), true)).toBe("F24");
  });

  it("rejects keys a global shortcut can't use", () => {
    expect(acceleratorFromEvent(press("Escape", { ctrlKey: true }), false)).toBeNull();
    expect(acceleratorFromEvent(press("CapsLock", { ctrlKey: true }), false)).toBeNull();
    expect(acceleratorFromEvent(press("IntlBackslash", { ctrlKey: true }), false)).toBeNull();
  });

  it("round-trips through the display text", () => {
    const accelerator = acceleratorFromEvent(press("Digit1", { ctrlKey: true, altKey: true }), false);
    expect(accelerator).toBe("CommandOrControl+Alt+Digit1");
    expect(displayShortcut(accelerator ?? "", false)).toBe("Ctrl+Alt+1");
    expect(displayShortcut(accelerator ?? "", true)).toBe("⌘⌥1");
  });
});
