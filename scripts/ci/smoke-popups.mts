// The smoke test's popups run (LJ47–LJ48) and its rules: sign-in popups open in a small window of
// Little Joan's own, with no IPC. Its own launch; the smoke server serves tests/smoke/ on a second
// port too, so the fake provider (signin.html) is another origin, like a real one:
// - `?popup=oauth`: a click on the page's "Sign in" opens the sign-in page with
//   `window.open(url, "lj-signin", "width=480,height=600")` — the app logs `popup-opened
//   http://127.0.0.1:<port2>` (its title is where it is) and the popup's load; the page posts
//   `lj-signed-in` to its opener and calls `window.close()` (`popup-closed by-page`); the opener
//   reports `LJ-POPUP received=1 closed=1`; nothing is handed on. `popup.png` while it shows.
// - `?popup=blank`: a sized `about:blank` popup whose location the opener then sets: the same.
// - `?popup=oauth&stay=1`, then `--hide`: `popup-closed hibernate` before the page goes, and no
//   webview helper that came with the page and its popup outlives the window.
// - LJ48, with a popup left open: the opener navigated on its own origin keeps it; to another
//   origin (the second port) closes it (`popup-closed opener-left`). Then in a new popup the
//   history key does nothing (no `history-view`) and Ctrl+W closes the popup only (`popup-closed
//   user`, and no `closing` of the window) — Windows and Linux; best effort on macOS (Cmd+W).
// - Phase 22's security review — `?popup=spoof`: a sized popup asked for the second origin's
//   never-answered `hang` address, stopped, a fake form written into its blank page: its title
//   (`popup-title`) must never be the second origin (nothing of it committed) — built titled by the
//   opener —, and nothing of the second origin loads. `?popup=onblur`: a page that opens a sized
//   popup when it loses focus or is hidden, then `--hide`: nothing opens once the window closes
//   (the page's new windows are `new-window-dropped closing`).
// A plain target=_blank link opening no popup is the handoff run's (smoke-handoff.mts). macOS
// runners may not deliver a synthetic click to the page (as in the fullscreen run): there a run
// whose click asked for no new window is reported, not failed.
import { count, lastTitle, newPids, survivors, type LogEvent } from "./smoke-log.mts";
import type { LinksApp } from "./smoke-links.mts";
import { ONBLUR_ARMED_TITLE, POPUP_READY_TITLE, SPOOF_HOLD_MS } from "../../tests/smoke/popup-logic.mjs";

export interface PopupsDriver {
  os: string;
  /** The smoke page (`http://127.0.0.1:<port>/smoke.html`). */
  pageUrl: string;
  /** The fake provider's origin (`http://127.0.0.1:<port2>`). */
  signinOrigin: string;
  launch(name: string, settings: Record<string, unknown>): Promise<LinksApp>;
  clickCentre(): void;
  screenshot(name: string): void;
  sleep(ms: number): Promise<void>;
  /** The webview engine's helper processes, system-wide. */
  helperPids(): number[];
  /** Ctrl+<letter> (Cmd on macOS) in the window titled `title` (the popup's: its place). */
  pressInPopup(letter: string, title: string, pid: number): void;
}

export type PopupMode = "oauth" | "blank" | "spoof" | "onblur";

/** One sign-in: what the app logged from the click on. */
export interface PopupFlow {
  mode: PopupMode;
  events: LogEvent[];
}

/** A popup open when the window closes. */
export interface HibernateCheck {
  events: LogEvent[];
  /** Helper processes that came with the page and its popup. */
  helpers: number;
  /** How many of them were left, sampled after the close. */
  left: number[];
}

/** LJ48: the opener left its origin with a popup open. */
export interface OpenerCheck {
  /** From the click to the same-origin navigation's page being up. */
  sameOrigin: LogEvent[];
  /** From the navigation to another origin on. */
  otherOrigin: LogEvent[];
}

