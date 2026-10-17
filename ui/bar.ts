// The URL bar: a trusted local page pinned above the web page. It always shows the page's
// address (the app pushes changes with window.littleJoan.setUrl); click it or press Ctrl/Cmd+L
// to edit, Enter to go, Esc to revert.
import { invoke } from "@tauri-apps/api/core";
import { barKey, mayReplace, textFor } from "./bar-logic.ts";

declare global {
  interface Window {
    littleJoan?: {
      setUrl(url: string | null): void;
      focusField(): void;
    };
  }
}

const field = document.querySelector<HTMLInputElement>("#url");

if (field) {
  let shown = "";

  const setUrl = (url: string | null): void => {
    const next = textFor(url);
    if (mayReplace(field.value, shown, document.activeElement === field)) {
      field.value = next;
    }
    shown = next;
  };

  window.littleJoan = {
    setUrl,
    focusField() {
      field.focus();
      field.select();
    },
  };

  field.addEventListener("focus", () => field.select());
  // Leaving the bar without pressing Enter drops the edit, as browsers do.
  field.addEventListener("blur", () => {
    field.value = shown;
  });
  field.addEventListener("keydown", (event) => {
    const outcome = barKey(event.key, field.value, shown, event.isComposing);
    switch (outcome.kind) {
      case "open":
        event.preventDefault();
        invoke("open_input", { input: outcome.input }).catch(() => {
          field.select();
        });
        break;
      case "revert":
        event.preventDefault();
        field.value = shown;
        field.select();
        break;
      case "focusPage":
        event.preventDefault();
        void invoke("focus_page");
        break;
      case "hibernate":
        event.preventDefault();
        void invoke("hibernate");
        break;
      case "none":
        break;
    }
  });

  invoke<string | null>("page_url")
    .then(setUrl)
    .catch(() => setUrl(null));
  field.focus();
}
