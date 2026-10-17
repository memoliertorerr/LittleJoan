import { describe, expect, it } from "vitest";
import {
  CLEAR_CHOICES,
  DEFAULT_CLEAR,
  clearChoice,
  clearedText,
  dayLabel,
  dayStart,
  emptyText,
  filterPages,
  firstRows,
  groupByDay,
  historyKey,
  keepSelection,
  pageAt,
  rowCount,
  rowText,
  timeText,
  type HistoryPage,
} from "./logic.ts";

/** Local times, so the tests hold in any time zone. */
const at = (day: number, hour: number, minute = 0) => new Date(2026, 9, day, hour, minute).getTime();
const NOW = at(18, 15, 30);

function page(url: string, lastVisit: number, title = "Title"): HistoryPage {
  return { url, title, host: new URL(url).host.replace(/^www\./, ""), lastVisit, visitCount: 1 };
}

const SMOKE = page("http://127.0.0.1:5000/smoke.html", at(18, 15, 2), "LJ-SMOKE ready");
const MORNING = page("https://news.test/", at(18, 8));
const LATE = page("https://late.test/a", at(17, 23, 59), "Late night");
const EARLY = page("https://early.test/", at(17, 0, 1), "Early");
const OLDER = page("https://older.test/", at(12, 10), "Older");
const LAST_YEAR = page("https://ago.test/", new Date(2025, 11, 31, 12).getTime(), "Ago");

describe("days", () => {
  it("start at local midnight", () => {
    expect(dayStart(at(18, 15, 30))).toBe(new Date(2026, 9, 18).getTime());
    expect(dayStart(at(18, 0, 0))).toBe(new Date(2026, 9, 18).getTime());
    expect(dayStart(at(17, 23, 59))).toBe(new Date(2026, 9, 17).getTime());
  });

  it("are Today, Yesterday, then dates (with the year when it isn't this one)", () => {
    expect(dayLabel(dayStart(at(18, 1)), NOW)).toBe("Today");
    expect(dayLabel(dayStart(at(17, 23)), NOW)).toBe("Yesterday");
    const options = { weekday: "long", day: "numeric", month: "long" } as const;
    expect(dayLabel(dayStart(at(12, 9)), NOW, "en-US")).toBe(
      new Intl.DateTimeFormat("en-US", options).format(new Date(2026, 9, 12)),
    );
    expect(dayLabel(dayStart(at(12, 9)), NOW, "en-US")).toContain("October");
    const ago = dayStart(LAST_YEAR.lastVisit);
    expect(dayLabel(ago, NOW, "en-US")).toBe(
      new Intl.DateTimeFormat("en-US", { ...options, year: "numeric" }).format(new Date(2025, 11, 31)),
    );
    expect(dayLabel(ago, NOW, "en-US")).toContain("2025");
  });

  it("cross month starts: the 1st's yesterday is the last of the month before", () => {
    const first = new Date(2026, 10, 1, 9).getTime();
    expect(dayLabel(dayStart(new Date(2026, 9, 31, 22).getTime()), first)).toBe("Yesterday");
  });

  it("show the time of the visit", () => {
    expect(timeText(at(18, 14, 5), "en-GB")).toBe("14:05");
    expect(timeText(at(18, 14, 5), "en-US")).toMatch(/^2:05\s?PM$/);
  });
});

describe("grouping by day", () => {
  it("puts each page under the day of its latest visit, latest first", () => {
    const groups = groupByDay([OLDER, EARLY, SMOKE, LATE, MORNING], NOW, "en-US");
    expect(groups.map((g) => g.label.split(",")[0])).toEqual(["Today", "Yesterday", "Monday"]);
    expect(groups[0]?.pages).toEqual([SMOKE, MORNING]);
    expect(groups[1]?.pages).toEqual([LATE, EARLY]);
    expect(groups[2]?.pages).toEqual([OLDER]);
    expect(groups[0]?.day).toBe(dayStart(NOW));
  });

  it("is empty for no pages", () => {
    expect(groupByDay([], NOW)).toEqual([]);
  });

  it("cuts long lists, keeping the headings of the days it reaches", () => {
    const groups = groupByDay([SMOKE, MORNING, LATE, EARLY, OLDER], NOW);
    const cut = firstRows(groups, 3);
    expect(cut.map((g) => g.pages.length)).toEqual([2, 1]);
    expect(rowCount(cut)).toBe(3);
    expect(firstRows(groups, 100)).toEqual(groups);
    expect(firstRows(groups, 0)).toEqual([]);
  });

  it("counts rows across the days", () => {
    const groups = groupByDay([SMOKE, MORNING, LATE, OLDER], NOW);
    expect(pageAt(groups, 0)).toBe(SMOKE);
    expect(pageAt(groups, 2)).toBe(LATE);
    expect(pageAt(groups, 3)).toBe(OLDER);
    expect(pageAt(groups, 4)).toBeUndefined();
  });
});

