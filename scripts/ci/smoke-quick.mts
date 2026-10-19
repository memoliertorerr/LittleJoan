// The smoke test's quick-commands run (LJ42) and its rules. Its own launch on a fresh data folder
// (default settings): a new window's command bar, typing "mobile", ↓ ↓ Enter (the search row
// first, then "Mobile sites — on") must log `quick-command setMobileSites false` and leave
// `"mobileSites": false` in settings.json; a new command bar, "dark", ↓ ↓ Enter must log
// `quick-command setThemeMode dark`, hand the bar `theme dark colors=<n>` and save the theme's mode
// `dark`; typing "theme" shows the theme rows (`quick-theme.png`). Then over the smoke page:
// Ctrl/Cmd+L, "copy" (`quick-page.png`), " link", ↓ ↓ Enter copies the page's address — read back
// from the clipboard where the runner has a tool for it (informational, as for quick answers).
// Typed on Windows and Linux; best effort on macOS (its rules apply when the typing showed up).
// smoke.mts supplies the driver; the rules are pure and unit-tested (smoke-quick.test.ts).
import type { LogEvent } from "./smoke-log.mts";
import type { SiteApp } from "./smoke-sitemode.mts";

/** What the run needs from smoke.mts. */
export interface QuickDriver {
  os: string;
  pageUrl: string;
  launch(name: string, settings: Record<string, unknown>): Promise<SiteApp>;
  screenshot(name: string): void;
  sleep(ms: number): Promise<void>;
  typeText(text: string, pid: number, activate: boolean): void;
  pressKeys(keys: string[], pid: number, activate: boolean): void;
  pressShortcut(letter: string, pid: number): void;
  readClipboard(): string | null;
}

/** One quick command run through the command bar. */
export interface QuickStep {
  /** The command bar showed the typing. */
  typed: boolean;
  /** Its `quick-command` lines. */
  logged: string[];
  /** settings.json right after. */
  settings: Record<string, unknown> | null;
}

export interface QuickRun {
  events: LogEvent[];
  mobile: QuickStep;
  dark: QuickStep;
  /** The `theme` lines after the dark step's command. */
  themes: string[];
  /** Copy link over the smoke page: typed, and what the clipboard held (null: no tool). */
  copy: { typed: boolean; clipboard: string | null; url: string };
}

/** settings.json as an object (null when missing or not JSON). */
export function settingsOf(text: string | undefined): Record<string, unknown> | null {
  if (!text) {
    return null;
  }
  try {
    const value = JSON.parse(text) as unknown;
    return typeof value === "object" && value !== null && !Array.isArray(value) ? (value as Record<string, unknown>) : null;
  } catch {
    return null;
  }
}

/** The theme's mode in settings.json. */
function themeMode(settings: Record<string, unknown> | null): unknown {
  const theme = settings?.theme;
  return typeof theme === "object" && theme !== null ? (theme as Record<string, unknown>).mode : undefined;
}

/**
 * The rules, where the typing is driven (Windows, Linux; macOS when it showed up): "mobile" logs
 * `quick-command setMobileSites false` and saves `"mobileSites": false`; "dark" logs
 * `quick-command setThemeMode dark`, hands the bar a `theme dark …` and saves the mode `dark`.
 * Copy link is informational.
 */
export function quickFailures(run: QuickRun, os: string): string[] {
  const failures: string[] = [];
  const driven = os !== "darwin";
  const check = (name: string, step: QuickStep, want: string, saved: () => boolean, savedText: string) => {
    if (!step.typed) {
      if (driven) {
        failures.push(`quick: the command bar never showed the typed "${name}"`);
      }
      return;
    }
    if (!step.logged.includes(want)) {
      failures.push(`quick: "${name}" logged [${step.logged.join(", ")}], not "quick-command ${want}"`);
    }
    if (!saved()) {
      failures.push(`quick: after "${name}" settings.json didn't say ${savedText}`);
    }
  };
  check("mobile", run.mobile, "setMobileSites false", () => run.mobile.settings?.mobileSites === false, '"mobileSites": false');
  check("dark", run.dark, "setThemeMode dark", () => themeMode(run.dark.settings) === "dark", 'the theme\'s mode "dark"');
  if (run.dark.typed && !run.themes.some((detail) => detail.startsWith("dark colors="))) {
    failures.push(`quick: after "dark" the bar was handed [${run.themes.join(", ")}], not a dark theme`);
  }
  return failures;
}

