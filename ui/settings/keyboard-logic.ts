// Settings → Keyboard (LJ44; pure, unit-tested): recording a key, the same rules as Rust's
// keymap::validate (keymap.rs) — so Save is refused here before Rust refuses it —, the OSes'
// reserved shortcuts (reserved-shortcuts.ts) as warnings, and the draft of the user's own keys
// (settings' `keys`: action id → accelerator, "" for no key; an action not named keeps its
// default). Accelerators use the global shortcut's syntax with KeyboardEvent.code key names.
import { displayShortcut, isShortcutKey } from "../lib/shortcut.ts";
import { RESERVED, type Reserved } from "./reserved-shortcuts.ts";

export type Os = "macos" | "windows" | "linux";

/** One action as Rust's keymap_info() describes it. */
export interface KeyActionInfo {
  id: string;
  /** As the macOS menu has it ("Open Location…"). */
  title: string;
  /** The saved key, or null for none. */
  binding: string | null;
  /** The defaults: the first is what Reset brings back, the rest fixed aliases. */
  defaults: string[];
}

/** An accelerator, unresolved. */
export interface Accel {
  primary: boolean;
  control: boolean;
  meta: boolean;
  alt: boolean;
  shift: boolean;
  code: string;
}

/** The keys a Little Joan key can end in (keymap.rs's Key). */
const BINDABLE = /^(Key[A-Z]|Digit[0-9]|F([1-9]|1[0-9]|2[0-4])|Arrow(Up|Down|Left|Right)|Home|End|PageUp|PageDown|Equal|Minus|BracketLeft|BracketRight|Comma|Period|Slash|Semicolon|Quote|Backquote|Backslash|Numpad[0-9]|Numpad(Add|Subtract|Multiply|Divide|Decimal))$/;
/** Keys that type nothing (keymap.rs's types_text is false). */
const SILENT = /^(F([1-9]|1[0-9]|2[0-4])|Arrow(Up|Down|Left|Right)|Home|End|PageUp|PageDown)$/;

const MODIFIERS: Record<string, keyof Omit<Accel, "code">> = {
  commandorcontrol: "primary",
  cmdorctrl: "primary",
  commandorctrl: "primary",
  cmdorcontrol: "primary",
  control: "control",
  ctrl: "control",
  super: "meta",
  cmd: "meta",
  command: "meta",
  meta: "meta",
  alt: "alt",
  option: "alt",
  shift: "shift",
};

/** An accelerator's parts, or null (an unknown or repeated modifier, no key). Any key name. */
export function parseAccelerator(text: string): Accel | null {
  const parts = text.split("+").map((part) => part.trim());
  const code = parts.pop() ?? "";
  if (!/^[A-Z][A-Za-z0-9]*$/.test(code)) {
    return null;
  }
  const accel: Accel = { primary: false, control: false, meta: false, alt: false, shift: false, code };
  for (const part of parts) {
    const flag = MODIFIERS[part.toLowerCase()];
    if (!flag || accel[flag]) {
      return null;
    }
    accel[flag] = true;
  }
  return accel;
}

/** Written as the keymap writes it: CommandOrControl, Control, Super, Alt, Shift, the key. */
export function writeAccelerator(accel: Accel): string {
  const names: [boolean, string][] = [
    [accel.primary, "CommandOrControl"],
    [accel.control, "Control"],
    [accel.meta, "Super"],
    [accel.alt, "Alt"],
    [accel.shift, "Shift"],
  ];
  return [...names.filter(([on]) => on).map(([, name]) => name), accel.code].join("+");
}

/** The modifiers a press needs on `os` (CommandOrControl is ⌘ on macOS, Ctrl elsewhere). */
export function resolve(accel: Accel, os: Os): { ctrl: boolean; shift: boolean; alt: boolean; meta: boolean } {
  return os === "macos"
    ? { ctrl: accel.control, shift: accel.shift, alt: accel.alt, meta: accel.primary || accel.meta }
    : { ctrl: accel.primary || accel.control, shift: accel.shift, alt: accel.alt, meta: accel.meta };
}

