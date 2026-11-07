// The smoke test's popups run (LJ47) and its rules: sign-in popups open in a small window of
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
// A plain target=_blank link opening no popup is the handoff run's (smoke-handoff.mts). macOS
// runners may not deliver a synthetic click to the page (as in the fullscreen run): there a run
// whose click asked for no new window is reported, not failed.
import { count, lastTitle, newPids, survivors, type LogEvent } from "./smoke-log.mts";
import type { LinksApp } from "./smoke-links.mts";
import { POPUP_READY_TITLE } from "../../tests/smoke/popup-logic.mjs";

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
}

export type PopupMode = "oauth" | "blank";

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

export interface PopupsRun {
  flows: PopupFlow[];
  hibernate: HibernateCheck | null;
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

export async function popupsRun(d: PopupsDriver): Promise<PopupsRun> {
  const app = await d.launch("popups", {});
  await app.waitFor("popups: started and prewarmed", (e) => count(e, "prewarmed") > 0, 30_000);
  const flows: PopupFlow[] = [];
  for (const mode of ["oauth", "blank"] as const) {
    flows.push(await flow(d, app, mode));
  }
  const closing = await hibernate(d, app);
  await app.quit();
  return { flows, hibernate: closing };
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

export function popupsFailures(run: PopupsRun, os: string, signinOrigin: string): string[] {
  const failures = run.flows.flatMap((flow) => flowFailures(flow, os, signinOrigin));
  if (run.flows.length !== 2) {
    failures.push(`popups: ${run.flows.length} sign-in flows ran, not 2`);
  }
  return failures.concat(hibernateFailures(run.hibernate, os));
}

export function popupsSummary(run: PopupsRun, os: string, signinOrigin: string): Record<string, unknown> {
  const lines = (events: LogEvent[]) =>
    events
      .filter((e) => e.event.startsWith("popup") || e.event.startsWith("handoff") || e.event === "new-window" || e.event === "new-window-refused" || (e.event === "title" && e.detail.startsWith("LJ-POPUP")))
      .map((e) => `${e.event} ${e.detail}`.trim());
  return {
    flows: run.flows.map((flow) => ({ mode: flow.mode, clicked: clicked(flow.events), log: lines(flow.events) })),
    hibernate: run.hibernate
      ? { clicked: clicked(run.hibernate.events), log: lines(run.hibernate.events), helpers: run.hibernate.helpers, left: run.hibernate.left }
      : null,
    rules: popupsFailures(run, os, signinOrigin),
  };
}
