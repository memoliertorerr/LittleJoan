// @ts-check
// Copy as Markdown link, pure: the page's title and address as a Markdown link — with whatever
// would break it escaped — or as HTML, plain text or an Org-mode link.

/** @typedef {"markdown" | "html" | "plain" | "org"} Format */
/** @type {{ id: Format, label: string }[]} */
export const FORMATS = [
  { id: "markdown", label: "Markdown" },
  { id: "html", label: "HTML" },
  { id: "plain", label: "Title and address" },
  { id: "org", label: "Org mode" },
];

/**
 * A title on one line: runs of whitespace (newlines too) become one space.
 * @param {string} text
 */
function oneLine(text) {
  return text.replace(/\s+/g, " ").trim();
}

/**
 * Link text for Markdown: backslashes, brackets and the characters that start emphasis or code
 * are escaped, so `[a] *b*` shows as written.
 * @param {string} text
 */
export function escapeMarkdown(text) {
  return oneLine(text).replace(/[\\[\]*_`<>]/g, (c) => `\\${c}`);
}

/**
 * An address as a Markdown link destination: spaces, parentheses and angle brackets
 * percent-encoded, so the link ends where it should.
 * @param {string} url
 */
export function markdownUrl(url) {
  return url.trim().replace(/[\s()<>]/g, (c) => `%${c.charCodeAt(0).toString(16).toUpperCase().padStart(2, "0")}`);
}

/**
 * Text for HTML (content and attribute values).
 * @param {string} text
 */
export function escapeHtml(text) {
  return text.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c] ?? c);
}

/**
 * The link in a format; an empty title falls back to the address.
 * @param {Format} format
 * @param {string} title
 * @param {string} url
 */
export function link(format, title, url) {
  const text = oneLine(title) || url.trim();
  switch (format) {
    case "markdown":
      return `[${escapeMarkdown(text)}](${markdownUrl(url)})`;
    case "html":
      return `<a href="${escapeHtml(url.trim())}">${escapeHtml(text)}</a>`;
    case "plain":
      return text === url.trim() ? text : `${text} — ${url.trim()}`;
    case "org":
      return `[[${url.trim().replace(/[[\]]/g, (c) => (c === "[" ? "%5B" : "%5D"))}][${text.replace(/[[\]]/g, (c) => (c === "[" ? "(" : ")"))}]]`;
  }
}

/**
 * A page title without the site's name tacked on (" - YouTube", " | The Site"), when the rest
 * still says something. Offered, never forced: the field can be edited back.
 * @param {string} title
 */
export function trimSiteName(title) {
  const match = /^(.{8,}?)\s+[-–—|·•]\s+[^-–—|·•]{2,40}$/.exec(oneLine(title));
  return match?.[1] ?? oneLine(title);
}

/**
 * The format ↑ / ↓ moves to (it stops at the ends).
 * @param {Format} current
 * @param {-1 | 1} step
 * @returns {Format}
 */
export function move(current, step) {
  const index = FORMATS.findIndex((f) => f.id === current);
  return FORMATS[Math.min(FORMATS.length - 1, Math.max(0, index + step))]?.id ?? current;
}
