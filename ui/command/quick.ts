// Quick commands in the command bar (LJ42; pure, unit-tested): actions and settings, each row
// showing its current state as text ("Mobile sites — on", "Theme: Dark ✓"). Rust's quick_state()
// says how things are when the command bar opens; quick_command() runs one of a fixed set
// (src-tauri/src/commands/quick.rs) — nothing a page supplies picks a command, rows are built
// here from this list. Copy link and Copy as Markdown link copy in the bar itself (copy.ts), the
// Markdown text from the bundled Copy as Markdown link extension's own logic (one escaping).
// "Clear history" asks for a second Enter on the same row. view.ts draws the rows as text only.
import { link } from "../../extensions/markdown-link/logic.js";
import { CLEAR_CHOICES, type ClearRange } from "../history/logic.ts";
import { PRESETS, type Preset } from "../lib/presets.ts";
import { themeFrom, type Theme, type ThemeMode } from "../lib/theme.ts";
import type { SiteModeInfo } from "../toolbar/logic.ts";
import type { Row } from "./rows.ts";

/** What Rust's quick_state() says. */
export interface QuickState {
  /** The open web page, or null (no page commands then). */
  page: { url: string; title: string } | null;
  theme: Theme;
  searchEngine: string;
  engines: { id: string; name: string }[];
  mobileSites: boolean;
  rememberHistory: boolean;
  /** The OS login entry exists; null when it can't be read. */
  startAtLogin: boolean | null;
  showTrayIcon: boolean;
  /** Shown although the setting is off: the shortcut can't work. */
  iconForced: boolean;
}

/** The fixed commands Rust's quick_command() takes (a serde enum there). */
export type QuickCommand =
  | { kind: "translate" }
  | { kind: "openInBrowser" }
  | { kind: "reload" }
  | { kind: "setThemeMode"; mode: ThemeMode }
  | { kind: "setTheme"; theme: Theme }
  | { kind: "setSearchEngine"; id: string }
  | { kind: "setMobileSites"; on: boolean }
  | { kind: "setRememberHistory"; on: boolean }
  | { kind: "clearHistory"; range: ClearRange }
  | { kind: "setStartAtLogin"; on: boolean }
  | { kind: "openSettings"; tab: string }
  | { kind: "reloadExtensions" }
  | { kind: "setTrayIcon"; on: boolean };

/** The mark on the choice that is current ("Theme: Dark ✓"). */
export const CHECK = "✓";

const bool = (value: unknown): value is boolean => typeof value === "boolean";
const text = (value: unknown): value is string => typeof value === "string";

/** Rust's answer, checked; null when it isn't one (then there are no quick rows). */
export function quickStateFrom(value: unknown): QuickState | null {
  if (typeof value !== "object" || value === null) {
    return null;
  }
  const raw = value as Record<string, unknown>;
  const pageRaw = raw.page as Record<string, unknown> | null | undefined;
  const page = pageRaw && text(pageRaw.url) && text(pageRaw.title) ? { url: pageRaw.url, title: pageRaw.title } : null;
  const engines = Array.isArray(raw.engines)
    ? raw.engines.flatMap((e: unknown) => {
        const engine = e as Record<string, unknown> | null;
        return engine && text(engine.id) && text(engine.name) ? [{ id: engine.id, name: engine.name }] : [];
      })
    : [];
  const login = raw.startAtLogin;
  if (
    !text(raw.searchEngine) ||
    !bool(raw.mobileSites) ||
    !bool(raw.rememberHistory) ||
    !bool(raw.showTrayIcon) ||
    !bool(raw.iconForced) ||
    !(login === null || bool(login))
  ) {
    return null;
  }
  return {
    page,
    theme: themeFrom(raw.theme),
    searchEngine: raw.searchEngine,
    engines,
    mobileSites: raw.mobileSites,
    rememberHistory: raw.rememberHistory,
    startAtLogin: login,
    showTrayIcon: raw.showTrayIcon,
    iconForced: raw.iconForced,
  };
}

/** Lower-case words of `value` (letters and digits). */
function words(value: string): string[] {
  return value.toLowerCase().split(/[^\p{L}\p{N}]+/u).filter(Boolean);
}

/** Every typed word (two letters or more) starts a word of the title or a keyword. */
export function quickMatches(query: string, title: string, keywords: readonly string[]): boolean {
  const typed = words(query);
  if (typed.length === 0 || typed.some((word) => word.length < 2)) {
    return false;
  }
  const have = [...words(title), ...keywords.flatMap(words)];
  return typed.every((word) => have.some((w) => w.startsWith(word)));
}

