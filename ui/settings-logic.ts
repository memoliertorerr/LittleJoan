// Types and small pure helpers for the settings window: its tabs, change detection across them
// (Save and Cancel cover every tab), and the Privacy tab's "Clear history" action.
import { CLEAR_CHOICES, clearChoice, clearedText, type ClearRange } from "./history/logic.ts";
import type { Theme } from "./lib/theme.ts";
import { sameTheme } from "./settings/appearance-logic.ts";
import { sameSites, type BuiltInSite, type SiteMode } from "./settings/sites-logic.ts";
import { sameOverrides, type ShortcutProblem } from "./settings/keyboard-logic.ts";

export interface Choice {
  id: string;
  name: string;
}

export interface Settings {
  version: number;
  shortcut: string;
  searchEngine: string;
  translateTo: string;
  windowWidth: number;
  windowHeight: number;
  mobileSites: boolean;
  rememberHistory: boolean;
  /** The look (ui/lib/theme.ts); settings.rs checks every field. */
  theme: Theme;
  /** Load unpacked extensions from the extensions folder (Settings → Extensions). */
  developerMode: boolean;
  /** Switched-off extensions (`hello`) and built-ins (`builtin:calc`). */
  disabledExtensions: string[];
  /** Phone or desktop per site (General; settings/site_modes.rs checks every entry). */
  siteModes: Record<string, SiteMode>;
  /** The tray / menu-bar icon; off, it hides while the shortcut works (General, LJ41). */
  showTrayIcon: boolean;
  /** The user's own keys (Keyboard, LJ43/44): action id → accelerator, "" for no key. */
  keys: Record<string, string>;
  /** What stays at rest after a close (General, LJ19.4; settings/after_close.rs). */
  afterClose: AfterClose;
}

/** "After closing" (LJ19.4, the owner's decision): the hidden window and the bar stay, for 15 minutes, or nothing. */
export type AfterClose = "ready" | "ready15" | "hibernate";

/**
 * About how much memory "Stay ready" keeps while the window is closed, per OS — the app's and its
 * web engine's processes together (resident memory), as CI's smoke runs measured it (LJ19.4's
 * notes). A rough figure on purpose: it varies with the machine.
 */
export const READY_MEMORY_MB: Readonly<Record<string, number>> = { macos: 90, windows: 140, linux: 110 };

/** General's "After closing" choices, worded for `platform`, drawn as text. */
export function afterCloseChoices(platform: string): Choice[] {
  const mb = READY_MEMORY_MB[platform];
  const memory = mb === undefined ? "some memory" : `about ${mb} MB of memory`;
  return [
    { id: "ready", name: `Stay ready — opens instantly; keeps ${memory} while closed` },
    { id: "ready15", name: "Stay ready for 15 minutes, then hibernate" },
    { id: "hibernate", name: "Hibernate — uses no memory while closed; opens a little slower" },
  ];
}

/** An "After closing" value from anything: one of the three, else the default, `ready`. */
export function afterCloseFrom(value: unknown): AfterClose {
  return value === "ready15" || value === "hibernate" ? value : "ready";
}

/** Where Little Joan keeps things (Rust's get_settings), for the Privacy tab. */
export interface Storage {
  /** The folder of settings.json and history.json. */
  folder: string;
  /** Where the web engine keeps cookies and site data. */
  siteData: string;
}

export interface SettingsView {
  settings: Settings;
  /** Whether the OS login entry exists; `null` when it can't be read. Not part of Settings. */
  startAtLogin: boolean | null;
  engines: Choice[];
  languages: Choice[];
  platform: string;
  storage: Storage;
  version: string;
  /** The sites built in on this OS (YouTube on the desktop site on macOS and Linux). */
  builtInSiteModes: BuiltInSite[];
  /** The saved shortcut couldn't be registered at launch (LJ44). */
  shortcutProblem: ShortcutProblem | null;
  /** A Linux Wayland session: the global shortcut can't fire. */
  waylandNoShortcut: boolean;
}

