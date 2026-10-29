// The smoke test's rest runs (LJ19.4) and their rules: what stays at rest after a close in each
// "After closing" mode, each mode on a launch of its own.
//   ready      (the default) the hidden window and the bar are built ~3 s after the launch
//              (`prewarmed`); a page that pings the smoke server every 250 ms (smoke.html's
//              ?ping=<id>) stops pinging within PINGS_STOP_MS of a close, and the engine's web
//              processes that came with it go, the bar's stay; the memory at rest is measured; the
//              real shortcut, then keys typed right after it, land in the command bar's field
//              (`typed=<n>`: none lost); the bar's web process killed while ready → the engine says so
//              (`bar-gone`) and the next summon builds everything anew (`released bar-gone`, a cold
//              `window-created` and its complete summon-timing line).
//   hibernate  nothing is built after the launch and nothing of ours runs in the engine at rest;
//              after a close the pings stop and every web process goes (LJ9's rule, as before — the
//              engine's network process may stay: WebKitGTK keeps its web context for the app's
//              lifetime, WebKit its Networking and GPU services; neither runs a page).
//   ready15    Stay ready for 15 minutes with the minutes shortened (LITTLEJOAN_SMOKE_READY_MS, read
//              only with the smoke log): the ready window goes by itself (`released timer`), the next
//              summon is cold, and its close rests again until the timer.
// Memory at rest is the resident memory of the app and the engine processes it started, summed —
// and their own memory, which counts shared libraries once (Windows: private bytes, Linux: Pss,
// macOS: the physical footprint Activity Monitor shows) — (the numbers go into summary.json's
// restRuns and the notes). Pure parts unit-tested
// (smoke-rest.test.ts); smoke.mts supplies the driver (process lists, the ping log, keys).
import type { LogEvent } from "./smoke-log.mts";
import type { SiteApp } from "./smoke-sitemode.mts";
import { summonTimings, type SummonTiming } from "./smoke-timing.mts";

/** The modes of "After closing". */
export type RestMode = "ready" | "hibernate" | "ready15";

/** One engine process of the app's (smoke.mts finds them per OS). */
export interface EngineProc {
  pid: number;
  /** What it is: WebContent / msedgewebview2 / WebKitWebProcess, … */
  name: string;
  /** A web (renderer) process — what runs a page's scripts. */
  web: boolean;
  rssKb: number;
  /** Its own memory: private bytes (Windows), Pss (Linux), the physical footprint (macOS); -1 unknown. */
  ownKb: number;
}

/** What the rest runs need from smoke.mts. */
export interface RestDriver {
  os: string;
  pageUrl: string;
  /** Launches on a fresh data folder with these settings and extra environment. */
  launch(name: string, settings: Record<string, unknown>, env: Record<string, string>): Promise<SiteApp>;
  sleep(ms: number): Promise<void>;
  /** The unix ms of every ping the smoke server got from `?ping=<id>`. */
  pings(id: string): number[];
  /** The app's engine processes now (macOS: the WebKit services that weren't there in `before`). */
  engine(appPid: number, before: readonly number[]): EngineProc[];
  /** Every WebKit / WebView2 process id on the machine (macOS's services aren't the app's children). */
  enginePids(): number[];
  /** The app's own resident memory, KB. */
  appRssKb(pid: number): number;
  /** The app's own memory as `EngineProc.ownKb` counts it, KB (-1 unknown). */
  appOwnKb(pid: number): number;
  kill(pids: readonly number[]): void;
  pressSummonShortcut(): number | null;
  typeText(text: string, pid: number, activate: boolean): void;
  screenshot(name: string): void;
}

/** Pings may still arrive this long after the close began (the page's unload, a request in flight). */
export const PINGS_STOP_MS = 2000;
/** What the typed-keys check types right after the shortcut. */
export const TYPED_KEYS = "ljkeys";
/** How long after the shortcut the keys go out — a quick typist. */
export const KEYS_AFTER_MS = 150;
/** Stay ready for 15 minutes, shortened for CI. */
export const READY15_SMOKE_MS = 6000;

/** One sample of what runs at rest. */
export interface RestSample {
  at: string;
  web: number;
  engine: number;
  /** The app's and its engine processes' resident memory, MB. */
  memoryMb: number;
  /** … and their own memory (shared libraries counted once), MB; null when a part is unknown. */
  ownMb: number | null;
}