/** One quick row before matching: its title, keywords, state and what it does. */
interface Candidate {
  /** Stable, for "press Enter again" (Clear history's ranges). */
  key: string;
  title: string;
  keywords: readonly string[];
  icon: string;
  /** The state, after the title ("on", "this site"); "" for none. */
  state: string;
  current?: boolean;
  action: Row["action"];
}

const quick = (command: QuickCommand, key: string, closes: boolean, confirm?: string): Row["action"] =>
  confirm === undefined ? { kind: "quick", command, key, closes } : { kind: "quick", command, key, closes, confirm };

const onOff = (on: boolean): string => (on ? "on" : "off");

/** Whether `theme` is `preset` (its colors, intensity and grain; the mode is the user's). */
export function isPreset(theme: Theme, preset: Preset): boolean {
  const close = (a: number, b: number) => Math.abs(a - b) < 1e-6;
  return (
    theme.colors.length === preset.colors.length &&
    theme.colors.every((color, i) => color.toLowerCase() === preset.colors[i]?.toLowerCase()) &&
    close(theme.intensity, preset.intensity) &&
    close(theme.grain, preset.grain)
  );
}

/** The page's link as Markdown: Copy as Markdown link's own escaping. */
export function markdownLink(page: { url: string; title: string }): string {
  return link("markdown", page.title, page.url);
}

/** What the second Enter on a Clear history row removes. */
function clearConfirm(phrase: string): string {
  return `press Enter again to remove ${phrase === "all time" ? "all of your history" : `the history of ${phrase}`}`;
}

const SITE_ROWS: readonly { mode: SiteModeInfo["mode"]; title: string; icon: string; keywords: readonly string[] }[] = [
  { mode: "desktop", title: "Desktop site", icon: "desktop", keywords: ["request", "desktop", "site"] },
  { mode: "phone", title: "Phone site", icon: "phone", keywords: ["request", "phone", "mobile", "site"] },
  { mode: "auto", title: "Automatic", icon: "auto", keywords: ["automatic", "auto", "site", "desktop", "phone", "mobile"] },
];

const MODES: readonly { mode: ThemeMode; title: string }[] = [
  { mode: "system", title: "Theme: System" },
  { mode: "light", title: "Theme: Light" },
  { mode: "dark", title: "Theme: Dark" },
];

/** Settings → <tab> (General is the built-in "Settings" row). */
const TABS: readonly { tab: string; title: string; keywords: readonly string[] }[] = [
  { tab: "appearance", title: "Settings → Appearance", keywords: ["preferences", "options", "theme", "colors"] },
  { tab: "extensions", title: "Settings → Extensions", keywords: ["preferences", "options", "developer"] },
  { tab: "privacy", title: "Settings → Privacy", keywords: ["preferences", "options", "data"] },
  { tab: "about", title: "Settings → About", keywords: ["preferences", "options", "version"] },
];

