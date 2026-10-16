// The start page: one field. Enter opens an address or a search (Rust decides which), Esc
// closes the window (or returns to the page it was opened over).
import { invoke } from "@tauri-apps/api/core";
import { displayShortcut } from "./lib/shortcut.ts";
import { onKey, prefillFrom, type StartInfo } from "./start-logic.ts";

const field = document.querySelector<HTMLInputElement>("#field");
const hint = document.querySelector<HTMLElement>("#hint");
const error = document.querySelector<HTMLElement>("#error");

function showError(message: string): void {
  if (error) {
    error.textContent = message;
    error.hidden = message.length === 0;
  }
}

if (field) {
  field.value = prefillFrom(location.search);
  field.focus();
  field.select();

  field.addEventListener("input", () => showError(""));
  field.addEventListener("keydown", (event) => {
    const outcome = onKey(event.key, field.value, location.search, event.isComposing);
    switch (outcome.kind) {
      case "open":
        event.preventDefault();
        invoke("open_input", { input: outcome.input }).catch((err: unknown) => showError(String(err)));
        break;
      case "back":
        event.preventDefault();
        history.back();
        break;
      case "hibernate":
        event.preventDefault();
        invoke("hibernate").catch((err: unknown) => showError(String(err)));
        break;
      case "none":
        break;
    }
  });
}

invoke<StartInfo>("start_info")
  .then((info) => {
    if (hint) {
      const shortcut = info.shortcut ? ` · ${displayShortcut(info.shortcut, info.platform === "macos")} to summon` : "";
      hint.textContent = `Enter to open · Esc to close${shortcut}`;
    }
  })
  .catch(() => {
    // The hint is decoration; the field works without it.
  });
