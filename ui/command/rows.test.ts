import { describe, expect, it } from "vitest";
import {
  HISTORY_ROW,
  MAX_SUGGESTIONS,
  SETTINGS_ROW,
  autofillFor,
  buildRows,
  builtInCommands,
  faviconUrl,
  hueFor,
  initialSelection,
  letterFor,
  listKey,
  queryFor,
  showsTyped,
  typedForm,
  type Row,
  type Suggestion,
} from "./rows.ts";

const SMOKE: Suggestion = {
  kind: "history",
  title: "LJ-SMOKE ready",
  url: "http://127.0.0.1:5000/smoke.html",
  host: "127.0.0.1:5000",
  score: 30,
};
const EXAMPLE: Suggestion = { kind: "history", title: "Example", url: "https://example.com/", host: "example.com", score: 6 };
const GOOGLE: Suggestion = {
  kind: "search",
  title: "Google",
  url: "https://www.google.com/search?q=127",
  host: "google.com",
  score: 0,
};
const STARTERS: Suggestion[] = ["Wikipedia", "YouTube", "GitHub", "Google Maps"].map((title, i) => ({
  kind: "starter",
  title,
  url: `https://starter${i}.test/`,
  host: `starter${i}.test`,
  score: 0,
}));

const kinds = (rows: Row[]) => rows.map((row) => row.kind);

describe("rows while nothing is typed", () => {
  it("shows the top sites, then Settings & extensions last", () => {
    const top: Suggestion = { ...SMOKE, kind: "topSite" };
    const { rows, autofill } = buildRows("  ", [top]);
    expect(kinds(rows)).toEqual(["topSite", "settings"]);
    expect(rows[0]).toMatchObject({ title: "LJ-SMOKE ready", domain: "127.0.0.1:5000", host: "127.0.0.1:5000" });
    expect(rows[0]?.action).toEqual({ kind: "open", url: SMOKE.url });
    expect(rows.at(-1)).toEqual(SETTINGS_ROW);
    expect(autofill).toBeNull();
  });

  it("ends with Settings & extensions, which opens Settings on Extensions", () => {
    expect(buildRows("", []).rows).toEqual([SETTINGS_ROW]);
    expect(SETTINGS_ROW).toMatchObject({ title: "Settings & extensions", action: { kind: "settings", tab: "extensions" } });
  });

  it("shows the four starters when there is no history", () => {
    const { rows } = buildRows("", STARTERS);
    expect(rows.map((row) => row.title)).toEqual(["Wikipedia", "YouTube", "GitHub", "Google Maps", "Settings & extensions"]);
  });

  it("shows at most 8 sites", () => {
    const many = Array.from({ length: 12 }, (_, i): Suggestion => ({ ...EXAMPLE, kind: "topSite", host: `s${i}.test` }));
    const { rows } = buildRows("", many);
    expect(rows).toHaveLength(MAX_SUGGESTIONS + 1);
    expect(rows.at(-1)?.kind).toBe("settings");
  });

  it("doesn't repeat a title that is just the site", () => {
    const { rows } = buildRows("", [{ ...EXAMPLE, kind: "topSite", title: "example.com" }]);
    expect(rows[0]).toMatchObject({ title: "example.com", domain: "" });
    const untitled = buildRows("", [{ ...EXAMPLE, kind: "topSite", title: "  " }]).rows[0];
    expect(untitled).toMatchObject({ title: "example.com", domain: "" });
  });
});

