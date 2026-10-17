// The bar's two views and what keys do in them (pure, unit-tested). The bar is either the 40 px
// toolbar above the page, or an overlay covering the whole window while the command bar is open
// (the page is hidden meanwhile). Rust lays out the webviews and tells this page which view it
// is in; a new window starts with the overlay.

export type View = "toolbar" | "overlay";

export interface BarState {
  view: View;
  /** The address shown for the current page; "" while there is no page (the start page). */
  shown: string;
}

export type KeyOutcome =
  /** Open the trimmed text: an address or a search (Rust decides). */
  | { kind: "open"; input: string }
  /** Put the shown address back (or clear the field when there is no page). */
  | { kind: "revert" }
  /** Close the overlay; the page behind it gets the keyboard. */
  | { kind: "closeOverlay" }
  /** Give the keyboard back to the page (the toolbar). */
  | { kind: "focusPage" }
  /** Close the window: back to hibernation. */
  | { kind: "hibernate" }
  | { kind: "none" };

/** A new window: the command bar is open and no page is loaded yet. */
export function initialState(): BarState {
  return { view: "overlay", shown: "" };
}

/** The state after Rust opened (`true`) or closed the overlay. */
export function withOverlay(state: BarState, overlay: boolean): BarState {
  return { ...state, view: overlay ? "overlay" : "toolbar" };
}

/** The state after the page's address changed (`""` = no page). */
export function withShown(state: BarState, shown: string): BarState {
  return { ...state, shown };
}

/** Whether a page is open behind the bar. */
export function hasPage(state: BarState): boolean {
  return state.shown !== "";
}

/**
 * What a key press does. `value` is the field's text (equal to `state.shown` when nothing was
 * edited, or when the key came from elsewhere in the bar).
 *
 * - Enter opens the trimmed text, not while an input method is composing.
 * - Esc first reverts an edit. With nothing to revert: no page → close the window (Esc on an
 *   empty command bar always closed Little Joan); a page → close the overlay and return to the
 *   page (in the toolbar: just return to the page).
 */
export function onKey(state: BarState, key: string, value: string, composing: boolean): KeyOutcome {
  if (composing) {
    return { kind: "none" };
  }
  if (key === "Enter") {
    const input = value.trim();
    return input.length > 0 ? { kind: "open", input } : { kind: "none" };
  }
  if (key === "Escape") {
    if (value !== state.shown) {
      return { kind: "revert" };
    }
    if (!hasPage(state)) {
      return { kind: "hibernate" };
    }
    return state.view === "overlay" ? { kind: "closeOverlay" } : { kind: "focusPage" };
  }
  return { kind: "none" };
}

/** A press on the field: in the toolbar it opens the command bar; in the overlay it's typing. */
export function onFieldPress(state: BarState): "openOverlay" | "none" {
  return state.view === "toolbar" ? "openOverlay" : "none";
}

/** The × button: like Esc with nothing to revert — back to the page, or with no page, close. */
export function onCloseButton(state: BarState): "closeOverlay" | "hibernate" {
  return hasPage(state) ? "closeOverlay" : "hibernate";
}
