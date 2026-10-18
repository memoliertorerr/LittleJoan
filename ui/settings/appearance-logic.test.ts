import { describe, expect, it } from "vitest";
import { fitGamut, hexToOklch } from "../lib/color.ts";
import { PRESETS } from "../lib/presets.ts";
import { DEFAULT_THEME, minimumContrast, type Theme } from "../lib/theme.ts";
import {
  PAD_CHROMA,
  PAD_LIGHTNESS,
  colorAt,
  describePoint,
  edit,
  fromSlider,
  isDirty,
  nextColor,
  nudge,
  padField,
  pointFor,
  presetOf,
  sameTheme,
  swatch,
  startEditing,
  toSlider,
  withPreset,
} from "./appearance-logic.ts";

const saved: Theme = { mode: "system", colors: ["#9d8fd0", "#e0a2a0"], intensity: 0.6, grain: 0.4 };

describe("color pads", () => {
  it("are hue across and chroma up, at a lightness where every hue fits", () => {
    expect(PAD_LIGHTNESS).toBe(0.75);
    expect(PAD_CHROMA).toBe(0.12);
    for (let hue = 0; hue < 360; hue += 5) {
      expect(fitGamut({ l: PAD_LIGHTNESS, c: PAD_CHROMA, h: hue }).c).toBe(PAD_CHROMA);
    }
    const top = hexToOklch(colorAt({ x: 0.5, y: 0 }));
    expect(top?.c ?? 0).toBeCloseTo(PAD_CHROMA, 2);
    expect(top?.h ?? 0).toBeCloseTo(180, 0);
    expect(hexToOklch(colorAt({ x: 0.5, y: 1 }))?.c ?? 1).toBeLessThan(0.002);
  });

  it("round-trip pad coordinates ↔ hue/chroma within the muted range", () => {
    for (let x = 0; x < 1; x += 0.05) {
      for (let y = 0; y <= 0.85; y += 0.05) {
        const back = pointFor(colorAt({ x, y }));
        // Hue wraps around (359° is next to 0°). Rounding to #rrggbb moves a color by about
        // 0.001 in OKLab, which costs more hue the greyer it is: about 0.07 / chroma degrees.
        const dx = Math.min(Math.abs(back.x - x), 1 - Math.abs(back.x - x));
        const chroma = (1 - y) * PAD_CHROMA;
        expect(dx * 360, `hue at ${x},${y}`).toBeLessThan(Math.max(1, 0.08 / chroma));
        expect(Math.abs(back.y - y), `chroma at ${x},${y}`).toBeLessThan(0.03);
      }
    }
    // A color more saturated than the pad sits on its top edge.
    expect(pointFor("#ff0000").y).toBe(0);
    expect(pointFor("#808080").x).toBe(0);
    expect(pointFor("#808080").y).toBeCloseTo(1, 5);
  });

  it("move with the arrow keys: hue wraps, chroma stops at the edges", () => {
    expect(nudge({ x: 0.5, y: 0.5 }, "ArrowRight", false)?.x).toBeCloseTo(0.5 + 5 / 360, 9);
    expect(nudge({ x: 0.5, y: 0.5 }, "ArrowLeft", true)?.x).toBeCloseTo(0.5 - 30 / 360, 9);
    expect(nudge({ x: 0.01, y: 0.5 }, "ArrowLeft", false)?.x).toBeCloseTo(1 + 0.01 - 5 / 360, 9);
    expect(nudge({ x: 0.5, y: 0.02 }, "ArrowUp", false)?.y).toBe(0);
    expect(nudge({ x: 0.5, y: 0.9 }, "ArrowDown", true)?.y).toBe(1);
    expect(nudge({ x: 0.5, y: 0.5 }, "ArrowUp", false)?.y).toBeCloseTo(0.45, 9);
    expect(nudge({ x: 0.5, y: 0.5 }, "Enter", false)).toBeNull();
    expect(describePoint({ x: 0.5, y: 0.4 })).toBe("hue 180°, color 60 %");
  });

  it("draw hues across and a fade to grey, in plain sRGB", () => {
    const field = padField();
    expect(field).toMatch(/^linear-gradient\(to bottom, rgba\(\d+, \d+, \d+, 0\) 0%, rgba\(\d+, \d+, \d+, 1\) 100%\), linear-gradient\(to right, /);
    expect(field.match(/#[0-9a-f]{6} \d+%/g)).toHaveLength(13);
    expect(field).not.toMatch(/oklch|color-mix/);
  });
});

describe("adding and removing colors", () => {
  it("adds up to three, each new one turned 45° from the last", () => {
    let editor = startEditing({ ...saved, colors: ["#9d8fd0"] });
    editor = edit(editor, { kind: "add" });
    expect(editor.draft.colors).toHaveLength(2);
    expect(editor.selected).toBe(1);
    const turn = ((pointFor(editor.draft.colors[1] ?? "").x - pointFor("#9d8fd0").x + 1) % 1) * 360;
    expect(turn).toBeGreaterThan(43);
    expect(turn).toBeLessThan(47);
    editor = edit(editor, { kind: "add" });
    expect(editor.draft.colors).toHaveLength(3);
    expect(edit(editor, { kind: "add" })).toBe(editor);
    expect(nextColor([])).toMatch(/^#[0-9a-f]{6}$/);
  });

  it("removes down to one, keeping a sensible selection", () => {
    let editor = startEditing({ ...saved, colors: ["#111111", "#222222", "#333333"] });
    editor = edit(editor, { kind: "select", index: 2 });
    editor = edit(editor, { kind: "remove", index: 0 });
    expect(editor.draft.colors).toEqual(["#222222", "#333333"]);
    expect(editor.selected).toBe(1);
    editor = edit(editor, { kind: "remove", index: 1 });
    expect(editor.draft.colors).toEqual(["#222222"]);
    expect(editor.selected).toBe(0);
    expect(edit(editor, { kind: "remove", index: 0 })).toBe(editor);
    expect(edit(editor, { kind: "select", index: 4 })).toBe(editor);
  });

  it("moves the selected color's dot", () => {
    const editor = edit(startEditing(saved), { kind: "move", index: 1, point: { x: 0.25, y: 0.2 } });
    expect(editor.draft.colors[1]).toBe(colorAt({ x: 0.25, y: 0.2 }));
    expect(editor.draft.colors[0]).toBe(saved.colors[0]);
    expect(editor.selected).toBe(1);
    expect(edit(editor, { kind: "move", index: 3, point: { x: 0, y: 0 } })).toBe(editor);
  });
});

describe("presets", () => {
  it("are at least ten, and every one passes the contrast test in light and dark", () => {
    expect(PRESETS.length).toBeGreaterThanOrEqual(10);
    for (const preset of PRESETS) {
      for (const mode of ["light", "dark"] as const) {
        const theme = withPreset({ ...DEFAULT_THEME, mode }, preset);
        expect(minimumContrast(theme, false), `${preset.id} ${mode}`).toBeGreaterThanOrEqual(4.5);
      }
    }
  });

  it("apply in one click, keeping the mode, and show which one is on", () => {
    const editor = edit(startEditing({ ...saved, mode: "dark" }), { kind: "preset", id: "glacier" });
    expect(editor.draft.mode).toBe("dark");
    expect(editor.draft.colors).toEqual([...(PRESETS.find((p) => p.id === "glacier")?.colors ?? [])]);
    expect(presetOf(editor.draft)).toBe("glacier");
    expect(presetOf(saved)).toBeNull();
    expect(presetOf(DEFAULT_THEME)).toBe("mist");
    expect(edit(editor, { kind: "preset", id: "nope" })).toBe(editor);
    expect(swatch(["#111111"])).toBe("#111111");
    expect(swatch(["#111111", "#222222"])).toBe("linear-gradient(135deg, #111111 25%, #222222 75%)");
    expect(swatch(["#111111", "#222222", "#333333"])).toContain("#333333 80%");
  });
});

describe("preview and saved", () => {
  it("previews every edit, and Cancel restores the saved theme", () => {
    let editor = startEditing(saved);
    expect(isDirty(editor)).toBe(false);
    editor = edit(editor, { kind: "mode", mode: "light" });
    editor = edit(editor, { kind: "intensity", value: 0.9 });
    editor = edit(editor, { kind: "grain", value: 1.4 });
    expect(editor.draft).toMatchObject({ mode: "light", intensity: 0.9, grain: 1 });
    expect(isDirty(editor)).toBe(true);
    expect(sameTheme(editor.saved, saved)).toBe(true);
    const cancelled = edit(editor, { kind: "cancel" });
    expect(cancelled.draft).toEqual(saved);
    expect(isDirty(cancelled)).toBe(false);
  });

  it("makes a saved draft the new saved theme", () => {
    let editor = edit(startEditing(saved), { kind: "preset", id: "sage" });
    editor = edit(editor, { kind: "saved", theme: editor.draft });
    expect(isDirty(editor)).toBe(false);
    expect(presetOf(editor.saved)).toBe("sage");
    // Edits never touch the saved theme they started from.
    const before = startEditing(saved);
    edit(before, { kind: "move", index: 0, point: { x: 0, y: 0 } });
    expect(before.saved).toEqual(saved);
  });

  it("maps sliders to 0–1", () => {
    expect(toSlider(0.456)).toBe(46);
    expect(fromSlider("46")).toBeCloseTo(0.46, 9);
    expect(fromSlider("abc")).toBe(0);
    expect(fromSlider(250)).toBe(1);
  });
});