/** A close of the ping page: when it began (the app's `closing` line), the pings after, what was left. */
export interface CloseCheck {
  closingAt: number | null;
  lastPing: number | null;
  pingsBefore: number;
  pingsAfter: number;
  /** The web processes the page came with (not there at rest before the summon). */
  pageWeb: number[];
  /** How many of them were still there, per sample after the close. */
  left: Record<string, number>;
}

export interface RestRun {
  mode: RestMode;
  events: LogEvent[];
  samples: RestSample[];
  close: CloseCheck | null;
  /** ready: the shortcut, then TYPED_KEYS — the `typed=` counts the command bar saw after. */
  typed: { sent: boolean; counts: number[] } | null;
  /** ready: the bar's web processes killed at rest (how many), and what the next summon logged. */
  killed: { pids: number; after: LogEvent[] } | null;
}

function round1(x: number): number {
  return Math.round(x * 10) / 10;
}

/** A sample of the engine and the memory now. */
export function sample(at: string, appKb: number, engine: readonly EngineProc[], appOwnKb = -1): RestSample {
  const own = [appOwnKb, ...engine.map((p) => p.ownKb)];
  return {
    at,
    web: engine.filter((p) => p.web).length,
    engine: engine.length,
    memoryMb: round1((appKb + engine.reduce((sum, p) => sum + p.rssKb, 0)) / 1024),
    ownMb: own.some((kb) => kb < 0) ? null : round1(own.reduce((sum, kb) => sum + kb, 0) / 1024),
  };
}

/** The `typed=<n>` counts of the `suggestions` lines in `events`. */
export function typedCounts(events: readonly LogEvent[]): number[] {
  return events.flatMap((e) => {
    const match = e.event === "suggestions" ? /typed=(\d+)$/.exec(e.detail) : null;
    return match?.[1] ? [Number(match[1])] : [];
  });
}

/** The pings after `from` (the page's own, since its summon). */
function pingsSince(times: readonly number[], from: number): number[] {
  return times.filter((t) => t >= from);
}

