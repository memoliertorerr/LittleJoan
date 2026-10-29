// The smoke test's theme runs and their rules (pure, unit-tested; scripts/ci/smoke.mts drives
// them). Each run is its own launch on a fresh data folder whose settings.json carries a theme,
// so the main run's memory numbers don't move. The bar logs `theme <mode> colors=<n>` each time
// it loads (start_info, src-tauri/src/commands.rs) — the mode and a count, never the colors —, and
// before that `theme-early <mode> colors=<n>` when it painted its initialization script's theme
// before its first frame (LJ38). A summon from the ready window (LJ19.4) hands the bar its theme
// again: `theme-summon <mode> colors=<n>`.
import type { LogEvent } from "./smoke-log.mts";
import { sawSettings, sawSmokeTopSite } from "./smoke-command.mts";

export interface ThemeEvent {
  mode: string;
  colors: number;
}

/** Reads a `theme` event's detail; anything else is undefined. */
export function parseTheme(event: LogEvent): ThemeEvent | undefined {
  return event.event === "theme" ? themeDetail(event.detail) : undefined;
}

/** Reads a `theme-summon` event's detail (a summon from ready handed the bar its theme, LJ19.4). */
export function parseSummonTheme(event: LogEvent): ThemeEvent | undefined {
  return event.event === "theme-summon" ? themeDetail(event.detail) : undefined;
}

/** Reads a `theme-early` event's detail (the bar painted its first theme, LJ38). */
export function parseEarlyTheme(event: LogEvent): ThemeEvent | undefined {
  return event.event === "theme-early" ? themeDetail(event.detail) : undefined;
}

function themeDetail(detail: string): ThemeEvent | undefined {
  const match = /^(system|light|dark) colors=(\d+)$/.exec(detail);
  return match?.[1] && match[2] ? { mode: match[1], colors: Number(match[2]) } : undefined;
}

/** A theme as settings.json holds it (fields left out take the default). */
export interface SmokeTheme {
  mode: "system" | "light" | "dark";
  colors?: string[];
  intensity?: number;
  grain?: number;
}

/** Screenshots a run takes, under <out>/theme/. */
export interface ThemeShots {
  page?: string;
  commandBar: string;
  history?: string;
  settings?: string;
}

export interface ThemeRun {
  /** Names the data folder and the log (`smoke-<name>.log`). */
  name: string;
  theme: SmokeTheme;
  shots: ThemeShots;
}

/** Two muted colors with grain 0.4, the plan's check (LJ27), in light and dark. */
const TWO_COLORS = ["#9d8fd0", "#e0a2a0"];

export const THEME_RUNS: readonly ThemeRun[] = [
  {
    name: "theme-light",
    theme: { mode: "light", colors: TWO_COLORS, intensity: 0.6, grain: 0.4 },
    shots: { page: "page.png", commandBar: "command-bar.png", history: "history.png", settings: "settings.png" },
  },
  {
    name: "theme-dark",
    theme: { mode: "dark", colors: TWO_COLORS, intensity: 0.6, grain: 0.4 },
    shots: { page: "page-dark.png", commandBar: "command-bar-dark.png", settings: "settings-dark.png" },
  },
  {
    // The default colors (none given), in dark.
    name: "theme-default-dark",
    theme: { mode: "dark" },
    shots: { commandBar: "command-bar-default-dark.png" },
  },
];

/**
 * A theme run: every time the bar loaded it was handed the run's theme (its mode, and its number
 * of colors — 1 for the default), the smoke page loaded when the run shows a page, the command bar
 * offered the smoke page's site, and Settings opened on Appearance (the theme editor) and loaded
 * when the run shows it.
 */
export function themeFailures(run: ThemeRun, events: LogEvent[]): string[] {
  const failures: string[] = [];
  const handed = events.map(parseTheme).filter((t) => t !== undefined);
  const wanted = { mode: run.theme.mode, colors: run.theme.colors?.length ?? 1 };
  if (handed.length === 0) {
    failures.push(`${run.name}: the bar never asked for its theme`);
  }
  for (const theme of handed) {
    if (theme.mode !== wanted.mode || theme.colors !== wanted.colors) {
      failures.push(`${run.name}: the bar got a ${theme.mode} theme with ${theme.colors} colors, not ${wanted.mode} with ${wanted.colors}`);
    }
  }
  for (const theme of events.map(parseSummonTheme).filter((t) => t !== undefined)) {
    if (theme.mode !== wanted.mode || theme.colors !== wanted.colors) {
      failures.push(`${run.name}: a summon from ready handed the bar a ${theme.mode} theme with ${theme.colors} colors, not ${wanted.mode} with ${wanted.colors}`);
    }
  }
  failures.push(...earlyThemeFailures(run, events));
  if (run.shots.page && !events.some((e) => e.event === "title" && e.detail.startsWith("LJ-SMOKE ready"))) {
    failures.push(`${run.name}: the smoke page never loaded`);
  }
  if (!sawSmokeTopSite(events)) {
    failures.push(`${run.name}: the command bar never offered the smoke page's site`);
  }
  if (run.shots.settings && !sawSettings(events, "appearance")) {
    failures.push(`${run.name}: Settings never opened on Appearance (or its page never loaded)`);
  }
  return failures;
}

/**
 * The bar painted its theme before its first frame (LJ38): every `theme` from start_info comes
 * after a `theme-early` of the run's theme, logged since the previous `theme`.
 */
export function earlyThemeFailures(run: ThemeRun, events: LogEvent[]): string[] {
  const failures: string[] = [];
  const wanted = { mode: run.theme.mode, colors: run.theme.colors?.length ?? 1 };
  let early: ThemeEvent | undefined;
  let window = 0;
  for (const event of events) {
    const painted = parseEarlyTheme(event);
    if (painted) {
      early = painted;
    }
    if (parseTheme(event)) {
      window += 1;
      if (!early) {
        failures.push(`${run.name}: window #${window}'s bar got its theme from start_info with no theme-early before it`);
      } else if (early.mode !== wanted.mode || early.colors !== wanted.colors) {
        failures.push(`${run.name}: window #${window}'s bar painted a ${early.mode} theme with ${early.colors} colors first, not ${wanted.mode} with ${wanted.colors}`);
      }
      early = undefined;
    }
  }
  return failures;
}
