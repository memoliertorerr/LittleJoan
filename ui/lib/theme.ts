// Themes (pure, unit-tested): one to three colors, an intensity, a grain and a mode (light, dark
// or the system's) become muted gradients and the CSS variables every surface Little Joan draws
// uses — the toolbar, the command bar, the history view, the settings window. Harmony comes from
// the derivation, not from the picks: each pick keeps its hue, its lightness is mapped into a
// narrow band (light 0.86–0.93, dark 0.22–0.32) and its chroma capped at 0.12 × intensity, so any
// three picks blend. Text is near-black or near-white, chosen so it reads on every stop.
import {
  contrastRgb,
  fitGamut,
  hexToOklch,
  mixOklab,
  normalizeHex,
  oklabToOklch,
  oklchToHex,
  oklchToOklab,
  oklchToRgb,
  over,
  parseHex,
  rgba,
  toHex,
  type Oklch,
  type Rgb,
} from "./color.ts";
import { DEFAULT_PRESET } from "./presets.ts";

export type ThemeMode = "system" | "light" | "dark";
/** What a theme resolves to on screen. */
export type Scheme = "light" | "dark";

export interface Theme {
  mode: ThemeMode;
  /** 1–3 colors, `#rrggbb`. */
  colors: string[];
  /** 0–1: how much color the surfaces get. */
  intensity: number;
  /** 0–1: how strong the grain is. */
  grain: number;
}

export const MAX_COLORS = 3;
/** The lightness band of the backgrounds, per scheme (OKLCH L). */
export const LIGHTNESS: Record<Scheme, readonly [number, number]> = { light: [0.86, 0.93], dark: [0.22, 0.32] };
/** The backgrounds' chroma at full intensity, at most. */
export const MAX_CHROMA = 0.12;
/** The grain tile's opacity at grain 1. */
export const GRAIN_OPACITY = 0.14;
/** The contrast every text color keeps against every background it can sit on. */
export const TEXT_CONTRAST = 4.5;

/** Settings' default theme (settings.rs has the same: a test compares them). */
export const DEFAULT_THEME: Theme = {
  mode: "system",
  colors: [...DEFAULT_PRESET.colors],
  intensity: DEFAULT_PRESET.intensity,
  grain: DEFAULT_PRESET.grain,
};

const clamp01 = (x: number) => Math.min(1, Math.max(0, x));

function unit(value: unknown, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) ? clamp01(value) : fallback;
}

/** A theme from anything (what Rust sent, a preview): each field falls back on its own. */
export function themeFrom(value: unknown): Theme {
  const raw = (typeof value === "object" && value !== null ? value : {}) as Record<string, unknown>;
  const mode = typeof raw.mode === "string" ? raw.mode.trim().toLowerCase() : "";
  const colors = (Array.isArray(raw.colors) ? raw.colors : [])
    .map(normalizeHex)
    .filter((hex): hex is string => hex !== null)
    .slice(0, MAX_COLORS);
  return {
    mode: mode === "light" || mode === "dark" || mode === "system" ? mode : DEFAULT_THEME.mode,
    colors: colors.length > 0 ? colors : [...DEFAULT_THEME.colors],
    intensity: unit(raw.intensity, DEFAULT_THEME.intensity),
    grain: unit(raw.grain, DEFAULT_THEME.grain),
  };
}

/** Light or dark, for `mode` and whether the system is dark. */
export function schemeFor(mode: ThemeMode, systemDark: boolean): Scheme {
  return mode === "system" ? (systemDark ? "dark" : "light") : mode;
}

function inBand(l: number, scheme: Scheme): number {
  const [low, high] = LIGHTNESS[scheme];
  return Math.min(high, Math.max(low, l));
}

/**
 * A pick as a background: its hue kept, its lightness mapped into the scheme's band (lighter
 * picks stay lighter), its chroma at most MAX_CHROMA × intensity, then fitted into sRGB.
 */