/** The same keys on `os`. */
export function sameKeys(a: string, b: string, os: Os): boolean {
  const x = parseAccelerator(a);
  const y = parseAccelerator(b);
  if (!x || !y || x.code !== y.code) {
    return false;
  }
  const rx = resolve(x, os);
  const ry = resolve(y, os);
  return rx.ctrl === ry.ctrl && rx.shift === ry.shift && rx.alt === ry.alt && rx.meta === ry.meta;
}

/** For people: "⌘⇧K", "Ctrl+Shift+K" (ui/lib/shortcut.ts); text that isn't one, as it is. */
export function show(accelerator: string, os: Os): string {
  return parseAccelerator(accelerator) ? displayShortcut(accelerator, os === "macos") : accelerator;
}

/** The fields of a KeyboardEvent recording needs. */
export interface Press {
  code: string;
  key: string;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  metaKey: boolean;
}

export type Recorded = { kind: "cancel" } | { kind: "clear" } | { kind: "wait" } | { kind: "set"; accelerator: string };

const MODIFIER_CODES = /^(Shift|Control|Alt|Meta|OS)(Left|Right)$/;

/**
 * What a key press does while a Little Joan key is being recorded: Esc cancels, Backspace means
 * no key, a key with a modifier (Shift counts; the rules then judge it) or an F-key is the new
 * key, anything else is waited past. A letter is the one the key types in the user's layout
 * (`key`), as the keymap matches it; other keys by their position (`code`).
 */
export function recordAction(press: Press, os: Os): Recorded {
  const anyModifier = press.ctrlKey || press.shiftKey || press.altKey || press.metaKey;
  if (!anyModifier && press.code === "Escape") {
    return { kind: "cancel" };
  }
  if (!anyModifier && press.code === "Backspace") {
    return { kind: "clear" };
  }
  const code = /^[a-z]$/i.test(press.key) ? `Key${press.key.toUpperCase()}` : press.code;
  if (MODIFIER_CODES.test(press.code) || !BINDABLE.test(code) || (!anyModifier && !/^F\d+$/.test(code))) {
    return { kind: "wait" };
  }
  const mac = os === "macos";
  return {
    kind: "set",
    accelerator: writeAccelerator({
      primary: mac ? press.metaKey : press.ctrlKey,
      control: mac ? press.ctrlKey : false,
      meta: mac ? false : press.metaKey,
      alt: press.altKey,
      shift: press.shiftKey,
      code,
    }),
  };
}

export interface Problem {
  level: "error" | "warning";
  message: string;
}

/** One of macOS's own shortcuts as Rust read it (system_shortcuts.rs). */
export interface SystemShortcut {
  id: number;
  enabled: boolean;
  /** Its key, when its entry says one. */
  accelerator: string | null;
}

/** The draft: the user's keys (`overrides`), the actions, the shortcut that summons Little Joan. */
export interface KeyboardDraft {
  os: Os;
  actions: readonly KeyActionInfo[];
  overrides: Readonly<Record<string, string>>;
  summon: string;
  /** macOS's own shortcuts as read (empty elsewhere, or when nothing could be read). */
  system?: readonly SystemShortcut[];
}

/** The action's key in the draft: its override (null for ""), else its first default. */
export function bindingOf(action: KeyActionInfo, overrides: Readonly<Record<string, string>>): string | null {
  const own = overrides[action.id];
  if (own !== undefined) {
    return own === "" ? null : own;
  }
  return action.defaults[0] ?? null;
}

/** Every key the action answers to in the draft: an override replaces the defaults' aliases. */
export function bindingsOf(action: KeyActionInfo, overrides: Readonly<Record<string, string>>): string[] {
  const own = overrides[action.id];
  if (own !== undefined) {
    return own === "" ? [] : [own];
  }
  return [...action.defaults];
}

/** A title without the menu's "…". */
function plain(title: string): string {
  return title.replace(/…$/, "");
}

const OS_NAMES: Record<Exclude<Reserved["os"], "all">, string> = {
  macos: "macOS",
  windows: "Windows",
  linux: "Linux desktops",
};

