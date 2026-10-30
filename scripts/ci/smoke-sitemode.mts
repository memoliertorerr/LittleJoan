// The smoke test's site-mode run (LJ40) and its rules. Its own launch (default theme, the window
// 480×640): the smoke page at 390 px — the phone agent, the toolbar's "Phone" pill
// (`phone-pill.png`) —; then the command bar's "Desktop site" (LJ42's quick row; Ctrl/Cmd+L, typed
// "request desktop", ↓ ↓ Enter: the search row first, then the row; typed on Windows and Linux,
// best effort on macOS): `site-mode 127.0.0.1 desktop`, settings.json keeps
// `"127.0.0.1": "desktop"`, and the page loads once more with the desktop agent; "Automatic"
// (typed "automatic") brings the phone agent back (one reload, the entry gone). Then, informational:
// `?fits=1` (the page without its desktop minimum width) at 720 px, then 390 px — Windows asks
// the engine whether the page fits and skips the reload (`ua-reload skipped fits`); macOS and
// Linux reload. Last, "Desktop site" once more and Settings → General listing the site
// (`settings-site-modes.png`).
//
// The smoke server writes each request's agent headers (User-Agent, Sec-CH-UA-Mobile,
// Sec-CH-UA-Platform, Sec-CH-UA) to agent-headers.jsonl; on Windows the phone agent must send
// `?1` and `"Android"` and the page must see `navigator.userAgentData.mobile === true`, the
// desktop agent `?0` and `"Windows"` — unless the app logged `ua-override fallback` (the DevTools
// protocol refused: the bare agent stays, and the rule is informational). WebKit sends no client
// hints. smoke.mts supplies the driver; the parsing and the rules are pure and unit-tested
// (smoke-sitemode.test.ts).
import type { FsApp } from "./smoke-fullscreen.mts";
import type { LogEvent } from "./smoke-log.mts";
import { PAGE_HEIGHT, parseViewport } from "./smoke-resize.mts";

/** One request the smoke server saw, with its agent headers. */
export interface AgentRequest {
  t: number;
  path: string;
  query: string;
  ua: string;
  mobile: string | null;
  platform: string | null;
}

/** agent-headers.jsonl → its requests (lines that don't parse are skipped). */
export function parseAgentLog(text: string): AgentRequest[] {
  return text.split(/\r?\n/).flatMap((line) => {
    try {
      const raw = JSON.parse(line) as Record<string, unknown>;
      const str = (key: string): string | null => {
        const value = raw[key];
        return typeof value === "string" ? value : null;
      };
      return typeof raw.t === "number" && typeof raw.path === "string"
        ? [{ t: raw.t, path: raw.path, query: str("query") ?? "", ua: str("ua") ?? "", mobile: str("mobile"), platform: str("platform") }]
        : [];
    } catch {
      return [];
    }
  });
}

/** The smoke page's document requests carrying `t=<token>`, in order. */
export function documentRequests(requests: AgentRequest[], token: string): AgentRequest[] {
  return requests.filter((r) => r.path === "smoke.html" && new URLSearchParams(r.query).get("t") === token);
}

/** The agent the headers describe: what a site sees. */
export interface HeaderAgent {
  android: boolean;
  mobile: string | null;
  platform: string | null;
}

export function headerAgent(request: AgentRequest): HeaderAgent {
  return { android: /\bAndroid\b/.test(request.ua), mobile: request.mobile, platform: request.platform };
}

/** One viewport report's `uamobile=` (what navigator.userAgentData.mobile said). */
export function uaMobile(title: string): string | undefined {
  return /(?:^| )uamobile=(\S+)/.exec(title)?.[1];
}

/** The page's last viewport report in `events` (after its last load of the smoke page). */
export function lastReport(events: LogEvent[], pageUrl: string): { width: number; height: number; ua: string; uamobile?: string } | undefined {
  const loaded = events.findLastIndex((e) => e.event === "page-started" && e.detail.startsWith(pageUrl));
  for (let i = events.length - 1; i > loaded; i -= 1) {
    const event = events[i];
    const report = event?.event === "title" ? parseViewport(event.detail) : undefined;
    if (event && report) {
      const mobile = uaMobile(event.detail);
      return { ...report, ...(mobile ? { uamobile: mobile } : {}) };
    }
  }
  return undefined;
}

