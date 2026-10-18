// @ts-check
// The focus timer's rules, pure: 25-minute focus sessions with 5-minute breaks and a 15-minute
// break after every fourth, pause and resume, and what storage keeps — the END TIME of a running
// session, so a reopened panel shows the time left. Nothing runs while the panel is closed (Little
// Joan hibernates); a session that ended meanwhile is reported when the panel opens again. The
// clock is passed in (`now`, unix ms), so logic.test.ts can move it.

/** @typedef {"focus" | "short" | "long"} Phase */
/**
 * @typedef {object} Timer
 * @property {Phase} phase
 * @property {number} round 1–`ROUNDS`: which focus session of the cycle (a break belongs to the one before it)
 * @property {number | null} endsAt when the running phase ends (unix ms); null when it isn't running
 * @property {number} remaining ms left when it isn't running
 * @property {number} completed focus sessions finished, ever
 */
/** @typedef {"toggle" | "reset" | "skip" | "escape"} Action */

export const MINUTE = 60_000;
/** @type {Record<Phase, number>} */
export const DURATIONS = { focus: 25 * MINUTE, short: 5 * MINUTE, long: 15 * MINUTE };
/** Focus sessions per cycle; the long break comes after the last. */
export const ROUNDS = 4;
/** @type {Record<Phase, string>} */
export const LABELS = { focus: "Focus", short: "Short break", long: "Long break" };

/** @returns {Timer} */
export function initial() {
  return { phase: "focus", round: 1, endsAt: null, remaining: DURATIONS.focus, completed: 0 };
}

/** @param {Timer} timer */
export const running = (timer) => timer.endsAt !== null;

/**
 * Milliseconds left at `now`.
 * @param {Timer} timer
 * @param {number} now
 */
export function timeLeft(timer, now) {
  return timer.endsAt === null ? timer.remaining : Math.max(0, timer.endsAt - now);
}

/**
 * How much of the phase has passed, 0–1.
 * @param {Timer} timer
 * @param {number} now
 */
export function progress(timer, now) {
  return 1 - timeLeft(timer, now) / DURATIONS[timer.phase];
}

/**
 * Start (or resume) the phase: it ends `remaining` from now.
 * @param {Timer} timer
 * @param {number} now
 * @returns {Timer}
 */
export function start(timer, now) {
  return running(timer) ? timer : { ...timer, endsAt: now + timer.remaining };
}

/**
 * Pause: keep what is left.
 * @param {Timer} timer
 * @param {number} now
 * @returns {Timer}
 */
export function pause(timer, now) {
  return running(timer) ? { ...timer, endsAt: null, remaining: timeLeft(timer, now) } : timer;
}

/**
 * Space: start or pause.
 * @param {Timer} timer
 * @param {number} now
 */
export function toggle(timer, now) {
  return running(timer) ? pause(timer, now) : start(timer, now);
}

/**
 * The phase from the start again (stopped).
 * @param {Timer} timer
 * @returns {Timer}
 */
export function reset(timer) {
  return { ...timer, endsAt: null, remaining: DURATIONS[timer.phase] };
}

/**
 * The next phase, ready to start: a focus session is followed by a short break — or by the long
 * one after the last round —, a break by the next focus session. A focus session that ends counts
 * as completed (skipping one doesn't).
 * @param {Timer} timer
 * @param {boolean} [finished] the phase ran out (rather than being skipped)
 * @returns {Timer}
 */
export function next(timer, finished = false) {
  const completed = timer.completed + (finished && timer.phase === "focus" ? 1 : 0);
  if (timer.phase === "focus") {
    const phase = timer.round >= ROUNDS ? "long" : "short";
    return { phase, round: timer.round, endsAt: null, remaining: DURATIONS[phase], completed };
  }
  const round = timer.phase === "long" ? 1 : timer.round + 1;
  return { phase: "focus", round, endsAt: null, remaining: DURATIONS.focus, completed };
}

/**
 * What happened by `now`: if the running phase ran out, the next phase (ready, not running) and
 * which phase ended when. Only one phase ends per call — nothing runs on by itself.
 * @param {Timer} timer
 * @param {number} now
 * @returns {{ timer: Timer, ended: { phase: Phase, at: number } | null }}
 */
