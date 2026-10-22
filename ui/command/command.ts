// The command bar's list (DOM glue): asks Rust for suggestions as you type (60 ms debounce, only
// the latest answer counts), builds the rows (rows.ts; a quick answer from ui/answers/, our own
// code; extension commands from extensions.ts and the quick commands' state from quick_state,
// both read each time the command bar opens; the quick rows from quick.ts), draws them as text
// (view.ts), and handles the inline autofill, the keys and the mouse. Enter (or a click) on a
// quick answer copies it and says "Copied"; on a quick command it runs it (Clear history asks for
// a second Enter), then the rows show the new state — or, for a page command, the page.
import { invoke } from "@tauri-apps/api/core";
import {
  buildRows,
  initialSelection,
  listKey,
  queryFor,
  showsTyped,
  type Action,
  type RowList,
  type Suggestion,
} from "./rows.ts";
import { renderRows } from "./view.ts";
import { commandRows, isWebPage, type ExtensionInfo, type ExtensionsView } from "./extensions.ts";
import { quickEnter, quickRows, quickStateFrom, type QuickState } from "./quick.ts";
import { quickAnswer } from "../answers/index.ts";
import { clearedText } from "../history/logic.ts";
import type { SiteModeInfo } from "../toolbar/logic.ts";
import { copyText } from "./copy.ts";

/** How long typing must pause before Rust is asked. */
export const DEBOUNCE_MS = 60;

const EMPTY: RowList = { rows: [], autofill: null };

export interface CommandBar {
  /** The command bar opened or closed: start over (asking for the top sites when it opens). */
  reset(open: boolean): void;
  /** The field's text changed (an `input` event). */
  onInput(event: Event): void;
  /** A key in the field; returns whether the list handled it. */
  onKey(event: KeyboardEvent): boolean;
  /** Esc: a quick command waiting for its second Enter stops waiting; returns whether one was. */
  cancel(): boolean;
}

/**
 * `shown` gives the open page's address ("" = none); `showHistory` switches to the history view;
 * `site` gives what the open web page's site is on (Rust's info, for "Request desktop site" & co);
 * `onRows` runs each time rows from an answer are drawn (the summon's timing, LJ19.1).
 */
