import { describe, expect, it } from "vitest";
import { bootTheme, earlyTheme, needsTheme, sameTheme } from "./bar-boot.ts";
import { themeFrom } from "./lib/theme.ts";

const DUSK = { mode: "dark", colors: ["#9d8fd0", "#e0a2a0"], intensity: 0.6, grain: 0.4 };

describe("the early theme", () => {
  it("is applied before bootTheme returns, so before anything awaits", () => {
    const applied: unknown[] = [];
    const theme = bootTheme(DUSK, (t) => applied.push(t));
    // Synchronously: the call is recorded the moment bootTheme returns.
    expect(applied).toEqual([themeFrom(DUSK)]);
    expect(theme).toEqual(themeFrom(DUSK));
  });

  it("is checked like any theme Rust sends", () => {
    const theme = earlyTheme({ mode: "LIGHT", colors: ["#ABC", "<script>", 3], intensity: 7, grain: -1 });
    expect(theme).toEqual({ mode: "light", colors: ["#aabbcc"], intensity: 1, grain: 0 });
  });

  it("carries its Liquid Glass to the first paint (LJ19.9: no flash of the solid theme)", () => {
    const applied: unknown[] = [];
    bootTheme({ ...DUSK, glass: "tinted" }, (t) => applied.push(t));
    expect(applied).toEqual([{ ...themeFrom(DUSK), glass: "tinted" }]);
  });

  it("falls back to start_info's path when missing or malformed", () => {
    for (const bad of [undefined, null, "dark", 3, [], { colors: ["#ffffff"] }, { mode: "dark" }, { mode: 1, colors: [] }]) {
      const applied: unknown[] = [];
      expect(bootTheme(bad, (t) => applied.push(t))).toBeNull();
      expect(applied).toEqual([]);
    }
  });
});

describe("start_info's theme", () => {
  it("is applied when there was no early theme", () => {
    expect(needsTheme(null, DUSK)).toBe(true);
  });

  it("is applied again only if it differs (a preview changed it meanwhile)", () => {
    const early = themeFrom(DUSK);
    expect(needsTheme(early, DUSK)).toBe(false);
    expect(needsTheme(early, { ...DUSK, mode: "light" })).toBe(true);
    expect(needsTheme(early, { ...DUSK, colors: ["#9d8fd0"] })).toBe(true);
    expect(needsTheme(early, { ...DUSK, grain: 0.5 })).toBe(true);
  });

  it("compares every part of a theme", () => {
    const a = themeFrom(DUSK);
    expect(sameTheme(a, themeFrom({ ...DUSK }))).toBe(true);
    // Liquid Glass too (LJ19.9): a summon after it changed applies the theme again.
    expect(sameTheme(a, themeFrom({ ...DUSK, glass: "clear" }))).toBe(false);
    expect(sameTheme(themeFrom({ ...DUSK, glass: "clear" }), themeFrom({ ...DUSK, glass: "tinted" }))).toBe(false);
    expect(sameTheme(themeFrom({ ...DUSK, glass: "tinted" }), themeFrom({ ...DUSK, glass: "tinted" }))).toBe(true);
    expect(needsTheme(a, { ...DUSK, glass: "clear" })).toBe(true);
    expect(sameTheme(a, themeFrom({ ...DUSK, colors: ["#e0a2a0", "#9d8fd0"] }))).toBe(false);
    expect(sameTheme(a, themeFrom({ ...DUSK, intensity: 0.61 }))).toBe(false);
  });
});