/** LJ48: Little Joan's keys in a popup. */
export interface KeysCheck {
  /** After the history key, before Ctrl+W. */
  afterHistoryKey: LogEvent[];
  /** After Ctrl+W. */
  afterClose: LogEvent[];
  /** Whether the popup had opened (else no key was pressed). */
  opened: boolean;
}

/** The review's spoof: what the app logged from the click on. */
export interface SpoofCheck {
  events: LogEvent[];
}

/** The review's blur popup: what the app logged from the `--hide` on (`before`: up to it). */
export interface OnBlurCheck {
  before: LogEvent[];
  afterHide: LogEvent[];
}

export interface PopupsRun {
  flows: PopupFlow[];
  spoof: SpoofCheck | null;
  hibernate: HibernateCheck | null;
  opener: OpenerCheck | null;
  keys: KeysCheck | null;
  onblur: OnBlurCheck | null;
}

/** The opener's address for `mode`. */
export function openerUrl(pageUrl: string, signinOrigin: string, mode: PopupMode, stay = false): string {
  return `${pageUrl}?popup=${mode}&at=${encodeURIComponent(signinOrigin)}${stay ? "&stay=1" : ""}`;
}

/** Shows the opener page and clicks its "Sign in"; returns the index of the log from the click. */
async function openPopup(d: PopupsDriver, app: LinksApp, mode: PopupMode, stay: boolean): Promise<number> {
  const from = app.events().length;
  await app.forward(["--show", openerUrl(d.pageUrl, d.signinOrigin, mode, stay)]);
  await app.waitFor(`popups: the ${mode} opener is up`, (e) => lastTitle(e.slice(from), POPUP_READY_TITLE) !== undefined, 30_000);
  await d.sleep(1500);
  const clicked = app.events().length;
  d.clickCentre();
  return clicked;
}

async function flow(d: PopupsDriver, app: LinksApp, mode: PopupMode): Promise<PopupFlow> {
  const from = await openPopup(d, app, mode, false);
  const opened = await app.waitFor(`popups: ${mode}: a popup opened`, (e) => count(e.slice(from), "popup-opened") > 0, 15_000);
  if (opened) {
    await app.waitFor(`popups: ${mode}: the popup loaded the sign-in page`, (e) => count(e.slice(from), "popup-loaded", d.signinOrigin) > 0, 15_000);
    if (mode === "oauth") {
      await d.sleep(600);
      d.screenshot("popup.png");
    }
    await app.waitFor(`popups: ${mode}: the opener heard back and saw it close`, (e) => lastTitle(e.slice(from), "LJ-POPUP received=1 closed=1") !== undefined, 20_000);
  }
  // Long enough for anything handed on by mistake to show up.
  await d.sleep(1500);
  return { mode, events: app.events().slice(from) };
}

async function spoof(d: PopupsDriver, app: LinksApp): Promise<SpoofCheck> {
  const from = await openPopup(d, app, "spoof", false);
  if (await app.waitFor("popups: spoof: a popup opened", (e) => count(e.slice(from), "popup-opened") > 0, 15_000)) {
    await app.waitFor("popups: spoof: the opener wrote into it", (e) => lastTitle(e.slice(from), "LJ-POPUP spoofed") !== undefined, 10_000);
    await d.sleep(SPOOF_HOLD_MS);
    await app.waitFor("popups: spoof: the opener closed it", (e) => lastTitle(e.slice(from), "LJ-POPUP received=0 closed=1") !== undefined, 10_000);
  }
  await d.sleep(1500);
  return { events: app.events().slice(from) };
}

async function onBlur(d: PopupsDriver, app: LinksApp): Promise<OnBlurCheck> {
  const from = app.events().length;
  await app.forward(["--show", openerUrl(d.pageUrl, d.signinOrigin, "onblur", true)]);
  await app.waitFor("popups: onblur: the page is armed", (e) => lastTitle(e.slice(from), ONBLUR_ARMED_TITLE) !== undefined, 30_000);
  await d.sleep(1500);
  // The page gets the keyboard, so the window's close takes it away (blur).
  d.clickCentre();
  await d.sleep(1500);
  const hide = app.events().length;
  await app.forward(["--hide"]);
  await app.waitFor("popups: onblur: closed", (e) => count(e.slice(hide), "hibernated") > 0, 20_000);
  await d.sleep(3000);
  return { before: app.events().slice(from, hide), afterHide: app.events().slice(hide) };
}

