// Settings → General's "Browser" section (LJ46; pure, unit-tested): whether Little Joan is the
// system's default browser, the button that opens the system's own default-apps settings (the only
// thing it does — Little Joan never changes a default), and the main browser pages are handed on
// to: "System default" or one of the browsers the system lists (never Little Joan; Rust leaves it
// out). The note under the list says where "Open in browser" and new-window links go with the
// choice shown — so a user who made Little Joan the default sees that pages go to another browser.
// Every name comes from the system and is drawn as text.

/** What Rust's browsers_info says (checked by `browsersInfoFrom`). */
export interface BrowsersInfo {
  platform: string;
  /** The system's default for web links: Little Joan, another browser, or unknown. */
  defaultKind: "self" | "other" | "none";
  defaultName: string | null;
  browsers: { id: string; name: string }[];
  /** settings.json's `mainBrowser` (it may name a browser that isn't installed any more). */
  mainBrowser: string | null;
  /** The first of `browsers`: where pages go while Little Joan is the default. */
  firstOther: string | null;
  /** Whether the button can open the system's settings here. */
  canOpenSettings: boolean;
  /** Whether this install can be the system's browser at all (not the AppImage, which installs nothing). */
  candidate: boolean;
}

/** "System default" in the list. */
export const SYSTEM_DEFAULT = "";

const text = (value: unknown, max = 300): string | null => (typeof value === "string" && value.length > 0 && value.length <= max ? value : null);

/** Rust's answer, each field checked; anything else is an empty, unknown answer. */
export function browsersInfoFrom(value: unknown): BrowsersInfo {
  const raw = typeof value === "object" && value !== null ? (value as Record<string, unknown>) : {};
  const kind = raw.defaultKind === "self" || raw.defaultKind === "other" ? raw.defaultKind : "none";
  const browsers = Array.isArray(raw.browsers)
    ? raw.browsers.flatMap((b: unknown) => {
        const entry = typeof b === "object" && b !== null ? (b as Record<string, unknown>) : {};
        const id = text(entry.id);
        const name = text(entry.name);
        return id && name ? [{ id, name }] : [];
      })
    : [];
  return {
    platform: text(raw.platform, 40) ?? "",
    defaultKind: kind,
    defaultName: text(raw.defaultName),
    browsers: browsers.slice(0, 64),
    mainBrowser: text(raw.mainBrowser),
    firstOther: text(raw.firstOther),
    canOpenSettings: raw.canOpenSettings === true,
    candidate: raw.candidate !== false,
  };
}

/** The line beside the button. */
export function defaultText(info: BrowsersInfo): string {
  switch (info.defaultKind) {
    case "self":
      return "Little Joan is your default browser";
    case "other":
      return info.defaultName ? `Little Joan isn't your default browser (${info.defaultName} is)` : "Little Joan isn't your default browser";
    case "none":
      return "Your default browser couldn't be read";
  }
}

/** What the system calls the place where the default browser is chosen. */
export function settingsPlace(platform: string): string {
  if (platform === "macos") {
    return "System Settings → Desktop & Dock → Default web browser";
  }
  if (platform === "windows") {
    return "Settings → Apps → Default apps";
  }
  return "your desktop's default applications";
}

/** The note under the button: where to choose Little Joan, or (no settings to open) how. */
export function settingsNote(info: BrowsersInfo): string {
  if (!info.candidate) {
    return "The AppImage can't be your system's browser: it installs nothing for your desktop to find (the .deb can). Pages you hand on still go to your main browser.";
  }
  if (info.canOpenSettings) {
    return `To use Little Joan for links from other apps, choose it in ${settingsPlace(info.platform)}. Little Joan never changes it by itself.`;
  }
  return 'Choose Little Joan in your desktop\'s settings, or run xdg-settings set default-web-browser "Little Joan.desktop" yourself. Little Joan never changes it by itself.';
}

/** Whether "Open default apps settings" shows: there is something to open, and a reason to. */
export function showsSettingsButton(info: BrowsersInfo): boolean {
  return info.candidate && info.canOpenSettings;
}

/** The main browser's list: System default, then the system's browsers; the saved one is kept selected. */
export function browserChoices(info: BrowsersInfo): { id: string; name: string }[] {
  const choices = [{ id: SYSTEM_DEFAULT, name: "System default" }, ...info.browsers];
  if (info.mainBrowser && !info.browsers.some((b) => b.id === info.mainBrowser)) {
    choices.push({ id: info.mainBrowser, name: `${info.mainBrowser} (not installed)` });
  }
  return choices;
}

/** Where pages go with `chosen` in the list (the `handoff_target` rules, as Settings explains them). */
export function goesToNote(info: BrowsersInfo, chosen: string): string {
  const picked = info.browsers.find((b) => b.id === chosen);
  if (picked) {
    return `"Open in browser" and links that open a new window go to ${picked.name}.`;
  }
  const missing = chosen === SYSTEM_DEFAULT ? "" : "That browser isn't installed any more. ";
  if (info.defaultKind === "self") {
    return info.firstOther
      ? `${missing}Opening pages in ${info.firstOther}, because Little Joan is your default browser.`
      : `${missing}No other browser to open pages in: they stay in Little Joan.`;
  }
  const name = info.defaultKind === "other" && info.defaultName ? info.defaultName : "your default browser";
  return `${missing}"Open in browser" and links that open a new window go to ${name}.`;
}
