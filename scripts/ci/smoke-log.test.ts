import { describe, expect, it } from "vitest";
import {
  count,
  desktopExec,
  evaluate,
  firstRunFailures,
  overlayFailures,
  historyFailures,
  lastTitle,
  newPids,
  parseCodecs,
  parseHistory,
  parseLog,
  parsePersist,
  parseRegValues,
  persistFailures,
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
3 overlay true
4 title Little Joan
4 overlay false
5 bar-url http://127.0.0.1:5000/smoke.html
5 title LJ-SMOKE ready h264=mse vp9=mse av1=no
6 page-loaded http://127.0.0.1:5000/smoke.html
7 fullscreen true
8 title LJ-SMOKE fullscreen=on
9 hibernated
10 window-created
10 overlay true
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
    // A window right after "off" still counts while the check lasts.
    const late = parseLog(`${GOOD}12 start-at-login true\n13 start-at-login false\n14 window-created\n`);
    expect(startAtLoginFailures(late, { afterOn: [entry], afterOff: [], eventsUntil: late.length })).toHaveLength(1);
  });

  it("ignores windows that later steps open", () => {
    const text = `${GOOD}12 start-at-login true\n13 start-at-login false\n`;
    const until = parseLog(text).length;
    const later = parseLog(`${text}20 window-created\n21 title LJ-PERSIST set\n`);
    expect(startAtLoginFailures(later, { afterOn: [entry], afterOff: [], eventsUntil: until })).toEqual([]);
    expect(startAtLoginFailures(later, { afterOn: [entry], afterOff: [] })).toHaveLength(1);
  });
});

describe("the command bar's overlay", () => {
  it("passes when every window opens with it and the smoke page closes it", () => {
    expect(overlayFailures(parseLog(GOOD), true)).toEqual([]);
  });

  it("fails a window that opened without it", () => {
    const toolbar = parseLog(GOOD.replace("10 overlay true\n", "10 overlay false\n"));
    expect(overlayFailures(toolbar, true)).toEqual([expect.stringContaining("window #2 didn't open with the command bar")]);
    const silent = parseLog(GOOD.replace("3 overlay true\n", ""));
    expect(overlayFailures(silent, true)).toEqual([
      "window #1 didn't open with the command bar (first overlay event: overlay false)",
    ]);
  });

  it("fails when opening the smoke page left it open", () => {
    const stuck = parseLog(GOOD.replace("4 overlay false\n", ""));
    expect(overlayFailures(stuck, true)).toEqual([expect.stringContaining('didn\'t close ("overlay false")')]);
    expect(overlayFailures(stuck, false)).toEqual([]);
    // Closing it only after the page was ready doesn't count.
    const late = parseLog(GOOD.replace("4 overlay false\n", "").replace("7 fullscreen true\n", "7 overlay false\n7 fullscreen true\n"));
    expect(overlayFailures(late, true)).toHaveLength(1);
  });

  it("is part of the main run's and the first launch's checks", () => {
    const stuck = parseLog(GOOD.replace("4 overlay false\n", ""));
    expect(evaluate(stuck, { windowFullscreen: false, pageFullscreen: false }, NONE)).toEqual([
      expect.stringContaining('didn\'t close ("overlay false")'),
    ]);
  });
});

