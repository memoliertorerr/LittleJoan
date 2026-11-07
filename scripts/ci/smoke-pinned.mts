// The smoke test's pinned run (LJ50) and its rules. Its own launch on a fresh data folder (no
// history) whose settings.json pins the smoke page (`pinnedSites: [{ url: <page>?pinned=1, title:
// "LJ pinned" }]`): the very first window's command bar must lead with it — the log's first
// `suggestions` line says `first=127.0.0.1:<port>` —, and `pinned.png` shows the row with its pin.
// Then over another page (the smoke page's `?other=1`): Ctrl/Cmd+L, typing "pin", ↓ ↓ Enter (the
// search row, then "Pin this site") must log `quick-command setPinned true` and leave that page
// last in settings.json's `pinnedSites`. Typed on Windows and Linux; best effort on macOS (its
// typing rule applies when the typing showed up). smoke.mts supplies the driver; the rules are pure
// and unit-tested (smoke-pinned.test.ts).
import type { LogEvent } from "./smoke-log.mts";
import type { SiteApp } from "./smoke-sitemode.mts";
import { settingsOf } from "./smoke-quick.mts";

/** What the run pins in settings.json before it starts. */
export const PINNED_TITLE = "LJ pinned";

/** What the run needs from smoke.mts. */
export interface PinnedDriver {
  os: string;
  pageUrl: string;
  launch(name: string, settings: Record<string, unknown>): Promise<SiteApp>;
  screenshot(name: string): void;
  sleep(ms: number): Promise<void>;
  typeText(text: string, pid: number, activate: boolean): void;
  pressKeys(keys: string[], pid: number, activate: boolean): void;
  pressShortcut(letter: string, pid: number): void;
}

export interface PinnedRun {
  events: LogEvent[];
  /** The page pinned in settings.json from the start, and the one "Pin this site" pins. */
  pinned: string;
  other: string;
  /** The command bar showed the typed "pin". */
  typed: boolean;
  /** settings.json at the end. */
  settings: Record<string, unknown> | null;
}

/** The address the run pins first, and the other page it pins from the command bar. */
export function pinnedUrls(pageUrl: string): { pinned: string; other: string } {
  return { pinned: `${pageUrl}?pinned=1`, other: `${pageUrl}?other=1` };
}

/** settings.json's pinned addresses, in order. */
export function pinnedAddresses(settings: Record<string, unknown> | null): string[] {
  const list = settings?.pinnedSites;
  return Array.isArray(list)
    ? list.flatMap((item: unknown) => {
        const url = (item as Record<string, unknown> | null)?.url;
        return typeof url === "string" ? [url] : [];
      })
    : [];
}

/**
 * The rules: the first `suggestions` line leads with the pinned page's site; where the typing is
 * driven (Windows, Linux; macOS when it showed up) "pin" ran `setPinned true` and settings.json
 * pins the first page and then the other one.
 */
export function pinnedFailures(run: PinnedRun, os: string): string[] {
  const failures: string[] = [];
  const site = new URL(run.pinned).host;
  const first = run.events.find((e) => e.event === "suggestions");
  if (!first) {
    failures.push("pinned: the first window's command bar never asked for suggestions");
  } else if (!first.detail.includes(` first=${site} `)) {
    failures.push(`pinned: the first window's command bar led with "${first.detail}", not the pinned ${site}`);
  }
  if (!run.typed) {
    if (os !== "darwin") {
      failures.push('pinned: the command bar never showed the typed "pin"');
    }
    return failures;
  }
  const logged = run.events.filter((e) => e.event === "quick-command").map((e) => e.detail);
  if (!logged.includes("setPinned true")) {
    failures.push(`pinned: "pin" logged [${logged.join(", ")}], not "quick-command setPinned true"`);
  }
  const saved = pinnedAddresses(run.settings);
  if (saved.length !== 2 || saved[0] !== run.pinned || saved[1] !== run.other) {
    failures.push(`pinned: settings.json pins [${saved.join(", ")}], not the first page and then ${run.other}`);
  }
  return failures;
}

/** Runs the pinned launch. */
export async function pinnedRun(d: PinnedDriver): Promise<PinnedRun> {
  const { pinned, other } = pinnedUrls(d.pageUrl);
  const app = await d.launch("pinned", { pinnedSites: [{ url: pinned, title: PINNED_TITLE }] });
  await app.waitFor("pinned: started", (e) => e.some((x) => x.event === "started"), 30_000);
  await d.sleep(1500);
  const from = app.events().length;
  await app.forward(["--show"]);
  await app.waitFor("pinned: the first command bar", (e) => e.slice(from).some((x) => x.event === "overlay" && x.detail === "true") && e.some((x) => x.event === "suggestions"), 20_000);
  await d.sleep(1500);
  d.screenshot("pinned.png");
  const pageFrom = app.events().length;
  await app.forward(["--show", other]);
  await app.waitFor("pinned: the other page", (e) => e.slice(pageFrom).some((x) => x.event === "title" && x.detail.startsWith("LJ-SMOKE")), 30_000);
  await d.sleep(1500);
  const pid = app.pid();
  const barFrom = app.events().length;
  d.pressShortcut("l", pid);
  await app.waitFor("pinned: the command bar over the page", (e) => e.slice(barFrom).some((x) => x.event === "overlay" && x.detail === "true"), 10_000);
  await d.sleep(800);
  // The window is in front already (Ctrl/Cmd+L went to it): no second activation.
  d.typeText("pin", pid, false);
  const typed = await app.waitFor('pinned: typed "pin"', (e) => e.slice(barFrom).some((x) => x.event === "suggestions" && x.detail.endsWith("typed=3")), 15_000);
  if (typed) {
    await d.sleep(800);
    d.pressKeys(["Down", "Down", "Return"], pid, false);
    await app.waitFor("pinned: Pin this site ran", (e) => e.slice(barFrom).some((x) => x.event === "quick-command" && x.detail.startsWith("setPinned")), 15_000);
    await d.sleep(800);
  }
  const settings = settingsOf(app.settings());
  await app.quit();
  return { events: app.events(), pinned, other, typed, settings };
}

/** What summary.json records of the run. */
export function pinnedSummary(run: PinnedRun, os: string) {
  return {
    firstSuggestions: run.events.find((e) => e.event === "suggestions")?.detail ?? null,
    typed: run.typed,
    logged: run.events.filter((e) => e.event === "quick-command").map((e) => e.detail),
    pinnedSites: pinnedAddresses(run.settings),
    rules: pinnedFailures(run, os),
  };
}
