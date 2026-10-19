// The OSes' own shortcuts (LJ44; data, curated): Settings → Keyboard warns when a key — or the
// shortcut that summons Little Joan — is one the system uses, and refuses the editing keys pages
// need. From the OSes' own documentation: Apple's "Mac keyboard shortcuts" (support.apple.com
// HT201236), Microsoft's "Keyboard shortcuts in Windows", GNOME's and KDE Plasma's default
// shortcuts and Ubuntu's. On macOS the user's own changes refine it: Rust reads them
// (keymap_info's systemShortcuts) — one switched off drops its warning, one on another key warns
// there instead. Accelerators use Little Joan's syntax (keymap.rs): on macOS
// `CommandOrControl` is ⌘, `Control` ⌃, `Alt` ⌥; `Super+*` matches Super with any key. Nothing is
// fetched at runtime; nothing here is ever changed on the system.

export type ReservedOs = "macos" | "windows" | "linux" | "all";

export interface Reserved {
  os: ReservedOs;
  /** An accelerator, or a modifier wildcard ("Super+*": that modifier with any key). */
  accelerator: string;
  /** What the system (or every page) uses it for. */
  name: string;
  /** `error`: never allowed (the editing keys); a warning otherwise. */
  level: "error" | "warning";
  /**
   * macOS: its number in the `com.apple.symbolichotkeys` preferences, where the user may have
   * switched it off or given it another key (Rust reads them: system_shortcuts.rs).
   */
  appleId?: number;
}

const mac = (accelerator: string, name: string, appleId?: number): Reserved => ({
  os: "macos",
  accelerator,
  name,
  level: "warning",
  ...(appleId === undefined ? {} : { appleId }),
});
const win = (accelerator: string, name: string): Reserved => ({ os: "windows", accelerator, name, level: "warning" });
const linux = (accelerator: string, name: string): Reserved => ({ os: "linux", accelerator, name, level: "warning" });
const editing = (accelerator: string): Reserved => ({ os: "all", accelerator, name: "editing in pages", level: "error" });

export const RESERVED: readonly Reserved[] = [
  // Every OS: the keys text fields in pages need.
  editing("CommandOrControl+KeyC"),
  editing("CommandOrControl+KeyV"),
  editing("CommandOrControl+KeyX"),
  editing("CommandOrControl+KeyA"),
  editing("CommandOrControl+KeyZ"),
  editing("CommandOrControl+Shift+KeyZ"),
  // macOS.
  mac("CommandOrControl+Space", "Spotlight", 64),
  mac("CommandOrControl+Alt+Space", "a Finder search window", 65),
  mac("Control+Space", "switching input sources", 60),
  mac("Control+Alt+Space", "switching input sources", 61),
  mac("CommandOrControl+Control+Space", "the emoji and symbols viewer"),
  mac("CommandOrControl+Shift+Digit3", "screenshots", 28),
  mac("CommandOrControl+Shift+Digit4", "screenshots", 30),
  mac("CommandOrControl+Shift+Digit5", "screenshots and recording", 184),
  mac("CommandOrControl+Tab", "switching apps"),
  mac("CommandOrControl+Backquote", "switching windows", 27),
  mac("CommandOrControl+Control+KeyQ", "locking the screen"),
  mac("CommandOrControl+Shift+KeyQ", "logging out"),
  mac("CommandOrControl+Alt+Escape", "Force Quit"),
  mac("CommandOrControl+KeyH", "hiding the app in front"),
  mac("CommandOrControl+Alt+KeyH", "hiding other apps"),
  mac("CommandOrControl+KeyM", "minimizing a window"),
  mac("CommandOrControl+Alt+KeyD", "showing or hiding the Dock", 52),
  mac("CommandOrControl+Control+KeyF", "full screen"),
  mac("Control+ArrowUp", "Mission Control", 32),
  mac("Control+ArrowDown", "app windows", 33),
  mac("Control+ArrowLeft", "moving between spaces", 79),
  mac("Control+ArrowRight", "moving between spaces", 81),
  // Windows.
  win("Super+*", "Windows shortcuts (the Windows key)"),
  win("Alt+Tab", "switching windows"),
  win("Alt+F4", "closing the window"),
  win("Alt+Space", "the window menu"),
  win("Control+Escape", "the Start menu"),
  win("Control+Shift+Escape", "Task Manager"),
  win("Control+Alt+Delete", "the security screen"),
  win("Control+Super+ArrowLeft", "switching desktops"),
  win("Control+Super+ArrowRight", "switching desktops"),
  win("PrintScreen", "screenshots"),
  // Linux: GNOME, KDE Plasma, Ubuntu.
  linux("Super+*", "the desktop's shortcuts (the Super key)"),
  linux("Super+Space", "switching input sources"),
  linux("Alt+Tab", "switching windows"),
  linux("Alt+F2", "running a command"),
  linux("Alt+F4", "closing the window"),
  linux("Alt+Space", "KRunner (KDE) or the window menu"),
  linux("Control+Alt+KeyT", "a terminal"),
  linux("Control+Alt+Delete", "logging out"),
  linux("Control+Alt+ArrowLeft", "switching workspaces"),
  linux("Control+Alt+ArrowRight", "switching workspaces"),
  linux("Control+Alt+ArrowUp", "switching workspaces"),
  linux("Control+Alt+ArrowDown", "switching workspaces"),
  linux("PrintScreen", "screenshots"),
];
