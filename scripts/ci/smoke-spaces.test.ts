import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import {
  FIRST_LAUNCH,
  PROBE_BITS,
  frontOf,
  parseFront,
  parseSpaces,
  parseStatus,
  repeatLaunch,
  repeatSummary,
  spacesFailures,
  spacesRepeat,
  spacesSummary,
  type HelperStatus,
  type SpacesRun,
  type Summon,
} from "./smoke-spaces.mts";

const log = (rows: string[]) => parseLog(rows.map((row, i) => `${1000 + i} ${row}`).join("\n") + "\n");

const APP = 4242;
const status = (over: Partial<HelperStatus> = {}): HelperStatus => ({ t: 5000, state: "status", fullscreen: true, ...over });

function summon(name: Summon["name"], label: Summon["label"], front: string, over: Partial<Summon> = {}): Summon {
  return {
    name,
    trigger: name === "shortcut" ? "shortcut" : "cli",
    label,
    helperActiveBefore: true,
    events: log([label === "joan" ? "window-created" : "settings-window general", `window-front ${label} ${front}`]),
    helperAfter: status({ active: false, frontPid: APP, onActiveSpace: true }),
    activePid: APP,
    ...over,
  };
}

function good(front = "focused=true top=true", over: Partial<SpacesRun> = {}): SpacesRun {
  return {
    helperUp: status(),
    appPid: APP,
    summons: [summon("show", "joan", front), summon("shortcut", "joan", front), summon("settings", "settings", front)],
    spaces: log([
      "window-spaces joan behavior=moveToActiveSpace|fullScreenAuxiliary was=default level=0",
      "window-spaces joan behavior=moveToActiveSpace|fullScreenAuxiliary was=default level=0",
      "window-spaces settings behavior=moveToActiveSpace|fullScreenAuxiliary was=default level=0",
    ]),
    probes: [],
    ...over,
  };
}

const MAC = "focused=false top=false space=true";

describe("the over-full-screen run's lines (LJ19.2)", () => {
  it("reads the helper's status file, skipping a line cut short", () => {
    const text = '{"t":1,"state":"shown","pid":7}\r\n\uFEFF{"t":2,"state":"status","active":true,"frontPid":7}\n{"t":3,"sta';
    expect(parseStatus(text)).toEqual([
      { t: 1, state: "shown", pid: 7 },
      { t: 2, state: "status", active: true, frontPid: 7 },
    ]);
    expect(parseStatus('{"state":"status"}\n[]\n')).toEqual([]);
  });

  it("reads window-front and window-spaces lines", () => {
    expect(parseFront("joan focused=true top=false")).toEqual({ label: "joan", unknown: false, focused: true, top: false });
    expect(parseFront("settings focused=false top=true space=true")).toEqual({ label: "settings", unknown: false, focused: false, top: true, space: true });
    expect(parseFront("joan unknown")).toEqual({ label: "joan", unknown: true });
    expect(parseFront("")).toBeUndefined();
    const events = log(["window-front joan focused=false top=false", "window-front settings focused=true top=true", "window-front joan focused=true top=true"]);
    expect(frontOf(events, "joan")).toEqual({ label: "joan", unknown: false, focused: true, top: true });
    expect(frontOf(events, "other")).toBeUndefined();
    expect(parseSpaces("joan behavior=moveToActiveSpace|fullScreenAuxiliary was=default level=0")).toEqual({
      label: "joan",
      behavior: ["moveToActiveSpace", "fullScreenAuxiliary"],
      was: ["default"],
      level: 0,
    });
    expect(parseSpaces("joan behavior=x")).toBeUndefined();
  });

  it("probes AppKit's default, Little Joan's behaviour, and it with canJoinAllApplications", () => {
    expect(PROBE_BITS).toEqual([0, 258, 258 + 262_144]);
  });
});

