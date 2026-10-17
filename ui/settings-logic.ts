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
