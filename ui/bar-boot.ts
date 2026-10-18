// The bar's first paint (LJ38). Rust gives the bar's webview — only the bar's, never the page's —
// an initialization script, `window.__littleJoanTheme = <theme>;`, which runs at document start,
// before this page's own scripts. bar.ts calls bootTheme first, at module top level, so the theme
// is on <html> before anything awaits — before the first frame of a page as small as bar.html —
// instead of the default theme (theme-defaults.css) flashing until start_info answers.
// start_info's theme is applied again only when it differs (a preview may have changed it).
// Pure, unit-tested; bar.ts does the DOM.
import { themeFrom, type Theme } from "./lib/theme.ts";

declare global {
  interface Window {
    __littleJoanTheme?: unknown;
  }
}

/**
 * The theme the initialization script left, checked like any theme Rust sends (themeFrom) — or
 * null when there is none or it isn't a theme at all (then the bar waits for start_info, as
 * before).
 */
export function earlyTheme(value: unknown): Theme | null {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return null;
  }
  const raw = value as Record<string, unknown>;
  if (typeof raw.mode !== "string" || !Array.isArray(raw.colors)) {
    return null;
  }
  return themeFrom(value);
}

/** Whether two themes look the same. */
export function sameTheme(a: Theme, b: Theme): boolean {
  return (
    a.mode === b.mode &&
    a.intensity === b.intensity &&
    a.grain === b.grain &&
    a.colors.length === b.colors.length &&
    a.colors.every((color, i) => color === b.colors[i])
  );
}

/**
 * Applies the early theme at once — `apply` runs before this returns, nothing awaits — and
 * returns it; null (and nothing applied) without one.
 */
export function bootTheme(value: unknown, apply: (theme: Theme) => void): Theme | null {
  const theme = earlyTheme(value);
  if (theme) {
    apply(theme);
  }
  return theme;
}

/** Whether start_info's theme must still be applied: always without an early one, else only when it differs. */
export function needsTheme(early: Theme | null, fromStart: unknown): boolean {
  return early === null || !sameTheme(early, themeFrom(fromStart));
}
