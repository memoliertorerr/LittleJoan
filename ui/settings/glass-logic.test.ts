import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { GLASS_CHOICES, NO_GLASS, glassNote, glassSettingFrom, glassSupportFrom } from "./glass-logic.ts";

const MAC26 = { available: true, blockedBy: null } as const;

describe("Liquid Glass in Settings (LJ19.9)", () => {
  it("offers Off, Clear and Tinted, like macOS 26.1's own setting", () => {
    expect(GLASS_CHOICES.map((c) => [c.id, c.name])).toEqual([
      ["off", "Off"],
      ["clear", "Clear"],
      ["tinted", "Tinted"],
    ]);
  });

  it("reads the setting as one of the three, Off by default", () => {
    expect(glassSettingFrom("clear")).toBe("clear");
    expect(glassSettingFrom("tinted")).toBe("tinted");
    for (const junk of [undefined, null, "", "Clear", "frosted", 1, true]) {
      expect(glassSettingFrom(junk)).toBe("off");
    }
  });

  it("shows only where Rust says this Mac has glass", () => {
    expect(glassSupportFrom({ available: true, blockedBy: null })).toEqual(MAC26);
    expect(glassSupportFrom({ available: true, blockedBy: "reduceTransparency" })).toEqual({ available: true, blockedBy: "reduceTransparency" });
    expect(glassSupportFrom({ available: true, blockedBy: "increaseContrast" }).blockedBy).toBe("increaseContrast");
    for (const none of [undefined, null, "yes", { available: "true" }, { available: 1 }, {}]) {
      expect(glassSupportFrom(none)).toEqual(NO_GLASS);
    }
    expect(glassSupportFrom({ available: true, blockedBy: "darkMode" })).toEqual(MAC26);
  });

  it("says what each choice does, and why the solid theme shows when the system asks for it", () => {
    expect(glassNote("off", MAC26)).toBe("Glass for the toolbar and the command bar, tinted by the theme's color. Pages stay as they are.");
    expect(glassNote("clear", MAC26)).toMatch(/^More transparent: what is behind the window shows through\./);
    expect(glassNote("tinted", MAC26)).toMatch(/^More opaque, with more contrast\./);
    for (const choice of ["clear", "tinted"] as const) {
      expect(glassNote(choice, MAC26)).toContain("the grain doesn't show on glass");
    }
    expect(glassNote("clear", { available: true, blockedBy: "reduceTransparency" })).toMatch(
      /Reduce Transparency is on \(System Settings → Accessibility → Display\), so Little Joan shows the solid theme\.$/,
    );
    expect(glassNote("tinted", { available: true, blockedBy: "increaseContrast" })).toMatch(/Increase Contrast is on .* solid theme\.$/);
  });

  it("is hidden in the markup until Rust says otherwise, its buttons named by data-glass-choice", () => {
    const html = readFileSync(resolve(import.meta.dirname, "..", "settings.html"), "utf8");
    expect(html).toMatch(/<div id="glass-setting" class="glass-setting" hidden>/);
    const choices = [...html.matchAll(/data-glass-choice="(\w+)"/g)].map((m) => m[1]);
    expect(choices).toEqual(["off", "clear", "tinted"]);
    // Inside the Appearance tab, after the sliders.
    const appearance = html.slice(html.indexOf('id="panel-appearance"'), html.indexOf('id="panel-extensions"'));
    expect(appearance.indexOf('id="glass-setting"')).toBeGreaterThan(appearance.indexOf('id="grain"'));
  });
});
