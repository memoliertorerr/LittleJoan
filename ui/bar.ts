// The bar: the trusted local page above (or over) the web page. In the toolbar view it is the
// Little Arc-style toolbar — back, the site's icon and domain, "open in default browser" — for
// the page's address (the app pushes changes with window.littleJoan.setUrl and setCanGoBack); in
// the overlay view it is the command bar, covering the window: the field at the top with
// suggestions under it (and the welcome on the first run), or the history view (Ctrl+H, Cmd+Y on
// macOS, which Rust passes on as window.littleJoan.showHistory, or the command bar's "History" row; Esc
// returns to the command bar). Rust switches the view with window.littleJoan.setOverlay; every
// summon, Ctrl/Cmd+L and a click on the domain open the overlay, opening a page closes it. The
// theme comes with start_info as the bar loads, and window.littleJoan.setTheme changes it (after
// Settings saves one); the bar reports the colors it painted (set_theme_tokens) for extension
// panels. window.littleJoan.setExtension names the extension whose panel the page shows.
import { invoke } from "@tauri-apps/api/core";
import { mayReplace, textFor } from "./bar-logic.ts";
import { commandBar } from "./command/command.ts";
import { fillPanel } from "./command/panel.ts";
import type { StartInfo } from "./command/welcome.ts";
import { applyTheme } from "./lib/apply-theme.ts";
import { initialState, onCloseButton, onKey, withOverlay, withShown, type KeyOutcome } from "./command/view-state.ts";
import { historyView } from "./history/history.ts";
import { extensionLabel, isMac, type ExtensionLabel } from "./toolbar/logic.ts";
import type { ThemeTokens } from "./lib/theme.ts";
import { toolbar } from "./toolbar/toolbar.ts";

declare global {
  interface Window {
    littleJoan?: {
      setUrl(url: string | null): void;
      setOverlay(on: boolean): void;
      setCanGoBack(on: boolean): void;
      focusField(): void;
      showHistory(): void;
      setTheme(theme: unknown): void;
      setExtension(label: unknown): void;
    };
  }
}

const field = document.querySelector<HTMLInputElement>("#url");
const rowList = document.querySelector<HTMLElement>("#rows");
const closeButton = document.querySelector<HTMLButtonElement>("#close");
const toolbarRoot = document.querySelector<HTMLElement>(".toolstrip");
const historyRoot = document.querySelector<HTMLElement>("#history");

// macOS: the traffic lights sit in the bar's top left corner (style.css leaves room for them).
document.documentElement.classList.toggle("os-macos", isMac(navigator.userAgent));

if (field && rowList && toolbarRoot && historyRoot) {
  let state = initialState();
  let canGoBack = false;
  /** The extension whose panel the page shows (the toolbar shows it instead of a domain). */
  let extension: ExtensionLabel | null = null;
  /** What the overlay shows: the command bar, or the history view. */
  let panel: "command" | "history" = "command";

  const render = (): void => {
    document.body.classList.toggle("overlay", state.view === "overlay");
    document.body.classList.toggle("toolbar", state.view === "toolbar");
    document.body.classList.toggle("history", state.view === "overlay" && panel === "history");
  };

  /** The overlay shows the history view (only while it is open). */
  const showHistory = (): void => {
    if (state.view !== "overlay") {
      return;
    }
    panel = "history";
    render();
    historyPanel.open();
  };

  /** Back from the history view to the command bar, the field focused. */
  const showCommand = (): void => {
    if (panel !== "history") {
      return;
    }
    panel = "command";
    historyPanel.close();
    render();
    field.value = state.shown;
    command.reset(state.view === "overlay");
    field.focus();
    field.select();
  };

  const command = commandBar(field, rowList, () => state.shown, showHistory);
  const historyPanel = historyView(historyRoot, showCommand);
  const tools = toolbar(toolbarRoot);

  const setUrl = (url: string | null): void => {
    const next = textFor(url);
    if (mayReplace(field.value, state.shown, document.activeElement === field)) {
      field.value = next;
    }
    state = withShown(state, next);
    tools.update(next, canGoBack, extension);
  };

  /** Extension panels match the bar: Rust keeps the variables it painted (littleJoan.theme). */
  const reportTheme = (tokens: ThemeTokens): void => {
    invoke("set_theme_tokens", { tokens }).catch(() => {
      // Panels then wait for the next report; the bar itself is unaffected.
    });
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
      if (changed && panel === "history") {
        // Opening or closing the overlay starts again at the command bar.
        panel = "command";
        historyPanel.close();
      }
      render();
      if (changed) {
        if (!on) {
          field.value = state.shown;
        }
        command.reset(on);
      }
    },
    setCanGoBack(on: boolean) {
      canGoBack = on;
      tools.update(state.shown, canGoBack, extension);
    },
    focusField() {
      field.focus();
      field.select();
    },
    showHistory,
    setTheme(theme: unknown) {
      applyTheme(theme, reportTheme);
    },
    setExtension(label: unknown) {
      extension = extensionLabel(label);
      tools.update(state.shown, canGoBack, extension);
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
  field.addEventListener("input", (event) => {
    if (state.view === "overlay") {
      command.onInput(event);
    }
  });
  field.addEventListener("keydown", (event) => {
    if (panel === "history") {
      return;
    }
    if (state.view === "overlay" && event.key !== "Escape" && command.onKey(event)) {
      return;
    }
    act(onKey(state, event.key, field.value, event.isComposing), event);
  });
  // Keys elsewhere in the bar (after a click in the welcome or the history list): Esc follows
  // the same rules — in the history view, its own (back to the command bar).
  document.addEventListener("keydown", (event) => {
    if (panel === "history") {
      if (!historyRoot.contains(event.target as Node | null)) {
        historyPanel.onKey(event);
      }
      return;
    }
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
  tools.update(state.shown, canGoBack, extension);
  invoke<StartInfo>("start_info")
    .then((info) => {
      applyTheme(info.theme, reportTheme);
      fillPanel(info);
    })
    .catch(() => {
      // Without the facts there is no welcome and the default theme stays; the bar works the same.
    });
  invoke<string | null>("page_url")
    .then(setUrl)
    .catch(() => setUrl(null))
    .finally(() => command.reset(state.view === "overlay"));
  field.focus();
}