export function mute(color: string, scheme: Scheme, intensity: number): Oklch {
  const pick = hexToOklch(color) ?? hexToOklch(DEFAULT_THEME.colors[0] ?? "#808080") ?? { l: 0.5, c: 0, h: 0 };
  const [low, high] = LIGHTNESS[scheme];
  return fitGamut({
    l: low + (high - low) * clamp01(pick.l),
    c: Math.min(pick.c, MAX_CHROMA) * clamp01(intensity),
    h: pick.h,
  });
}

/** A neighbour of `base` for themes with fewer picks: lightness and hue nudged, kept in the band. */
function sibling(base: Oklch, scheme: Scheme, lightness: number, hue: number): Oklch {
  const [low, high] = LIGHTNESS[scheme];
  return fitGamut({ l: inBand(base.l + (high - low) * lightness, scheme), c: base.c, h: (base.h + hue + 360) % 360 });
}

function mix(a: Oklch, b: Oklch, t: number): Oklch {
  return oklabToOklch(mixOklab(oklchToOklab(a), oklchToOklab(b), t));
}

/**
 * The three backgrounds (--lj-bg-0..2): one pick plus two neighbours (one lighter and warmer, one
 * darker and cooler), two picks and the OKLab point between them, or three picks.
 */
export function backgrounds(theme: Theme, scheme: Scheme): [Oklch, Oklch, Oklch] {
  const [first, second, third] = theme.colors.slice(0, MAX_COLORS).map((color) => mute(color, scheme, theme.intensity));
  const base = first ?? mute(DEFAULT_THEME.colors[0] ?? "#808080", scheme, theme.intensity);
  if (!second) {
    return [base, sibling(base, scheme, 0.35, 24), sibling(base, scheme, -0.4, -30)];
  }
  if (!third) {
    const between = mix(base, second, 0.5);
    return [base, second, fitGamut({ ...between, l: inBand(between.l, scheme) })];
  }
  return [base, second, third];
}

/** The colors a theme's surfaces are drawn with (the CSS variables, minus gradient and grain). */
export interface Palette {
  scheme: Scheme;
  /** --lj-bg-0..2: the gradient's colors, `#rrggbb`. */
  backgrounds: [string, string, string];
  /** Body text: near-black or near-white. */
  fg: string;
  fgMuted: string;
  /** Focus rings, links, checkboxes. */
  accent: string;
  /** A selected row (and the primary button): white text on it. */
  rowSelected: string;
  onSelected: string;
  /** The command bar's card and the settings window's content, over the gradient. */
  panel: { rgb: Rgb; alpha: number };
  /** An opaque surface: the panel over --lj-bg-0 (favicon tiles, buttons in a box). */
  surface: string;
  /** The command bar's drop shadow. */
  shadow: string;
  /** Behind favicons: light in both schemes (most favicons are drawn for light pages). */
  tile: string;
}

/** `l` stepped from `start` by `step` until `color` reads on every one of `under`. */
function readable(under: readonly Rgb[], start: Oklch, step: number): string {
  for (let l = start.l; l >= 0 && l <= 1; l += step) {
    const rgb = oklchToRgb({ ...start, l });
    if (under.every((bg) => contrastRgb(rgb, bg) >= TEXT_CONTRAST + 0.05)) {
      return oklchToHex({ ...start, l });
    }
  }
  return step < 0 ? "#000000" : "#ffffff";
}

/** The panel's tint and opacity: a frosted card that lets the gradient through. */
function panelFor(base: Oklch, scheme: Scheme): { rgb: Rgb; alpha: number } {
  return scheme === "light"
    ? { rgb: oklchToRgb({ l: 0.99, c: base.c * 0.12, h: base.h }), alpha: 0.66 }
    : { rgb: oklchToRgb({ l: 0.17, c: base.c * 0.35, h: base.h }), alpha: 0.52 };
}

/**
 * Every background text can sit on: the stops, each with the grain's darkest and lightest
 * speck over it, and the panel over all of those.
 */
