import { describe, expect, it } from "vitest";
import { parseAccelerator, writeAccelerator } from "./keyboard-logic.ts";
import { RESERVED } from "./reserved-shortcuts.ts";

const has = (os: string, accelerator: string) => RESERVED.some((entry) => entry.os === os && entry.accelerator === accelerator);

describe("the OSes' reserved shortcuts (LJ44)", () => {
  it("every entry parses, has a name and an OS, and no OS lists one twice", () => {
    const seen = new Set<string>();
    for (const entry of RESERVED) {
      const accelerator = entry.accelerator.endsWith("+*") ? `${entry.accelerator.slice(0, -2)}+KeyA` : entry.accelerator;
      const parsed = parseAccelerator(accelerator);
      expect(parsed, entry.accelerator).not.toBeNull();
      expect(writeAccelerator(parsed ?? { primary: false, control: false, meta: false, alt: false, shift: false, code: "" }), entry.accelerator).toBe(accelerator);
      expect(entry.name.trim(), entry.accelerator).not.toBe("");
      expect(["macos", "windows", "linux", "all"]).toContain(entry.os);
      const key = `${entry.os} ${entry.accelerator}`;
      expect(seen.has(key), key).toBe(false);
      seen.add(key);
    }
  });

  it("has the shortcuts the plan names", () => {
    for (const accelerator of [
      "CommandOrControl+Space",
      "Control+Space",
      "Control+Alt+Space",
      "CommandOrControl+Shift+Digit3",
      "CommandOrControl+Shift+Digit4",
      "CommandOrControl+Shift+Digit5",
      "CommandOrControl+Tab",
      "CommandOrControl+Backquote",
      "CommandOrControl+Control+KeyQ",
      "CommandOrControl+Control+Space",
      "CommandOrControl+Alt+Escape",
      "CommandOrControl+KeyH",
      "CommandOrControl+KeyM",
    ]) {
      expect(has("macos", accelerator), accelerator).toBe(true);
    }
    for (const accelerator of ["Alt+Tab", "Alt+F4", "Alt+Space", "Control+Escape", "Control+Shift+Escape", "Super+*", "Control+Super+ArrowLeft", "Control+Super+ArrowRight", "PrintScreen"]) {
      expect(has("windows", accelerator), accelerator).toBe(true);
    }
    for (const accelerator of ["Super+*", "Alt+Tab", "Alt+F2", "Alt+F4", "Alt+Space", "Control+Alt+KeyT", "Control+Alt+ArrowLeft", "Control+Alt+ArrowRight", "Super+Space"]) {
      expect(has("linux", accelerator), accelerator).toBe(true);
    }
    for (const accelerator of ["CommandOrControl+KeyC", "CommandOrControl+KeyV", "CommandOrControl+KeyX", "CommandOrControl+KeyA", "CommandOrControl+KeyZ", "CommandOrControl+Shift+KeyZ"]) {
      expect(RESERVED.find((entry) => entry.os === "all" && entry.accelerator === accelerator)?.level, accelerator).toBe("error");
    }
  });
});
