import { describe, expect, it } from "vitest";
import {
  count,
  desktopExec,
  evaluate,
  lastTitle,
  newPids,
  parseCodecs,
  parseLog,
  parseRegValues,
  plistProgram,
  pointsAt,
  startAtLoginFailures,
  survivors,
  sawViewport,
  type LogEvent,
} from "./smoke-log.mts";

const GOOD = `1 started
2 shortcut-registered CommandOrControl+Shift+Space
3 bar-loaded
3 window-created
4 title Little Joan
5 bar-url http://127.0.0.1:5000/smoke.html
5 title LJ-SMOKE ready h264=mse vp9=mse av1=no
6 page-loaded http://127.0.0.1:5000/smoke.html
7 fullscreen true
8 title LJ-SMOKE fullscreen=on
9 hibernated
10 window-created
11 hibernated
`;

const NONE = { pageHelpersLeft: 0 };

describe("parseLog", () => {
  it("reads events with and without details, skipping junk", () => {
    const events = parseLog("12 started\n13 title A  b\n\nnot a line\n14 hibernated\r\n");
    expect(events).toEqual<LogEvent[]>([
      { millis: 12, event: "started", detail: "" },
      { millis: 13, event: "title", detail: "A  b" },
      { millis: 14, event: "hibernated", detail: "" },
    ]);
  });
});

describe("queries", () => {
  const events = parseLog(GOOD);

  it("counts and finds titles", () => {
    expect(count(events, "window-created")).toBe(2);
    expect(count(events, "fullscreen", "true")).toBe(1);
    expect(lastTitle(events, "LJ-SMOKE ready")).toBe("LJ-SMOKE ready h264=mse vp9=mse av1=no");
    expect(lastTitle(events, "nope")).toBeUndefined();
  });

  it("parses codecs from the ready title", () => {
    expect(parseCodecs(lastTitle(events, "LJ-SMOKE ready"))).toEqual({ h264: "mse", vp9: "mse", av1: "no" });
    expect(parseCodecs(undefined)).toEqual({});
  });
});

describe("evaluate", () => {
  it("passes a complete run", () => {
    expect(evaluate(parseLog(GOOD), { windowFullscreen: true, pageFullscreen: true }, NONE)).toEqual([]);
  });

  it("reports missing steps and a second instance", () => {
    const failures = evaluate(
      parseLog("1 started\n2 started\n3 window-created\n"),
      { windowFullscreen: false, pageFullscreen: false },
      NONE,
    );
    expect(failures.join("\n")).toContain('missing "shortcut-registered"');
    expect(failures.join("\n")).toContain('exactly one "started"');
    expect(failures.join("\n")).toContain("second window");
  });

  it("requires fullscreen only where expected", () => {
    const noFullscreen = parseLog(GOOD.replace("7 fullscreen true\n", "").replace("8 title LJ-SMOKE fullscreen=on\n", ""));
    expect(evaluate(noFullscreen, { windowFullscreen: false, pageFullscreen: false }, NONE)).toEqual([]);
    expect(evaluate(noFullscreen, { windowFullscreen: true, pageFullscreen: false }, NONE)).toHaveLength(1);
    expect(evaluate(noFullscreen, { windowFullscreen: false, pageFullscreen: true }, NONE)).toHaveLength(1);
  });

  it("requires the URL bar to load and to show the page's address", () => {
    const noBar = parseLog(GOOD.replace("3 bar-loaded\n", "").replace("5 bar-url http://127.0.0.1:5000/smoke.html\n", ""));
    const failures = evaluate(noBar, { windowFullscreen: false, pageFullscreen: false }, NONE);
    expect(failures).toEqual([
      expect.stringContaining("bar never loaded"),
      expect.stringContaining("smoke page's address"),
    ]);
  });

  it("fails when the page's webview helpers outlive the window", () => {
    const failures = evaluate(parseLog(GOOD), { windowFullscreen: false, pageFullscreen: false }, {
      pageHelpersLeft: 3,
    });
    expect(failures).toEqual([expect.stringContaining("helper processes of the page left")]);
  });
});

describe("viewports", () => {
  const log = parseLog(`1 title LJ-SMOKE viewport=390x560 ua=phone dpr=1
2 title LJ-SMOKE viewport=900x560 ua=phone dpr=1
3 title LJ-SMOKE viewport=900x560 ua=desktop dpr=1
`);

  it("matches size and agent kind exactly", () => {
    expect(sawViewport(log, { width: 390, height: 560, ua: "phone" })).toBe(true);
    expect(sawViewport(log, { width: 900, height: 560, ua: "desktop" })).toBe(true);
    expect(sawViewport(log, { width: 390, height: 560, ua: "desktop" })).toBe(false);
    expect(sawViewport(log, { width: 39, height: 560, ua: "phone" })).toBe(false);
  });

  it("fails a run whose page never reflowed to the window", () => {
    const failures = evaluate(
      parseLog(GOOD),
      { windowFullscreen: false, pageFullscreen: false, viewports: [{ width: 390, height: 560, ua: "phone" }] },
      NONE,
    );
    expect(failures).toEqual([expect.stringContaining("never saw a 390x560 viewport with a phone user agent")]);
  });
});

