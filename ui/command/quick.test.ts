import { describe, expect, it } from "vitest";
import { link } from "../../extensions/markdown-link/logic.js";
import { DEFAULT_THEME } from "../lib/theme.ts";
import { PRESETS } from "../lib/presets.ts";
import type { SiteModeInfo } from "../toolbar/logic.ts";
import { CHECK, isPreset, markdownLink, quickEnter, quickMatches, quickRows, quickStateFrom, rowText, type QuickState } from "./quick.ts";
import { buildRows, type Row } from "./rows.ts";

const PAGE = { url: "https://example.com/a?b=1", title: "Example [draft] *page*" };

function state(over: Partial<QuickState> = {}): QuickState {
  return {
    page: null,
    theme: DEFAULT_THEME,
    searchEngine: "google",
    engines: [
      { id: "google", name: "Google" },
      { id: "duckduckgo", name: "DuckDuckGo" },
      { id: "bing", name: "Bing" },
      { id: "brave", name: "Brave" },
    ],
    mobileSites: true,
    rememberHistory: true,
    startAtLogin: false,
    showTrayIcon: true,
    iconForced: false,
    ...over,
  };
}

const site = (mode: SiteModeInfo["mode"]): SiteModeInfo => ({ host: "example.com", mode, builtIn: false, agent: "phone" });
const texts = (rows: Row[]) => rows.map(rowText);
const titles = (rows: Row[]) => rows.map((row) => row.title);