export function textBackgrounds(palette: Palette, stops: readonly string[], grain: number): Rgb[] {
  const opacity = grainOpacity(grain);
  const plain = stops.flatMap((hex) => {
    const rgb = parseHex(hex) ?? { r: 0, g: 0, b: 0 };
    return [rgb, over({ r: 0, g: 0, b: 0 }, opacity, rgb), over({ r: 1, g: 1, b: 1 }, opacity, rgb)];
  });
  return plain.concat(plain.map((rgb) => over(palette.panel.rgb, palette.panel.alpha, rgb)));
}

export function palette(theme: Theme, systemDark: boolean): Palette {
  const scheme = schemeFor(theme.mode, systemDark);
  const bgs = backgrounds(theme, scheme);
  const [base] = bgs;
  const hexes = bgs.map(oklchToHex) as [string, string, string];
  const panel = panelFor(base, scheme);
  const draft: Palette = {
    scheme,
    backgrounds: hexes,
    fg: "",
    fgMuted: "",
    accent: "",
    rowSelected: "",
    onSelected: "#ffffff",
    panel,
    surface: "",
    shadow: "",
    tile: "",
  };
  const under = textBackgrounds(draft, gradientStops(hexes), theme.grain);
  const dark = scheme === "dark";
  // Text: a near-black or near-white with a trace of the theme's hue.
  const ink = { c: Math.min(0.02, base.c * 0.4), h: base.h };
  const fg = readable(under, { ...ink, l: dark ? 0.96 : 0.24 }, dark ? 0.01 : -0.01);
  const fgMuted = readable(under, { ...ink, l: dark ? 0.74 : 0.5 }, dark ? 0.01 : -0.01);
  // The accent: the first pick's hue, more saturated (a grey pick gives a grey accent).
  const accentChroma = (0.035 + 0.115 * theme.intensity) * Math.min(1, (hexToOklch(theme.colors[0] ?? "") ?? base).c / 0.06);
  const tint = { c: accentChroma, h: base.h };
  const accent = readable(under, { ...tint, l: dark ? 0.74 : 0.52 }, dark ? 0.01 : -0.01);
  const white = { r: 1, g: 1, b: 1 };
  const rowSelected = readable([white], { ...tint, l: 0.56 }, -0.01);
  const surface = toHex(over(panel.rgb, panel.alpha, oklchToRgb(base)));
  const shadow = dark ? rgba({ r: 0, g: 0, b: 0 }, 0.45) : rgba(oklchToRgb({ l: 0.25, c: base.c, h: base.h }), 0.16);
  const tile = dark ? oklchToHex({ l: 0.93, c: Math.min(0.012, base.c * 0.3), h: base.h }) : surface;
  return { ...draft, fg, fgMuted, accent, rowSelected, surface, shadow, tile };
}

/** Positions (0–1) of the base gradient's stops: the midpoints are computed in OKLab. */
const BASE_STOPS = [0, 0.25, 0.5, 0.75, 1];

/** The base gradient's sRGB stops: --lj-bg-0 to a point between it and --lj-bg-1. */
function baseStops(bgs: readonly [string, string, string]): string[] {
  const [from, toward] = bgs.map((hex) => oklchToOklab(hexToOklch(hex) ?? { l: 0, c: 0, h: 0 }));
  if (!from || !toward) {
    return [];
  }
  const end = mixOklab(from, toward, 0.4);
  return BASE_STOPS.map((t) => oklchToHex(oklabToOklch(mixOklab(from, end, t))));
}

/** Every opaque color the gradient is made of (text must read on each). */
export function gradientStops(bgs: readonly [string, string, string]): string[] {
  return [...baseStops(bgs), bgs[1], bgs[2]];
}

/** A blob's alpha falls off along a smooth curve, so its edge never shows as a ring. */
const BLOB_FALLOFF = [
  [0, 1],
  [0.2, 0.9],
  [0.4, 0.62],
  [0.6, 0.3],
  [0.8, 0.08],
  [1, 0],
] as const;