async function hibernate(d: PopupsDriver, app: LinksApp): Promise<HibernateCheck> {
  // At rest first: the helpers there are the ready bar's.
  const rested = app.events().length;
  await app.forward(["--hide"]);
  await app.waitFor("popups: at rest before the hibernate check", (e) => count(e.slice(rested), "hibernated") > 0, 20_000);
  await d.sleep(10_000);
  const before = d.helperPids();
  const from = await openPopup(d, app, "oauth", true);
  if (await app.waitFor("popups: a popup to leave open", (e) => count(e.slice(from), "popup-opened") > 0, 15_000)) {
    await app.waitFor("popups: it loaded the sign-in page", (e) => count(e.slice(from), "popup-loaded", d.signinOrigin) > 0, 15_000);
    await d.sleep(3000);
  }
  const ours = newPids(before, d.helperPids());
  await app.forward(["--hide"]);
  await app.waitFor("popups: closed with the popup open", (e) => count(e.slice(from), "hibernated") > 0, 20_000);
  const left: number[] = [];
  for (const wait of [10_000, 20_000]) {
    await d.sleep(wait);
    left.push(survivors(ours, d.helperPids()));
    if (left.at(-1) === 0) {
      break;
    }
  }
  return { events: app.events().slice(from), helpers: ours.length, left };
}

/** Opens a popup that stays, waits for its sign-in page; returns the log index of the click. */
async function openStaying(d: PopupsDriver, app: LinksApp, what: string): Promise<{ from: number; opened: boolean }> {
  const from = await openPopup(d, app, "oauth", true);
  const opened = await app.waitFor(`popups: ${what}: a popup to leave open`, (e) => count(e.slice(from), "popup-opened") > 0, 15_000);
  if (opened) {
    await app.waitFor(`popups: ${what}: it loaded the sign-in page`, (e) => count(e.slice(from), "popup-loaded", d.signinOrigin) > 0, 15_000);
    await d.sleep(3000);
  }
  return { from, opened };
}

async function openerLeaves(d: PopupsDriver, app: LinksApp): Promise<OpenerCheck> {
  const { from } = await openStaying(d, app, "opener");
  // The opener loads a page of its own origin: the popup stays.
  const same = app.events().length;
  await app.forward(["--show", `${openerUrl(d.pageUrl, d.signinOrigin, "oauth", true)}&again=1`]);
  await app.waitFor("popups: the opener reloaded on its origin", (e) => lastTitle(e.slice(same), POPUP_READY_TITLE) !== undefined, 30_000);
  await d.sleep(2000);
  const sameOrigin = app.events().slice(from);
  // Another origin: the popup closes.
  const other = app.events().length;
  await app.forward(["--show", `${d.signinOrigin}/smoke.html`]);
  await app.waitFor("popups: the popup closed with its opener gone", (e) => count(e.slice(other), "popup-closed", "opener-left") > 0, 15_000);
  await d.sleep(1500);
  return { sameOrigin, otherOrigin: app.events().slice(other) };
}

async function keysInPopup(d: PopupsDriver, app: LinksApp, historyKey: string): Promise<KeysCheck> {
  const { opened } = await openStaying(d, app, "keys");
  if (!opened) {
    return { afterHistoryKey: [], afterClose: [], opened };
  }
  const title = d.signinOrigin;
  const history = app.events().length;
  d.pressInPopup(historyKey, title, app.pid());
  await d.sleep(2500);
  const afterHistoryKey = app.events().slice(history);
  const close = app.events().length;
  d.pressInPopup("w", title, app.pid());
  await app.waitFor("popups: Ctrl+W closed the popup", (e) => count(e.slice(close), "popup-closed") > 0, 10_000);
  await d.sleep(3000);
  return { afterHistoryKey, afterClose: app.events().slice(close), opened };
}

