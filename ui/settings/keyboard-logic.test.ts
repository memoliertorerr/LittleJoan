import { describe, expect, it } from "vitest";
import {
  DEFAULT_SUMMON,
  actionProblems,
  allProblems,
  bindingOf,
  canResetAll,
  hasErrors,
  isCustom,
  parseAccelerator,
  portalText,
  PORTAL_NOTE,
  problemText,
  recordAction,
  resetAll,
  resetKey,
  sameKeys,
  reservedProblems,
  sameOverrides,
  setKey,
  show,
  summonProblems,
  writeAccelerator,
  type KeyActionInfo,
  type KeyboardDraft,
  type Os,
  type Press,
} from "./keyboard-logic.ts";

const action = (id: string, title: string, defaults: string[]): KeyActionInfo => ({ id, title, binding: defaults[0] ?? null, defaults });

function actions(os: Os): KeyActionInfo[] {
  const mac = os === "macos";
  return [
    action("open-location", "Open Location…", ["CommandOrControl+KeyL"]),
    action("history", "History", [mac ? "CommandOrControl+KeyY" : "CommandOrControl+KeyH"]),
    action("translate", "Translate Page", ["CommandOrControl+Shift+KeyT"]),
    action("reload", "Reload", os === "linux" ? ["CommandOrControl+KeyR", "F5"] : mac ? ["CommandOrControl+KeyR"] : []),
    action("back", "Back", os === "linux" ? ["Alt+ArrowLeft"] : mac ? ["CommandOrControl+BracketLeft"] : []),
    action("close", "Close Window", ["CommandOrControl+KeyW"]),
  ];
}

const draft = (os: Os, overrides: Record<string, string> = {}, summon = DEFAULT_SUMMON): KeyboardDraft => ({ os, actions: actions(os), overrides, summon });

const press = (code: string, key: string, mods: Partial<Pick<Press, "ctrlKey" | "shiftKey" | "altKey" | "metaKey">> = {}): Press => ({
  code,
  key,
  ctrlKey: false,
  shiftKey: false,
  altKey: false,
  metaKey: false,
  ...mods,
});

const messages = (problems: { message: string }[]) => problems.map((p) => p.message);

describe("recording a key (LJ44)", () => {
  it("takes the next key with a modifier, or an F-key", () => {
    expect(recordAction(press("KeyK", "K", { metaKey: true, shiftKey: true }), "macos")).toEqual({ kind: "set", accelerator: "CommandOrControl+Shift+KeyK" });
    expect(recordAction(press("KeyK", "k", { ctrlKey: true }), "windows")).toEqual({ kind: "set", accelerator: "CommandOrControl+KeyK" });
    expect(recordAction(press("KeyK", "k", { ctrlKey: true }), "macos")).toEqual({ kind: "set", accelerator: "Control+KeyK" });
    expect(recordAction(press("KeyK", "k", { metaKey: true }), "linux")).toEqual({ kind: "set", accelerator: "Super+KeyK" });
    expect(recordAction(press("F6", "F6"), "windows")).toEqual({ kind: "set", accelerator: "F6" });
    expect(recordAction(press("BracketLeft", "[", { altKey: true }), "linux")).toEqual({ kind: "set", accelerator: "Alt+BracketLeft" });
    expect(recordAction(press("KeyK", "K", { shiftKey: true }), "windows")).toEqual({ kind: "set", accelerator: "Shift+KeyK" });
  });

  it("names a letter by what the key types in the user's layout", () => {
    // AZERTY: the key in QWERTY's Q place types "a".
    expect(recordAction(press("KeyQ", "a", { ctrlKey: true }), "linux")).toEqual({ kind: "set", accelerator: "CommandOrControl+KeyA" });
  });

  it("cancels on Esc, clears on Backspace, waits past plain keys and modifiers", () => {
    expect(recordAction(press("Escape", "Escape"), "macos")).toEqual({ kind: "cancel" });
    expect(recordAction(press("Backspace", "Backspace"), "windows")).toEqual({ kind: "clear" });
    expect(recordAction(press("KeyK", "k"), "windows")).toEqual({ kind: "wait" });
    expect(recordAction(press("ShiftLeft", "Shift", { shiftKey: true }), "windows")).toEqual({ kind: "wait" });
    expect(recordAction(press("Space", " ", { ctrlKey: true }), "windows")).toEqual({ kind: "wait" });
    expect(recordAction(press("Escape", "Escape", { ctrlKey: true }), "windows")).toEqual({ kind: "wait" });
  });
});

