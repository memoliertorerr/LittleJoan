import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  FIND_DEBOUNCE_MS,
  FIND_UNAVAILABLE,
  MAX_FIND_QUERY,
  findLabel,
  findResult,
  findStrip,
  type FindAction,
  type FindResult,
} from "./find-logic.ts";

/** A strip whose sends are recorded, on vitest's fake clock. */
function strip() {
  const sent: FindAction[] = [];
  const find = findStrip({
    send: (action) => sent.push(action),
    schedule: (run, ms) => setTimeout(run, ms),
    cancel: (timer) => clearTimeout(timer as ReturnType<typeof setTimeout>),
  });
  return { find, sent };
}

const counted = (matches: number, active: number | null): FindResult => ({ matches, active, found: matches > 0, unavailable: false });

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("the find strip", () => {
  it("opens closed, and only Ctrl/Cmd+F opens it", () => {
    const { find, sent } = strip();
    expect(find.isOpen()).toBe(false);
    find.input("lj");
    vi.advanceTimersByTime(FIND_DEBOUNCE_MS * 2);
    expect(sent).toEqual([]);
    find.open();
    expect(find.isOpen()).toBe(true);
    expect(sent, "nothing typed yet").toEqual([]);
  });

  it("sends what is typed 120 ms after the last key", () => {
    const { find, sent } = strip();
    find.open();
    for (const text of ["l", "lj", "lj-", "lj-needle"]) {
      find.input(text);
      vi.advanceTimersByTime(FIND_DEBOUNCE_MS - 20);
    }
    expect(sent).toEqual([]);
    vi.advanceTimersByTime(20);
    expect(sent).toEqual([{ kind: "start", query: "lj-needle" }]);
    find.input("lj-needle");
    vi.advanceTimersByTime(FIND_DEBOUNCE_MS);
    expect(sent, "the same text again is no new query").toHaveLength(1);
  });

  it("cuts a query at 1,000 characters", () => {
    const { find, sent } = strip();
    find.open();
    find.input("é".repeat(MAX_FIND_QUERY + 10));
    vi.advanceTimersByTime(FIND_DEBOUNCE_MS);
    const first = sent[0];
    expect(first?.kind).toBe("start");
    expect(first?.kind === "start" ? Array.from(first.query) : []).toHaveLength(MAX_FIND_QUERY);
  });

  it("clears the find at once when the field is emptied", () => {
    const { find, sent } = strip();
    find.open();
    find.input("needle");
    vi.advanceTimersByTime(FIND_DEBOUNCE_MS);
    find.input("");
    expect(sent.at(-1)).toEqual({ kind: "start", query: "" });
    expect(find.isOpen()).toBe(true);
    expect(find.key("Enter", false, false)).toBe("next");
    expect(sent, "nothing to find").toHaveLength(2);
  });

  it("finds the next match on Enter and the previous on Shift+Enter", () => {
    const { find, sent } = strip();
    find.open();
    find.input("needle");
    vi.advanceTimersByTime(FIND_DEBOUNCE_MS);
    expect(find.key("Enter", false, false)).toBe("next");
    expect(find.key("Enter", true, false)).toBe("previous");
    find.next();
    find.previous();
    expect(sent.slice(1)).toEqual([{ kind: "next" }, { kind: "previous" }, { kind: "next" }, { kind: "previous" }]);
  });

  it("sends a query still waiting on Enter, instead of the next match", () => {
    const { find, sent } = strip();
    find.open();
    find.input("needle");
    find.key("Enter", false, false);
    expect(sent).toEqual([{ kind: "start", query: "needle" }]);
    vi.advanceTimersByTime(FIND_DEBOUNCE_MS * 2);
    expect(sent, "not twice").toHaveLength(1);
  });

  it("does nothing while an input method composes", () => {
    const { find, sent } = strip();
    find.open();
    find.input("検索");
    vi.advanceTimersByTime(FIND_DEBOUNCE_MS);
    expect(find.key("Enter", false, true)).toBe("none");
    expect(find.key("Escape", false, true)).toBe("none");
    expect(sent).toHaveLength(1);
    expect(find.isOpen()).toBe(true);
  });

  it("closes on Esc: Rust ends the find and gives the page the keyboard", () => {
    const { find, sent } = strip();
    find.open();
    find.input("needle");
    expect(find.key("Escape", false, false)).toBe("close");
    expect(find.isOpen()).toBe(false);
    expect(sent, "the waiting query never went").toEqual([{ kind: "stop" }]);
    vi.advanceTimersByTime(FIND_DEBOUNCE_MS * 2);
    expect(sent).toHaveLength(1);
    find.close();
    expect(sent, "closed already").toHaveLength(1);
  });

  it("finds the last query again when it opens again", () => {
    const { find, sent } = strip();
    find.open();
    find.input("needle");
    vi.advanceTimersByTime(FIND_DEBOUNCE_MS);
    find.close();
    find.open();
    expect(find.query()).toBe("needle");
    expect(sent.at(-1)).toEqual({ kind: "start", query: "needle" });
    find.open();
    expect(sent.filter((a) => a.kind === "start"), "open already: nothing new").toHaveLength(2);
  });

  it("starts empty again when the window rests", () => {
    const { find, sent } = strip();
    find.open();
    find.input("needle");
    find.reset();
    vi.advanceTimersByTime(FIND_DEBOUNCE_MS * 2);
    expect(find.isOpen()).toBe(false);
    expect(find.query()).toBe("");
    find.open();
    expect(sent, "nothing to find again").toEqual([]);
  });

  it("closes without a word to Rust when Rust closed it", () => {
    const { find, sent } = strip();
    find.open();
    find.input("needle");
    find.closed();
    vi.advanceTimersByTime(FIND_DEBOUNCE_MS * 2);
    expect(find.isOpen()).toBe(false);
    expect(sent).toEqual([]);
  });

  it("shows Rust's result, and drops it once the query changes or the strip closes", () => {
    const { find } = strip();
    find.open();
    find.input("lj-needle");
    vi.advanceTimersByTime(FIND_DEBOUNCE_MS);
    expect(find.label(), "nothing said yet").toBe("");
    find.result({ matches: 3, active: 1, found: true });
    expect(find.label()).toBe("1 of 3");
    find.input("lj-needle2");
    expect(find.label()).toBe("");
    find.result({ matches: 0, active: null, found: false });
    expect(find.label()).toBe("No matches");
    find.close();
    find.result({ matches: 3, active: 1, found: true });
    expect(find.label()).toBe("");
  });
});