/** Runs one rest launch. */
export async function restRun(d: RestDriver, mode: RestMode): Promise<RestRun> {
  const settings = mode === "ready" ? {} : { afterClose: mode };
  const env: Record<string, string> = mode === "ready15" ? { LITTLEJOAN_SMOKE_READY_MS: String(READY15_SMOKE_MS) } : {};
  const before = d.enginePids();
  const app = await d.launch(`rest-${mode}`, settings, env);
  const run: RestRun = { mode, events: [], samples: [], close: null, typed: null, killed: null };
  await app.waitFor(`rest ${mode}: started`, (e) => e.some((x) => x.event === "started"), 30_000);
  const take = (at: string): void => {
    const pid = app.pid();
    run.samples.push(sample(at, d.appRssKb(pid), d.engine(pid, before), d.appOwnKb(pid)));
  };
  if (mode === "hibernate") {
    // Nothing is built after the launch.
    await d.sleep(6000);
  } else {
    await app.waitFor(`rest ${mode}: prewarmed`, (e) => e.some((x) => x.event === "prewarmed"), 20_000);
    await d.sleep(3000);
  }
  take("start");
  if (mode === "ready15") {
    await app.waitFor("rest ready15: released by its timer", (e) => e.some((x) => x.event === "released" && x.detail === "timer"), READY15_SMOKE_MS + 15_000);
    await d.sleep(2000);
    take("after-timer");
  }

  // The ping page: summoned, pinging, closed.
  const atRest = d.engine(app.pid(), before).filter((p) => p.web).map((p) => p.pid);
  const id = `rest-${mode}`;
  const summonedAt = Date.now();
  await app.forward(["--show", `${d.pageUrl}?ping=${id}`]);
  await app.waitFor(`rest ${mode}: the ping page`, () => pingsSince(d.pings(id), summonedAt).length >= 4, 30_000);
  await d.sleep(1000);
  take("page");
  const pageWeb = d
    .engine(app.pid(), before)
    .filter((p) => p.web && !atRest.includes(p.pid))
    .map((p) => p.pid);
  d.screenshot(`rest-${mode}-page.png`);
  const closedBefore = app.events().filter((e) => e.event === "hibernated").length;
  await app.forward(["--hide"]);
  await app.waitFor(`rest ${mode}: closed`, (e) => e.filter((x) => x.event === "hibernated").length > closedBefore, 20_000);
  const left: Record<string, number> = {};
  let waited = 0;
  for (const at of [2, 10, 30]) {
    await d.sleep((at - waited) * 1000);
    waited = at;
    const now = new Set(d.engine(app.pid(), before).map((p) => p.pid));
    left[`${at}s`] = pageWeb.filter((pid) => now.has(pid)).length;
    if (at === 10) {
      take("after-close");
    }
  }
  const closing = app.events().findLast((e) => e.event === "closing");
  const pings = pingsSince(d.pings(id), summonedAt);
  const closingAt = closing?.millis ?? null;
  run.close = {
    closingAt,
    lastPing: pings.at(-1) ?? null,
    pingsBefore: closingAt === null ? pings.length : pings.filter((t) => t <= closingAt).length,
    pingsAfter: closingAt === null ? 0 : pings.filter((t) => t > closingAt + PINGS_STOP_MS).length,
    pageWeb,
    left,
  };

  if (mode === "ready") {
    // Keys typed right after the real shortcut land in the field, none lost.
    const from = app.events().length;
    const sent = d.pressSummonShortcut() !== null;
    await d.sleep(KEYS_AFTER_MS);
    d.typeText(TYPED_KEYS, app.pid(), false);
    await app.waitFor("rest ready: the typed keys", (e) => typedCounts(e.slice(from)).includes(TYPED_KEYS.length), 8000);
    await d.sleep(800);
    d.screenshot("rest-ready-typed.png");
    run.typed = { sent, counts: typedCounts(app.events().slice(from)) };
    const shown = app.events().slice(from).some((e) => e.event === "window-created");
    if (shown) {
      const closed = app.events().filter((e) => e.event === "hibernated").length;
      await app.forward(["--hide"]);
      await app.waitFor("rest ready: closed after typing", (e) => e.filter((x) => x.event === "hibernated").length > closed, 20_000);
    }
    await d.sleep(2000);
    // The bar's web process ends while ready: the next summon builds everything anew.
    const bars = d.engine(app.pid(), before).filter((p) => p.web).map((p) => p.pid);
    d.kill(bars);
    await app.waitFor("rest ready: the engine said the bar is gone", (e) => e.some((x) => x.event === "bar-gone"), 15_000);
    await d.sleep(1000);
    const killedFrom = app.events().length;
    await app.forward(["--show"]);
    await app.waitFor(
      "rest ready: the summon after the kill",
      (e) => e.slice(killedFrom).some((x) => x.event === "summon-timing"),
      30_000,
    );
    await d.sleep(1000);
    d.screenshot("rest-ready-rebuilt.png");
    run.killed = { pids: bars.length, after: app.events().slice(killedFrom) };
    const closed = app.events().filter((e) => e.event === "hibernated").length;
    await app.forward(["--hide"]);
    await app.waitFor("rest ready: closed after the rebuild", (e) => e.filter((x) => x.event === "hibernated").length > closed, 20_000);
  }
  if (mode === "ready15") {
    // The close rests again, until the timer.
    await app.waitFor("rest ready15: released again", (e) => e.filter((x) => x.event === "released" && x.detail === "timer").length >= 2, READY15_SMOKE_MS + 15_000);
  }
  await app.quit();
  run.events = app.events();
  return run;
}

/** The summon after the kill: its summon-timing line. */
function rebuiltTiming(after: readonly LogEvent[]): SummonTiming | undefined {
  return summonTimings([...after])[0];
}

