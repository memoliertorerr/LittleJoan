import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { keysFailures, onBlurFailures, openerFailures, openerUrl, placeOf, popupsFailures, popupsSummary, spoofFailures, type PopupFlow, type PopupsRun } from "./smoke-popups.mts";
import { popupRequest } from "../../tests/smoke/popup-logic.mjs";

const AT = "http://127.0.0.1:8124";
const PAGE = "http://127.0.0.1:8123/smoke.html";
const OPENER = "http://127.0.0.1:8123";
const log = (lines: string[]) => parseLog(lines.map((line, i) => `${i + 1} ${line}`).join("\n") + "\n");

function flow(mode: "oauth" | "blank", over: string[] | null = null): PopupFlow {
  const opened = mode === "oauth" ? AT : "about:blank";
  return {
    mode,
    events: log(
      over ?? [
        "new-window",
        `popup-opened ${opened}`,
        `popup-loaded ${AT}`,
        "title LJ-POPUP opened=1",
        "title LJ-POPUP received=1 closed=0",
        "popup-closed by-page",
        "title LJ-POPUP received=1 closed=1",
      ],
    ),
  };
}

function run(over: Partial<PopupsRun> = {}): PopupsRun {
  return {
    flows: [flow("oauth"), flow("blank")],
    spoof: {
      events: log([
        "new-window",
        `popup-title ${OPENER}`,
        `popup-opened ${AT}`,
        "title LJ-POPUP spoofed opened=1 written=1",
        `popup-title ${OPENER}`,
        "popup-closed by-page",
        "title LJ-POPUP received=0 closed=1",
      ]),
    },
    hibernate: {
      events: log(["new-window", `popup-opened ${AT}`, `popup-loaded ${AT}`, "closing ready", "popup-closed hibernate", "popups-closed 1", "page-closed", "hibernated ready"]),
      helpers: 3,
      left: [0],
    },
    opener: {
      sameOrigin: log(["new-window", `popup-opened ${AT}`, `popup-loaded ${AT}`, "page-started http://127.0.0.1:8123/smoke.html?popup=oauth", "title LJ-POPUP ready"]),
      otherOrigin: log([`page-started ${AT}/smoke.html`, "popup-closed opener-left"]),
    },
    keys: {
      afterHistoryKey: log([]),
      afterClose: log(["popup-closed user"]),
      opened: true,
    },
    onblur: {
      before: log(["title LJ-POPUP armed"]),
      afterHide: log(["closing ready", "new-window", "new-window-dropped closing", "page-closed", "hibernated ready"]),
    },
    ...over,
  };
}

describe("the popups run (LJ47)", () => {
  it("passes a sign-in that posts back and closes itself, and a popup closed with the window", () => {
    expect(popupsFailures(run(), "linux", AT, PAGE)).toEqual([]);
    expect(popupsSummary(run(), "linux", AT, PAGE).rules).toEqual([]);
  });

  it("opens the opener with the sign-in origin the page accepts", () => {
    const url = openerUrl("http://127.0.0.1:8123/smoke.html", AT, "blank", true);
    expect(popupRequest(new URL(url).search)).toEqual({ mode: "blank", at: AT, stay: true });
  });

  it("fails a popup that went to the browser, never posted back, or stayed open", () => {
    const handed = run({ flows: [flow("oauth", ["new-window", "handoff os-default"]), flow("blank")] });
    const failures = popupsFailures(handed, "win32", AT, PAGE).join("\n");
    expect(failures).toContain(`no popup-opened ${AT}`);
    expect(failures).toContain("logged handoff");
    expect(failures).toContain("LJ-POPUP received=1 closed=1");
    expect(failures).toContain("popup-closed by-page");
    const silent = run({ flows: [flow("oauth"), flow("blank", ["new-window", "popup-opened about:blank", `popup-loaded ${AT}`, "title LJ-POPUP received=0 closed=0"])] });
    expect(popupsFailures(silent, "linux", AT, PAGE).join("\n")).toContain("reported LJ-POPUP received=0 closed=0");
  });

  it("requires the click on Windows and Linux, and reports it on macOS", () => {
    const missed = run({ flows: [flow("oauth", ["title LJ-POPUP ready"]), flow("blank")] });
    expect(popupsFailures(missed, "linux", AT, PAGE)[0]).toContain("asked for no new window");
    expect(popupsFailures(missed, "darwin", AT, PAGE)).toEqual([]);
  });

  it("closes the popup before the page, and leaves no helper behind", () => {
    const late = run({ hibernate: { events: log(["new-window", `popup-opened ${AT}`, "page-closed", "popup-closed hibernate", "hibernated"]), helpers: 2, left: [0] } });
    expect(popupsFailures(late, "linux", AT, PAGE)).toEqual(["popups: hibernate: the page went before its popup"]);
    const kept = run({ hibernate: { events: log(["new-window", `popup-opened ${AT}`, "page-closed", "hibernated"]), helpers: 2, left: [1, 1] } });
    const failures = popupsFailures(kept, "win32", AT, PAGE).join("\n");
    expect(failures).toContain("popup-closed hibernate");
    expect(failures).toContain("1 of the 2 webview helpers");
    expect(popupsFailures(run({ hibernate: null }), "linux", AT, PAGE)).toEqual(["popups: the hibernate check never ran"]);
  });
});

