import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { quickFailures, quickSummary, settingsOf, type QuickRun } from "./smoke-quick.mts";

const URL = "http://127.0.0.1:5000/smoke.html?quick=1";

function good(over: Partial<QuickRun> = {}): QuickRun {
  return {
    events: parseLog("1000 started\n1001 quick-command setMobileSites false\n1002 quick-command setThemeMode dark\n1003 theme dark colors=1\n"),
    mobile: { typed: true, logged: ["setMobileSites false"], settings: { mobileSites: false, theme: { mode: "system" } } },
    dark: { typed: true, logged: ["setThemeMode dark"], settings: { mobileSites: false, theme: { mode: "dark" } } },
    themes: ["dark colors=1"],
    copy: { typed: true, clipboard: URL, url: URL },
    ...over,
  };
}

describe("the quick-commands run's rules (LJ42)", () => {
  it("passes a run where both commands ran and were saved", () => {
    for (const os of ["win32", "linux", "darwin"]) {
      expect(quickFailures(good(), os), os).toEqual([]);
    }
  });

  it("wants the command logged and settings.json changed", () => {
    const notLogged = good({ mobile: { typed: true, logged: [], settings: { mobileSites: false } } });
    expect(quickFailures(notLogged, "linux").join()).toContain('not "quick-command setMobileSites false"');
    const notSaved = good({ mobile: { typed: true, logged: ["setMobileSites false"], settings: { mobileSites: true } } });
    expect(quickFailures(notSaved, "win32").join()).toContain('"mobileSites": false');
    const lightSaved = good({ dark: { typed: true, logged: ["setThemeMode dark"], settings: { theme: { mode: "light" } } } });
    expect(quickFailures(lightSaved, "win32").join()).toContain('mode "dark"');
    expect(quickFailures(good({ themes: ["light colors=1"] }), "linux").join()).toContain("not a dark theme");
  });

  it("wants the typing on Windows and Linux; on macOS the rules apply only when it showed up", () => {
    const untyped = good({ mobile: { typed: false, logged: [], settings: null }, dark: { typed: false, logged: [], settings: null }, themes: [] });
    expect(quickFailures(untyped, "darwin")).toEqual([]);
    expect(quickFailures(untyped, "win32")).toHaveLength(2);
    const typedOnMac = good({ mobile: { typed: true, logged: [], settings: null } });
    expect(quickFailures(typedOnMac, "darwin")).toHaveLength(2);
  });

  it("keeps Copy link informational", () => {
    expect(quickFailures(good({ copy: { typed: true, clipboard: "something else", url: URL } }), "linux")).toEqual([]);
    expect(quickSummary(good(), "linux").copyLink).toEqual({ typed: true, clipboard: URL, matches: true });
    expect(quickSummary(good({ copy: { typed: true, clipboard: null, url: URL } }), "linux").copyLink.matches).toBe(false);
  });

  it("reads settings.json leniently", () => {
    expect(settingsOf('{"mobileSites": false}')).toEqual({ mobileSites: false });
    expect(settingsOf("[1]")).toBeNull();
    expect(settingsOf("nope")).toBeNull();
    expect(settingsOf(undefined)).toBeNull();
  });
});
