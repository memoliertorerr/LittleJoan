import { describe, expect, it } from "vitest";
import { afterPaint, timingReporter, type TimingMark } from "./bar-timing.ts";

function harness() {
  let clock = 0;
  const sent: Partial<Record<TimingMark, number>>[] = [];
  const reporter = timingReporter(
    (marks) => sent.push(marks),
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
  it("goes out once, with every mark as how long ago, when the command bar's rows are painted", () => {
    const h = harness();
    h.reporter.saw("navigation", 0);
    h.at(40);
    h.reporter.saw("module", 40);
    h.reporter.saw("painted", 55);
    h.reporter.enable(true);
    expect(h.sent).toEqual([]);
    h.at(90);
    h.reporter.saw("commandBar", 90);
    expect(h.sent).toEqual([{ navigation: 90, module: 50, painted: 35, commandBar: 0 }]);
    // Nothing goes out twice, and later marks stay out.
    h.reporter.saw("suggestions", 95);
    h.reporter.flush();
    expect(h.sent).toHaveLength(1);
  });

  it("waits for start_info when the rows came first", () => {
    const h = harness();
    h.at(10);
    h.reporter.saw("commandBar", 10);
    expect(h.sent).toEqual([]);
    h.at(25);
    h.reporter.enable(true);
    expect(h.sent).toEqual([{ commandBar: 15 }]);
  });

  it("goes out on a flush without rows (a window opened on a page), also before start_info answered", () => {
    const h = harness();
    h.reporter.saw("painted", 5);
    h.at(20);
    h.reporter.flush();
    expect(h.sent).toEqual([]);
    h.reporter.saw("startInfo", 20);
    h.at(30);
    h.reporter.enable(true);
    expect(h.sent).toEqual([{ painted: 25, startInfo: 10 }]);
  });

  it("keeps the first time of each mark", () => {
    const h = harness();
    h.reporter.saw("painted", 10);
    h.reporter.saw("painted", 30);
    h.reporter.enable(true);
    h.at(40);
    h.reporter.flush();
    expect(h.sent).toEqual([{ painted: 30 }]);
  });

  it("sends nothing outside smoke tests", () => {
    const h = harness();
    h.reporter.saw("painted", 1);
    h.reporter.enable(false);
    h.reporter.saw("commandBar", 2);
    h.reporter.flush();
    // A later answer can't switch it on.
    h.reporter.enable(true);
    h.reporter.flush();
    expect(h.sent).toEqual([]);
  });

  it("never reports a time in the future", () => {
    const h = harness();
    h.at(5);
    h.reporter.enable(true);
    h.reporter.saw("commandBar", 9);
    expect(h.sent).toEqual([{ commandBar: 0 }]);
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
