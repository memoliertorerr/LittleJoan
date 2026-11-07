// The toolbar's find strip (DOM glue, LJ49): the field, what was found ("3 of 12", "Found", "No
// matches"), ↑ ↓ and ×, in the toolbar's middle in place of the site while open (the body's
// `finding` class; back and "open in browser" stay). What it does comes from find-logic.ts; what it
// finds, from Rust (the page's engine — the page gets no script). Drawn as text.
import { invoke } from "@tauri-apps/api/core";
import { findStrip, MAX_FIND_QUERY, type FindAction } from "./find-logic.ts";

export interface FindBar {
  /** Ctrl/Cmd+F (Rust): open, the field with the keyboard and its text selected. */
  open(): void;
  /** Rust closed it (a new page, the command bar opened). */
  closed(): void;
  /** Rust's result (anything: checked). */
  setResult(value: unknown): void;
  /** The window rests (LJ19.4): closed and empty for the next summon. */
  rest(): void;
}

export function findBar(root: HTMLElement, body: HTMLElement): FindBar | null {
  const field = root.querySelector<HTMLInputElement>("#find-field");
  const label = root.querySelector<HTMLElement>("#find-count");
  const previous = root.querySelector<HTMLButtonElement>("#find-previous");
  const next = root.querySelector<HTMLButtonElement>("#find-next");
  const close = root.querySelector<HTMLButtonElement>("#find-close");
  if (!field || !label || !previous || !next || !close) {
    return null;
  }
  field.maxLength = MAX_FIND_QUERY;
  const strip = findStrip({
    send: (action: FindAction) => {
      invoke("find", { action }).catch(() => undefined);
    },
    schedule: (run, ms) => setTimeout(run, ms),
    cancel: (timer) => clearTimeout(timer as ReturnType<typeof setTimeout>),
  });

  /** Shows the strip as it is now. */
  const render = (): void => {
    const open = strip.isOpen();
    root.hidden = !open;
    body.classList.toggle("finding", open);
    const text = strip.label();
    label.textContent = text;
    label.classList.toggle("find-none", text === "No matches");
    const stepping = open && strip.query() !== "";
    previous.disabled = !stepping;
    next.disabled = !stepping;
  };

  field.addEventListener("input", () => {
    strip.input(field.value);
    render();
  });
  // Its keys stay here: Esc mustn't reach the bar's own Esc (the URL field's rules).
  root.addEventListener("keydown", (event) => {
    if (event.key !== "Enter" && event.key !== "Escape") {
      return;
    }
    event.stopPropagation();
    let outcome = "none";
    if (event.target === field) {
      outcome = strip.key(event.key, event.shiftKey, event.isComposing);
    } else if (event.key === "Escape") {
      // On a button: Esc closes too (Enter clicks the button).
      strip.close();
      outcome = "close";
    }
    if (outcome !== "none") {
      event.preventDefault();
    }
    render();
  });
  previous.addEventListener("click", () => {
    strip.previous();
    field.focus();
  });
  next.addEventListener("click", () => {
    strip.next();
    field.focus();
  });
  close.addEventListener("click", () => {
    strip.close();
    render();
  });

  return {
    open() {
      strip.open();
      if (field.value !== strip.query()) {
        field.value = strip.query();
      }
      render();
      field.focus();
      field.select();
    },
    closed() {
      strip.closed();
      render();
    },
    setResult(value: unknown) {
      strip.result(value);
      render();
    },
    rest() {
      strip.reset();
      field.value = "";
      render();
    },
  };
}
