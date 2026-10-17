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

/** The fields of a KeyboardEvent the recorder needs. */
export interface KeyPress {
  code: string;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  metaKey: boolean;
}

/** KeyboardEvent.code values that are only modifiers. */
const MODIFIER_CODES = new Set([
  "ShiftLeft",
  "ShiftRight",
  "ControlLeft",
  "ControlRight",
  "AltLeft",
  "AltRight",
  "MetaLeft",
  "MetaRight",
  "OSLeft",
  "OSRight",
]);

/** Keys a global shortcut may end in (all of them parse on the Rust side too). */
const SHORTCUT_KEY =
  /^(Key[A-Z]|Digit[0-9]|F([1-9]|1[0-9]|2[0-4])|Space|Enter|Tab|Backspace|Delete|Insert|Home|End|PageUp|PageDown|Arrow(Up|Down|Left|Right)|Backquote|Backslash|BracketLeft|BracketRight|Comma|Equal|Minus|Period|Quote|Semicolon|Slash|Numpad[0-9]|Numpad(Add|Subtract|Multiply|Divide|Decimal|Enter))$/;

/** F13–F24 exist on few keyboards, so they may be used without a modifier. */
const LONE_KEY = /^F(1[3-9]|2[0-4])$/;

/**
 * Turns a key press into an accelerator ("CommandOrControl+Shift+Space"), or null when it can't
 * be a global shortcut: a modifier on its own, a key no shortcut can use, or a key without a
 * modifier (Shift alone doesn't count — it would steal capital letters). The platform's primary
 * modifier (Cmd on macOS, Ctrl elsewhere) is stored as "CommandOrControl" so settings travel.
 */
export function acceleratorFromEvent(press: KeyPress, isMac: boolean): string | null {
  if (MODIFIER_CODES.has(press.code) || !SHORTCUT_KEY.test(press.code)) {
    return null;
  }
  const primary = isMac ? press.metaKey : press.ctrlKey;
  const secondary = isMac ? press.ctrlKey : press.metaKey;
  const modifiers = [
    primary ? "CommandOrControl" : null,
    secondary ? (isMac ? "Control" : "Super") : null,
    press.altKey ? "Alt" : null,
    press.shiftKey ? "Shift" : null,
  ].filter((m): m is string => m !== null);
  const hasRealModifier = modifiers.some((m) => m !== "Shift");
  if (!hasRealModifier && !LONE_KEY.test(press.code)) {
    return null;
  }
  return [...modifiers, press.code].join("+");
}
