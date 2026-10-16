// Pure logic of the start page, kept apart from the DOM so it can be unit-tested.

export type KeyOutcome =
  | { kind: "open"; input: string }
  | { kind: "back" }
  | { kind: "hibernate" }
  | { kind: "none" };

/** The address to pre-fill, from the page's own `?u=` query (set by Ctrl/Cmd+L on a page). */
export function prefillFrom(search: string): string {
  return new URLSearchParams(search).get("u") ?? "";
}

/**
 * What a key press in the field does. Enter opens the trimmed input (not while an input method
 * is composing text); Esc goes back to the page the user came from (when the start page was
 * opened over one with Ctrl/Cmd+L), otherwise closes the window.
 */
export function onKey(key: string, value: string, search: string, composing: boolean): KeyOutcome {
  if (composing) {
    return { kind: "none" };
  }
  if (key === "Enter") {
    const input = value.trim();
    return input.length > 0 ? { kind: "open", input } : { kind: "none" };
  }
  if (key === "Escape") {
    return new URLSearchParams(search).has("u") ? { kind: "back" } : { kind: "hibernate" };
  }
  return { kind: "none" };
}

export interface StartInfo {
  shortcut: string | null;
  platform: string;
}
