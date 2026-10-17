// Smoke-log rules for the command bar and its history view (pure, unit-tested;
// scripts/ci/smoke.mts uses them). The app logs `suggestions <rows> first=<site>
// typed=<characters>` for every suggest() call the bar makes, `history-view` when Ctrl/Cmd+Y
// opens the history view and `history-list <rows>` when the view asks for the pages
// (src-tauri/src/commands.rs, bar.rs) — counts and a site, never the typed text or an address.
import type { LogEvent } from "./smoke-log.mts";

export interface SuggestionsEvent {
  /** Rows Rust returned. */
  rows: number;
  /** The first row's site ("-" for none). */
  first: string;
  /** How many characters were typed (0 = the empty command bar). */
  typed: number;
}

/** Reads a `suggestions` event's detail; anything else is undefined. */
export function parseSuggestions(event: LogEvent): SuggestionsEvent | undefined {
  if (event.event !== "suggestions") {
    return undefined;
  }
  const match = /^(\d+) first=(\S+) typed=(\d+)$/.exec(event.detail);
  if (!match?.[1] || !match[2] || !match[3]) {
    return undefined;
  }
  return { rows: Number(match[1]), first: match[2], typed: Number(match[3]) };
}

/** The first of the four starters (src-tauri/src/suggest.rs). */
export const FIRST_STARTER = "wikipedia.org";
/** The smoke page's site, without its port (the server picks one). */
export const SMOKE_SITE = "127.0.0.1";
/** What the smoke test types into the command bar. */
export const TYPED = "127";

/** The empty command bar showed the smoke page's site first (a top site). */
export function sawSmokeTopSite(events: LogEvent[]): boolean {
  return events.some((e) => {
    const s = parseSuggestions(e);
    return s !== undefined && s.typed === 0 && s.first.startsWith(SMOKE_SITE);
  });
}

/** Typing `127` found the smoke page in history first. */
export function sawTypedMatch(events: LogEvent[]): boolean {
  return events.some((e) => {
    const s = parseSuggestions(e);
    return s !== undefined && s.typed === TYPED.length && s.first.startsWith(SMOKE_SITE);
  });
}

export interface CommandBarCheck {
  /** Typing must have reached the command bar (Linux, Windows; macOS is best effort). */
  typing: boolean;
}

/**
 * The main run: before any page was visited the empty command bar offered the starters
 * (`first=wikipedia.org`); after the smoke page was visited, a new window's empty command bar
 * offered its site first; and (where typing is driven) typing `127` found it first.
 */
export function commandBarFailures(events: LogEvent[], check: CommandBarCheck): string[] {
  const failures: string[] = [];
  const visited = events.findIndex((e) => e.event === "page-loaded" && e.detail.startsWith(`http://${SMOKE_SITE}`));
  const before = visited < 0 ? events : events.slice(0, visited);
  const firstEmpty = before.map(parseSuggestions).find((s) => s !== undefined && s.typed === 0);
  if (!firstEmpty) {
    failures.push("the command bar never asked for suggestions before the smoke page opened");
  } else if (firstEmpty.first !== FIRST_STARTER) {
    failures.push(`with no history the command bar offered ${firstEmpty.first} first, not the starters (${FIRST_STARTER})`);
  }
  const after = visited < 0 ? [] : events.slice(visited);
  const later = after.findIndex((e) => e.event === "window-created");
  if (visited < 0 || later < 0 || !sawSmokeTopSite(after.slice(later))) {
    failures.push(`after the smoke page, a new window's command bar never offered ${SMOKE_SITE} as a top site`);
  }
  if (check.typing && !sawTypedMatch(after)) {
    failures.push(`typing "${TYPED}" in the command bar never found ${SMOKE_SITE} first`);
  }
  return failures;
}

/** A `history-list <rows>` event's count (the history view asked for the pages); else undefined. */
export function parseHistoryList(event: LogEvent): number | undefined {
  if (event.event !== "history-list" || !/^\d+$/.test(event.detail)) {
    return undefined;
  }
  return Number(event.detail);
}

/**
 * Ctrl/Cmd+Y reached the app (`history-view`, the page action) and the bar's history view then
 * listed history with at least one page (`history-list <n>`, n ≥ 1).
 */
export function sawHistoryView(events: LogEvent[]): boolean {
  const opened = events.findIndex((e) => e.event === "history-view");
  return opened >= 0 && events.slice(opened).some((e) => (parseHistoryList(e) ?? 0) > 0);
}

export interface HistoryViewCheck {
  /** The keys were driven (Linux, Windows; macOS is best effort). */
  driven: boolean;
}

/** The history view opened with Ctrl/Cmd+Y and listed the smoke page's history. */
export function historyViewFailures(events: LogEvent[], check: HistoryViewCheck): string[] {
  if (!check.driven || sawHistoryView(events)) {
    return [];
  }
  return ["Ctrl/Cmd+Y never opened the history view with the smoke page's history in it"];
}
