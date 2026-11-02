// Puts a theme on the page (DOM glue): theme.ts's variables go on <html> as inline custom
// properties — CSSOM, which the CSP's `style-src 'self'` allows — and `color-scheme` follows, so
// form controls match. A "system" theme follows the system's light / dark as it changes. The bar
// passes `onPaint`, which hears every set of variables it puts on (for extension panels). With
// Liquid Glass (LJ19.9: a theme Rust hands the bar on macOS 26+) <html> gets `data-glass` and the
// card's colour over the glass (`--lj-glass-card`, never reported); style.css then draws no
// background under the bar, so the native glass behind it shows.
import { glassCard, themeFrom, tokens, type Theme, type ThemeTokens } from "./theme.ts";

let current: Theme | null = null;
let media: MediaQueryList | null = null;
let painted: ((tokens: ThemeTokens) => void) | null = null;

function systemDark(): boolean {
  return media?.matches ?? false;
}

function paint(): void {
  if (!current) {
    return;
  }
  const shown = tokens(current, systemDark());
  const root = document.documentElement;
  const style = root.style;
  for (const [name, value] of Object.entries(shown.vars)) {
    style.setProperty(name, value);
  }
  style.setProperty("color-scheme", shown.scheme);
  if (current.glass) {
    root.dataset.glass = current.glass;
    style.setProperty("--lj-glass-card", glassCard(current, systemDark(), current.glass));
  } else if (root.dataset.glass !== undefined) {
    delete root.dataset.glass;
    style.removeProperty("--lj-glass-card");
  }
  painted?.(shown);
}

/**
 * Shows `value` (what Rust sent: anything, each field checked) from now on; `onPaint` (once
 * given, kept) hears the variables each time they change.
 */
export function applyTheme(value: unknown, onPaint?: (tokens: ThemeTokens) => void): Theme {
  current = themeFrom(value);
  if (onPaint) {
    painted = onPaint;
  }
  if (!media && typeof window.matchMedia === "function") {
    media = window.matchMedia("(prefers-color-scheme: dark)");
    media.addEventListener("change", paint);
  }
  paint();
  return current;
}
