// The Appearance tab's logic (pure, unit-tested): color pads (x = hue, y = chroma, a dot per
// color, like Arc's), adding and removing colors (1–3), presets, the mode and the sliders, and the
// draft against the saved theme — the open window previews the draft, Cancel goes back to the
// saved one, Save keeps the draft.
import { hexToOklch, oklchToHex } from "../lib/color.ts";
import { PRESETS, type Preset } from "../lib/presets.ts";
import { MAX_CHROMA, MAX_COLORS, type Theme, type ThemeMode } from "../lib/theme.ts";

/** The pads' lightness (OKLCH L): there every hue reaches PAD_CHROMA inside sRGB. */
export const PAD_LIGHTNESS = 0.75;
/** A pad's top edge: the muted maximum, the themes' chroma cap. */
export const PAD_CHROMA = MAX_CHROMA;

/** A spot on a pad, 0–1 each way: x is the hue (0–360°) left to right, y the chroma, most at the top. */
export interface PadPoint {
  x: number;
  y: number;
}

const clamp01 = (x: number) => Math.min(1, Math.max(0, x));

/** The color at `point` (`#rrggbb`, at the pads' lightness). */
export function colorAt(point: PadPoint): string {
  return oklchToHex({ l: PAD_LIGHTNESS, c: (1 - clamp01(point.y)) * PAD_CHROMA, h: clamp01(point.x) * 360 });
}

/** Where `hex` sits on a pad (chroma above the pad's top sits on its top edge). */
export function pointFor(hex: string): PadPoint {
  const lch = hexToOklch(hex) ?? { l: PAD_LIGHTNESS, c: 0, h: 0 };
  return { x: (((lch.h % 360) + 360) % 360) / 360, y: clamp01(1 - lch.c / PAD_CHROMA) };
}

/** Steps the arrow keys take: 5° of hue or a twentieth of the chroma; with Shift, 30° or a fifth. */
const STEP = { hue: 5 / 360, hueBig: 30 / 360, chroma: 0.05, chromaBig: 0.2 };

/** Where an arrow key moves the dot (hue wraps around, chroma stops at the edges); `null` for other keys. */
export function nudge(point: PadPoint, key: string, big: boolean): PadPoint | null {
  const hue = big ? STEP.hueBig : STEP.hue;
  const chroma = big ? STEP.chromaBig : STEP.chroma;
  const wrap = (x: number) => ((x % 1) + 1) % 1;
  switch (key) {
    case "ArrowLeft":
      return { ...point, x: wrap(point.x - hue) };
    case "ArrowRight":
      return { ...point, x: wrap(point.x + hue) };
    case "ArrowUp":
      return { ...point, y: clamp01(point.y - chroma) };
    case "ArrowDown":
      return { ...point, y: clamp01(point.y + chroma) };
    default:
      return null;
  }
}

/** What a pad's dot says to a screen reader: "hue 210°, color 60 %". */
export function describePoint(point: PadPoint): string {
  return `hue ${Math.round(clamp01(point.x) * 360) % 360}°, color ${Math.round((1 - clamp01(point.y)) * 100)} %`;
}

/** A pad's background: the hues left to right at full pad chroma, fading to grey at the bottom. */
export function padField(): string {
  const hues = Array.from({ length: 13 }, (_, i) => `${colorAt({ x: i / 12, y: 0 })} ${Math.round((i / 12) * 100)}%`);
  const grey = colorAt({ x: 0, y: 1 });
  const rgb = [1, 3, 5].map((at) => parseInt(grey.slice(at, at + 2), 16)).join(", ");
  return `linear-gradient(to bottom, rgba(${rgb}, 0) 0%, rgba(${rgb}, 1) 100%), linear-gradient(to right, ${hues.join(", ")})`;
}

/** A preset's swatch: its colors as a diagonal gradient (one color is a plain circle). */
export function swatch(colors: readonly string[]): string {
  const [first = "#808080", second, third] = colors;
  if (!second) {
    return first;
  }
  return third
    ? `linear-gradient(135deg, ${first} 20%, ${second} 50%, ${third} 80%)`
    : `linear-gradient(135deg, ${first} 25%, ${second} 75%)`;
}

/** A color for "+": the last color's chroma, its hue turned 45°, so it blends. */
export function nextColor(colors: readonly string[]): string {
  const last = pointFor(colors.at(-1) ?? colorAt({ x: 0.6, y: 0.5 }));
  return colorAt({ x: (last.x + 45 / 360) % 1, y: last.y });
}