describe("first run", () => {
  const first = parseLog(`1 started
2 shortcut-registered CommandOrControl+Shift+Space
3 first-run
4 bar-loaded
4 welcome
5 window-created
5 overlay true
6 title Little Joan
7 hibernated
`);
  const relaunch = parseLog("1 started\n2 shortcut-registered CommandOrControl+Shift+Space\n");

  it("passes a welcome once, then a hibernating relaunch", () => {
    expect(firstRunFailures({ first, relaunch, settingsSaved: true })).toEqual([]);
  });

  it("fails without the welcome, the saved settings, or the window", () => {
    const noWelcome = parseLog("1 started\n2 first-run\n3 window-created\n3 overlay true\n4 bar-loaded\n5 hibernated\n");
    expect(firstRunFailures({ first: noWelcome, relaunch, settingsSaved: true })).toEqual([
      expect.stringContaining('never showed the welcome ("welcome")'),
    ]);
    expect(firstRunFailures({ first, relaunch, settingsSaved: false })).toEqual([
      expect.stringContaining("settings.json wasn't saved"),
    ]);
    const hibernating = parseLog("1 started\n2 shortcut-registered x\n");
    expect(firstRunFailures({ first: hibernating, relaunch, settingsSaved: true }).join("\n")).toContain(
      'missing "first-run"',
    );
  });

  it("fails a first window without the command bar", () => {
    const noOverlay = parseLog("1 started\n2 first-run\n3 window-created\n4 bar-loaded\n4 welcome\n5 hibernated\n");
    expect(firstRunFailures({ first: noOverlay, relaunch, settingsSaved: true })).toEqual([
      expect.stringContaining("first launch: window #1 didn't open with the command bar"),
    ]);
  });

  it("fails when the relaunch shows a window or is a first run again", () => {
    const shown = parseLog("1 started\n2 window-created\n");
    expect(firstRunFailures({ first, relaunch: shown, settingsSaved: true })).toEqual([
      expect.stringContaining("relaunch: a window opened"),
    ]);
    const again = parseLog("1 started\n2 first-run\n3 window-created\n");
    expect(firstRunFailures({ first, relaunch: again, settingsSaved: true })).toHaveLength(2);
    expect(firstRunFailures({ first, relaunch: [], settingsSaved: true })).toEqual([
      expect.stringContaining('relaunch: expected one "started", saw 0'),
    ]);
  });

  it("fails the main run if it was a first run", () => {
    const main = parseLog(`${GOOD}12 first-run\n`);
    expect(evaluate(main, { windowFullscreen: false, pageFullscreen: false }, { pageHelpersLeft: 0 })).toEqual([
      expect.stringContaining("the main run was a first run"),
    ]);
  });
});

describe("persistence", () => {
  const main = parseLog(`${GOOD}20 title LJ-PERSIST set
21 hibernated
22 window-created
23 title LJ-PERSIST p=1 s=1 l=1
30000 title LJ-PERSIST set
30600 quit
`);
  const relaunch = parseLog("1 started\n2 shortcut-registered CommandOrControl+Shift+Space\n3 window-created\n4 title LJ-PERSIST p=1 s=0 l=1\n");
  const good = {
    afterClose: "LJ-PERSIST p=1 s=1 l=1",
    afterQuit: "LJ-PERSIST p=1 s=0 l=1",
    quitExited: true,
    relaunch,
  };

  it("reads the page's report", () => {
    expect(parsePersist("LJ-PERSIST p=1 s=0 l=1")).toEqual({ p: true, s: false, l: true });
    expect(parsePersist("LJ-PERSIST p=0 s=1 l=0")).toEqual({ p: false, s: true, l: false });
    for (const bad of [undefined, "", "LJ-PERSIST set", "LJ-PERSIST p=2 s=0 l=1", "LJ-PERSIST p=1 s=0", "x LJ-PERSIST p=1 s=0 l=1"]) {
      expect(parsePersist(bad), String(bad)).toBeUndefined();
    }
  });

  it("passes everything kept across close, and the session cookie gone after quitting", () => {
    expect(persistFailures(main, good)).toEqual([]);
  });

  it("names what close → summon lost", () => {
    expect(persistFailures(main, { ...good, afterClose: "LJ-PERSIST p=1 s=0 l=1" })).toEqual([
      "close → summon lost the session cookie (p=1 s=0 l=1, want p=1 s=1 l=1)",
    ]);
    expect(persistFailures(main, { ...good, afterClose: "LJ-PERSIST p=0 s=0 l=0" })).toEqual([
      "close → summon lost the persistent cookie, the session cookie, localStorage (p=0 s=0 l=0, want p=1 s=1 l=1)",
    ]);
    expect(persistFailures(main, { ...good, afterClose: undefined })).toEqual([
      "close → summon: the page never reported (got nothing)",
    ]);
  });

  it("names what quitting lost, and a session cookie that outlived the app run", () => {
    expect(persistFailures(main, { ...good, afterQuit: "LJ-PERSIST p=0 s=0 l=0" })).toEqual([
      "quit → relaunch lost the persistent cookie, localStorage (p=0 s=0 l=0, want p=1 s=0 l=1)",
    ]);
    expect(persistFailures(main, { ...good, afterQuit: "LJ-PERSIST p=1 s=1 l=1" })).toEqual([
      "quit → relaunch kept the session cookie (p=1 s=1 l=1); it must end with the app run",
    ]);
    expect(persistFailures(main, { ...good, afterQuit: "LJ-SMOKE ready" })).toEqual([
      "quit → relaunch: the page never reported (got LJ-SMOKE ready)",
    ]);
  });

  it("needs the quit within a second of the set, and the process to end by itself", () => {
    const late = parseLog("1 started\n10 title LJ-PERSIST set\n1011 quit\n");
    expect(persistFailures(late, good)).toEqual([
      "quit → relaunch: the quit came 1001 ms after the set (at most 1000)",
    ]);
    const noQuit = parseLog("1 started\n10 title LJ-PERSIST set\n");
    expect(persistFailures(noQuit, { ...good, quitExited: false })).toEqual([
      'quit → relaunch: the app never logged "quit" after the set',
      "--quit did not end the app's process",
    ]);
    // A quit logged before the last set doesn't count.
    const early = parseLog("1 started\n5 quit\n10 title LJ-PERSIST set\n");
    expect(persistFailures(early, good)).toEqual(['quit → relaunch: the app never logged "quit" after the set']);
    expect(persistFailures(parseLog(GOOD), good)).toEqual(['quit → relaunch: the page never reported "LJ-PERSIST set"']);
  });

  it("needs a plain relaunch on the same data folder", () => {
    expect(persistFailures(main, { ...good, relaunch: parseLog("1 started\n2 first-run\n") })).toEqual([
      "quit → relaunch: it was a first run (the data folder lost its settings)",
    ]);
    expect(persistFailures(main, { ...good, relaunch: [] })).toEqual([
      'quit → relaunch: expected one "started", saw 0',
    ]);
  });
});

