// The command bar's rows (pure, unit-tested): what to show for what is typed, in what order, the
// inline autofill, and what the keys do. Rust's suggest() supplies the suggestions
// (src-tauri/src/suggest.rs); view.ts draws the rows, as text only.

/** One suggestion from Rust's `suggest(query)`. */
export interface Suggestion {
  kind: "topSite" | "history" | "search" | "starter";
  /** A page title; for `search`, the engine's name. Shown as text, never as HTML. */
  title: string;
  url: string;
  /** The site: lower case, `www.` stripped, with a non-default port. */
  host: string;
  score: number;
}

export type RowKind = "answer" | "topSite" | "history" | "search" | "starter" | "command" | "settings";

/** What choosing a row does. */
export type Action =
  /** Open this address (through open_input, so the visit counts as typed). */
  | { kind: "open"; url: string }
  /** What open_input makes of the text: an address or a search. */
  | { kind: "input"; text: string }
  /** Open the settings window on a tab. */
  | { kind: "settings"; tab: string }
  /** Show the history view (the built-in "History" command). */
  | { kind: "history" }
  /** An extension's command, or a page action for the open web page (extensions.ts). */
  | { kind: "command"; extension: string; command: string; pageAction: boolean }
  /** A quick answer's text, to copy (LJ31). */
  | { kind: "copy"; text: string };

export interface Row {
  kind: RowKind;
  /** The main text. */
  title: string;
  /** Muted after the title ("Title — domain"); "" for none. */
  domain: string;
  /** Whose favicon to show; "" = the row kind's own icon. */
  host: string;
  /** A built-in symbol instead of the kind's icon (style.css: `row-icon-<icon>`). */
  icon?: string;
  /** An extension's own icon (a checked data: URI), drawn over the symbol. */
  image?: string;
  action: Action;
  /** The history record Shift+Delete removes, if this row is one. */
  removable?: string;
}

/** Hooks for rows that don't come from history: quick answers (LJ31), extension commands (LJ30). */
export interface Hooks {
  quickAnswer?: (query: string) => Row | null;
  commands?: (query: string) => Row[];
}

/** The inline completion in the field: the full text it shows, and the row it belongs to. */
export interface Autofill {
  text: string;
  row: number;
}

export interface RowList {
  rows: Row[];
  autofill: Autofill | null;
}

/** The most suggestion rows; "Settings & extensions" comes after them. */
export const MAX_SUGGESTIONS = 8;

/** Always the last row — where Little Arc has "Contact the Team": Settings, on Extensions. */
export const SETTINGS_ROW: Row = {
  kind: "settings",
  title: "Settings & extensions",
  domain: "",
  host: "",
  action: { kind: "settings", tab: "extensions" },
};

/** The built-in "History" command: it opens the history view (also Ctrl/Cmd+Y). */
export const HISTORY_ROW: Row = {
  kind: "command",
  title: "History",
  domain: "",
  host: "",
  icon: "clock",
  action: { kind: "history" },
};

const HISTORY_WORD = "history";

/**
 * The built-in commands for what is typed: "History" when the text starts the word (two letters
 * or more: "hi", "hist") or contains it ("history", "clear history").
 */
export function builtInCommands(query: string): Row[] {
  const typed = query.trim().toLowerCase();
  const starts = typed.length >= 2 && HISTORY_WORD.startsWith(typed);
  const names = typed.split(/\s+/).includes(HISTORY_WORD);
  return starts || names ? [HISTORY_ROW] : [];
}

function rowFor(suggestion: Suggestion): Row {
  const title = suggestion.title.trim();
  const row: Row = {
    kind: suggestion.kind,
    title: title || suggestion.host,
    domain: title && title !== suggestion.host ? suggestion.host : "",
    host: suggestion.host,
    action: { kind: "open", url: suggestion.url },
  };
  if (suggestion.kind === "history") {
    row.removable = suggestion.url;
  }
  return row;
}

function searchRow(query: string, engine: Suggestion | undefined): Row {
  const name = engine?.title.trim() ?? "";
  return {
    kind: "search",
    title: name ? `Search ${name} for “${query}”` : `Search for “${query}”`,
    domain: "",
    host: engine?.host ?? "",
    action: engine ? { kind: "open", url: engine.url } : { kind: "input", text: query },
  };
}

