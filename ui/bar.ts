// The bar: the trusted local page above (or over) the web page. In the toolbar view it shows the
// page's address (the app pushes changes with window.littleJoan.setUrl); in the overlay view it
// is the command bar, covering the window: the field at the top, the welcome on the first run
// below it. Rust switches the view with window.littleJoan.setOverlay; every summon and Ctrl/Cmd+L
// open the overlay, opening a page closes it.
import { invoke } from "@tauri-apps/api/core";
import { mayReplace, textFor } from "./bar-logic.ts";
import { fillPanel } from "./command/panel.ts";
import { initialState, onFieldPress, onKey, withOverlay, withShown, type KeyOutcome } from "./command/view-state.ts";

declare global {
  interface Window {
    littleJoan?: {
      setUrl(url: string | null): void;
      setOverlay(on: boolean): void;
      focusField(): void;
    };
  }
}

const field = document.querySelector<HTMLInputElement>("#url");

if (field) {
  let state = initialState();

  const render = (): void => {
    document.body.classList.toggle("overlay", state.view === "overlay");
    document.body.classList.toggle("toolbar", state.view === "toolbar");
  };

  const setUrl = (url: string | null): void => {
    const next = textFor(url);
    if (mayReplace(field.value, state.shown, document.activeElement === field)) {
      field.value = next;
    }
    state = withShown(state, next);
  };

  const act = (outcome: KeyOutcome, event: KeyboardEvent): void => {
    switch (outcome.kind) {
      case "open":
        event.preventDefault();
        invoke("open_input", { input: outcome.input }).catch(() => {
          field.select();
        });
        break;
      case "revert":
        event.preventDefault();
        field.value = state.shown;
        field.select();
        break;
      case "closeOverlay":
        event.preventDefault();
        void invoke("set_overlay", { on: false });
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
  };

  window.littleJoan = {
    setUrl,
    setOverlay(on: boolean) {
      state = withOverlay(state, on);
      render();
    },
    focusField() {
      field.focus();
      field.select();
    },
  };

  field.addEventListener("focus", () => field.select());
  // Leaving the field without pressing Enter drops the edit, as browsers do.
  field.addEventListener("blur", () => {
    field.value = state.shown;
  });
  // A click on the address in the toolbar opens the command bar.
  field.addEventListener("pointerdown", () => {
    if (onFieldPress(state) === "openOverlay") {
      void invoke("set_overlay", { on: true });
    }
  });
  field.addEventListener("keydown", (event) => {
    act(onKey(state, event.key, field.value, event.isComposing), event);
  });
  // Keys elsewhere in the bar (after a click in the welcome): Esc follows the same rules.
  document.addEventListener("keydown", (event) => {
    if (event.target !== field && event.key === "Escape") {
      act(onKey(state, event.key, state.shown, event.isComposing), event);
    }
  });

  render();
  fillPanel();
  invoke<string | null>("page_url")
    .then(setUrl)
    .catch(() => setUrl(null));
  field.focus();
}
