// The OSes' own shortcuts (LJ44; data, curated): Settings → Keyboard warns when a key — or the
// shortcut that summons Little Joan — is one the system uses, and refuses the editing keys pages
// need. From the OSes' own documentation: Apple's "Mac keyboard shortcuts" (support.apple.com
// HT201236), Microsoft's "Keyboard shortcuts in Windows", GNOME's and KDE Plasma's default
// shortcuts and Ubuntu's. Accelerators use Little Joan's syntax (keymap.rs): on macOS
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
}

const mac = (accelerator: string, name: string): Reserved => ({ os: "macos", accelerator, name, level: "warning" });
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
  mac("CommandOrControl+Space", "Spotlight"),
  mac("CommandOrControl+Alt+Space", "a Finder search window"),
  mac("Control+Space", "switching input sources"),
  mac("Control+Alt+Space", "switching input sources"),
  mac("CommandOrControl+Control+Space", "the emoji and symbols viewer"),
  mac("CommandOrControl+Shift+Digit3", "screenshots"),
  mac("CommandOrControl+Shift+Digit4", "screenshots"),
  mac("CommandOrControl+Shift+Digit5", "screenshots and recording"),
  mac("CommandOrControl+Tab", "switching apps"),
  mac("CommandOrControl+Backquote", "switching windows"),
  mac("CommandOrControl+Control+KeyQ", "locking the screen"),
  mac("CommandOrControl+Shift+KeyQ", "logging out"),
  mac("CommandOrControl+Alt+Escape", "Force Quit"),
  mac("CommandOrControl+KeyH", "hiding the app in front"),
  mac("CommandOrControl+Alt+KeyH", "hiding other apps"),
  mac("CommandOrControl+KeyM", "minimizing a window"),
  mac("CommandOrControl+Alt+KeyD", "showing or hiding the Dock"),
  mac("CommandOrControl+Control+KeyF", "full screen"),
  mac("Control+ArrowUp", "Mission Control"),
  mac("Control+ArrowDown", "app windows"),
  mac("Control+ArrowLeft", "moving between spaces"),
  mac("Control+ArrowRight", "moving between spaces"),
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