describe("the over-full-screen run's rules (LJ19.2)", () => {
  it("passes a run where every summon came up in front with the keyboard", () => {
    expect(spacesFailures(good(), "win32")).toEqual([]);
    expect(spacesFailures(good(), "linux")).toEqual([]);
    expect(spacesFailures(good(MAC), "darwin")).toEqual([]);
  });

  it("wants the full-screen window up and active before each summon on Windows and Linux", () => {
    expect(spacesFailures(good("focused=true top=true", { helperUp: null }), "linux").join()).toContain("never came up full screen");
    const notActive = good();
    notActive.summons[1] = summon("shortcut", "joan", "focused=true top=true", { helperActiveBefore: false });
    expect(spacesFailures(notActive, "win32").join()).toContain("(shortcut): the full-screen window wasn't the active window");
  });

  it("wants each summoned window focused, on top and the active window", () => {
    for (const front of ["focused=false top=true", "focused=true top=false", "unknown"]) {
      const run = good();
      run.summons[0] = summon("show", "joan", front);
      const failures = spacesFailures(run, "linux");
      expect(failures, front).toHaveLength(1);
      expect(failures[0], front).toMatch(front === "unknown" ? /no window-front line for joan/ : /not in front with the keyboard/);
    }
    const elsewhere = good();
    elsewhere.summons[2] = summon("settings", "settings", "focused=true top=true", { activePid: 99 });
    expect(spacesFailures(elsewhere, "win32").join()).toContain("(settings): the active window belongs to pid 99, not the app (4242)");
    const noLine = good();
    noLine.summons[0] = { ...summon("show", "joan", "focused=true top=true"), events: log(["window-created"]) };
    expect(spacesFailures(noLine, "darwin").join()).toContain("(show): no window-front line for joan");
  });

  it("wants no Space switch on macOS, and the window on the Space on screen", () => {
    const left = good(MAC);
    left.summons[0] = summon("show", "joan", MAC, { helperAfter: status({ onActiveSpace: false }) });
    expect(spacesFailures(left, "darwin").join()).toContain("(show): the full-screen app's Space was left");
    const away = good(MAC);
    away.summons[1] = summon("shortcut", "joan", "focused=true top=true space=false");
    expect(spacesFailures(away, "darwin").join()).toContain("(shortcut): joan isn't on the Space on screen (space=false)");
  });

  it("on a macOS runner that refused full screen, still wants the collection behaviour", () => {
    const refused = good("focused=false top=false space=false", { helperUp: null });
    refused.summons = refused.summons.map((s) => ({ ...s, helperAfter: status({ onActiveSpace: false, fullscreen: false }) }));
    expect(spacesFailures(refused, "darwin")).toEqual([]);
    refused.spaces = log(["window-spaces joan behavior=fullScreenPrimary was=default level=0", "window-spaces settings behavior=moveToActiveSpace|fullScreenAuxiliary was=default level=3"]);
    const failures = spacesFailures(refused, "darwin");
    expect(failures).toHaveLength(2);
    expect(failures[0]).toContain("joan's collection behaviour is fullScreenPrimary");
    expect(failures[1]).toContain("settings's window level is 3, not the normal level (0)");
    refused.spaces = log(["window-spaces joan behavior=moveToActiveSpace|fullScreenAuxiliary|canJoinAllSpaces was=default level=0"]);
    const joined = spacesFailures(refused, "darwin");
    expect(joined[0]).toContain("not moveToActiveSpace + fullScreenAuxiliary");
    expect(joined[1]).toContain("no window-spaces line for settings");
  });

  it("summarizes each summon and probe", () => {
    const run = good(MAC, {
      probes: [{ bits: 0, probe: { t: 9, state: "probe", onActiveSpace: false, key: true, active: true, top: false, behavior: 0, level: 0 }, helperDuring: status({ onActiveSpace: false, active: false }) }],
    });
    const summary = spacesSummary(run, "darwin");
    expect(summary.helperUp).toBe(true);
    expect(summary.summons.map((s) => [s.name, s.trigger, s.front?.space])).toEqual([
      ["show", "cli", true],
      ["shortcut", "shortcut", true],
      ["settings", "cli", true],
    ]);
    expect(summary.probes[0]).toEqual({
      bits: 0,
      window: { onActiveSpace: false, key: true, active: true, top: false, behavior: 0, level: 0 },
      helperDuring: { active: false, fullscreen: true, onActiveSpace: false, frontPid: undefined, top: undefined },
    });
    expect(summary.rules).toEqual([]);
  });
});

describe("the repeated over-full-screen run (LJ19.6, LJ_SPACES_REPEAT)", () => {
  it("is one launch unless asked for more, at most 200", () => {
    expect(spacesRepeat(undefined)).toBe(1);
    expect(spacesRepeat("")).toBe(1);
    expect(spacesRepeat("abc")).toBe(1);
    expect(spacesRepeat("0")).toBe(1);
    expect(spacesRepeat("-3")).toBe(1);
    expect(spacesRepeat("2.5")).toBe(1);
    expect(spacesRepeat(" 60 ")).toBe(60);
    expect(spacesRepeat("1000")).toBe(200);
  });

  it("names each launch's log; only the first probes and screenshots", () => {
    expect(FIRST_LAUNCH).toEqual({ name: "spaces", probes: true, shots: true });
    expect(repeatLaunch(2)).toEqual({ name: "spaces-2", probes: false, shots: false });
    expect(repeatLaunch(60).name).toBe("spaces-60");
  });

  it("counts the launches whose rules failed", () => {
    const left = good(MAC);
    left.summons = left.summons.map((s, i) => (i < 2 ? { ...s, helperAfter: status({ onActiveSpace: false }) } : s));
    const summary = repeatSummary(
      [
        { name: "spaces", run: good(MAC) },
        { name: "spaces-2", run: left },
        { name: "spaces-3", run: good(MAC, { helperUp: null }) },
      ],
      "darwin",
    );
    expect(summary.launches).toBe(3);
    expect(summary.failed).toBe(1);
    expect(summary.results.map((r) => [r.name, r.helperUp, r.rules.length])).toEqual([
      ["spaces", true, 0],
      ["spaces-2", true, 2],
      ["spaces-3", false, 0],
    ]);
    expect(summary.results[1]?.rules[0]).toContain("(show): the full-screen app's Space was left");
  });
});
