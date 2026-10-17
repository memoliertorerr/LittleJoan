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
  return failures;
}
