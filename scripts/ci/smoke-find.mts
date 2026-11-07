// The smoke test's find run (LJ49) and its rules. Its own launch over the smoke page's `?find=1`
// (lj-needle three times, in plain text): Ctrl/Cmd+F opens the toolbar's find strip
// (`find-open`); typing "lj-needle" there finds it with the page's engine — `find-result matches=3
// active=1` where the engine counts (Windows, Linux), `find-result found=true` on macOS —; Enter
// goes to the next match (`find-action next`, then `active=2` where the engine counts); Esc closes
// the strip (`find-closed user`). Screenshot find.png with the strip open and the matches found.
// Keys are driven on Windows and Linux; on macOS (best effort, as every synthetic key there) the
// View menu must show Find… on ⌘F, and the rest applies once the key reached the app. smoke.mts
// supplies the driver; the rules are pure and unit-tested (smoke-find.test.ts).
import type { LogEvent } from "./smoke-log.mts";
import type { SiteApp } from "./smoke-sitemode.mts";
import { menuKeys } from "./smoke-keymap.mts";
import { FIND_NEEDLE, FIND_NEEDLES, FIND_READY_TITLE } from "../../tests/smoke/smoke-logic.mjs";

/** What the run needs from smoke.mts. */
export interface FindDriver {
  os: string;
  pageUrl: string;
  launch(name: string, settings: Record<string, unknown>): Promise<SiteApp>;
  screenshot(name: string): void;
  sleep(ms: number): Promise<void>;
  typeText(text: string, pid: number, activate: boolean): void;
  pressKeys(keys: string[], pid: number, activate: boolean): void;
  pressShortcut(letter: string, pid: number): void;
}

export interface FindRun {
  events: LogEvent[];
}

/** What a typed needle's result says: a count and the first match where the engine counts. */
export function foundLine(os: string, active = 1): string {
  return os === "darwin" ? "found=true" : `matches=${FIND_NEEDLES} active=${active}`;
}

/** The index of the first event at or after `from` that `test` accepts (-1: none). */
function after(events: LogEvent[], from: number, test: (e: LogEvent) => boolean): number {
  if (from < 0) {
    return -1;
  }
  const at = events.slice(from).findIndex(test);
  return at < 0 ? -1 : from + at;
}

/** The `find-result` details from `from` on. */
function results(events: LogEvent[], from: number): string[] {
  return events
    .slice(Math.max(0, from))
    .filter((e) => e.event === "find-result")
    .map((e) => e.detail);
}

/**
 * The rules. Windows and Linux: the strip opens, the typed needle reaches Rust (`find-action start
 * length=9`), the engine finds three and the first active, Enter asks for the next and the engine
 * (or, on Linux, Little Joan's count) says the second, Esc closes the strip; the engine never says
 * it has no find. macOS: the menu shows Find… on ⌘F; once the key reached the app, the needle is
 * found (`found=true`), Enter asks for the next, Esc closes.
 */
export function findFailures(run: FindRun, os: string): string[] {
  const events = run.events;
  const mac = os === "darwin";
  const failures: string[] = [];
  if (mac) {
    const menu = menuKeys(events);
    if (menu["find"] !== "⌘F") {
      failures.push(`find: the menu showed Find… as ${menu["find"] ?? "nothing"}, not ⌘F`);
    }
  }
  const opened = events.findIndex((e) => e.event === "find-open");
  if (opened < 0) {
    if (!mac) {
      failures.push("find: Ctrl+F didn't open the find strip (no find-open)");
    }
    return failures;
  }
  if (results(events, 0).includes("unavailable")) {
    failures.push("find: the page's engine has no find (find-result unavailable)");
  }
  const want = `start length=${FIND_NEEDLE.length}`;
  let started = -1;
  events.forEach((e, i) => {
    if (i > opened && e.event === "find-action" && e.detail === want) {
      started = i;
    }
  });
  if (started < 0) {
    if (!mac) {
      failures.push(`find: the typed "${FIND_NEEDLE}" never reached Rust (no "find-action ${want}")`);
    }
    return failures;
  }
  const found = after(events, started, (e) => e.event === "find-result" && e.detail === foundLine(os));
  if (found < 0) {
    failures.push(`find: after typing "${FIND_NEEDLE}" no "find-result ${foundLine(os)}" (got [${results(events, started).join(", ")}])`);
  }
  const next = after(events, started, (e) => e.event === "find-action" && e.detail === "next");
  if (next < 0) {
    failures.push("find: Enter never asked for the next match (no find-action next)");
  } else if (!mac && after(events, next, (e) => e.event === "find-result" && e.detail === foundLine(os, 2)) < 0) {
    failures.push(`find: after Enter no "find-result ${foundLine(os, 2)}" (got [${results(events, next).join(", ")}])`);
  }
  if (after(events, started, (e) => e.event === "find-closed" && e.detail === "user") < 0) {
    failures.push("find: Esc didn't close the strip (no find-closed user)");
  }
  return failures;
}

/** Runs the find launch. */
export async function findRun(d: FindDriver): Promise<FindRun> {
  const app = await d.launch("find", {});
  await app.waitFor("find: started", (e) => e.some((x) => x.event === "started"), 30_000);
  await d.sleep(1500);
  const from = app.events().length;
  await app.forward(["--show", `${d.pageUrl}?find=1`]);
  await app.waitFor("find: the ?find=1 page", (e) => e.slice(from).some((x) => x.event === "title" && x.detail === FIND_READY_TITLE), 30_000);
  await d.sleep(1500);
  const pid = app.pid();
  d.pressShortcut("f", pid);
  const opened = await app.waitFor("find: Ctrl/Cmd+F opened the strip", (e) => e.slice(from).some((x) => x.event === "find-open"), 10_000);
  if (!opened) {
    d.screenshot("find.png");
    await app.quit();
    return { events: app.events() };
  }
  await d.sleep(800);
  // The window is in front already (the shortcut went to it): no second activation, which could
  // hand the keyboard back to the page.
  d.typeText(FIND_NEEDLE, pid, false);
  await app.waitFor(
    `find: ${foundLine(d.os)}`,
    (e) => e.slice(from).some((x) => x.event === "find-result" && x.detail === foundLine(d.os)),
    15_000,
  );
  await d.sleep(1000);
  d.screenshot("find.png");
  const nextFrom = app.events().length;
  d.pressKeys(["Return"], pid, false);
  await app.waitFor(
    "find: Enter went to the next match",
    (e) => e.slice(nextFrom).some((x) => (d.os === "darwin" ? x.event === "find-action" && x.detail === "next" : x.event === "find-result" && x.detail === foundLine(d.os, 2))),
    10_000,
  );
  await d.sleep(800);
  d.pressKeys(["Escape"], pid, false);
  await app.waitFor("find: Esc closed the strip", (e) => e.slice(from).some((x) => x.event === "find-closed"), 10_000);
  await d.sleep(500);
  await app.quit();
  return { events: app.events() };
}

/** What summary.json records of the run. */
export function findSummary(run: FindRun, os: string) {
  const events = run.events;
  return {
    engine: events.find((e) => e.event === "find-engine")?.detail ?? null,
    opened: events.some((e) => e.event === "find-open"),
    actions: events.filter((e) => e.event === "find-action").map((e) => e.detail),
    results: results(events, 0),
    closed: events.filter((e) => e.event === "find-closed").map((e) => e.detail),
    rules: findFailures(run, os),
  };
}
