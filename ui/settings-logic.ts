// Types and small pure helpers for the settings window.

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
}

export interface SettingsView {
  settings: Settings;
  /** Whether the OS login entry exists; `null` when it can't be read. Not part of Settings. */
  startAtLogin: boolean | null;
  engines: Choice[];
  languages: Choice[];
  platform: string;
}

export interface SettingsInput {
  shortcut: string;
  searchEngine: string;
  translateTo: string;
  mobileSites: boolean;
}

/** What the Save button sends: only the fields the window edits. */
export function toInput(shortcut: string, searchEngine: string, translateTo: string, mobileSites: boolean): SettingsInput {
  return { shortcut, searchEngine, translateTo, mobileSites };
}

/** Whether anything differs from the saved settings (Save is disabled otherwise). */
export function isChanged(saved: Settings, input: SettingsInput): boolean {
  return (
    saved.shortcut !== input.shortcut ||
    saved.searchEngine !== input.searchEngine ||
    saved.translateTo !== input.translateTo ||
    saved.mobileSites !== input.mobileSites
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