describe("searching", () => {
  const pages = [SMOKE, MORNING, LATE, OLDER];

  it("keeps the pages whose title or address has every word, in any case", () => {
    expect(filterPages(pages, "smoke")).toEqual([SMOKE]);
    expect(filterPages(pages, "LJ-SMOKE 127")).toEqual([SMOKE]);
    expect(filterPages(pages, "late night")).toEqual([LATE]);
    expect(filterPages(pages, "night news")).toEqual([]);
    expect(filterPages(pages, ".test")).toEqual([MORNING, LATE, OLDER]);
  });

  it("keeps everything for an empty search", () => {
    expect(filterPages(pages, "   ")).toEqual(pages);
  });

  it("says why the list is empty", () => {
    expect(emptyText(0, "")).toBe("No history yet.");
    expect(emptyText(4, " zebra ")).toBe("Nothing in history matches “zebra”.");
  });
});

describe("rows", () => {
  it("show the title, then the site when it differs", () => {
    expect(rowText(SMOKE)).toEqual({ title: "LJ-SMOKE ready", domain: "127.0.0.1:5000" });
    expect(rowText({ ...MORNING, title: "  " })).toEqual({ title: "news.test", domain: "" });
    expect(rowText({ ...MORNING, title: "news.test" })).toEqual({ title: "news.test", domain: "" });
    const hostile = '<img src=x onerror="alert(1)">';
    expect(rowText({ ...MORNING, title: hostile }).title).toBe(hostile);
  });
});

describe("clearing", () => {
  it("offers the last hour, 24 hours, 7 days and everything, the smallest first", () => {
    expect(CLEAR_CHOICES.map((c) => c.range)).toEqual(["hour", "day", "week", "all"]);
    expect(CLEAR_CHOICES.map((c) => c.label)).toEqual(["Last hour", "Last 24 hours", "Last 7 days", "Everything"]);
    expect(DEFAULT_CLEAR).toBe("hour");
    expect(clearChoice("week").phrase).toBe("the last 7 days");
  });

  it("says what went", () => {
    expect(clearedText("hour", 0)).toBe("Nothing to remove from the last hour.");
    expect(clearedText("day", 1)).toBe("Removed 1 page from the last 24 hours.");
    expect(clearedText("week", 12)).toBe("Removed 12 pages from the last 7 days.");
    expect(clearedText("all", 30)).toBe("History cleared.");
    expect(clearedText("all", 0)).toBe("History cleared.");
  });
});

describe("keys", () => {
  it("↑/↓ move without wrapping; ↑ from the first row goes back to the search field", () => {
    expect(historyKey(3, null, "ArrowDown", false, false)).toEqual({ kind: "select", selected: 0 });
    expect(historyKey(3, 1, "ArrowDown", false, false)).toEqual({ kind: "select", selected: 2 });
    expect(historyKey(3, 2, "ArrowDown", false, false)).toEqual({ kind: "select", selected: 2 });
    expect(historyKey(3, 2, "ArrowUp", false, false)).toEqual({ kind: "select", selected: 1 });
    expect(historyKey(3, 0, "ArrowUp", false, false)).toEqual({ kind: "select", selected: null });
    expect(historyKey(3, null, "ArrowUp", false, false)).toEqual({ kind: "none" });
    expect(historyKey(0, null, "ArrowDown", false, false)).toEqual({ kind: "none" });
  });

  it("Enter opens the selected row, or the first", () => {
    expect(historyKey(3, 2, "Enter", false, false)).toEqual({ kind: "open", index: 2 });
    expect(historyKey(3, null, "Enter", false, false)).toEqual({ kind: "open", index: 0 });
    expect(historyKey(0, null, "Enter", false, false)).toEqual({ kind: "none" });
  });

  it("Shift+Delete removes the selected page, nothing else", () => {
    expect(historyKey(3, 1, "Delete", true, false)).toEqual({ kind: "remove", index: 1 });
    expect(historyKey(3, 1, "Delete", false, false)).toEqual({ kind: "none" });
    expect(historyKey(3, null, "Delete", true, false)).toEqual({ kind: "none" });
  });

  it("Esc closes the clear choices first, then returns to the command bar", () => {
    expect(historyKey(3, 1, "Escape", false, true)).toEqual({ kind: "closeClear" });
    expect(historyKey(3, 1, "Escape", false, false)).toEqual({ kind: "back" });
    expect(historyKey(0, null, "Escape", false, false)).toEqual({ kind: "back" });
  });

  it("keeps the selection inside a shorter list", () => {
    expect(keepSelection(2, 2)).toBe(1);
    expect(keepSelection(0, 3)).toBe(0);
    expect(keepSelection(1, 0)).toBeNull();
    expect(keepSelection(null, 3)).toBeNull();
  });
});