/** What one switch through the command bar showed. */
export interface SwitchResult {
  /** The command bar showed the typed text (the `suggestions` line). */
  typed: boolean;
  /** `site-mode <site> <mode>` lines. */
  logged: string[];
  /** Loads of the smoke page after the request. */
  reloads: number;
  /** The page's report at the end. */
  report?: { width: number; height: number; ua: string; uamobile?: string };
  /** settings.json right after (its siteModes, or null when the file had none). */
  siteModes: Record<string, string> | null;
  /** The document requests made with the new agent (the reload's). */
  headers: HeaderAgent[];
}

/** What the whole run showed. */
export interface SiteModeRun {
  events: LogEvent[];
  /** The first document request (the phone agent, from the window's creation). */
  phoneHeaders: HeaderAgent[];
  phoneReport?: { width: number; height: number; ua: string; uamobile?: string };
  desktop?: SwitchResult;
  auto?: SwitchResult;
  /** ?fits=1 going narrow: `skipped` (Windows: the page fits), `reloaded`, or `none`. */
  fits: "skipped" | "reloaded" | "none";
}

/** settings.json's siteModes (null when the file has none or isn't there). */
export function siteModesOf(text: string | undefined): Record<string, string> | null {
  if (!text) {
    return null;
  }
  try {
    const raw = (JSON.parse(text) as Record<string, unknown>).siteModes;
    return typeof raw === "object" && raw !== null && !Array.isArray(raw) ? (raw as Record<string, string>) : null;
  } catch {
    return null;
  }
}

/** Whether the DevTools protocol refused the agent override (then the bare agent stayed). */
export function overrideRefused(events: LogEvent[]): boolean {
  return events.some((e) => e.event === "ua-override" && e.detail.startsWith("fallback"));
}

const SITE = "127.0.0.1";
const WIDTH = 390;

/**
 * The rules. Everywhere the command bar was driven (Windows, Linux; macOS when the typing showed
 * up): "Request desktop site" logs `site-mode 127.0.0.1 desktop`, keeps it in settings.json and
 * loads the page once more with the desktop agent at 390 px; "Automatic for this site" logs
 * `site-mode 127.0.0.1 auto`, removes the entry, and loads it once more with the phone agent.
 * Windows, unless the protocol refused: the phone agent's request says Android, `?1`, `"Android"`
 * and the page `uamobile=true`; the desktop agent's request `?0`, `"Windows"`, `uamobile=false`.
 */
export function siteModeFailures(run: SiteModeRun, os: string): string[] {
  const failures: string[] = [];
  const driven = os !== "darwin";
  const check = (name: string, result: SwitchResult | undefined, mode: "desktop" | "auto", ua: "phone" | "desktop") => {
    if (!result || (!result.typed && !driven)) {
      return;
    }
    if (!result.typed) {
      failures.push(`site mode: the command bar never showed the typing for "${name}"`);
      return;
    }
    if (!result.logged.includes(`${SITE} ${mode}`)) {
      failures.push(`site mode: "${name}" logged [${result.logged.join(", ")}], not "${SITE} ${mode}"`);
    }
    const want = `${WIDTH}x${PAGE_HEIGHT} ua=${ua}`;
    const got = result.report ? `${result.report.width}x${result.report.height} ua=${result.report.ua}` : "no report";
    if (got !== want) {
      failures.push(`site mode: after "${name}" the page reported ${got}, not ${want}`);
    }
    if (result.reloads !== 1) {
      failures.push(`site mode: "${name}" loaded the page ${result.reloads} times, not once`);
    }
    const kept = result.siteModes?.[SITE];
    if (mode === "desktop" ? kept !== "desktop" : kept !== undefined) {
      failures.push(`site mode: after "${name}" settings.json's siteModes were ${JSON.stringify(result.siteModes)}`);
    }
  };
  check("Request desktop site", run.desktop, "desktop", "desktop");
  check("Automatic for this site", run.auto, "auto", "phone");
  if (os === "win32" && !overrideRefused(run.events)) {
    const phone = run.phoneHeaders[0];
    if (!phone || !phone.android || phone.mobile !== "?1" || phone.platform !== '"Android"') {
      failures.push(`site mode: Windows' phone agent sent ${JSON.stringify(phone ?? null)}, not Android with Sec-CH-UA-Mobile ?1 and "Android"`);
    }
    if (run.phoneReport?.uamobile !== "true") {
      failures.push(`site mode: the page saw navigator.userAgentData.mobile = ${run.phoneReport?.uamobile ?? "nothing"} under the phone agent`);
    }
    if (run.desktop?.typed) {
      const desktop = run.desktop.headers.at(-1);
      if (!desktop || desktop.android || desktop.mobile !== "?0" || desktop.platform !== '"Windows"') {
        failures.push(`site mode: Windows' desktop agent sent ${JSON.stringify(desktop ?? null)}, not Sec-CH-UA-Mobile ?0 and "Windows"`);
      }
      if (run.desktop.report?.uamobile !== "false") {
        failures.push(`site mode: the page saw navigator.userAgentData.mobile = ${run.desktop.report?.uamobile ?? "nothing"} under the desktop agent`);
      }
    }
  }
  return failures;
}

