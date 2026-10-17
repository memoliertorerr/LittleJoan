// Parsing and checking the app's smoke log (LITTLEJOAN_SMOKE_LOG, see src-tauri/src/smoke.rs).
// Pure functions, unit-tested; scripts/ci/smoke.mts drives the installed app and uses these.

export interface LogEvent {
  millis: number;
  event: string;
  detail: string;
}

/** Parses `<millis> <event>[ <detail>]` lines; malformed lines are skipped. */
export function parseLog(text: string): LogEvent[] {
  const events: LogEvent[] = [];
  for (const line of text.split(/\r?\n/)) {
    const match = /^(\d+) (\S+)(?: (.*))?$/.exec(line);
    if (match?.[1] && match[2]) {
      events.push({ millis: Number(match[1]), event: match[2], detail: match[3] ?? "" });
    }
  }
  return events;
}

/** How many events match `event` (and, if given, a detail starting with `detailPrefix`). */
export function count(events: LogEvent[], event: string, detailPrefix = ""): number {
  return events.filter((e) => e.event === event && e.detail.startsWith(detailPrefix)).length;
}

/** The detail of the last `title` event starting with `prefix`, if any. */
export function lastTitle(events: LogEvent[], prefix: string): string | undefined {
  return events.filter((e) => e.event === "title" && e.detail.startsWith(prefix)).at(-1)?.detail;
}

/** Codec support from the smoke page's ready title ("LJ-SMOKE ready h264=mse vp9=no …"). */
export function parseCodecs(title: string | undefined): Record<string, string> {
  const codecs: Record<string, string> = {};
  for (const match of (title ?? "").matchAll(/(\w+)=(\S+)/g)) {
    if (match[1] && match[2]) {
      codecs[match[1]] = match[2];
    }
  }
  return codecs;
}

/** What the smoke page must report after a `--size` resize (see smoke-logic.mjs). */
export interface Viewport {
  width: number;
  height: number;
  ua: "phone" | "desktop";
}

/** The page reported exactly this viewport and agent kind at some point. */
export function sawViewport(events: LogEvent[], want: Viewport): boolean {
  const prefix = `LJ-SMOKE viewport=${want.width}x${want.height} ua=${want.ua} `;
  return events.some((e) => e.event === "title" && e.detail.startsWith(prefix));
}

export interface Expectations {
  /** The window itself must report fullscreen after the click (Windows, Linux). */
  windowFullscreen: boolean;
  /** The page must report element fullscreen after the click. */
  pageFullscreen: boolean;
  /** Viewports (and agent kinds) the page must have reported after the scripted resizes. */
  viewports?: Viewport[];
}

export interface Measurements {
  /**
   * Webview helper processes that served the page (they appeared while it was shown) and are
   * still alive after hibernating — the worst sample. Other apps' helpers don't count.
   */
  pageHelpersLeft: number;
  /** What `--start-at-login on`, then `off`, left in the OS's login items (omitted: not checked). */
  loginEntry?: LoginEntryCheck;
}

/** A login item found in the OS: where it lives and the command it runs. */
export interface LoginEntry {
  /** Registry value name, plist file or .desktop file. */
  where: string;
  command: string;
}

export interface LoginEntryCheck {
  /** Entries pointing at the installed app after `--start-at-login on`. */
  afterOn: LoginEntry[];
  /** Entries pointing at the installed app after `--start-at-login off` (should be none). */
  afterOff: LoginEntry[];
}

/** Values in `reg query <key>` output: `    <name>    REG_SZ    <data>` lines. */
export function parseRegValues(output: string): { name: string; type: string; data: string }[] {
  const values: { name: string; type: string; data: string }[] = [];
  for (const line of output.split(/\r?\n/)) {
    const match = /^ {4}(.+?) {4}(REG_\w+) {4}(.*)$/.exec(line);
    if (match?.[1] && match[2] && match[3] !== undefined) {
      values.push({ name: match[1], type: match[2], data: match[3] });
    }
  }
  return values;
}

