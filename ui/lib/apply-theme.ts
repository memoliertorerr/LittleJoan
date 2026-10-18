// Puts a theme on the page (DOM glue): theme.ts's variables go on <html> as inline custom
// properties — CSSOM, which the CSP's `style-src 'self'` allows — and `color-scheme` follows, so
// form controls match. A "system" theme follows the system's light / dark as it changes.
import { themeFrom, tokens, type Theme } from "./theme.ts";

let current: Theme | null = null;
let media: MediaQueryList | null = null;

function systemDark(): boolean {
  return media?.matches ?? false;
}

function paint(): void {
  if (!current) {
    return;
  }
  const { scheme, vars } = tokens(current, systemDark());
  const style = document.documentElement.style;
  for (const [name, value] of Object.entries(vars)) {
    style.setProperty(name, value);
  }
  style.setProperty("color-scheme", scheme);
}

/** Shows `value` (what Rust sent: anything, each field checked) from now on. */
export function applyTheme(value: unknown): Theme {
  current = themeFrom(value);
  if (!media && typeof window.matchMedia === "function") {
    media = window.matchMedia("(prefers-color-scheme: dark)");
    media.addEventListener("change", paint);
  }
  paint();
  return current;
}
