// Pure logic of the URL bar, kept apart from the DOM so it can be unit-tested.

export type BarOutcome =
  | { kind: "open"; input: string }
  | { kind: "revert" }
  | { kind: "focusPage" }
  | { kind: "hibernate" }
  | { kind: "none" };

/**
 * What a key press in the bar does. `shown` is the address the bar shows for the current page
 * ("" on the start page).
 *
 * - Enter opens the trimmed text (an address or a search; Rust decides), not while an input
 *   method is composing text.
 * - Esc first reverts an edit; with nothing to revert it hands the keyboard back to the page, or
 *   on the start page closes the window (Esc on an empty field always closed Little Joan).
 */
export function barKey(key: string, value: string, shown: string, composing: boolean): BarOutcome {
  if (composing) {
    return { kind: "none" };
  }
  if (key === "Enter") {
    const input = value.trim();
    return input.length > 0 ? { kind: "open", input } : { kind: "none" };
  }
  if (key === "Escape") {
    if (value !== shown) {
      return { kind: "revert" };
    }
    return shown === "" ? { kind: "hibernate" } : { kind: "focusPage" };
  }
  return { kind: "none" };
}

/**
 * Whether a new address from the app may replace the field's text: always while the user isn't
 * in the field, and while they are, only if they haven't changed what it showed.
 */
export function mayReplace(value: string, shown: string, editing: boolean): boolean {
  return !editing || value === shown;
}

/** The field's text for an address from the app (`null` = the start page). */
export function textFor(url: string | null): string {
  return url ?? "";
}
