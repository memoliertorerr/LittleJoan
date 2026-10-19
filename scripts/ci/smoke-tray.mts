// The smoke test's tray run (LJ41) and its rules. Hiding the tray / menu-bar icon must never leave
// Little Joan unreachable, so: a launch whose settings.json says `"showTrayIcon": false` must log
// `tray-visible false` once its shortcut is registered (the shortcut works, so the icon may hide);
// a second launch without flags must still open the window (`window-created`); typing "quit" in
// its command bar (↓ ↓ Enter: the search row first, then "Quit Little Joan") must log `quit`, and
// the process must end by itself (typed on Windows and Linux; best effort on macOS, as the other
// typing). A relaunch opens Settings → General with the switch off (`settings-general-tray.png`).
// On Linux a launch with `XDG_SESSION_TYPE=wayland` in its environment — it still runs on Xvfb's
// X11, but counts as a Wayland session, where the X11 shortcut can't fire — must log
// `tray-visible true` despite the setting. smoke.mts supplies the driver; the rules are pure and
// unit-tested (smoke-tray.test.ts).
import type { LogEvent } from "./smoke-log.mts";
import type { SiteApp } from "./smoke-sitemode.mts";

/** What the run needs from smoke.mts. */
export interface TrayDriver {
  os: string;
  /** Launches the app on a fresh data folder whose settings.json gets `settings`, with `env` added. */
  launch(name: string, settings: Record<string, unknown>, env?: Record<string, string>): Promise<SiteApp>;
  screenshot(name: string): void;
  sleep(ms: number): Promise<void>;
  typeText(text: string, pid: number, activate: boolean): void;
  pressKeys(keys: string[], pid: number, activate: boolean): void;
  /** Waits up to `timeoutMs` for the process to end on its own. */
  exited(pid: number, timeoutMs: number): Promise<boolean>;
}

/** What the run showed. */
export interface TrayRun {
  /** The launch with the icon switched off. */
  hidden: LogEvent[];
  /** A plain second launch logged `window-created`. */
  secondLaunch: boolean;
  /** The command bar showed the typed "quit" (its `suggestions` line). */
  typed: boolean;
  /** The process ended by itself after Enter on "Quit Little Joan". */
  exitedByItself: boolean;
  /** The relaunch that opened Settings → General. */
  settings: LogEvent[];
  /** Linux: the launch in a "Wayland" session. */
  wayland?: LogEvent[];
}

/** The `tray-visible` lines of a launch, in order. */
export function trayStates(events: LogEvent[]): string[] {
  return events.filter((e) => e.event === "tray-visible").map((e) => e.detail);
}

const QUIT_TYPED = "quit";

/**
 * The rules: with the setting off, `tray-visible false` after `shortcut-registered` and never
 * `true`; a plain second launch opens the window; where the typing is driven (Windows, Linux;
 * macOS when it showed up), "quit" + Enter logs `quit` and the process ends by itself; on Linux,
 * a Wayland session shows the icon despite the setting.
 */
export function trayFailures(run: TrayRun, os: string): string[] {
  const failures: string[] = [];
  const registered = run.hidden.findIndex((e) => e.event === "shortcut-registered");
  const hiddenAfter = run.hidden.findIndex((e, i) => i > registered && e.event === "tray-visible" && e.detail === "false");
  if (registered < 0) {
    failures.push("tray: the launch with the icon off never logged shortcut-registered");
  } else if (hiddenAfter < 0) {
    failures.push(`tray: with "showTrayIcon": false the log had [${trayStates(run.hidden).join(", ")}], not tray-visible false after shortcut-registered`);
  }
  if (trayStates(run.hidden).includes("true")) {
    failures.push("tray: the icon was shown although the setting was off and the shortcut worked");
  }
  if (!run.secondLaunch) {
    failures.push("tray: a second launch without flags didn't open the window (no window-created)");
  }
  const driven = os !== "darwin";
  if (driven && !run.typed) {
    failures.push(`tray: the command bar never showed the typed "${QUIT_TYPED}"`);
  }
  if (driven || run.typed) {
    if (!run.hidden.some((e) => e.event === "quit")) {
      failures.push('tray: "Quit Little Joan" in the command bar never logged quit');
    }
    if (!run.exitedByItself) {
      failures.push('tray: the app didn\'t end by itself after "Quit Little Joan"');
    }
  }
  if (os === "linux") {
    const states = trayStates(run.wayland ?? []);
    if (!states.includes("true") || states.includes("false")) {
      failures.push(`tray: in a Wayland session (no working shortcut) the log had [${states.join(", ")}], not tray-visible true`);
    }
  }
  return failures;
}