/** What smoke.mts provides. */
export interface SiteApp extends FsApp {
  pid(): number;
  /** The launch's settings.json, now. */
  settings(): string | undefined;
  /** The launch's data folder (a relaunch on it: smoke-tray.mts). */
  dir?(): string;
}

export interface SiteDriver {
  os: string;
  pageUrl: string;
  launch(name: string, settings: Record<string, unknown>): Promise<SiteApp>;
  screenshot(name: string): void;
  sleep(ms: number): Promise<void>;
  typeText(text: string, pid: number, activate: boolean): void;
  pressKeys(keys: string[], pid: number, activate: boolean): void;
  pressShortcut(letter: string, pid: number): void;
  agentRequests(): AgentRequest[];
}

/** Waits until the page's last report says `width` × 560 with `ua`, unchanged for 2 s. */
async function settled(d: SiteDriver, app: SiteApp, from: number, width: number, ua: string): Promise<boolean> {
  const at = (events: LogEvent[]) => {
    const report = lastReport(events.slice(from), d.pageUrl);
    return report?.width === width && report.height === PAGE_HEIGHT && report.ua === ua;
  };
  for (let tries = 0; tries < 5; tries += 1) {
    if (!(await app.waitFor(`site mode: ${width} px, ${ua} agent`, at, 30_000))) {
      return false;
    }
    await d.sleep(2000);
    if (at(app.events())) {
      return true;
    }
  }
  return false;
}

/** Ctrl/Cmd+L, `typed`, ↓ ↓ Enter in the command bar; then what it did. */
async function viaCommandBar(d: SiteDriver, app: SiteApp, token: string, typed: string, ua: string): Promise<SwitchResult> {
  const pid = app.pid();
  const from = app.events().length;
  const requestsBefore = documentRequests(d.agentRequests(), token).length;
  d.pressShortcut("l", pid);
  await app.waitFor(`site mode: the command bar for "${typed}"`, (e) => e.slice(from).some((x) => x.event === "overlay" && x.detail === "true"), 10_000);
  await d.sleep(500);
  // The window is in front already (Ctrl/Cmd+L went to it): no second activation.
  d.typeText(typed, pid, false);
  const shown = await app.waitFor(
    `site mode: typed "${typed}"`,
    (e) => e.slice(from).some((x) => x.event === "suggestions" && x.detail.endsWith(`typed=${typed.length}`)),
    15_000,
  );
  if (shown) {
    await d.sleep(800);
    d.pressKeys(["Down", "Down", "Return"], pid, false);
    await app.waitFor(`site mode: "${typed}" applied`, (e) => e.slice(from).some((x) => x.event === "site-mode"), 15_000);
    await settled(d, app, from, WIDTH, ua);
  }
  const events = app.events().slice(from);
  const report = lastReport(events, d.pageUrl);
  return {
    typed: shown,
    logged: events.filter((e) => e.event === "site-mode").map((e) => e.detail),
    reloads: events.filter((e) => e.event === "page-started" && e.detail.startsWith(d.pageUrl)).length,
    ...(report ? { report } : {}),
    siteModes: siteModesOf(app.settings()),
    headers: documentRequests(d.agentRequests(), token).slice(requestsBefore).map(headerAgent),
  };
}

