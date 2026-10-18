// The smoke test's theme runs and their rules (pure, unit-tested; scripts/ci/smoke.mts drives
// them). Each run is its own launch on a fresh data folder whose settings.json carries a theme,
// so the main run's memory numbers don't move. The bar logs `theme <mode> colors=<n>` each time
// it loads (start_info, src-tauri/src/commands.rs) — the mode and a count, never the colors.
import type { LogEvent } from "./smoke-log.mts";
import { sawSettings, sawSmokeTopSite } from "./smoke-command.mts";

export interface ThemeEvent {
  mode: string;
  colors: number;
}

/** Reads a `theme` event's detail; anything else is undefined. */
export function parseTheme(event: LogEvent): ThemeEvent | undefined {
  if (event.event !== "theme") {
    return undefined;
  }
  const match = /^(system|light|dark) colors=(\d+)$/.exec(event.detail);
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
 * offered the smoke page's site, and Settings opened and loaded when the run shows it.
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
  if (run.shots.page && !events.some((e) => e.event === "title" && e.detail.startsWith("LJ-SMOKE ready"))) {
    failures.push(`${run.name}: the smoke page never loaded`);
  }
  if (!sawSmokeTopSite(events)) {
    failures.push(`${run.name}: the command bar never offered the smoke page's site`);
  }
  if (run.shots.settings && !sawSettings(events, "general")) {
    failures.push(`${run.name}: Settings never opened (or its page never loaded)`);
  }
  return failures;
}
