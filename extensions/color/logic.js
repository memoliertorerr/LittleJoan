// @ts-check
// Color tools, pure: reading a color as CSS writes it (hex, rgb(), hsl(), oklch(), a few names),
// converting between hex / RGB / HSL / OKLCH, and WCAG contrast with the levels it passes.
// OKLab per Björn Ottosson's published matrices; sRGB transfer per IEC 61966-2-1.

/** @typedef {{ r: number, g: number, b: number, a: number }} Rgb r, g, b 0–255 (may be fractional), a 0–1 */
/** @typedef {{ h: number, s: number, l: number }} Hsl h 0–360, s and l 0–100 */
/** @typedef {{ l: number, c: number, h: number }} Oklch l 0–1, c ≥ 0, h 0–360 */

/** The CSS names people type most (the 16 basic colors and a few more). */
const NAMES = {
  black: "#000000",
  white: "#ffffff",
  red: "#ff0000",
  lime: "#00ff00",
  green: "#008000",
  blue: "#0000ff",
  yellow: "#ffff00",
  cyan: "#00ffff",
  aqua: "#00ffff",
  magenta: "#ff00ff",
  fuchsia: "#ff00ff",
  silver: "#c0c0c0",
  gray: "#808080",
  grey: "#808080",
  maroon: "#800000",
  olive: "#808000",
  purple: "#800080",
  teal: "#008080",
  navy: "#000080",
  orange: "#ffa500",
  pink: "#ffc0cb",
  brown: "#a52a2a",
  rebeccapurple: "#663399",
};

const clamp = (/** @type {number} */ v, /** @type {number} */ lo, /** @type {number} */ hi) => Math.min(hi, Math.max(lo, v));

/**
 * A number with a unit for an alpha or a channel: `50%` → 0.5 of `scale`.
 * @param {string} token
 * @param {number} scale what 100% is
 * @returns {number | null}
 */