export function commandBar(
  field: HTMLInputElement,
  list: HTMLElement,
  shown: () => string,
  showHistory: () => void,
  site: () => SiteModeInfo | null = () => null,
  onRows: () => void = () => undefined,
): CommandBar {
  let rows: RowList = EMPTY;
  let selected: number | null = null;
  /** What the user typed, without the inline completion. */
  let typed = "";
  /** The last edit added text (autofill completes only then, never after a deletion). */
  let inserting = false;
  /** The selection to keep for the next answer (after Shift+Delete). */
  let keep: number | null | undefined;
  let latest = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;
  /** The extensions (their commands and page actions), as of the command bar's opening. */
  let extensions: ExtensionInfo[] = [];
  /** The built-in quick answers switched off in Settings. */
  let disabledBuiltIns: string[] = [];
  /** How things are for the quick commands, as of the command bar's opening (or the last one run). */
  let quickState: QuickState | null = null;
  /** The quick row waiting for its second Enter (Clear history), by key. */
  let armed: string | null = null;
  /** What the rows were last built from, to build them again with a new state. */
  let lastQuery = "";
  let lastSuggestions: Suggestion[] = [];

  const loadExtensions = (): void => {
    invoke<ExtensionsView>("extensions_list")
      .then((view) => {
        extensions = view.extensions;
        disabledBuiltIns = view.disabledBuiltIns;
      })
      .catch(() => {
        extensions = [];
      });
  };

  const loadQuick = (then?: () => void): void => {
    invoke<unknown>("quick_state")
      .then((value) => {
        quickState = quickStateFrom(value);
      })
      .catch(() => {
        quickState = null;
      })
      .finally(() => then?.());
  };

  const draw = (): void => {
    list.replaceChildren(...renderRows<HTMLElement>(document, rows.rows, selected));
  };

  const markSelection = (): void => {
    for (const item of Array.from(list.children)) {
      const on = Number((item as HTMLElement).dataset["index"]) === selected;
      item.classList.toggle("selected", on);
      item.setAttribute("aria-selected", on ? "true" : "false");
      if (on) {
        item.scrollIntoView({ block: "nearest" });
      }
    }
  };

  /**
   * Puts the autofill's completion after the typed text, selected, if it still fits; otherwise
   * takes away a completion an earlier answer left (keys can outrun answers: the answer for
   * "12" may land after "7" was typed, and the answer for "127" must still own the field).
   */
  const applyAutofill = (): boolean => {
    const fill = rows.autofill;
    const untouched = showsTyped(field.value, typed, field.selectionStart, field.selectionEnd);
    const fits = fill !== null && fill.text.toLowerCase().startsWith(typed.toLowerCase());
    if (!fill || !inserting || !untouched || !fits) {
      if (untouched && field.value !== typed) {
        field.value = typed;
      }
      return false;
    }
    field.value = typed + fill.text.slice(typed.length);
    field.setSelectionRange(typed.length, field.value.length);
    return true;
  };

  const show = (query: string, suggestions: Suggestion[]): void => {
    lastQuery = query;
    lastSuggestions = suggestions;
    rows = buildRows(query, suggestions, {
      quickAnswer: (typedText) => quickAnswer(typedText, { now: Date.now(), disabled: disabledBuiltIns }),
      commands: (typedText) => commandRows(typedText, extensions, isWebPage(shown())),
      quick: (typedText) => quickRows(typedText, quickState, isWebPage(shown()) ? site() : null, armed),
    });
    if (!applyAutofill()) {
      rows = { ...rows, autofill: null };
    }
    selected = initialSelection(rows);
    if (keep !== undefined) {
      selected = keep === null ? null : Math.min(keep, rows.rows.length - 1);
      keep = undefined;
    }
    document.body.classList.toggle("typing", query.trim() !== "");
    draw();
    onRows();
  };

  const ask = (query: string): void => {
    const mine = ++latest;
    invoke<Suggestion[]>("suggest", { query })
      .then((suggestions) => {
        if (mine === latest) {
          show(query, suggestions);
        }
      })
      .catch(() => {
        if (mine === latest) {
          show(query, []);
        }
      });
  };

  /** Leaves the inline completion when the selection moves off its row. */
  const dropAutofill = (): void => {
    if (rows.autofill && selected !== rows.autofill.row && field.value !== typed) {
      field.value = typed;
    }
  };

  /** The rows again from what they were built from (a new state, a row armed), the selection kept. */
  const redraw = (): void => {
    keep = selected;
    inserting = false;
    show(lastQuery, lastSuggestions);
  };

  /** Stops a quick command waiting for its second Enter; returns whether one was. */
  const disarm = (): boolean => {
    if (armed === null) {
      return false;
    }
    armed = null;
    redraw();
    return true;
  };

  /** Says, in a row for a moment, what happened (as text): the quick answer's, or `selector`'s. */
  const flash = (message: string, selector = ".row-answer .row-domain"): void => {
    const domain = list.querySelector<HTMLElement>(selector);
    if (!domain) {
      return;
    }
    const before = domain.textContent;
    domain.textContent = ` — ${message}`;
    setTimeout(() => {
      if (domain.isConnected) {
        domain.textContent = before;
      }
    }, 1200);
  };

  const act = (action: Action): void => {
    switch (action.kind) {
      case "open":
        void invoke("open_input", { input: action.url });
        break;
      case "input":
        invoke("open_input", { input: action.text }).catch(() => {
          field.select();
        });
        break;
      case "settings":
        void invoke("open_settings", { tab: action.tab });
        break;
      case "history":
        showHistory();
        break;
      case "command":
        void invoke("open_extension", {
          id: action.extension,
          command: action.command,
          pageAction: action.pageAction,
        });
        break;
      case "copy":
        void copyText(action.text, field).then((copied) => {
          if (copied && action.close) {
            // The page's link (LJ42): copied, back to the page.
            void invoke("set_overlay", { on: false });
          } else {
            flash(copied ? "Copied" : "Couldn't copy", action.close ? `[data-index="${selected ?? -1}"] .row-domain` : undefined);
          }
        });
        break;
      case "quick": {
        const step = quickEnter(armed, action);
        armed = step.armed;
        if (!step.run) {
          // The first Enter on Clear history: the row says what a second one removes.
          redraw();
          break;
        }
        const command = action.command;
        invoke<number | null>("quick_command", { command })
          .then((removed) => {
            if (action.closes) {
              void invoke("set_overlay", { on: false });
              return;
            }
            if (command.kind === "reloadExtensions") {
              loadExtensions();
            }
            loadQuick(() => {
              if (command.kind === "clearHistory") {
                // History changed: the matches too.
                keep = selected;
                ask(queryFor(typed, shown()));
                setTimeout(() => flash(clearedText(command.range, removed ?? 0), `[data-index="${keep ?? selected ?? -1}"] .row-domain`), 150);
              } else {
                redraw();
              }
            });
          })
          .catch((err: unknown) => flash(String(err), `[data-index="${selected ?? -1}"] .row-domain`));
        break;
      }
      case "siteMode":
        // Rust applies it to the open page's site (and reloads it if its agent changed); the
        // command bar closes so the page shows.
        void invoke("set_site_mode", { mode: action.mode }).then(() => invoke("set_overlay", { on: false }));
        break;
      case "quit":
        void invoke("quit_app");
        break;
    }
  };

  list.addEventListener("mousedown", (event) => {
    // The keyboard stays in the field.
    event.preventDefault();
  });
  const indexOf = (event: Event): number | null => {
    const item = (event.target as HTMLElement | null)?.closest<HTMLElement>("[data-index]");
    const index = Number(item?.dataset["index"]);
    return item && Number.isInteger(index) ? index : null;
  };
  list.addEventListener("click", (event) => {
    const index = indexOf(event);
    const row = index === null ? undefined : rows.rows[index];
    if (row) {
      act(row.action);
    }
  });
  list.addEventListener("mousemove", (event) => {
    const index = indexOf(event);
    if (index !== null && index !== selected) {
      selected = index;
      dropAutofill();
      if (!disarm()) {
        markSelection();
      }
    }
  });

  return {
    reset(open: boolean) {
      clearTimeout(timer);
      typed = "";
      inserting = false;
      keep = undefined;
      rows = EMPTY;
      selected = null;
      armed = null;
      lastQuery = "";
      lastSuggestions = [];
      document.body.classList.remove("typing");
      if (open) {
        loadExtensions();
        // Typing may beat the state: its rows come again once it is there.
        loadQuick(() => {
          if (lastQuery.trim() !== "") {
            redraw();
          }
        });
        ask("");
      } else {
        latest++;
        draw();
      }
    },

    onInput(event: Event) {
      armed = null;
      typed = field.value;
      inserting = event instanceof InputEvent && event.inputType.startsWith("insert");
      clearTimeout(timer);
      const query = queryFor(typed, shown());
      timer = setTimeout(() => ask(query), DEBOUNCE_MS);
    },

    onKey(event: KeyboardEvent) {
      if (event.isComposing) {
        return false;
      }
      const outcome = listKey(rows, selected, event.key, event.shiftKey, field.value);
      switch (outcome.kind) {
        case "select":
          event.preventDefault();
          selected = outcome.selected;
          dropAutofill();
          if (!disarm()) {
            markSelection();
          }
          return true;
        case "act":
          event.preventDefault();
          act(outcome.action);
          return true;
        case "acceptAutofill":
          event.preventDefault();
          field.value = outcome.text;
          field.setSelectionRange(outcome.text.length, outcome.text.length);
          typed = outcome.text;
          rows = { ...rows, autofill: null };
          return true;
        case "remove":
          event.preventDefault();
          keep = selected;
          invoke("history_remove", { url: outcome.url })
            .catch(() => undefined)
            .finally(() => ask(queryFor(typed, shown())));
          return true;
        case "none":
          return false;
      }
    },

    cancel() {
      return disarm();
    },
  };
}
