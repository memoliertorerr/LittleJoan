import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { openerUrl, popupsFailures, popupsSummary, type PopupFlow, type PopupsRun } from "./smoke-popups.mts";
import { popupRequest } from "../../tests/smoke/popup-logic.mjs";

const AT = "http://127.0.0.1:8124";
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
    hibernate: {
      events: log(["new-window", `popup-opened ${AT}`, `popup-loaded ${AT}`, "closing ready", "popup-closed hibernate", "popups-closed 1", "page-closed", "hibernated ready"]),
      helpers: 3,
      left: [0],
    },
    ...over,
  };
}

describe("the popups run (LJ47)", () => {
  it("passes a sign-in that posts back and closes itself, and a popup closed with the window", () => {
    expect(popupsFailures(run(), "linux", AT)).toEqual([]);
    expect(popupsSummary(run(), "linux", AT).rules).toEqual([]);
  });

  it("opens the opener with the sign-in origin the page accepts", () => {
    const url = openerUrl("http://127.0.0.1:8123/smoke.html", AT, "blank", true);
    expect(popupRequest(new URL(url).search)).toEqual({ mode: "blank", at: AT, stay: true });
  });

  it("fails a popup that went to the browser, never posted back, or stayed open", () => {
    const handed = run({ flows: [flow("oauth", ["new-window", "handoff os-default"]), flow("blank")] });
    const failures = popupsFailures(handed, "win32", AT).join("\n");
    expect(failures).toContain(`no popup-opened ${AT}`);
    expect(failures).toContain("logged handoff");
    expect(failures).toContain("LJ-POPUP received=1 closed=1");
    expect(failures).toContain("popup-closed by-page");
    const silent = run({ flows: [flow("oauth"), flow("blank", ["new-window", "popup-opened about:blank", `popup-loaded ${AT}`, "title LJ-POPUP received=0 closed=0"])] });
    expect(popupsFailures(silent, "linux", AT).join("\n")).toContain("reported LJ-POPUP received=0 closed=0");
  });

  it("requires the click on Windows and Linux, and reports it on macOS", () => {
    const missed = run({ flows: [flow("oauth", ["title LJ-POPUP ready"]), flow("blank")] });
    expect(popupsFailures(missed, "linux", AT)[0]).toContain("asked for no new window");
    expect(popupsFailures(missed, "darwin", AT)).toEqual([]);
  });

  it("closes the popup before the page, and leaves no helper behind", () => {
    const late = run({ hibernate: { events: log(["new-window", `popup-opened ${AT}`, "page-closed", "popup-closed hibernate", "hibernated"]), helpers: 2, left: [0] } });
    expect(popupsFailures(late, "linux", AT)).toEqual(["popups: hibernate: the page went before its popup"]);
    const kept = run({ hibernate: { events: log(["new-window", `popup-opened ${AT}`, "page-closed", "hibernated"]), helpers: 2, left: [1, 1] } });
    const failures = popupsFailures(kept, "win32", AT).join("\n");
    expect(failures).toContain("popup-closed hibernate");
    expect(failures).toContain("1 of the 2 webview helpers");
    expect(popupsFailures(run({ hibernate: null }), "linux", AT)).toEqual(["popups: the hibernate check never ran"]);
  });
});