function blob(hex: string, strength: number, shape: string): string {
  const rgb = parseHex(hex) ?? { r: 0, g: 0, b: 0 };
  const stops = BLOB_FALLOFF.map(([at, alpha]) => `${rgba(rgb, alpha * strength)} ${Math.round(at * 100)}%`);
  return `radial-gradient(${shape}, ${stops.join(", ")})`;
}

/**
 * The gradient, as a `background-image` list (top layer first): soft radial blobs of --lj-bg-1
 * (top right) and --lj-bg-2 (bottom left) over a base linear gradient whose sRGB stops were
 * computed in OKLab. Deterministic: the same theme always gives the same text.
 */
export function gradientCss(bgs: readonly [string, string, string]): string {
  const base = `linear-gradient(160deg, ${baseStops(bgs)
    .map((hex, index) => `${hex} ${Math.round((BASE_STOPS[index] ?? 0) * 100)}%`)
    .join(", ")})`;
  return [
    blob(bgs[1], 0.9, "120% 95% at 100% 0%"),
    blob(bgs[2], 0.8, "110% 90% at 0% 100%"),
    base,
  ].join(", ");
}

/** The grain tile's opacity for `grain` (0–1). */
export function grainOpacity(grain: number): number {
  return Math.round(clamp01(grain) * GRAIN_OPACITY * 1000) / 1000;
}

/**
 * The grain overlay: a small SVG tile of `feTurbulence` noise, made grey and stretched to full
 * contrast (black to white specks, so it shows at a low opacity), at the grain's opacity, as a
 * data URI (the CSP allows `data:` images). `""` for no grain.
 */
export function grainUri(grain: number): string {
  const opacity = grainOpacity(grain);
  if (opacity === 0) {
    return "";
  }
  const svg =
    "<svg xmlns='http://www.w3.org/2000/svg' width='160' height='160'>" +
    "<filter id='g' x='0' y='0' width='100%' height='100%'>" +
    "<feTurbulence type='fractalNoise' baseFrequency='0.8' numOctaves='2' stitchTiles='stitch'/>" +
    "<feColorMatrix values='3 0 0 0 -1 3 0 0 0 -1 3 0 0 0 -1 0 0 0 0 1'/></filter>" +
    `<rect width='160' height='160' filter='url(#g)' opacity='${opacity}'/></svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}

/** A theme's CSS: what it resolves to and every --lj-* variable. */
export interface ThemeTokens {
  scheme: Scheme;
  vars: Record<string, string>;
}

/** The CSS variables for `theme` (`systemDark`: whether the system is dark now). */
export function tokens(theme: Theme, systemDark: boolean): ThemeTokens {
  const p = palette(theme, systemDark);
  const fg = parseHex(p.fg) ?? { r: 0, g: 0, b: 0 };
  const dark = p.scheme === "dark";
  const grain = grainUri(theme.grain);
  return {
    scheme: p.scheme,
    vars: {
      "--lj-bg-0": p.backgrounds[0],
      "--lj-bg-1": p.backgrounds[1],
      "--lj-bg-2": p.backgrounds[2],
      "--lj-fg": p.fg,
      "--lj-fg-muted": p.fgMuted,
      "--lj-accent": p.accent,
      "--lj-field": rgba(fg, dark ? 0.1 : 0.06),
      "--lj-row-hover": rgba(fg, dark ? 0.08 : 0.05),
      "--lj-row-selected": p.rowSelected,
      "--lj-on-selected": p.onSelected,
      "--lj-panel": rgba(p.panel.rgb, p.panel.alpha),
      "--lj-surface": p.surface,
      "--lj-tile": p.tile,
      "--lj-hairline": rgba(fg, dark ? 0.12 : 0.1),
      "--lj-shadow": p.shadow,
      "--lj-gradient": gradientCss(p.backgrounds),
      "--lj-grain": grain ? `url("${grain}")` : "none",
    },
  };
}
