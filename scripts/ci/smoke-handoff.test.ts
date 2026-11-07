import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { BLANK_TARGET, FAKE_DESKTOP_ID, FLOOD_TARGET, fakeDesktopFile, fakeReceived, floodFailures, handoffFailures, ourMimeapps, type HandoffRun } from "./smoke-handoff.mts";
import { BLANK_TARGET as PAGE_TARGET, FLOOD_TARGET as PAGE_FLOOD } from "../../tests/smoke/smoke-logic.mjs";
import { expandExec } from "./smoke-links-read.mts";

const log = (lines: string[]) => parseLog(lines.map((line, i) => `${i + 1} ${line}`).join("\n") + "\n");
/** Lines at given milliseconds: [ms, line]. */
const timed = (lines: [number, string][]) => parseLog(lines.map(([ms, line]) => `${ms} ${line}`).join("\n") + "\n");
const FLOOD_URL = (n: number) => `http://127.0.0.1:8123/${FLOOD_TARGET}&n=${n}`;
const URL = `http://127.0.0.1:8123/${BLANK_TARGET}`;

function run(over: Partial<HandoffRun> = {}): HandoffRun {
  return {
    fake: FAKE_DESKTOP_ID,
    skipped: null,
    listed: ["2 default=other"],
    scrolled: true,
    main: { afterClick: log(["browsers-listed 2 default=other", `handoff browser=${FAKE_DESKTOP_ID}`]), received: [URL] },
    flood: {
      afterClicks: timed([
        [1000, "new-window"],
        [1000, "handoff-admitted"],
        [1001, `handoff browser=${FAKE_DESKTOP_ID}`],
        [1002, "new-window"],
        [1003, "handoff-throttled"],
        [1004, "new-window"],
        [1005, "handoff-throttled"],
        [2100, "new-window"],
        [2100, "handoff-admitted"],
        [2101, `handoff browser=${FAKE_DESKTOP_ID}`],
      ]),
      received: [FLOOD_URL(1), FLOOD_URL(4)],
    },
    loop: {
      afterClick: log(["browsers-listed 2 default=self", "handoff-self-refused", `handoff browser=${FAKE_DESKTOP_ID}`]),
      received: [URL],
      listed: ["2 default=self"],
    },
    ...over,
  };
}

