// Color math for themes (pure, unit-tested): hex ↔ sRGB ↔ OKLab ↔ OKLCH, fitting a color into
// the sRGB gamut, mixing in OKLab, and WCAG contrast. Results are plain sRGB (hex or rgba()):
// WebView2, WKWebView and WebKitGTK differ in CSS color-4 support, so the stylesheets never see
// oklch() or color-mix(). Matrices: Björn Ottosson, "A perceptual color space for image
// processing" (2020).

/** sRGB, gamma-encoded, each channel 0–1 (out of gamut when outside). */
export interface Rgb {
  r: number;
  g: number;
  b: number;
}

export interface Oklab {
  l: number;
  a: number;
  b: number;
}

/** OKLCH: lightness 0–1, chroma (0 = grey), hue in degrees 0–360. */
export interface Oklch {
  l: number;
  c: number;
  h: number;
}

const HEX = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i;

/** `#rgb` or `#rrggbb` (any case) as `#rrggbb` in lower case; `null` for anything else. */
export function normalizeHex(value: unknown): string | null {
  if (typeof value !== "string") {
    return null;
  }
  const match = HEX.exec(value.trim());
  const digits = match?.[1]?.toLowerCase();
  if (!digits) {
    return null;
  }
  return digits.length === 3 ? `#${[...digits].map((d) => d + d).join("")}` : `#${digits}`;
}

export function parseHex(value: string): Rgb | null {
  const hex = normalizeHex(value);
  if (!hex) {
    return null;
  }
  const channel = (at: number) => parseInt(hex.slice(at, at + 2), 16) / 255;
  return { r: channel(1), g: channel(3), b: channel(5) };
}

const clamp01 = (x: number) => Math.min(1, Math.max(0, x));

/** The nearest `#rrggbb` (channels clamped into 0–1 first). */
export function toHex(rgb: Rgb): string {
  const part = (x: number) =>
    Math.round(clamp01(x) * 255)
      .toString(16)
      .padStart(2, "0");
  return `#${part(rgb.r)}${part(rgb.g)}${part(rgb.b)}`;
}

/** `rgba(r, g, b, a)` with 0–255 channels (commas: every engine reads that form). */
export function rgba(rgb: Rgb, alpha: number): string {
  const part = (x: number) => Math.round(clamp01(x) * 255);
  return `rgba(${part(rgb.r)}, ${part(rgb.g)}, ${part(rgb.b)}, ${Math.round(clamp01(alpha) * 1000) / 1000})`;
}

// The sRGB transfer curve, kept odd so out-of-gamut (negative) values survive a round trip.
const toLinear = (x: number) => Math.sign(x) * (Math.abs(x) <= 0.04045 ? Math.abs(x) / 12.92 : ((Math.abs(x) + 0.055) / 1.055) ** 2.4);
const toGamma = (x: number) =>
  Math.sign(x) * (Math.abs(x) <= 0.0031308 ? 12.92 * Math.abs(x) : 1.055 * Math.abs(x) ** (1 / 2.4) - 0.055);

export function rgbToOklab(rgb: Rgb): Oklab {
  const r = toLinear(rgb.r);
  const g = toLinear(rgb.g);
  const b = toLinear(rgb.b);
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  return {
    l: 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    a: 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    b: 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
  };
}

/** The sRGB color for `lab`; channels fall outside 0–1 when sRGB can't show it. */
export function oklabToRgb(lab: Oklab): Rgb {
  const l = (lab.l + 0.3963377774 * lab.a + 0.2158037573 * lab.b) ** 3;
  const m = (lab.l - 0.1055613458 * lab.a - 0.0638541728 * lab.b) ** 3;
  const s = (lab.l - 0.0894841775 * lab.a - 1.291485548 * lab.b) ** 3;
  return {
    r: toGamma(4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s),
    g: toGamma(-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s),
    b: toGamma(-0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s),
  };
}

export function oklabToOklch(lab: Oklab): Oklch {
  const c = Math.hypot(lab.a, lab.b);
  // A grey has no hue; 0 keeps the result stable.
  const h = c < 1e-7 ? 0 : (((Math.atan2(lab.b, lab.a) * 180) / Math.PI) % 360 + 360) % 360;
  return { l: lab.l, c, h };
}

export function oklchToOklab(lch: Oklch): Oklab {
  const angle = (lch.h * Math.PI) / 180;
  return { l: lch.l, a: lch.c * Math.cos(angle), b: lch.c * Math.sin(angle) };
}

export function hexToOklab(hex: string): Oklab | null {
  const rgb = parseHex(hex);
  return rgb ? rgbToOklab(rgb) : null;
}

export function hexToOklch(hex: string): Oklch | null {
  const lab = hexToOklab(hex);
  return lab ? oklabToOklch(lab) : null;
}

export function inGamut(rgb: Rgb): boolean {
  const eps = 1e-6;
  return [rgb.r, rgb.g, rgb.b].every((x) => x >= -eps && x <= 1 + eps);
}

/**
 * `lch` with its chroma lowered (lightness and hue kept) until sRGB can show it: CSS Color 4's
 * idea, without the "just noticeable difference" shortcut, since themes stay muted anyway.
 */
export function fitGamut(lch: Oklch): Oklch {
  const l = clamp01(lch.l);
  const c = Math.max(0, lch.c);
  if (inGamut(oklabToRgb(oklchToOklab({ l, c, h: lch.h })))) {
    return { l, c, h: lch.h };
  }
  let low = 0;
  let high = c;
  for (let step = 0; step < 24; step += 1) {
    const mid = (low + high) / 2;
    if (inGamut(oklabToRgb(oklchToOklab({ l, c: mid, h: lch.h })))) {
      low = mid;
    } else {
      high = mid;
    }
  }
  return { l, c: low, h: lch.h };
}

export function oklchToRgb(lch: Oklch): Rgb {
  return oklabToRgb(oklchToOklab(fitGamut(lch)));
}

/** The sRGB hex for `lch`, fitted into the gamut first. */
export function oklchToHex(lch: Oklch): string {
  return toHex(oklchToRgb(lch));
}

/** The point `t` (0–1) of the way from `from` to `to`, in OKLab (no hue detours, no grey dip). */
export function mixOklab(from: Oklab, to: Oklab, t: number): Oklab {
  return {
    l: from.l + (to.l - from.l) * t,
    a: from.a + (to.a - from.a) * t,
    b: from.b + (to.b - from.b) * t,
  };
}

/** WCAG 2 relative luminance. */
export function luminance(rgb: Rgb): number {
  const lin = (x: number) => toLinear(clamp01(x));
  return 0.2126 * lin(rgb.r) + 0.7152 * lin(rgb.g) + 0.0722 * lin(rgb.b);
}

/** WCAG 2 contrast ratio of two colors, 1–21. */
export function contrastRgb(a: Rgb, b: Rgb): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x) as [number, number];
  return (hi + 0.05) / (lo + 0.05);
}

/** WCAG 2 contrast ratio of two hex colors (an unreadable one counts as black). */
export function contrast(a: string, b: string): number {
  const black = { r: 0, g: 0, b: 0 };
  return contrastRgb(parseHex(a) ?? black, parseHex(b) ?? black);
}

/** `top` drawn over `bottom` at `alpha`, as the engines composite (in gamma-encoded sRGB). */
export function over(top: Rgb, alpha: number, bottom: Rgb): Rgb {
  const mix = (t: number, b: number) => t * alpha + b * (1 - alpha);
  return { r: mix(top.r, bottom.r), g: mix(top.g, bottom.g), b: mix(top.b, bottom.b) };
}