export function tick(timer, now) {
  if (timer.endsAt === null || now < timer.endsAt) {
    return { timer, ended: null };
  }
  return { timer: next(timer, true), ended: { phase: timer.phase, at: timer.endsAt } };
}

/**
 * "24:59" — minutes and seconds left, rounded up (a phase shows its full length until a second
 * has passed); "1:05:00" past an hour.
 * @param {number} ms
 */
export function format(ms) {
  const total = Math.max(0, Math.ceil(ms / 1000));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = String(total % 60).padStart(2, "0");
  return hours > 0 ? `${hours}:${String(minutes).padStart(2, "0")}:${seconds}` : `${minutes}:${seconds}`;
}

/**
 * What the panel says when a phase ended (the chime plays only while the panel is open).
 * @param {Phase} ended
 * @param {Timer} now the phase that follows
 */
export function endedMessage(ended, now) {
  if (ended === "focus") {
    return now.phase === "long" ? "Focus session done — time for a long break." : "Focus session done — time for a short break.";
  }
  return "Break's over — ready to focus?";
}

/**
 * What storage keeps.
 * @param {Timer} timer
 */
export function save(timer) {
  return { ...timer };
}

/**
 * A saved timer, checked from scratch (anything odd starts over), brought up to `now`: if its
 * phase ran out while the panel was closed, `ended` says which and when.
 * @param {unknown} saved
 * @param {number} now
 * @returns {{ timer: Timer, ended: { phase: Phase, at: number } | null }}
 */
export function restore(saved, now) {
  if (typeof saved !== "object" || saved === null) {
    return { timer: initial(), ended: null };
  }
  const value = /** @type {Record<string, unknown>} */ (saved);
  /** @type {Phase | null} */
  const phase = value.phase === "focus" || value.phase === "short" || value.phase === "long" ? value.phase : null;
  const { round, endsAt, remaining, completed } = value;
  const whole = (/** @type {unknown} */ n, /** @type {number} */ min, /** @type {number} */ max) =>
    typeof n === "number" && Number.isInteger(n) && n >= min && n <= max;
  if (
    !phase ||
    !whole(round, 1, ROUNDS) ||
    !whole(remaining, 0, DURATIONS[phase]) ||
    !whole(completed, 0, Number.MAX_SAFE_INTEGER) ||
    !(endsAt === null || (typeof endsAt === "number" && Number.isFinite(endsAt) && endsAt <= now + DURATIONS[phase]))
  ) {
    return { timer: initial(), ended: null };
  }
  /** @type {Timer} */
  const timer = { phase, round: /** @type {number} */ (round), endsAt, remaining: /** @type {number} */ (remaining), completed: /** @type {number} */ (completed) };
  return tick(timer, now);
}

/**
 * The notes of the chime (Web Audio): a rising triad after focus, a falling pair after a break.
 * Each is a sine at `frequency` Hz from `at` seconds for `length` seconds.
 * @param {Phase} ended
 * @returns {{ frequency: number, at: number, length: number }[]}
 */
export function chime(ended) {
  return ended === "focus"
    ? [
        { frequency: 659.25, at: 0, length: 0.9 },
        { frequency: 783.99, at: 0.16, length: 0.9 },
        { frequency: 987.77, at: 0.32, length: 1.2 },
      ]
    : [
        { frequency: 880, at: 0, length: 0.8 },
        { frequency: 659.25, at: 0.2, length: 1.1 },
      ];
}

/**
 * What a key press means: Space start/pause, R reset, S skip to the next phase, Escape leave.
 * Presses with Ctrl, Alt or Cmd belong to Little Joan and the system.
 * @param {{ key: string, code?: string, ctrlKey?: boolean, altKey?: boolean, metaKey?: boolean }} event
 * @returns {Action | null}
 */
export function keyAction(event) {
  if (event.ctrlKey || event.altKey || event.metaKey) {
    return null;
  }
  if (event.key === " " || event.key === "Spacebar") {
    return "toggle";
  }
  if (event.key === "Escape") {
    return "escape";
  }
  /** @type {Record<string, Action>} */
  const codes = { KeyR: "reset", KeyS: "skip" };
  return (event.code ? codes[event.code] : undefined) ?? null;
}
