// The smoke test's summon-timing run (LJ19.1, LJ19.4) and its rules. The app logs one
// `summon-timing <json>` line per summon (src-tauri/src/summon_timing.rs): its kind — `cold` (the
// window and the bar built for it) or `ready` (the hidden window and its bar were there, LJ19.4) —,
// the "After closing" mode, and milliseconds from the request to each step — the window built, the
// bar added, the window shown, history loaded (on its own thread), the page webview built after the
// bar, the bar's navigation, load, script and first answers, its first frame, the command bar's
// first rows and the field taking the keyboard (or, with a URL, the web page started) — and on
// macOS App Nap's state at the request (`nap`). It runs twice, each on its own launch (no gdb, so
// Linux's numbers aren't gdb's) with a seeded history of HISTORY_SEED pages:
// Stay ready (the default; `ready` summons after the prewarm):
//   first           the first summon after the launch (the window was prewarmed)
//   warm ×3         a summon a few seconds after the last one closed
//   shortcut        the same through the real global shortcut (best effort, as below)
//   idle            after IDLE_MS with nothing happening
//   url             a `--show <url>`
// Hibernate (LJ19.1's run, `cold` summons):
//   first           the first summon after the launch
//   warm ×3         a summon a few seconds after the last one closed
//   shortcut        the same through the real global shortcut (best effort: the key may not reach
//                   the app on a runner — then it is recorded as not reached)
//   idle            after IDLE_MS with nothing happening (no cache eviction)
//   cold            after the OS's file cache was emptied (macOS `sudo purge`, Linux
//                   `drop_caches`); Windows has no such command, so a long gap (COLD_GAP_WINDOWS_MS)
//   cold-url        the same, with `--show <url>` (the bar's frame and the page's start)
//   cold-shortcut   evicted, IDLE_MS of quiet, then the global shortcut (best effort)
//   url             a warm `--show <url>`
// Each CLI summon (forwarded `--show`) must log a complete line with every mark of its kind; the
// shortcut ones count only when the key arrived. The ceilings (CEILINGS, per OS and kind, from
// LJ19.1's and LJ19.4's numbers) are loose on purpose — CI runners vary a lot —: they catch a summon
// that got several times slower, not a few hundred milliseconds. The numbers go into summary.json,
// timing.md and the job's summary. Pure parts unit-tested (smoke-timing.test.ts); smoke.mts
// supplies the driver.
import type { LogEvent } from "./smoke-log.mts";
import type { SiteApp } from "./smoke-sitemode.mts";

/** One `summon-timing` line. */
export interface SummonTiming {
  source: string;
  /** `cold` or `ready` (LJ19.4; lines from before it are cold). */
  kind: "cold" | "ready";
  /** The "After closing" mode at the request. */
  mode: string;
  url: boolean;
  /** The request's unix ms (the app's wall clock). */
  at: number;
  /** macOS: App Nap's state at the request (`active=<0|1> lowpri= timer= disk=`), else null. */
  nap: string | null;
  partial: boolean;
  /** Milliseconds after the request, per mark. */
  marks: Record<string, number>;
}

/** The window code's marks of a cold summon, in order (it runs them one after another). */
export const NATIVE_MARKS = ["dispatched", "windowBuilt", "webviewsAdded", "shown", "created"] as const;
/** … of a summon from ready (the window and the bar were there). */
export const READY_NATIVE_MARKS = ["dispatched", "shown", "created"] as const;
/** The page webview's builder's marks, in order (after the bar, or at once for a URL). */
export const PAGE_MARKS = ["pageBuilt", "cookiesRestored", "agentSet"] as const;
/** The other marks a cold summon has (history loads on its own thread; the bar's in its own order). */
export const BAR_MARKS = ["historyLoaded", "barLoaded", "barScript", "barReady", "barPainted"] as const;
/** … a summon from ready has. */
export const READY_BAR_MARKS = ["historyLoaded", "barPainted"] as const;

