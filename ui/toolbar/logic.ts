// The Little Arc-style toolbar's logic (pure, unit-tested): what it shows for the open page —
// the site's icon and domain in the middle, a back button on the left, "open in default browser"
// on the right. toolbar.ts draws it.

/** Domains longer than this are shortened in the middle. */
export const MAX_DOMAIN = 32;

/** Whether `host` (a URL's hostname) is an IP address: shown as is. */
function isIp(host: string): boolean {
  return /^\d{1,3}(\.\d{1,3}){3}$/.test(host) || host.startsWith("[");
}

/** `text` shortened to `max` characters with an ellipsis in the middle. */
export function middleEllipsis(text: string, max = MAX_DOMAIN): string {
  const chars = Array.from(text);
  if (chars.length <= max) {
    return text;
  }
  const keep = max - 1;
  const head = Math.ceil(keep / 2);
  return `${chars.slice(0, head).join("")}…${chars.slice(chars.length - (keep - head)).join("")}`;
}

/** The domain the toolbar shows for `address`: `www.` stripped, IPs as is, long ones shortened. */
export function displayDomain(address: string): string {
  let host: string;
  try {
    host = new URL(address).hostname;
  } catch {
    return "";
  }
  if (!isIp(host)) {
    host = host.replace(/^www\./i, "");
  }
  return middleEllipsis(host);
}

/** The site whose icon the toolbar shows (as the command bar names sites: with a port). */
export function siteFor(address: string): string {
  try {
    const url = new URL(address);
    return isIp(url.hostname) ? url.host : url.host.replace(/^www\./i, "");
  } catch {
    return "";
  }
}

export interface ToolbarView {
  /** Shown in the middle; "" with no page. */
  domain: string;
  /** Whose icon to show; "" for none. */
  site: string;
  /** The back button works: the page navigated within this window. */
  back: boolean;
  /** "Open in default browser" works: a web page is open. */
  openInBrowser: boolean;
}

/**
 * The toolbar for the page at `address` ("" = none). `canGoBack` comes from Rust, which counts
 * this window's page loads and backs (back_stack.rs): it is false in every new window.
 */
export function toolbarView(address: string, canGoBack: boolean): ToolbarView {
  const web = /^https?:\/\//i.test(address);
  return {
    domain: web ? displayDomain(address) : "",
    site: web ? siteFor(address) : "",
    back: web && canGoBack,
    openInBrowser: web,
  };
}

/** The platform the bar runs on, from its user agent (macOS puts the traffic lights in the bar). */
export function isMac(userAgent: string): boolean {
  return /Macintosh|Mac OS X/.test(userAgent);
}
