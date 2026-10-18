// The Appearance tab (DOM glue): mode buttons, preset swatches, up to three color pads (drag the
// dot, or move it with the arrow keys), intensity and grain sliders, and a live preview of the
// toolbar and the command bar. What each control does comes from appearance-logic.ts. Every
// edit re-themes this window and is previewed in the open page window (`preview_theme`, not
// saved); Save keeps the draft, and closing the window without saving puts the saved theme back.
import { invoke } from "@tauri-apps/api/core";
import { applyTheme } from "../lib/apply-theme.ts";
import { PRESETS } from "../lib/presets.ts";
import { DEFAULT_THEME, type Theme } from "../lib/theme.ts";
import {
  describePoint,
  edit,
  fromSlider,
  nudge,
  padField,
  pointFor,
  presetOf,
  startEditing,
  swatch,
  toSlider,
  type Edit,
  type PadPoint,
} from "./appearance-logic.ts";

export interface Appearance {
  /** The theme the window shows now (what Save keeps). */
  draft(): Theme;
  /** The saved theme arrived (on load, after a save): editing starts again from it. */
  load(theme: Theme): void;
}

function find<T extends HTMLElement>(root: ParentNode, selector: string): T {
  const found = root.querySelector<T>(selector);
  if (!found) {
    throw new Error(`missing ${selector}`);
  }
  return found;
}

const clamp01 = (x: number) => Math.min(1, Math.max(0, x));

export function appearance(root: HTMLElement, onChange: () => void): Appearance {
  let editor = startEditing(DEFAULT_THEME);
  const modes = [...root.querySelectorAll<HTMLButtonElement>("[data-mode]")];
  const presetRow = find<HTMLElement>(root, "#presets");
  const pads = [...root.querySelectorAll<HTMLElement>(".pad")];
  const add = find<HTMLButtonElement>(root, "#pad-add");
  const intensity = find<HTMLInputElement>(root, "#intensity");
  const grain = find<HTMLInputElement>(root, "#grain");

  const presetButtons = PRESETS.map((preset) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "preset";
    button.title = preset.name;
    button.setAttribute("aria-label", `Preset ${preset.name}`);
    button.addEventListener("click", () => change({ kind: "preset", id: preset.id }));
    // The preset's own colors (its pads' dots), so swatches tell apart at a glance.
    button.style.setProperty("--swatch", swatch(preset.colors));
    presetRow.append(button);
    return { preset, button };
  });

  for (const pad of pads) {
    pad.style.setProperty("--pad-field", padField());
  }

  // Previews go to the page window at most once a frame.
  let previewQueued = false;
  function preview(): void {
    if (previewQueued) {
      return;
    }
    previewQueued = true;
    requestAnimationFrame(() => {
      previewQueued = false;
      invoke("preview_theme", { theme: editor.draft }).catch(() => {
        // No page window open: nothing to preview there.
      });
    });
  }

  function draw(): void {
    const { draft, selected } = editor;
    for (const button of modes) {
      button.setAttribute("aria-pressed", button.dataset.mode === draft.mode ? "true" : "false");
    }
    const on = presetOf(draft);
    for (const { button, preset } of presetButtons) {
      button.setAttribute("aria-pressed", preset.id === on ? "true" : "false");
    }
    pads.forEach((pad, index) => {
      const color = draft.colors[index];
      pad.hidden = color === undefined;
      if (color === undefined) {
        return;
      }
      const point = pointFor(color);
      const dot = find<HTMLElement>(pad, ".pad-dot");
      dot.style.setProperty("left", `${point.x * 100}%`);
      dot.style.setProperty("top", `${point.y * 100}%`);
      dot.style.setProperty("background", color);
      pad.classList.toggle("selected", index === selected);
      pad.setAttribute("aria-valuetext", describePoint(point));
      find<HTMLElement>(pad, ".pad-remove").hidden = index !== selected || draft.colors.length <= 1;
    });
    add.hidden = draft.colors.length >= pads.length;
    intensity.value = String(toSlider(draft.intensity));
    grain.value = String(toSlider(draft.grain));
  }

  function change(next: Edit): void {
    const before = editor.draft;
    editor = edit(editor, next);
    draw();
    if (editor.draft !== before) {
      applyTheme(editor.draft);
      preview();
      onChange();
    }
  }

  function pointIn(pad: HTMLElement, event: PointerEvent): PadPoint {
    const box = pad.getBoundingClientRect();
    return {
      x: clamp01((event.clientX - box.left) / Math.max(1, box.width)),
      y: clamp01((event.clientY - box.top) / Math.max(1, box.height)),
    };
  }

  for (const button of modes) {
    button.addEventListener("click", () => {
      const mode = button.dataset.mode;
      if (mode === "system" || mode === "light" || mode === "dark") {
        change({ kind: "mode", mode });
      }
    });
  }

  pads.forEach((pad, index) => {
    pad.addEventListener("pointerdown", (event) => {
      if ((event.target as Element | null)?.closest(".pad-remove")) {
        return;
      }
      event.preventDefault();
      pad.focus();
      pad.setPointerCapture(event.pointerId);
      change({ kind: "move", index, point: pointIn(pad, event) });
    });
    pad.addEventListener("pointermove", (event) => {
      if (pad.hasPointerCapture(event.pointerId)) {
        change({ kind: "move", index, point: pointIn(pad, event) });
      }
    });
    pad.addEventListener("focus", () => change({ kind: "select", index }));
    pad.addEventListener("keydown", (event) => {
      const color = editor.draft.colors[index];
      const moved = color === undefined ? null : nudge(pointFor(color), event.key, event.shiftKey);
      if (moved) {
        event.preventDefault();
        change({ kind: "move", index, point: moved });
      }
    });
    find<HTMLButtonElement>(pad, ".pad-remove").addEventListener("click", () => {
      change({ kind: "remove", index });
      pads[editor.selected]?.focus();
    });
  });

  add.addEventListener("click", () => {
    change({ kind: "add" });
    pads[editor.selected]?.focus();
  });
  intensity.addEventListener("input", () => change({ kind: "intensity", value: fromSlider(intensity.value) }));
  grain.addEventListener("input", () => change({ kind: "grain", value: fromSlider(grain.value) }));

  draw();
  return {
    draft: () => editor.draft,
    load(theme: Theme) {
      editor = startEditing(theme);
      applyTheme(theme);
      draw();
    },
  };
}