describe("history", () => {
  const pageUrl = "http://127.0.0.1:5000/smoke.html";
  const record = (url: string, title = "LJ-SMOKE ready h264=mse", visits = 1) => ({
    url,
    title,
    host: new URL(url).host,
    visitCount: visits,
    visits: Array.from({ length: visits }, (_, i) => ({ at: 1000 + i, kind: "link" })),
  });
  const file = (...records: object[]) => JSON.stringify({ version: 1, entries: records });

  it("reads history.json", () => {
    expect(parseHistory(file(record(pageUrl)))?.[0]?.url).toBe(pageUrl);
    expect(parseHistory(undefined)).toBeUndefined();
    expect(parseHistory("nope")).toBeUndefined();
    expect(parseHistory('{"version": 1}')).toBeUndefined();
  });

  it("passes the smoke page recorded with its title and visits, and nothing with history off", () => {
    const main = file(record(pageUrl, "LJ-SMOKE viewport=720x560 ua=desktop dpr=1", 3), record(`${pageUrl}?persist=set&t=x`));
    expect(historyFailures({ main, pageUrl, off: undefined })).toEqual([]);
    expect(historyFailures({ main, pageUrl, off: file() })).toEqual([]);
  });

  it("fails without the file, the record, its title or a visit", () => {
    expect(historyFailures({ main: undefined, pageUrl, off: undefined })).toEqual(["history.json of the main run is missing"]);
    expect(historyFailures({ main: "{", pageUrl, off: undefined })).toEqual(["history.json of the main run is not history"]);
    expect(historyFailures({ main: file(record("https://other.test/")), pageUrl, off: undefined })).toEqual([
      `history.json has no record of ${pageUrl} (1 records)`,
    ]);
    expect(historyFailures({ main: file(record(pageUrl, "")), pageUrl, off: undefined })).toEqual([
      'history: the smoke page\'s title is "", not the page\'s own',
    ]);
    expect(historyFailures({ main: file(record(pageUrl, "LJ-SMOKE ready", 0)), pageUrl, off: undefined })).toEqual([
      "history: the smoke page has no visit",
    ]);
  });

  it("fails when the app's own pages were recorded, or history off still wrote records", () => {
    const own = file(record(pageUrl), record("http://tauri.localhost/index.html"), record("tauri://localhost/index.html"));
    expect(historyFailures({ main: own, pageUrl, off: undefined })).toEqual([
      "history recorded the app's own pages: http://tauri.localhost/index.html, tauri://localhost/index.html",
    ]);
    expect(historyFailures({ main: file(record(pageUrl)), pageUrl, off: file(record(pageUrl)) })).toEqual([
      "history off: 1 records were still written",
    ]);
  });
});
