// The smoke test's rules for element fullscreen switched off (LJ14.1). WebKitGTK older than 2.52
// (Ubuntu 22.04's 2.50.4) aborts the process as a page enters element fullscreen, even when the app
// declines the request (LJ14's experiment), so Little Joan switches WebKitGTK's `enable-fullscreen`
// off there and logs `fullscreen-off <version>` for each page window. On the job that runs such an
// engine (`linux-22.04`, LJ_SMOKE_FULLSCREEN=off) every launch that opened a page must log it with
// a version older than 2.52, and the window must never go fullscreen; everywhere else
// `fullscreen-off` must never appear. That nothing crashed and the pages still load and report is
// what the main run's and the fullscreen run's own rules check. Pure, unit-tested
// (smoke-fullscreen-off.test.ts).
import { lastTitle, type LogEvent } from "./smoke-log.mts";
import type { FsRunEvents } from "./smoke-fullscreen.mts";

/** The versions of a log's `fullscreen-off <major>.<minor>.<micro>` lines, in order. */
export function fullscreenOffVersions(events: LogEvent[]): string[] {
  return events.filter((e) => e.event === "fullscreen-off").map((e) => e.detail);
}

/** Whether `version` (`<major>.<minor>.<micro>`) is a WebKitGTK older than 2.52; false when unreadable. */
export function olderThan252(version: string): boolean {
  const match = /^(\d+)\.(\d+)\.\d+$/.exec(version);
  if (!match?.[1] || !match[2]) {
    return false;
  }
  const [major, minor] = [Number(match[1]), Number(match[2])];
  return major < 2 || (major === 2 && minor < 52);
}

/** The launches whose pages asked for fullscreen: the main run and the fullscreen run's two. */
export interface FullscreenOffLogs {
  main: LogEvent[];
  fullscreen: FsRunEvents;
}

/**
 * With `expected` (the engine aborts on element fullscreen): every launch logged `fullscreen-off`
 * with a version older than 2.52, no window went fullscreen (`fullscreen true`, `element-fullscreen
 * true`), and the main run's page never reported fullscreen. Without: no launch logged
 * `fullscreen-off`.
 */
export function fullscreenOffFailures(expected: boolean, logs: FullscreenOffLogs): string[] {
  const failures: string[] = [];
  const launches: [string, LogEvent[]][] = [
    ["main", logs.main],
    ["fullscreen", logs.fullscreen.main],
    ...(logs.fullscreen.site ? ([["siteModes", logs.fullscreen.site]] as [string, LogEvent[]][]) : []),
  ];
  for (const [name, events] of launches) {
    const versions = fullscreenOffVersions(events);
    if (!expected) {
      if (versions.length > 0) {
        failures.push(`${name}: logged "fullscreen-off ${versions[0] ?? ""}" on an engine whose fullscreen must stay on`);
      }
      continue;
    }
    if (versions.length === 0) {
      failures.push(`${name}: no "fullscreen-off" — element fullscreen must be off on this WebKitGTK`);
    }
    for (const version of versions.filter((v) => !olderThan252(v))) {
      failures.push(`${name}: "fullscreen-off ${version}", but that WebKitGTK isn't older than 2.52`);
    }
    for (const event of ["fullscreen", "element-fullscreen"]) {
      if (events.some((e) => e.event === event && e.detail === "true")) {
        failures.push(`${name}: "${event} true" — something went fullscreen though fullscreen is off`);
      }
    }
  }
  if (expected && lastTitle(logs.main, "LJ-SMOKE fullscreen=on") !== undefined) {
    failures.push("main: the page reported element fullscreen though fullscreen is off");
  }
  return failures;
}
