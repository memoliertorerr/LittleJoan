import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { settingsQuitFailures, trayFailures, traySummary, trayStates, type TrayRun } from "./smoke-tray.mts";

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

const SETTINGS = ["started", "tray-visible false", "title LJ-PERSIST set", "settings-window general", "settings-loaded", "settings-quit-click", "settings-quit", "quit"];

function good(over: Partial<TrayRun> = {}): TrayRun {
  return {
    hidden: log(HIDDEN),
    secondLaunch: true,
    typed: true,
    exitedByItself: true,
    settings: log(SETTINGS),
    settingsQuit: { exitedByItself: true, relaunch: log(["started", "tray-visible true"]), persisted: "LJ-PERSIST p=1 s=0 l=1" },
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

describe("Settings' Quit Little Joan (LJ19.7)", () => {
  it("passes when the click reached the app, which quit, ended by itself and kept the login", () => {
    expect(settingsQuitFailures(good())).toEqual([]);
    const summary = traySummary(good(), "darwin");
    expect(summary.settingsQuit).toEqual({ clicked: true, called: true, exitedByItself: true, persisted: "LJ-PERSIST p=1 s=0 l=1" });
  });

  it("names where the quit stopped", () => {
    const without = (event: string) => good({ settings: log(SETTINGS.filter((row) => row !== event)) });
    expect(settingsQuitFailures(without("settings-quit-click"))[0]).toContain("click on Settings' Quit Little Joan never came");
    expect(settingsQuitFailures(without("settings-quit"))[0]).toContain("never reached the app (no settings-quit)");
    expect(settingsQuitFailures(without("quit"))[0]).toContain("didn't quit");
    const stayed = good();
    stayed.settingsQuit = { exitedByItself: false, relaunch: log(["started"]), persisted: "LJ-PERSIST p=1 s=0 l=1" };
    expect(settingsQuitFailures(stayed)).toEqual(["settings quit: the app didn't end by itself within 15000 ms of the click"]);
    expect(trayFailures(stayed, "win32").join()).toContain("didn't end by itself within 15000 ms");
  });

  it("wants the login back after the relaunch, and a later launch", () => {
    const lost = good();
    lost.settingsQuit = { exitedByItself: true, relaunch: log(["started"]), persisted: "LJ-PERSIST p=0 s=0 l=1" };
    expect(settingsQuitFailures(lost)[0]).toContain("want p=1 s=0 l=1");
    const kept = good();
    kept.settingsQuit = { exitedByItself: true, relaunch: log(["started"]), persisted: "LJ-PERSIST p=1 s=1 l=1" };
    expect(settingsQuitFailures(kept)[0]).toContain("said LJ-PERSIST p=1 s=1 l=1");
    const first = good();
    first.settingsQuit = { exitedByItself: true, relaunch: log(["first-run", "started"]), persisted: undefined };
    expect(settingsQuitFailures(first)).toEqual([
      "settings quit: the relaunch on the same data folder didn't start as a later launch",
      "settings quit: the relaunched page never reported its login (got nothing)",
    ]);
    const none = good();
    delete none.settingsQuit;
    expect(settingsQuitFailures(none)).toEqual(["settings quit: the run didn't happen"]);
  });
});