export async function popupsRun(d: PopupsDriver, historyKey: string): Promise<PopupsRun> {
  const app = await d.launch("popups", {});
  await app.waitFor("popups: started and prewarmed", (e) => count(e, "prewarmed") > 0, 30_000);
  const flows: PopupFlow[] = [];
  for (const mode of ["oauth", "blank"] as const) {
    flows.push(await flow(d, app, mode));
  }
  const spoofed = await spoof(d, app);
  const closing = await hibernate(d, app);
  const opener = await openerLeaves(d, app);
  const keys = await keysInPopup(d, app, historyKey);
  const blurred = await onBlur(d, app);
  await app.quit();
  return { flows, spoof: spoofed, hibernate: closing, opener, keys, onblur: blurred };
}

/** Whether the click reached the page (it asked for a new window). */
const clicked = (events: LogEvent[]) => count(events, "new-window") > 0;

function flowFailures(flow: PopupFlow, os: string, signinOrigin: string): string[] {
  const what = `popups: ${flow.mode}`;
  if (!clicked(flow.events)) {
    return os === "darwin" ? [] : [`${what}: the click asked for no new window`];
  }
  const failures: string[] = [];
  const opened = flow.mode === "oauth" ? signinOrigin : "about:blank";
  if (count(flow.events, "popup-opened", opened) === 0) {
    const got = flow.events.filter((e) => e.event.startsWith("popup-") || e.event.startsWith("handoff") || e.event === "new-window-refused");
    failures.push(`${what}: no popup-opened ${opened} (got [${got.map((e) => `${e.event} ${e.detail}`.trim()).join(", ")}])`);
  }
  if (count(flow.events, "popup-loaded", signinOrigin) === 0) {
    failures.push(`${what}: the popup never loaded the sign-in page (popup-loaded ${signinOrigin})`);
  }
  if (lastTitle(flow.events, "LJ-POPUP received=1 closed=1") === undefined) {
    const last = flow.events.filter((e) => e.event === "title" && e.detail.startsWith("LJ-POPUP")).at(-1)?.detail ?? "nothing";
    failures.push(`${what}: the opener reported ${last}, not LJ-POPUP received=1 closed=1 (window.opener and window.close())`);
  }
  if (count(flow.events, "popup-closed", "by-page") === 0) {
    failures.push(`${what}: the popup's window.close() didn't close it (popup-closed by-page)`);
  }
  for (const wrong of ["handoff", "handoff-throttled", "new-window-refused", "popup-limited", "popup-failed"]) {
    if (count(flow.events, wrong) > 0) {
      failures.push(`${what}: a sign-in popup logged ${wrong}`);
    }
  }
  return failures;
}

function hibernateFailures(check: HibernateCheck | null, os: string): string[] {
  if (!check) {
    return ["popups: the hibernate check never ran"];
  }
  if (!clicked(check.events)) {
    return os === "darwin" ? [] : ["popups: hibernate: the click asked for no new window"];
  }
  const failures: string[] = [];
  const at = (event: string, detail = "") => check.events.findIndex((e) => e.event === event && e.detail.startsWith(detail));
  const closed = at("popup-closed", "hibernate");
  const pageClosed = at("page-closed");
  if (count(check.events, "popup-opened") === 0) {
    failures.push("popups: hibernate: no popup opened");
  } else if (closed < 0) {
    failures.push("popups: hibernate: closing the window didn't close its popup (popup-closed hibernate)");
  } else if (pageClosed >= 0 && pageClosed < closed) {
    failures.push("popups: hibernate: the page went before its popup");
  }
  if (at("hibernated") < 0) {
    failures.push("popups: hibernate: the window never closed");
  }
  const left = check.left.at(-1) ?? 0;
  if (left > 0) {
    failures.push(`popups: hibernate: ${left} of the ${check.helpers} webview helpers of the page and its popup outlived the window`);
  }
  return failures;
}

