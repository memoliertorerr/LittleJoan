// The smoke test's tray run (LJ41) and its rules. Hiding the tray / menu-bar icon must never leave
// Little Joan unreachable, so: a launch whose settings.json says `"showTrayIcon": false` must log
// `tray-visible false` once its shortcut is registered (the shortcut works, so the icon may hide);
// a second launch without flags must still open the window (`window-created`); typing "quit" in
// its command bar (↓ ↓ Enter: the search row first, then "Quit Little Joan") must log `quit`, and
// the process must end by itself (typed on Windows and Linux; best effort on macOS, as the other
// typing). A relaunch opens Settings → General with the switch off (`settings-general-tray.png`),
// after its page set a login (`?persist=set`), and then (LJ19.7) its "Quit Little Joan" button is
// clicked (a smoke-only click, `LITTLEJOAN_SMOKE_SETTINGS_QUIT=1`: CI has no driver for Settings'
// controls): the app must log `settings-quit` and `quit` and end by itself, and a relaunch on the
// same data folder still have the login (the persistent cookie and localStorage).
// On Linux a launch with `XDG_SESSION_TYPE=wayland` in its environment — it still runs on Xvfb's
// X11, but counts as a Wayland session, where the X11 shortcut can't fire — must log
// `tray-visible true` despite the setting. smoke.mts supplies the driver; the rules are pure and
// unit-tested (smoke-tray.test.ts).
import { count, parsePersist, type LogEvent } from "./smoke-log.mts";
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
  /** The smoke page's address (no query): Settings' quit run sets and checks a login with it. */
  pageUrl: string;
  /** Launches the app again on `app`'s data folder (a new log, `smoke-<name>.log`). */
  relaunch(app: SiteApp, name: string): Promise<SiteApp>;
}

/** Settings → General's "Quit Little Joan" (LJ19.7): what the quit and the relaunch after it showed. */
export interface SettingsQuit {
  /** The process ended by itself after the button's click. */
  exitedByItself: boolean;
  /** The relaunch on the same data folder. */
  relaunch: LogEvent[];
  /** The smoke page's login check after the relaunch (`LJ-PERSIST p= s= l=`), if it reported. */
  persisted: string | undefined;
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
  /** The relaunch that opened Settings → General (and quit through its button). */
  settings: LogEvent[];
  /** Settings' Quit Little Joan (LJ19.7). */
  settingsQuit?: SettingsQuit;
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
  failures.push(...settingsQuitFailures(run));
  if (os === "linux") {
    const states = trayStates(run.wayland ?? []);
    if (!states.includes("true") || states.includes("false")) {
      failures.push(`tray: in a Wayland session (no working shortcut) the log had [${states.join(", ")}], not tray-visible true`);
    }
  }
  return failures;
}

/** How long the app may take to end by itself after Settings' Quit (LJ20's quit limit). */
export const SETTINGS_QUIT_LIMIT_MS = 15_000;

/**
 * Settings' "Quit Little Joan" (LJ19.7), on every OS: the click reached the page's button
 * (`settings-quit-click`, the smoke's own) and the app (`settings-quit`, the Settings-only command),
 * which quit (`quit`) and ended by itself; the relaunch on the same data folder is no first run and
 * still has the login set before (persistent cookie and localStorage; the session cookie ends
 * with the app run).
 */