/** The program a LaunchAgent plist runs: the first string of its ProgramArguments array. */
export function plistProgram(plist: string): string | undefined {
  const array = /<key>ProgramArguments<\/key>\s*<array>([\s\S]*?)<\/array>/.exec(plist)?.[1];
  const first = /<string>([\s\S]*?)<\/string>/.exec(array ?? "")?.[1];
  return first
    ?.replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&apos;", "'")
    .replaceAll("&amp;", "&");
}

/** The command an XDG autostart .desktop file runs (its `Exec=` line). */
export function desktopExec(desktop: string): string | undefined {
  return /^Exec=(.*)$/m.exec(desktop)?.[1]?.trim();
}

/**
 * Whether a login entry's command runs one of `installed` (the installed binary, and its
 * canonical path): the path alone, quoted or not, optionally followed by arguments. Windows
 * paths compare without case.
 */
export function pointsAt(command: string, installed: readonly string[], ignoreCase: boolean): boolean {
  const fold = (text: string) => (ignoreCase ? text.toLowerCase() : text);
  const cmd = fold(command.trim());
  return installed.some((path) => {
    const p = fold(path);
    return [p, `"${p}"`].some((program) => cmd === program || cmd.startsWith(`${program} `));
  });
}

/** Process ids in `now` that weren't in `before` (the helpers that appeared meanwhile). */
export function newPids(before: readonly number[], now: readonly number[]): number[] {
  const old = new Set(before);
  return now.filter((pid) => !old.has(pid));
}

/** How many of `ours` are still in `now`. */
export function survivors(ours: readonly number[], now: readonly number[]): number {
  const alive = new Set(now);
  return ours.filter((pid) => alive.has(pid)).length;
}

/** Checks a full smoke run. Returns the list of failures (empty = pass). */
export function evaluate(events: LogEvent[], expect: Expectations, measured: Measurements): string[] {
  const failures: string[] = [];
  const ordered: [string, string][] = [
    ["started", ""],
    ["shortcut-registered", ""],
    ["window-created", ""],
    ["title", "LJ-SMOKE ready"],
    ["hibernated", ""],
  ];
  let from = 0;
  for (const [event, prefix] of ordered) {
    const index = events.findIndex((e, i) => i >= from && e.event === event && e.detail.startsWith(prefix));
    if (index < 0) {
      failures.push(`missing "${event}${prefix ? ` ${prefix}` : ""}" (in order)`);
    } else {
      from = index + 1;
    }
  }
  for (const want of expect.viewports ?? []) {
    if (!sawViewport(events, want)) {
      failures.push(`the page never saw a ${want.width}x${want.height} viewport with a ${want.ua} user agent`);
    }
  }
  if (count(events, "bar-loaded") === 0) {
    failures.push('the URL bar never loaded ("bar-loaded")');
  }
  if (count(events, "bar-url", "http://127.0.0.1") === 0) {
    failures.push("the URL bar was never told the smoke page's address");
  }
  if (count(events, "started") !== 1) {
    failures.push(`expected exactly one "started" (single instance), saw ${count(events, "started")}`);
  }
  if (expect.windowFullscreen && count(events, "fullscreen", "true") === 0) {
    failures.push('the window never reported "fullscreen true" after the click');
  }
  if (expect.pageFullscreen && lastTitle(events, "LJ-SMOKE fullscreen=on") === undefined) {
    failures.push("the page never reported element fullscreen after the click");
  }
  if (count(events, "window-created") < 2 || count(events, "hibernated") < 2) {
    failures.push("summoning again after hibernating did not create and destroy a second window");
  }
  if (measured.pageHelpersLeft > 0) {
    failures.push(`webview helper processes of the page left after hibernating: ${measured.pageHelpersLeft}`);
  }
  if (measured.loginEntry) {
    failures.push(...startAtLoginFailures(events, measured.loginEntry));
  }
  if (count(events, "first-run") > 0) {
    failures.push("the main run was a first run (first-run), so it didn't start hibernating");
  }
  return failures;
}

