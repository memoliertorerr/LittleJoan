import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { contrast, contrastRgb, hexToOklch, normalizeHex, over, parseHex, toHex } from "./color.ts";
import { DEFAULT_PRESET, PRESETS } from "./presets.ts";
import { defaultsCss } from "./theme-css.ts";
import {
  DEFAULT_THEME,
  GRAIN_OPACITY,
  LIGHTNESS,
  MAX_CHROMA,
  backgrounds,
  gradientCss,
  gradientStops,
  grainOpacity,
  grainUri,
  mute,
  palette,
  schemeFor,
  textBackgrounds,
  themeFrom,
  tokens,
  type Scheme,
  type Theme,
} from "./theme.ts";

const SCHEMES: Scheme[] = ["light", "dark"];

/** A small seeded generator, so the "any colors" tests are the same every run. */
function random(seed: number): () => number {
  let state = seed;
  return () => {
    state = (state * 1664525 + 1013904223) % 4294967296;
    return state / 4294967296;
  };
}

function randomHex(next: () => number): string {
  return toHex({ r: next(), g: next(), b: next() });
}

function randomTheme(next: () => number): Theme {
  const count = 1 + Math.floor(next() * 3);
  return {
    mode: (["system", "light", "dark"] as const)[Math.floor(next() * 3)] ?? "system",
    colors: Array.from({ length: count }, () => randomHex(next)),
    intensity: next(),
    grain: next(),
  };
}

function presetTheme(preset: (typeof PRESETS)[number], mode: Theme["mode"]): Theme {
  return { mode, colors: [...preset.colors], intensity: preset.intensity, grain: preset.grain };
}

const EPSILON = 1e-9;

describe("muting a pick", () => {
  it("keeps every background inside the lightness band and the chroma cap", () => {
    const next = random(7);
    // The extremes too: black, white, the primaries.
    const picks = ["#000000", "#ffffff", "#ff0000", "#00ff00", "#0000ff", "#ffff00"];
    for (let i = 0; i < 300; i += 1) {
      picks.push(randomHex(next));
    }
    for (const scheme of SCHEMES) {
      const [low, high] = LIGHTNESS[scheme];
      for (const pick of picks) {
        for (const intensity of [0, 0.3, 0.6, 1]) {
          const muted = mute(pick, scheme, intensity);
          expect(muted.l).toBeGreaterThanOrEqual(low - EPSILON);
          expect(muted.l).toBeLessThanOrEqual(high + EPSILON);
          expect(muted.c).toBeLessThanOrEqual(MAX_CHROMA * intensity + EPSILON);
        }
      }
    }
    expect(LIGHTNESS).toEqual({ light: [0.86, 0.93], dark: [0.22, 0.32] });
    expect(MAX_CHROMA).toBe(0.12);
  });

  it("holds the clamps for the backgrounds themes are drawn with, as written in hex", () => {
    const next = random(11);
    for (let i = 0; i < 200; i += 1) {
      const theme = randomTheme(next);
      for (const scheme of SCHEMES) {
        const [low, high] = LIGHTNESS[scheme];
        for (const bg of backgrounds(theme, scheme)) {
          expect(bg.l).toBeGreaterThanOrEqual(low - EPSILON);
          expect(bg.l).toBeLessThanOrEqual(high + EPSILON);
          expect(bg.c).toBeLessThanOrEqual(MAX_CHROMA * theme.intensity + EPSILON);
        }
        // Rounding to #rrggbb moves a color by at most half a step per channel.
        for (const hex of palette({ ...theme, mode: scheme }, false).backgrounds) {
          const lch = hexToOklch(hex);
          expect(lch?.l ?? 0).toBeGreaterThanOrEqual(low - 0.005);
          expect(lch?.l ?? 1).toBeLessThanOrEqual(high + 0.005);
        }
      }
    }
  });

  it("keeps the pick's hue and orders lighter picks lighter", () => {
    const teal = mute("#2a9d8f", "light", 1);
    expect(Math.abs(teal.h - (hexToOklch("#2a9d8f")?.h ?? 0))).toBeLessThan(1e-6);
    expect(mute("#eeeeee", "dark", 0.5).l).toBeGreaterThan(mute("#222222", "dark", 0.5).l);
    // Intensity 0 is grey.
    expect(mute("#ff0000", "light", 0).c).toBe(0);
  });
});

