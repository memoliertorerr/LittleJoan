// The history view in the command bar's overlay (DOM glue): asks Rust for the pages
// (history_list), filters and groups them (logic.ts), draws them as text (view.ts), and handles
// the search field, the keys, the mouse, removing a page and "Clear history…". A click or Enter
// opens a page through open_input (the overlay then closes); Esc returns to the command bar.
import { invoke } from "@tauri-apps/api/core";
import {
  CLEAR_CHOICES,
  DEFAULT_CLEAR,
  PAGE_SIZE,
  clearedText,
  emptyText,
  filterPages,
  firstRows,
  groupByDay,
  historyKey,
  keepSelection,
  pageAt,
  rowCount,
  type ClearRange,
  type DayGroup,
  type HistoryPage,
} from "./logic.ts";
import { renderHistory } from "./view.ts";

export interface HistoryView {
  /** Shows the view (the overlay is open) and loads the pages; the search field gets the keys. */
  open(): void;
  /** Hides it (back to the command bar, or the overlay closed). */
  close(): void;
  /** A key that reached the document from elsewhere in the bar while the view is shown. */
  onKey(event: KeyboardEvent): void;
}

function child<T extends HTMLElement>(root: HTMLElement, selector: string): T {
  const found = root.querySelector<T>(selector);
  if (!found) {
    throw new Error(`missing ${selector}`);
  }
  return found;
}

