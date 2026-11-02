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

  it("says in one short line what each choice does, or why the solid theme shows", () => {
    expect(glassNote("off", MAC26)).toBe("Glass for the toolbar and the command bar, tinted by your theme.");
    expect(glassNote("clear", MAC26)).toBe("More transparent. Tinted by your theme; no grain on glass.");
    expect(glassNote("tinted", MAC26)).toBe("More opaque, with more contrast. Tinted by your theme.");
    for (const choice of ["off", "clear", "tinted"] as const) {
      expect(glassNote(choice, { available: true, blockedBy: "reduceTransparency" })).toBe(
        "Reduce Transparency is on in Accessibility settings, so the theme shows solid.",
      );
      expect(glassNote(choice, { available: true, blockedBy: "increaseContrast" })).toMatch(/^Increase Contrast is on/);
    }
    // One line in the Appearance tab (about 70 characters at 12 px).
    for (const choice of ["off", "clear", "tinted"] as const) {
      for (const blockedBy of [null, "reduceTransparency", "increaseContrast"] as const) {
        expect(glassNote(choice, { available: true, blockedBy }).length).toBeLessThanOrEqual(80);
      }
    }
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
