// The smoke test's logins runs (LJ51) and their rules: Windows' web engine's own password saving and
// form autofill as Settings → Privacy switches them, and removing what it saved. Two launches:
// - "logins" (default switches, "afterClose": "hibernate"): `--show <smoke page>` — the page's
//   engine reads back `logins password-autosave=false autofill=true webview=page` and its profile
//   folder (`webview-profile page=<path>`); `--hide` (it hibernates: no window, no webview); then
//   `--settings privacy` with LITTLEJOAN_SMOKE_SETTINGS_SCROLL=logins and
//   LITTLEJOAN_SMOKE_SETTINGS_LOGINS=remove: the settings window's profile (`webview-profile
//   settings=<path>`) must be the page's, the section scrolls into view for
//   `settings-privacy-logins.png` (every OS: the switches on Windows, the "isn't available" note on
//   macOS and Linux), and on Windows its "Remove saved passwords and form data…" and the question's
//   Remove are clicked (smoke hooks in settings_window.rs): `saved-logins-cleared ok`, while
//   hibernating; `settings-privacy-logins-removed.png` shows the status line.
// - "logins-on" (`"savePasswords": true`): the page reads back `password-autosave=true`, and so
//   does a sign-in popup (`?popup=oauth`, its "Sign in" clicked); Settings, opened after, reports what
//   its engine says without setting anything (`logins-seen …`, evidence that WebView2 shares the
//   switches across the profile; informational).
// The rules are Windows'; macOS and Linux must log no switch and no removal. The smoke never types a
// password and nothing reads the engine's stores (Ground Rule 26). smoke.mts supplies the driver;
// the rules are pure and unit-tested (smoke-logins.test.ts).
import { count, lastTitle, type LogEvent } from "./smoke-log.mts";
import type { LinksApp } from "./smoke-links.mts";
import { openerUrl } from "./smoke-popups.mts";
import { POPUP_READY_TITLE } from "../../tests/smoke/popup-logic.mjs";

/** What the runs need from smoke.mts. */
export interface LoginsDriver {
  os: string;
  /** The smoke page (`http://127.0.0.1:<port>/smoke.html`). */
  pageUrl: string;
  /** The fake sign-in provider's origin (the server's second port). */
  signinOrigin: string;
  launch(name: string, settings: Record<string, unknown>, extra?: Record<string, string>): Promise<LinksApp>;
  clickCentre(): void;
  screenshot(name: string): void;
  sleep(ms: number): Promise<void>;
}

export interface LoginsRun {
  /** The default launch: the page, hibernating, Settings → Privacy, the removal. */
  first: LogEvent[];
  /** The launch with `savePasswords: true`: the page, a popup, Settings. */
  on: LogEvent[];
  /** Whether the "logins-on" popup opened (its "Sign in" was clicked). */
  popupOpened: boolean;
}

/** `webview-profile <place>=<path>` lines: the last path per place. */
export function profiles(events: LogEvent[]): Record<string, string> {
  const found: Record<string, string> = {};
  for (const e of events.filter((x) => x.event === "webview-profile")) {
    const at = e.detail.indexOf("=");
    if (at > 0) {
      found[e.detail.slice(0, at)] = e.detail.slice(at + 1);
    }
  }
  return found;
}

/** The switches each webview's engine read back (`logins …`), or only reported (`logins-seen …`). */
export function switches(events: LogEvent[], event: "logins" | "logins-seen" = "logins"): string[] {
  return events.filter((e) => e.event === event).map((e) => e.detail);
}

const line = (save: boolean, autofill: boolean, place: string) => `password-autosave=${save} autofill=${autofill} webview=${place}`;

/**
 * The rules. Windows: the default page reads back saving off and autofill on; the settings window's
 * profile folder is the page's; removing while hibernating answers `ok`; with `savePasswords: true`
 * the page and the popup read back saving on; no switch failed. macOS and Linux: nothing to switch,
 * so no `logins` line and no removal; Settings → Privacy still scrolled for its screenshot.
 */
export function loginsFailures(run: LoginsRun, os: string): string[] {
  const failures: string[] = [];
  if (count(run.first, "settings-scrolled", "logins") === 0) {
    failures.push("logins: Settings → Privacy never scrolled to its saved passwords for the screenshot");
  }
  if (os !== "win32") {
    for (const [name, events] of [["logins", run.first], ["logins-on", run.on]] as const) {
      if (count(events, "logins") > 0 || count(events, "saved-logins-cleared") > 0) {
        failures.push(`${name}: this engine has no switch, yet the log says [${switches(events).join(", ")}]`);
      }
    }
    return failures;
  }
  const failed = [...switches(run.first), ...switches(run.on)].filter((d) => d.startsWith("failed"));
  if (failed.length > 0) {
    failures.push(`logins: switching failed: ${failed.join("; ")}`);
  }
  if (!switches(run.first).includes(line(false, true, "page"))) {
    failures.push(`logins: the default page read back [${switches(run.first).join(", ")}], not "${line(false, true, "page")}"`);
  }
  const paths = profiles(run.first);
  const { page, settings } = paths;
  if (!page || !settings || page.startsWith("unknown") || settings.startsWith("unknown")) {
    failures.push(`logins: no profile folder for both webviews (${JSON.stringify(paths)})`);
  } else if (page !== settings) {
    failures.push(`logins: the settings window's profile (${settings}) isn't the page's (${page}): removing wouldn't reach what the page saved`);
  }
  const hibernated = run.first.findIndex((e) => e.event === "hibernated");
  const opened = run.first.findIndex((e, i) => i > hibernated && e.event === "settings-window" && e.detail === "privacy");
  const cleared = run.first.findIndex((e) => e.event === "saved-logins-cleared");
  if (hibernated < 0 || opened < 0) {
    failures.push("logins: Settings → Privacy didn't open after the window hibernated");
  } else if (cleared < 0) {
    failures.push("logins: Remove saved passwords and form data logged no saved-logins-cleared");
  } else if (run.first[cleared]?.detail !== "ok") {
    failures.push(`logins: removing answered "${run.first[cleared]?.detail ?? ""}", not ok`);
  } else if (cleared < opened || run.first.slice(hibernated, cleared).some((e) => e.event === "page-built")) {
    failures.push("logins: the removal didn't happen while hibernating (a page was built first)");
  }
  if (!switches(run.on).includes(line(true, true, "page"))) {
    failures.push(`logins-on: with savePasswords the page read back [${switches(run.on).join(", ")}], not "${line(true, true, "page")}"`);
  }
  if (!run.popupOpened) {
    failures.push("logins-on: the sign-in popup never opened");
  } else if (!switches(run.on).includes(line(true, true, "popup"))) {
    failures.push(`logins-on: the popup read back [${switches(run.on).join(", ")}], not "${line(true, true, "popup")}"`);
  }
  return failures;
}