describe("the label", () => {
  it("counts where the engine counts (Windows, Linux)", () => {
    expect(findLabel(counted(12, 3), "a")).toBe("3 of 12");
    expect(findLabel(counted(12, null), "a")).toBe("12 matches");
    expect(findLabel(counted(1, null), "a")).toBe("1 match");
    expect(findLabel(counted(0, null), "a")).toBe("No matches");
  });

  it("says only found or not where the engine doesn't count (macOS)", () => {
    expect(findLabel({ matches: null, active: null, found: true, unavailable: false }, "a")).toBe("Found");
    expect(findLabel({ matches: null, active: null, found: false, unavailable: false }, "a")).toBe("No matches");
  });

  it("says when Windows' web engine has no find", () => {
    expect(findLabel({ matches: null, active: null, found: false, unavailable: true }, "a")).toBe(FIND_UNAVAILABLE);
    expect(FIND_UNAVAILABLE).toBe("Find isn't available in this version of Windows' web engine");
  });

  it("is empty with nothing typed or nothing said", () => {
    expect(findLabel(counted(3, 1), "")).toBe("");
    expect(findLabel(null, "a")).toBe("");
  });
});

describe("Rust's result", () => {
  it("is checked", () => {
    expect(findResult({ matches: 3, active: 2, found: true })).toEqual({ matches: 3, active: 2, found: true, unavailable: false });
    expect(findResult({ matches: null, active: null, found: true })).toEqual({ matches: null, active: null, found: true, unavailable: false });
    expect(findResult({ found: false, unavailable: true, matches: null, active: null })?.unavailable).toBe(true);
    for (const bad of [null, "3 of 12", { matches: -1, active: 1, found: true }, { matches: 1.5, active: 1, found: true }, { matches: 3, active: 1 }, { matches: "3", active: 1, found: true }]) {
      expect(findResult(bad)).toBeNull();
    }
  });
});
