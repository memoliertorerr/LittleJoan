// Shortcut text for people: "⌘⇧Space" on macOS, "Ctrl+Shift+Space" elsewhere.
// Accelerators use Tauri's syntax ("CommandOrControl+Shift+Space"; keys are KeyboardEvent.code
// names such as "KeyJ", "Digit1", "Space", "F5").

const MAC_SYMBOLS: Record<string, string> = {
  commandorcontrol: "⌘",
  cmdorctrl: "⌘",
  command: "⌘",
  cmd: "⌘",
  super: "⌘",
  control: "⌃",
  ctrl: "⌃",
  alt: "⌥",
  option: "⌥",
  shift: "⇧",
};

const OTHER_NAMES: Record<string, string> = {
  commandorcontrol: "Ctrl",
  cmdorctrl: "Ctrl",
  command: "Super",
  cmd: "Super",
  super: "Super",
  control: "Ctrl",
  ctrl: "Ctrl",
  alt: "Alt",
  option: "Alt",
  shift: "Shift",
};

/** "KeyJ" → "J", "Digit1" → "1"; other codes ("Space", "F5", "ArrowUp") stay as they are. */
export function keyName(code: string): string {
  const letter = /^Key([A-Z])$/.exec(code);
  if (letter?.[1]) {
    return letter[1];
  }
  const digit = /^Digit([0-9])$/.exec(code);
  if (digit?.[1]) {
    return digit[1];
  }
  return code;
}

export function displayShortcut(accelerator: string, isMac: boolean): string {
  const parts = accelerator
    .split("+")
    .map((part) => part.trim())
    .filter((part) => part.length > 0);
  const key = parts.pop();
  if (key === undefined) {
    return "";
  }
  const names = isMac ? MAC_SYMBOLS : OTHER_NAMES;
  const modifiers = parts.map((part) => names[part.toLowerCase()] ?? part);
  return isMac ? [...modifiers, keyName(key)].join("") : [...modifiers, keyName(key)].join("+");
}
