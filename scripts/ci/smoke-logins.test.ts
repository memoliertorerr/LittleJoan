import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { loginsFailures, loginsSummary, profiles, switches, type LoginsRun } from "./smoke-logins.mts";

const log = (rows: string[]) => parseLog(rows.map((row, i) => `${1000 + i} ${row}`).join("\n") + "\n");
const PROFILE = "C:\\Users\\runner\\AppData\\Local\\Temp\\lj\\logins\\webview\\EBWebView\\Default";

const FIRST = [
  "started",
  "page-built",
  "logins password-autosave=false autofill=true webview=page",
  `webview-profile page=${PROFILE}`,
  "logins-seen password-autosave=false autofill=true webview=page",
  "title LJ-SMOKE ready h264=mse vp9=mse av1=mse",
  "hibernated",
  `webview-profile settings=${PROFILE}`,
  "logins-seen password-autosave=false autofill=true webview=settings",
  "settings-window privacy",
  "settings-loaded",
  "settings-scrolled logins",
  "settings-logins-click remove",
  "settings-logins-click confirm",
  "saved-logins-cleared ok",
];
const ON = [
  "started",
  "page-built",
  "logins password-autosave=true autofill=true webview=page",
  "popup-opened http://127.0.0.1:5001",
  "logins password-autosave=true autofill=true webview=popup",
  "hibernated ready",
  "settings-loaded",
  "logins-seen password-autosave=true autofill=true webview=settings",
];

function run(over: Partial<{ first: string[]; on: string[]; popupOpened: boolean }> = {}): LoginsRun {
  return { first: log(over.first ?? FIRST), on: log(over.on ?? ON), popupOpened: over.popupOpened ?? true };
}

/** macOS and Linux: the same runs without anything of the engine's switches. */
const MAC_FIRST = ["started", "page-built", "title LJ-SMOKE ready", "hibernated", "settings-window privacy", "settings-loaded", "settings-scrolled logins"];

describe("the logins runs' rules (LJ51)", () => {
  it("passes on Windows when the switches read back, the profile is shared and removing answered ok", () => {
    expect(loginsFailures(run(), "win32")).toEqual([]);
  });

  it("wants the default page to read back saving off and autofill on", () => {
    const on = run({ first: FIRST.map((row) => row.replace("logins password-autosave=false", "logins password-autosave=true")) });
    expect(loginsFailures(on, "win32").join()).toContain('not "password-autosave=false autofill=true webview=page"');
    const failed = run({ first: [...FIRST, "logins failed webview=page No such interface supported"] });
    expect(loginsFailures(failed, "win32").join()).toContain("switching failed: failed webview=page");
  });

  it("stops on two profile folders: removing wouldn't reach what the page saved", () => {
    const other = run({ first: FIRST.map((row) => (row.startsWith("webview-profile settings=") ? "webview-profile settings=C:\\elsewhere\\EBWebView\\Default" : row)) });
    expect(loginsFailures(other, "win32").join()).toContain("isn't the page's");
    const unknown = run({ first: FIRST.map((row) => (row.startsWith("webview-profile page=") ? "webview-profile page=unknown No such interface" : row)) });
    expect(loginsFailures(unknown, "win32").join()).toContain("no profile folder for both");
    const none = run({ first: FIRST.filter((row) => !row.startsWith("webview-profile settings=")) });
    expect(loginsFailures(none, "win32").join()).toContain("no profile folder for both");
  });

  it("wants the removal to answer ok, after the window hibernated, with no page built", () => {
    const failed = run({ first: FIRST.map((row) => (row === "saved-logins-cleared ok" ? "saved-logins-cleared failed Windows' web engine didn't answer." : row)) });
    expect(loginsFailures(failed, "win32").join()).toContain("not ok");
    const missing = run({ first: FIRST.filter((row) => !row.startsWith("saved-logins-cleared")) });
    expect(loginsFailures(missing, "win32").join()).toContain("logged no saved-logins-cleared");
    const withPage = run({ first: FIRST.flatMap((row) => (row === "settings-loaded" ? ["page-built", row] : [row])) });
    expect(loginsFailures(withPage, "win32").join()).toContain("didn't happen while hibernating");
    const noHibernate = run({ first: FIRST.filter((row) => row !== "hibernated") });
    expect(loginsFailures(noHibernate, "win32").join()).toContain("didn't open after the window hibernated");
  });

  it("wants savePasswords to reach the page and the popup", () => {
    const pageOff = run({ on: ON.map((row) => (row.endsWith("webview=page") ? "logins password-autosave=false autofill=true webview=page" : row)) });
    expect(loginsFailures(pageOff, "win32").join()).toContain("with savePasswords the page read back");
    const popupOff = run({ on: ON.filter((row) => !row.endsWith("webview=popup")) });
    expect(loginsFailures(popupOff, "win32").join()).toContain("the popup read back");
    expect(loginsFailures(run({ popupOpened: false }), "win32").join()).toContain("never opened");
  });

  it("wants nothing switched or removed on macOS and Linux, and the screenshot's scroll everywhere", () => {
    const quiet = run({ first: MAC_FIRST, on: ["started", "page-built", "popup-opened http://127.0.0.1:5001", "settings-loaded"], popupOpened: false });
    expect(loginsFailures(quiet, "darwin")).toEqual([]);
    expect(loginsFailures(quiet, "linux")).toEqual([]);
    expect(loginsFailures(run(), "linux").join()).toContain("has no switch");
    const noScroll = run({ first: MAC_FIRST.filter((row) => row !== "settings-scrolled logins") });
    expect(loginsFailures(noScroll, "darwin").join()).toContain("never scrolled");
    expect(loginsFailures(run({ first: FIRST.filter((row) => row !== "settings-scrolled logins") }), "win32").join()).toContain("never scrolled");
  });

  it("reads the profile lines and the switches, and sums them up", () => {
    expect(profiles(log(FIRST))).toEqual({ page: PROFILE, settings: PROFILE });
    expect(profiles(log(["webview-profile page=a=b", "webview-profile nonsense"]))).toEqual({ page: "a=b" });
    expect(switches(log(ON))).toEqual(["password-autosave=true autofill=true webview=page", "password-autosave=true autofill=true webview=popup"]);
    expect(switches(log(ON), "logins-seen")).toEqual(["password-autosave=true autofill=true webview=settings"]);
    const summary = loginsSummary(run(), "win32");
    expect(summary).toMatchObject({ sameProfile: true, cleared: "ok", popupOpened: true, rules: [] });
    expect(summary.settingsSaw).toEqual(["password-autosave=true autofill=true webview=settings"]);
    expect(loginsSummary(run({ first: MAC_FIRST }), "darwin")).toMatchObject({ sameProfile: false, cleared: null, profiles: {} });
  });
});
