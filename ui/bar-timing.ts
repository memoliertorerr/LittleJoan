// The bar's part of a summon's timing (LJ19.1) — what only the bar can see: when its navigation
// started and its HTML arrived (the engine's navigation timing), when this script ran, when
// set_theme_tokens, start_info and the first suggestions answered, when its first frame was
// painted, when the command bar's first rows were, and (LJ19.4) when its field had the keyboard.
// Rust asks for it only in smoke tests (start_info's `timing`, or a summon from ready's own); then
// it all goes out once per summon, in one call — once the rows are painted and the field has the
// keyboard, or right after the first frame when the window opened on a page (no rows) — as "this
// long ago" per mark (report_timing), which Rust places on its own clock after the summon's
// request, so the two clocks never need to agree and the report itself adds no call while the bar
// starts. A summon from ready gets a reporter of its own. Without `timing` nothing is sent. Pure,
// unit-tested; bar.ts does the DOM.

export type TimingMark =
  | "navigation"
  | "html"
  | "module"
  | "themeAnswered"
  | "startInfo"
  | "suggestions"
  | "painted"
  | "focused"
  | "commandBar";

/** The marks a report waits for (whichever comes last sends it), unless it is flushed. */
export const FINAL_MARKS: readonly TimingMark[] = ["commandBar", "focused"];

export interface TimingReporter {
  /** `mark` happened at `at` (the same clock as `now`). Only the first time of each counts. */
  saw(mark: TimingMark, at: number): void;
  /** start_info's answer: whether to report. The first answer counts. */
  enable(on: boolean): void;
  /** Sends what was seen now (once), if reporting is on: the window opened on a page, say. */
  flush(): void;
}

/** `send(marks)` reports each mark as milliseconds ago; `now` is the clock `saw` was given. */
export function timingReporter(send: (marks: Partial<Record<TimingMark, number>>) => void, now: () => number): TimingReporter {
  const seen = new Map<TimingMark, number>();
  let on: boolean | null = null;
  let sent = false;
  let wanted = false;

  const go = (): void => {
    if (on !== true || sent) {
      return;
    }
    sent = true;
    const at = now();
    const marks: Partial<Record<TimingMark, number>> = {};
    for (const [mark, when] of seen) {
      marks[mark] = Math.max(0, at - when);
    }
    send(marks);
  };

  const finished = (): boolean => FINAL_MARKS.every((mark) => seen.has(mark));

  return {
    saw(mark, at) {
      if (sent || seen.has(mark)) {
        return;
      }
      seen.set(mark, at);
      if (finished() || wanted) {
        go();
      }
    },
    enable(value) {
      if (on === null) {
        on = value;
        if (wanted || finished()) {
          go();
        }
      }
    },
    flush() {
      wanted = true;
      go();
    },
  };
}

/** Runs `then` once the next frame has been painted (a frame callback, then a task after it). */
export function afterPaint(then: () => void, frame: (callback: () => void) => void, task: (callback: () => void) => void): void {
  frame(() => task(then));
}