describe("the keymap's rules (as Rust's keymap::validate)", () => {
  it("refuses two actions on one key, aliases included", () => {
    expect(messages(actionProblems("history", "CommandOrControl+KeyL", draft("macos")))).toEqual(["⌘L is already Open Location."]);
    expect(messages(actionProblems("history", "F5", draft("linux")))).toEqual(["F5 is already Reload."]);
    expect(actionProblems("history", "F5", draft("windows"))).toEqual([]);
    // Once reload has its own key, F5 is free.
    expect(actionProblems("history", "F5", draft("linux", { reload: "CommandOrControl+Shift+KeyR" }))).toEqual([]);
  });

  it("refuses the summon shortcut, typing keys, and macOS keys without ⌘ or ⌃", () => {
    expect(messages(actionProblems("history", "Alt+KeyJ", draft("linux", {}, "Alt+KeyJ")))).toEqual(["That's the shortcut that summons Little Joan."]);
    for (const typing of ["KeyK", "Shift+KeyK", "Shift+Digit1", "Shift+Equal", "ArrowLeft"]) {
      expect(actionProblems("history", typing, draft("windows")), typing).toEqual([{ level: "error", message: "Plain keys type text — add Ctrl or Alt." }]);
    }
    expect(messages(actionProblems("history", "KeyK", draft("macos")))).toEqual(["Plain keys type text — add ⌘ or ⌃."]);
    expect(messages(actionProblems("history", "Alt+Shift+KeyK", draft("macos")))).toEqual(["On macOS a key needs ⌘ or ⌃."]);
    expect(actionProblems("history", "Control+KeyK", draft("macos"))).toEqual([]);
    for (const fine of ["F6", "Shift+F6", "Alt+KeyK", "Shift+ArrowLeft"]) {
      expect(actionProblems("history", fine, draft("windows")), fine).toEqual([]);
    }
  });

  it("warns, without refusing, for Ctrl+Alt on Windows and Linux", () => {
    const problems = actionProblems("history", "CommandOrControl+Alt+KeyK", draft("windows"));
    expect(problems).toHaveLength(1);
    expect(problems[0]?.level).toBe("warning");
    expect(problems[0]?.message).toContain("AltGr");
    expect(actionProblems("history", "CommandOrControl+Alt+KeyK", draft("macos"))).toEqual([]);
  });

  it("warns about the system's own shortcuts and refuses the editing keys", () => {
    expect(summonProblems("CommandOrControl+Space", draft("macos"))).toEqual([{ level: "warning", message: "macOS uses ⌘Space for Spotlight." }]);
    expect(summonProblems("CommandOrControl+Space", draft("windows"))).toEqual([]);
    expect(messages(summonProblems("Super+KeyE", draft("windows")))).toEqual(["Windows uses Super+E for Windows shortcuts (the Windows key)."]);
    expect(messages(summonProblems("Alt+F2", draft("linux")))).toEqual(["Linux desktops use Alt+F2 for running a command."]);
    expect(actionProblems("history", "CommandOrControl+KeyC", draft("windows"))).toEqual([{ level: "error", message: "Ctrl+C is used for editing in pages." }]);
    expect(summonProblems("CommandOrControl+Shift+KeyZ", draft("macos"))[0]?.level).toBe("error");
    expect(messages(actionProblems("history", "CommandOrControl+Shift+Digit3", draft("macos")))).toEqual(["macOS uses ⌘⇧3 for screenshots."]);
  });

  it("refuses a summon shortcut that is one of Little Joan's keys", () => {
    expect(messages(summonProblems("CommandOrControl+KeyL", draft("linux")))).toEqual(["Ctrl+L is already Open Location."]);
    expect(summonProblems("CommandOrControl+KeyH", draft("linux", { history: "" }))).toEqual([]);
  });
});