/** The marks a summon of `kind` must log, in the groups whose order is checked. */
export function wantedMarks(t: Pick<SummonTiming, "kind" | "url">): { native: readonly string[]; page: readonly string[]; other: readonly string[] } {
  const ready = t.kind === "ready";
  return {
    native: ready ? READY_NATIVE_MARKS : NATIVE_MARKS,
    page: PAGE_MARKS,
    other: [...(ready ? READY_BAR_MARKS : BAR_MARKS), ...(t.url ? ["pageStarted"] : ["commandBar", "barFocused"])],
  };
}

/** A `summon-timing` line's JSON, or undefined when it isn't one. */
export function parseTiming(detail: string): SummonTiming | undefined {
  try {
    const value = JSON.parse(detail) as Partial<SummonTiming>;
    if (typeof value.source !== "string" || typeof value.url !== "boolean" || typeof value.marks !== "object" || value.marks === null) {
      return undefined;
    }
    const marks: Record<string, number> = {};
    for (const [name, ms] of Object.entries(value.marks)) {
      if (typeof ms === "number" && Number.isFinite(ms)) {
        marks[name] = ms;
      }
    }
    return {
      source: value.source,
      kind: value.kind === "ready" ? "ready" : "cold",
      mode: typeof value.mode === "string" ? value.mode : "hibernate",
      url: value.url,
      at: typeof value.at === "number" ? value.at : 0,
      nap: typeof value.nap === "string" ? value.nap : null,
      partial: value.partial === true,
      marks,
    };
  } catch {
    return undefined;
  }
}

/** Every `summon-timing` line in `events`, in order. */
export function summonTimings(events: LogEvent[]): SummonTiming[] {
  return events.filter((e) => e.event === "summon-timing").flatMap((e) => parseTiming(e.detail) ?? []);
}

/**
 * When the summon was usable: the command bar's rows on screen, or with a URL the later of the
 * bar's first frame and the page's start. Undefined when that mark is missing.
 */
export function readyMs(t: SummonTiming): number | undefined {
  if (t.url) {
    const painted = t.marks["barPainted"];
    const started = t.marks["pageStarted"];
    return painted === undefined || started === undefined ? undefined : Math.max(painted, started);
  }
  return t.marks["commandBar"];
}

/** When the field took the keyboard (a summon without a URL); undefined when that mark is missing. */
export function focusedMs(t: SummonTiming): number | undefined {
  return t.url ? undefined : t.marks["barFocused"];
}

export type Kind = "first" | "warm" | "shortcut" | "idle" | "cold" | "cold-url" | "cold-shortcut" | "url";

/** Which run a summon was measured in: Stay ready's or Hibernate's. */
export type RunMode = "ready" | "hibernate";

/** One measured summon. */
export interface Measured {
  /** The run's "After closing" mode. */
  run: RunMode;
  kind: Kind;
  trigger: "cli" | "shortcut";
  /** What emptied the file cache first: "purge", "drop_caches", "failed", or "none". */
  evicted: string;
  /** Quiet before it (ms since the last window closed, or since the launch). */
  gapMs: number;
  /** The harness's unix ms just before the request went out (the key, or the forwarding launch). */
  sentAt: number | null;
  timing: SummonTiming | null;
}

/**
 * The ceilings (ms to `readyMs`, and to `focusedMs`) per kind of summon: loose, from LJ19.1's
 * measurements (cold) and LJ19.4's (`ready`: every summon from the ready window).
 */
export interface Ceilings {
  warm: number;
  first: number;
  cold: number;
  ready: number;
}

/**
 * Per OS (process.platform), several times what LJ19.1 measured on CI's runners before its changes
 * (ready, ms — first / warm / cold): Linux 280–500 / 210–390 / 330–730, macOS 2,740 / 2,520–2,600
 * / 4,870–5,450 (CI's macOS VMs are slow to start WebKit), Windows 965 / 820–920 / 770–830; and for
 * a summon from ready, about three times what LJ19.4 measured (its notes).
 */
export const CEILINGS: Record<string, Ceilings> = {
  linux: { warm: 2000, first: 3000, cold: 5000, ready: 1500 },
  darwin: { warm: 8000, first: 10_000, cold: 15_000, ready: 4000 },
  win32: { warm: 4000, first: 5000, cold: 6000, ready: 2000 },
};

