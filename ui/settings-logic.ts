// Types and small pure helpers for the settings window: its tabs, change detection across them
// (Save and Cancel cover every tab), and the Privacy tab's "Clear history" action.
import { CLEAR_CHOICES, clearChoice, clearedText, type ClearRange } from "./history/logic.ts";
import type { Theme } from "./lib/theme.ts";
import { sameTheme } from "./settings/appearance-logic.ts";

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
}

export interface SettingsInput {
  shortcut: string;
  searchEngine: string;
  translateTo: string;
  mobileSites: boolean;
  rememberHistory: boolean;
  /** The Appearance tab's theme; left out, Rust keeps the saved one. */
  theme?: Theme;
}

/** The tabs, in the sidebar's order. */
export const TABS = [
  { id: "general", label: "General" },
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
): SettingsInput {
  return { shortcut, searchEngine, translateTo, mobileSites, rememberHistory, ...(theme ? { theme } : {}) };
}

/** Whether anything differs from the saved settings (Save is disabled otherwise). */
export function isChanged(saved: Settings, input: SettingsInput): boolean {
  return (
    saved.shortcut !== input.shortcut ||
    saved.searchEngine !== input.searchEngine ||
    saved.translateTo !== input.translateTo ||
    saved.mobileSites !== input.mobileSites ||
    saved.rememberHistory !== input.rememberHistory ||
    (input.theme !== undefined && !sameTheme(saved.theme, input.theme))
  );
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