describe("the handoff run", () => {
  it("passes a handoff to the main browser and, with Little Joan the default, to the first other one", () => {
    expect(handoffFailures(run(), "linux")).toEqual([]);
    expect(handoffFailures(run({ loop: null, fake: "LJFakeBrowser", main: { afterClick: log(["handoff browser=LJFakeBrowser"]), received: [URL] } }), "win32")).toEqual([]);
  });

  it("lets a flood hand on at most one page a second, and logs every one it drops", () => {
    expect(floodFailures(run().flood)).toEqual([]);
    const tooMany = { afterClicks: timed([[0, "new-window"], [0, "handoff-admitted"], [1, "handoff browser=x"], [5, "new-window"], [5, "handoff-admitted"], [6, "handoff browser=x"]]), received: [] };
    expect(floodFailures(tooMany)[0]).toContain("5 ms apart");
    const unaccounted = { afterClicks: timed([[0, "new-window"], [1, "new-window"], [1, "handoff-admitted"], [2, "handoff browser=x"]]), received: [] };
    expect(floodFailures(unaccounted)[0]).toContain("every one must be one or the other");
    const received = { afterClicks: timed([[0, "new-window"], [0, "handoff-admitted"], [1, "handoff browser=x"]]), received: [FLOOD_URL(1), FLOOD_URL(2)] };
    expect(floodFailures(received)[0]).toContain("more than the 1 handed on");
    expect(floodFailures({ afterClicks: [], received: [] })[0]).toContain("no new window");
    // Linux (LJ47): a request decided at ready-to-show is handed on after the last request — still
    // one a second at the gate: admissions 1.1 s apart pass, 0.1 s apart fail.
    const deferred = { afterClicks: timed([[0, "new-window"], [1, "handoff-admitted"], [2, "handoff browser=x"], [5, "new-window"], [1101, "handoff-admitted"], [1102, "handoff browser=x"]]), received: [] };
    expect(floodFailures(deferred)).toEqual([]);
    const leaked = { afterClicks: timed([[0, "new-window"], [1, "handoff-admitted"], [2, "handoff browser=x"], [5, "new-window"], [101, "handoff-admitted"], [102, "handoff browser=x"]]), received: [] };
    expect(floodFailures(leaked)[0]).toContain("100 ms apart");
    // The review's case: two admissions close together, their `handoff` lines bunched up later by
    // the browser list — the old span rule (to the last handoff) let it pass.
    const bunched = {
      afterClicks: timed([[0, "new-window"], [0, "handoff-admitted"], [10, "new-window"], [400, "handoff-admitted"], [2500, "handoff browser=x"], [2501, "handoff browser=x"]]),
      received: [],
    };
    expect(floodFailures(bunched)[0]).toContain("400 ms apart");
    // A handoff the gate never admitted.
    const unadmitted = { afterClicks: timed([[0, "new-window"], [1, "handoff browser=x"]]), received: [] };
    expect(floodFailures(unadmitted)[0]).toContain("admitted 0");
    expect(floodFailures(null)[0]).toContain("never ran");
    expect(handoffFailures(run({ flood: null }), "linux").join("\n")).toContain("never ran");
  });

  it("asks only for the list and the screenshot on macOS", () => {
    const mac = { fake: null, main: null, flood: null, loop: null };
    expect(handoffFailures(run(mac), "darwin")).toEqual([]);
    expect(handoffFailures(run({ ...mac, listed: [] }), "darwin")[0]).toContain("browsers-listed");
    expect(handoffFailures(run({ ...mac, scrolled: false }), "darwin")[0]).toContain("Browser section");
  });

  it("fails a page that went elsewhere, or came back", () => {
    const toDefault = run({ main: { afterClick: log(["handoff os-default"]), received: [] } });
    const text = handoffFailures(toDefault, "linux").join("\n");
    expect(text).toContain("not handoff browser=");
    expect(text).toContain("the fake browser received []");
    const bounced = run({ loop: { afterClick: log([`handoff browser=${FAKE_DESKTOP_ID}`, "handoff-self-refused", "link-opened"]), received: [URL], listed: ["1 default=self"] } });
    expect(handoffFailures(bounced, "linux").join("\n")).toContain("a loop");
    const notSelf = run({ loop: { afterClick: log([`handoff browser=${FAKE_DESKTOP_ID}`]), received: [URL], listed: ["1 default=other"] } });
    const loopText = handoffFailures(notSelf, "linux").join("\n");
    expect(loopText).toContain("handoff-self-refused");
    expect(loopText).toContain("default=self");
    // LJ47: a plain target=_blank link never opens a sign-in popup.
    const popup = run({ main: { afterClick: log([`handoff browser=${FAKE_DESKTOP_ID}`, "popup-opened 127.0.0.1:8123"]), received: [URL] } });
    expect(handoffFailures(popup, "linux").join("\n")).toContain("opened a popup inside Little Joan");
  });

  it("reads what the fake browser got", () => {
    expect(fakeReceived(`${JSON.stringify([URL])}\n\nnot json\n${JSON.stringify(["b", 3])}\n`)).toEqual([URL, "b"]);
    expect(fakeReceived(undefined)).toEqual([]);
  });

  it("writes a fake desktop file that passes the URL as one argument", () => {
    const file = fakeDesktopFile("/opt/node/bin/node", "/w/out/fake-browser.mjs", "/w/out/fake.jsonl");
    const exec = /^Exec=(.*)$/m.exec(file)?.[1] ?? "";
    expect(expandExec(exec, URL)).toEqual(["/opt/node/bin/node", "/w/out/fake-browser.mjs", "/w/out/fake.jsonl", URL]);
    expect(file).toContain("MimeType=x-scheme-handler/http;x-scheme-handler/https;");
  });

  it("makes Little Joan the runner's default only in the copy it writes", () => {
    const before = "[Added Associations]\ntext/plain=a.desktop;\n[Default Applications]\nx-scheme-handler/https=firefox.desktop\n";
    const after = ourMimeapps(before);
    expect(after).toContain("x-scheme-handler/https=Little Joan.desktop");
    expect(after).not.toContain("firefox.desktop");
    expect(after).toContain("text/plain=a.desktop;");
    expect(ourMimeapps(undefined)).toContain("[Default Applications]");
  });

  it("clicks the smoke page's own link, and counts its flood", () => {
    expect(BLANK_TARGET).toBe(PAGE_TARGET);
    expect(FLOOD_TARGET).toBe(PAGE_FLOOD);
  });
});
