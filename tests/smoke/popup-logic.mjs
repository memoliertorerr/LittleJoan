// @ts-check
// Logic of the sign-in popups smoke (LJ47–LJ48): the opener (smoke.html?popup=oauth|blank&at=…)
// and the sign-in page (signin.html), which the smoke server serves on a second port — another
// origin, like a real provider. Plain JavaScript so the pages load it as is; type-checked through
// JSDoc and unit-tested by popup-logic.test.ts. The pages report through document.title (the
// opener's reaches the smoke log; a popup's title is never the page's, so signin.html reports
// nothing there).

/** The opener's title once its "Sign in" button is up. */
export const POPUP_READY_TITLE = "LJ-POPUP ready";
/** What the sign-in page posts to its opener. */
export const SIGNED_IN = "lj-signed-in";
/** The popup's window name and features, as sign-in libraries open one. */
export const POPUP_NAME = "lj-signin";
export const POPUP_FEATURES = "width=480,height=600";
/** How long the sign-in page waits before it posts back and closes (the smoke's screenshot). */
export const SIGNIN_DELAY_MS = 2500;
/**
 * The spoof's target (phase 22's security review): a path on the sign-in origin that the smoke
 * server never answers in time, so nothing of that origin can ever commit in the popup.
 */
export const HANG_PATH = "hang";
/** How long the spoof's popup stays before the opener closes it (the smoke reads its title). */
export const SPOOF_HOLD_MS = 3000;
/** The opener's report once its blur / visibilitychange popup is armed (`?popup=onblur`). */
export const ONBLUR_ARMED_TITLE = "LJ-POPUP armed";

/** @typedef {"oauth" | "blank" | "spoof" | "onblur"} PopupMode */

/** @type {readonly PopupMode[]} */
export const POPUP_MODES = ["oauth", "blank", "spoof", "onblur"];

/**
 * `?popup=oauth|blank|spoof|onblur&at=<sign-in origin>[&stay=1]`: `oauth` opens the sign-in page
 * in a sized popup, `blank` opens a sized `about:blank` popup and then sets its location; `stay`
 * keeps the sign-in page open (it posts, but doesn't close). `spoof` (the security review) opens a
 * sized popup at the sign-in origin's `HANG_PATH`, stops it and writes a fake form into it; `onblur`
 * opens a sized popup when the page loses focus or is hidden (the window closing). `at` must be
 * `http://127.0.0.1:<port>`.
 * @param {string} search location.search
 * @returns {{ mode: PopupMode, at: string, stay: boolean } | undefined}
 */
export function popupRequest(search) {
  const params = new URLSearchParams(search);
  const mode = POPUP_MODES.find((known) => known === params.get("popup"));
  const at = params.get("at") ?? "";
  if (mode && /^http:\/\/127\.0\.0\.1:\d{1,5}$/.test(at)) {
    return { mode, at, stay: params.get("stay") === "1" };
  }
  return undefined;
}

/**
 * The spoof's target: the sign-in origin's path that never answers in time.
 * @param {string} at the sign-in origin
 */
export function hangUrl(at) {
  return `${at}/${HANG_PATH}`;
}

/**
 * What the spoof writes into its stopped popup: a fake sign-in form (inert — no action, nothing
 * sent).
 */
export const SPOOF_HTML =
  '<!doctype html><title>Sign in</title><body style="font:20px system-ui;background:#b71c1c;color:#fff;display:flex;align-items:center;justify-content:center;height:100vh;margin:0">' +
  "<form onsubmit=\"return false\"><p>Fake sign-in (spoof test)</p><input type=password placeholder=Password></form></body>";

/**
 * The opener's report after the spoof: whether window.open gave a window and the write went in.
 * @param {boolean} opened
 * @param {boolean} written
 */
export function spoofTitle(opened, written) {
  return `LJ-POPUP spoofed opened=${opened ? 1 : 0} written=${written ? 1 : 0}`;
}

/**
 * The sign-in page's address.
 * @param {string} at the sign-in origin
 * @param {boolean} stay
 */
export function signinUrl(at, stay) {
  return `${at}/signin.html${stay ? "?stay=1" : ""}`;
}

/**
 * Whether a message is the sign-in page's: exactly SIGNED_IN, from the sign-in origin.
 * @param {unknown} data
 * @param {string} origin the message's origin
 * @param {string} at the sign-in origin
 */
export function isSignedIn(data, origin, at) {
  return data === SIGNED_IN && origin === at;
}

/**
 * The opener's report: how many sign-in messages arrived, and whether its popup is closed.
 * @param {number} received
 * @param {boolean} closed
 */
export function popupTitle(received, closed) {
  return `LJ-POPUP received=${received} closed=${closed ? 1 : 0}`;
}

/**
 * The opener's report right after the click: whether window.open gave it a window.
 * @param {boolean} opened
 */
export function openedTitle(opened) {
  return `LJ-POPUP opened=${opened ? 1 : 0}`;
}

/**
 * signin.html's own request: `?stay=1` keeps it open.
 * @param {string} search location.search
 */
export function signinStays(search) {
  return new URLSearchParams(search).get("stay") === "1";
}