/** The default launch: the page, hibernate, Settings → Privacy (scrolled; on Windows the removal). */
async function firstRun(d: LoginsDriver): Promise<LogEvent[]> {
  const windows = d.os === "win32";
  const app = await d.launch("logins", { afterClose: "hibernate" }, { LITTLEJOAN_SMOKE_SETTINGS_SCROLL: "logins", LITTLEJOAN_SMOKE_SETTINGS_LOGINS: "remove" });
  await app.waitFor("logins: started", (e) => e.some((x) => x.event === "started"), 30_000);
  await d.sleep(1000);
  const from = app.events().length;
  await app.forward(["--show", d.pageUrl]);
  await app.waitFor("logins: the page", (e) => lastTitle(e.slice(from), "LJ-SMOKE") !== undefined, 30_000);
  if (windows) {
    await app.waitFor("logins: the page's profile", (e) => e.slice(from).some((x) => x.event === "webview-profile" && x.detail.startsWith("page=")), 10_000);
  }
  await d.sleep(1000);
  const hide = app.events().length;
  await app.forward(["--hide"]);
  await app.waitFor("logins: hibernated", (e) => count(e.slice(hide), "hibernated") > 0, 20_000);
  await d.sleep(1500);
  const open = app.events().length;
  await app.forward(["--settings", "privacy"]);
  await app.waitFor("logins: Settings → Privacy", (e) => count(e.slice(open), "settings-scrolled", "logins") > 0, 30_000);
  await d.sleep(1000);
  d.screenshot("settings-privacy-logins.png");
  if (windows) {
    await app.waitFor("logins: removed", (e) => count(e.slice(open), "saved-logins-cleared") > 0, 30_000);
    await d.sleep(800);
    d.screenshot("settings-privacy-logins-removed.png");
  }
  await app.quit();
  return app.events();
}

/** `savePasswords: true`: the page, a sign-in popup, then Settings (what its engine says). */
async function onRun(d: LoginsDriver): Promise<{ events: LogEvent[]; popupOpened: boolean }> {
  const windows = d.os === "win32";
  const app = await d.launch("logins-on", { savePasswords: true });
  await app.waitFor("logins-on: started", (e) => e.some((x) => x.event === "started"), 30_000);
  await d.sleep(1000);
  const from = app.events().length;
  await app.forward(["--show", openerUrl(d.pageUrl, d.signinOrigin, "oauth")]);
  await app.waitFor("logins-on: the opener", (e) => lastTitle(e.slice(from), POPUP_READY_TITLE) !== undefined, 30_000);
  await d.sleep(1500);
  const click = app.events().length;
  d.clickCentre();
  const popupOpened = await app.waitFor("logins-on: a popup opened", (e) => count(e.slice(click), "popup-opened") > 0, 15_000);
  if (popupOpened && windows) {
    await app.waitFor("logins-on: the popup's switches", (e) => e.slice(click).some((x) => x.event === "logins" && x.detail.endsWith("webview=popup")), 10_000);
  }
  await d.sleep(2000);
  const hide = app.events().length;
  await app.forward(["--hide"]);
  await app.waitFor("logins-on: closed", (e) => count(e.slice(hide), "hibernated") > 0, 20_000);
  const open = app.events().length;
  await app.forward(["--settings", "privacy"]);
  await app.waitFor("logins-on: Settings", (e) => count(e.slice(open), "settings-loaded") > 0, 30_000);
  if (windows) {
    await app.waitFor("logins-on: what Settings' engine says", (e) => count(e.slice(open), "logins-seen") > 0, 10_000);
  }
  await d.sleep(1000);
  await app.quit();
  return { events: app.events(), popupOpened };
}

/** Runs both launches. */
export async function loginsRun(d: LoginsDriver): Promise<LoginsRun> {
  const first = await firstRun(d);
  const on = await onRun(d);
  return { first, on: on.events, popupOpened: on.popupOpened };
}

/** What summary.json records of the runs. */
export function loginsSummary(run: LoginsRun, os: string) {
  const paths = profiles(run.first);
  return {
    switches: switches(run.first),
    switchesWithSaving: switches(run.on),
    settingsSaw: switches(run.on, "logins-seen"),
    profiles: paths,
    sameProfile: paths.page !== undefined && paths.page === paths.settings,
    cleared: run.first.find((e) => e.event === "saved-logins-cleared")?.detail ?? null,
    popupOpened: run.popupOpened,
    rules: loginsFailures(run, os),
  };
}