function amount(token, scale) {
  const match = /^([+-]?(?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?)(%?)$/i.exec(token);
  if (!match) {
    return null;
  }
  const n = Number(match[1]);
  return match[2] ? (n / 100) * scale : n;
}

/**
 * An angle in degrees (`deg`, `turn`, `rad`, `grad` or none).
 * @param {string} token
 * @returns {number | null}
 */
function angle(token) {
  const match = /^([+-]?(?:\d+\.?\d*|\.\d+))(deg|turn|rad|grad)?$/i.exec(token);
  if (!match) {
    return token.toLowerCase() === "none" ? 0 : null;
  }
  const n = Number(match[1]);
  const unit = (match[2] ?? "deg").toLowerCase();
  const degrees = unit === "turn" ? n * 360 : unit === "rad" ? (n * 180) / Math.PI : unit === "grad" ? n * 0.9 : n;
  return ((degrees % 360) + 360) % 360;
}

/**
 * The arguments of `name(…)`: three values and an optional alpha, with commas or spaces and `/`.
 * @param {string} inner
 * @returns {{ parts: string[], alpha: string | null } | null}
 */
function args(inner) {
  const [main, alpha, extra] = inner.split("/").map((p) => p.trim());
  if (extra !== undefined || !main) {
    return null;
  }
  const parts = main.split(/[\s,]+/).filter(Boolean);
  if (parts.length === 4 && alpha === undefined && main.includes(",")) {
    return { parts: parts.slice(0, 3), alpha: parts[3] ?? null };
  }
  return parts.length === 3 ? { parts, alpha: alpha ?? null } : null;
}

/**
 * A color as CSS writes it, or null.
 * @param {string} text
 * @returns {Rgb | null}
 */
export function parse(text) {
  const input = text.trim().toLowerCase();
  const named = /** @type {Record<string, string>} */ (NAMES)[input];
  if (named) {
    return parse(named);
  }
  const hex = /^#?([0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/.exec(input);
  if (hex?.[1]) {
    const digits = hex[1].length <= 4 ? [...hex[1]].map((d) => d + d).join("") : hex[1];
    const n = (/** @type {number} */ i) => parseInt(digits.slice(i, i + 2), 16);
    return { r: n(0), g: n(2), b: n(4), a: digits.length === 8 ? Math.round((n(6) / 255) * 1000) / 1000 : 1 };
  }
  const call = /^(rgba?|hsla?|oklch)\((.*)\)$/.exec(input);
  if (!call?.[1] || call[2] === undefined) {
    return null;
  }
  const parsed = args(call[2]);
  if (!parsed) {
    return null;
  }
  const alpha = parsed.alpha === null ? 1 : amount(parsed.alpha, 1);
  if (alpha === null) {
    return null;
  }
  const [x, y, z] = parsed.parts;
  if (x === undefined || y === undefined || z === undefined) {
    return null;
  }
  const a = clamp(alpha, 0, 1);
  if (call[1].startsWith("rgb")) {
    const [r, g, b] = [x, y, z].map((t) => amount(t, 255));
    return r === null || g === null || b === null || r === undefined || g === undefined || b === undefined ? null : { r: clamp(r, 0, 255), g: clamp(g, 0, 255), b: clamp(b, 0, 255), a };
  }
  if (call[1].startsWith("hsl")) {
    const h = angle(x);
    const s = amount(y.endsWith("%") ? y : `${y}%`, 100);
    const l = amount(z.endsWith("%") ? z : `${z}%`, 100);
    return h === null || s === null || l === null ? null : { ...fromHsl({ h, s: clamp(s, 0, 100), l: clamp(l, 0, 100) }), a };
  }
  const l = amount(x, 1);
  const c = amount(y, 0.4);
  const h = angle(z);
  return l === null || c === null || h === null ? null : { ...fromOklch({ l: clamp(l, 0, 1), c: Math.max(0, c), h }), a };
}

/** @param {number} v */
const hex2 = (v) => Math.round(clamp(v, 0, 255)).toString(16).padStart(2, "0");

/**
 * `#rrggbb`, or `#rrggbbaa` when the color isn't opaque.
 * @param {Rgb} color
 */
export function toHex(color) {
  return `#${hex2(color.r)}${hex2(color.g)}${hex2(color.b)}${color.a < 1 ? hex2(color.a * 255) : ""}`;
}

/**
 * A number for CSS: at most `digits` decimals, no trailing zeros.
 * @param {number} n
 * @param {number} digits
 */
const short = (n, digits) => String(Number(n.toFixed(digits)));

/**
 * `rgb(255 136 0)` (with ` / 0.5` when not opaque).
 * @param {Rgb} color
 */
export function toRgbString(color) {
  const alpha = color.a < 1 ? ` / ${short(color.a, 3)}` : "";
  return `rgb(${Math.round(color.r)} ${Math.round(color.g)} ${Math.round(color.b)}${alpha})`;
}

/**
 * RGB → HSL.
 * @param {Rgb} color
 * @returns {Hsl}
 */
export function toHsl(color) {
  const [r, g, b] = [color.r / 255, color.g / 255, color.b / 255];
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  const d = max - min;
  if (d === 0) {
    return { h: 0, s: 0, l: l * 100 };
  }
  const s = d / (1 - Math.abs(2 * l - 1));
  let h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  h = ((h * 60) % 360 + 360) % 360;
  return { h, s: s * 100, l: l * 100 };
}

/**
 * HSL → RGB (opaque).
 * @param {Hsl} hsl
 * @returns {Rgb}
 */
export function fromHsl({ h, s, l }) {
  const sat = s / 100;
  const light = l / 100;
  const k = (/** @type {number} */ n) => (n + h / 30) % 12;
  const a = sat * Math.min(light, 1 - light);
  const f = (/** @type {number} */ n) => light - a * Math.max(-1, Math.min(k(n) - 3, 9 - k(n), 1));
  return { r: f(0) * 255, g: f(8) * 255, b: f(4) * 255, a: 1 };
}

/**
 * `hsl(210 50% 40%)`.
 * @param {Rgb} color
 */
export function toHslString(color) {
  const { h, s, l } = toHsl(color);
  const alpha = color.a < 1 ? ` / ${short(color.a, 3)}` : "";
  return `hsl(${short(h, 1)} ${short(s, 1)}% ${short(l, 1)}%${alpha})`;
}

/** @param {number} v 0–1 sRGB → linear */
const toLinear = (v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
/** @param {number} v linear → 0–1 sRGB */
const fromLinear = (v) => (v <= 0.0031308 ? 12.92 * v : 1.055 * v ** (1 / 2.4) - 0.055);

/**
 * RGB → OKLCH.
 * @param {Rgb} color
 * @returns {Oklch}
 */
export function toOklch(color) {
  const [r, g, b] = [color.r, color.g, color.b].map((v) => toLinear(v / 255));
  const lr = r ?? 0;
  const lg = g ?? 0;
  const lb = b ?? 0;
  const l = Math.cbrt(0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb);
  const m = Math.cbrt(0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb);
  const s = Math.cbrt(0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb);
  const L = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
  const A = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
  const B = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
  const C = Math.hypot(A, B);
  const H = C < 1e-4 ? 0 : ((Math.atan2(B, A) * 180) / Math.PI + 360) % 360;
  return { l: L, c: C < 1e-4 ? 0 : C, h: H };
}

/**
 * OKLCH → linear sRGB (may be out of 0–1).
 * @param {Oklch} lch
 */
function oklchToLinear({ l: L, c, h }) {
  const A = c * Math.cos((h * Math.PI) / 180);
  const B = c * Math.sin((h * Math.PI) / 180);
  const l = (L + 0.3963377774 * A + 0.2158037573 * B) ** 3;
  const m = (L - 0.1055613458 * A - 0.0638541728 * B) ** 3;
  const s = (L - 0.0894841775 * A - 1.291485548 * B) ** 3;
  return [4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s, -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s, -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s];
}

/**
 * OKLCH → RGB (opaque); a color outside sRGB keeps its lightness and hue and loses chroma until
 * it fits.
 * @param {Oklch} lch
 * @returns {Rgb}
 */
export function fromOklch(lch) {
  const fits = (/** @type {number[]} */ rgb) => rgb.every((v) => v >= -1e-6 && v <= 1 + 1e-6);
  let linear = oklchToLinear(lch);
  if (!fits(linear)) {
    let lo = 0;
    let hi = lch.c;
    for (let i = 0; i < 30; i += 1) {
      const mid = (lo + hi) / 2;
      if (fits(oklchToLinear({ ...lch, c: mid }))) {
        lo = mid;
      } else {
        hi = mid;
      }
    }
    linear = oklchToLinear({ ...lch, c: lo });
  }
  const [r, g, b] = linear.map((v) => clamp(fromLinear(clamp(v, 0, 1)), 0, 1) * 255);
  return { r: r ?? 0, g: g ?? 0, b: b ?? 0, a: 1 };
}

/**
 * `oklch(0.628 0.258 29.23)`.
 * @param {Rgb} color
 */
export function toOklchString(color) {
  const { l, c, h } = toOklch(color);
  const alpha = color.a < 1 ? ` / ${short(color.a, 3)}` : "";
  return `oklch(${short(l, 3)} ${short(c, 3)} ${c === 0 ? "0" : short(h, 2)}${alpha})`;
}

/**
 * Relative luminance (WCAG).
 * @param {Rgb} color
 */
export function luminance(color) {
  const [r, g, b] = [color.r, color.g, color.b].map((v) => toLinear(clamp(v, 0, 255) / 255));
  return 0.2126 * (r ?? 0) + 0.7152 * (g ?? 0) + 0.0722 * (b ?? 0);
}

/**
 * `top` painted over `bottom` (alpha compositing; the result is opaque when `bottom` is).
 * @param {Rgb} top
 * @param {Rgb} bottom
 * @returns {Rgb}
 */
export function over(top, bottom) {
  const a = top.a + bottom.a * (1 - top.a);
  if (a === 0) {
    return { r: 0, g: 0, b: 0, a: 0 };
  }
  const mix = (/** @type {number} */ t, /** @type {number} */ b) => (t * top.a + b * bottom.a * (1 - top.a)) / a;
  return { r: mix(top.r, bottom.r), g: mix(top.g, bottom.g), b: mix(top.b, bottom.b), a };
}

/**
 * WCAG contrast of text `fg` on `bg` (1–21). A see-through background sits on white; see-through
 * text is painted over its background first.
 * @param {Rgb} fg
 * @param {Rgb} bg
 */
export function contrast(fg, bg) {
  const back = over(bg, { r: 255, g: 255, b: 255, a: 1 });
  const front = over(fg, back);
  const [hi, lo] = [luminance(front), luminance(back)].sort((x, y) => y - x);
  return ((hi ?? 0) + 0.05) / ((lo ?? 0) + 0.05);
}

/**
 * The WCAG 2 levels a contrast ratio passes: AA (4.5:1; 3:1 for large text), AAA (7:1; 4.5:1).
 * @param {number} ratio
 */
export function levels(ratio) {
  return { aa: ratio >= 4.5, aaLarge: ratio >= 3, aaa: ratio >= 7, aaaLarge: ratio >= 4.5 };
}

/**
 * "4.48:1" — a ratio as WCAG tools show it (rounded down to two decimals, so a 4.499 doesn't read
 * as passing 4.5).
 * @param {number} ratio
 */
export function ratioText(ratio) {
  return `${(Math.floor(ratio * 100) / 100).toFixed(2)}:1`;
}

/**
 * Black or white, whichever reads better on `color`.
 * @param {Rgb} color
 */
export function textOn(color) {
  const white = { r: 255, g: 255, b: 255, a: 1 };
  const black = { r: 0, g: 0, b: 0, a: 1 };
  return contrast(white, color) >= contrast(black, color) ? "#ffffff" : "#000000";
}