/** Whether a reserved entry covers `accelerator` on `os` ("Super+*": that modifier with any key). */
export function reservedMatch(entry: Reserved, accelerator: string, os: Os): boolean {
  if (entry.os !== "all" && entry.os !== os) {
    return false;
  }
  if (entry.accelerator.endsWith("+*")) {
    const wild = parseAccelerator(`${entry.accelerator.slice(0, -2)}+KeyA`);
    const accel = parseAccelerator(accelerator);
    if (!wild || !accel) {
      return false;
    }
    const want = resolve(wild, os);
    const have = resolve(accel, os);
    return (!want.ctrl || have.ctrl) && (!want.shift || have.shift) && (!want.alt || have.alt) && (!want.meta || have.meta);
  }
  return sameKeys(entry.accelerator, accelerator, os);
}

/**
 * The reserved shortcuts `accelerator` is: the editing keys as errors, the system's as warnings.
 * On macOS `system` (what Rust read from its preferences) refines the curated table: an entry
 * switched off — or moved to another key — drops that warning; an entry that is on with this key
 * warns (under the curated name when its id is known).
 */
export function reservedProblems(accelerator: string, os: Os, system: readonly SystemShortcut[] = []): Problem[] {
  const shown = show(accelerator, os);
  const read = new Map((os === "macos" ? system : []).map((entry) => [entry.id, entry]));
  const moved = (entry: Reserved): boolean => {
    const own = entry.appleId === undefined ? undefined : read.get(entry.appleId);
    return own !== undefined && (!own.enabled || (own.accelerator !== null && !sameKeys(own.accelerator, entry.accelerator, os)));
  };
  const problems: Problem[] = RESERVED.filter((entry) => reservedMatch(entry, accelerator, os) && !moved(entry)).map((entry) =>
    entry.os === "all"
      ? { level: entry.level, message: `${shown} is used for editing in pages.` }
      : { level: entry.level, message: `${OS_NAMES[entry.os]} use${entry.os === "linux" ? "" : "s"} ${shown} for ${entry.name}.` },
  );
  for (const entry of read.values()) {
    if (entry.enabled && entry.accelerator !== null && sameKeys(entry.accelerator, accelerator, os)) {
      const name = RESERVED.find((r) => r.appleId === entry.id)?.name ?? "one of its keyboard shortcuts";
      problems.push({ level: "warning", message: `macOS uses ${shown} for ${name}.` });
    }
  }
  return problems.filter((problem, index) => problems.findIndex((p) => p.message === problem.message) === index);
}

/** Ctrl+Alt on Windows and Linux: AltGr types characters on many layouts. */
function altGr(accelerator: string, os: Os): Problem[] {
  const accel = parseAccelerator(accelerator);
  const mods = accel && resolve(accel, os);
  return os !== "macos" && mods?.ctrl && mods.alt ? [{ level: "warning", message: "Ctrl+Alt is AltGr on many keyboards: typing a character there may set this off." }] : [];
}

/** What's wrong with `accelerator` as `actionId`'s key in `draft` (the keymap's rules first). */
export function actionProblems(actionId: string, accelerator: string, draft: KeyboardDraft): Problem[] {
  const { os } = draft;
  const accel = parseAccelerator(accelerator);
  if (!accel || !BINDABLE.test(accel.code)) {
    return [{ level: "error", message: "That isn't a key Little Joan can use." }];
  }
  const real = accel.primary || accel.control || accel.meta || accel.alt;
  const typing = !real && (accel.shift ? !SILENT.test(accel.code) : !/^F\d+$/.test(accel.code));
  if (typing) {
    return [{ level: "error", message: `Plain keys type text — add ${os === "macos" ? "⌘ or ⌃" : "Ctrl or Alt"}.` }];
  }
  if (os === "macos" && !(accel.primary || accel.control || accel.meta)) {
    return [{ level: "error", message: "On macOS a key needs ⌘ or ⌃." }];
  }
  const problems: Problem[] = [];
  const shown = show(accelerator, os);
  for (const other of draft.actions) {
    if (other.id !== actionId && bindingsOf(other, draft.overrides).some((b) => sameKeys(b, accelerator, os))) {
      problems.push({ level: "error", message: `${shown} is already ${plain(other.title)}.` });
    }
  }
  if (sameKeys(draft.summon, accelerator, os)) {
    problems.push({ level: "error", message: "That's the shortcut that summons Little Joan." });
  }
  return [...problems, ...reservedProblems(accelerator, os, draft.system), ...altGr(accelerator, os)];
}

