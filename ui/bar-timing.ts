// The bar's part of a summon's timing (LJ19.1): when its first frame was painted, and when the
// command bar's first rows were — only the bar can see either. Rust asks for them only in smoke
// tests (start_info's `timing`); then each goes out once, as "this long ago" (report_timing),
// which Rust places on its own clock after the summon's request, so the two clocks never need to
// agree. Marks seen before start_info answered wait for it; without `timing` nothing is sent.
// Pure, unit-tested; bar.ts does the DOM.

export type TimingMark = "painted" | "commandBar";

export interface TimingReporter {
  /** `mark` happened at `at` (the same clock as `now`). Only the first time of each counts. */
  saw(mark: TimingMark, at: number): void;
  /** start_info's answer: whether to report. What was seen goes out now, the rest when seen. */
  enable(on: boolean): void;
}

/** `send(mark, agoMs)` reports one mark; `now` is the clock `saw` was given times on. */
export function timingReporter(send: (mark: TimingMark, agoMs: number) => void, now: () => number): TimingReporter {
  const seen = new Map<TimingMark, number>();
  const sent = new Set<TimingMark>();
  let on: boolean | null = null;

  const flush = (): void => {
    if (on !== true) {
      return;
    }
    for (const [mark, at] of seen) {
      if (!sent.has(mark)) {
        sent.add(mark);
        send(mark, Math.max(0, now() - at));
      }
    }
  };

  return {
    saw(mark, at) {
      if (!seen.has(mark)) {
        seen.set(mark, at);
        flush();
      }
    },
    enable(value) {
      if (on === null) {
        on = value;
        flush();
      }
    },
  };
}

/** Runs `then` once the next frame has been painted (a frame callback, then a task after it). */
export function afterPaint(then: () => void, frame: (callback: () => void) => void, task: (callback: () => void) => void): void {
  frame(() => task(then));
}