/** The page's place as a popup's title shows it (`http://127.0.0.1:<port>`). */
export function placeOf(pageUrl: string): string {
  return new URL(pageUrl).origin;
}

/**
 * The review's spoof: the popup opened, built titled by the opener, never titled by the second
 * origin (nothing of it committed), and nothing of the second origin loaded.
 */
export function spoofFailures(check: SpoofCheck | null, os: string, signinOrigin: string, pageUrl: string): string[] {
  if (!check) {
    return ["popups: the spoof check never ran"];
  }
  if (!clicked(check.events)) {
    return os === "darwin" ? [] : ["popups: spoof: the click asked for no new window"];
  }
  const failures: string[] = [];
  if (count(check.events, "popup-opened") === 0) {
    const got = check.events.filter((e) => e.event.startsWith("popup-") || e.event.startsWith("handoff") || e.event === "new-window-refused");
    return [`popups: spoof: no popup opened (got [${got.map((e) => `${e.event} ${e.detail}`.trim()).join(", ")}])`];
  }
  const titles = check.events.filter((e) => e.event === "popup-title").map((e) => e.detail);
  if (titles.length === 0) {
    failures.push("popups: spoof: the popup's title was never logged (popup-title)");
  } else if (titles[0] !== placeOf(pageUrl)) {
    failures.push(`popups: spoof: the popup was built titled ${titles[0]}, not its opener's ${placeOf(pageUrl)}`);
  }
  if (titles.some((title) => title.startsWith(signinOrigin))) {
    failures.push(`popups: spoof: the popup was titled ${signinOrigin} with nothing of it loaded (titles: ${titles.join(", ")})`);
  }
  if (count(check.events, "popup-loaded", signinOrigin) > 0) {
    failures.push(`popups: spoof: something of ${signinOrigin} loaded in the popup (its hang address must never answer)`);
  }
  const report = check.events.filter((e) => e.event === "title" && e.detail.startsWith("LJ-POPUP spoofed")).at(-1)?.detail;
  if (report !== "LJ-POPUP spoofed opened=1 written=1") {
    failures.push(`popups: spoof: the opener reported ${report ?? "nothing"}, not that it wrote into the popup`);
  }
  return failures;
}

/** The review's blur popup: nothing opens once the window closes. */
export function onBlurFailures(check: OnBlurCheck | null): string[] {
  if (!check) {
    return ["popups: the onblur check never ran"];
  }
  const failures: string[] = [];
  if (lastTitle(check.before, ONBLUR_ARMED_TITLE) === undefined) {
    failures.push("popups: onblur: the page never armed");
  }
  const opened = count(check.afterHide, "popup-opened") + count(check.afterHide, "popup-pending");
  if (opened > 0) {
    failures.push(`popups: onblur: ${opened} popup(s) opened while the window closed`);
  }
  const handed = count(check.afterHide, "handoff-admitted") + count(check.afterHide, "handoff");
  if (handed > 0) {
    failures.push("popups: onblur: a new window of the closing page was handed on");
  }
  if (count(check.afterHide, "hibernated") === 0) {
    failures.push("popups: onblur: the window never closed");
  }
  return failures;
}

/** LJ48: a same-origin navigation keeps the popup, another origin closes it. */
export function openerFailures(check: OpenerCheck | null, os: string): string[] {
  if (!check) {
    return ["popups: the opener check never ran"];
  }
  if (!clicked(check.sameOrigin)) {
    return os === "darwin" ? [] : ["popups: opener: the click asked for no new window"];
  }
  const failures: string[] = [];
  if (count(check.sameOrigin, "popup-opened") === 0) {
    return ["popups: opener: no popup opened"];
  }
  if (count(check.sameOrigin, "popup-closed") > 0) {
    failures.push("popups: opener: a navigation on the opener's own origin closed its popup");
  }
  if (count(check.otherOrigin, "popup-closed", "opener-left") === 0) {
    failures.push("popups: opener: the opener left for another origin and its popup stayed (popup-closed opener-left)");
  }
  return failures;
}