/** The first-launch check: a launch on an empty data folder, then a relaunch on the same one. */
export interface FirstRunCheck {
  /** Events of the launch on the empty folder (shown, then `--hide`). */
  first: LogEvent[];
  /** Events of the relaunch on the same folder, 5 s after it started. */
  relaunch: LogEvent[];
  /** settings.json existed in the folder after the first launch. */
  settingsSaved: boolean;
}

/**
 * A first launch opens the window once with the welcome (the start page read `firstRun`), saves
 * the default settings, and the next launch on the same folder hibernates.
 */
export function firstRunFailures(check: FirstRunCheck): string[] {
  const failures: string[] = [];
  let from = 0;
  for (const event of ["started", "first-run", "window-created", "hibernated"]) {
    const index = check.first.findIndex((e, i) => i >= from && e.event === event);
    if (index < 0) {
      failures.push(`first launch: missing "${event}" (in order)`);
    } else {
      from = index + 1;
    }
  }
  if (count(check.first, "bar-loaded") === 0) {
    failures.push('first launch: the URL bar never loaded ("bar-loaded")');
  }
  if (count(check.first, "welcome") === 0) {
    failures.push('first launch: the start page never showed the welcome ("welcome")');
  }
  if (count(check.first, "window-created") !== 1) {
    failures.push(`first launch: expected one window, saw ${count(check.first, "window-created")}`);
  }
  if (!check.settingsSaved) {
    failures.push("first launch: settings.json wasn't saved");
  }
  if (count(check.relaunch, "started") !== 1) {
    failures.push(`relaunch: expected one "started", saw ${count(check.relaunch, "started")}`);
  }
  if (count(check.relaunch, "first-run") > 0) {
    failures.push("relaunch: it was a first run again");
  }
  if (count(check.relaunch, "window-created") > 0) {
    failures.push("relaunch: a window opened (it should hibernate)");
  }
  return failures;
}

/**
 * `--start-at-login on` then `off`: the app reports `true` then `false` (read back from the OS),
 * an entry pointing at the installed app exists in between and is gone after, and neither flag
 * opened a window.
 */
export function startAtLoginFailures(events: LogEvent[], check: LoginEntryCheck): string[] {
  const failures: string[] = [];
  const on = events.findIndex((e) => e.event === "start-at-login" && e.detail === "true");
  const off = events.findIndex((e, i) => i > on && e.event === "start-at-login" && e.detail === "false");
  if (on < 0 || off < 0) {
    failures.push('expected "start-at-login true", then "start-at-login false"');
  }
  for (const failed of events.filter((e) => e.event === "start-at-login-failed")) {
    failures.push(`start at login failed: ${failed.detail}`);
  }
  if (check.afterOn.length === 0) {
    failures.push("no login entry pointing at the installed app after --start-at-login on");
  }
  if (check.afterOff.length > 0) {
    failures.push(`login entry left after --start-at-login off: ${check.afterOff.map((e) => e.where).join(", ")}`);
  }
  const first = events.findIndex((e) => e.event.startsWith("start-at-login"));
  if (first >= 0 && events.some((e, i) => i > first && e.event === "window-created")) {
    failures.push("--start-at-login opened a window");
  }
  return failures;
}

/** What the smoke page found in a persistence check: "LJ-PERSIST p=<0|1> s=<0|1> l=<0|1>". */
export interface Persisted {
  /** The persistent cookie (Max-Age a year) still held the token. */
  p: boolean;
  /** The session cookie (no expiry) still held the token. */
  s: boolean;
  /** localStorage still held the token. */
  l: boolean;
}

/** Reads a `LJ-PERSIST p=… s=… l=…` title; anything else is undefined. */
export function parsePersist(title: string | undefined): Persisted | undefined {
  const match = /^LJ-PERSIST p=([01]) s=([01]) l=([01])$/.exec(title ?? "");
  if (!match) {
    return undefined;
  }
  return { p: match[1] === "1", s: match[2] === "1", l: match[3] === "1" };
}

