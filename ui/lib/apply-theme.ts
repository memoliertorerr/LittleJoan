// Puts a theme on the page (DOM glue): theme.ts's variables go on <html> as inline custom
// properties — CSSOM, which the CSP's `style-src 'self'` allows — and `color-scheme` follows, so
// form controls match. A "system" theme follows the system's light / dark as it changes. The bar
// passes `onPaint`, which hears every set of variables it puts on (for extension panels).
import { themeFrom, tokens, type Theme, type ThemeTokens } from "./theme.ts";

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
  const style = document.documentElement.style;
  for (const [name, value] of Object.entries(shown.vars)) {
    style.setProperty(name, value);
  }
  style.setProperty("color-scheme", shown.scheme);
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
