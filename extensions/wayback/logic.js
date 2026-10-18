// @ts-check
// Wayback Machine's addresses for a page, pure: the calendar of every snapshot
// (`https://web.archive.org/web/*/<url>`) and the newest snapshot (`…/web/<url>`). Little Joan
// fetches nothing here — the panel only opens one of these in the window.

export const ARCHIVE = "https://web.archive.org/web/";

/**
 * The page's address as the Wayback Machine takes it: http(s) only, without user name, password
 * or fragment (those never reach a server).
 * @param {string} address
 * @returns {string | null}
 */
export function archivable(address) {
  /** @type {URL} */
  let url;
  try {
    url = new URL(address.trim());
  } catch {
    return null;
  }
  if (url.protocol !== "http:" && url.protocol !== "https:") {
    return null;
  }
  url.username = "";
  url.password = "";
  url.hash = "";
  return url.toString();
}

/**
 * The Wayback Machine's pages for an address, or null when it isn't a web address.
 * @param {string} address
 * @returns {{ calendar: string, newest: string } | null}
 */
export function archiveUrls(address) {
  const page = archivable(address);
  return page ? { calendar: `${ARCHIVE}*/${page}`, newest: `${ARCHIVE}${page}` } : null;
}

/** @typedef {"calendar" | "newest"} Choice */
/** @type {Choice[]} */
export const CHOICES = ["calendar", "newest"];

/**
 * The choice ↑ / ↓ moves to (it stops at the ends).
 * @param {Choice} current
 * @param {-1 | 1} step
 * @returns {Choice}
 */
export function move(current, step) {
  const index = Math.min(CHOICES.length - 1, Math.max(0, CHOICES.indexOf(current) + step));
  return CHOICES[index] ?? current;
}