export function settingsQuitFailures(run: TrayRun): string[] {
  const quit = run.settingsQuit;
  if (!quit) {
    return ["settings quit: the run didn't happen"];
  }
  const failures: string[] = [];
  const clicked = run.settings.findIndex((e) => e.event === "settings-quit-click");
  const called = run.settings.findIndex((e, i) => i > clicked && e.event === "settings-quit");
  const quitAt = run.settings.findIndex((e, i) => i > called && e.event === "quit");
  if (clicked < 0) {
    failures.push("settings quit: the smoke test's click on Settings' Quit Little Joan never came (no settings-quit-click)");
  } else if (called < 0) {
    failures.push("settings quit: the button's click never reached the app (no settings-quit)");
  } else if (quitAt < 0) {
    failures.push("settings quit: quit_from_settings didn't quit (no quit after settings-quit)");
  }
  if (!quit.exitedByItself) {
    failures.push(`settings quit: the app didn't end by itself within ${SETTINGS_QUIT_LIMIT_MS} ms of the click`);
  }
  if (count(quit.relaunch, "started") !== 1 || count(quit.relaunch, "first-run") > 0) {
    failures.push("settings quit: the relaunch on the same data folder didn't start as a later launch");
  }
  const persisted = parsePersist(quit.persisted);
  if (!persisted) {
    failures.push(`settings quit: the relaunched page never reported its login (got ${quit.persisted ?? "nothing"})`);
  } else if (!persisted.p || !persisted.l || persisted.s) {
    failures.push(`settings quit: after the quit and a relaunch the login check said ${quit.persisted ?? ""} (want p=1 s=0 l=1)`);
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

  // Settings → General with the switch off, for the screenshot — after its page set a login —,
  // then (LJ19.7) its Quit Little Joan, clicked by the smoke-only knob 4 s after Settings loaded.
  const token = `settings-quit-${Date.now().toString(36)}`;
  const shown = await d.launch("tray-settings", { showTrayIcon: false }, { LITTLEJOAN_SMOKE_SETTINGS_QUIT: "1" });
  await shown.waitFor("tray: Settings' launch started", (e) => e.some((x) => x.event === "tray-visible"), 30_000);
  await d.sleep(1500);
  const shownPid = shown.pid();
  await shown.forward(["--show", `${d.pageUrl}?persist=set&t=${token}`]);
  await shown.waitFor("settings quit: the login set", (e) => e.some((x) => x.event === "title" && x.detail === "LJ-PERSIST set"), 30_000);
  await shown.forward(["--settings", "general"]);
  await shown.waitFor("tray: Settings → General", (e) => e.some((x) => x.event === "settings-loaded"), 20_000);
  await d.sleep(1500);
  d.screenshot("settings-general-tray.png");
  await shown.waitFor("settings quit: Quit Little Joan clicked, the app quit", (e) => e.some((x) => x.event === "quit"), 15_000);
  const quitExited = await d.exited(shownPid, SETTINGS_QUIT_LIMIT_MS);
  const settings = shown.events();
  if (!quitExited) {
    await shown.quit();
  }
  const again = await d.relaunch(shown, "tray-settings-after");
  await again.waitFor("settings quit: relaunched", (e) => e.some((x) => x.event === "started"), 30_000);
  await d.sleep(1500);
  await again.forward(["--show", `${d.pageUrl}?persist=check&t=${token}`]);
  await again.waitFor("settings quit: the login checked", (e) => e.some((x) => x.event === "title" && x.detail.startsWith("LJ-PERSIST p=")), 30_000);
  const relaunch = again.events();
  const persisted = relaunch.findLast((e) => e.event === "title" && e.detail.startsWith("LJ-PERSIST p="))?.detail;
  await again.quit();
  const settingsQuit: SettingsQuit = { exitedByItself: quitExited, relaunch, persisted };

  let wayland: LogEvent[] | undefined;
  if (d.os === "linux") {
    // Still X11 underneath (Xvfb), but a Wayland session as far as Little Joan can tell.
    const session = await d.launch("tray-wayland", { showTrayIcon: false }, { XDG_SESSION_TYPE: "wayland" });
    await session.waitFor("tray: the Wayland session's icon", (e) => e.some((x) => x.event === "tray-visible"), 30_000);
    await d.sleep(1000);
    wayland = session.events();
    await session.quit();
  }
  return { hidden, secondLaunch, typed, exitedByItself, settings, settingsQuit, ...(wayland ? { wayland } : {}) };
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
    settingsQuit: run.settingsQuit
      ? {
          clicked: run.settings.some((e) => e.event === "settings-quit-click"),
          called: run.settings.some((e) => e.event === "settings-quit"),
          exitedByItself: run.settingsQuit.exitedByItself,
          persisted: run.settingsQuit.persisted ?? null,
        }
      : null,
    wayland: run.wayland ? trayStates(run.wayland) : null,
    rules: trayFailures(run, os),
  };
}
