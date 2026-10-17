// The history view's logic (pure, unit-tested): which pages a search keeps, how they group by
// local day ("Today", "Yesterday", then dates), the times shown, the "Clear history…" choices,
// and what the keys do. Rust's history_list() supplies the pages; view.ts draws them as text.

/** One page from Rust's `history_list()`: a history record at its latest visit. */
export interface HistoryPage {
  url: string;
  /** From the web page: shown as text, never as HTML. */
  title: string;
  /** The site: lower case, `www.` stripped, with a non-default port. */
  host: string;
  /** Unix milliseconds. */
  lastVisit: number;
  visitCount: number;
}

/** The pages last visited on one local day, latest first. */
export interface DayGroup {
  /** Local midnight that starts the day (unix ms). */
  day: number;
  label: string;
  pages: HistoryPage[];
}

/** What "Clear history…" removes (Rust's `history_clear(range)`): rolling windows from now. */
export type ClearRange = "hour" | "day" | "week" | "all";

export interface ClearChoice {
  range: ClearRange;
  label: string;
  /** For sentences: "from the last hour". */
  phrase: string;
}

export const CLEAR_CHOICES: readonly ClearChoice[] = [
  { range: "hour", label: "Last hour", phrase: "the last hour" },
  { range: "day", label: "Last 24 hours", phrase: "the last 24 hours" },
  { range: "week", label: "Last 7 days", phrase: "the last 7 days" },
  { range: "all", label: "Everything", phrase: "all time" },
];

/** The choice a "Clear history…" panel starts on: the smallest. */
export const DEFAULT_CLEAR: ClearRange = "hour";

/** How many rows are drawn at first, and added each time the list is scrolled to its end. */
export const PAGE_SIZE = 200;

export function clearChoice(range: ClearRange): ClearChoice {
  return CLEAR_CHOICES.find((choice) => choice.range === range) ?? { range, label: range, phrase: range };
}

/** What happened after a clear (Rust says how many pages went). */
export function clearedText(range: ClearRange, removed: number): string {
  if (range === "all") {
    return "History cleared.";
  }
  const { phrase } = clearChoice(range);
  if (removed === 0) {
    return `Nothing to remove from ${phrase}.`;
  }
  return `Removed ${removed} ${removed === 1 ? "page" : "pages"} from ${phrase}.`;
}

/** Local midnight at the start of the day `at` falls on. */
export function dayStart(at: number): number {
  const date = new Date(at);
  return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();
}

/** "Today", "Yesterday", then the date (with the year when it isn't this year's). */
export function dayLabel(day: number, now: number, locale?: string): string {
  const today = new Date(dayStart(now));
  if (day === today.getTime()) {
    return "Today";
  }
  if (day === new Date(today.getFullYear(), today.getMonth(), today.getDate() - 1).getTime()) {
    return "Yesterday";
  }
  const date = new Date(day);
  const options: Intl.DateTimeFormatOptions = { weekday: "long", day: "numeric", month: "long" };
  if (date.getFullYear() !== today.getFullYear()) {
    options.year = "numeric";
  }
  return new Intl.DateTimeFormat(locale, options).format(date);
}

/** The time of a visit, as the system shows times ("14:05" or "2:05 PM"). */
export function timeText(at: number, locale?: string): string {
  return new Intl.DateTimeFormat(locale, { hour: "numeric", minute: "2-digit" }).format(at);
}

/** The pages a search keeps: every word must appear in the title or the address (any case). */
export function filterPages(pages: readonly HistoryPage[], query: string): HistoryPage[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) {
    return [...pages];
  }
  return pages.filter((page) => {
    const text = `${page.title}\n${page.url}`.toLowerCase();
    return words.every((word) => text.includes(word));
  });
}

/** The pages by local day of their latest visit, latest day and latest visit first. */
export function groupByDay(pages: readonly HistoryPage[], now: number, locale?: string): DayGroup[] {
  const sorted = [...pages].sort((a, b) => b.lastVisit - a.lastVisit);
  const groups: DayGroup[] = [];
  for (const page of sorted) {
    const day = dayStart(page.lastVisit);
    const last = groups.at(-1);
    if (last?.day === day) {
      last.pages.push(page);
    } else {
      groups.push({ day, label: dayLabel(day, now, locale), pages: [page] });
    }
  }
  return groups;
}

/** The groups cut to `limit` pages in all (a day cut short keeps its heading). */
export function firstRows(groups: readonly DayGroup[], limit: number): DayGroup[] {
  const kept: DayGroup[] = [];
  let left = limit;
  for (const group of groups) {
    if (left <= 0) {
      break;
    }
    kept.push({ ...group, pages: group.pages.slice(0, left) });
    left -= group.pages.length;
  }
  return kept;
}

export function rowCount(groups: readonly DayGroup[]): number {
  return groups.reduce((sum, group) => sum + group.pages.length, 0);
}

/** The page at row `index`, counting across the days. */
export function pageAt(groups: readonly DayGroup[], index: number): HistoryPage | undefined {
  let left = index;
  for (const group of groups) {
    if (left < group.pages.length) {
      return group.pages[left];
    }
    left -= group.pages.length;
  }
  return undefined;
}

/** A row's text: the title (or the site when there is none), then the site when it differs. */
export function rowText(page: HistoryPage): { title: string; domain: string } {
  const title = page.title.trim();
  return { title: title || page.host, domain: title && title !== page.host ? page.host : "" };
}

/** What the list says when it has no rows. */
export function emptyText(total: number, query: string): string {
  return total === 0 ? "No history yet." : `Nothing in history matches “${query.trim()}”.`;
}

export type HistoryKeyOutcome =
  | { kind: "select"; selected: number | null }
  | { kind: "open"; index: number }
  | { kind: "remove"; index: number }
  /** Close the "Clear history…" choices. */
  | { kind: "closeClear" }
  /** Back to the command bar. */
  | { kind: "back" }
  | { kind: "none" };

/**
 * What a key does in the history view, over `count` rows. ↑/↓ move without wrapping (↑ from
 * the first row goes back to the search field); Enter opens the selected row, or the first one;
 * Shift+Delete removes the selected page from history; Esc closes the "Clear history…" choices,
 * else returns to the command bar.
 */
export function historyKey(
  count: number,
  selected: number | null,
  key: string,
  shift: boolean,
  clearing: boolean,
): HistoryKeyOutcome {
  switch (key) {
    case "Escape":
      return clearing ? { kind: "closeClear" } : { kind: "back" };
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
    case "Enter":
      if (selected !== null && selected < count) {
        return { kind: "open", index: selected };
      }
      return count > 0 ? { kind: "open", index: 0 } : { kind: "none" };
    case "Delete":
      return shift && selected !== null && selected < count ? { kind: "remove", index: selected } : { kind: "none" };
    default:
      return { kind: "none" };
  }
}

/** The selection after the list changed to `count` rows (a removed last row moves it up). */
export function keepSelection(selected: number | null, count: number): number | null {
  if (selected === null || count === 0) {
    return null;
  }
  return Math.min(selected, count - 1);
}