describe("rows while typing", () => {
  it("orders: quick answer, best match, search, more matches, commands; Settings last", () => {
    const answer: Row = { kind: "answer", title: "84", domain: "= 12*7", host: "", action: { kind: "copy", text: "84" } };
    const command: Row = { kind: "command", title: "Snake", domain: "", host: "", action: { kind: "command", id: "snake" } };
    const { rows } = buildRows("e", [SMOKE, EXAMPLE, GOOGLE], {
      quickAnswer: () => answer,
      commands: () => [command],
    });
    expect(kinds(rows)).toEqual(["answer", "history", "search", "history", "command", "settings"]);
    expect(rows[2]).toMatchObject({ title: "Search Google for “e”", host: "google.com" });
    expect(rows[2]?.action).toEqual({ kind: "open", url: GOOGLE.url });
  });

  it("without hooks: best match, search, the rest", () => {
    const { rows } = buildRows("127", [SMOKE, EXAMPLE, GOOGLE]);
    expect(kinds(rows)).toEqual(["history", "search", "history", "settings"]);
  });

  it("searches even when nothing matches, and without Rust's search row through open_input", () => {
    expect(kinds(buildRows("rust", [GOOGLE]).rows)).toEqual(["search", "settings"]);
    const fallback = buildRows(" rust ", []).rows[0];
    expect(fallback).toMatchObject({ title: "Search for “rust”", action: { kind: "input", text: "rust" } });
  });

  it("caps the suggestions at 8, Settings & extensions after them", () => {
    const many = Array.from({ length: 12 }, (_, i): Suggestion => ({ ...EXAMPLE, url: `https://e${i}.test/`, host: `e${i}.test` }));
    const { rows } = buildRows("e", [...many, GOOGLE], { commands: () => [{ ...SETTINGS_ROW, kind: "command" }] });
    expect(rows).toHaveLength(MAX_SUGGESTIONS + 1);
    expect(rows.slice(0, 2).map((row) => row.kind)).toEqual(["history", "search"]);
    expect(rows.filter((row) => row.kind === "command")).toHaveLength(0);
    expect(rows.at(-1)).toEqual(SETTINGS_ROW);
  });

  it("offers the History command for its name, after the matches and before extensions'", () => {
    const command: Row = { kind: "command", title: "Snake", domain: "", host: "", action: { kind: "command", id: "snake" } };
    const { rows } = buildRows("hist", [SMOKE, GOOGLE], { commands: () => [command] });
    expect(rows.map((row) => row.title)).toEqual([SMOKE.title, "Search Google for “hist”", "History", "Snake", SETTINGS_ROW.title]);
    expect(rows[2]).toEqual(HISTORY_ROW);
    expect(HISTORY_ROW.action).toEqual({ kind: "history" });
    expect(HISTORY_ROW.icon).toBe("clock");
  });

  it("keeps room for the History command among many matches", () => {
    const many = Array.from({ length: 12 }, (_, i): Suggestion => ({ ...EXAMPLE, url: `https://hi${i}.test/`, host: `hi${i}.test` }));
    const { rows } = buildRows("hi", [...many, GOOGLE]);
    expect(rows).toHaveLength(MAX_SUGGESTIONS + 1);
    expect(rows.at(-2)).toEqual(HISTORY_ROW);
    expect(rows.at(-1)).toEqual(SETTINGS_ROW);
  });

  it("matches History on its start (two letters or more) or the whole word", () => {
    for (const query of ["hi", "HIST", " history ", "clear history", "history please"]) {
      expect(builtInCommands(query), query).toEqual([HISTORY_ROW]);
    }
    for (const query of ["h", "", "historyx", "his tory", "story", "chistory"]) {
      expect(builtInCommands(query), query).toEqual([]);
    }
    expect(buildRows("", [], {}).rows).not.toContainEqual(HISTORY_ROW);
  });

  it("only history rows can be removed", () => {
    const { rows } = buildRows("127", [SMOKE, GOOGLE]);
    expect(rows[0]?.removable).toBe(SMOKE.url);
    expect(rows[1]?.removable).toBeUndefined();
    expect(buildRows("", [{ ...SMOKE, kind: "topSite" }]).rows[0]?.removable).toBeUndefined();
  });

  it("keeps a hostile title as plain text in the row", () => {
    const title = '<img src=x onerror="alert(1)">';
    const { rows } = buildRows("x", [{ ...EXAMPLE, title }, GOOGLE]);
    expect(rows[0]?.title).toBe(title);
  });
});

describe("autofill", () => {
  it("completes a host prefix with the best match's address, keeping what was typed", () => {
    expect(autofillFor("127", [SMOKE, EXAMPLE])).toBe("127.0.0.1:5000/smoke.html");
    expect(autofillFor("EXA", [EXAMPLE])).toBe("EXAmple.com");
    const list = buildRows("127", [SMOKE, EXAMPLE, GOOGLE]);
    expect(list.autofill).toEqual({ text: "127.0.0.1:5000/smoke.html", row: 0 });
    expect(initialSelection(list)).toBe(0);
  });

  it("only on a host prefix", () => {
    expect(autofillFor("smoke", [SMOKE])).toBeNull();
    expect(autofillFor("0.0.1", [SMOKE])).toBeNull();
    expect(autofillFor("127 0", [SMOKE])).toBeNull();
    expect(autofillFor("", [SMOKE])).toBeNull();
    expect(autofillFor("127", [])).toBeNull();
  });

  it("only at or above the average score", () => {
    const weak = { ...SMOKE, score: 1 };
    // Sorted by Rust, the best comes first; a weak first row below the average doesn't autofill.
    expect(autofillFor("127", [weak, { ...SMOKE, score: 9 }])).toBeNull();
    expect(autofillFor("127", [{ ...SMOKE, score: 5 }, { ...SMOKE, score: 5 }])).toBe("127.0.0.1:5000/smoke.html");
  });

  it("points at the best match's row after a quick answer", () => {
    const answer: Row = { kind: "answer", title: "127", domain: "", host: "", action: { kind: "copy", text: "127" } };
    const list = buildRows("127", [SMOKE, GOOGLE], { quickAnswer: () => answer });
    expect(list.autofill?.row).toBe(1);
  });

  it("strips the scheme, www. and a lone trailing slash from the typed form", () => {
    expect(typedForm("https://www.github.com/")).toBe("github.com");
    expect(typedForm("http://127.0.0.1:5000/smoke.html")).toBe("127.0.0.1:5000/smoke.html");
    expect(typedForm("https://example.com/a/")).toBe("example.com/a/");
  });

  it("no autofill: nothing is selected, Enter is the field's", () => {
    const list = buildRows("smoke", [SMOKE, GOOGLE]);
    expect(list.autofill).toBeNull();
    expect(initialSelection(list)).toBeNull();
  });
});

