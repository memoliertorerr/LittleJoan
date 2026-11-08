// @ts-check
// The smoke page's `?webauthn=1` (LJ52): what the page's web engine offers for passkeys, as a page
// can ask without showing anything — whether `PublicKeyCredential` exists, whether a platform
// authenticator that verifies the user is available (`isUserVerifyingPlatformAuthenticatorAvailable`:
// Windows Hello, Touch ID…), and whether passkey autofill is (`isConditionalMediationAvailable`). It
// never creates or asks for a credential, so no prompt shows. Evidence for the plan, not a rule:
// smoke-logins.mts puts it into summary.json on every OS. The page reports through document.title,
// which the app writes to its smoke log — no IPC. Plain JavaScript, type-checked through JSDoc and
// unit-tested by webauthn-logic.test.ts.

export const WEBAUTHN_PREFIX = "LJ-WEBAUTHN";

/** How long an availability question may take before it counts as an error. */
export const ASK_TIMEOUT_MS = 5000;

/**
 * Whether the page was asked for the report (`?webauthn=1`).
 * @param {string} search location.search
 */
export function webauthnRequest(search) {
  return new URLSearchParams(search).get("webauthn") === "1";
}

/** @typedef {boolean | "error"} Answer */

/**
 * @typedef {object} WebauthnReport
 * @property {boolean} api `PublicKeyCredential` exists
 * @property {Answer} platform a user-verifying platform authenticator is available
 * @property {Answer} conditional passkey autofill (conditional mediation) is available
 * @property {boolean} secure the page is a secure context (WebAuthn needs one; 127.0.0.1 is)
 */

/**
 * The report as the page's title: `LJ-WEBAUTHN api=<0|1> platform=<true|false|error>
 * conditional=<true|false|error> secure=<true|false>`.
 * @param {WebauthnReport} report
 */
export function webauthnTitle(report) {
  return `${WEBAUTHN_PREFIX} api=${report.api ? 1 : 0} platform=${report.platform} conditional=${report.conditional} secure=${report.secure}`;
}

/**
 * The report back from a title (`null` for anything else).
 * @param {string | undefined} title
 * @returns {WebauthnReport | null}
 */
export function parseWebauthnTitle(title) {
  const match = /^LJ-WEBAUTHN api=([01]) platform=(true|false|error) conditional=(true|false|error)(?: secure=(true|false))?$/.exec(title ?? "");
  if (!match) {
    return null;
  }
  /** @param {string | undefined} word @returns {Answer} */
  const answer = (word) => (word === "error" ? "error" : word === "true");
  return { api: match[1] === "1", platform: answer(match[2]), conditional: answer(match[3]), secure: match[4] === "true" };
}

/**
 * Asks one of `PublicKeyCredential`'s static availability questions: `false` when the engine
 * doesn't have it, `"error"` when it throws, rejects, answers something else or takes too long.
 * @param {unknown} credential the `PublicKeyCredential` constructor
 * @param {string} name the static method's name
 * @param {number} timeoutMs
 * @returns {Promise<Answer>}
 */
async function ask(credential, name, timeoutMs) {
  const method = /** @type {Record<string, unknown>} */ (credential)[name];
  if (typeof method !== "function") {
    return false;
  }
  /** @type {ReturnType<typeof globalThis.setTimeout> | undefined} */
  let timer;
  /** @type {Promise<unknown>} */
  const late = new Promise((done) => {
    timer = globalThis.setTimeout(() => done("late"), timeoutMs);
  });
  try {
    const asked = /** @type {unknown} */ (method.call(credential));
    const answer = await Promise.race([Promise.resolve(asked), late]);
    return typeof answer === "boolean" ? answer : "error";
  } catch {
    return "error";
  } finally {
    globalThis.clearTimeout(timer);
  }
}

/**
 * What the engine offers, asked of `win` (the page's window).
 * @param {{ PublicKeyCredential?: unknown, isSecureContext?: boolean }} win
 * @param {number} [timeoutMs]
 * @returns {Promise<WebauthnReport>}
 */
export async function webauthnReport(win, timeoutMs = ASK_TIMEOUT_MS) {
  const credential = win.PublicKeyCredential;
  const secure = win.isSecureContext === true;
  if (typeof credential !== "function") {
    return { api: false, platform: false, conditional: false, secure };
  }
  const [platform, conditional] = await Promise.all([
    ask(credential, "isUserVerifyingPlatformAuthenticatorAvailable", timeoutMs),
    ask(credential, "isConditionalMediationAvailable", timeoutMs),
  ]);
  return { api: true, platform, conditional, secure };
}