/** Waits for the command bar of a new window (its extensions list, read as it opens). */
async function freshCommandBar(d: QuickDriver, app: SiteApp): Promise<number> {
  const from = app.events().length;
  if (app.events().some((e) => e.event === "window-created")) {
    const hibernated = app.events().filter((e) => e.event === "hibernated").length;
    await app.forward(["--hide"]);
    await app.waitFor("quick: hibernated", (e) => e.filter((x) => x.event === "hibernated").length > hibernated, 20_000);
  }
  await app.forward(["--show"]);
  await app.waitFor("quick: the command bar", (e) => e.slice(from).some((x) => x.event === "extensions-list"), 20_000);
  await d.sleep(1500);
  return from;
}

/** Types `text` in a new command bar and, once it showed up, ↓ ↓ Enter. */
async function viaCommandBar(d: QuickDriver, app: SiteApp, text: string, settings: () => string | undefined): Promise<QuickStep> {
  const from = await freshCommandBar(d, app);
  const pid = app.pid();
  d.typeText(text, pid, true);
  const typed = await app.waitFor(
    `quick: typed "${text}"`,
    (e) => e.slice(from).some((x) => x.event === "suggestions" && x.detail.endsWith(`typed=${text.length}`)),
    15_000,
  );
  if (typed) {
    await d.sleep(800);
    d.pressKeys(["Down", "Down", "Return"], pid, true);
    await app.waitFor(`quick: "${text}" ran`, (e) => e.slice(from).some((x) => x.event === "quick-command"), 15_000);
    await d.sleep(800);
  }
  return {
    typed,
    logged: app
      .events()
      .slice(from)
      .filter((e) => e.event === "quick-command")
      .map((e) => e.detail),
    settings: settingsOf(settings()),
  };
}

/** Runs the quick-commands launch. */
export async function quickRun(d: QuickDriver): Promise<QuickRun> {
  const app = await d.launch("quick", {});
  await app.waitFor("quick: started", (e) => e.some((x) => x.event === "started"), 30_000);
  await d.sleep(1500);
  const mobile = await viaCommandBar(d, app, "mobile", () => app.settings());
  const darkFrom = app.events().length;
  const dark = await viaCommandBar(d, app, "dark", () => app.settings());
  const themes = app
    .events()
    .slice(darkFrom)
    .filter((e) => e.event === "theme")
    .map((e) => e.detail);

  // The theme rows (a screenshot to look at: their state marks).
  const themeFrom = await freshCommandBar(d, app);
  d.typeText("theme", app.pid(), true);
  await app.waitFor("quick: typed theme", (e) => e.slice(themeFrom).some((x) => x.event === "suggestions" && x.detail.endsWith("typed=5")), 15_000);
  await d.sleep(1000);
  d.screenshot("quick-theme.png");

  // Over the smoke page: the page commands.
  const url = `${d.pageUrl}?quick=1`;
  const pageFrom = app.events().length;
  await app.forward(["--show", url]);
  await app.waitFor("quick: the smoke page", (e) => e.slice(pageFrom).some((x) => x.event === "title" && x.detail.startsWith("LJ-SMOKE")), 30_000);
  await d.sleep(1500);
  const pid = app.pid();
  const barFrom = app.events().length;
  d.pressShortcut("l", pid);
  await app.waitFor("quick: the command bar over the page", (e) => e.slice(barFrom).some((x) => x.event === "overlay" && x.detail === "true"), 10_000);
  await d.sleep(800);
  // The window is in front already (Ctrl/Cmd+L went to it): no second activation.
  d.typeText("copy", pid, false);
  await app.waitFor("quick: typed copy", (e) => e.slice(barFrom).some((x) => x.event === "suggestions" && x.detail.endsWith("typed=4")), 15_000);
  await d.sleep(1000);
  d.screenshot("quick-page.png");
  d.typeText(" link", pid, false);
  const copyTyped = await app.waitFor(
    "quick: typed copy link",
    (e) => e.slice(barFrom).some((x) => x.event === "suggestions" && x.detail.endsWith("typed=9")),
    15_000,
  );
  let clipboard: string | null = null;
  if (copyTyped) {
    await d.sleep(800);
    d.pressKeys(["Down", "Down", "Return"], pid, false);
    await d.sleep(1500);
    clipboard = d.readClipboard();
  }
  await app.quit();
  return { events: app.events(), mobile, dark, themes, copy: { typed: copyTyped, clipboard, url } };
}

/** What summary.json records of the run. */
export function quickSummary(run: QuickRun, os: string) {
  return {
    mobile: { typed: run.mobile.typed, logged: run.mobile.logged, mobileSites: run.mobile.settings?.mobileSites ?? null },
    dark: { typed: run.dark.typed, logged: run.dark.logged, mode: themeMode(run.dark.settings) ?? null, themes: run.themes },
    copyLink: { typed: run.copy.typed, clipboard: run.copy.clipboard, matches: run.copy.clipboard?.trim() === run.copy.url },
    rules: quickFailures(run, os),
  };
}