/** The Appearance tab's state: what is saved, what is shown, which color the pads edit. */
export interface Editor {
  saved: Theme;
  draft: Theme;
  /** The selected color (its pad has the ring, the remove button and the keyboard). */
  selected: number;
}

export type Edit =
  | { kind: "mode"; mode: ThemeMode }
  | { kind: "preset"; id: string }
  | { kind: "select"; index: number }
  | { kind: "move"; index: number; point: PadPoint }
  | { kind: "add" }
  | { kind: "remove"; index: number }
  | { kind: "intensity"; value: number }
  | { kind: "grain"; value: number }
  /** Back to the saved theme. */
  | { kind: "cancel" }
  /** Save stored `theme`: it is the saved one now. */
  | { kind: "saved"; theme: Theme };

function copy(theme: Theme): Theme {
  return { ...theme, colors: [...theme.colors] };
}

export function startEditing(saved: Theme): Editor {
  return { saved: copy(saved), draft: copy(saved), selected: 0 };
}

export function edit(editor: Editor, change: Edit): Editor {
  const { draft } = editor;
  const count = draft.colors.length;
  switch (change.kind) {
    case "mode":
      return { ...editor, draft: { ...draft, mode: change.mode } };
    case "preset": {
      const preset = PRESETS.find((p) => p.id === change.id);
      return preset ? { ...editor, draft: withPreset(draft, preset), selected: 0 } : editor;
    }
    case "select":
      return change.index >= 0 && change.index < count ? { ...editor, selected: change.index } : editor;
    case "move": {
      if (change.index < 0 || change.index >= count) {
        return editor;
      }
      const colors = draft.colors.map((color, i) => (i === change.index ? colorAt(change.point) : color));
      return { ...editor, draft: { ...draft, colors }, selected: change.index };
    }
    case "add":
      return count >= MAX_COLORS
        ? editor
        : { ...editor, draft: { ...draft, colors: [...draft.colors, nextColor(draft.colors)] }, selected: count };
    case "remove": {
      if (count <= 1 || change.index < 0 || change.index >= count) {
        return editor;
      }
      const colors = draft.colors.filter((_, i) => i !== change.index);
      const selected = editor.selected > change.index ? editor.selected - 1 : Math.min(editor.selected, colors.length - 1);
      return { ...editor, draft: { ...draft, colors }, selected };
    }
    case "intensity":
      return { ...editor, draft: { ...draft, intensity: clamp01(change.value) } };
    case "grain":
      return { ...editor, draft: { ...draft, grain: clamp01(change.value) } };
    case "cancel":
      return { ...editor, draft: copy(editor.saved), selected: 0 };
    case "saved":
      return { saved: copy(change.theme), draft: copy(change.theme), selected: Math.min(editor.selected, change.theme.colors.length - 1) };
  }
}

/** A preset's colors, intensity and grain; the mode stays the person's. */
export function withPreset(theme: Theme, preset: Preset): Theme {
  return { mode: theme.mode, colors: [...preset.colors], intensity: preset.intensity, grain: preset.grain };
}

const close = (a: number, b: number) => Math.abs(a - b) < 1e-9;

export function sameTheme(a: Theme, b: Theme): boolean {
  return (
    a.mode === b.mode &&
    close(a.intensity, b.intensity) &&
    close(a.grain, b.grain) &&
    a.colors.length === b.colors.length &&
    a.colors.every((color, i) => color.toLowerCase() === b.colors[i]?.toLowerCase())
  );
}

/** The draft differs from the saved theme (Save has something to keep). */
export function isDirty(editor: Editor): boolean {
  return !sameTheme(editor.saved, editor.draft);
}

/** The preset the theme is (its colors, intensity and grain), if any — its swatch shows pressed. */
export function presetOf(theme: Theme): string | null {
  return PRESETS.find((preset) => sameTheme(withPreset(theme, preset), theme))?.id ?? null;
}

/** A slider's 0–100 value for a 0–1 setting, and back. */
export function toSlider(value: number): number {
  return Math.round(clamp01(value) * 100);
}

export function fromSlider(value: string | number): number {
  const number = typeof value === "number" ? value : Number(value);
  return Number.isFinite(number) ? clamp01(number / 100) : 0;
}