/** `back` returns to the command bar (Esc, the back button). */
export function historyView(root: HTMLElement, back: () => void): HistoryView {
  const search = child<HTMLInputElement>(root, "#history-search");
  const list = child<HTMLElement>(root, "#history-rows");
  const scroller = child<HTMLElement>(root, ".history-scroll");
  const empty = child<HTMLElement>(root, "#history-empty");
  const status = child<HTMLElement>(root, "#history-status");
  const clearButton = child<HTMLButtonElement>(root, "#history-clear");
  const clearing = child<HTMLElement>(root, "#history-clearing");
  const ranges = child<HTMLElement>(root, "#history-ranges");
  const confirm = child<HTMLButtonElement>(root, "#history-clear-confirm");
  const cancel = child<HTMLButtonElement>(root, "#history-clear-cancel");
  const backButton = child<HTMLButtonElement>(root, "#history-back");

  let pages: HistoryPage[] = [];
  let shown: DayGroup[] = [];
  let selected: number | null = null;
  let limit = PAGE_SIZE;
  let range: ClearRange = DEFAULT_CLEAR;
  /** Bumped per load, so only the latest answer is drawn. */
  let latest = 0;

  const draw = (): void => {
    const groups = groupByDay(filterPages(pages, search.value), Date.now());
    shown = firstRows(groups, limit);
    selected = keepSelection(selected, rowCount(shown));
    list.replaceChildren(...renderHistory<HTMLElement>(document, shown, selected));
    empty.textContent = emptyText(pages.length, search.value);
    empty.hidden = rowCount(shown) > 0;
  };

  const markSelection = (): void => {
    for (const item of Array.from(list.querySelectorAll<HTMLElement>("[data-index]"))) {
      const on = Number(item.dataset["index"]) === selected;
      item.classList.toggle("selected", on);
      item.setAttribute("aria-selected", on ? "true" : "false");
      if (on) {
        item.scrollIntoView({ block: "nearest" });
      }
    }
  };

  const load = (): void => {
    const mine = ++latest;
    invoke<HistoryPage[]>("history_list")
      .then((answer) => {
        if (mine === latest) {
          pages = answer;
          draw();
        }
      })
      .catch(() => {
        if (mine === latest) {
          pages = [];
          draw();
        }
      });
  };

  const say = (text: string): void => {
    status.textContent = text;
    status.hidden = text.length === 0;
  };

  const openRow = (index: number): void => {
    const page = pageAt(shown, index);
    if (page) {
      void invoke("open_input", { input: page.url });
    }
  };

  const removeRow = (index: number): void => {
    const page = pageAt(shown, index);
    if (!page) {
      return;
    }
    invoke("history_remove", { url: page.url })
      .then(() => say(""))
      .catch((err: unknown) => say(String(err)))
      .finally(load);
  };

  const drawRanges = (): void => {
    ranges.replaceChildren(
      ...CLEAR_CHOICES.map((choice) => {
        const button = document.createElement("button");
        button.type = "button";
        button.className = "choice";
        button.textContent = choice.label;
        button.setAttribute("aria-pressed", choice.range === range ? "true" : "false");
        button.addEventListener("click", () => {
          range = choice.range;
          drawRanges();
        });
        return button;
      }),
    );
  };

  /** Whether the "Clear history…" choices are showing. */
  const clearOpen = (): boolean => clearing.hidden === false;

  const showClearing = (on: boolean): void => {
    clearing.hidden = !on;
    clearButton.setAttribute("aria-expanded", on ? "true" : "false");
    if (on) {
      range = DEFAULT_CLEAR;
      drawRanges();
      say("");
    } else {
      search.focus();
    }
  };

  /** Keys in the search field move, open and remove; elsewhere (a button, the list) only Esc. */
  const onKey = (event: KeyboardEvent, inField: boolean): void => {
    if (event.isComposing || (!inField && event.key !== "Escape")) {
      return;
    }
    const outcome = historyKey(rowCount(shown), selected, event.key, event.shiftKey, clearOpen());
    if (outcome.kind === "none") {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    switch (outcome.kind) {
      case "select":
        selected = outcome.selected;
        markSelection();
        break;
      case "open":
        openRow(outcome.index);
        break;
      case "remove":
        removeRow(outcome.index);
        break;
      case "closeClear":
        showClearing(false);
        break;
      case "back":
        back();
        break;
    }
  };

  search.addEventListener("input", () => {
    selected = null;
    limit = PAGE_SIZE;
    scroller.scrollTop = 0;
    draw();
  });
  root.addEventListener("keydown", (event) => onKey(event, event.target === search));
  backButton.addEventListener("click", back);
  clearButton.addEventListener("click", () => showClearing(!clearOpen()));
  cancel.addEventListener("click", () => showClearing(false));
  confirm.addEventListener("click", () => {
    const chosen = range;
    confirm.disabled = true;
    invoke<number>("history_clear", { range: chosen })
      .then((removed) => {
        showClearing(false);
        say(clearedText(chosen, removed));
      })
      .catch((err: unknown) => say(String(err)))
      .finally(() => {
        confirm.disabled = false;
        load();
      });
  });

  list.addEventListener("mousedown", (event) => {
    // The keyboard stays in the search field.
    event.preventDefault();
  });
  list.addEventListener("click", (event) => {
    const target = event.target as HTMLElement | null;
    const remove = target?.closest<HTMLElement>("[data-remove]");
    if (remove) {
      removeRow(Number(remove.dataset["remove"]));
      return;
    }
    const item = target?.closest<HTMLElement>("[data-index]");
    if (item) {
      openRow(Number(item.dataset["index"]));
    }
  });
  list.addEventListener("mousemove", (event) => {
    const item = (event.target as HTMLElement | null)?.closest<HTMLElement>("[data-index]");
    const index = Number(item?.dataset["index"]);
    if (item && Number.isInteger(index) && index !== selected) {
      selected = index;
      markSelection();
    }
  });
  // More rows once the list is scrolled near its end.
  scroller.addEventListener("scroll", () => {
    const nearEnd = scroller.scrollTop + scroller.clientHeight >= scroller.scrollHeight - 200;
    if (nearEnd && rowCount(shown) >= limit) {
      limit += PAGE_SIZE;
      draw();
    }
  });

  return {
    open() {
      root.hidden = false;
      search.value = "";
      selected = null;
      limit = PAGE_SIZE;
      clearing.hidden = true;
      clearButton.setAttribute("aria-expanded", "false");
      say("");
      pages = [];
      list.replaceChildren();
      empty.hidden = true;
      scroller.scrollTop = 0;
      load();
      search.focus();
    },
    close() {
      latest++;
      root.hidden = true;
      pages = [];
      shown = [];
      list.replaceChildren();
    },
    onKey(event: KeyboardEvent) {
      onKey(event, false);
    },
  };
}