/** The rules of one rest run (`os`: process.platform). */
export function restFailures(run: RestRun, os: string): string[] {
  const name = `rest ${run.mode}`;
  const failures: string[] = [];
  const has = (event: string, detail?: string) => run.events.some((e) => e.event === event && (detail === undefined || e.detail === detail));
  const start = run.samples.find((s) => s.at === "start");
  if (run.mode === "hibernate") {
    if (has("prewarmed")) {
      failures.push(`${name}: it built the ready window although the setting says Hibernate`);
    }
    if (start && start.engine > 0) {
      failures.push(`${name}: ${start.engine} engine processes of ours at rest after the launch (none allowed)`);
    }
  } else {
    if (!has("prewarmed", run.mode)) {
      failures.push(`${name}: no "prewarmed ${run.mode}" after the launch`);
    }
    if (start && start.web === 0 && os !== "darwin") {
      failures.push(`${name}: no web process for the ready bar at rest`);
    }
    if (run.events.some((e) => e.event === "window-created" && run.events.indexOf(e) < run.events.findIndex((x) => x.event === "prewarmed"))) {
      failures.push(`${name}: a window showed before the prewarm`);
    }
  }
  const close = run.close;
  if (!close) {
    failures.push(`${name}: the ping page never ran`);
  } else {
    if (close.pingsBefore === 0) {
      failures.push(`${name}: the ping page never pinged before the close`);
    }
    if (close.closingAt === null) {
      failures.push(`${name}: no "closing" line for the close`);
    } else if (close.pingsAfter > 0) {
      failures.push(
        `${name}: the page kept running after the close — ${close.pingsAfter} pings more than ${PINGS_STOP_MS} ms after it began (the last ${close.lastPing === null ? "?" : close.lastPing - close.closingAt} ms after)`,
      );
    }
    const finalLeft = close.left["30s"] ?? 0;
    if (finalLeft > 0) {
      failures.push(`${name}: ${finalLeft} of the page's ${close.pageWeb.length} web processes were still there 30 s after the close`);
    }
    const after = run.samples.find((s) => s.at === "after-close");
    if (run.mode === "hibernate" && after && after.web > 0) {
      failures.push(`${name}: ${after.web} web processes of ours at rest after the close (none allowed)`);
    }
    if (run.mode !== "hibernate" && after && start && after.web > Math.max(start.web, 1)) {
      failures.push(`${name}: ${after.web} web processes at rest after the close, more than the bar's ${start.web}`);
    }
  }
  if (run.mode === "ready") {
    const typed = run.typed;
    if (!typed?.sent) {
      failures.push(`${name}: the shortcut for the typed-keys check didn't go out`);
    } else if (os !== "darwin" && !typed.counts.includes(TYPED_KEYS.length)) {
      failures.push(
        `${name}: keys typed ${KEYS_AFTER_MS} ms after the shortcut were lost — the field saw ${typed.counts.length === 0 ? "none" : `typed=${typed.counts.join(",")}`}, not typed=${TYPED_KEYS.length}`,
      );
    }
    const killed = run.killed;
    if (!killed || killed.pids === 0) {
      failures.push(`${name}: no web process of the ready bar to kill`);
    } else {
      if (!has("bar-gone")) {
        failures.push(`${name}: the engine never said the bar's web process ended ("bar-gone")`);
      }
      const after = killed.after;
      if (!after.some((e) => e.event === "released" && e.detail === "bar-gone")) {
        failures.push(`${name}: the summon after the kill didn't release the gone bar's window`);
      }
      if (!after.some((e) => e.event === "window-created" && e.detail === "cold")) {
        failures.push(`${name}: the summon after the kill wasn't built anew (no "window-created cold")`);
      }
      const timing = rebuiltTiming(after);
      if (!timing || timing.partial || timing.marks["commandBar"] === undefined) {
        failures.push(`${name}: the summon after the kill never drew the command bar's rows (no complete summon-timing)`);
      }
    }
  }
  if (run.mode === "ready15") {
    const released = run.events.filter((e) => e.event === "released" && e.detail === "timer").length;
    if (released < 2) {
      failures.push(`${name}: the ready window went by itself ${released} times, not twice (after the launch, after the close)`);
    }
    if (!run.events.some((e) => e.event === "window-created" && e.detail === "cold")) {
      failures.push(`${name}: the summon after the timer wasn't cold`);
    }
    if (!has("hibernated", "ready15")) {
      failures.push(`${name}: the close didn't rest ("hibernated ready15")`);
    }
  }
  return failures;
}

/** What summary.json records of a rest run. */
export function restSummary(run: RestRun) {
  return {
    mode: run.mode,
    samples: run.samples,
    close: run.close,
    typed: run.typed,
    killed: run.killed ? { pids: run.killed.pids, after: run.killed.after.map((e) => `${e.event} ${e.detail}`.trim()).filter((l) => /^(released|window-created|bar-gone|summon-timing)/.test(l)) } : null,
  };
}

/** A Markdown table of the rest runs (the job's summary): memory and processes at rest per mode. */
export function restTable(runs: readonly RestRun[], os: string): string {
  const lines = [
    `### At rest per "After closing" mode (${os})`,
    "",
    "| mode | sample | web processes | engine processes | resident, app + engine (MB) | own, app + engine (MB) |",
    "|---|---|---|---|---|---|",
  ];
  for (const run of runs) {
    for (const s of run.samples) {
      lines.push(`| ${run.mode} | ${s.at} | ${s.web} | ${s.engine} | ${s.memoryMb} | ${s.ownMb ?? "–"} |`);
    }
  }
  for (const run of runs) {
    if (run.close) {
      const c = run.close;
      lines.push("", `- ${run.mode}: ${c.pingsBefore} pings before the close, ${c.pingsAfter} more than ${PINGS_STOP_MS} ms after; the page's web processes left ${JSON.stringify(c.left)}`);
    }
  }
  return `${lines.join("\n")}\n`;
}