describe("the draft", () => {
  it("resets per row and all at once, and a default is no override", () => {
    let d = setKey(draft("macos"), "history", "CommandOrControl+Shift+KeyK");
    expect(d.overrides).toEqual({ history: "CommandOrControl+Shift+KeyK" });
    expect(isCustom(d, "history")).toBe(true);
    expect(bindingOf(d.actions[1] as KeyActionInfo, d.overrides)).toBe("CommandOrControl+Shift+KeyK");
    d = setKey(d, "close", null);
    expect(d.overrides["close"]).toBe("");
    expect(bindingOf(d.actions[5] as KeyActionInfo, d.overrides)).toBeNull();
    d = resetKey(d, "history");
    expect(d.overrides).toEqual({ close: "" });
    expect(setKey(d, "open-location", "Command+KeyL").overrides["open-location"]).toBeUndefined();
    const all = resetAll({ ...d, summon: "Alt+KeyJ" });
    expect(all.overrides).toEqual({});
    expect(all.summon).toBe(DEFAULT_SUMMON);
    // No key for an action without defaults is no override at all.
    expect(setKey(draft("windows"), "reload", null).overrides).toEqual({});
  });

  it("blocks Save on errors only, and finds changes", () => {
    expect(hasErrors(draft("macos"))).toBe(false);
    expect(hasErrors(draft("macos", { history: "CommandOrControl+KeyL" }))).toBe(true);
    expect(hasErrors(draft("windows", { history: "CommandOrControl+Alt+KeyK" }))).toBe(false);
    expect(hasErrors(draft("macos", {}, "CommandOrControl+KeyW"))).toBe(true);
    expect(allProblems(draft("macos", { history: "CommandOrControl+KeyL" }))["history"]).toHaveLength(1);
    expect(sameOverrides({ a: "x" }, { a: "x" })).toBe(true);
    expect(sameOverrides({ a: "x" }, {})).toBe(false);
    expect(sameOverrides({ a: "" }, { a: "x" })).toBe(false);
  });

  it("parses and writes accelerators as the keymap does", () => {
    expect(writeAccelerator(parseAccelerator("shift+cmdorctrl+KeyK") ?? { primary: false, control: false, meta: false, alt: false, shift: false, code: "" })).toBe("CommandOrControl+Shift+KeyK");
    expect(parseAccelerator("Ctrl+Ctrl+KeyK")).toBeNull();
    expect(parseAccelerator("Hyper+KeyK")).toBeNull();
    expect(parseAccelerator("Ctrl+")).toBeNull();
    expect(sameKeys("Control+KeyL", "CommandOrControl+KeyL", "windows")).toBe(true);
    expect(sameKeys("Control+KeyL", "CommandOrControl+KeyL", "macos")).toBe(false);
  });

  it("says why the saved shortcut didn't work at launch", () => {
    expect(problemText({ wanted: "CommandOrControl+Shift+KeyK", active: DEFAULT_SUMMON, reason: "taken" }, "macos")).toBe(
      "⌘⇧K couldn't be registered: another app or the system already uses it. Little Joan is using ⌘⇧Space instead.",
    );
    expect(problemText({ wanted: "Hyper+Nope", active: DEFAULT_SUMMON, reason: "invalid" }, "linux")).toBe(
      "“Hyper+Nope” couldn't be registered: it isn't a shortcut Little Joan can use. Little Joan is using Ctrl+Shift+Space instead.",
    );
    expect(problemText({ wanted: "Alt+KeyJ", active: null, reason: "taken" }, "windows")).toContain("Little Joan has no shortcut now.");
  });
});

