// @ts-check
// Logic of the smoke page's fullscreen run (smoke.html?fs=both, LJ36): where its buttons sit,
// which route the "Phone player" takes, and what one attempt saw. Plain JavaScript, JSDoc-typed and
// unit-tested (fullscreen-logic.test.ts), like smoke-logic.mjs. The page reports through
// document.title, which the app writes to its smoke log — no IPC.

/** @typedef {{ x: number, y: number, width: number, height: number }} Box  fractions of the viewport */
/** @typedef {"element" | "phone"} FsPath */
/** @typedef {"ios" | "element"} PlayerRoute */

/**
 * Where the buttons sit, as fractions of the viewport, so the smoke test can click each one.
 * In the window "Element fullscreen" is on top and "Phone player" below, with a wide gap around
 * the middle; in fullscreen (the viewport is then the screen) the exit button is a big target in
 * the middle, so a click at the screen's centre reaches it.
 */
export const FS_BUTTONS = /** @type {const} */ ({
  element: { x: 0.06, y: 0.05, width: 0.88, height: 0.33 },
  phone: { x: 0.06, y: 0.62, width: 0.88, height: 0.33 },
  exit: { x: 0.25, y: 0.3, width: 0.5, height: 0.4 },
});

/**
 * The middle of a box.
 * @param {Box} box
 * @returns {{ x: number, y: number }}
 */
export function centre(box) {
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

/** How long an attempt waits for fullscreen to start after its click. */
export const ENTER_WAIT_MS = 4000;
/** How long an attempt stays in fullscreen before it gives up waiting for its exit click. */
export const HOLD_LIMIT_MS = 6000;
/** How long the iOS route waits for `webkitbeginfullscreen` before trying the orientation lock. */
export const BEGIN_WAIT_MS = 1000;

/**
 * Whether the page was asked for the fullscreen run: `?fs=both`.
 * @param {string} search location.search
 */
export function fsRequest(search) {
  return new URLSearchParams(search).get("fs") === "both";
}

/**
 * The route the "Phone player" takes, like m.youtube.com's player: under an iPhone agent the iOS
 * way (`video.webkitEnterFullscreen()`, then the iOS events and an orientation lock), otherwise
 * element fullscreen of the player's container.
 * @param {string} userAgent navigator.userAgent
 * @returns {PlayerRoute}
 */
export function playerPath(userAgent) {
  return /\biPhone\b/.test(userAgent) ? "ios" : "element";
}

/**
 * The kind of user agent, as the page reports it: "phone" (a "Mobile" token) or "desktop".
 * @param {string} userAgent
 */
export function agentKind(userAgent) {
  return /\bMobile\b/.test(userAgent) ? "phone" : "desktop";
}

/**
 * The title the page shows while it waits for a click at this size.
 * @param {number} width window.innerWidth
 * @param {string} userAgent
 */
export function fsReadyTitle(width, userAgent) {
  return `LJ-FS ready width=${Math.round(width)} ua=${agentKind(userAgent)}`;
}

/**
 * The title for a "Phone player" that can't play: no `captureStream`, or the video never got its
 * metadata.
 * @param {number} width
 * @param {string} userAgent
 * @param {string} reason
 */
export function unavailableTitle(width, userAgent, reason) {
  return `LJ-FS phone width=${Math.round(width)} ua=${agentKind(userAgent)} unavailable=${clean(reason)}`;
}

/** @param {string} name */
function clean(name) {
  return name.toLowerCase().replace(/[^a-z0-9:-]+/g, "-").slice(0, 40) || "-";
}

/**
 * One fullscreen attempt: a click on a button, then what the page saw until fullscreen ended (or
 * the time ran out). Times are milliseconds from one clock (`performance.now()` in the page).
 */
export class FsAttempt {
  /**
   * @param {FsPath} path
   * @param {number} now when the click arrived
   */
  constructor(path, now) {
    /** @type {FsPath} */
    this.path = path;
    this.clickedAt = now;
    /** @type {number | undefined} */
    this.enteredAt = undefined;
    /** @type {number | undefined} */
    this.leftAt = undefined;
    /** @type {number | undefined} */
    this.exitAskedAt = undefined;
    /** @type {number | undefined} */
    this.endedAt = undefined;
    /** @type {string[]} */
    this.events = ["click"];
  }

  /**
   * Records something that happened (an event name, a call), once in a row.
   * @param {string} name
   */
  note(name) {
    const event = clean(name);
    if (this.events.at(-1) !== event && this.events.length < 40) {
      this.events.push(event);
    }
  }

  /**
   * The page's fullscreen state as an event or a poll saw it. Returns true when this ended the
   * attempt (fullscreen left after it had been entered).
   * @param {boolean} on
   * @param {number} now
   * @param {string} cause what saw it ("fullscreenchange", "poll", …)
   */
  observe(on, now, cause) {
    if (this.endedAt !== undefined) {
      return false;
    }
    if (on && this.enteredAt === undefined) {
      this.enteredAt = now;
      this.note(`on:${cause}`);
    } else if (!on && this.enteredAt !== undefined && this.leftAt === undefined) {
      this.leftAt = now;
      this.note(`off:${cause}`);
      this.endedAt = now;
      return true;
    }
    return false;
  }

  /**
   * The page's own exit was clicked.
   * @param {number} now
   */
  askExit(now) {
    if (this.exitAskedAt === undefined && this.endedAt === undefined) {
      this.exitAskedAt = now;
      this.note("exit");
    }
  }

  /**
   * Ends the attempt when its time ran out: fullscreen never started within ENTER_WAIT_MS of the
   * click, or it was still on HOLD_LIMIT_MS after it started. Returns true when this ended it.
   * @param {number} now
   */
  expire(now) {
    if (this.endedAt !== undefined) {
      return false;
    }
    const neverEntered = this.enteredAt === undefined && now - this.clickedAt >= ENTER_WAIT_MS;
    const heldTooLong = this.enteredAt !== undefined && now - this.enteredAt >= HOLD_LIMIT_MS;
    if (neverEntered || heldTooLong) {
      this.note("timeout");
      this.endedAt = now;
      return true;
    }
    return false;
  }

  /** Whether the attempt is over (reported). */
  get done() {
    return this.endedAt !== undefined;
  }

  /**
   * The report: `LJ-FS <element|phone> width=<w> ua=<phone|desktop> entered=<0|1>
   * held_ms=<n> exited_by_page=<0|1> events=<list>`. `exited_by_page` is 1 when fullscreen ended
   * after the page's own exit was clicked; `held_ms` is how long it lasted (until the attempt
   * ended, when it never left).
   * @param {number} width
   * @param {string} userAgent
   */
  report(width, userAgent) {
    const end = this.leftAt ?? this.endedAt ?? this.clickedAt;
    const entered = this.enteredAt !== undefined;
    const held = entered ? Math.max(0, Math.round(end - (this.enteredAt ?? end))) : 0;
    const byPage = this.leftAt !== undefined && this.exitAskedAt !== undefined && this.exitAskedAt <= this.leftAt;
    return (
      `LJ-FS ${this.path} width=${Math.round(width)} ua=${agentKind(userAgent)} entered=${entered ? 1 : 0} ` +
      `held_ms=${held} exited_by_page=${byPage ? 1 : 0} events=${this.events.join(",")}`
    );
  }
}