/** An address as typed: no scheme, no `www.`, no lone trailing slash. */
export function typedForm(url: string): string {
  const bare = url.replace(/^https?:\/\//i, "").replace(/^www\./i, "");
  return /^[^/]+\/$/.test(bare) ? bare.slice(0, -1) : bare;
}

/**
 * The inline autofill for `query`: only when the best history match's site starts with what was
 * typed (a host prefix) and its score is at least the average of the matches. It completes the
 * match's address as typed (`typedForm`), keeping the user's own characters.
 */
export function autofillFor(query: string, history: readonly Suggestion[]): string | null {
  const typed = query.trim();
  const best = history[0];
  if (!best || typed.length === 0 || /\s/.test(typed)) {
    return null;
  }
  const lower = typed.toLowerCase();
  const form = typedForm(best.url);
  if (!best.host.startsWith(lower) || !form.toLowerCase().startsWith(lower)) {
    return null;
  }
  const average = history.reduce((sum, row) => sum + row.score, 0) / history.length;
  if (best.score < average) {
    return null;
  }
  return typed + form.slice(typed.length);
}

/**
 * The rows for `query`. Nothing typed: the top sites (or the starters), up to 8. Typing: a quick
 * answer if one applies, the best history match, "Search <engine> for …", more history matches,
 * matching commands (the built-in ones, which always find room, then extensions') — at most 8 in
 * all. "Settings & extensions" is always last.
 */
export function buildRows(query: string, suggestions: readonly Suggestion[], hooks: Hooks = {}): RowList {
  const typed = query.trim();
  if (typed.length === 0) {
    const sites = suggestions.filter((s) => s.kind === "topSite" || s.kind === "starter");
    return { rows: [...sites.slice(0, MAX_SUGGESTIONS).map(rowFor), SETTINGS_ROW], autofill: null };
  }
  const history = suggestions.filter((s) => s.kind === "history");
  const engine = suggestions.find((s) => s.kind === "search");
  const rows: Row[] = [];
  const answer = hooks.quickAnswer?.(typed) ?? null;
  if (answer) {
    rows.push(answer);
  }
  const [best, ...more] = history;
  const bestIndex = best ? rows.length : -1;
  if (best) {
    rows.push(rowFor(best));
  }
  rows.push(searchRow(typed, engine));
  const builtIn = builtInCommands(typed);
  rows.push(...more.slice(0, Math.max(0, MAX_SUGGESTIONS - rows.length - builtIn.length)).map(rowFor));
  rows.push(...builtIn);
  rows.push(...(hooks.commands?.(typed) ?? []));
  const shown = rows.slice(0, MAX_SUGGESTIONS);
  const text = autofillFor(typed, history);
  return {
    rows: [...shown, SETTINGS_ROW],
    autofill: text !== null && bestIndex >= 0 ? { text, row: bestIndex } : null,
  };
}

/**
 * What the command bar searches for: the field's text — or nothing while it still shows the
 * open page's address untouched (Ctrl/Cmd+L), which means "show the top sites".
 */
export function queryFor(value: string, shown: string): string {
  return value === shown ? "" : value;
}

/**
 * Whether the field shows what was typed and nothing of the user's own after it: the typed text
 * alone with the caret at its end, or followed by an inline completion that is still selected
 * (so typing replaces it). `start` / `end` are the field's selection. Only then may a new
 * autofill go in — also when the answer for the previous keystroke already completed it.
 */
export function showsTyped(value: string, typed: string, start: number | null, end: number | null): boolean {
  if (end !== value.length) {
    return false;
  }
  if (value === typed) {
    return start === value.length;
  }
  return value.startsWith(typed) && start === typed.length;
}

/** The row selected when a list appears: the autofill's row, else none (Enter = the field). */
export function initialSelection(list: RowList): number | null {
  return list.autofill?.row ?? null;
}

export type ListOutcome =
  | { kind: "select"; selected: number | null }
  | { kind: "act"; action: Action }
  | { kind: "acceptAutofill"; text: string }
  | { kind: "remove"; url: string }
  | { kind: "none" };

/**
 * What a key does to the list. ↑/↓ move without wrapping (↑ from the first row goes back to the
 * field); Enter opens the selected row, or with none what the field says (open_input: an address
 * or a search); Tab accepts the autofill; Shift+Delete removes the selected history row.
 */
export function listKey(
  list: RowList,
  selected: number | null,
  key: string,
  shift: boolean,
  fieldText: string,
): ListOutcome {
  const count = list.rows.length;
  switch (key) {
    case "ArrowDown":
      if (count === 0) {
        return { kind: "none" };
      }
      return { kind: "select", selected: selected === null ? 0 : Math.min(selected + 1, count - 1) };
    case "ArrowUp":
      if (selected === null) {
        return { kind: "none" };
      }
      return { kind: "select", selected: selected === 0 ? null : selected - 1 };
    case "Enter": {
      const row = selected === null ? undefined : list.rows[selected];
      if (row) {
        return { kind: "act", action: row.action };
      }
      const text = fieldText.trim();
      return text ? { kind: "act", action: { kind: "input", text } } : { kind: "none" };
    }
    case "Tab":
      return list.autofill ? { kind: "acceptAutofill", text: list.autofill.text } : { kind: "none" };
    case "Delete": {
      const url = selected === null ? undefined : list.rows[selected]?.removable;
      return shift && url ? { kind: "remove", url } : { kind: "none" };
    }
    default:
      return { kind: "none" };
  }
}

/** The letter of a site's stand-in icon (its first letter or digit), upper case. */
export function letterFor(host: string): string {
  const match = /[\p{L}\p{N}]/u.exec(host);
  return match ? match[0].toUpperCase() : "•";
}

/** A stable hue (0–359) for a site's stand-in icon, so each site keeps its tint. */
export function hueFor(host: string): number {
  let hash = 0;
  for (const char of host) {
    hash = (hash * 31 + (char.codePointAt(0) ?? 0)) % 360;
  }
  return hash;
}

/** Where a site's favicon is fetched from: straight from the site, over HTTPS. */
export function faviconUrl(host: string): string | null {
  return /^[a-z0-9.-]+(:\d{1,5})?$/i.test(host) ? `https://${host}/favicon.ico` : null;
}
