import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { findFailures, findSummary, foundLine } from "./smoke-find.mts";

const log = (rows: string[]) => parseLog(rows.map((row, i) => `${1000 + i} ${row}`).join("\n") + "\n");

/** A whole run as Windows and Linux log it. */
const COUNTED = [
  "started",
  "find-engine webview2 147.0.3001.0 find=true",
  "title LJ-FIND ready",
  "find-open",
  "find-action start length=2",
  "find-result matches=3 active=1",
  "find-action start length=9",
  "find-result matches=3",
  "find-result matches=3 active=1",
  "find-action next",
  "find-result matches=3 active=2",
  "find-action stop",
  "find-closed user",
];

/** A whole run as macOS logs it. */
const FOUND = [
  "menu-keys find=⌘F",
  "started",
  "find-open",
  "find-action start length=9",
  "find-engine wkwebview find=true",
  "find-result found=true",
  "find-action next",
  "find-action stop",
  "find-closed user",
];

describe("the find run's rules (LJ49)", () => {
  it("passes a whole run", () => {
    expect(findFailures({ events: log(COUNTED) }, "win32")).toEqual([]);
    expect(findFailures({ events: log(COUNTED) }, "linux")).toEqual([]);
    expect(findFailures({ events: log(FOUND) }, "darwin")).toEqual([]);
  });

  it("wants Ctrl+F to open the strip on Windows and Linux", () => {
    expect(findFailures({ events: log(["started"]) }, "linux").join()).toContain("didn't open the find strip");
    // macOS: best effort, but the menu must have Find… on ⌘F.
    expect(findFailures({ events: log(["menu-keys find=⌘F", "started"]) }, "darwin")).toEqual([]);
    expect(findFailures({ events: log(["menu-keys find=none", "started"]) }, "darwin").join()).toContain("not ⌘F");
  });

  it("wants the typed needle to reach Rust where keys are driven", () => {
    const noTyping = log(["started", "find-open"]);
    expect(findFailures({ events: noTyping }, "win32").join()).toContain("never reached Rust");
    expect(findFailures({ events: log(["menu-keys find=⌘F", "find-open"]) }, "darwin")).toEqual([]);
  });

  it("wants three matches with the first active, after the needle's own query", () => {
    const early = COUNTED.filter((row) => row !== "find-result matches=3 active=1");
    expect(findFailures({ events: log(early) }, "win32").join()).toContain('no "find-result matches=3 active=1"');
    // A count for an earlier, shorter query doesn't count.
    const onlyEarlier = COUNTED.slice(0, 6).concat(["find-action start length=9", "find-result matches=0", "find-action next", "find-action stop", "find-closed user"]);
    expect(findFailures({ events: log(onlyEarlier) }, "linux").join()).toContain("got [matches=0]");
  });

  it("wants Enter to go to the second match", () => {
    const stuck = COUNTED.filter((row) => row !== "find-result matches=3 active=2");
    expect(findFailures({ events: log(stuck) }, "linux").join()).toContain("after Enter");
    const noEnter = COUNTED.filter((row) => row !== "find-action next");
    expect(findFailures({ events: log(noEnter) }, "win32").join()).toContain("never asked for the next match");
    const macNoEnter = FOUND.filter((row) => row !== "find-action next");
    expect(findFailures({ events: log(macNoEnter) }, "darwin").join()).toContain("never asked for the next match");
  });

  it("wants Esc to close the strip", () => {
    const open = COUNTED.filter((row) => row !== "find-closed user");
    expect(findFailures({ events: log(open) }, "win32").join()).toContain("didn't close the strip");
  });

  it("fails when the engine has no find", () => {
    const old = [...COUNTED, "find-result unavailable"];
    expect(findFailures({ events: log(old) }, "win32").join()).toContain("has no find");
  });

  it("knows what each engine says", () => {
    expect(foundLine("win32")).toBe("matches=3 active=1");
    expect(foundLine("linux", 2)).toBe("matches=3 active=2");
    expect(foundLine("darwin")).toBe("found=true");
  });

  it("summarises the run", () => {
    const summary = findSummary({ events: log(COUNTED) }, "win32");
    expect(summary.engine).toBe("webview2 147.0.3001.0 find=true");
    expect(summary.opened).toBe(true);
    expect(summary.closed).toEqual(["user"]);
    expect(summary.results).toContain("matches=3 active=2");
    expect(summary.rules).toEqual([]);
  });
});
