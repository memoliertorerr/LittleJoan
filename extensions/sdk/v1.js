// @ts-check
// The Little Joan extension SDK, version 1 (experimental). Little Joan serves this file to every
// extension panel at /__lj/sdk/v1.js; load it with <script src="/__lj/sdk/v1.js"></script> and use
// the global `littleJoan`. Each call is a POST to the panel's own origin (/__lj/api/v1/<method>),
// which Little Joan checks before it answers (docs/extensions.md, "Security model").
(() => {
  const API = "/__lj/api/v1/";

  /** An API call Little Joan refused or couldn't do; `code` says why ("permission", "quota", …). */
  class LittleJoanError extends Error {
    /**
     * @param {string} message
     * @param {string} code
     */
    constructor(message, code) {
      super(message);
      this.name = "LittleJoanError";
      /** @type {string} */
      this.code = code;
    }
  }

  /**
   * @param {string} method
   * @param {Record<string, unknown>} [params]
   * @returns {Promise<unknown>}
   */
  async function call(method, params = {}) {
    const response = await fetch(API + method, {
      method: "POST",
      headers: { "Content-Type": "application/json", "X-LJ-Api": "1" },
      body: JSON.stringify(params),
      cache: "no-store",
      credentials: "same-origin",
      // Little Joan checks the Origin of every call; under a "no-referrer" policy (a panel's own
      // <meta name="referrer">) engines would send "null" instead of the panel's origin.
      referrerPolicy: "same-origin",
    });
    /** @type {unknown} */
    let raw;
    try {
      raw = await response.json();
    } catch {
      raw = null;
    }
    /** @typedef {{ ok?: unknown, value?: unknown, error?: unknown, message?: unknown }} Answer */
    const answer = typeof raw === "object" && raw !== null ? /** @type {Answer} */ (raw) : null;
    if (!response.ok || answer?.ok !== true) {
      const message = typeof answer?.message === "string" ? answer.message : `Little Joan refused ${method} (${response.status}).`;
      const code = typeof answer?.error === "string" ? answer.error : "failed";
      throw new LittleJoanError(message, code);
    }
    return answer.value;
  }

  /** @typedef {{ scheme: "light" | "dark", vars: Record<string, string> }} Theme */

  const littleJoan = Object.freeze({
    apiVersion: 1,
    LittleJoanError,
    storage: Object.freeze({
      /** @param {string} key */
      get: (key) => call("storage.get", { key }),
      /**
       * @param {string} key
       * @param {unknown} value any JSON value
       */
      set: async (key, value) => {
        await call("storage.set", { key, value });
      },
      /** @param {string} key */
      remove: async (key) => (await call("storage.remove", { key })) === true,
      keys: async () => /** @type {string[]} */ (await call("storage.keys")),
    }),
    theme: Object.freeze({
      /** The bar's current colors: `scheme` and the `--lj-*` CSS variables. */
      get: async () => /** @type {Theme} */ (await call("theme")),
      /**
       * Puts the current colors on `root` (default: the <html> element) as CSS variables and sets
       * `color-scheme`; returns them.
       * @param {HTMLElement} [root]
       */
      apply: async (root = document.documentElement) => {
        const theme = /** @type {Theme} */ (await call("theme"));
        for (const [name, value] of Object.entries(theme.vars)) {
          root.style.setProperty(name, value);
        }
        root.style.setProperty("color-scheme", theme.scheme);
        return theme;
      },
    }),
    /** The web page a page action was opened for (`pageInfo` permission), or null. */
    pageInfo: async () => /** @type {{ url: string, title: string } | null} */ (await call("pageInfo")),
    /**
     * Opens an http(s) address in the page (replacing the panel) or in the default browser.
     * @param {string} url
     * @param {{ where?: "page" | "browser" }} [options]
     */
    openUrl: async (url, options = {}) => {
      await call("openUrl", { url: String(url), where: options.where ?? "page" });
    },
    clipboard: Object.freeze({
      /** @param {string} text */
      writeText: async (text) => {
        await call("clipboard.writeText", { text: String(text) });
      },
    }),
    /** Closes the panel: back to the page it was opened from, or to the command bar. */
    close: async () => {
      await call("close");
    },
  });

  Object.defineProperty(globalThis, "littleJoan", { value: littleJoan, enumerable: true });
})();
