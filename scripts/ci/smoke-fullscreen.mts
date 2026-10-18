// The smoke test's fullscreen run (LJ36) and its rules. The run is its own launch (so the main
// run's memory numbers don't move): the smoke page in `?fs=both` mode at 390×600 (phone agent) and
// at 720×600 (desktop agent), each button clicked in turn — "Element fullscreen" (requestFullscreen
// on a container, as desktop YouTube), its Exit 2 s later, then "Phone player" (m.youtube.com's
// way under an iPhone agent) and its own exit —, then a second launch whose settings say
// `"siteModes": { "127.0.0.1": "desktop" }` at 390×600. Only clicks, never keys: the owner's bug
// was about clicks. scripts/ci/smoke.mts supplies the driver; the parsing and the rules are pure
// and unit-tested (smoke-fullscreen.test.ts). The page reports `LJ-FS …` titles
// (tests/smoke/fullscreen-logic.mjs); the app logs `page-screen` (where the page is on the screen,
// for the clicks), `page-bounds` / `page-visible` (every layout of the page) and on macOS
// `wk-fullscreen <not|entering|in|exiting>` (WebKit's own fullscreen state).
import { FS_BUTTONS, centre } from "../../tests/smoke/fullscreen-logic.mjs";
import type { LogEvent } from "./smoke-log.mts";

/** One `LJ-FS <path> …` report of the smoke page. */
export interface FsReport {
  path: "element" | "phone";
  width: number;
  ua: "phone" | "desktop";
  entered: boolean;
  heldMs: number;
  exitedByPage: boolean;
  events: string[];
  /** Set when the phone player couldn't play (no captureStream, no metadata). */
  unavailable?: string;
}

/** Reads a title's `LJ-FS element|phone …` report; anything else is undefined. */
export function parseFsReport(title: string): FsReport | undefined {
  const head = /^LJ-FS (element|phone) width=(\d+) ua=(phone|desktop) /.exec(title);
  if (!head?.[1] || !head[2] || !head[3]) {
    return undefined;
  }
  const base = { path: head[1] as FsReport["path"], width: Number(head[2]), ua: head[3] as FsReport["ua"] };
  const rest = title.slice(head[0].length);
  const unavailable = /^unavailable=(\S+)$/.exec(rest)?.[1];
  if (unavailable) {
    return { ...base, entered: false, heldMs: 0, exitedByPage: false, events: [], unavailable };
  }
  const fields = /^entered=([01]) held_ms=(\d+) exited_by_page=([01]) events=(\S*)$/.exec(rest);
  if (!fields?.[1] || !fields[2] || !fields[3] || fields[4] === undefined) {
    return undefined;
  }
  return {
    ...base,
    entered: fields[1] === "1",
    heldMs: Number(fields[2]),
    exitedByPage: fields[3] === "1",
    events: fields[4] ? fields[4].split(",") : [],
  };
}

/** Every report in a log, in order. */
export function fsReports(events: LogEvent[]): FsReport[] {
  return events.flatMap((e) => {
    const report = e.event === "title" ? parseFsReport(e.detail) : undefined;
    return report ? [report] : [];
  });
}

/** How a report's fullscreen was asked for: the iOS video fullscreen, or element fullscreen. */
export function route(report: FsReport): "ios" | "element" {
  return report.events.includes("webkit-enter") ? "ios" : "element";
}

/** Where the page is on the screen (`page-screen`): physical pixels, and the scale. */
export interface PageScreen {
  x: number;
  y: number;
  width: number;
  height: number;
  scale: number;
}

/** Reads a `page-screen <x>,<y>,<w>,<h> scale=<s>` event; anything else is undefined. */
export function parsePageScreen(event: LogEvent): PageScreen | undefined {
  if (event.event !== "page-screen") {
    return undefined;
  }
  const match = /^(-?[\d.]+),(-?[\d.]+),([\d.]+),([\d.]+) scale=([\d.]+)$/.exec(event.detail);
  if (!match?.[1] || !match[2] || !match[3] || !match[4] || !match[5]) {
    return undefined;
  }
  const [x, y, width, height, scale] = match.slice(1, 6).map(Number) as [number, number, number, number, number];
  return { x, y, width, height, scale };
}

/** The page's latest place on the screen. */
export function lastPageScreen(events: LogEvent[]): PageScreen | undefined {
  for (let i = events.length - 1; i >= 0; i -= 1) {
    const screen = events[i] && parsePageScreen(events[i] as LogEvent);
    if (screen) {
      return screen;
    }
  }
  return undefined;
}

