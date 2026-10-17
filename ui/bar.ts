// The bar: the trusted local page above (or over) the web page. In the toolbar view it shows the
// page's address (the app pushes changes with window.littleJoan.setUrl); in the overlay view it
// is the command bar, covering the window: the field at the top with suggestions under it (and
// the welcome on the first run). Rust switches the view with window.littleJoan.setOverlay;
// every summon and Ctrl/Cmd+L open the overlay, opening a page closes it.
import { invoke } from "@tauri-apps/api/core";
import { mayReplace, textFor } from "./bar-logic.ts";
import { commandBar } from "./command/command.ts";
import { fillPanel } from "./command/panel.ts";
import {
  initialState,
  onCloseButton,
  onFieldPress,
  onKey,
  withOverlay,
  withShown,
  type KeyOutcome,
} from "./command/view-state.ts";

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
const rowList = document.querySelector<HTMLElement>("#rows");
const closeButton = document.querySelector<HTMLButtonElement>("#close");

if (field && rowList) {
  let state = initialState();
  const command = commandBar(field, rowList, () => state.shown);

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
        if (state.view === "overlay") {
          command.reset(true);
        }
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
      const changed = (state.view === "overlay") !== on;
      state = withOverlay(state, on);
      render();
      if (changed) {
        if (!on) {
          field.value = state.shown;
        }
        command.reset(on);
      }
    },
    focusField() {
      field.focus();
      field.select();
    },
  };

  field.addEventListener("focus", () => field.select());
  // Leaving the toolbar's field without pressing Enter drops the edit, as browsers do. (In the
  // command bar the text stays until it closes: a click on a row or the welcome mustn't lose it.)
  field.addEventListener("blur", () => {
    if (state.view === "toolbar") {
      field.value = state.shown;
    }
  });
  // A click on the address in the toolbar opens the command bar.
  field.addEventListener("pointerdown", () => {
    if (onFieldPress(state) === "openOverlay") {
      void invoke("set_overlay", { on: true });
    }
  });
  field.addEventListener("input", (event) => {
    if (state.view === "overlay") {
      command.onInput(event);
    }
  });
  field.addEventListener("keydown", (event) => {
    if (state.view === "overlay" && event.key !== "Escape" && command.onKey(event)) {
      return;
    }
    act(onKey(state, event.key, field.value, event.isComposing), event);
  });
  // Keys elsewhere in the bar (after a click in the welcome): Esc follows the same rules.
  document.addEventListener("keydown", (event) => {
    if (event.target !== field && event.key === "Escape") {
      act(onKey(state, event.key, state.shown, event.isComposing), event);
    }
  });
  closeButton?.addEventListener("click", () => {
    if (onCloseButton(state) === "closeOverlay") {
      void invoke("set_overlay", { on: false });
    } else {
      void invoke("hibernate");
    }
  });

  render();
  fillPanel();
  invoke<string | null>("page_url")
    .then(setUrl)
    .catch(() => setUrl(null))
    .finally(() => command.reset(state.view === "overlay"));
  field.focus();
}