describe("macOS's own shortcuts, read from its preferences (LJ44)", () => {
  // The CI runner's export (system_shortcuts.rs' test data): the input sources switched off.
  const RUNNER = [
    { id: 60, enabled: false, accelerator: "Control+Space" },
    { id: 61, enabled: false, accelerator: "Control+Alt+Space" },
    { id: 79, enabled: true, accelerator: null },
    { id: 81, enabled: true, accelerator: null },
  ];

  it("drops a curated warning the user switched off, and keeps the rest", () => {
    expect(messages(reservedProblems("Control+Space", "macos"))).toEqual(["macOS uses ⌃Space for switching input sources."]);
    expect(reservedProblems("Control+Space", "macos", RUNNER)).toEqual([]);
    expect(reservedProblems("Control+Alt+Space", "macos", RUNNER)).toEqual([]);
    expect(messages(reservedProblems("Control+ArrowLeft", "macos", RUNNER))).toEqual(["macOS uses ⌃← for moving between spaces."]);
    expect(messages(summonProblems("Control+Space", { ...draft("macos"), system: RUNNER }))).toEqual([]);
  });

  it("follows a shortcut the user moved, and warns about one the table doesn't know", () => {
    const moved = [{ id: 64, enabled: true, accelerator: "Alt+Space" }];
    expect(reservedProblems("CommandOrControl+Space", "macos", moved)).toEqual([]);
    expect(messages(reservedProblems("Alt+Space", "macos", moved))).toEqual(["macOS uses ⌥Space for Spotlight."]);
    const other = [{ id: 190, enabled: true, accelerator: "CommandOrControl+Shift+KeyK" }];
    expect(messages(actionProblems("history", "CommandOrControl+Shift+KeyK", { ...draft("macos"), system: other }))).toEqual([
      "macOS uses ⌘⇧K for one of its keyboard shortcuts.",
    ]);
    expect(reservedProblems("CommandOrControl+Shift+KeyK", "macos", [{ id: 190, enabled: false, accelerator: "CommandOrControl+Shift+KeyK" }])).toEqual([]);
    // The same key warned once, and nothing read counts off macOS.
    expect(messages(reservedProblems("CommandOrControl+Space", "macos", [{ id: 64, enabled: true, accelerator: "CommandOrControl+Space" }]))).toEqual([
      "macOS uses ⌘Space for Spotlight.",
    ]);
    expect(reservedProblems("CommandOrControl+Shift+KeyK", "windows", other)).toEqual([]);
  });
});

describe("a saved summon shortcut that can't be one", () => {
  it("shows as written and must be replaced before Save", () => {
    expect(show("Hyper+Nope", "macos")).toBe("Hyper+Nope");
    expect(show("CommandOrControl+Shift+Space", "macos")).toBe("⌘⇧Space");
    expect(summonProblems("Hyper+Nope", draft("macos"))[0]?.level).toBe("error");
    expect(summonProblems("CommandOrControl+Nope", draft("linux"))[0]?.level).toBe("error");
    expect(hasErrors(draft("linux", {}, "Hyper+Nope"))).toBe(true);
    expect(summonProblems(DEFAULT_SUMMON, draft("linux"))).toEqual([]);
  });
});

describe("the portal's shortcut (LJ17)", () => {
  const portalDraft = (summon: string): KeyboardDraft => ({
    os: "linux",
    actions: actions("linux"),
    overrides: {},
    summon,
    portal: { trigger: "Press <ctrl><shift>space" },
  });

  it("shows the desktop's own words for its key, and where to change it", () => {
    expect(portalText({ trigger: "Press <ctrl><shift>space" })).toBe("Press <ctrl><shift>space");
    expect(portalText({ trigger: null })).toBe("No key yet");
    expect(PORTAL_NOTE).toBe("Change it in your desktop's keyboard settings.");
  });

  it("never judges the desktop's key, so it can't block Save", () => {
    const broken = portalDraft("Hyper+Nope");
    expect(allProblems(broken)["summon"]).toEqual([]);
    expect(hasErrors(broken)).toBe(false);
    expect(hasErrors({ ...broken, portal: null })).toBe(true);
  });

  it("keeps the preferred trigger on Reset all, which then resets the keys only", () => {
    const draft = { ...portalDraft("Alt+KeyJ"), overrides: { history: "CommandOrControl+Shift+KeyK" } };
    expect(resetAll(draft)).toEqual({ ...draft, overrides: {} });
    expect(canResetAll(draft)).toBe(true);
    expect(canResetAll(resetAll(draft))).toBe(false);
    expect(canResetAll({ ...portalDraft("Alt+KeyJ"), portal: null })).toBe(true);
    expect(resetAll({ ...draft, portal: null }).summon).toBe(DEFAULT_SUMMON);
  });
});
