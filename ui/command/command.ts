// The command bar's list (DOM glue): asks Rust for suggestions as you type (60 ms debounce, only
// the latest answer counts), builds the rows (rows.ts), draws them as text (view.ts), and
// handles the inline autofill, the keys and the mouse.
import { invoke } from "@tauri-apps/api/core";
import { buildRows, initialSelection, listKey, queryFor, type Action, type RowList, type Suggestion } from "./rows.ts";
import { renderRows } from "./view.ts";

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
}

/** `shown` gives the open page's address ("" = none). */
export function commandBar(field: HTMLInputElement, list: HTMLElement, shown: () => string): CommandBar {
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

  /** Puts the autofill's completion after the typed text, selected, if it still fits. */
  const applyAutofill = (): boolean => {
    const fill = rows.autofill;
    const caretAtEnd = field.selectionStart === field.value.length && field.selectionEnd === field.value.length;
    if (!fill || !inserting || field.value !== typed || !caretAtEnd) {
      return false;
    }
    field.value = fill.text;
    field.setSelectionRange(typed.length, fill.text.length);
    return true;
  };

  const show = (query: string, suggestions: Suggestion[]): void => {
    rows = buildRows(query, suggestions);
    if (applyAutofill()) {
      selected = initialSelection(rows);
    } else {
      rows = { ...rows, autofill: null };
      selected = null;
    }
    if (keep !== undefined) {
      selected = keep === null ? null : Math.min(keep, rows.rows.length - 1);
      keep = undefined;
    }
    document.body.classList.toggle("typing", query.trim() !== "");
    draw();
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
        void invoke("open_settings", {});
        break;
      case "command":
      case "copy":
        // Extension commands (LJ30) and quick answers (LJ31) come later.
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
      markSelection();
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
      document.body.classList.remove("typing");
      if (open) {
        ask("");
      } else {
        latest++;
        draw();
      }
    },

    onInput(event: Event) {
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
          markSelection();
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
  };
}