/** LJ48: in a popup Ctrl+W closes the popup only, and no other key of Little Joan's acts. */
export function keysFailures(check: KeysCheck | null, os: string): string[] {
  if (!check) {
    return ["popups: the keys check never ran"];
  }
  // macOS: synthetic keys reach the app best effort (and the click may not have opened anything).
  if (os === "darwin" || !check.opened) {
    return os === "darwin" || check.opened ? [] : ["popups: keys: no popup opened"];
  }
  const failures: string[] = [];
  if (count(check.afterHistoryKey, "history-view") > 0 || count(check.afterHistoryKey, "popup-closed") > 0) {
    failures.push("popups: keys: the history key acted in a popup (only Close may)");
  }
  if (count(check.afterClose, "popup-closed", "user") === 0) {
    const got = check.afterClose.filter((e) => e.event.startsWith("popup") || e.event === "closing").map((e) => `${e.event} ${e.detail}`.trim());
    failures.push(`popups: keys: Ctrl+W in the popup didn't close it (popup-closed user; got [${got.join(", ")}])`);
  }
  if (count(check.afterClose, "closing") > 0 || count(check.afterClose, "hibernated") > 0) {
    failures.push("popups: keys: Ctrl+W in the popup closed the Little Joan window too");
  }
  return failures;
}

export function popupsFailures(run: PopupsRun, os: string, signinOrigin: string, pageUrl: string): string[] {
  const failures = run.flows.flatMap((flow) => flowFailures(flow, os, signinOrigin));
  if (run.flows.length !== 2) {
    failures.push(`popups: ${run.flows.length} sign-in flows ran, not 2`);
  }
  return failures.concat(
    spoofFailures(run.spoof, os, signinOrigin, pageUrl),
    hibernateFailures(run.hibernate, os),
    openerFailures(run.opener, os),
    keysFailures(run.keys, os),
    onBlurFailures(run.onblur),
  );
}

export function popupsSummary(run: PopupsRun, os: string, signinOrigin: string, pageUrl: string): Record<string, unknown> {
  const lines = (events: LogEvent[]) =>
    events
      .filter(
        (e) =>
          e.event.startsWith("popup") ||
          e.event.startsWith("handoff") ||
          e.event.startsWith("new-window") ||
          e.event === "closing" ||
          e.event === "hibernated" ||
          (e.event === "title" && e.detail.startsWith("LJ-POPUP")),
      )
      .map((e) => `${e.event} ${e.detail}`.trim());
  return {
    flows: run.flows.map((flow) => ({ mode: flow.mode, clicked: clicked(flow.events), log: lines(flow.events) })),
    spoof: run.spoof ? { clicked: clicked(run.spoof.events), log: lines(run.spoof.events) } : null,
    onblur: run.onblur ? { before: lines(run.onblur.before), afterHide: lines(run.onblur.afterHide) } : null,
    hibernate: run.hibernate
      ? { clicked: clicked(run.hibernate.events), log: lines(run.hibernate.events), helpers: run.hibernate.helpers, left: run.hibernate.left }
      : null,
    opener: run.opener ? { sameOrigin: lines(run.opener.sameOrigin), otherOrigin: lines(run.opener.otherOrigin) } : null,
    keys: run.keys
      ? {
          opened: run.keys.opened,
          afterHistoryKey: run.keys.afterHistoryKey.filter((e) => e.event.startsWith("popup") || e.event === "history-view").map((e) => `${e.event} ${e.detail}`.trim()),
          afterClose: lines(run.keys.afterClose).concat(run.keys.afterClose.filter((e) => e.event === "closing" || e.event === "hibernated").map((e) => e.event)),
        }
      : null,
    rules: popupsFailures(run, os, signinOrigin, pageUrl),
  };
}