/** Anywhere else: only a summon that is plainly broken fails. */
export const DEFAULT_CEILINGS: Ceilings = { warm: 15_000, first: 20_000, cold: 30_000, ready: 15_000 };

/** The ceilings for `os` (process.platform). */
export function ceilingsFor(os: string): Ceilings {
  return CEILINGS[os] ?? DEFAULT_CEILINGS;
}

function ceilingOf(m: Measured, t: SummonTiming, ceilings: Ceilings): number {
  if (t.kind === "ready") {
    return ceilings.ready;
  }
  const kind = m.kind;
  if (kind === "warm" || kind === "url" || kind === "shortcut") {
    return ceilings.warm;
  }
  return kind === "first" ? ceilings.first : ceilings.cold;
}

/** The rules: complete lines with every mark for the CLI summons, in order, under the ceilings. */
export function timingFailures(run: Measured[], os: string, ceilings: Ceilings = ceilingsFor(os)): string[] {
  const failures: string[] = [];
  for (const mode of new Set(run.map((m) => m.run))) {
    if (!run.some((m) => m.run === mode && m.kind === "warm")) {
      failures.push(`summon timing: the ${mode} run measured no warm summon`);
    }
  }
  for (const m of run) {
    const name = `summon timing (${m.run === "ready" ? "ready " : ""}${m.kind})`;
    const t = m.timing;
    if (!t) {
      if (m.trigger === "cli") {
        failures.push(`${name}: no summon-timing line`);
      }
      continue;
    }
    if (t.partial) {
      failures.push(`${name}: the line is partial (marks: ${Object.keys(t.marks).join(", ")})`);
      continue;
    }
    const expectedKind = m.run === "ready" ? "ready" : "cold";
    if (t.kind !== expectedKind) {
      failures.push(`${name}: a ${t.kind} summon, not ${expectedKind}`);
    }
    const wanted = wantedMarks(t);
    const missing = [...wanted.native, ...wanted.page, ...wanted.other].filter((mark) => t.marks[mark] === undefined);
    if (missing.length > 0) {
      failures.push(`${name}: missing ${missing.join(", ")}`);
      continue;
    }
    for (const [group, marks] of [
      ["native", wanted.native],
      ["page", wanted.page],
    ] as const) {
      const times = marks.map((mark) => t.marks[mark] ?? 0);
      if (times.some((ms, i) => i > 0 && ms < (times[i - 1] ?? 0))) {
        failures.push(`${name}: the ${group} marks are out of order (${times.join(", ")})`);
      }
    }
    const ceiling = ceilingOf(m, t, ceilings);
    const ready = readyMs(t) ?? Infinity;
    if (ready > ceiling) {
      failures.push(`${name}: ready after ${ready} ms, over the ${ceiling} ms ceiling`);
    }
    const focused = focusedMs(t);
    if (focused !== undefined && focused > ceiling) {
      failures.push(`${name}: the field took the keyboard after ${focused} ms, over the ${ceiling} ms ceiling`);
    }
  }
  return failures;
}

/** The middle value (the lower one of two), or undefined for none. */
export function median(values: number[]): number | undefined {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.floor((sorted.length - 1) / 2)];
}

/** One row of the summary: the kind, the trigger's latency, and the main marks. */
export function summaryRow(m: Measured) {
  const t = m.timing;
  return {
    run: m.run,
    kind: m.kind,
    trigger: m.trigger,
    evicted: m.evicted,
    gapMs: m.gapMs,
    reached: t !== null,
    // The key (or the forwarding launch) to the app's request mark, both wall clocks.
    sentToRequestMs: t && m.sentAt !== null ? t.at - m.sentAt : null,
    readyMs: t ? (readyMs(t) ?? null) : null,
    focusedMs: t ? (focusedMs(t) ?? null) : null,
    summon: t?.kind ?? null,
    nap: t?.nap ?? null,
    marks: t?.marks ?? null,
    partial: t?.partial ?? null,
  };
}

