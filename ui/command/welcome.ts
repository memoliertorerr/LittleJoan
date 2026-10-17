// The one-time welcome the command bar shows on the first launch (pure, unit-tested). The bar
// shows it above the suggestions; start_info hands over the facts.
import { displayShortcut } from "../lib/shortcut.ts";

export interface StartInfo {
  /** The registered global shortcut (accelerator syntax), if any. */
  shortcut: string | null;
  platform: string;
  /** The first launch: the command bar shows the welcome (only the first summon does). */
  firstRun: boolean;
  /** Where the tray / menu-bar icon is on this OS. */
  trayHint: string;
  /** Whether the OS login entry exists; `null` when it can't be read. */
  startAtLogin: boolean | null;
  /** A Linux Wayland session, where the global shortcut can't fire. */
  waylandNoShortcut: boolean;
}

/** A sentence with one part in monospace: a key combination or a command. */
export interface ShortcutLine {
  kind: "keys" | "command" | "none";
  before: string;
  code: string;
  after: string;
}

export interface Welcome {
  title: string;
  shortcut: ShortcutLine;
  tray: string;
  /** The start-at-login checkbox's state, or `null` to leave it out (the entry can't be read). */
  startAtLogin: boolean | null;
  close: string;
}

/** The welcome to show, or `null` when this summon isn't the first launch's. */
export function welcome(info: StartInfo): Welcome | null {
  if (!info.firstRun) {
    return null;
  }
  return {
    title: "Little Joan is ready.",
    shortcut: shortcutLine(info),
    tray: info.trayHint,
    startAtLogin: info.startAtLogin,
    close: "Esc or the shortcut closes this window.",
  };
}

function shortcutLine(info: StartInfo): ShortcutLine {
  if (info.waylandNoShortcut) {
    return {
      kind: "command",
      before: "Bind ",
      code: "little-joan --toggle",
      after: " to a key in your keyboard settings: it summons and dismisses this window.",
    };
  }
  if (!info.shortcut) {
    return { kind: "none", before: "No global shortcut is set yet: choose one in Settings, from the icon's menu.", code: "", after: "" };
  }
  return {
    kind: "keys",
    before: "",
    code: displayShortcut(info.shortcut, info.platform === "macos"),
    after: " summons and dismisses this window.",
  };
}

/** The whole sentence, as read aloud or in tests. */
export function lineText(line: ShortcutLine): string {
  return `${line.before}${line.code}${line.after}`;
}