/** Runs the site-mode launch. */
export async function siteModeRun(d: SiteDriver): Promise<SiteModeRun> {
  const app = await d.launch("site-mode", {});
  await app.waitFor("site mode: started", (e) => e.some((x) => x.event === "started"), 30_000);
  await d.sleep(1500);
  const token = `sm-${Date.now().toString(36)}`;
  const page = `${d.pageUrl}?t=${token}`;
  await app.forward(["--show", page]);
  await app.waitFor("site mode: the page", (e) => lastReport(e, d.pageUrl) !== undefined, 30_000);
  const sizedFrom = app.events().length;
  await app.forward(["--size", `${WIDTH}x600`]);
  await settled(d, app, sizedFrom, WIDTH, "phone");
  const phoneReport = lastReport(app.events(), d.pageUrl);
  const phoneHeaders = documentRequests(d.agentRequests(), token).map(headerAgent);
  await d.sleep(500);
  d.screenshot("phone-pill.png");

  const desktop = await viaCommandBar(d, app, token, "request desktop", "desktop");
  const auto = await viaCommandBar(d, app, token, "automatic", "phone");

  // Informational: a page that fits a narrow window (Windows skips its reload).
  const wideFrom = app.events().length;
  await app.forward(["--size", "720x600"]);
  await settled(d, app, wideFrom, 720, "desktop");
  const fitsToken = `fits-${Date.now().toString(36)}`;
  const fitsFrom = app.events().length;
  await app.forward(["--show", `${d.pageUrl}?fits=1&t=${fitsToken}`]);
  await app.waitFor("site mode: ?fits=1", (e) => lastReport(e.slice(fitsFrom), d.pageUrl)?.width === 720, 30_000);
  await d.sleep(1000);
  const narrowFrom = app.events().length;
  await app.forward(["--size", `${WIDTH}x600`]);
  await app.waitFor(
    "site mode: ?fits=1 at 390",
    (e) => e.slice(narrowFrom).some((x) => (x.event === "ua-reload" && x.detail.startsWith("skipped")) || (x.event === "page-started" && x.detail.startsWith(d.pageUrl))),
    10_000,
  );
  await d.sleep(1500);
  const narrow = app.events().slice(narrowFrom);
  const fits = narrow.some((e) => e.event === "ua-reload" && e.detail.startsWith("skipped"))
    ? "skipped"
    : narrow.some((e) => e.event === "page-started" && e.detail.startsWith(d.pageUrl))
      ? "reloaded"
      : "none";

  // Last, the site on desktop again, and Settings → General listing it (and YouTube where it is
  // built in). Settings comes last: it would take the keyboard from the page window.
  await viaCommandBar(d, app, fitsToken, "request desktop", "desktop");
  const settingsFrom = app.events().length;
  await app.forward(["--settings", "general"]);
  await app.waitFor("site mode: Settings", (e) => e.slice(settingsFrom).some((x) => x.event === "settings-loaded"), 20_000);
  await d.sleep(1500);
  d.screenshot("settings-site-modes.png");
  await app.quit();
  return {
    events: app.events(),
    phoneHeaders,
    ...(phoneReport ? { phoneReport } : {}),
    desktop,
    auto,
    fits,
  };
}

/** What summary.json records of the run. */
export function siteModeSummary(run: SiteModeRun, os: string) {
  return {
    phoneHeaders: run.phoneHeaders,
    phoneReport: run.phoneReport ?? null,
    desktop: run.desktop ?? null,
    auto: run.auto ?? null,
    fits: run.fits,
    override: run.events.filter((e) => e.event === "ua-override").map((e) => e.detail),
    layoutMetrics: run.events.filter((e) => e.event === "layout-metrics").map((e) => e.detail),
    rules: siteModeFailures(run, os),
  };
}
