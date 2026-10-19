import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { trayFailures, traySummary, trayStates, type TrayRun } from "./smoke-tray.mts";

const log = (rows: string[]) => parseLog(rows.map((row, i) => `${1000 + i} ${row}`).join("\n") + "\n");

const HIDDEN = [
  "started",
  "shortcut-registered CommandOrControl+Shift+Space",
  "tray-visible false",
  "window-created",
  "extensions-list 13 skipped=0",
  "suggestions 1 first=- typed=4",
  "quit",
];

function good(over: Partial<TrayRun> = {}): TrayRun {
  return {
    hidden: log(HIDDEN),
    secondLaunch: true,
    typed: true,
    exitedByItself: true,
    settings: log(["started", "tray-visible false", "settings-window general", "settings-loaded"]),
    wayland: log(["started", "shortcut-registered CommandOrControl+Shift+Space", "tray-visible true"]),
    ...over,
  };
}

describe("the tray run's rules (LJ41)", () => {
  it("passes a run where the icon hid, a second launch opened the window and Quit quit", () => {
    for (const os of ["win32", "linux", "darwin"]) {
      expect(trayFailures(good(), os), os).toEqual([]);
    }
    expect(trayStates(good().hidden)).toEqual(["false"]);
  });

  it("wants the icon hidden only after the shortcut registered, and never shown", () => {
    const before = good({ hidden: log(["started", "tray-visible false", "shortcut-registered X"]) });
    expect(trayFailures(before, "win32").join()).toContain("not tray-visible false after shortcut-registered");
    const shown = good({ hidden: log([...HIDDEN, "tray-visible true"]) });
    expect(trayFailures(shown, "win32").join()).toContain("the icon was shown although the setting was off");
    const noShortcut = good({ hidden: log(["started", "tray-visible true"]) });
    expect(trayFailures(noShortcut, "linux").join()).toContain("never logged shortcut-registered");
  });

  it("wants a plain second launch to open the window", () => {
    expect(trayFailures(good({ secondLaunch: false }), "darwin").join()).toContain("second launch without flags");
  });

  it("wants Quit to quit where the typing is driven, and on macOS only when it showed up", () => {
    const noQuit = good({ hidden: log(HIDDEN.slice(0, -1)), exitedByItself: false });
    expect(trayFailures(noQuit, "win32")).toHaveLength(2);
    expect(trayFailures(noQuit, "linux")).toHaveLength(2);
    expect(trayFailures(noQuit, "darwin")).toHaveLength(2);
    const untyped = good({ hidden: log(HIDDEN.slice(0, -2)), typed: false, exitedByItself: false });
    expect(trayFailures(untyped, "darwin")).toEqual([]);
    expect(trayFailures(untyped, "win32").join()).toContain('never showed the typed "quit"');
    expect(trayFailures(good({ exitedByItself: false }), "linux").join()).toContain("didn't end by itself");
  });

  it("wants the icon shown in a Linux Wayland session despite the setting", () => {
    expect(trayFailures(good({ wayland: log(["started", "tray-visible false"]) }), "linux").join()).toContain("Wayland session");
    expect(trayFailures(good({ wayland: log(["started"]) }), "linux").join()).toContain("Wayland session");
    const withoutWayland = good();
    delete withoutWayland.wayland;
    expect(trayFailures(withoutWayland, "linux").join()).toContain("Wayland session");
    expect(trayFailures(withoutWayland, "win32")).toEqual([]);
  });

  it("summarises what it saw", () => {
    expect(traySummary(good(), "linux")).toMatchObject({
      hidden: ["false"],
      secondLaunch: true,
      quitLogged: true,
      exitedByItself: true,
      settingsLoaded: true,
      wayland: ["true"],
      rules: [],
    });
  });
});
