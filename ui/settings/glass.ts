// Settings → Appearance's Liquid Glass (LJ19.9; DOM glue): three buttons — Off, Clear, Tinted — and
// the note under them (glass-logic.ts says what each does), shown only where Rust says this Mac has
// glass. A choice is a draft like the theme's: previewed in the open page window (`onChange` asks
// the Appearance tab to preview), kept by Save, dropped by Cancel.
import { GLASS_CHOICES, NO_GLASS, glassNote, type GlassSetting, type GlassSupport } from "./glass-logic.ts";

export interface GlassControl {
  /** The choice the window shows now; `undefined` where the setting doesn't show (Save leaves it). */
  draft(): GlassSetting | undefined;
  /** The saved setting arrived, and whether this system has glass. */
  load(setting: GlassSetting, support: GlassSupport): void;
}

export function glassControl(root: HTMLElement, onChange: () => void): GlassControl {
  const buttons = [...root.querySelectorAll<HTMLButtonElement>("[data-glass-choice]")];
  const note = root.querySelector<HTMLElement>("#glass-note");
  let choice: GlassSetting = "off";
  let support: GlassSupport = NO_GLASS;

  function draw(): void {
    root.hidden = !support.available;
    for (const button of buttons) {
      button.setAttribute("aria-pressed", button.dataset.glassChoice === choice ? "true" : "false");
    }
    if (note) {
      note.textContent = glassNote(choice, support);
    }
  }

  for (const button of buttons) {
    button.addEventListener("click", () => {
      const picked = GLASS_CHOICES.find((c) => c.id === button.dataset.glassChoice)?.id;
      if (picked && picked !== choice) {
        choice = picked;
        draw();
        onChange();
      }
    });
  }

  draw();
  return {
    draft: () => (support.available ? choice : undefined),
    load(setting: GlassSetting, next: GlassSupport) {
      choice = setting;
      support = next;
      draw();
    },
  };
}
