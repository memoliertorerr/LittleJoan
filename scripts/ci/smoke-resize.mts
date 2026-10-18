// The smoke test's resize run (LJ39) and its rules. Its own launch (so the main run's memory
// numbers don't move), in a dark theme (a white gutter would show in the screenshots), with the
// window created 820 px wide so every width of the run stays on the runner's screen (the top-left
// stays where it was after a `--size`). The smoke page, then: `--size 380x600` (phone agent),
// a burst of 12 `--size` steps from 380 to 820 px wide sent through single-instance 30 ms apart
// (desktop agent; `resize-820.png`), the same burst back to 380 (phone; `resize-380.png`), then
// the hysteresis: 620 coming from phone stays phone, 660 goes desktop, 620 coming from desktop
// stays desktop. The app logs `bounds-set <n>` for every layout (how many webviews' bounds it
// set), `page-visible` when the page is shown or hidden, `user-agent <phone|desktop>` when phone
// mode switches, and `page-started <url>` for each load; the smoke page reports
// `LJ-SMOKE viewport=<w>x<h> ua=<phone|desktop> dpr=<r>` once a resize has settled. The parsing
// and the rules are pure and unit-tested (smoke-resize.test.ts); smoke.mts supplies the driver.
import type { FsApp, FsDriver } from "./smoke-fullscreen.mts";
import type { LogEvent } from "./smoke-log.mts";

/** The window's height through the run, and what the page gets of it (minus the 40 px toolbar). */
export const HEIGHT = 600;
export const PAGE_HEIGHT = HEIGHT - 40;
/** How far apart the burst's `--size` steps are sent. */
export const BURST_GAP_MS = 30;

/** The run's settings: a dark two-color theme, and the window created at the widest width. */
export const RESIZE_SETTINGS = {
  theme: { mode: "dark", colors: ["#9d8fd0", "#e0a2a0"], intensity: 0.6, grain: 0.4 },
  windowWidth: 820,
  windowHeight: HEIGHT,
} as const;

/** `steps` widths from `from` to `to`, evenly spaced, both ends included. */
export function burst(from: number, to: number, steps = 12): number[] {
  return Array.from({ length: steps }, (_, i) => Math.round(from + ((to - from) * i) / (steps - 1)));
}

/** One step of the run: the widths it sends and what the page must end with. */
export interface ResizeStep {
  name: string;
  /** Sent in order; more than one is a burst (BURST_GAP_MS apart). */
  widths: number[];
  /** The agent the page must report at the end. */
  ua: "phone" | "desktop";
  /** How many agent switches (and so reloads) the step must cause: one per crossing. */
  switches: 0 | 1;
  /** A screenshot once it settled. */
  shot?: string;
}

export const RESIZE_STEPS: readonly ResizeStep[] = [
  { name: "narrow", widths: [380], ua: "phone", switches: 1 },
  { name: "burst up", widths: burst(380, 820), ua: "desktop", switches: 1, shot: "resize-820.png" },
  { name: "burst down", widths: burst(820, 380), ua: "phone", switches: 1, shot: "resize-380.png" },
  { name: "620 from phone", widths: [620], ua: "phone", switches: 0 },
  { name: "660", widths: [660], ua: "desktop", switches: 1 },
  { name: "620 from desktop", widths: [620], ua: "desktop", switches: 0 },
];

/** One `LJ-SMOKE viewport=…` report of the smoke page. */
export interface ViewportReport {
  width: number;
  height: number;
  ua: "phone" | "desktop";
}

/** Reads a title's `LJ-SMOKE viewport=<w>x<h> ua=<phone|desktop> …`; anything else is undefined. */
export function parseViewport(title: string): ViewportReport | undefined {
  const match = /^LJ-SMOKE viewport=(\d+)x(\d+) ua=(phone|desktop)(?: |$)/.exec(title);
  if (!match?.[1] || !match[2] || !match[3]) {
    return undefined;
  }
  return { width: Number(match[1]), height: Number(match[2]), ua: match[3] as ViewportReport["ua"] };
}

/** Whether `event` is a load of the smoke page (`page-started <pageUrl>…`). */
function loadOf(event: LogEvent, pageUrl: string): boolean {
  return event.event === "page-started" && event.detail.startsWith(pageUrl);
}

/**
 * The page's viewport as it stands at the end of `events`: its last report after its last load
 * (a reload starts it again), or undefined while it hasn't reported since.
 */
export function standing(events: LogEvent[], pageUrl: string): ViewportReport | undefined {
  const loaded = events.findLastIndex((e) => loadOf(e, pageUrl));
  for (let i = events.length - 1; i > loaded; i -= 1) {
    const event = events[i];
    const report = event?.event === "title" ? parseViewport(event.detail) : undefined;
    if (report) {
      return report;
    }
  }
  return undefined;
}

/** What one step's part of the log shows. */
export interface StepResult {
  name: string;
  /** The `user-agent` switches, in order. */
  switches: string[];
  /** Loads of the smoke page (reloads after a switch). */
  reloads: number;
  /** Every `bounds-set` count. */
  boundsSet: number[];
  /** `page-visible` lines (a resize must show or hide nothing). */
  visibility: string[];
  /** The page's viewport at the end. */
  viewport?: ViewportReport;
}

