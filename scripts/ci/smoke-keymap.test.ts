import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import {
  BAD_SHORTCUT,
  exportCounts,
  keyboardProblemFailures,
  keymapFailures,
  keymapSummary,
  menuKeys,
  systemShortcutFailures,
  type KeymapRun,
} from "./smoke-keymap.mts";

const log = (rows: string[]) => parseLog(rows.map((row, i) => `${1000 + i} ${row}`).join("\n") + "\n");

const MAC_MENU = ["menu-keys open-location=⌘L", "menu-keys history=⌘⇧K", "menu-keys close=⌘W", "started"];

function run(over: Partial<KeymapRun> = {}): KeymapRun {
  return { events: log(["started", "history-view"]), newKeyOpened: true, afterOldKey: log(["overlay true"]), ...over };
}

describe("the keymap run's rules (LJ43)", () => {
  it("passes when the new key opens history and the old one nothing", () => {
    expect(keymapFailures(run(), "win32")).toEqual([]);
    expect(keymapFailures(run(), "linux")).toEqual([]);
    expect(keymapFailures(run({ events: log(MAC_MENU) }), "darwin")).toEqual([]);
  });

  it("wants the new key to work on Windows and Linux", () => {
    expect(keymapFailures(run({ newKeyOpened: false }), "linux").join()).toContain("Ctrl+Shift+K didn't open");
    expect(keymapFailures(run({ afterOldKey: log(["history-view"]) }), "win32").join()).toContain("still opened");
    const notPressed: KeymapRun = { events: log([]), newKeyOpened: true };
    expect(keymapFailures(notPressed, "linux").join()).toContain("never pressed");
  });

  it("wants the macOS menu built from the keymap, the presses best effort", () => {
    expect(keymapFailures(run({ events: log(["menu-keys history=⌘Y", "menu-keys open-location=⌘L"]) }), "darwin").join()).toContain("not ⌘⇧K");
    expect(keymapFailures(run({ events: log(["menu-keys history=⌘⇧K"]) }), "darwin").join()).toContain("Open Location");
    const unreached: KeymapRun = { events: log(MAC_MENU), newKeyOpened: false };
    expect(keymapFailures(unreached, "darwin")).toEqual([]);
    expect(keymapFailures({ ...run({ events: log(MAC_MENU) }), afterOldKey: log(["history-view"]) }, "darwin").join()).toContain("⌘Y");
  });

  it("reads the menu's keys and summarises", () => {
    expect(menuKeys(log(MAC_MENU))).toEqual({ "open-location": "⌘L", history: "⌘⇧K", close: "⌘W" });
    expect(keymapSummary(run({ events: log(MAC_MENU) }), "darwin")).toMatchObject({ newKeyOpened: true, afterOldKey: ["overlay"], rules: [] });
  });

  it("wants the problem launch to fall back to the default and open Keyboard (LJ44)", () => {
    const good = log([`shortcut-failed ${BAD_SHORTCUT}: invalid`, "shortcut-registered CommandOrControl+Shift+Space", "settings-window keyboard", "settings-loaded"]);
    expect(keyboardProblemFailures(good)).toEqual([]);
    expect(keyboardProblemFailures(log(["shortcut-registered CommandOrControl+Shift+Space", "settings-window keyboard", "settings-loaded"])).join()).toContain("no shortcut-failed");
    expect(keyboardProblemFailures(log([`shortcut-failed ${BAD_SHORTCUT}: invalid`, "settings-window keyboard", "settings-loaded"])).join()).toContain("default shortcut");
    expect(keyboardProblemFailures(log([`shortcut-failed ${BAD_SHORTCUT}: invalid`, "shortcut-registered CommandOrControl+Shift+Space", "settings-window general", "settings-loaded"])).join()).toContain("Keyboard");
  });

  it("wants macOS's read to match the runner's own export of its shortcuts (LJ44)", () => {
    const exported = JSON.stringify({
      AppleSymbolicHotKeys: {
        "15": { enabled: false },
        "60": { enabled: false, value: { type: "standard", parameters: [32, 49, 262144] } },
        "164": { enabled: true, value: { type: "modifier", parameters: [262144, 1] } },
      },
    });
    expect(exportCounts(exported)).toEqual({ entries: 3, keys: 1 });
    expect(exportCounts("nope")).toBeNull();
    expect(exportCounts(undefined)).toBeNull();
    expect(systemShortcutFailures(log(["system-shortcuts 3 keys=1"]), exported, "darwin")).toEqual([]);
    expect(systemShortcutFailures(log(["system-shortcuts 2 keys=1"]), exported, "darwin").join()).toContain("export has 3");
    expect(systemShortcutFailures(log([]), exported, "darwin").join()).toContain("never logged");
    expect(systemShortcutFailures(log(["system-shortcuts 0 keys=0"]), undefined, "darwin")).toEqual([]);
    expect(systemShortcutFailures(log([]), exported, "linux")).toEqual([]);
  });
});