describe("quick commands (LJ42)", () => {
  it("match when every typed word, two letters or more, starts a word of the title or a keyword", () => {
    expect(quickMatches("mob", "Mobile sites", ["phone"])).toBe(true);
    expect(quickMatches("copy link", "Copy as Markdown link", [])).toBe(true);
    expect(quickMatches("copy lnk", "Copy link", [])).toBe(false);
    expect(quickMatches("m", "Mobile sites", [])).toBe(false);
    expect(quickMatches("mobile s", "Mobile sites", [])).toBe(false);
    expect(quickMatches("", "Mobile sites", [])).toBe(false);
    expect(quickMatches("bile", "Mobile sites", [])).toBe(false);
  });

  it("show each row's state as text", () => {
    expect(texts(quickRows("mobile", state()))).toEqual(["Mobile sites — on"]);
    expect(texts(quickRows("mobile", state({ mobileSites: false })))).toEqual(["Mobile sites — off"]);
    expect(texts(quickRows("dark", state({ theme: { ...DEFAULT_THEME, mode: "dark" } })))).toEqual([`Theme: Dark ${CHECK}`]);
    expect(texts(quickRows("dark", state()))).toEqual(["Theme: Dark"]);
    expect(texts(quickRows("duck", state({ searchEngine: "duckduckgo" })))).toEqual([`Search with DuckDuckGo ${CHECK}`]);
    expect(texts(quickRows("start login", state()))).toEqual(["Start at login — off"]);
    expect(texts(quickRows("start login", state({ startAtLogin: null })))).toEqual(["Start at login — unknown"]);
    expect(texts(quickRows("icon", state({ showTrayIcon: false, iconForced: true })))).toEqual(["Icon — shown (the shortcut can't work)"]);
    expect(texts(quickRows("icon", state({ showTrayIcon: false })))).toEqual(["Icon — hidden"]);
    expect(texts(quickRows("icon", state()))).toEqual(["Icon — shown"]);
    expect(texts(quickRows("remember", state({ rememberHistory: false })))).toEqual(["Remember history — off"]);
  });

  it("ask Rust for the opposite of a switch's state", () => {
    expect(quickRows("mobile", state())[0]?.action).toEqual({ kind: "quick", command: { kind: "setMobileSites", on: false }, key: "mobile", closes: false });
    expect(quickRows("icon", state({ showTrayIcon: false, iconForced: true }))[0]?.action).toMatchObject({ command: { kind: "setTrayIcon", on: true } });
    expect(quickRows("dark", state())[0]?.action).toMatchObject({ command: { kind: "setThemeMode", mode: "dark" }, closes: false });
    expect(quickRows("start login", state({ startAtLogin: null }))[0]?.action).toMatchObject({ command: { kind: "setStartAtLogin", on: true } });
  });

  it("offer the theme's modes and the presets, the current one marked, the user's mode kept", () => {
    const sand = PRESETS.find((p) => p.id === "sand");
    expect(sand).toBeDefined();
    const rows = quickRows("theme", state({ theme: { mode: "light", colors: ["#C9A27E", "#d8c49a"], intensity: 0.55, grain: 0.3 } }));
    expect(titles(rows).slice(0, 3)).toEqual(["Theme: System", "Theme: Light", "Theme: Dark"]);
    expect(rows.filter((row) => row.mark === CHECK).map((row) => row.title)).toEqual(["Theme: Light", "Theme: Sand"]);
    expect(rows.find((row) => row.title === "Theme: Sand")?.action).toMatchObject({
      command: { kind: "setTheme", theme: { mode: "light", colors: ["#c9a27e", "#d8c49a"], intensity: 0.55, grain: 0.3 } },
    });
    expect(rows).toHaveLength(3 + PRESETS.length + 1);
    expect(rows.at(-1)?.title).toBe("Settings → Appearance");
    expect(isPreset(DEFAULT_THEME, PRESETS[0] ?? { id: "", name: "", colors: [], intensity: 0, grain: 0 })).toBe(true);
  });

  it("offer the page commands only while a web page is open", () => {
    for (const query of ["translate", "open browser", "copy link", "copy markdown", "reload"]) {
      expect(quickRows(query, state()).filter((row) => row.action.kind !== "quick" || row.action.command.kind !== "reloadExtensions"), query).toEqual([]);
    }
    const open = state({ page: PAGE });
    expect(titles(quickRows("copy", open))).toEqual(["Copy link", "Copy as Markdown link"]);
    expect(quickRows("copy link", open)[0]?.action).toEqual({ kind: "copy", text: PAGE.url, close: true });
    expect(quickRows("translate", open)[0]?.action).toMatchObject({ command: { kind: "translate" }, closes: true });
    expect(quickRows("open browser", open)[0]?.action).toMatchObject({ command: { kind: "openInBrowser" }, closes: true });
    expect(titles(quickRows("reload", open))).toEqual(["Reload", "Reload extensions"]);
    expect(quickRows("reload", open)[0]?.action).toMatchObject({ command: { kind: "reload" }, closes: true });
  });

  it("copy the Markdown link with the bundled extension's own escaping", () => {
    expect(markdownLink(PAGE)).toBe(link("markdown", PAGE.title, PAGE.url));
    expect(markdownLink(PAGE)).toBe("[Example \\[draft\\] \\*page\\*](https://example.com/a?b=1)");
    expect(quickRows("markdown", state({ page: PAGE }))[0]?.action).toEqual({ kind: "copy", text: markdownLink(PAGE), close: true });
  });

  it("move the site rows in, with the site's mode marked (LJ40's rows)", () => {
    const open = state({ page: PAGE });
    expect(texts(quickRows("site", open, site("desktop")))).toEqual([`Desktop site — this site ${CHECK}`, "Phone site — this site", "Automatic — this site", "Mobile sites — on"]);
    expect(texts(quickRows("request desktop", open, site("auto")))).toEqual(["Desktop site — this site"]);
    expect(texts(quickRows("automatic", open, site("auto")))).toEqual([`Automatic — this site ${CHECK}`]);
    expect(titles(quickRows("mobile", open, site("auto")))).toEqual(["Phone site", "Automatic", "Mobile sites"]);
    expect(titles(quickRows("site", open, null))).toEqual(["Mobile sites"]);
    expect(titles(quickRows("site", state(), site("auto")))).toEqual(["Mobile sites"]);
    expect(quickRows("site", open, site("phone")).slice(0, 3).map((row) => row.action)).toEqual([
      { kind: "siteMode", mode: "desktop" },
      { kind: "siteMode", mode: "phone" },
      { kind: "siteMode", mode: "auto" },
    ]);
  });

  it("offer Clear history's four ranges, each asking for a second Enter", () => {
    const rows = quickRows("clear history", state());
    expect(texts(rows)).toEqual(["Clear history — last hour", "Clear history — last 24 hours", "Clear history — last 7 days", "Clear history — everything"]);
    const hour = rows[0]?.action;
    expect(hour).toMatchObject({ command: { kind: "clearHistory", range: "hour" }, key: "clear:hour" });
    if (!hour) {
      throw new Error("no row");
    }
    const first = quickEnter(null, hour);
    expect(first).toEqual({ run: false, armed: "clear:hour" });
    expect(texts(quickRows("clear history", state(), null, first.armed))[0]).toBe("Clear history — press Enter again to remove the history of the last hour");
    expect(texts(quickRows("clear everything", state(), null, "clear:all"))).toEqual(["Clear history — press Enter again to remove all of your history"]);
    expect(quickEnter(first.armed, hour)).toEqual({ run: true, armed: null });
    // Another row's Enter runs at once and disarms.
    const week = rows[2]?.action;
    expect(week && quickEnter("clear:hour", week)).toEqual({ run: false, armed: "clear:week" });
    const mobile = quickRows("mobile", state())[0]?.action;
    expect(mobile && quickEnter("clear:hour", mobile)).toEqual({ run: true, armed: null });
  });

  it("offer the settings tabs, reload extensions and nothing that changes keys or extensions", () => {
    expect(titles(quickRows("settings", state()))).toEqual(["Settings → Appearance", "Settings → Extensions", "Settings → Privacy", "Settings → About"]);
    expect(quickRows("privacy", state())[0]?.action).toMatchObject({ command: { kind: "openSettings", tab: "privacy" }, closes: false });
    expect(titles(quickRows("reload ext", state()))).toEqual(["Reload extensions"]);
    for (const query of ["shortcut", "keys", "developer mode", "disable"]) {
      const kinds = quickRows(query, state({ page: PAGE }), site("auto")).map((row) => (row.action.kind === "quick" ? row.action.command.kind : row.action.kind));
      expect(kinds.filter((kind) => kind !== "openSettings"), query).toEqual([]);
    }
  });

  it("share the 8 rows like the built-ins: three always find room, history matches make way", () => {
    const many = Array.from({ length: 12 }, (_, i) => ({ kind: "history" as const, title: `Theme park ${i}`, url: `https://theme${i}.test/`, host: `theme${i}.test`, score: 10 - i }));
    const rows = buildRows("theme", many, { quick: (q) => quickRows(q, state()) }).rows;
    expect(rows).toHaveLength(9);
    expect(rows.filter((row) => row.kind === "history")).toHaveLength(4);
    expect(titles(rows.slice(5, 8))).toEqual(["Theme: System", "Theme: Light", "Theme: Dark"]);
  });

  it("take Rust's state only when it has the right shape", () => {
    const raw = { ...state({ page: PAGE }), theme: { mode: "dark", colors: ["#123456"], intensity: 0.5, grain: 0.2 } };
    expect(quickStateFrom(raw)).toMatchObject({ page: PAGE, searchEngine: "google", theme: { mode: "dark" } });
    expect(quickStateFrom({ ...raw, page: { url: 3 } })?.page).toBeNull();
    for (const bad of [null, "x", { ...raw, mobileSites: "yes" }, { ...raw, startAtLogin: "no" }, { ...raw, iconForced: undefined }]) {
      expect(quickStateFrom(bad)).toBeNull();
    }
    expect(quickRows("mobile", null)).toEqual([]);
  });

  it("keep hostile page titles as text in the copied link, never in a row", () => {
    const hostile = { url: "https://example.com/", title: "<img src=x onerror=alert(1)>" };
    const rows = quickRows("copy", state({ page: hostile }));
    expect(rows.map((row) => row.title)).toEqual(["Copy link", "Copy as Markdown link"]);
    expect(rows[1]?.action).toEqual({ kind: "copy", text: "[\\<img src=x onerror=alert(1)\\>](https://example.com/)", close: true });
  });
});