describe("the popups run's lifecycle and keys (LJ48)", () => {
  it("keeps a popup over a same-origin navigation and closes it when the opener leaves", () => {
    expect(openerFailures(run().opener, "linux")).toEqual([]);
    const closedEarly = run({
      opener: { sameOrigin: log(["new-window", `popup-opened ${AT}`, "popup-closed opener-left"]), otherOrigin: log([]) },
    });
    const failures = openerFailures(closedEarly.opener, "win32").join("\n");
    expect(failures).toContain("own origin closed its popup");
    expect(failures).toContain("its popup stayed");
    expect(openerFailures(null, "linux")).toEqual(["popups: the opener check never ran"]);
  });

  it("lets only Ctrl+W act in a popup, and closes the popup only", () => {
    expect(keysFailures(run().keys, "linux")).toEqual([]);
    const acted = { afterHistoryKey: log(["history-view"]), afterClose: log(["closing ready", "popup-closed hibernate", "hibernated ready"]), opened: true };
    const failures = keysFailures(acted, "win32").join("\n");
    expect(failures).toContain("the history key acted");
    expect(failures).toContain("didn't close it");
    expect(failures).toContain("closed the Little Joan window too");
    // Best effort on macOS.
    expect(keysFailures(acted, "darwin")).toEqual([]);
    expect(keysFailures({ afterHistoryKey: [], afterClose: [], opened: false }, "linux")).toEqual(["popups: keys: no popup opened"]);
  });

  it("counts the lifecycle and keys among the run's rules", () => {
    expect(popupsFailures(run({ opener: null, keys: null }), "linux", AT, PAGE)).toEqual([
      "popups: the opener check never ran",
      "popups: the keys check never ran",
    ]);
  });
});

describe("the security review's popup rules (phase 22)", () => {
  it("never titles a spoofed popup by the origin that never loaded", () => {
    expect(placeOf(PAGE)).toBe(OPENER);
    expect(spoofFailures(run().spoof, "linux", AT, PAGE)).toEqual([]);
    // The bug: built titled by its target.
    const spoofed = { events: log(["new-window", `popup-title ${AT}`, `popup-opened ${AT}`, "title LJ-POPUP spoofed opened=1 written=1"]) };
    const failures = spoofFailures(spoofed, "win32", AT, PAGE).join("\n");
    expect(failures).toContain(`built titled ${AT}`);
    expect(failures).toContain(`titled ${AT} with nothing of it loaded`);
    // Titled by it later, without a commit.
    const later = { events: log(["new-window", `popup-title ${OPENER}`, `popup-opened ${AT}`, `popup-title ${AT}`, "title LJ-POPUP spoofed opened=1 written=1"]) };
    expect(spoofFailures(later, "linux", AT, PAGE)[0]).toContain("with nothing of it loaded");
    // Something of the hang address loaded: the test proves nothing then.
    const loaded = { events: log(["new-window", `popup-title ${OPENER}`, `popup-opened ${AT}`, `popup-loaded ${AT}`, "title LJ-POPUP spoofed opened=1 written=1"]) };
    expect(spoofFailures(loaded, "linux", AT, PAGE)[0]).toContain("loaded in the popup");
    // The opener couldn't write into it.
    const unwritten = { events: log(["new-window", `popup-title ${OPENER}`, `popup-opened ${AT}`, "title LJ-POPUP spoofed opened=1 written=0"]) };
    expect(spoofFailures(unwritten, "linux", AT, PAGE)[0]).toContain("not that it wrote");
    // No popup; no click (reported on macOS only).
    expect(spoofFailures({ events: log(["new-window", "handoff browser=x"]) }, "linux", AT, PAGE)[0]).toContain("no popup opened");
    expect(spoofFailures({ events: log([]) }, "linux", AT, PAGE)[0]).toContain("asked for no new window");
    expect(spoofFailures({ events: log([]) }, "darwin", AT, PAGE)).toEqual([]);
    expect(spoofFailures(null, "linux", AT, PAGE)).toEqual(["popups: the spoof check never ran"]);
  });

  it("opens nothing for a closing page", () => {
    expect(onBlurFailures(run().onblur)).toEqual([]);
    const opened = { before: log(["title LJ-POPUP armed"]), afterHide: log(["closing ready", "new-window", "popup-pending", "popup-opened 127.0.0.1", "hibernated ready"]) };
    expect(onBlurFailures(opened)[0]).toContain("2 popup(s) opened while the window closed");
    const handed = { before: log(["title LJ-POPUP armed"]), afterHide: log(["closing ready", "new-window", "handoff-admitted", "hibernated ready"]) };
    expect(onBlurFailures(handed)[0]).toContain("handed on");
    expect(onBlurFailures({ before: log([]), afterHide: log(["hibernated ready"]) })[0]).toContain("never armed");
    expect(onBlurFailures(null)).toEqual(["popups: the onblur check never ran"]);
    expect(popupsFailures(run({ spoof: null, onblur: null }), "linux", AT, PAGE)).toEqual([
      "popups: the spoof check never ran",
      "popups: the onblur check never ran",
    ]);
  });
});