/** Reads one step's events (from just before it sent its first `--size` until it settled). */
export function stepResult(step: ResizeStep, events: LogEvent[], pageUrl: string): StepResult {
  const viewport = standing(events, pageUrl);
  return {
    name: step.name,
    switches: events.filter((e) => e.event === "user-agent").map((e) => e.detail),
    reloads: events.filter((e) => loadOf(e, pageUrl)).length,
    boundsSet: events.filter((e) => e.event === "bounds-set").map((e) => Number(e.detail)),
    visibility: events.filter((e) => e.event === "page-visible").map((e) => e.detail),
    ...(viewport ? { viewport } : {}),
  };
}

/** The final width a step leaves the window at. */
function finalWidth(step: ResizeStep): number {
  return step.widths.at(-1) ?? 0;
}

/**
 * The rules, per step: the final viewport is the window's width × its height minus the toolbar,
 * with the step's agent; exactly `switches` agent switches (to that agent) and as many reloads;
 * every `bounds-set` at most 2 (and at least one layout ran); nothing shown or hidden.
 */
export function resizeFailures(steps: readonly ResizeStep[], results: StepResult[]): string[] {
  const failures: string[] = [];
  steps.forEach((step, i) => {
    const result = results[i];
    const where = `resize: ${step.name}`;
    if (!result) {
      failures.push(`${where}: never ran`);
      return;
    }
    const want = `${finalWidth(step)}x${PAGE_HEIGHT} ua=${step.ua}`;
    const got = result.viewport ? `${result.viewport.width}x${result.viewport.height} ua=${result.viewport.ua}` : "no report";
    if (got !== want) {
      failures.push(`${where}: the page ended at ${got}, not ${want}`);
    }
    if (result.switches.length !== step.switches || result.switches.some((ua) => ua !== step.ua)) {
      failures.push(`${where}: ${step.switches} switch to ${step.ua} expected, saw [${result.switches.join(", ")}]`);
    }
    if (result.reloads !== step.switches) {
      failures.push(`${where}: ${step.switches} reload expected, saw ${result.reloads}`);
    }
    if (result.boundsSet.length === 0) {
      failures.push(`${where}: no layout ran (no bounds-set line)`);
    }
    const over = result.boundsSet.filter((n) => !(n >= 0 && n <= 2));
    if (over.length > 0) {
      failures.push(`${where}: a layout set more than two webviews' bounds (${over.join(", ")})`);
    }
    if (result.visibility.length > 0) {
      failures.push(`${where}: a resize showed or hid the page (page-visible ${result.visibility.join(", ")})`);
    }
  });
  return failures;
}

/** What summary.json records of the run: per step, the switches, reloads and bounds-set counts. */
export function resizeSummary(results: StepResult[]) {
  return results.map((result) => {
    const counts: Record<string, number> = {};
    for (const n of result.boundsSet) {
      counts[String(n)] = (counts[String(n)] ?? 0) + 1;
    }
    return { ...result, boundsSet: counts, layouts: result.boundsSet.length };
  });
}

/** Waits until the page stands at `want` with no new load or switch for 2 s. */
async function settled(d: FsDriver, app: FsApp, from: number, step: ResizeStep): Promise<boolean> {
  const label = `resize: ${step.name} → ${finalWidth(step)} px, ${step.ua} agent`;
  const at = (events: LogEvent[]) => {
    const now = standing(events.slice(from), d.pageUrl);
    return now?.width === finalWidth(step) && now.height === PAGE_HEIGHT && now.ua === step.ua;
  };
  const marks = (events: LogEvent[]) => events.slice(from).filter((e) => e.event === "user-agent" || loadOf(e, d.pageUrl) || e.event === "title").length;
  for (let tries = 0; tries < 6; tries += 1) {
    if (!(await app.waitFor(label, at, 30_000))) {
      return false;
    }
    const seen = marks(app.events());
    await d.sleep(2000);
    if (marks(app.events()) === seen && at(app.events())) {
      return true;
    }
  }
  return false;
}

/** Sends a step's widths: one `--size`, or a burst 30 ms apart, then the last one again (waited). */
async function send(d: FsDriver, app: FsApp, step: ResizeStep): Promise<void> {
  if (step.widths.length === 1) {
    await app.forward(["--size", `${finalWidth(step)}x${HEIGHT}`]);
    return;
  }
  const sent: Promise<void>[] = [];
  for (const width of step.widths) {
    sent.push(app.forward(["--size", `${width}x${HEIGHT}`]));
    await d.sleep(BURST_GAP_MS);
  }
  await Promise.all(sent);
  // The second instances may land out of order: the last width once more, so the run ends there.
  await app.forward(["--size", `${finalWidth(step)}x${HEIGHT}`]);
}

/** Runs the resize launch. Returns each step's result (in order) and the whole log. */
export async function resizeRun(d: FsDriver): Promise<{ results: StepResult[]; events: LogEvent[] }> {
  const app = await d.launch("resize", RESIZE_SETTINGS);
  await app.waitFor("resize: started", (e) => e.some((x) => x.event === "started"), 30_000);
  await d.sleep(1500);
  await app.forward(["--show", d.pageUrl]);
  await app.waitFor("resize: the page at 820 px", (e) => standing(e, d.pageUrl)?.width === 820, 30_000);
  await d.sleep(1500);
  const results: StepResult[] = [];
  for (const step of RESIZE_STEPS) {
    const from = app.events().length;
    await send(d, app, step);
    await settled(d, app, from, step);
    results.push(stepResult(step, app.events().slice(from), d.pageUrl));
    if (step.shot) {
      d.screenshot(step.shot);
    }
  }
  await app.quit();
  return { results, events: app.events() };
}
