import { describe, expect, it } from "vitest";
import { afterPaint, timingReporter, type TimingMark } from "./bar-timing.ts";

function harness() {
  let clock = 0;
  const sent: [TimingMark, number][] = [];
  const reporter = timingReporter(
    (mark, ago) => sent.push([mark, ago]),
    () => clock,
  );
  return {
    reporter,
    sent,
    at(ms: number) {
      clock = ms;
    },
  };
}

describe("the bar's summon timing", () => {
  it("waits for start_info, then reports what it saw as how long ago", () => {
    const h = harness();
    h.at(40);
    h.reporter.saw("painted", 40);
    expect(h.sent).toEqual([]);
    h.at(55);
    h.reporter.enable(true);
    expect(h.sent).toEqual([["painted", 15]]);
    h.at(90);
    h.reporter.saw("commandBar", 90);
    expect(h.sent).toEqual([
      ["painted", 15],
      ["commandBar", 0],
    ]);
  });

  it("reports each mark once, its first time", () => {
    const h = harness();
    h.reporter.enable(true);
    h.at(10);
    h.reporter.saw("painted", 10);
    h.at(30);
    h.reporter.saw("painted", 30);
    h.reporter.enable(true);
    expect(h.sent).toEqual([["painted", 0]]);
  });

  it("sends nothing outside smoke tests", () => {
    const h = harness();
    h.reporter.saw("painted", 1);
    h.reporter.enable(false);
    h.reporter.saw("commandBar", 2);
    // A later answer can't switch it on.
    h.reporter.enable(true);
    expect(h.sent).toEqual([]);
  });

  it("never reports a time in the future", () => {
    const h = harness();
    h.at(5);
    h.reporter.enable(true);
    h.reporter.saw("painted", 9);
    expect(h.sent).toEqual([["painted", 0]]);
  });

  it("runs after the frame and a task behind it", () => {
    const order: string[] = [];
    const frames: (() => void)[] = [];
    const tasks: (() => void)[] = [];
    afterPaint(
      () => order.push("then"),
      (cb) => frames.push(cb),
      (cb) => tasks.push(cb),
    );
    expect(order).toEqual([]);
    frames.shift()?.();
    expect(order).toEqual([]);
    tasks.shift()?.();
    expect(order).toEqual(["then"]);
  });
});
