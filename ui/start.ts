// The start page under the URL bar: a hint, nothing to type into (the bar has the focus).
import { invoke } from "@tauri-apps/api/core";
import { displayShortcut } from "./lib/shortcut.ts";
import type { StartInfo } from "./start-logic.ts";

const hint = document.querySelector<HTMLElement>("#hint");

invoke<StartInfo>("start_info")
  .then((info) => {
    if (hint && info.shortcut) {
      hint.textContent = `Enter to open · Esc to close · ${displayShortcut(info.shortcut, info.platform === "macos")} to summon`;
    }
  })
  .catch(() => {
    // The hint is decoration.
  });
