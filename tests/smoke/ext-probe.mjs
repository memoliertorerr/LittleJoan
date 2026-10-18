// @ts-check
// The smoke page's extension probe (smoke.html?ext=probe&base=<an extension's origin>): a web
// page must not be able to call an extension's API, load its files, or navigate the page webview
// to it. Plain JavaScript like smoke-logic.mjs, unit-tested by ext-probe.test.ts. The page reports
// through its title, `LJ-EXTPROBE ljext-api=… ljext-file=… ljext-nav=…`.

/** An extension origin in either platform's form, nothing else. */
const ORIGIN = /^(ljext:\/\/[a-z0-9][a-z0-9-]{1,31}\.localhost|http:\/\/ljext\.[a-z0-9][a-z0-9-]{1,31}\.localhost)$/;

/**
 * The race a page URL asks for: `?ext=race&base=<origin>` — the page keeps putting the panel's
 * URL (without the app's nonce) into iframes and window.open while the app opens the extension.
 * @param {string} search location.search
 * @returns {{ base: string } | undefined}
 */
export function raceRequest(search) {
  const params = new URLSearchParams(search);
  const base = params.get("base") ?? "";
  return params.get("ext") === "race" && ORIGIN.test(base) ? { base } : undefined;
}

/** How long the race page keeps trying, and how often. */
export const RACE = { durationMs: 12_000, everyMs: 25 };

/**
 * The probe a page URL asks for: `?ext=probe&base=<origin>`.
 * @param {string} search location.search
 * @returns {{ base: string } | undefined}
 */
export function probeRequest(search) {
  const params = new URLSearchParams(search);
  const base = params.get("base") ?? "";
  return params.get("ext") === "probe" && ORIGIN.test(base) ? { base } : undefined;
}

/** @typedef {(url: string, init?: RequestInit) => Promise<{ ok: boolean }>} FetchLike */

/**
 * Tries the extension's API the way its own SDK would (POST, `X-LJ-Api: 1`, JSON), and as a
 * "simple" request a web page can send without a CORS preflight. "blocked" when neither got an
 * answer the page could read as a success.
 * @param {FetchLike} fetchFn
 * @param {string} base
 */
export async function probeApi(fetchFn, base) {
  /** @param {RequestInit} init */
  const attempt = async (init) => {
    try {
      const response = await fetchFn(`${base}/__lj/api/v1/storage.set`, init);
      return response.ok ? "open" : "blocked";
    } catch {
      return "blocked";
    }
  };
  const body = JSON.stringify({ key: "smoke-probe", value: 1 });
  const results = [
    await attempt({ method: "POST", headers: { "Content-Type": "application/json", "X-LJ-Api": "1" }, body }),
    await attempt({ method: "POST", headers: { "Content-Type": "text/plain" }, body }),
  ];
  return results.every((result) => result === "blocked") ? "blocked" : "open";
}

/**
 * Tries to show one of the extension's files as an image (images load across origins without
 * CORS): "blocked" when it didn't load.
 * @param {(src: string) => Promise<boolean>} loadImage resolves whether the image loaded
 * @param {string} base
 */
export async function probeFile(loadImage, base) {
  return (await loadImage(`${base}/icon.svg`)) ? "open" : "blocked";
}

/**
 * The title the probe reports.
 * @param {{ api: string, file: string, nav: string }} results
 */
export function probeTitle(results) {
  return `LJ-EXTPROBE ljext-api=${results.api} ljext-file=${results.file} ljext-nav=${results.nav}`;
}
