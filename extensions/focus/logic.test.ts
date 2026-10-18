import { describe, expect, it } from "vitest";
import {
  DURATIONS,
  MINUTE,
  ROUNDS,
  chime,
  endedMessage,
  format,
  initial,
  keyAction,
  next,
  pause,
  progress,
  reset,
  restore,
  running,
  save,
  start,
  tick,
  timeLeft,
  toggle,
  type Timer,
} from "./logic.js";

const T0 = Date.UTC(2026, 9, 18, 9, 0);

/** Runs the phase to its end and takes the next one. */
function finish(timer: Timer, now: number): { timer: Timer; now: number } {
  const started = start(timer, now);
  const end = started.endsAt ?? now;
  return { timer: tick(started, end).timer, now: end };
}

describe("focus timer", () => {
  it("cycles 25 / 5 four times, then a 15-minute break", () => {
    expect(DURATIONS).toEqual({ focus: 25 * MINUTE, short: 5 * MINUTE, long: 15 * MINUTE });
    let timer = initial();
    let now = T0;
    const seen: string[] = [];
    for (let i = 0; i < ROUNDS * 2 + 1; i += 1) {
      seen.push(`${timer.phase}${timer.round}`);
      ({ timer, now } = finish(timer, now));
    }
    expect(seen).toEqual(["focus1", "short1", "focus2", "short2", "focus3", "short3", "focus4", "long4", "focus1"]);
    expect(timer.completed).toBe(5);
    expect(now - T0).toBe(4 * 25 * MINUTE + 3 * 5 * MINUTE + 15 * MINUTE + 25 * MINUTE);
  });

  it("pauses and resumes where it was", () => {
    let timer = start(initial(), T0);
    expect(running(timer)).toBe(true);
    expect(timeLeft(timer, T0 + 10 * MINUTE)).toBe(15 * MINUTE);
    timer = pause(timer, T0 + 10 * MINUTE);
    expect(running(timer)).toBe(false);
    // Paused: time stands still.
    expect(timeLeft(timer, T0 + 60 * MINUTE)).toBe(15 * MINUTE);
    expect(tick(timer, T0 + 60 * MINUTE).ended).toBeNull();
    timer = toggle(timer, T0 + 60 * MINUTE);
    expect(timer.endsAt).toBe(T0 + 75 * MINUTE);
    expect(progress(timer, T0 + 65 * MINUTE)).toBeCloseTo(0.6);
    expect(toggle(timer, T0 + 70 * MINUTE).remaining).toBe(5 * MINUTE);
    expect(start(timer, T0 + 70 * MINUTE)).toBe(timer);
    expect(pause(initial(), T0)).toEqual(initial());
  });

  it("resets the phase, and skipping doesn't count as done", () => {
    const halfway = pause(start(initial(), T0), T0 + 12 * MINUTE);
    expect(reset(halfway)).toEqual({ ...halfway, remaining: DURATIONS.focus });
    const skipped = next(halfway);
    expect(skipped.phase).toBe("short");
    expect(skipped.completed).toBe(0);
    expect(next(skipped)).toMatchObject({ phase: "focus", round: 2 });
  });

  it("ends one phase per tick and never starts the next by itself", () => {
    const timer = start(initial(), T0);
    expect(tick(timer, T0 + 25 * MINUTE - 1).ended).toBeNull();
    const done = tick(timer, T0 + 3 * 60 * MINUTE);
    expect(done.ended).toEqual({ phase: "focus", at: T0 + 25 * MINUTE });
    expect(done.timer).toMatchObject({ phase: "short", endsAt: null, remaining: DURATIONS.short, completed: 1 });
    expect(endedMessage("focus", done.timer)).toMatch(/short break/);
    expect(endedMessage("short", initial())).toMatch(/focus/);
  });

  it("keeps the end time, so a reopened panel shows the time left", () => {
    const timer = start(initial(), T0);
    const saved = JSON.parse(JSON.stringify(save(timer))) as unknown;
    const reopened = restore(saved, T0 + 7 * MINUTE);
    expect(reopened.ended).toBeNull();
    expect(running(reopened.timer)).toBe(true);
    expect(timeLeft(reopened.timer, T0 + 7 * MINUTE)).toBe(18 * MINUTE);
    // Closed past the end: the panel says so, and the break waits to be started.
    const late = restore(saved, T0 + 40 * MINUTE);
    expect(late.ended).toEqual({ phase: "focus", at: T0 + 25 * MINUTE });
    expect(late.timer).toMatchObject({ phase: "short", endsAt: null });
    // A paused timer comes back paused.
    const paused = restore(save(pause(timer, T0 + 5 * MINUTE)), T0 + 99 * MINUTE);
    expect(paused.timer).toMatchObject({ endsAt: null, remaining: 20 * MINUTE });
  });

  it("starts over from anything odd in storage", () => {
    for (const bad of [
      undefined,
      "x",
      { ...initial(), phase: "nap" },
      { ...initial(), round: 9 },
      { ...initial(), remaining: -1 },
      { ...initial(), remaining: DURATIONS.focus + 1 },
      { ...initial(), endsAt: "soon" },
      { ...initial(), endsAt: T0 + 2 * 60 * MINUTE },
      { ...initial(), completed: 1.5 },
    ]) {
      expect(restore(bad, T0)).toEqual({ timer: initial(), ended: null });
    }
  });

  it("shows minutes and seconds, rounded up", () => {
    expect(format(DURATIONS.focus)).toBe("25:00");
    expect(format(DURATIONS.focus - 1)).toBe("25:00");
    expect(format(DURATIONS.focus - 1000)).toBe("24:59");
    expect(format(61_000)).toBe("1:01");
    expect(format(0)).toBe("0:00");
    expect(format(-5)).toBe("0:00");
    expect(format(65 * MINUTE)).toBe("1:05:00");
  });

  it("chimes differently after focus and after a break", () => {
    expect(chime("focus").map((n) => n.frequency)).toEqual([659.25, 783.99, 987.77]);
    expect(chime("short")).toHaveLength(2);
    for (const note of [...chime("focus"), ...chime("long")]) {
      expect(note.length).toBeGreaterThan(0);
      expect(note.at).toBeGreaterThanOrEqual(0);
    }
  });

  it("knows its keys", () => {
    expect(keyAction({ key: " " })).toBe("toggle");
    expect(keyAction({ key: "r", code: "KeyR" })).toBe("reset");
    expect(keyAction({ key: "s", code: "KeyS" })).toBe("skip");
    expect(keyAction({ key: "Escape" })).toBe("escape");
    expect(keyAction({ key: "r", code: "KeyR", metaKey: true })).toBeNull();
    expect(keyAction({ key: "x", code: "KeyX" })).toBeNull();
  });
});