/** What's wrong with `accelerator` as the shortcut that summons Little Joan. */
export function summonProblems(accelerator: string, draft: KeyboardDraft): Problem[] {
  const { os } = draft;
  const shown = show(accelerator, os);
  const taken = draft.actions
    .filter((action) => bindingsOf(action, draft.overrides).some((b) => sameKeys(b, accelerator, os)))
    .map((action): Problem => ({ level: "error", message: `${shown} is already ${plain(action.title)}.` }));
  const usable = parseAccelerator(accelerator);
  const broken: Problem[] =
    usable && isShortcutKey(usable.code)
      ? []
      : [{ level: "error", message: "That isn't a shortcut Little Joan can use: record another one, or Reset." }];
  return [...broken, ...taken, ...reservedProblems(accelerator, os, draft.system), ...altGr(accelerator, os)];
}

/** Every action's problems and the summon row's, by row ("summon" or the action id). */
export function allProblems(draft: KeyboardDraft): Record<string, Problem[]> {
  const rows: Record<string, Problem[]> = { summon: summonProblems(draft.summon, draft) };
  for (const action of draft.actions) {
    const binding = bindingOf(action, draft.overrides);
    rows[action.id] = binding === null || draft.overrides[action.id] === undefined ? [] : actionProblems(action.id, binding, draft);
  }
  return rows;
}

/** Whether any row has an error (Save is refused then; warnings never block it). */
export function hasErrors(draft: KeyboardDraft): boolean {
  return Object.values(allProblems(draft)).some((problems) => problems.some((p) => p.level === "error"));
}

/** Sets `actionId`'s key (null: no key); an override equal to the default is no override. */
export function setKey(draft: KeyboardDraft, actionId: string, accelerator: string | null): KeyboardDraft {
  const action = draft.actions.find((a) => a.id === actionId);
  if (!action) {
    return draft;
  }
  const overrides = { ...draft.overrides };
  const first = action.defaults[0];
  if (accelerator === null) {
    if (first === undefined) {
      delete overrides[actionId];
    } else {
      overrides[actionId] = "";
    }
  } else if (first !== undefined && sameKeys(first, accelerator, draft.os)) {
    delete overrides[actionId];
  } else {
    overrides[actionId] = accelerator;
  }
  return { ...draft, overrides };
}

/** Reset: the action's default again. */
export function resetKey(draft: KeyboardDraft, actionId: string): KeyboardDraft {
  const overrides = { ...draft.overrides };
  delete overrides[actionId];
  return { ...draft, overrides };
}

/** The default shortcut that summons Little Joan (settings.rs). */
export const DEFAULT_SUMMON = "CommandOrControl+Shift+Space";

/** Reset all: every key and the summon shortcut back to their defaults. */
export function resetAll(draft: KeyboardDraft): KeyboardDraft {
  return { ...draft, overrides: {}, summon: DEFAULT_SUMMON };
}

/** Whether the action's key isn't its default (Reset shows then). */
export function isCustom(draft: KeyboardDraft, actionId: string): boolean {
  return draft.overrides[actionId] !== undefined;
}

/** Two sets of overrides say the same. */
export function sameOverrides(a: Readonly<Record<string, string>>, b: Readonly<Record<string, string>>): boolean {
  const keys = new Set([...Object.keys(a), ...Object.keys(b)]);
  return [...keys].every((key) => a[key] === b[key]);
}

/** Why the saved shortcut didn't work at launch (Rust's shortcutProblem). */
export interface ShortcutProblem {
  wanted: string;
  active: string | null;
  reason: "taken" | "invalid";
}

/** The problem line, Keyboard's and General's. */
export function problemText(problem: ShortcutProblem, os: Os): string {
  const wanted = problem.reason === "invalid" ? `“${problem.wanted}”` : show(problem.wanted, os);
  const why = problem.reason === "taken" ? "another app or the system already uses it" : "it isn't a shortcut Little Joan can use";
  const then = problem.active ? `Little Joan is using ${show(problem.active, os)} instead.` : "Little Joan has no shortcut now.";
  return `${wanted} couldn't be registered: ${why}. ${then}`;
}