const TABLE_MARKS = [
  "windowBuilt",
  "webviewsAdded",
  "shown",
  "historyLoaded",
  "barNavigation",
  "barLoaded",
  "barModule",
  "barReady",
  "barPainted",
  "barFocused",
  "barSuggestions",
  "commandBar",
  "pageBuilt",
  "agentSet",
  "pageStarted",
];

/** A Markdown table of the run (timing.md and the job's summary); App Nap's state below it. */
export function timingTable(run: Measured[], os: string): string {
  const head = ["run", "summon", "trigger", "sent→request", ...TABLE_MARKS, "ready"];
  const lines = [
    `### Summon timing (${os}), ms after the request`,
    "",
    `| ${head.join(" | ")} |`,
    `|${head.map(() => "---").join("|")}|`,
  ];
  for (const m of run) {
    const row = summaryRow(m);
    const cells = [
      m.run,
      `${m.kind}${m.evicted !== "none" ? ` (${m.evicted})` : ""}`,
      m.trigger,
      row.sentToRequestMs === null ? "–" : String(row.sentToRequestMs),
      ...TABLE_MARKS.map((mark) => (row.marks?.[mark] === undefined ? "–" : String(row.marks[mark]))),
      row.reached ? String(row.readyMs ?? "–") : "not reached",
    ];
    lines.push(`| ${cells.join(" | ")} |`);
  }
  const naps = run.filter((m) => m.timing?.nap);
  if (naps.length > 0) {
    lines.push("", "App Nap at the request (macOS, the task's suppression policy):", "");
    for (const m of naps) {
      lines.push(`- ${m.run} ${m.kind}: ${m.timing?.nap ?? ""}`);
    }
  }
  return `${lines.join("\n")}\n`;
}

/** How many pages the run's history.json holds (a heavy user's; the cap is 5,000). */
export const HISTORY_SEED = 3000;

/** A history.json of `pages` pages on 300 sites under `.test` (never resolved), visited over 60 days. */
export function seedHistory(now: number, pages = HISTORY_SEED): string {
  const day = 86_400_000;
  const entries = Array.from({ length: pages }, (_, i) => {
    const host = `site-${i % 300}.lj-smoke.test`;
    const visits = Array.from({ length: 10 }, (_, v) => ({ at: now - ((i * 7 + v * 131) % 60) * day - v * 60_000, kind: v % 3 === 0 ? "typed" : "link" })).sort(
      (a, b) => a.at - b.at,
    );
    return { url: `https://${host}/page-${i}`, title: `Page ${i} of site ${i % 300}`, host, visitCount: 10 + (i % 17), visits };
  });
  return JSON.stringify({ version: 1, entries });
}

/** Quiet before an idle summon (no eviction). */
export const IDLE_MS = 60_000;
/** Windows' "cold" summon: no command empties its file cache, so a long gap instead. */
export const COLD_GAP_WINDOWS_MS = 180_000;
/** Quiet after an eviction, for the eviction's own work to settle. */
const AFTER_EVICT_MS = 10_000;
/** Between warm summons. */
const WARM_GAP_MS = 3000;

/** What the run needs from smoke.mts. */
export interface TimingDriver {
  os: string;
  pageUrl: string;
  /** Launches the app (no gdb) on a fresh data folder with these settings and files. */
  launch(name: string, settings: Record<string, unknown>, files: Record<string, string>): Promise<SiteApp>;
  sleep(ms: number): Promise<void>;
  /** Empties the OS's file cache where a command can (sudo on CI runners): what it did. */
  evict(): string;
  /** Presses the global shortcut; the unix ms just before the key went out, or null. */
  pressSummonShortcut(): number | null;
}

/** Runs the timing launches: Stay ready's, then Hibernate's (LJ19.1's). */
export async function timingRun(d: TimingDriver): Promise<Measured[]> {
  return [...(await modeRun(d, "ready")), ...(await modeRun(d, "hibernate"))];
}

