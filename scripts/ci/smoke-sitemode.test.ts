import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import {
  documentRequests,
  headerAgent,
  lastReport,
  overrideRefused,
  parseAgentLog,
  siteModeFailures,
  siteModesOf,
  uaMobile,
  type HeaderAgent,
  type SiteModeRun,
  type SwitchResult,
} from "./smoke-sitemode.mts";

const PAGE = "http://127.0.0.1:5000/smoke.html";
const log = (rows: string[]) => parseLog(rows.map((row, i) => `${1000 + i} ${row}`).join("\n") + "\n");
const PHONE: HeaderAgent = { android: true, mobile: "?1", platform: '"Android"' };
const DESKTOP: HeaderAgent = { android: false, mobile: "?0", platform: '"Windows"' };

function switched(mode: "desktop" | "auto", over: Partial<SwitchResult> = {}): SwitchResult {
  const desktop = mode === "desktop";
  return {
    typed: true,
    logged: [`127.0.0.1 ${mode}`],
    reloads: 1,
    report: { width: 390, height: 560, ua: desktop ? "desktop" : "phone", uamobile: desktop ? "false" : "true" },
    siteModes: desktop ? { "127.0.0.1": "desktop" } : {},
    headers: [desktop ? DESKTOP : PHONE],
    ...over,
  };
}

function good(over: Partial<SiteModeRun> = {}): SiteModeRun {
  return {
    events: log(["ua-override phone", "ua-override desktop"]),
    phoneHeaders: [PHONE],
    phoneReport: { width: 390, height: 560, ua: "phone", uamobile: "true" },
    desktop: switched("desktop"),
    auto: switched("auto"),
    fits: "skipped",
    ...over,
  };
}

describe("the agent headers", () => {
  it("are read from the server's log, per document and token", () => {
    const text = [
      JSON.stringify({ t: 1, path: "smoke.html", query: "?t=a", ua: "Mozilla/5.0 (Linux; Android 10; K)", mobile: "?1", platform: '"Android"' }),
      JSON.stringify({ t: 2, path: "smoke-logic.mjs", query: "", ua: "x", mobile: "?1", platform: '"Android"' }),
      "not json",
      JSON.stringify({ t: 3, path: "smoke.html", query: "?t=b", ua: "Mozilla/5.0 (Windows NT 10.0)", mobile: null, platform: null }),
    ].join("\n");
    const requests = parseAgentLog(text);
    expect(requests).toHaveLength(3);
    const [first] = documentRequests(requests, "a");
    expect(first && headerAgent(first)).toEqual(PHONE);
    const [webkit] = documentRequests(requests, "b");
    expect(webkit && headerAgent(webkit)).toEqual({ android: false, mobile: null, platform: null });
  });

  it("read uamobile from a report", () => {
    expect(uaMobile("LJ-SMOKE viewport=390x560 ua=phone dpr=1 uamobile=true")).toBe("true");
    expect(uaMobile("LJ-SMOKE viewport=390x560 ua=phone dpr=1 uamobile=undefined")).toBe("undefined");
    expect(uaMobile("LJ-SMOKE ready")).toBeUndefined();
    const events = log([`page-started ${PAGE}?t=x`, "title LJ-SMOKE viewport=390x560 ua=desktop dpr=1 uamobile=false"]);
    expect(lastReport(events, PAGE)).toEqual({ width: 390, height: 560, ua: "desktop", uamobile: "false" });
  });

  it("read settings.json's siteModes", () => {
    expect(siteModesOf('{"siteModes":{"127.0.0.1":"desktop"}}')).toEqual({ "127.0.0.1": "desktop" });
    expect(siteModesOf('{"version":1}')).toBeNull();
    expect(siteModesOf(undefined)).toBeNull();
    expect(siteModesOf("{")).toBeNull();
  });
});

describe("the rules", () => {
  it("pass a good run on every OS", () => {
    for (const os of ["win32", "linux", "darwin"]) {
      expect(siteModeFailures(good(), os), os).toEqual([]);
    }
  });

  it("fail a switch that didn't apply, reload once, or keep the entry", () => {
    const run = good({
      desktop: switched("desktop", { logged: [], reloads: 2, siteModes: null, report: { width: 390, height: 560, ua: "phone" } }),
      auto: switched("auto", { siteModes: { "127.0.0.1": "desktop" } }),
    });
    const failures = siteModeFailures(run, "linux");
    expect(failures).toContain('site mode: "Request desktop site" logged [], not "127.0.0.1 desktop"');
    expect(failures).toContain('site mode: after "Request desktop site" the page reported 390x560 ua=phone, not 390x560 ua=desktop');
    expect(failures).toContain('site mode: "Request desktop site" loaded the page 2 times, not once');
    expect(failures).toContain('site mode: after "Request desktop site" settings.json\'s siteModes were null');
    expect(failures).toContain('site mode: after "Automatic for this site" settings.json\'s siteModes were {"127.0.0.1":"desktop"}');
  });

  it("need the typing on Windows and Linux, and take macOS's when it arrived", () => {
    const untyped = good({ desktop: switched("desktop", { typed: false, logged: [], reloads: 0 }) });
    expect(siteModeFailures(untyped, "linux")).toEqual(['site mode: the command bar never showed the typing for "Request desktop site"']);
    expect(siteModeFailures(untyped, "darwin")).toEqual([]);
    const typedButWrong = good({ auto: switched("auto", { reloads: 0 }) });
    expect(siteModeFailures(typedButWrong, "darwin")).toEqual(['site mode: "Automatic for this site" loaded the page 0 times, not once']);
  });

  it("check Windows' client hints both ways", () => {
    const bare = good({
      phoneHeaders: [{ android: true, mobile: "?0", platform: '"Windows"' }],
      phoneReport: { width: 390, height: 560, ua: "phone", uamobile: "false" },
      desktop: switched("desktop", { headers: [{ android: false, mobile: "?1", platform: '"Android"' }], report: { width: 390, height: 560, ua: "desktop", uamobile: "true" } }),
    });
    const failures = siteModeFailures(bare, "win32");
    expect(failures).toHaveLength(4);
    expect(failures[0]).toContain("Windows' phone agent sent");
    expect(failures[1]).toContain("navigator.userAgentData.mobile = false under the phone agent");
    expect(failures[2]).toContain("Windows' desktop agent sent");
    expect(failures[3]).toContain("navigator.userAgentData.mobile = true under the desktop agent");
    // WebKit sends no client hints: nothing to check there.
    expect(siteModeFailures(bare, "linux")).toEqual([]);
  });

  it("make Windows' hints informational when the protocol refused the override", () => {
    const refused = good({ events: log(["ua-override fallback E_ACCESSDENIED"]), phoneHeaders: [{ android: true, mobile: null, platform: null }] });
    expect(overrideRefused(refused.events)).toBe(true);
    expect(siteModeFailures(refused, "win32")).toEqual([]);
  });
});