describe("keys", () => {
  const list = buildRows("127", [SMOKE, EXAMPLE, GOOGLE]);
  const last = list.rows.length - 1;

  it("↑/↓ move without wrapping; ↑ from the first row goes back to the field", () => {
    expect(listKey(list, null, "ArrowDown", false, "127")).toEqual({ kind: "select", selected: 0 });
    expect(listKey(list, 0, "ArrowDown", false, "127")).toEqual({ kind: "select", selected: 1 });
    expect(listKey(list, last, "ArrowDown", false, "127")).toEqual({ kind: "select", selected: last });
    expect(listKey(list, 1, "ArrowUp", false, "127")).toEqual({ kind: "select", selected: 0 });
    expect(listKey(list, 0, "ArrowUp", false, "127")).toEqual({ kind: "select", selected: null });
    expect(listKey(list, null, "ArrowUp", false, "127")).toEqual({ kind: "none" });
    expect(listKey({ rows: [], autofill: null }, null, "ArrowDown", false, "")).toEqual({ kind: "none" });
  });

  it("Enter opens the selected row, or with none what the field says", () => {
    expect(listKey(list, 0, "Enter", false, "127")).toEqual({ kind: "act", action: { kind: "open", url: SMOKE.url } });
    expect(listKey(list, last, "Enter", false, "127")).toEqual({ kind: "act", action: { kind: "settings", tab: "extensions" } });
    expect(listKey(list, null, "Enter", false, " github.com ")).toEqual({ kind: "act", action: { kind: "input", text: "github.com" } });
    expect(listKey(list, null, "Enter", false, "  ")).toEqual({ kind: "none" });
  });

  it("Tab accepts the autofill", () => {
    expect(listKey(list, 0, "Tab", false, "127")).toEqual({ kind: "acceptAutofill", text: "127.0.0.1:5000/smoke.html" });
    expect(listKey(buildRows("smoke", [SMOKE]), null, "Tab", false, "smoke")).toEqual({ kind: "none" });
  });

  it("Shift+Delete removes a history row, nothing else", () => {
    expect(listKey(list, 0, "Delete", true, "127")).toEqual({ kind: "remove", url: SMOKE.url });
    expect(listKey(list, 0, "Delete", false, "127")).toEqual({ kind: "none" });
    expect(listKey(list, 1, "Delete", true, "127")).toEqual({ kind: "none" });
    expect(listKey(list, null, "Delete", true, "127")).toEqual({ kind: "none" });
  });
});

describe("site icons", () => {
  it("fetch the favicon from the site over HTTPS, if the host is a plain one", () => {
    expect(faviconUrl("example.com")).toBe("https://example.com/favicon.ico");
    expect(faviconUrl("127.0.0.1:5000")).toBe("https://127.0.0.1:5000/favicon.ico");
    expect(faviconUrl("")).toBeNull();
    expect(faviconUrl('x"><img src=y>')).toBeNull();
    expect(faviconUrl("[::1]:80")).toBeNull();
  });

  it("fall back to the site's first letter in a stable tint", () => {
    expect(letterFor("example.com")).toBe("E");
    expect(letterFor("127.0.0.1:5000")).toBe("1");
    expect(letterFor("ünï.test")).toBe("Ü");
    expect(letterFor("")).toBe("•");
    expect(hueFor("example.com")).toBe(hueFor("example.com"));
    expect(hueFor("example.com")).toBeGreaterThanOrEqual(0);
    expect(hueFor("example.com")).toBeLessThan(360);
    expect(hueFor("a.test")).not.toBe(hueFor("b.test"));
  });
});

describe("the query", () => {
  it("is empty while the field still shows the open page's address", () => {
    expect(queryFor("https://example.com/", "https://example.com/")).toBe("");
    expect(queryFor("https://example.com/x", "https://example.com/")).toBe("https://example.com/x");
    expect(queryFor("127", "")).toBe("127");
    expect(queryFor("", "")).toBe("");
  });
});

describe("whether the field still shows what was typed", () => {
  it("is the typed text with the caret at its end", () => {
    expect(showsTyped("127", "127", 3, 3)).toBe(true);
    expect(showsTyped("127", "127", 1, 1)).toBe(false);
    expect(showsTyped("127", "127", 0, 3)).toBe(false);
  });

  it("or the typed text followed by a completion that is still selected", () => {
    // The answer for "12" completed the field just after "7" was typed.
    expect(showsTyped("127.0.0.1:5000/smoke.html", "127", 3, 25)).toBe(true);
    expect(showsTyped("127.0.0.1:5000/smoke.html", "12", 2, 25)).toBe(true);
    // The user moved the caret or selected something else: it's theirs now.
    expect(showsTyped("127.0.0.1:5000/smoke.html", "127", 25, 25)).toBe(false);
    expect(showsTyped("127.0.0.1:5000/smoke.html", "127", 3, 10)).toBe(false);
    expect(showsTyped("128.0.0.1", "127", 3, 9)).toBe(false);
    expect(showsTyped("127", "127", null, null)).toBe(false);
  });
});