async function modeRun(d: TimingDriver, mode: RunMode): Promise<Measured[]> {
  const ready = mode === "ready";
  const app = await d.launch(ready ? "timing-ready" : "timing", ready ? {} : { afterClose: "hibernate" }, { "history.json": seedHistory(Date.now()) });
  await app.waitFor(`timing ${mode}: started, shortcut`, (e) => e.some((x) => x.event === "started") && e.some((x) => x.event.startsWith("shortcut-")), 30_000);
  if (ready) {
    await app.waitFor("timing ready: prewarmed", (e) => e.some((x) => x.event === "prewarmed"), 20_000);
  }
  await d.sleep(3000);
  const run: Measured[] = [];
  let closedAt = Date.now();

  const summon = async (kind: Kind, trigger: "cli" | "shortcut", evicted: string, url?: string): Promise<void> => {
    const from = app.events().length;
    const hibernated = app.events().filter((e) => e.event === "hibernated").length;
    const gapMs = Date.now() - closedAt;
    let sentAt: number | null = Date.now();
    if (trigger === "shortcut") {
      sentAt = d.pressSummonShortcut();
    } else {
      await app.forward(url ? ["--show", url] : ["--show"]);
    }
    const logged = (e: LogEvent[]) => e.slice(from).some((x) => x.event === "summon-timing");
    const reached = await app.waitFor(`timing ${mode}: ${kind}`, logged, trigger === "shortcut" ? 12_000 : 30_000);
    const timing = reached ? (summonTimings(app.events().slice(from))[0] ?? null) : null;
    run.push({ run: mode, kind, trigger, evicted, gapMs, sentAt, timing });
    const opened = app.events().slice(from).some((x) => x.event === "window-created");
    if (opened) {
      await d.sleep(1000);
      await app.forward(["--hide"]);
      await app.waitFor(`timing ${mode}: ${kind} closed`, (e) => e.filter((x) => x.event === "hibernated").length > hibernated, 20_000);
    }
    closedAt = Date.now();
  };

  await summon("first", "cli", "none");
  for (let i = 0; i < 3; i += 1) {
    await d.sleep(WARM_GAP_MS);
    await summon("warm", "cli", "none");
  }
  await d.sleep(WARM_GAP_MS);
  await summon("shortcut", "shortcut", "none");
  await d.sleep(IDLE_MS);
  await summon("idle", "cli", "none");
  if (ready) {
    await d.sleep(WARM_GAP_MS);
    await summon("url", "cli", "none", `${d.pageUrl}?fits=1`);
    await app.quit();
    return run;
  }
  // Windows: the long gap once; the summons after it wait IDLE_MS (it has no eviction to repeat).
  const cold = async (gapOnWindows: number): Promise<string> => {
    if (d.os === "win32") {
      await d.sleep(gapOnWindows);
      return "none";
    }
    const evicted = d.evict();
    await d.sleep(AFTER_EVICT_MS);
    return evicted;
  };
  await summon("cold", "cli", await cold(COLD_GAP_WINDOWS_MS));
  await summon("cold-url", "cli", await cold(IDLE_MS), `${d.pageUrl}?fits=1`);
  const evicted = await cold(0);
  await d.sleep(IDLE_MS);
  await summon("cold-shortcut", "shortcut", evicted);
  await d.sleep(WARM_GAP_MS);
  await summon("url", "cli", "none", `${d.pageUrl}?fits=1`);
  await app.quit();
  return run;
}

/** What summary.json records of the run. */
export function timingSummary(run: Measured[], os: string) {
  const of = (mode: RunMode, kind: Kind, measure: (t: SummonTiming) => number | undefined) =>
    run.flatMap((m) => (m.run === mode && m.kind === kind && m.timing && !m.timing.partial ? (measure(m.timing) ?? []) : []));
  const modeSummary = (mode: RunMode) => ({
    warmMedianMs: median(of(mode, "warm", readyMs)) ?? null,
    warmFocusedMedianMs: median(of(mode, "warm", focusedMs)) ?? null,
    firstMs: of(mode, "first", readyMs)[0] ?? null,
    firstFocusedMs: of(mode, "first", focusedMs)[0] ?? null,
    idleMs: of(mode, "idle", readyMs)[0] ?? null,
    coldMs: of(mode, "cold", readyMs)[0] ?? null,
  });
  return {
    ready: modeSummary("ready"),
    hibernate: modeSummary("hibernate"),
    rows: run.map(summaryRow),
    rules: timingFailures(run, os),
  };
}