/** Every quick command for `state` (and the open page's site), in the order they are offered. */
function candidates(state: QuickState, site: SiteModeInfo | null, armed: string | null): Candidate[] {
  const list: Candidate[] = [];
  const page = state.page;
  if (page) {
    list.push(
      { key: "translate", title: "Translate page", keywords: ["translation"], icon: "translate", state: "", action: quick({ kind: "translate" }, "translate", true) },
      { key: "openInBrowser", title: "Open in browser", keywords: ["default", "main"], icon: "browser", state: "", action: quick({ kind: "openInBrowser" }, "openInBrowser", true) },
      { key: "copyLink", title: "Copy link", keywords: ["url", "address"], icon: "copy", state: "", action: { kind: "copy", text: page.url, close: true } },
      { key: "copyMarkdown", title: "Copy as Markdown link", keywords: [], icon: "copy", state: "", action: { kind: "copy", text: markdownLink(page), close: true } },
      { key: "reload", title: "Reload", keywords: ["refresh", "page"], icon: "reload", state: "", action: quick({ kind: "reload" }, "reload", true) },
    );
  }
  if (page && site) {
    for (const row of SITE_ROWS) {
      list.push({
        key: `site:${row.mode}`,
        title: row.title,
        keywords: row.keywords,
        icon: row.icon,
        state: "this site",
        current: row.mode === site.mode,
        action: { kind: "siteMode", mode: row.mode },
      });
    }
  }
  for (const { mode, title } of MODES) {
    list.push({ key: `mode:${mode}`, title, keywords: ["appearance", "mode"], icon: "theme", state: "", current: state.theme.mode === mode, action: quick({ kind: "setThemeMode", mode }, `mode:${mode}`, false) });
  }
  for (const preset of PRESETS) {
    const theme: Theme = { mode: state.theme.mode, colors: [...preset.colors], intensity: preset.intensity, grain: preset.grain };
    list.push({ key: `preset:${preset.id}`, title: `Theme: ${preset.name}`, keywords: ["appearance", "preset", "colors"], icon: "theme", state: "", current: isPreset(state.theme, preset), action: quick({ kind: "setTheme", theme }, `preset:${preset.id}`, false) });
  }
  for (const engine of state.engines) {
    list.push({ key: `engine:${engine.id}`, title: `Search with ${engine.name}`, keywords: ["engine"], icon: "search", state: "", current: state.searchEngine === engine.id, action: quick({ kind: "setSearchEngine", id: engine.id }, `engine:${engine.id}`, false) });
  }
  list.push(
    { key: "mobile", title: "Mobile sites", keywords: ["phone", "narrow"], icon: "phone", state: onOff(state.mobileSites), action: quick({ kind: "setMobileSites", on: !state.mobileSites }, "mobile", false) },
    { key: "remember", title: "Remember history", keywords: ["record"], icon: "clock", state: onOff(state.rememberHistory), action: quick({ kind: "setRememberHistory", on: !state.rememberHistory }, "remember", false) },
  );
  for (const choice of CLEAR_CHOICES) {
    const key = `clear:${choice.range}`;
    const label = choice.label.charAt(0).toLowerCase() + choice.label.slice(1);
    list.push({
      key,
      title: "Clear history",
      keywords: ["delete", "forget", "remove", ...words(choice.label), ...(choice.range === "all" ? ["all"] : [])],
      icon: "clock",
      state: armed === key ? clearConfirm(choice.phrase) : label,
      action: quick({ kind: "clearHistory", range: choice.range }, key, false, clearConfirm(choice.phrase)),
    });
  }
  list.push({
    key: "login",
    title: "Start at login",
    keywords: ["startup", "launch", "boot"],
    icon: "power",
    state: state.startAtLogin === null ? "unknown" : onOff(state.startAtLogin),
    action: quick({ kind: "setStartAtLogin", on: !(state.startAtLogin ?? false) }, "login", false),
  });
  for (const { tab, title, keywords } of TABS) {
    list.push({ key: `tab:${tab}`, title, keywords, icon: "settings", state: "", action: quick({ kind: "openSettings", tab }, `tab:${tab}`, false) });
  }
  list.push(
    { key: "reloadExtensions", title: "Reload extensions", keywords: ["refresh"], icon: "extension", state: "", action: quick({ kind: "reloadExtensions" }, "reloadExtensions", false) },
    {
      key: "icon",
      title: "Icon",
      keywords: ["tray", "menu", "bar", "hide", "show"],
      icon: "settings",
      state: state.iconForced ? "shown (the shortcut can't work)" : state.showTrayIcon ? "shown" : "hidden",
      action: quick({ kind: "setTrayIcon", on: !state.showTrayIcon }, "icon", false),
    },
  );
  return list;
}

/**
 * The quick rows for what is typed: every command whose title or keywords start each typed word
 * (two letters or more), page commands only with a web page, the site rows only with its site's
 * info; `armed` is the Clear history row waiting for its second Enter.
 */
export function quickRows(query: string, state: QuickState | null, site: SiteModeInfo | null = null, armed: string | null = null): Row[] {
  if (!state) {
    return [];
  }
  return candidates(state, site, armed)
    .filter((c) => quickMatches(query, c.title, c.keywords))
    .map((c) => ({
      kind: "command",
      title: c.title,
      domain: c.state,
      host: "",
      icon: c.icon,
      ...(c.current ? { mark: CHECK } : {}),
      action: c.action,
    }));
}

/** A row as one line of text: "Title — state ✓" (tests, and what the view shows). */
export function rowText(row: Row): string {
  return `${row.title}${row.domain ? ` — ${row.domain}` : ""}${row.mark ? ` ${row.mark}` : ""}`;
}

/**
 * What Enter does on a quick row: run it — or, for one that asks first (Clear history), arm it on
 * the first Enter and run it on a second Enter on the same row. Anything else (Esc, another row,
 * typing) disarms: the caller passes `null` as `armed` then.
 */
export function quickEnter(armed: string | null, action: Row["action"]): { run: boolean; armed: string | null } {
  if (action.kind !== "quick" || action.confirm === undefined) {
    return { run: true, armed: null };
  }
  return armed === action.key ? { run: true, armed: null } : { run: false, armed: action.key };
}
