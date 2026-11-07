// The toolbar's find strip (LJ49), its logic (pure, unit-tested): Ctrl/Cmd+F — a keymap action Rust
// catches natively — opens it in the toolbar's middle, in place of the site (window.littleJoan.openFind).
// Typing asks Rust to find (the `find` command, 120 ms after the last key); Enter finds the next
// match, Shift+Enter the previous, Esc closes the strip and gives the page the keyboard. Rust
// finds with the page's engine and pushes what it found (window.littleJoan.setFindResult): "3 of
// 12" where the engine counts (Windows, Linux), "Found" where it only says found or not (macOS),
// "No matches", or that this version of Windows' web engine has no find. find.ts draws it.

/** How long after the last key a query goes to Rust. */
export const FIND_DEBOUNCE_MS = 120;
/** The longest query (Rust cuts longer ones there too). */
export const MAX_FIND_QUERY = 1_000;
/** What the strip says when WebView2's runtime has no find. */
export const FIND_UNAVAILABLE = "Find isn't available in this version of Windows' web engine";

/** What the bar asks Rust for (the `find` command). */
export type FindAction = { kind: "start"; query: string } | { kind: "next" } | { kind: "previous" } | { kind: "stop" };

/** What Rust says the engine found. */
export interface FindResult {
  /** How many matches; null where the engine doesn't count (macOS). */
  matches: number | null;
  /** The active match, from 1; null where the engine doesn't say. */
  active: number | null;
  found: boolean;
  /** WebView2's runtime has no find. */
  unavailable: boolean;
}

const count = (value: unknown): number | null | undefined =>
  value === null || value === undefined ? null : typeof value === "number" && Number.isInteger(value) && value >= 0 ? value : undefined;

/** Rust's result (anything: checked) — or null for anything else. */
export function findResult(value: unknown): FindResult | null {
  if (typeof value !== "object" || value === null) {
    return null;
  }
  const raw = value as Record<string, unknown>;
  const matches = count(raw.matches);
  const active = count(raw.active);
  if (matches === undefined || active === undefined || typeof raw.found !== "boolean") {
    return null;
  }
  return { matches, active, found: raw.found, unavailable: raw.unavailable === true };
}

/** What the strip says next to the field (as text): nothing until the engine has said something. */
export function findLabel(result: FindResult | null, query: string): string {
  if (query === "" || result === null) {
    return "";
  }
  if (result.unavailable) {
    return FIND_UNAVAILABLE;
  }
  if (result.matches !== null) {
    if (result.matches === 0 || !result.found) {
      return "No matches";
    }
    return result.active !== null ? `${result.active} of ${result.matches}` : `${result.matches} ${result.matches === 1 ? "match" : "matches"}`;
  }
  return result.found ? "Found" : "No matches";
}

/** What the strip needs from its page: Rust, and a clock. */
export interface FindDeps {
  send(action: FindAction): void;
  schedule(run: () => void, ms: number): unknown;
  cancel(timer: unknown): void;
}

/** What a key in the strip did. */
export type FindKeyOutcome = "next" | "previous" | "close" | "none";

export interface FindStrip {
  isOpen(): boolean;
  query(): string;
  /** What the strip says now. */
  label(): string;
  /** Ctrl/Cmd+F (Rust): open — finding the last query again, as browsers do — or open already. */
  open(): void;
  /** The field's text changed: it goes to Rust FIND_DEBOUNCE_MS after the last change. */
  input(text: string): void;
  /** A key in the field: Enter, Shift+Enter, Esc (none while an input method composes). */
  key(key: string, shift: boolean, composing: boolean): FindKeyOutcome;
  /** The ↓ and ↑ buttons. */
  next(): void;
  previous(): void;
  /** Esc or ×: the strip closes, Rust ends the find and gives the page the keyboard. */
  close(): void;
  /** Rust closed it (a new page, the command bar opened): nothing to send. */
  closed(): void;
  /** The window rests (LJ19.4): closed and empty, nothing to send. */
  reset(): void;
  /** Rust's result (anything: checked); dropped while the strip is closed. */
  result(value: unknown): void;
}

export function findStrip(deps: FindDeps): FindStrip {
  let open = false;
  let query = "";
  let result: FindResult | null = null;
  /** A query typed but not sent yet. */
  let pending: unknown = null;

  const cancelPending = (): void => {
    if (pending !== null) {
      deps.cancel(pending);
      pending = null;
    }
  };
  const sendQuery = (): void => {
    pending = null;
    deps.send({ kind: "start", query });
  };
  /** Next or previous — or, with a query still waiting, that query now (its first match). */
  const step = (kind: "next" | "previous"): void => {
    if (!open || query === "") {
      return;
    }
    if (pending !== null) {
      cancelPending();
      sendQuery();
      return;
    }
    deps.send({ kind });
  };
  const close = (): void => {
    if (!open) {
      return;
    }
    cancelPending();
    open = false;
    result = null;
    deps.send({ kind: "stop" });
  };

  return {
    isOpen: () => open,
    query: () => query,
    label: () => findLabel(result, query),
    open() {
      if (open) {
        return;
      }
      open = true;
      result = null;
      if (query !== "") {
        sendQuery();
      }
    },
    input(text: string) {
      if (!open) {
        return;
      }
      const next = Array.from(text).slice(0, MAX_FIND_QUERY).join("");
      if (next === query) {
        return;
      }
      query = next;
      result = null;
      cancelPending();
      if (query === "") {
        // Clears the find at once (the strip stays).
        deps.send({ kind: "start", query: "" });
        return;
      }
      pending = deps.schedule(sendQuery, FIND_DEBOUNCE_MS);
    },
    key(key: string, shift: boolean, composing: boolean): FindKeyOutcome {
      if (composing || !open) {
        return "none";
      }
      if (key === "Enter") {
        const kind = shift ? "previous" : "next";
        step(kind);
        return kind;
      }
      if (key === "Escape") {
        close();
        return "close";
      }
      return "none";
    },
    next: () => step("next"),
    previous: () => step("previous"),
    close,
    closed() {
      cancelPending();
      open = false;
      result = null;
    },
    reset() {
      cancelPending();
      open = false;
      result = null;
      query = "";
    },
    result(value: unknown) {
      if (open && query !== "") {
        result = findResult(value);
      }
    },
  };
}