export interface Box {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * Where to click the middle of `box` (fractions of the page's viewport): screen pixels for
 * Windows (SetCursorPos) and Linux (xdotool), points for macOS (CGEvent), rounded.
 */
export function clickPoint(screen: PageScreen, box: Box, os: string): { x: number; y: number } {
  const mid = centre(box);
  const unit = os === "darwin" && screen.scale > 0 ? screen.scale : 1;
  return {
    x: Math.round((screen.x + mid.x * screen.width) / unit),
    y: Math.round((screen.y + mid.y * screen.height) / unit),
  };
}

/**
 * The index of the page's `LJ-FS ready width=<width> ua=<ua>` title that came after the page's
 * last `page-started` (a reload after a phone-mode switch starts the page again), or -1.
 */
export function readyIndex(events: LogEvent[], width: number, ua: "phone" | "desktop"): number {
  const started = events.findLastIndex((e) => e.event === "page-started");
  const want = `LJ-FS ready width=${width} ua=${ua}`;
  const ready = events.findLastIndex((e) => e.event === "title" && e.detail.startsWith("LJ-FS "));
  return ready > started && events[ready]?.detail === want ? ready : -1;
}

/** The first report for `path` after event `from`. */
export function reportAfter(events: LogEvent[], from: number, path: FsReport["path"]): FsReport | undefined {
  return fsReports(events.slice(from)).find((report) => report.path === path);
}

/** What the run does at one size. */
export interface FsSize {
  width: number;
  /** The agent the page must report before anything is clicked. */
  ua: "phone" | "desktop";
  paths: FsReport["path"][];
  /** Screenshots 1 s after a path's click, under <out>/fs/. */
  shots: Partial<Record<FsReport["path"], string>>;
}

export const FS_SIZES: readonly FsSize[] = [
  { width: 390, ua: "phone", paths: ["element", "phone"], shots: { element: "fs/element-390.png", phone: "fs/phone-390.png" } },
  { width: 720, ua: "desktop", paths: ["element", "phone"], shots: { element: "fs/element-720.png" } },
];

/** The second launch: the smoke page's host on the desktop agent, at a phone width. */
export const SITE_MODES = { "127.0.0.1": "desktop" } as const;
export const SITE_SIZE: FsSize = { width: 390, ua: "desktop", paths: ["phone"], shots: { phone: "fs/desktop-site-390.png" } };

/** One launch of the app, as smoke.mts drives it. */
export interface FsApp {
  forward(args: string[]): Promise<void>;
  events(): LogEvent[];
  waitFor(what: string, test: (events: LogEvent[]) => boolean, timeoutMs: number): Promise<boolean>;
  quit(): Promise<void>;
}

/** What the run needs from smoke.mts. */
export interface FsDriver {
  os: string;
  /** The smoke page's address (no query). */
  pageUrl: string;
  /** Launches the app on a fresh data folder whose settings.json gets `settings` too. */
  launch(name: string, settings: Record<string, unknown>): Promise<FsApp>;
  /** Clicks a screen point (see clickPoint), or the screen's centre. */
  clickAt(target: { x: number; y: number } | "centre"): void;
  screenshot(name: string): void;
  sleep(ms: number): Promise<void>;
}

/** The logs of the fullscreen launches. */
export interface FsRunEvents {
  main: LogEvent[];
  /** The siteModes launch (undefined: not run). */
  site?: LogEvent[];
}

/** Waits until the page reports `ready` at `width` with `ua` and stays that way for 1.5 s. */
async function settled(d: FsDriver, app: FsApp, from: number, size: FsSize): Promise<boolean> {
  const label = `fullscreen: ready at ${size.width} (${size.ua} agent)`;
  for (let tries = 0; tries < 6; tries += 1) {
    if (!(await app.waitFor(label, (e) => readyIndex(e.slice(from), size.width, size.ua) >= 0, 30_000))) {
      return false;
    }
    const seen = readyIndex(app.events().slice(from), size.width, size.ua);
    await d.sleep(1500);
    if (readyIndex(app.events().slice(from), size.width, size.ua) === seen) {
      return true;
    }
  }
  return false;
}

/** Clicks `path`'s button, screenshots 1 s later, clicks its exit 2 s after the click, waits for the report. */
async function attempt(d: FsDriver, app: FsApp, size: FsSize, path: FsReport["path"]): Promise<void> {
  const from = app.events().length;
  const screen = lastPageScreen(app.events());
  if (!screen) {
    console.log(`fullscreen: no page-screen line, so no click on ${path}`);
    return;
  }
  d.clickAt(clickPoint(screen, FS_BUTTONS[path], d.os));
  await d.sleep(1000);
  const shot = size.shots[path];
  if (shot) {
    d.screenshot(shot);
  }
  await d.sleep(1000);
  // Already reported (fullscreen ended, or the player can't play): there is no exit to click.
  if (!reportAfter(app.events(), from, path)) {
    d.clickAt("centre");
  }
  await app.waitFor(`fullscreen: ${path} at ${size.width} reported`, (e) => reportAfter(e, from, path) !== undefined, 9_000);
  // Windows leaves window fullscreen, macOS WebKit's fullscreen Space: let it settle.
  await d.sleep(1500);
}

async function atSize(d: FsDriver, app: FsApp, size: FsSize): Promise<void> {
  const from = app.events().length;
  await app.forward(["--size", `${size.width}x600`]);
  if (!(await settled(d, app, from, size))) {
    return;
  }
  for (const path of size.paths) {
    await attempt(d, app, size, path);
  }
}

async function launchWithPage(d: FsDriver, name: string, settings: Record<string, unknown>): Promise<FsApp> {
  const app = await d.launch(name, settings);
  await app.waitFor(`${name}: started`, (e) => e.some((x) => x.event === "started"), 30_000);
  await d.sleep(1500);
  await app.forward(["--show", `${d.pageUrl}?fs=both`]);
  await app.waitFor(`${name}: the page`, (e) => e.some((x) => x.event === "title" && x.detail.startsWith("LJ-FS ready")), 30_000);
  return app;
}

/** Runs the fullscreen launch, and with `siteModes` the second one. */
export async function fullscreenRun(d: FsDriver, siteModes: boolean): Promise<FsRunEvents> {
  const app = await launchWithPage(d, "fullscreen", {});
  for (const size of FS_SIZES) {
    await atSize(d, app, size);
  }
  await app.quit();
  if (!siteModes) {
    return { main: app.events() };
  }
  const site = await launchWithPage(d, "fullscreen-site", { siteModes: SITE_MODES });
  await atSize(d, site, SITE_SIZE);
  await site.quit();
  return { main: app.events(), site: site.events() };
}

/**
 * The `page-bounds` / `page-visible` lines logged while WebKit's fullscreen (macOS:
 * `wk-fullscreen`) wasn't `not` — layout touching the page inside WebKit's fullscreen window.
 */
export function layoutDuringWebKitFullscreen(events: LogEvent[]): LogEvent[] {
  let state = "not";
  return events.filter((e) => {
    if (e.event === "wk-fullscreen") {
      state = e.detail;
      return false;
    }
    return state !== "not" && (e.event === "page-bounds" || e.event === "page-visible");
  });
}

/** WebKit's fullscreen states in the order the log saw them (macOS). */
export function webkitStates(events: LogEvent[]): string[] {
  return events.filter((e) => e.event === "wk-fullscreen").map((e) => e.detail);
}

/**
 * The rules (turned on by LJ36's fix):
 * - every element-fullscreen path enters, holds until the page's own exit and leaves when asked —
 *   required on Windows and Linux; on macOS whenever the click reached the page (it reported);
 *   the phone player under an iPhone agent (the iOS route) is informational;
 * - the siteModes launch reports the desktop agent at 390 on every OS;
 * - macOS: no page-bounds / page-visible line while WebKit's fullscreen isn't `not`.
 */
export function fullscreenFailures(run: FsRunEvents, os: string): string[] {
  const failures: string[] = [];
  const launches: { name: string; events: LogEvent[]; sizes: readonly FsSize[] }[] = [
    { name: "fullscreen", events: run.main, sizes: FS_SIZES },
    ...(run.site ? [{ name: "siteModes", events: run.site, sizes: [SITE_SIZE] }] : []),
  ];
  for (const launch of launches) {
    const reports = fsReports(launch.events);
    for (const size of launch.sizes) {
      for (const path of size.paths) {
        const where = `${launch.name}: ${path} at ${size.width}`;
        const report = reports.find((r) => r.path === path && r.width === size.width);
        // The phone player takes the iOS route under an iPhone agent (macOS, Linux at a phone width).
        const ios = path === "phone" && size.ua === "phone" && os !== "win32";
        if (!report) {
          if (os !== "darwin" && !ios) {
            failures.push(`${where}: the page never reported (the click didn't reach it)`);
          }
          continue;
        }
        if (report.unavailable || route(report) === "ios") {
          continue;
        }
        if (!report.entered) {
          failures.push(`${where}: never entered fullscreen (events ${report.events.join(",")})`);
        } else if (!report.exitedByPage && report.events.includes("timeout")) {
          failures.push(`${where}: still fullscreen ${report.heldMs} ms after entering, though its Exit was clicked`);
        } else if (!report.exitedByPage || report.heldMs < 1000) {
          failures.push(`${where}: left fullscreen by itself after ${report.heldMs} ms (events ${report.events.join(",")})`);
        }
      }
    }
  }
  if (run.site && !run.site.some((e) => e.event === "title" && e.detail === `LJ-FS ready width=${SITE_SIZE.width} ua=desktop`)) {
    failures.push(`siteModes: the page never reported the desktop agent at ${SITE_SIZE.width}`);
  }
  if (os === "darwin") {
    for (const launch of launches) {
      const touched = layoutDuringWebKitFullscreen(launch.events);
      if (touched.length > 0) {
        failures.push(`${launch.name}: the page was laid out during WebKit's fullscreen (${touched.map((e) => `${e.event} ${e.detail}`).join("; ")})`);
      }
    }
  }
  return failures;
}

/** What summary.json records of the run (informational in LJ36's first commit). */
export function fullscreenSummary(run: FsRunEvents, os: string) {
  return {
    reports: fsReports(run.main),
    webkit: webkitStates(run.main),
    windowEvents: run.main.filter((e) => e.event === "wk-window").map((e) => e.detail),
    siteModes: run.site ? { reports: fsReports(run.site), webkit: webkitStates(run.site) } : null,
    rules: fullscreenFailures(run, os),
  };
}
