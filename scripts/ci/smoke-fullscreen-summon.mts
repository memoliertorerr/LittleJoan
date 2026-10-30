// The rules of the fullscreen run's summon during the page's element fullscreen (LJ19.5; the run
// itself is smoke-fullscreen.mts's `summonRun`): the shortcut's command while the page is full screen
// must leave the fullscreen — the page goes on —, open the command bar ready for typing, and a second
// press close the window; and the main run's `--show <url>` during its page's fullscreen must leave
// it first too. Pure and unit-tested (smoke-fullscreen-summon.test.ts).
import type { LogEvent } from "./smoke-log.mts";

/** The fullscreen launch's page pings the smoke server as `?ping=fs` (the summon run's "the page goes on"). */
export const FS_PING = "fs";

/** What the summon during the page's element fullscreen saw (LJ19.5, `summonRun`). */
export interface FsSummon {
  /** The page went into element fullscreen after its click (`element-fullscreen true`, macOS `wk-fullscreen in`). */
  entered: boolean;
  /** The fullscreen launch's log from the click on. */
  events: LogEvent[];
  /** Where in `events` the first `--toggle` went out, and the second (-1: not sent). */
  first: number;
  second: number;
  /** The unix ms each `--toggle` went out at (0: not sent). */
  firstAt: number;
  secondAt: number;
  /** The page's pings (unix ms). */
  pings: number[];
}

/** Whether `events` show the page entering element fullscreen (macOS: WebKit's own state). */
export function enteredFullscreen(events: LogEvent[], os: string): boolean {
  return events.some((e) => (os === "darwin" ? e.event === "wk-fullscreen" && e.detail === "in" : e.event === "element-fullscreen" && e.detail === "true"));
}

/**
 * The main run's `--show <url>` comes while the smoke page is in element fullscreen. Since LJ19.5 a
 * summon leaves the page's fullscreen first, on every OS, and only then focuses the window and
 * loads the URL (`summon-fullscreen left`). On macOS that is also LJ19.4-fix4's guard: ordered
 * front during WebKit's fullscreen, the window joins WebKit's full-screen Space, the navigation
 * closes that Space with it inside, and the Dock may then refuse every app's full screen for
 * minutes — the fullscreen launches a few minutes later fail with it. Only when the page entered
 * the fullscreen (the click reached it).
 */
export function summonDuringFullscreenFailures(main: LogEvent[], os: string): string[] {
  if (!enteredFullscreen(main, os)) {
    return [];
  }
  if (main.some((e) => e.event === "summon-fullscreen" && e.detail.startsWith("left"))) {
    return [];
  }
  const said = main.filter((e) => e.event === "summon-fullscreen").map((e) => e.detail);
  return [`fullscreen: the summon with a URL during the page's fullscreen didn't leave it first (${said.length > 0 ? `summon-fullscreen ${said.join(", ")}` : "no summon-fullscreen line"})`];
}

/**
 * The summon during the page's fullscreen (LJ19.5), once the page entered it — required on every
 * OS (macOS: when the click reached the page):
 * - the first press doesn't close the window: the app leaves the fullscreen itself
 *   (`summon-fullscreen left`) — after WebKit gave the page back on macOS (`wk-fullscreen not`
 *   before it: nothing ordered meanwhile) —, the page's fullscreen ends (`element-fullscreen
 *   false`, macOS `wk-fullscreen not`), then the command bar opens (`overlay true`);
 * - the page went on — not reloaded (no `page-started`), still pinging a second after the press;
 * - the letter typed reached the command bar's field (`typed=1`) — Windows and Linux; macOS's
 *   synthetic keys are best effort, so it is recorded there;
 * - the second press closes the window (`closing`).
 */
export function fsSummonFailures(summon: FsSummon | undefined, os: string): string[] {
  if (!summon) {
    return [];
  }
  const mac = os === "darwin";
  if (!summon.entered) {
    return mac ? [] : ["fullscreen summon: the page never entered element fullscreen after its click"];
  }
  const failures: string[] = [];
  const firstEvents = summon.events.slice(summon.first, summon.second < 0 ? undefined : summon.second);
  const at = (test: (e: LogEvent) => boolean) => firstEvents.findIndex(test);
  const closed = at((e) => e.event === "closing");
  const left = at((e) => e.event === "summon-fullscreen" && e.detail.startsWith("left"));
  const ended = at((e) => (mac ? e.event === "wk-fullscreen" && e.detail === "not" : e.event === "element-fullscreen" && e.detail === "false"));
  const overlay = firstEvents.findIndex((e, i) => i > Math.max(ended, left) && e.event === "overlay" && e.detail === "true");
  if (closed >= 0) {
    failures.push("fullscreen summon: the first press closed the window instead of leaving the page's fullscreen");
  }
  if (left < 0) {
    const gaveUp = firstEvents.find((e) => e.event === "summon-fullscreen" && e.detail.startsWith("gave-up"));
    failures.push(`fullscreen summon: the app didn't leave the page's fullscreen (${gaveUp ? `summon-fullscreen ${gaveUp.detail}` : "no summon-fullscreen left line"})`);
  } else if (ended < 0 || (mac && ended > left)) {
    failures.push(`fullscreen summon: summon-fullscreen left came ${ended < 0 ? "without the page's fullscreen ending" : "before WebKit gave the page back (wk-fullscreen not)"}`);
  }
  if (overlay < 0) {
    failures.push("fullscreen summon: the command bar didn't open after the fullscreen ended (no overlay true)");
  }
  if (firstEvents.some((e) => e.event === "page-started")) {
    failures.push("fullscreen summon: the page was loaded again (page-started after the press)");
  }
  const stop = summon.secondAt > 0 ? summon.secondAt : Number.POSITIVE_INFINITY;
  if (!summon.pings.some((t) => t > summon.firstAt + 1000 && t < stop)) {
    failures.push("fullscreen summon: the page stopped pinging after the press (it didn't go on)");
  }
  if (!mac && !firstEvents.some((e) => e.event === "suggestions" && e.detail.endsWith("typed=1"))) {
    failures.push("fullscreen summon: the letter typed after the press didn't reach the command bar's field");
  }
  if (summon.second < 0 || !summon.events.slice(summon.second).some((e) => e.event === "closing")) {
    failures.push("fullscreen summon: the second press didn't close the window");
  }
  return failures;
}

/** What summary.json records of the summon during the page's fullscreen. */
export function fsSummonSummary(summon: FsSummon | undefined, os: string) {
  if (!summon) {
    return null;
  }
  const lines = (from: number, to?: number) =>
    summon.events
      .slice(from, to)
      .filter((e) => ["summon-fullscreen", "wk-fullscreen", "element-fullscreen", "fullscreen", "overlay", "closing", "hibernated", "window-created", "window-front", "page-started"].includes(e.event) || (e.event === "suggestions" && e.detail.includes("typed=")))
      .map((e) => `${e.millis - (summon.events[0]?.millis ?? e.millis)} ${e.event} ${e.detail}`.trim());
  return {
    entered: summon.entered,
    firstPress: lines(summon.first, summon.second < 0 ? undefined : summon.second),
    secondPress: summon.second < 0 ? [] : lines(summon.second),
    pingsAfterFirstPress: summon.pings.filter((t) => t > summon.firstAt && (summon.secondAt <= 0 || t < summon.secondAt)).length,
    pingsAfterSecondPress: summon.secondAt > 0 ? summon.pings.filter((t) => t > summon.secondAt + 2000).length : null,
    rules: fsSummonFailures(summon, os),
  };
}