export interface SettingsInput {
  shortcut: string;
  searchEngine: string;
  translateTo: string;
  mobileSites: boolean;
  rememberHistory: boolean;
  /** The Appearance tab's theme; left out, Rust keeps the saved one. */
  theme?: Theme;
  /** The Extensions tab's switches; left out, Rust keeps the saved ones. */
  disabledExtensions?: string[];
  developerMode?: boolean;
  /** General's per-site list; left out (not edited), Rust keeps the saved one. */
  siteModes?: Record<string, SiteMode>;
  /** General's icon switch; left out, Rust keeps the saved one. */
  showTrayIcon?: boolean;
  /** Keyboard's keys; left out (not edited), Rust keeps the saved ones. */
  keys?: Record<string, string>;
  /** General's "After closing"; left out, Rust keeps the saved one. */
  afterClose?: AfterClose;
}

/** The Extensions tab's part of Save (settings/extensions-logic.ts's draft). */
export interface ExtensionsInput {
  disabledExtensions: string[];
  developerMode: boolean;
}

/** General's icon switch: the menu bar on macOS, the system tray elsewhere. */
export function trayLabelFor(platform: string): string {
  return platform === "macos" ? "Show the icon in the menu bar" : "Show the icon in the system tray";
}

/** The note under the icon switch, naming the shortcut as Settings shows it ("" = none yet). */
export function trayText(shortcut: string): string {
  const press = shortcut ? `press ${shortcut}` : "press your shortcut";
  return `Without it, ${press} or open Little Joan again from your apps. The icon comes back by itself whenever the shortcut can't work.`;
}

/** The tabs, in the sidebar's order. */
export const TABS = [
  { id: "general", label: "General" },
  { id: "keyboard", label: "Keyboard" },
  { id: "appearance", label: "Appearance" },
  { id: "extensions", label: "Extensions" },
  { id: "privacy", label: "Privacy" },
  { id: "about", label: "About" },
] as const;

export type Tab = (typeof TABS)[number]["id"];

/** A tab by name (`--settings <tab>`, `open_settings(tab)`); General for anything else. */
export function tabFrom(name: unknown): Tab {
  const wanted = typeof name === "string" ? name.trim().toLowerCase() : "";
  return TABS.find((tab) => tab.id === wanted)?.id ?? "general";
}

/** The tab ↑/↓ (and Home/End) move to in the sidebar, without wrapping; `null` for other keys. */
export function tabForKey(current: Tab, key: string): Tab | null {
  const index = TABS.findIndex((tab) => tab.id === current);
  const last = TABS.length - 1;
  const next =
    key === "ArrowDown"
      ? Math.min(index + 1, last)
      : key === "ArrowUp"
        ? Math.max(index - 1, 0)
        : key === "Home"
          ? 0
          : key === "End"
            ? last
            : null;
  return next === null ? null : (TABS[next]?.id ?? current);
}

/** What the Save button sends: only the fields the window edits, from every tab. */
export function toInput(
  shortcut: string,
  searchEngine: string,
  translateTo: string,
  mobileSites: boolean,
  rememberHistory: boolean,
  theme?: Theme,
  extensions?: ExtensionsInput,
  siteModes?: Record<string, SiteMode>,
  showTrayIcon?: boolean,
  keys?: Record<string, string>,
  afterClose?: AfterClose,
): SettingsInput {
  return {
    shortcut,
    searchEngine,
    translateTo,
    mobileSites,
    rememberHistory,
    ...(theme ? { theme } : {}),
    ...(extensions ?? {}),
    ...(siteModes ? { siteModes } : {}),
    ...(showTrayIcon === undefined ? {} : { showTrayIcon }),
    ...(keys ? { keys } : {}),
    ...(afterClose === undefined ? {} : { afterClose }),
  };
}

function sameIds(a: readonly string[], b: readonly string[]): boolean {
  const left = [...a].sort();
  const right = [...b].sort();
  return left.length === right.length && left.every((id, i) => id === right[i]);
}

