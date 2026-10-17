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