/** A persistence report as text for failure messages, e.g. "p=1 s=0 l=1". */
function persistText(persisted: Persisted): string {
  const bit = (on: boolean) => (on ? "1" : "0");
  return `p=${bit(persisted.p)} s=${bit(persisted.s)} l=${bit(persisted.l)}`;
}

/** The longest a `--quit` may come after the page reported `LJ-PERSIST set`. */
export const QUIT_AFTER_SET_MAX_MS = 1000;

/** The login-persistence check: set, close → summon, check; set, quit at once, relaunch, check. */
export interface PersistCheck {
  /** The page's report after `?persist=set`, `--hide`, `--show ?persist=check` (same app run). */
  afterClose: string | undefined;
  /** The page's report after a new `?persist=set`, `--quit` within 1 s, relaunch, check. */
  afterQuit: string | undefined;
  /** The app's process ended by itself after `--quit` (no signal needed). */
  quitExited: boolean;
  /** Events of the relaunch on the same data folder. */
  relaunch: LogEvent[];
}

/**
 * Cookies and storage outlive closing the window (all three), and quitting right after they
 * were set (persistent cookie and localStorage; the session cookie ends with the app run, as in
 * browsers). `main` is the first app run's log: its last `LJ-PERSIST set` title must be followed
 * by `quit` within QUIT_AFTER_SET_MAX_MS, or the quit check proves nothing.
 */
export function persistFailures(main: LogEvent[], check: PersistCheck): string[] {
  const failures: string[] = [];
  const afterClose = parsePersist(check.afterClose);
  if (!afterClose) {
    failures.push(`close → summon: the page never reported (got ${check.afterClose ?? "nothing"})`);
  } else if (!afterClose.p || !afterClose.s || !afterClose.l) {
    const lost = [!afterClose.p && "the persistent cookie", !afterClose.s && "the session cookie", !afterClose.l && "localStorage"];
    failures.push(`close → summon lost ${lost.filter(Boolean).join(", ")} (${persistText(afterClose)}, want p=1 s=1 l=1)`);
  }

  const setIndex = main.findLastIndex((e) => e.event === "title" && e.detail === "LJ-PERSIST set");
  const quitIndex = main.findIndex((e, i) => i > setIndex && e.event === "quit");
  const set = main[setIndex];
  const quit = main[quitIndex];
  if (setIndex < 0 || !set) {
    failures.push('quit → relaunch: the page never reported "LJ-PERSIST set"');
  } else if (quitIndex < 0 || !quit) {
    failures.push('quit → relaunch: the app never logged "quit" after the set');
  } else if (quit.millis - set.millis > QUIT_AFTER_SET_MAX_MS) {
    failures.push(`quit → relaunch: the quit came ${quit.millis - set.millis} ms after the set (at most ${QUIT_AFTER_SET_MAX_MS})`);
  }
  if (!check.quitExited) {
    failures.push("--quit did not end the app's process");
  }
  if (count(check.relaunch, "started") !== 1) {
    failures.push(`quit → relaunch: expected one "started", saw ${count(check.relaunch, "started")}`);
  }
  if (count(check.relaunch, "first-run") > 0) {
    failures.push("quit → relaunch: it was a first run (the data folder lost its settings)");
  }
  const afterQuit = parsePersist(check.afterQuit);
  if (!afterQuit) {
    failures.push(`quit → relaunch: the page never reported (got ${check.afterQuit ?? "nothing"})`);
  } else {
    const lost = [!afterQuit.p && "the persistent cookie", !afterQuit.l && "localStorage"].filter(Boolean);
    if (lost.length > 0) {
      failures.push(`quit → relaunch lost ${lost.join(", ")} (${persistText(afterQuit)}, want p=1 s=0 l=1)`);
    }
    if (afterQuit.s) {
      failures.push(`quit → relaunch kept the session cookie (${persistText(afterQuit)}); it must end with the app run`);
    }
  }
  return failures;
}