/** Whether anything differs from the saved settings (Save is disabled otherwise). */
export function isChanged(saved: Settings, input: SettingsInput): boolean {
  return (
    saved.shortcut !== input.shortcut ||
    saved.searchEngine !== input.searchEngine ||
    saved.translateTo !== input.translateTo ||
    saved.mobileSites !== input.mobileSites ||
    saved.rememberHistory !== input.rememberHistory ||
    (input.theme !== undefined && !sameTheme(saved.theme, input.theme)) ||
    (input.developerMode !== undefined && input.developerMode !== saved.developerMode) ||
    (input.disabledExtensions !== undefined && !sameIds(saved.disabledExtensions, input.disabledExtensions)) ||
    (input.siteModes !== undefined && !sameSites(saved.siteModes, input.siteModes)) ||
    (input.showTrayIcon !== undefined && input.showTrayIcon !== saved.showTrayIcon) ||
    (input.keys !== undefined && !sameOverrides(saved.keys, input.keys)) ||
    (input.afterClose !== undefined && input.afterClose !== afterCloseFrom(saved.afterClose))
  );
}

/**
 * The window's edits on new saved settings (a quick command changed some from the command bar,
 * LJ42): a field the window didn't edit — still the old saved value — follows the change; an
 * edited one keeps the edit, so a later Save neither undoes the change nor loses the edit.
 */
export function rebase(draft: Settings, oldSaved: Settings, newSaved: Settings): Settings {
  const follow = <K extends keyof Settings>(key: K, same: (a: Settings[K], b: Settings[K]) => boolean = Object.is): Settings[K] =>
    same(draft[key], oldSaved[key]) ? newSaved[key] : draft[key];
  return {
    ...newSaved,
    shortcut: follow("shortcut"),
    searchEngine: follow("searchEngine"),
    translateTo: follow("translateTo"),
    mobileSites: follow("mobileSites"),
    rememberHistory: follow("rememberHistory"),
    showTrayIcon: follow("showTrayIcon"),
    afterClose: follow("afterClose"),
    developerMode: follow("developerMode"),
    theme: follow("theme", sameTheme),
    disabledExtensions: follow("disabledExtensions", sameIds),
    siteModes: follow("siteModes", sameSites),
    keys: follow("keys", sameOverrides),
  };
}

/** What Save has to apply: the settings, the login entry, both, or neither. */
export interface Changes {
  settings: boolean;
  startAtLogin: boolean;
}

/**
 * Compares the window with what was saved. The start-at-login switch is the OS login entry, not
 * a setting, so it is applied by its own call; it can't change when the entry couldn't be read.
 */
export function changes(
  saved: Settings,
  savedStartAtLogin: boolean | null,
  input: SettingsInput,
  startAtLogin: boolean,
): Changes {
  return {
    settings: isChanged(saved, input),
    startAtLogin: savedStartAtLogin !== null && savedStartAtLogin !== startAtLogin,
  };
}

/** Whether Save has anything to do (it is disabled otherwise). */
export function hasChanges(todo: Changes): boolean {
  return todo.settings || todo.startAtLogin;
}

/**
 * The Privacy tab's "Clear history": pick a range, ask, then Rust's answer. It acts at once
 * (it isn't a setting, so Save and Cancel don't cover it).
 */
export interface ClearState {
  range: ClearRange;
  /** "Clear…" was pressed: the question and its buttons show. */
  asking: boolean;
  /** Waiting for Rust. */
  busy: boolean;
  /** What happened last ("" for nothing). */
  message: string;
}

export type ClearEvent =
  | { kind: "pick"; range: ClearRange }
  | { kind: "ask" }
  | { kind: "cancel" }
  | { kind: "confirm" }
  | { kind: "done"; removed: number }
  | { kind: "failed"; error: string };

export const CLEAR_START: ClearState = { range: "hour", asking: false, busy: false, message: "" };

export { CLEAR_CHOICES };

export function clearStep(state: ClearState, event: ClearEvent): ClearState {
  switch (event.kind) {
    case "pick":
      return state.busy ? state : { ...state, range: event.range, asking: false };
    case "ask":
      return state.busy ? state : { ...state, asking: true, message: "" };
    case "cancel":
      return state.busy ? state : { ...state, asking: false };
    case "confirm":
      return state.asking && !state.busy ? { ...state, busy: true } : state;
    case "done":
      return { ...state, asking: false, busy: false, message: clearedText(state.range, event.removed) };
    case "failed":
      return { ...state, asking: false, busy: false, message: event.error };
  }
}

/** The question before clearing. */
export function clearQuestion(range: ClearRange): string {
  return range === "all"
    ? "Remove all of your history?"
    : `Remove the history of ${clearChoice(range).phrase}?`;
}
