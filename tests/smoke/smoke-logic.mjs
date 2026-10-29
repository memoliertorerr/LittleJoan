// @ts-check
// Logic of the CI smoke page (smoke.html). Plain JavaScript so the page can load it as is from a
// throwaway HTTP server; type-checked through JSDoc and unit-tested by smoke-logic.test.ts.
// The page reports through document.title, which the app writes to its smoke log — no IPC.

/** Media types probed on each engine. */
export const CODECS = {
  h264: 'video/mp4; codecs="avc1.42E01E"',
  vp9: 'video/webm; codecs="vp9"',
  av1: 'video/mp4; codecs="av01.0.05M.08"',
};

/**
 * How each codec is supported: "mse" (Media Source Extensions — what YouTube uses), else what
 * canPlayType says ("probably" / "maybe"), else "no".
 * @param {{ canPlayType(type: string): string }} video
 * @param {{ isTypeSupported(type: string): boolean } | undefined} mediaSource
 * @returns {Record<keyof typeof CODECS, string>}
 */
export function codecSupport(video, mediaSource) {
  /** @param {string} type */
  const probe = (type) => {
    if (mediaSource && mediaSource.isTypeSupported(type)) {
      return "mse";
    }
    return video.canPlayType(type) || "no";
  };
  return { h264: probe(CODECS.h264), vp9: probe(CODECS.vp9), av1: probe(CODECS.av1) };
}

/** @param {Record<keyof typeof CODECS, string>} support */
export function readyTitle(support) {
  return `LJ-SMOKE ready h264=${support.h264} vp9=${support.vp9} av1=${support.av1}`;
}

/** @param {boolean} on */
export function fullscreenTitle(on) {
  return `LJ-SMOKE fullscreen=${on ? "on" : "off"}`;
}

/**
 * Whether the document has a fullscreen element (standard or older WebKit-prefixed API).
 * @param {{ fullscreenElement?: Element | null, webkitFullscreenElement?: Element | null }} doc
 */
export function isFullscreen(doc) {
  return Boolean(doc.fullscreenElement ?? doc.webkitFullscreenElement);
}

/**
 * Requests fullscreen for `element`, using the WebKit-prefixed API where the standard one is
 * missing (older WKWebView).
 * @param {{ requestFullscreen?: () => Promise<void>, webkitRequestFullscreen?: () => void }} element
 */
export function requestFullscreen(element) {
  if (typeof element.requestFullscreen === "function") {
    element.requestFullscreen().catch(() => undefined);
  } else if (typeof element.webkitRequestFullscreen === "function") {
    element.webkitRequestFullscreen();
  }
}

/**
 * Whether a user agent is a phone browser's: iPhone Safari and Chrome on Android both carry a
 * "Mobile" token, desktop engines don't.
 * @param {string} userAgent
 */
export function isPhoneAgent(userAgent) {
  return /\bMobile\b/.test(userAgent);
}

/**
 * The title that reports what the page sees: its CSS viewport, the kind of user agent, and what
 * the client hints say (`navigator.userAgentData.mobile`: Chromium only, so `undefined` on
 * WebKit — LJ40 checks Windows' phone agent says `true`).
 * @param {number} width window.innerWidth
 * @param {number} height window.innerHeight
 * @param {string} userAgent navigator.userAgent
 * @param {number} pixelRatio window.devicePixelRatio
 * @param {boolean | undefined} [uaMobile] navigator.userAgentData?.mobile
 */
export function viewportTitle(width, height, userAgent, pixelRatio, uaMobile) {
  const agent = isPhoneAgent(userAgent) ? "phone" : "desktop";
  return `LJ-SMOKE viewport=${Math.round(width)}x${Math.round(height)} ua=${agent} dpr=${pixelRatio} uamobile=${String(uaMobile)}`;
}

/**
 * Whether the page lays itself out like a desktop page with a minimum width (600 px) — under a
 * desktop agent, unless `?fits=1` — so it doesn't fit a phone-width window. Windows skips a
 * phone-mode reload for a page that already fits (CDP's layout metrics, LJ40); with this the
 * smoke page reloads like most desktop sites do, and `?fits=1` shows the skip.
 * @param {string} userAgent navigator.userAgent
 * @param {string} search location.search
 */