describe("helper attribution", () => {
  it("counts only helpers that appeared while the page was shown", () => {
    // 10 belongs to another app and stays; 21 and 22 served the page.
    const ours = newPids([10], [10, 21, 22]);
    expect(ours).toEqual([21, 22]);
    // After hibernating: 22 is gone, 30 and 31 are another app's new helpers.
    expect(survivors(ours, [10, 21, 30, 31])).toBe(1);
    expect(survivors(ours, [10, 30, 31])).toBe(0);
  });
});

describe("login entries", () => {
  it("reads values from reg query output, names with spaces included", () => {
    const output = [
      "",
      "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
      "    OneDrive    REG_SZ    \"C:\\Program Files\\OneDrive\\OneDrive.exe\" /background",
      "    Little Joan    REG_SZ    C:\\Users\\runneradmin\\AppData\\Local\\Little Joan\\little-joan.exe ",
      "",
    ].join("\r\n");
    expect(parseRegValues(output)).toEqual([
      { name: "OneDrive", type: "REG_SZ", data: '"C:\\Program Files\\OneDrive\\OneDrive.exe" /background' },
      {
        name: "Little Joan",
        type: "REG_SZ",
        data: "C:\\Users\\runneradmin\\AppData\\Local\\Little Joan\\little-joan.exe ",
      },
    ]);
    expect(parseRegValues("ERROR: The system was unable to find the specified registry key or value.")).toEqual([]);
  });

  it("reads the program of a LaunchAgent plist (auto-launch's format)", () => {
    const plist = `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
  <key>Label</key>
  <string>Little Joan</string>
  <key>ProgramArguments</key>
  <array><string>/Users/runner/work/_temp/Applications/Little Joan.app/Contents/MacOS/little-joan</string></array>
  <key>RunAtLoad</key>
  <true/>
  </dict>
</plist>`;
    expect(plistProgram(plist)).toBe("/Users/runner/work/_temp/Applications/Little Joan.app/Contents/MacOS/little-joan");
    expect(plistProgram("<plist><dict><key>Program</key><string>/x</string></dict></plist>")).toBeUndefined();
    expect(plistProgram("<key>ProgramArguments</key><array><string>/a &amp; b</string></array>")).toBe("/a & b");
  });

  it("reads Exec= from an autostart .desktop file", () => {
    const desktop = "[Desktop Entry]\nType=Application\nName=Little Joan\nExec=/usr/bin/little-joan \nTerminal=false";
    expect(desktopExec(desktop)).toBe("/usr/bin/little-joan");
    expect(desktopExec("[Desktop Entry]\nName=x")).toBeUndefined();
  });

  it("matches the installed binary, quoted or not, with or without arguments", () => {
    const exe = "C:\\Users\\r\\AppData\\Local\\Little Joan\\little-joan.exe";
    expect(pointsAt(`${exe} `, [exe], true)).toBe(true);
    expect(pointsAt(`"${exe}" --flag`, [exe], true)).toBe(true);
    expect(pointsAt(exe.toUpperCase(), [exe], true)).toBe(true);
    expect(pointsAt(exe.toUpperCase(), [exe], false)).toBe(false);
    expect(pointsAt(`${exe}x`, [exe], true)).toBe(false);
    expect(pointsAt("C:\\Other\\little-joan.exe", [exe], true)).toBe(false);
    expect(pointsAt("/private/tmp/x/little-joan", ["/tmp/x/little-joan", "/private/tmp/x/little-joan"], false)).toBe(true);
  });
});

describe("start at login", () => {
  const entry = { where: "Little Joan.desktop", command: "/usr/bin/little-joan" };
  const log = parseLog(`${GOOD}12 start-at-login true\n13 start-at-login false\n`);

  it("passes when the entry appears after on and is gone after off", () => {
    expect(startAtLoginFailures(log, { afterOn: [entry], afterOff: [] })).toEqual([]);
    expect(evaluate(log, { windowFullscreen: false, pageFullscreen: false }, { pageHelpersLeft: 0, loginEntry: { afterOn: [entry], afterOff: [] } })).toEqual([]);
  });

  it("fails without the read-back events, in order", () => {
    const reversed = parseLog(`${GOOD}12 start-at-login false\n13 start-at-login true\n`);
    expect(startAtLoginFailures(reversed, { afterOn: [entry], afterOff: [] })).toEqual([
      expect.stringContaining('"start-at-login true", then "start-at-login false"'),
    ]);
    expect(startAtLoginFailures(parseLog(GOOD), { afterOn: [entry], afterOff: [] })).toHaveLength(1);
  });

  it("fails when the entry is missing, left behind, or the app reported an error", () => {
    expect(startAtLoginFailures(log, { afterOn: [], afterOff: [] })).toEqual([expect.stringContaining("no login entry")]);
    expect(startAtLoginFailures(log, { afterOn: [entry], afterOff: [entry] })).toEqual([
      expect.stringContaining("left after --start-at-login off: Little Joan.desktop"),
    ]);
    const failed = parseLog(`${GOOD}12 start-at-login-failed Move Little Joan to Applications first, then turn this on.\n`);
    expect(startAtLoginFailures(failed, { afterOn: [], afterOff: [] }).join("\n")).toContain(
      "start at login failed: Move Little Joan to Applications first",
    );
  });

  it("fails when the flag opened a window", () => {
    const opened = parseLog(`${GOOD}12 start-at-login true\n13 window-created\n14 start-at-login false\n`);
    expect(startAtLoginFailures(opened, { afterOn: [entry], afterOff: [] })).toEqual([
      expect.stringContaining("opened a window"),
    ]);
  });
});