/** Runs the tray launches. */
export async function trayRun(d: TrayDriver): Promise<TrayRun> {
  const app = await d.launch("tray", { showTrayIcon: false });
  await app.waitFor("tray: started, icon decided", (e) => e.some((x) => x.event === "started") && e.some((x) => x.event === "tray-visible"), 30_000);
  await d.sleep(1500);
  const pid = app.pid();
  const shownFrom = app.events().length;
  // A second launch without flags: the window, as from the app list.
  await app.forward([]);
  const secondLaunch = await app.waitFor("tray: the window from a second launch", (e) => e.slice(shownFrom).some((x) => x.event === "window-created"), 20_000);
  await app.waitFor("tray: the command bar", (e) => e.slice(shownFrom).some((x) => x.event === "extensions-list"), 20_000);
  await d.sleep(1500);
  d.typeText(QUIT_TYPED, pid, true);
  const typed = await app.waitFor(
    `tray: typed "${QUIT_TYPED}"`,
    (e) => e.slice(shownFrom).some((x) => x.event === "suggestions" && x.detail.endsWith(`typed=${QUIT_TYPED.length}`)),
    15_000,
  );
  let exitedByItself = false;
  if (typed) {
    await d.sleep(800);
    d.screenshot("command-quit.png");
    // Only when the typing arrived: otherwise ↓ ↓ Enter would open whatever the empty command
    // bar offers.
    d.pressKeys(["Down", "Down", "Return"], pid, true);
    await app.waitFor("tray: quit", (e) => e.some((x) => x.event === "quit"), 15_000);
    exitedByItself = await d.exited(pid, 15_000);
  }
  const hidden = app.events();
  if (!exitedByItself) {
    // Still running (macOS without the typing, or a failure): end it the usual way.
    await app.quit();
  }

  // Settings → General with the switch off, for the screenshot.
  const shown = await d.launch("tray-settings", { showTrayIcon: false });
  await shown.waitFor("tray: Settings' launch started", (e) => e.some((x) => x.event === "tray-visible"), 30_000);
  await d.sleep(1500);
  await shown.forward(["--settings", "general"]);
  await shown.waitFor("tray: Settings → General", (e) => e.some((x) => x.event === "settings-loaded"), 20_000);
  await d.sleep(1500);
  d.screenshot("settings-general-tray.png");
  const settings = shown.events();
  await shown.quit();

  let wayland: LogEvent[] | undefined;
  if (d.os === "linux") {
    // Still X11 underneath (Xvfb), but a Wayland session as far as Little Joan can tell.
    const session = await d.launch("tray-wayland", { showTrayIcon: false }, { XDG_SESSION_TYPE: "wayland" });
    await session.waitFor("tray: the Wayland session's icon", (e) => e.some((x) => x.event === "tray-visible"), 30_000);
    await d.sleep(1000);
    wayland = session.events();
    await session.quit();
  }
  return { hidden, secondLaunch, typed, exitedByItself, settings, ...(wayland ? { wayland } : {}) };
}

/** What summary.json records of the run. */
export function traySummary(run: TrayRun, os: string) {
  return {
    hidden: trayStates(run.hidden),
    secondLaunch: run.secondLaunch,
    typed: run.typed,
    quitLogged: run.hidden.some((e) => e.event === "quit"),
    exitedByItself: run.exitedByItself,
    settingsLoaded: run.settings.some((e) => e.event === "settings-loaded"),
    wayland: run.wayland ? trayStates(run.wayland) : null,
    rules: trayFailures(run, os),
  };
}