export function desktopMinWidth(userAgent, search) {
  return !isPhoneAgent(userAgent) && new URLSearchParams(search).get("fits") !== "1";
}

/** The localStorage key the persistence check writes and reads. */
export const PERSIST_STORAGE_KEY = "lj_l";

/** The title `?persist=set` reports once it has written everything. */
export const PERSIST_SET_TITLE = "LJ-PERSIST set";

/**
 * The persistence check a page URL asks for: `?persist=set&t=<token>` or
 * `?persist=check&t=<token>`. Tokens are 1–64 letters, digits, `_` or `-` (cookie-safe).
 * @param {string} search location.search
 * @returns {{ mode: "set" | "check", token: string } | undefined}
 */
export function persistRequest(search) {
  const params = new URLSearchParams(search);
  const mode = params.get("persist");
  const token = params.get("t") ?? "";
  if ((mode === "set" || mode === "check") && /^[\w-]{1,64}$/.test(token)) {
    return { mode, token };
  }
  return undefined;
}

/** How often a `?ping=<id>` page pings the smoke server (LJ19.4). */
export const PING_EVERY_MS = 250;

/**
 * `?ping=<id>`: the "nothing of the page survives a close" check (LJ19.4) — the page pings the
 * smoke server every PING_EVERY_MS for as long as it runs. The id is the smoke test's own word.
 * @param {string} search location.search
 * @returns {{ id: string } | undefined}
 */
export function pingRequest(search) {
  const id = new URLSearchParams(search).get("ping") ?? "";
  return /^[\w-]{1,32}$/.test(id) ? { id } : undefined;
}

/**
 * The ping's address on the smoke server (same origin).
 * @param {string} id
 * @param {number} n
 */
export function pingUrl(id, n) {
  return `/ping?id=${encodeURIComponent(id)}&n=${n}`;
}

/**
 * The cookies `?persist=set` writes: a persistent one (a year), and a session one (no expiry,
 * so it must end with the app run).
 * @param {string} token
 */
export function persistCookies(token) {
  return [`lj_p=${token}; Max-Age=31536000; Path=/; SameSite=Lax`, `lj_s=${token}; Path=/`];
}

/**
 * Writes the persistent cookie, the session cookie and the localStorage value; returns the
 * title to report.
 * @param {{ cookie: string }} doc document
 * @param {{ setItem(key: string, value: string): void }} storage localStorage
 * @param {string} token
 */
export function persistSet(doc, storage, token) {
  for (const cookie of persistCookies(token)) {
    doc.cookie = cookie;
  }
  storage.setItem(PERSIST_STORAGE_KEY, token);
  return PERSIST_SET_TITLE;
}

/**
 * `document.cookie` as name → value (the first of a repeated name wins).
 * @param {string} header
 * @returns {Map<string, string>}
 */
export function parseCookies(header) {
  /** @type {Map<string, string>} */
  const cookies = new Map();
  for (const pair of header.split(";")) {
    const at = pair.indexOf("=");
    const name = at > 0 ? pair.slice(0, at).trim() : "";
    if (name && !cookies.has(name)) {
      cookies.set(name, pair.slice(at + 1).trim());
    }
  }
  return cookies;
}

/**
 * What survived, as the title `?persist=check` reports: 1 where the persistent cookie (p), the
 * session cookie (s) or the localStorage value (l) still holds `token`, else 0.
 * @param {string} cookieHeader document.cookie
 * @param {string | null} stored localStorage's value (null when missing or unreadable)
 * @param {string} token
 */
export function persistTitle(cookieHeader, stored, token) {
  const cookies = parseCookies(cookieHeader);
  /** @param {string | null | undefined} value */
  const bit = (value) => (value === token ? 1 : 0);
  return `LJ-PERSIST p=${bit(cookies.get("lj_p"))} s=${bit(cookies.get("lj_s"))} l=${bit(stored)}`;
}