describe("the gradient", () => {
  const themes: Record<string, Theme> = {
    one: { mode: "light", colors: ["#8c9bb5"], intensity: 0.5, grain: 0 },
    two: { mode: "light", colors: ["#9d8fd0", "#e0a2a0"], intensity: 0.6, grain: 0.4 },
    three: { mode: "dark", colors: ["#94bce2", "#a9d8d2", "#c7bfe8"], intensity: 0.5, grain: 0.2 },
  };

  it("is deterministic for 1, 2 and 3 colors", () => {
    const seen = new Set<string>();
    for (const theme of Object.values(themes)) {
      const css = gradientCss(palette(theme, false).backgrounds);
      expect(gradientCss(palette({ ...theme, colors: [...theme.colors] }, false).backgrounds)).toBe(css);
      expect(tokens(theme, false).vars["--lj-gradient"]).toBe(css);
      seen.add(css);
    }
    expect(seen.size).toBe(3);
  });

  it("is two soft blobs over a linear gradient, in plain sRGB", () => {
    const css = gradientCss(palette(themes.two ?? DEFAULT_THEME, false).backgrounds);
    const layers = css.split(/,\s*(?=(?:radial|linear)-gradient\()/);
    expect(layers).toHaveLength(3);
    expect(layers[0]).toMatch(/^radial-gradient\(120% 95% at 100% 0%, /);
    expect(layers[1]).toMatch(/^radial-gradient\(110% 90% at 0% 100%, /);
    expect(layers[2]).toMatch(/^linear-gradient\(160deg, #[0-9a-f]{6} 0%, (#[0-9a-f]{6} \d+%, ){3}#[0-9a-f]{6} 100%\)$/);
    // Blobs fade to nothing (alpha 0 at their edge).
    expect(layers[0]).toMatch(/, 0\) 100%\)$/);
    expect(css).not.toMatch(/oklch|oklab|color-mix|transparent/);
  });

  it("starts at --lj-bg-0 and only takes colors from the theme's backgrounds", () => {
    const p = palette(themes.two ?? DEFAULT_THEME, false);
    const stops = gradientStops(p.backgrounds);
    expect(stops[0]).toBe(p.backgrounds[0]);
    expect(stops).toContain(p.backgrounds[1]);
    expect(stops).toContain(p.backgrounds[2]);
  });
});

describe("text contrast", () => {
  function check(theme: Theme, systemDark: boolean, label: string): void {
    const p = palette(theme, systemDark);
    const under = textBackgrounds(p, gradientStops(p.backgrounds), theme.grain);
    const fg = parseHex(p.fg) ?? { r: 0, g: 0, b: 0 };
    const muted = parseHex(p.fgMuted) ?? { r: 0, g: 0, b: 0 };
    const accent = parseHex(p.accent) ?? { r: 0, g: 0, b: 0 };
    for (const bg of under) {
      expect(contrastRgb(fg, bg), `${label}: text`).toBeGreaterThanOrEqual(4.5);
      expect(contrastRgb(muted, bg), `${label}: muted text`).toBeGreaterThanOrEqual(4.5);
      expect(contrastRgb(accent, bg), `${label}: accent`).toBeGreaterThanOrEqual(4.5);
    }
    // Every gradient stop as written, on its own.
    for (const stop of gradientStops(p.backgrounds)) {
      expect(contrast(p.fg, stop), `${label}: text on ${stop}`).toBeGreaterThanOrEqual(4.5);
    }
    expect(contrast(p.onSelected, p.rowSelected), `${label}: selected row`).toBeGreaterThanOrEqual(4.5);
    // The panel and the opaque surface carry text too.
    expect(contrast(p.fg, p.surface), `${label}: text on surface`).toBeGreaterThanOrEqual(4.5);
    expect(contrast(p.fgMuted, toHex(over(p.panel.rgb, p.panel.alpha, parseHex(p.surface) ?? fg)))).toBeGreaterThanOrEqual(4.5);
  }

  it("reaches 4.5:1 against every gradient stop for every preset, light and dark", () => {
    for (const preset of PRESETS) {
      for (const mode of ["light", "dark"] as const) {
        check(presetTheme(preset, mode), false, `${preset.id} ${mode}`);
      }
      check(presetTheme(preset, "system"), true, `${preset.id} system (dark)`);
    }
  });

  it("holds for any colors a person can pick", () => {
    const next = random(42);
    for (let i = 0; i < 150; i += 1) {
      const theme = randomTheme(next);
      check(theme, next() > 0.5, JSON.stringify(theme));
    }
  });

  it("uses near-black text in light themes and near-white in dark ones", () => {
    const light = palette({ ...DEFAULT_THEME, mode: "light" }, false);
    const dark = palette({ ...DEFAULT_THEME, mode: "dark" }, false);
    expect(hexToOklch(light.fg)?.l ?? 1).toBeLessThan(0.3);
    expect(hexToOklch(dark.fg)?.l ?? 0).toBeGreaterThan(0.9);
    expect(light.onSelected).toBe("#ffffff");
  });
});

describe("the grain", () => {
  it("is an SVG data URI with feTurbulence at the grain's opacity", () => {
    for (const grain of [0.15, 0.4, 1]) {
      const uri = grainUri(grain);
      expect(uri.startsWith("data:image/svg+xml,")).toBe(true);
      const svg = decodeURIComponent(uri.slice("data:image/svg+xml,".length));
      expect(svg).toContain("<feTurbulence type='fractalNoise'");
      expect(svg).toContain(`opacity='${grainOpacity(grain)}'`);
      // It goes into url("…"): no double quote may end it early.
      expect(svg).not.toContain('"');
    }
    expect(grainOpacity(0.4)).toBeCloseTo(0.4 * GRAIN_OPACITY, 9);
    expect(grainOpacity(1)).toBe(GRAIN_OPACITY);
    expect(grainOpacity(7)).toBe(GRAIN_OPACITY);
    expect(grainUri(0)).toBe("");
    // In CSS, as an url() — and "none" without grain.
    expect(tokens({ ...DEFAULT_THEME, grain: 0.4 }, false).vars["--lj-grain"]).toBe(`url("${grainUri(0.4)}")`);
    expect(tokens({ ...DEFAULT_THEME, grain: 0 }, false).vars["--lj-grain"]).toBe("none");
  });
});

describe("tokens", () => {
  it("are the documented CSS variables, in plain sRGB", () => {
    const { vars } = tokens(DEFAULT_THEME, false);
    expect(Object.keys(vars)).toEqual([
      "--lj-bg-0",
      "--lj-bg-1",
      "--lj-bg-2",
      "--lj-fg",
      "--lj-fg-muted",
      "--lj-accent",
      "--lj-field",
      "--lj-row-hover",
      "--lj-row-selected",
      "--lj-on-selected",
      "--lj-panel",
      "--lj-surface",
      "--lj-tile",
      "--lj-hairline",
      "--lj-shadow",
      "--lj-gradient",
      "--lj-grain",
    ]);
    for (const [name, value] of Object.entries(vars)) {
      if (name !== "--lj-gradient" && name !== "--lj-grain") {
        expect(value, name).toMatch(/^(#[0-9a-f]{6}|rgba\(\d+, \d+, \d+, [\d.]+\))$/);
      }
    }
  });

  it("follow the mode, and the system's scheme in system mode", () => {
    expect(schemeFor("system", true)).toBe("dark");
    expect(schemeFor("system", false)).toBe("light");
    expect(schemeFor("light", true)).toBe("light");
    expect(schemeFor("dark", false)).toBe("dark");
    expect(tokens({ ...DEFAULT_THEME, mode: "system" }, true).vars).toEqual(tokens({ ...DEFAULT_THEME, mode: "dark" }, false).vars);
    expect(tokens({ ...DEFAULT_THEME, mode: "light" }, true).scheme).toBe("light");
  });
});

describe("reading a theme", () => {
  it("checks every field on its own", () => {
    expect(themeFrom(undefined)).toEqual(DEFAULT_THEME);
    expect(themeFrom("dark")).toEqual(DEFAULT_THEME);
    expect(themeFrom({ mode: " Dark ", colors: ["#ABC", "nope", 3, "#123456", "#654321", "#111111"], intensity: 2, grain: -1 })).toEqual({
      mode: "dark",
      colors: ["#aabbcc", "#123456", "#654321"],
      intensity: 1,
      grain: 0,
    });
    expect(themeFrom({ mode: "sepia", colors: [], intensity: "x", grain: Number.NaN })).toEqual(DEFAULT_THEME);
  });
});

describe("the default theme and presets", () => {
  it("has about ten muted presets with one to three valid colors", () => {
    expect(PRESETS.length).toBeGreaterThanOrEqual(10);
    expect(new Set(PRESETS.map((p) => p.id)).size).toBe(PRESETS.length);
    for (const preset of PRESETS) {
      expect(preset.colors.length).toBeGreaterThanOrEqual(1);
      expect(preset.colors.length).toBeLessThanOrEqual(3);
      for (const color of preset.colors) {
        expect(normalizeHex(color)).toBe(color);
      }
      expect(preset.intensity).toBeGreaterThanOrEqual(0);
      expect(preset.intensity).toBeLessThanOrEqual(1);
      expect(preset.grain).toBeGreaterThanOrEqual(0);
      expect(preset.grain).toBeLessThanOrEqual(1);
    }
  });

  it("is the first, neutral preset, the same as settings.rs's default", () => {
    expect(DEFAULT_THEME).toEqual({ mode: "system", colors: [...DEFAULT_PRESET.colors], intensity: DEFAULT_PRESET.intensity, grain: DEFAULT_PRESET.grain });
    const rust = readFileSync(resolve(import.meta.dirname, "../../src-tauri/src/settings/theme.rs"), "utf8");
    const constant = (name: string) => new RegExp(`pub const ${name}: [^=]+= "?([^";]+)"?;`).exec(rust)?.[1];
    expect(constant("DEFAULT_COLOR")).toBe(DEFAULT_THEME.colors[0]);
    expect(Number(constant("DEFAULT_INTENSITY"))).toBe(DEFAULT_THEME.intensity);
    expect(Number(constant("DEFAULT_GRAIN"))).toBe(DEFAULT_THEME.grain);
    expect(DEFAULT_THEME.colors).toHaveLength(1);
  });

  it("is what theme-defaults.css says (run `npm run theme:css` after changing the theme math)", () => {
    // A Windows checkout may turn the line endings into CRLF; they mean nothing in CSS.
    const css = readFileSync(resolve(import.meta.dirname, "theme-defaults.css"), "utf8").replace(/\r\n/g, "\n");
    expect(css).toBe(defaultsCss());
  });
});

describe("the base background", () => {
  // The same cases as src-tauri/src/theme_color.rs' test (LJ19.4): Rust computes --lj-bg-0 for the
  // window and the bar's webview before the bar paints, so the two must agree.
  const cases: [string[], number, boolean, string][] = [
    [["#8c9bb5"], 0.45, false, "#dae1ee"],
    [["#8c9bb5"], 0.45, true, "#262b35"],
    [["#9d8fd0", "#e0a2a0"], 0.6, false, "#e2dbff"],
    [["#9d8fd0", "#e0a2a0"], 0.6, true, "#2d2545"],
    [["#ff0000"], 1, false, "#ffd4cc"],
    [["#00ff00"], 1, true, "#003b00"],
    [["#000000"], 0.5, false, "#d1d1d1"],
    [["#ffffff"], 0, true, "#333333"],
    [["#1e90ff", "#123456", "#abcdef"], 0.8, true, "#002a54"],
    [["#2f6f4f"], 0.3, false, "#cfe2d6"],
  ];
  it("is what Rust's theme_color.rs computes", () => {
    for (const [colors, intensity, dark, want] of cases) {
      const theme = { mode: "system" as const, colors, intensity, grain: 0 };
      expect(palette(theme, dark).backgrounds[0], `${colors.join(",")} ${intensity} ${dark}`).toBe(want);
    }
  });
});
