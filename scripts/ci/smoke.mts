// CI smoke test of the INSTALLED app (what users download, not the build folder).
//
//   node scripts/ci/smoke.mts --app <installed binary> --out <dir>            full run
//   node scripts/ci/smoke.mts --app <binary> --out <dir> --started-only      launch check only
//
// The full run: serve tests/smoke/ on 127.0.0.1, launch the app with LITTLEJOAN_SMOKE_LOG,
// summon it with `--show` (single-instance forwarding; the command bar covers the window), load
// the smoke page (the command bar closes: the 40 px bar above the page), click its big
// fullscreen button, resize the window (phone and desktop agents), zoom the page with Ctrl+= and
// back with Ctrl+0 (smoke-zoom.mts), hibernate with `--hide`, summon and hibernate once more, summon again for
// the command bar (the smoke page's site as a top site, then typing "127", then Ctrl+H — Cmd+Y on
// macOS — for the history view, and Ctrl+Y opening nothing), turn start at login on and off (reading the OS's own login entry each
// time), check that cookies and localStorage outlive closing the window and quitting (`--quit`,
// then a relaunch on the same data folder), that history.json recorded the smoke page (and that
// a run with history off records nothing; that run also opens Settings with `--settings` and
// `--settings privacy`), then the extension run (its own launch: `--extension hello`, then the
// smoke page's probe of what a web page can't do to an extension), then the bundled extensions
// run (its own launch: `--extension <id>` for each bundled panel, screenshots), then the theme runs (each its
// own launch with a theme in settings.json:
// the page, the command bar, the history view and Settings in a two-color theme, light and dark,
// and the default theme in dark; screenshots in <out>/theme/), then the fullscreen run (its own
// launch: the smoke page's ?fs=both buttons clicked at 390 and 720 px, screenshots in <out>/fs/;
// smoke-fullscreen.mts), then the resize run (its own launch in a dark theme: bursts of `--size`
// 380 ↔ 820 px 30 ms apart and the phone-mode hysteresis, screenshots resize-380.png and
// resize-820.png; smoke-resize.mts), then the site-mode run (its own launch: the toolbar's phone
// pill, the command bar's "Request desktop site" and "Automatic for this site", Settings' list,
// Windows' client hints from the server's agent-headers.jsonl; smoke-sitemode.mts), then the
// quick-commands run (its own launch: "mobile" and "dark" in the command bar change and save those
// settings, the theme rows, Copy link over the smoke page; smoke-quick.mts), then the keymap run
// (its own launch with History on Ctrl/Cmd+Shift+K: the new key works, the old one doesn't, the
// macOS menu shows it; smoke-keymap.mts), then the downloads run (its own launch: a download named
// by its Content-Disposition lands in the OS's Downloads folder with its exact bytes, the same one
// again as "lj-smoke (1).bin", none while an extension's panel shows; download.png;
// smoke-downloads.mts), then the tray
// run (its own launches with the icon switched off: it hides, a plain second launch opens the
// window, the command bar's "Quit Little Joan" quits, Settings → General shows the switch, and on
// Linux a Wayland session brings the icon back; smoke-tray.mts), then the summon-timing run (its own
// launch without gdb, a seeded history: first, warm, idle and cold summons — the file cache emptied
// with sudo on macOS and Linux —, through `--show` and the real global shortcut; timing.md;
// smoke-timing.mts), then the over-full-screen run (another app's full-screen window first — a
// Swift app in native full screen on macOS, a borderless WinForms window on Windows, an X11
// _NET_WM_STATE_FULLSCREEN window on Linux —, then Little Joan summoned over it through `--show`,
// the real shortcut and `--settings`: in front with the keyboard, on the same Space;
// over-fullscreen*.png; smoke-spaces.mts), then the rest runs (LJ19.4, smoke-rest.mts: one launch per
// "After closing" mode — what stays at rest, a pinging page stopping at the close, memory at rest,
// keys typed right after the shortcut, the ready bar's web process killed), then check the logs
// (scripts/ci/smoke-log.mts, smoke-command.mts, smoke-theme.mts) and the webview helper processes.
// The app runs on a fresh LITTLEJOAN_DATA_DIR that already holds the default settings.json, so it
// starts at rest as every launch after the first does — since LJ19.4 that is "Stay ready", the
// default: the hidden window and the bar built a few seconds after the launch. Last, the first-launch check: quit, launch on an empty
// data folder (the window opens once with the welcome), --hide, quit, relaunch on the same
// folder (it must hibernate). Screenshots and a summary land in <out>. Linux runs inside
// xvfb-run + dbus-run-session with a window manager.
import { spawn, execFileSync, type ChildProcess } from "node:child_process";
import { appendFileSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { createServer, type Server, type ServerResponse } from "node:http";
import type { AddressInfo } from "node:net";
import { homedir, tmpdir } from "node:os";
import { extname, join, resolve } from "node:path";
import {
  count,
  desktopExec,
  evaluate,
  firstRunFailures,
  historyFailures,
  lastTitle,
  newPids,
  parseCodecs,
  parseHistory,
  parseLog,
  parseRegValues,
  persistFailures,
  plistProgram,
  pointsAt,
  sawViewport,
  survivors,
  type LoginEntry,
  type LogEvent,
  type Viewport,
} from "./smoke-log.mts";
import {
  commandBarFailures,
  historyViewFailures,
  sawHistoryView,
  sawSettings,
  sawSmokeTopSite,
  sawTypedMatch,
  settingsFailures,
  settingsTabs,
  TYPED,
} from "./smoke-command.mts";
import { THEME_RUNS, parseTheme, themeFailures, type ThemeRun } from "./smoke-theme.mts";
import { macKeyToken } from "./smoke-keys.mts";
import {
  BROKEN_EXTENSION,
  DEV_EXTENSION,
  barReadyFailures,
  commandHelloFailures,
  extensionFailures,
  helloOrigin,
  raceFailures,
  parseStore,
  settingsExtensionsFailures,
} from "./smoke-extensions.mts";
import { BUNDLED_PANELS, PAGE_ACTIONS, bundledFailures, keysReached, pageActionFailures, panelUp, seedFile } from "./smoke-bundled.mts";
import { fullscreenFailures, fullscreenRun, fullscreenSummary, type FsDriver } from "./smoke-fullscreen.mts";
import { RESIZE_STEPS, resizeFailures, resizeRun, resizeSummary } from "./smoke-resize.mts";
import { parseAgentLog, siteModeFailures, siteModeRun, siteModeSummary, type SiteApp, type SiteDriver } from "./smoke-sitemode.mts";
import { trayFailures, trayRun, traySummary, type TrayDriver } from "./smoke-tray.mts";
import { findApp, listProcs } from "./smoke-linux.mts";
import { fullscreenOffFailures, fullscreenOffVersions } from "./smoke-fullscreen-off.mts";
import { quickFailures, quickRun, quickSummary, type QuickDriver } from "./smoke-quick.mts";
import { summonTimings, timingFailures, timingRun, timingSummary, timingTable, type TimingDriver } from "./smoke-timing.mts";
import { spacesFailures, spacesRun, spacesSummary } from "./smoke-spaces.mts";
import { restFailures, restRun, restSummary, restTable, type EngineProc, type RestRun } from "./smoke-rest.mts";
import { ZOOMED, zoomFailures, zoomReset, zoomSummary, zoomedViewport } from "./smoke-zoom.mts";
import {
  DOWNLOAD_NAME,
  DOWNLOAD_PATH,
  PDF_BODY,
  PDF_NAME,
  PDF_PATH,
  SLOW_CHUNK,
  SLOW_CHUNKS,
  SLOW_EVERY_MS,
  SLOW_NAME,
  SLOW_PATH,
  attachmentHeaders,
  downloadBody,
  downloadFailures,
  downloadRun,
  downloadSummary,
  type SlowServe,
} from "./smoke-downloads.mts";
import {
  keyboardProblemFailures,
  keyboardProblemRun,
  keymapFailures,
  keymapRun,
  keymapSummary,
  systemShortcutFailures,
  type KeymapDriver,
} from "./smoke-keymap.mts";

const root = resolve(import.meta.dirname, "..", "..");
const pageDir = join(root, "tests", "smoke");
/** How many extensions the app bundles: the repository's extensions/<id>/ with a manifest. */
const bundledCount = readdirSync(join(root, "extensions")).filter((name) => existsSync(join(root, "extensions", name, "extension.json"))).length;
const os = process.platform;

function arg(name: string): string | undefined {
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

const app = arg("--app");
const out = resolve(arg("--out") ?? "test-results/smoke");
const startedOnly = process.argv.includes("--started-only");
if (!app || !existsSync(app)) {
  console.error(`usage: smoke.mts --app <installed binary> --out <dir>  (got ${app ?? "nothing"})`);
  process.exit(2);
}
mkdirSync(out, { recursive: true });
const logPath = join(out, startedOnly ? "smoke-started-only.log" : "smoke.log");
rmSync(logPath, { force: true });
// LJ_SMOKE_EXTRA_ENV="KEY=VALUE KEY2=VALUE2" adds variables to the app's environment (CI
// experiments, e.g. WebKitGTK renderer switches); it is recorded in the summary.
const extraEnv = Object.fromEntries(
  (process.env.LJ_SMOKE_EXTRA_ENV ?? "")
    .split(/\s+/)
    .filter((pair) => pair.includes("="))
    .map((pair) => [pair.slice(0, pair.indexOf("=")), pair.slice(pair.indexOf("=") + 1)]),
);
// LJ_SMOKE_FULLSCREEN=off: this job's engine has element fullscreen switched off (WebKitGTK older
// than 2.52, which aborts on it: LJ14.1), so a page's request must change nothing — and crash
// nothing (smoke-fullscreen-off.mts).
const fullscreenOff = process.env.LJ_SMOKE_FULLSCREEN === "off";
// Data folders live outside <out>, which is uploaded (WebView2 keeps its profile there).
const dataRoot = process.env.RUNNER_TEMP ?? tmpdir();
/**
 * A fresh data folder; with `settings`, it already holds a default settings.json (plus
 * `overrides`).
 */
function dataDir(name: string, settings: boolean, overrides: Record<string, unknown> = {}): string {
  const dir = mkdtempSync(join(dataRoot, `lj-${name}-`));
  if (settings) {
    const defaults = {
      version: 1,
      shortcut: "CommandOrControl+Shift+Space",
      searchEngine: "google",
      translateTo: "en",
      windowWidth: 480,
      windowHeight: 640,
      mobileSites: true,
      ...overrides,
    };
    writeFileSync(join(dir, "settings.json"), `${JSON.stringify(defaults, null, 2)}\n`);
  }
  return dir;
}

/**
 * Puts two developer extensions into `dir`'s extensions folder: a copy of the bundled hello under
 * the id `dev-hello`, which must load, and one with a bad manifest, which must be skipped.
 */
function writeDeveloperExtensions(dir: string): void {
  const hello = join(root, "extensions", "hello");
  const copy = join(dir, "extensions", DEV_EXTENSION);
  mkdirSync(copy, { recursive: true });
  for (const name of readdirSync(hello)) {
    writeFileSync(join(copy, name), readFileSync(join(hello, name)));
  }
  const manifest = JSON.parse(readFileSync(join(hello, "extension.json"), "utf8")) as Record<string, unknown>;
  writeFileSync(join(copy, "extension.json"), JSON.stringify({ ...manifest, id: DEV_EXTENSION, name: "Hello (unpacked)" }, null, 2));
  const broken = join(dir, "extensions", BROKEN_EXTENSION);
  mkdirSync(broken, { recursive: true });
  writeFileSync(join(broken, "extension.json"), JSON.stringify({ id: BROKEN_EXTENSION, name: "Broken", version: "1", apiVersion: 2 }));
}

/** A file's text, or undefined when it doesn't exist (or can't be read). */
function readIfThere(path: string): string | undefined {
  try {
    return existsSync(path) ? readFileSync(path, "utf8") : undefined;
  } catch {
    return undefined;
  }
}
const env = {
  ...process.env,
  // A panic in the app prints a backtrace into the job log (every launch inherits this env).
  RUST_BACKTRACE: "1",
  ...extraEnv,
  LITTLEJOAN_SMOKE_LOG: logPath,
  LITTLEJOAN_DATA_DIR: dataDir(startedOnly ? "started-only" : "main", true),
};

const sleep = (ms: number) => new Promise((done) => setTimeout(done, ms));

function events(path = logPath): LogEvent[] {
  return existsSync(path) ? parseLog(readFileSync(path, "utf8")) : [];
}

async function waitFor(
  what: string,
  test: (e: LogEvent[]) => boolean,
  timeoutMs = 30_000,
  path = logPath,
  pollMs = 250,
): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (test(events(path))) {
      console.log(`ok: ${what}`);
      return true;
    }
    await sleep(pollMs);
  }
  console.log(`TIMEOUT (${timeoutMs} ms): ${what}`);
  return false;
}

function run(command: string, args: string[]): string {
  try {
    return execFileSync(command, args, { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"], timeout: 60_000 });
  } catch (err) {
    const e = err as { stdout?: string; stderr?: string; message: string };
    console.log(`(command failed: ${command} ${args.join(" ")}: ${e.stderr ?? e.message})`);
    return e.stdout ?? "";
  }
}

function powershell(script: string): string {
  return run("powershell", ["-NoProfile", "-NonInteractive", "-Command", script]);
}

function screenshot(name: string): void {
  const file = join(out, name);
  if (os === "win32") {
    powershell(`Add-Type -AssemblyName System.Windows.Forms, System.Drawing
$b = [System.Windows.Forms.SystemInformation]::VirtualScreen
$bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($b.Left, $b.Top, 0, 0, $bmp.Size)
$bmp.Save('${file}', [System.Drawing.Imaging.ImageFormat]::Png)`);
  } else if (os === "darwin") {
    run("screencapture", ["-x", file]);
  } else {
    run("import", ["-window", "root", file]);
  }
  console.log(existsSync(file) ? `screenshot: ${name}` : `screenshot FAILED: ${name}`);
}

/**
 * Clicks a point on the screen — pixels on Windows (SetCursorPos) and Linux (xdotool), points on
 * macOS (CGEvent) — or the centre of the primary screen ("centre": its work area on Windows).
 */
function clickAt(target: { x: number; y: number } | "centre"): void {
  const point = target === "centre" ? undefined : target;
  if (os === "win32") {
    const where = point ? `${point.x}, ${point.y}` : "$a.Left + [int]($a.Width / 2), $a.Top + [int]($a.Height / 2)";
    powershell(`Add-Type -AssemblyName System.Windows.Forms
Add-Type -Namespace LJ -Name Mouse -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y); [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint x, uint y, uint d, System.UIntPtr e);'
$a = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea
[LJ.Mouse]::SetCursorPos(${where}) | Out-Null
Start-Sleep -Milliseconds 200
[LJ.Mouse]::mouse_event(0x0002, 0, 0, 0, [System.UIntPtr]::Zero)
Start-Sleep -Milliseconds 80
[LJ.Mouse]::mouse_event(0x0004, 0, 0, 0, [System.UIntPtr]::Zero)`);
  } else if (os === "darwin") {
    const swift = join(out, "click.swift");
    writeFileSync(
      swift,
      `import AppKit
let f = NSScreen.main!.frame
let args = CommandLine.arguments.dropFirst().compactMap { Double($0) }
let p = args.count == 2 ? CGPoint(x: args[0], y: args[1]) : CGPoint(x: f.midX, y: f.midY)
for t in [CGEventType.leftMouseDown, CGEventType.leftMouseUp] {
  let event = CGEvent(mouseEventSource: nil, mouseType: t, mouseCursorPosition: p, mouseButton: .left)
  event?.flags = []
  event?.post(tap: .cghidEventTap)
  usleep(80000)
}
`,
    );
    run("swift", [swift, ...(point ? [String(point.x), String(point.y)] : [])]);
  } else if (point) {
    run("xdotool", ["mousemove", String(point.x), String(point.y), "click", "1"]);
  } else {
    const [width, height] = run("xdotool", ["getdisplaygeometry"]).trim().split(/\s+/).map(Number);
    run("xdotool", ["mousemove", String(Math.round((width ?? 1280) / 2)), String(Math.round((height ?? 800) / 2)), "click", "1"]);
  }
  console.log(point ? `clicked ${point.x},${point.y}` : "clicked the screen centre");
}

/** Clicks the centre of the primary screen, where the smoke page's big button sits. */
function clickCentre(): void {
  clickAt("centre");
}

/**
 * Types `text` into the focused window: xdotool on Linux, SendKeys on Windows (after bringing the
 * app's window to the front — unless `activate` is false: activating a window that is already in
 * front can hand the keyboard back to the page webview after the app moved it to the bar), key
 * events from a small Swift script on macOS (best effort: the runner may not let them through).
 */
function typeText(text: string, appPid: number, activate = true): void {
  if (os === "win32") {
    powershell(`Add-Type -AssemblyName System.Windows.Forms
${activate ? `$shell = New-Object -ComObject WScript.Shell
$null = $shell.AppActivate(${appPid})
Start-Sleep -Milliseconds 400` : "Start-Sleep -Milliseconds 100"}
[System.Windows.Forms.SendKeys]::SendWait('${text}')`);
  } else if (os === "darwin") {
    // Real key codes (US layout) where there is one — events that only carry a Unicode string
    // never reached the app on CI's macOS runners —, the Unicode string otherwise.
    const swift = join(out, "type.swift");
    writeFileSync(
      swift,
      `import AppKit
let pid = pid_t(CommandLine.arguments[1]) ?? 0
NSRunningApplication(processIdentifier: pid)?.activate(options: [])
usleep(400000)
for token in CommandLine.arguments.dropFirst(2) {
  let kind = token.first
  let rest = String(token.dropFirst())
  for down in [true, false] {
    if kind == "k" {
      let event = CGEvent(keyboardEventSource: nil, virtualKey: CGKeyCode(rest.filter { $0.isNumber }) ?? 0, keyDown: down)
      // Explicit flags: an event made from no source takes the session's, and a Command left
      // down by a synthetic Cmd+L turned a typed "q" into Cmd+Q (LJ40's CI).
      event?.flags = rest.hasSuffix("s") ? .maskShift : []
      event?.post(tap: .cghidEventTap)
    } else {
      let event = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: down)
      event?.flags = []
      var units = Array(rest.utf16)
      event?.keyboardSetUnicodeString(stringLength: units.count, unicodeString: &units)
      event?.post(tap: .cghidEventTap)
    }
    usleep(60000)
  }
}
`,
    );
    run("swift", [swift, String(appPid), ...[...text].map(macKeyToken)]);
  } else {
    run("xdotool", ["type", "--delay", "120", text]);
  }
  console.log(`typed ${JSON.stringify(text)}`);
}

/** macOS virtual key codes of the keys `pressKeys` sends. */
const MAC_NAMED_KEYS: Record<string, number> = { Left: 123, Right: 124, Down: 125, Up: 126, Return: 36, Escape: 53 };
const SEND_KEYS: Record<string, string> = { Left: "{LEFT}", Right: "{RIGHT}", Down: "{DOWN}", Up: "{UP}", Return: "{ENTER}", Escape: "{ESC}" };

/** Presses named keys (arrows, Return) in the app's window, as typeText does (best effort on macOS). */
function pressKeys(keys: string[], appPid: number, activate = true): void {
  if (os === "win32") {
    powershell(`${activate ? `$shell = New-Object -ComObject WScript.Shell
$null = $shell.AppActivate(${appPid})
Start-Sleep -Milliseconds 400` : "Start-Sleep -Milliseconds 100"}
Add-Type -AssemblyName System.Windows.Forms
${keys.map((key) => `[System.Windows.Forms.SendKeys]::SendWait('${SEND_KEYS[key] ?? ""}'); Start-Sleep -Milliseconds 150`).join("\n")}`);
  } else if (os === "darwin") {
    const swift = join(out, "keys.swift");
    writeFileSync(
      swift,
      `import AppKit
let pid = pid_t(CommandLine.arguments[1]) ?? 0
NSRunningApplication(processIdentifier: pid)?.activate(options: [])
usleep(400000)
for code in CommandLine.arguments.dropFirst(2) {
  for down in [true, false] {
    let event = CGEvent(keyboardEventSource: nil, virtualKey: CGKeyCode(code) ?? 0, keyDown: down)
    event?.flags = []
    event?.post(tap: .cghidEventTap)
    usleep(60000)
  }
  usleep(150000)
}
`,
    );
    run("swift", [swift, String(appPid), ...keys.map((key) => String(MAC_NAMED_KEYS[key] ?? 0))]);
  } else {
    run("xdotool", ["key", "--delay", "150", ...keys]);
  }
  console.log(`pressed ${keys.join(" ")}`);
}

/** macOS virtual key codes (kVK_ANSI_*) of the keys `pressShortcut` sends. */
const MAC_KEYS: Record<string, number> = { y: 16, l: 37, h: 4, k: 40, "=": 24, "0": 29 };
/** xdotool's names for the keys `pressShortcut` sends that aren't letters. */
const XDOTOOL_KEYS: Record<string, string> = { "=": "equal" };
/** The history view's key: Ctrl+H on Windows and Linux, Cmd+Y on macOS (LJ37). */
const HISTORY_KEY = os === "darwin" ? "y" : "h";

/**
 * Presses Ctrl+<letter> (Cmd+<letter> on macOS), with Shift when asked, in the app's window:
 * xdotool on Linux, SendKeys on Windows (after bringing the window to the front), a key event
 * with the Command (and Shift) flag from a small Swift script on macOS (best effort, as for
 * typing).
 */
function pressShortcut(letter: string, appPid: number, shift = false): void {
  if (os === "win32") {
    powershell(`Add-Type -AssemblyName System.Windows.Forms
$shell = New-Object -ComObject WScript.Shell
$null = $shell.AppActivate(${appPid})
Start-Sleep -Milliseconds 400
[System.Windows.Forms.SendKeys]::SendWait('^${shift ? "+" : ""}${letter}')`);
  } else if (os === "darwin") {
    const swift = join(out, "shortcut.swift");
    writeFileSync(
      swift,
      `import AppKit
let pid = pid_t(CommandLine.arguments[1]) ?? 0
let key = CGKeyCode(CommandLine.arguments[2]) ?? 0
let shift = CommandLine.arguments.count > 3 && CommandLine.arguments[3] == "shift"
NSRunningApplication(processIdentifier: pid)?.activate(options: [])
usleep(400000)
for down in [true, false] {
  let event = CGEvent(keyboardEventSource: nil, virtualKey: key, keyDown: down)
  event?.flags = shift ? [.maskCommand, .maskShift] : .maskCommand
  event?.post(tap: .cghidEventTap)
  usleep(60000)
}
// Command up (kVK_Command), so the session doesn't keep it down for the next synthetic keys.
let release = CGEvent(keyboardEventSource: nil, virtualKey: 55, keyDown: false)
release?.flags = []
release?.post(tap: .cghidEventTap)
usleep(60000)
`,
    );
    run("swift", [swift, String(appPid), String(MAC_KEYS[letter] ?? 0), ...(shift ? ["shift"] : [])]);
  } else {
    run("xdotool", ["key", `ctrl+${shift ? "shift+" : ""}${XDOTOOL_KEYS[letter] ?? letter}`]);
  }
  console.log(`pressed ${os === "darwin" ? "Cmd" : "Ctrl"}+${shift ? "Shift+" : ""}${letter.toUpperCase()}`);
}

/** Resident memory of `pid` in MB (working set on Windows, RSS elsewhere). */
function memoryMb(pid: number): number {
  if (os === "win32") {
    return Number(powershell(`[math]::Round((Get-Process -Id ${pid}).WorkingSet64 / 1MB, 1)`).trim()) || -1;
  }
  return Math.round((Number(run("ps", ["-o", "rss=", "-p", String(pid)]).trim()) / 1024) * 10) / 10 || -1;
}

/**
 * Memory that is the app's own, in MB — RSS also counts shared system libraries: private bytes
 * on Windows, private dirty pages on Linux (/proc/<pid>/smaps_rollup: what the kernel can't
 * just drop and re-read from the library files), the physical footprint on macOS.
 */
function ownMemoryMb(pid: number): number {
  if (os === "win32") {
    return Number(powershell(`[math]::Round((Get-Process -Id ${pid}).PrivateMemorySize64 / 1MB, 1)`).trim()) || -1;
  }
  if (os === "linux") {
    const rollup = existsSync(`/proc/${pid}/smaps_rollup`) ? readFileSync(`/proc/${pid}/smaps_rollup`, "utf8") : "";
    const dirty = /^Private_Dirty:\s+(\d+) kB/m.exec(rollup)?.[1];
    return dirty ? Math.round((Number(dirty) / 1024) * 10) / 10 : -1;
  }
  const footprint = /Footprint:\s+([\d.]+)\s*(KB|MB|GB)/i.exec(run("footprint", ["-p", String(pid)]));
  if (!footprint?.[1] || !footprint[2]) {
    return -1;
  }
  const scale: Record<string, number> = { KB: 1 / 1024, MB: 1, GB: 1024 };
  return Math.round(Number(footprint[1]) * (scale[footprint[2].toUpperCase()] ?? 1) * 10) / 10;
}

/** A listing of the webview helper processes, for the artifact. */
function helperListing(): string {
  if (os === "win32") {
    return powershell(
      `Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'" | Select-Object ProcessId, ParentProcessId, WorkingSetSize | Format-Table -AutoSize | Out-String -Width 200`,
    );
  }
  const pattern = os === "darwin" ? "com.apple.WebKit" : "WebKit";
  return run("ps", ["-axo", "pid,ppid,rss,etime,command"])
    .split("\n")
    .filter((line, index) => index === 0 || line.includes(pattern))
    .join("\n");
}

/**
 * Process ids of the webview engine's helper processes, system-wide. On macOS they are XPC
 * services whose parent is launchd, so other apps' WebKit helpers show up too; the checks
 * therefore follow the ids that appeared while our page was shown.
 */
function helperPids(): number[] {
  const output =
    os === "win32"
      ? powershell(`(Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'").ProcessId`)
      : run("pgrep", ["-f", os === "darwin" ? "com.apple.WebKit.WebContent" : "WebKitWebProcess"]);
  return output
    .split(/\s+/)
    .map(Number)
    .filter((pid) => Number.isInteger(pid) && pid > 0);
}

/**
 * The app's engine processes now (LJ19.4's rest runs), with their resident memory: on Windows its
 * msedgewebview2 tree (renderers are `--type=renderer`), on Linux its WebKitGTK children (web
 * processes are WebKitWebProcess), on macOS the WebKit services — XPC services, launchd's children,
 * so the ones that weren't running before the launch (`before`; WebContent runs pages).
 */
function engineProcs(appPid: number, before: readonly number[]): EngineProc[] {
  if (appPid <= 0) {
    return [];
  }
  if (os === "win32") {
    type Row = { ProcessId: number; ParentProcessId: number; Name: string; WorkingSetSize: number; CommandLine: string | null };
    let rows: Row[];
    try {
      const parsed = JSON.parse(
        powershell("Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name, WorkingSetSize, CommandLine | ConvertTo-Json -Compress") || "[]",
      ) as Row | Row[];
      rows = Array.isArray(parsed) ? parsed : [parsed];
    } catch {
      rows = [];
    }
    const tree = descendants(rows.map((r) => ({ pid: r.ProcessId, ppid: r.ParentProcessId })), appPid);
    return rows
      .filter((r) => tree.has(r.ProcessId) && r.ProcessId !== appPid && /msedgewebview2/i.test(r.Name))
      .map((r) => ({ pid: r.ProcessId, name: r.Name, web: (r.CommandLine ?? "").includes("--type=renderer"), rssKb: Math.round(Number(r.WorkingSetSize) / 1024) }));
  }
  if (os === "linux") {
    const procs = listProcs();
    const tree = descendants(procs, appPid);
    return procs
      .filter((p) => tree.has(p.pid) && p.pid !== appPid)
      .map((p) => {
        let name = "";
        let rssKb = 0;
        try {
          name = readFileSync(`/proc/${p.pid}/comm`, "utf8").trim();
          rssKb = Number(/^VmRSS:\s+(\d+) kB/m.exec(readFileSync(`/proc/${p.pid}/status`, "utf8"))?.[1] ?? 0);
        } catch {
          // Gone meanwhile.
        }
        return { pid: p.pid, name, web: name.startsWith("WebKitWebProces"), rssKb };
      })
      .filter((p) => p.name.startsWith("WebKit"));
  }
  const old = new Set(before);
  return run("ps", ["-axo", "pid=,rss=,comm="])
    .split("\n")
    .flatMap((line) => {
      const match = /^\s*(\d+)\s+(\d+)\s+(.*)$/.exec(line);
      if (!match?.[1] || !match[2] || !match[3]?.includes("com.apple.WebKit.")) {
        return [];
      }
      const pid = Number(match[1]);
      const name = match[3].split("/").at(-1) ?? match[3];
      return old.has(pid) ? [] : [{ pid, name, web: name.includes("WebContent"), rssKb: Number(match[2]) }];
    });
}

/** The pids under `root` (itself included) in a process list. */
function descendants(procs: readonly { pid: number; ppid: number }[], root: number): Set<number> {
  const children = new Map<number, number[]>();
  for (const p of procs) {
    children.set(p.ppid, [...(children.get(p.ppid) ?? []), p.pid]);
  }
  const tree = new Set<number>();
  const queue = [root];
  while (queue.length > 0) {
    const pid = queue.pop() as number;
    if (!tree.has(pid)) {
      tree.add(pid);
      queue.push(...(children.get(pid) ?? []));
    }
  }
  return tree;
}

/** Every WebKit service running now (macOS; elsewhere the app's tree says which are ours). */
function enginePidsNow(): number[] {
  if (os !== "darwin") {
    return [];
  }
  return run("ps", ["-axo", "pid=,comm="])
    .split("\n")
    .filter((line) => line.includes("com.apple.WebKit."))
    .map((line) => Number(line.trim().split(/\s+/)[0]))
    .filter((pid) => Number.isInteger(pid) && pid > 0);
}

/** Ends processes at once (the rest run's "the bar's web process died"). */
function killPids(pids: readonly number[]): void {
  for (const pid of pids) {
    if (os === "win32") {
      run("taskkill", ["/F", "/PID", String(pid)]);
    } else {
      try {
        process.kill(pid, "SIGKILL");
      } catch (err) {
        console.log(`(could not kill ${pid}: ${String(err)})`);
      }
    }
  }
  console.log(`killed ${pids.join(", ") || "nothing"}`);
}

/**
 * The OS's login items that run the installed app (the binary and its canonical path): values
 * under HKCU's Run key on Windows, LaunchAgent plists on macOS, XDG autostart files on Linux.
 * Every entry is read, not just the name tauri-plugin-autostart uses, so a renamed entry counts.
 */
function loginEntries(): LoginEntry[] {
  const all: LoginEntry[] = [];
  if (os === "win32") {
    const key = "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    for (const value of parseRegValues(run("reg", ["query", key]))) {
      all.push({ where: `${key}\\${value.name}`, command: value.data });
    }
  } else {
    const [dir, extension] =
      os === "darwin" ? [join(homedir(), "Library", "LaunchAgents"), ".plist"] : [join(homedir(), ".config", "autostart"), ".desktop"];
    for (const name of existsSync(dir) ? readdirSync(dir).filter((file) => file.endsWith(extension)) : []) {
      let text = "";
      try {
        text = readFileSync(join(dir, name), "utf8");
      } catch (err) {
        console.log(`(can't read ${join(dir, name)}: ${String(err)})`);
      }
      const command = os === "darwin" ? plistProgram(text) : desktopExec(text);
      if (command !== undefined) {
        all.push({ where: join(dir, name), command });
      }
    }
  }
  const installed = [app as string, realpathSync(app as string)];
  return all.filter((entry) => pointsAt(entry.command, installed, os === "win32"));
}

/** Runs a second instance with `args`; single-instance forwards them and it exits. */
async function forward(args: string[], withEnv: NodeJS.ProcessEnv = env): Promise<void> {
  console.log(`forward: ${args.join(" ")}`);
  const child = spawn(app as string, args, { env: withEnv, stdio: "ignore" });
  await new Promise<void>((done) => {
    const timer = setTimeout(() => {
      console.log("(the forwarding instance did not exit within 15 s)");
      child.kill();
      done();
    }, 15_000);
    child.on("exit", () => {
      clearTimeout(timer);
      done();
    });
  });
}

/** Each request's agent headers, one JSON line each (the site-mode run reads them, LJ40). */
const agentLogPath = join(out, "agent-headers.jsonl");

/** The ping page's pings (LJ19.4's rest runs): the id and the server's unix ms of each. */
const pingLog: { id: string; t: number }[] = [];

/** What the server saw of the downloads run's slow download (LJ15). */
const slowServe: SlowServe = { started: false, chunksSent: 0, aborted: false, completed: false };

/** The slow download: 1 KB every SLOW_EVERY_MS until SLOW_CHUNKS, unless the client goes away. */
function serveSlow(res: ServerResponse): void {
  Object.assign(slowServe, { started: true, chunksSent: 0, aborted: false, completed: false });
  res.writeHead(200, attachmentHeaders(SLOW_NAME, SLOW_CHUNKS * SLOW_CHUNK));
  const chunk = Buffer.alloc(SLOW_CHUNK, 0x6c);
  let timer: ReturnType<typeof setTimeout> | undefined;
  res.on("close", () => {
    clearTimeout(timer);
    if (!slowServe.completed) {
      slowServe.aborted = true;
    }
  });
  const send = (): void => {
    if (res.destroyed) {
      return;
    }
    res.write(chunk);
    slowServe.chunksSent += 1;
    if (slowServe.chunksSent >= SLOW_CHUNKS) {
      slowServe.completed = true;
      res.end();
      return;
    }
    timer = setTimeout(send, SLOW_EVERY_MS);
  };
  send();
}

function serve(): Promise<Server> {
  const types: Record<string, string> = { ".html": "text/html; charset=utf-8", ".mjs": "text/javascript" };
  rmSync(agentLogPath, { force: true });
  const server = createServer((req, res) => {
    const name = (req.url ?? "/").split("?")[0]?.replace(/^\//, "") ?? "";
    const header = (key: string) => {
      const value = req.headers[key];
      return typeof value === "string" ? value : null;
    };
    appendFileSync(
      agentLogPath,
      `${JSON.stringify({
        t: Date.now(),
        path: name,
        query: (req.url ?? "").includes("?") ? (req.url ?? "").slice((req.url ?? "").indexOf("?")) : "",
        ua: header("user-agent") ?? "",
        mobile: header("sec-ch-ua-mobile"),
        platform: header("sec-ch-ua-platform"),
        brands: header("sec-ch-ua"),
      })}\n`,
    );
    // The ping page (LJ19.4): ?ping=<id> pages ping here while they run.
    if (name === "ping") {
      const id = new URLSearchParams((req.url ?? "").split("?")[1] ?? "").get("id") ?? "";
      pingLog.push({ id, t: Date.now() });
      res.writeHead(204, { "cache-control": "no-store" }).end();
      return;
    }
    // The downloads run's attachments (LJ15): the address names something else than the header.
    if (name === DOWNLOAD_PATH) {
      const body = downloadBody();
      res.writeHead(200, attachmentHeaders(DOWNLOAD_NAME, body.length)).end(Buffer.from(body));
      return;
    }
    if (name === SLOW_PATH) {
      serveSlow(res);
      return;
    }
    if (name === PDF_PATH) {
      res.writeHead(200, { ...attachmentHeaders(PDF_NAME, Buffer.byteLength(PDF_BODY)), "content-type": "application/pdf" }).end(PDF_BODY);
      return;
    }
    const file = join(pageDir, name);
    if (!/^[\w.-]+$/.test(name) || !types[extname(name)] || !existsSync(file)) {
      res.writeHead(404).end();
      return;
    }
    res.writeHead(200, { "content-type": types[extname(name)] ?? "text/plain" }).end(readFileSync(file));
  });
  return new Promise((done) => server.listen(0, "127.0.0.1", () => done(server)));
}

/** The AppImage (LJ13): its runtime starts the app as a child (smoke-linux.mts). */
const appImage = os === "linux" && app.endsWith(".AppImage");
/** gdb's commands once the app runs: a crash prints a backtrace. */
const GDB_AFTER = ["-ex", "bt 40", "-ex", "thread apply all bt 8"];

/**
 * Launches the app; with LJ_SMOKE_GDB=1 (Linux) it runs under gdb so a crash prints a backtrace —
 * the AppImage's app gets gdb attached once it runs (attachGdb) — unless `gdb` is false (the
 * timing run: gdb slows every thread and library the app starts).
 */
function launch(withEnv: NodeJS.ProcessEnv = env, gdb = true): ChildProcess {
  const underGdb = gdb && os === "linux" && process.env.LJ_SMOKE_GDB === "1";
  const [command, args] =
    underGdb && !appImage ? ["gdb", ["-q", "-batch", "-ex", "run", ...GDB_AFTER, "--args", app as string]] : [app as string, []];
  const child = spawn(command, args, { env: withEnv, stdio: ["ignore", "pipe", "pipe"] });
  child.stdout?.on("data", (chunk: Buffer) => process.stdout.write(`[app] ${chunk.toString()}`));
  child.stderr?.on("data", (chunk: Buffer) => process.stdout.write(`[app] ${chunk.toString()}`));
  child.on("exit", (code, signal) => console.log(`app exited with code ${code} signal ${signal}`));
  if (underGdb && appImage) {
    void attachGdb(child);
  }
  return child;
}

/**
 * The AppImage under gdb: gdb can't follow its runtime into the app (AppRun's subshells come
 * first), so it attaches to the app once that runs (CI allows ptrace beyond a process's children
 * for this, ci.yml). A crash before that has no backtrace; the runtime's exit status still shows it.
 */
async function attachGdb(runtime: ChildProcess): Promise<void> {
  for (let waited = 0; waited < 30_000 && runtime.exitCode === null && runtime.signalCode === null; waited += 100) {
    const pid = runtime.pid === undefined ? undefined : findApp(listProcs(), runtime.pid);
    if (pid !== undefined) {
      const gdb = spawn("gdb", ["-q", "-batch", "-p", String(pid), "-ex", "continue", ...GDB_AFTER], { stdio: ["ignore", "pipe", "pipe"] });
      gdb.stdout.on("data", (chunk: Buffer) => process.stdout.write(`[gdb] ${chunk.toString()}`));
      gdb.stderr.on("data", (chunk: Buffer) => process.stdout.write(`[gdb] ${chunk.toString()}`));
      console.log(`gdb attached to the AppImage's app (pid ${pid})`);
      return;
    }
    await sleep(100);
  }
  console.log("(gdb: the AppImage's app never showed up)");
}

function alive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

/**
 * Quits the app and waits until its process is gone, so the next launch is the only instance
 * (single-instance would otherwise forward to the old one). Under gdb the app is killed first.
 */
async function quit(child: ChildProcess, pid: number): Promise<void> {
  for (const signal of ["SIGTERM", "SIGKILL"] as const) {
    if (pid > 0 && alive(pid)) {
      try {
        process.kill(pid, signal);
      } catch {
        // Already gone.
      }
    }
    child.kill(signal);
    for (let waited = 0; waited < 5_000 && pid > 0 && alive(pid); waited += 250) {
      await sleep(250);
    }
    if (pid <= 0 || !alive(pid)) {
      break;
    }
  }
  console.log(pid > 0 && alive(pid) ? `(app ${pid} is still running)` : `app ${pid} has quit`);
}

/** Waits up to `timeoutMs` for the process to end on its own (after `--quit`). */
async function exited(pid: number, timeoutMs: number): Promise<boolean> {
  for (let waited = 0; waited < timeoutMs && pid > 0 && alive(pid); waited += 250) {
    await sleep(250);
  }
  const gone = pid <= 0 || !alive(pid);
  console.log(gone ? `app ${pid} ended by itself` : `(app ${pid} still running ${timeoutMs} ms after --quit)`);
  return gone;
}

/**
 * The pid of the app itself: on Linux the newest little-joan in the launched process's tree (gdb's
 * child; the AppImage runtime's — smoke-linux.mts), elsewhere the launched process.
 */
function appPid(child: ChildProcess): number {
  if (os === "linux" && child.pid !== undefined) {
    return findApp(listProcs(), child.pid) ?? -1;
  }
  return child.pid ?? -1;
}

if (startedOnly) {
  const child = launch();
  const ok = await waitFor("started", (e) => count(e, "started") > 0);
  child.kill();
  process.exit(ok ? 0 : 1);
}

const server = await serve();
const port = (server.address() as AddressInfo).port;
const child = launch();

await waitFor("started + shortcut", (e) => count(e, "started") > 0 && (count(e, "shortcut-registered") > 0 || count(e, "shortcut-failed") > 0));
if (count(events(), "started") === 0 && (child.exitCode !== null || child.signalCode !== null)) {
  // Every later step would only time out (CI's AppImage job once spent its 45 minutes so).
  console.log("\nSMOKE FAILED:\n- the app ended before it started (its output is above)");
  process.exit(1);
}
// Stay ready (LJ19.4, the default): the hidden window and the bar a few seconds after the launch.
// The page's helpers are what comes after this (the bar's stay; smoke-rest.mts checks them).
await waitFor("prewarmed (the hidden window and the bar)", (e) => count(e, "prewarmed") > 0, 20_000);
await sleep(2000);
const pid = appPid(child);
const idleMb = memoryMb(pid);
const idleOwnMb = ownMemoryMb(pid);
const helpersAtStart = helperPids();
console.log(`ready at start: ${idleMb} MB resident, ${idleOwnMb} MB own, ${helpersAtStart.length} webview helpers (the bar's)`);
screenshot("hibernating.png");

await forward(["--show"]);
await waitFor("window-created", (e) => count(e, "window-created") > 0, 20_000);
await waitFor("command bar open (overlay true)", (e) => count(e, "overlay", "true") > 0, 20_000);
await waitFor("start page title", (e) => count(e, "title", "Little Joan") > 0, 20_000);
// The command bar's first suggestions (a slow runner's first window can take a moment).
await waitFor("command bar: first suggestions", (e) => count(e, "suggestions") > 0, 15_000);
await sleep(1500);
// The command bar covers the whole window (the page behind it is hidden).
screenshot("start.png");

await forward(["--show", `http://127.0.0.1:${port}/smoke.html`]);
await waitFor("smoke page ready", (e) => lastTitle(e, "LJ-SMOKE ready") !== undefined, 30_000);
await waitFor("command bar closed (overlay false)", (e) => count(e, "overlay", "false") > 0, 5_000);
await sleep(1500);
const shownMb = memoryMb(pid);
const shownOwnMb = ownMemoryMb(pid);
const pageHelpers = newPids(helpersAtStart, helperPids());
writeFileSync(join(out, "helpers-with-page.txt"), helperListing());
screenshot("page.png");

clickCentre();
const windowWanted = os !== "darwin" && !fullscreenOff;
if (fullscreenOff) {
  // Fullscreen is off on this engine: the click must leave the window as it is (and the app alive,
  // which the page's next load shows).
  await sleep(4000);
} else {
  await waitFor(
    "fullscreen",
    (e) => lastTitle(e, "LJ-SMOKE fullscreen=on") !== undefined && (!windowWanted || count(e, "fullscreen", "true") > 0),
    15_000,
  );
  await sleep(2000);
}
screenshot("fullscreen.png");

// Leave fullscreen by loading the page again, then resize the window: the page must reflow to
// it (innerWidth × innerHeight = the window minus the 40 px URL bar, in logical pixels) and get
// a phone user agent while narrow (< 600 px), the desktop one when wide. 720 px keeps the wide
// window on a 1024 px runner screen.
const readyBefore = count(events(), "title", "LJ-SMOKE ready");
await forward(["--show", `http://127.0.0.1:${port}/smoke.html`]);
await waitFor("smoke page loaded again", (e) => count(e, "title", "LJ-SMOKE ready") > readyBefore, 30_000);
if (windowWanted) {
  await waitFor("window left fullscreen", (e) => e.filter((x) => x.event === "fullscreen").at(-1)?.detail === "false", 15_000);
}
await sleep(1500);
const NARROW: Viewport = { width: 390, height: 560, ua: "phone" };
const WIDE: Viewport = { width: 720, height: 560, ua: "desktop" };
await forward(["--size", "390x600"]);
await waitFor("narrow viewport, phone user agent", (e) => sawViewport(e, NARROW), 30_000);
await sleep(1000);
screenshot("narrow.png");
await forward(["--size", "720x600"]);
await waitFor("wide viewport, desktop user agent", (e) => sawViewport(e, WIDE), 30_000);
await sleep(1000);
screenshot("wide.png");

// Page zoom (LJ16, smoke-zoom.mts): Ctrl+= (⌘= on macOS, best effort) zooms the page to 110 %,
// so it reports a narrower viewport; Ctrl+0 brings 720 px back.
const zoomFrom = events().length;
pressShortcut("=", pid);
if (await waitFor("zoom: Ctrl+= zoomed the page", (e) => count(e.slice(zoomFrom), "zoom", String(ZOOMED)) > 0, 10_000)) {
  // The toolbar says "Zoom 110 %" for 1.5 s.
  screenshot("zoom.png");
  await waitFor("zoom: the page narrower", (e) => zoomedViewport(e.slice(zoomFrom), WIDE) !== undefined, 15_000);
  await sleep(500);
}
pressShortcut("0", pid);
await waitFor("zoom: Ctrl+0 brought 720 px back", (e) => zoomReset(e.slice(zoomFrom), WIDE), 15_000);
await sleep(1000);
const zoomEvents = events().slice(zoomFrom);

await forward(["--hide"]);
await waitFor("hibernated", (e) => count(e, "hibernated") > 0, 20_000);
// Engines may keep helper processes briefly after the last view closes; sample for a minute.
const helperSamples: Record<string, number> = {};
let waited = 0;
for (const at of [2, 10, 30, 60]) {
  await sleep((at - waited) * 1000);
  waited = at;
  helperSamples[`${at}s`] = survivors(pageHelpers, helperPids());
}
const afterMb = memoryMb(pid);
const afterOwnMb = ownMemoryMb(pid);
const helpersAfter = helperSamples["60s"] ?? survivors(pageHelpers, helperPids());
writeFileSync(join(out, "helpers-after-hibernate.txt"), helperListing());
console.log(`ready after a page: ${afterMb} MB resident, ${afterOwnMb} MB own, the page's helpers left ${JSON.stringify(helperSamples)}`);

// Two more summon / close cycles: helper counts that grow per cycle would mean webviews leak.
const helpersPerCycle: number[] = [helpersAfter];
for (const cycle of [2, 3]) {
  const before = helperPids();
  await forward(["--show"]);
  await waitFor(`window-created #${cycle}`, (e) => count(e, "window-created") >= cycle, 20_000);
  await sleep(1500);
  const ours = newPids(before, helperPids());
  await forward(["--hide"]);
  await waitFor(`hibernated #${cycle}`, (e) => count(e, "hibernated") >= cycle, 20_000);
  await sleep(10_000);
  helpersPerCycle.push(survivors(ours, helperPids()));
}
console.log(`webview helpers after each hibernate: ${JSON.stringify(helpersPerCycle)}`);

// The command bar (LJ23): after the smoke page was visited, a new window's empty command bar
// offers its site as a top site; typing "127" finds it in history.
const commandFrom = events().length;
const hibernatedBeforeCommand = count(events(), "hibernated");
await forward(["--show"]);
await waitFor("command bar: the smoke page's site as a top site", (e) => sawSmokeTopSite(e.slice(commandFrom)), 20_000);
await sleep(1500);
screenshot("command-bar.png");
typeText(TYPED, pid);
const typingDriven = os !== "darwin";
await waitFor(`command bar: typing "${TYPED}" finds the smoke page`, (e) => sawTypedMatch(e.slice(commandFrom)), 15_000);
await sleep(1000);
screenshot("command-typing.png");
// The history view (LJ25): Ctrl+H (Cmd+Y on macOS, best effort; LJ37) opens it over the command
// bar, the smoke page under Today.
const historyFrom = events().length;
pressShortcut(HISTORY_KEY, pid);
await waitFor("history view: Ctrl+H (Cmd+Y) lists the history", (e) => sawHistoryView(e.slice(historyFrom)), 15_000);
await sleep(1000);
screenshot("history.png");
// Windows and Linux: with the view closed again (Esc: back to the command bar), Ctrl+Y must open
// nothing within 2 s — it is the page's redo again.
let afterCtrlY: LogEvent[] | undefined;
if (typingDriven) {
  pressKeys(["Escape"], pid, false);
  await sleep(1000);
  const ctrlYFrom = events().length;
  pressShortcut("y", pid);
  await sleep(2000);
  afterCtrlY = events().slice(ctrlYFrom);
  console.log(`after Ctrl+Y: ${afterCtrlY.map((e) => e.event).join(", ") || "nothing"}`);
}
await forward(["--hide"]);
await waitFor("command bar: hibernated", (e) => count(e, "hibernated") > hibernatedBeforeCommand, 20_000);

// Start at login: `on` must write the OS's login entry for the INSTALLED binary and `off` must
// remove it again, without touching the window (hibernating here).
if (os === "win32") {
  // A Windows profile can lack the per-user Run key (one runner did, and every call failed);
  // remove it on this throwaway runner so the app has to create it, every run. (`run` logs a
  // failure, e.g. when there is no key to remove, and carries on.)
  run("reg", ["delete", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/f"]);
}
await forward(["--start-at-login", "on"]);
await waitFor(
  "start-at-login true",
  (e) => count(e, "start-at-login", "true") > 0 || count(e, "start-at-login-failed") > 0,
  15_000,
);
const loginAfterOn = loginEntries();
console.log(`login entries after on: ${JSON.stringify(loginAfterOn)}`);
await forward(["--start-at-login", "off"]);
await waitFor(
  "start-at-login false",
  (e) => count(e, "start-at-login", "false") > 0 || count(e, "start-at-login-failed") > 0,
  15_000,
);
const loginAfterOff = loginEntries();
console.log(`login entries after off: ${JSON.stringify(loginAfterOff)}`);
// A window the flags opened by mistake would show up by now; later steps open their own.
await sleep(1500);
const loginEventsUntil = events().length;

// Logins survive: the smoke page sets a persistent cookie, a session cookie and a localStorage
// value. All three must outlive closing the window (`--hide`, then a new window); after a
// `--quit` right after setting new values and a relaunch on the same data folder, the
// persistent cookie and localStorage must still be there and the session cookie gone (it ends
// with the app run, as in browsers). The server keeps its port across the relaunch, because
// localStorage is per origin (cookies ignore the port).
const persistUrl = (mode: "set" | "check", token: string) => `http://127.0.0.1:${port}/smoke.html?persist=${mode}&t=${token}`;
const token = (kind: string) => `${kind}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
const closeToken = token("close");
const quitToken = token("quit");
const sets = (e: LogEvent[]) => count(e, "title", "LJ-PERSIST set");
const checks = (e: LogEvent[]) => count(e, "title", "LJ-PERSIST p=");

const setsBeforeClose = sets(events());
await forward(["--show", persistUrl("set", closeToken)]);
await waitFor("persist: set", (e) => sets(e) > setsBeforeClose, 30_000);
const hibernatedBeforeClose = count(events(), "hibernated");
await forward(["--hide"]);
await waitFor("persist: hibernated", (e) => count(e, "hibernated") > hibernatedBeforeClose, 20_000);
await sleep(1000);
const checksBeforeClose = checks(events());
await forward(["--show", persistUrl("check", closeToken)]);
await waitFor("persist: checked after close → summon", (e) => checks(e) > checksBeforeClose, 30_000);
const persistAfterClose = lastTitle(events(), "LJ-PERSIST p=");
console.log(`persist after close → summon: ${persistAfterClose ?? "no report"}`);

// New values, then quit at once (polling fast so the quit follows the set within a second).
const setsBeforeQuit = sets(events());
await forward(["--show", persistUrl("set", quitToken)]);
await waitFor("persist: set before quitting", (e) => sets(e) > setsBeforeQuit, 30_000, logPath, 50);
await forward(["--quit"]);
const quitExited = await exited(pid, 15_000);
if (!quitExited) {
  await quit(child, pid);
}
const afterQuitLogPath = join(out, "smoke-after-quit.log");
rmSync(afterQuitLogPath, { force: true });
const afterQuitEnv = { ...env, LITTLEJOAN_SMOKE_LOG: afterQuitLogPath };
// Let the engine's processes of the old run finish before the same data folder is reopened.
await sleep(2000);
const afterQuitChild = launch(afterQuitEnv);
await waitFor("after quit: started", (e) => count(e, "started") > 0, 30_000, afterQuitLogPath);
await sleep(1500);
const afterQuitPid = appPid(afterQuitChild);
await forward(["--show", persistUrl("check", quitToken)], afterQuitEnv);
await waitFor("persist: checked after quit → relaunch", (e) => checks(e) > 0, 30_000, afterQuitLogPath);
const afterQuitEvents = events(afterQuitLogPath);
const persistAfterQuit = lastTitle(afterQuitEvents, "LJ-PERSIST p=");
console.log(`persist after quit → relaunch: ${persistAfterQuit ?? "no report"}`);
await forward(["--quit"], afterQuitEnv);
if (!(await exited(afterQuitPid, 15_000))) {
  await quit(afterQuitChild, afterQuitPid);
}

// History (LJ21): the main run's history.json must hold the smoke page; a run whose settings say
// "rememberHistory": false loads the page too and must record nothing.
const smokePageUrl = `http://127.0.0.1:${port}/smoke.html`;
const historyMain = readIfThere(join(env.LITTLEJOAN_DATA_DIR, "history.json"));
// Developer mode on, with two developer extensions, for Settings → Extensions (LJ30).
const historyOffDir = dataDir("history-off", true, { rememberHistory: false, developerMode: true });
writeDeveloperExtensions(historyOffDir);
const historyOffLogPath = join(out, "smoke-history-off.log");
rmSync(historyOffLogPath, { force: true });
const historyOffEnv = { ...env, LITTLEJOAN_DATA_DIR: historyOffDir, LITTLEJOAN_SMOKE_LOG: historyOffLogPath };
const historyOffChild = launch(historyOffEnv);
await waitFor("history off: started", (e) => count(e, "started") > 0, 30_000, historyOffLogPath);
await sleep(1500);
const historyOffPid = appPid(historyOffChild);
await forward(["--show", smokePageUrl], historyOffEnv);
await waitFor("history off: smoke page", (e) => lastTitle(e, "LJ-SMOKE ready") !== undefined, 30_000, historyOffLogPath);
await forward(["--hide"], historyOffEnv);
await waitFor("history off: hibernated", (e) => count(e, "hibernated") > 0, 20_000, historyOffLogPath);
// Settings (LJ26), while hibernating: `--settings` opens it on General, `--settings privacy`
// switches the open window to Privacy (where "Remember history" is off in this run), `--settings
// appearance` to the theme editor (LJ28); none opens the page window.
const settingsFrom = events(historyOffLogPath).length;
await forward(["--settings"], historyOffEnv);
await waitFor("settings: General", (e) => sawSettings(e.slice(settingsFrom), "general"), 20_000, historyOffLogPath);
await sleep(1500);
screenshot("settings-general.png");
await forward(["--settings", "privacy"], historyOffEnv);
await waitFor("settings: Privacy", (e) => settingsTabs(e.slice(settingsFrom)).includes("privacy"), 15_000, historyOffLogPath);
await sleep(1000);
screenshot("settings-privacy.png");
// The theme editor (LJ28), in the default theme.
await forward(["--settings", "appearance"], historyOffEnv);
await waitFor("settings: Appearance", (e) => settingsTabs(e.slice(settingsFrom)).includes("appearance"), 15_000, historyOffLogPath);
await sleep(1000);
screenshot("settings-appearance.png");
// The Keyboard tab (LJ44): the summon shortcut and every Little Joan key.
await forward(["--settings", "keyboard"], historyOffEnv);
await waitFor("settings: Keyboard", (e) => settingsTabs(e.slice(settingsFrom)).includes("keyboard"), 15_000, historyOffLogPath);
await sleep(1000);
screenshot("settings-keyboard.png");
// The extensions (LJ30): Hello, the developer copy, and the broken one's reason.
await forward(["--settings", "extensions"], historyOffEnv);
await waitFor(
  "settings: Extensions",
  (e) => settingsTabs(e.slice(settingsFrom)).includes("extensions") && e.slice(settingsFrom).some((x) => x.event === "extensions-list"),
  15_000,
  historyOffLogPath,
);
await sleep(1000);
screenshot("settings-extensions.png");
await forward(["--quit"], historyOffEnv);
if (!(await exited(historyOffPid, 15_000))) {
  await quit(historyOffChild, historyOffPid);
}
const historyOff = readIfThere(join(historyOffDir, "history.json"));
const historyOffEvents = events(historyOffLogPath);
const historyCheck = { main: historyMain, pageUrl: smokePageUrl, off: historyOff };
console.log(`history: main ${historyMain === undefined ? "no file" : `${historyMain.length} bytes`}, off ${historyOff ?? "no file"}`);

/**
 * A theme run (LJ27): its own launch on a fresh data folder whose settings.json carries the
 * run's theme. The smoke page (the toolbar above it), a new window's command bar with the page's
 * site, the history view (Ctrl+H / Cmd+Y, best effort as in the main run) and Settings, each
 * screenshotted into <out>/theme/ when the run names it.
 */
async function themeRun(run: ThemeRun): Promise<LogEvent[]> {
  const logFile = join(out, `smoke-${run.name}.log`);
  rmSync(logFile, { force: true });
  const runEnv = { ...env, LITTLEJOAN_DATA_DIR: dataDir(run.name, true, { theme: run.theme }), LITTLEJOAN_SMOKE_LOG: logFile };
  const shot = (name: string) => screenshot(join("theme", name));
  const runChild = launch(runEnv);
  await waitFor(`${run.name}: started`, (e) => count(e, "started") > 0, 30_000, logFile);
  await sleep(1500);
  const runPid = appPid(runChild);
  // The page first, so the command bar has its site to offer.
  await forward(["--show", smokePageUrl], runEnv);
  await waitFor(`${run.name}: smoke page`, (e) => lastTitle(e, "LJ-SMOKE ready") !== undefined && count(e, "overlay", "false") > 0, 30_000, logFile);
  await sleep(1500);
  if (run.shots.page) {
    shot(run.shots.page);
  }
  await forward(["--hide"], runEnv);
  await waitFor(`${run.name}: hibernated`, (e) => count(e, "hibernated") > 0, 20_000, logFile);
  const from = events(logFile).length;
  await forward(["--show"], runEnv);
  await waitFor(`${run.name}: command bar`, (e) => sawSmokeTopSite(e.slice(from)), 20_000, logFile);
  await sleep(1500);
  shot(run.shots.commandBar);
  if (run.shots.history) {
    const historyAt = events(logFile).length;
    pressShortcut(HISTORY_KEY, runPid);
    await waitFor(`${run.name}: history view`, (e) => sawHistoryView(e.slice(historyAt)), 10_000, logFile);
    await sleep(1000);
    shot(run.shots.history);
  }
  await forward(["--hide"], runEnv);
  await waitFor(`${run.name}: hibernated again`, (e) => count(e, "hibernated") > 1, 20_000, logFile);
  if (run.shots.settings) {
    const settingsAt = events(logFile).length;
    await forward(["--settings", "appearance"], runEnv);
    await waitFor(`${run.name}: Settings`, (e) => sawSettings(e.slice(settingsAt), "appearance"), 20_000, logFile);
    await sleep(1500);
    shot(run.shots.settings);
  }
  await forward(["--quit"], runEnv);
  if (!(await exited(runPid, 15_000))) {
    await quit(runChild, runPid);
  }
  const runEvents = events(logFile);
  console.log(`${run.name}: themes handed to the bar ${JSON.stringify(runEvents.map(parseTheme).filter((t) => t !== undefined))}`);
  return runEvents;
}

/**
 * The extension run (LJ29): its own launch on a fresh data folder. `--extension hello` opens the
 * bundled example, which reports its checks in its title; then the smoke page's probe (a web
 * page) tries hello's API, one of its files, and a navigation to its panel.
 */
/** The system clipboard's text, where the runner has a tool to read it (else null). */
function readClipboard(): string | null {
  if (os === "win32") {
    return powershell("Get-Clipboard -Raw").replace(/\r?\n$/, "");
  }
  if (os === "darwin") {
    return run("pbpaste", []);
  }
  for (const [tool, args] of [["xclip", ["-o", "-selection", "clipboard"]], ["xsel", ["--clipboard", "--output"]]] as const) {
    if (run("which", [tool]).trim()) {
      return run(tool, [...args]);
    }
  }
  return null;
}

/** What Enter on the quick answer left on the clipboard (informational, see extensionRun). */
let clipboard: string | null = null;

async function extensionRun(): Promise<{ events: LogEvent[]; store: string | undefined }> {
  const logFile = join(out, "smoke-extensions.log");
  rmSync(logFile, { force: true });
  const dir = dataDir("extensions", true);
  const runEnv = { ...env, LITTLEJOAN_DATA_DIR: dir, LITTLEJOAN_SMOKE_LOG: logFile };
  const runChild = launch(runEnv);
  await waitFor("extensions: started", (e) => count(e, "started") > 0, 30_000, logFile);
  await sleep(1500);
  const runPid = appPid(runChild);
  await forward(["--extension", "hello"], runEnv);
  await waitFor("extensions: hello reported", (e) => lastTitle(e, "LJ-EXT hello") !== undefined, 30_000, logFile);
  await sleep(1500);
  screenshot("extension-hello.png");
  // The command bar (LJ30): typing "hello" offers hello's command, "Say hello" (after the search
  // row, with no history yet); ↓ ↓ Enter opens it again.
  const hibernatedBefore = count(events(logFile), "hibernated");
  await forward(["--hide"], runEnv);
  await waitFor("extensions: hibernated", (e) => count(e, "hibernated") > hibernatedBefore, 20_000, logFile);
  const commandFrom = events(logFile).length;
  await forward(["--show"], runEnv);
  await waitFor("extensions: command bar", (e) => e.slice(commandFrom).some((x) => x.event === "extensions-list"), 20_000, logFile);
  await sleep(1500);
  typeText("hello", runPid);
  const typedHello = await waitFor(
    "extensions: typed hello",
    (e) => e.slice(commandFrom).some((x) => x.event === "suggestions" && x.detail.endsWith("typed=5")),
    15_000,
    logFile,
  );
  await sleep(1000);
  screenshot("command-hello.png");
  // Only when the typing arrived: otherwise ↓ ↓ Enter would open whatever the empty command bar
  // offers (CI's macOS once opened a starter site that way).
  if (typedHello) {
    pressKeys(["Down", "Down", "Return"], runPid);
    await waitFor(
      "extensions: Say hello opened hello",
      (e) => e.filter((x) => x.event === "extension-opened" && x.detail === "hello").length >= 2,
      15_000,
      logFile,
    );
  }
  await sleep(1500);
  // A quick answer (LJ31): "= 12*7" answers 84 in the command bar's first row; Enter copies it
  // (read back from the system clipboard where the runner has a tool for it: informational).
  const answerHibernated = count(events(logFile), "hibernated");
  await forward(["--hide"], runEnv);
  await waitFor("answers: hibernated", (e) => count(e, "hibernated") > answerHibernated, 20_000, logFile);
  const answerFrom = events(logFile).length;
  await forward(["--show"], runEnv);
  await waitFor("answers: command bar", (e) => e.slice(answerFrom).some((x) => x.event === "extensions-list"), 20_000, logFile);
  await sleep(1500);
  typeText("= 12*7", runPid);
  const typedAnswer = await waitFor(
    "answers: typed",
    (e) => e.slice(answerFrom).some((x) => x.event === "suggestions" && x.detail.endsWith("typed=6")),
    15_000,
    logFile,
  );
  await sleep(1000);
  screenshot("answer.png");
  if (typedAnswer) {
    pressKeys(["Return"], runPid);
    await sleep(1000);
    clipboard = readClipboard();
  }
  console.log(`answers: the clipboard holds ${JSON.stringify(clipboard)}`);
  const probeUrl = `${smokePageUrl}?ext=probe&base=${encodeURIComponent(helloOrigin(os))}`;
  await forward(["--show", probeUrl], runEnv);
  await waitFor("extensions: the probe reported", (e) => lastTitle(e, "LJ-EXTPROBE ljext-api") !== undefined, 40_000, logFile);
  await sleep(500);
  screenshot("extension-probe.png");
  // The race (security review, M1): the page keeps putting hello's panel URL (without the app's
  // nonce) into iframes and window.open while `--extension hello` opens it; the app's navigation
  // must win, the racers be refused, and the toolbar never name hello over the web page.
  const raceUrl = `${smokePageUrl}?ext=race&base=${encodeURIComponent(helloOrigin(os))}`;
  await forward(["--show", raceUrl], runEnv);
  await waitFor("extensions: the race runs", (e) => lastTitle(e, "LJ-EXTRACE running") !== undefined, 30_000, logFile);
  await sleep(1500);
  const reportsBefore = events(logFile).filter((e) => e.event === "title" && e.detail.startsWith("LJ-EXT hello")).length;
  await forward(["--extension", "hello"], runEnv);
  await waitFor(
    "extensions: hello opened through the race",
    (e) => e.filter((x) => x.event === "title" && x.detail.startsWith("LJ-EXT hello")).length > reportsBefore,
    30_000,
    logFile,
  );
  await sleep(1000);
  screenshot("extension-race.png");
  await forward(["--quit"], runEnv);
  if (!(await exited(runPid, 15_000))) {
    await quit(runChild, runPid);
  }
  return { events: events(logFile), store: readIfThere(join(dir, "extension-data", "hello.json")) };
}

const extensionCheck = await extensionRun();
console.log(`extensions: hello ${lastTitle(extensionCheck.events, "LJ-EXT hello") ?? "no report"}; probe ${lastTitle(extensionCheck.events, "LJ-EXTPROBE") ?? "no report"}`);

/**
 * The bundled extensions run (LJ32–LJ34): its own launch on a fresh data folder. Each bundled
 * panel in BUNDLED_PANELS opens with `--extension <id>` and is screenshotted once it is up (some
 * get a few keys first — whether they reached the panel is recorded, informational).
 */
async function bundledRun(): Promise<{ events: LogEvent[]; keys: Record<string, boolean> }> {
  const logFile = join(out, "smoke-bundled.log");
  rmSync(logFile, { force: true });
  const dir = dataDir("bundled", true);
  mkdirSync(join(dir, "extension-data"), { recursive: true });
  for (const panel of BUNDLED_PANELS) {
    if (panel.seed) {
      writeFileSync(join(dir, "extension-data", `${panel.id}.json`), seedFile(panel.seed(Date.now())));
    }
  }
  const runEnv = { ...env, LITTLEJOAN_DATA_DIR: dir, LITTLEJOAN_SMOKE_LOG: logFile };
  const runChild = launch(runEnv);
  await waitFor("bundled: started", (e) => count(e, "started") > 0, 30_000, logFile);
  await sleep(1500);
  const runPid = appPid(runChild);
  // The window first (hello's run covers opening from hibernation, where the opening waits for the
  // bar's script since LJ38).
  await forward(["--show"], runEnv);
  await waitFor("bundled: the window", (e) => count(e, "bar-loaded") > 0 && count(e, "suggestions") > 0, 20_000, logFile);
  await sleep(500);
  const keys: Record<string, boolean> = {};
  for (const panel of BUNDLED_PANELS) {
    await forward(["--extension", panel.id], runEnv);
    await waitFor(`bundled: ${panel.id} is up`, (e) => panelUp(e, panel.id), 30_000, logFile);
    await sleep(1500);
    if (panel.keys) {
      pressKeys(panel.keys, runPid);
      await sleep(1200);
      keys[panel.id] = keysReached(events(logFile), panel.id);
    }
    screenshot(panel.shot);
  }
  // Page actions (LJ34) on the smoke page, from the command bar: Ctrl/Cmd+L, type, ↓ ↓ Enter
  // (search row first, then the page action; best effort on macOS).
  for (const action of PAGE_ACTIONS) {
    const pageFrom = events(logFile).length;
    await forward(["--show", smokePageUrl], runEnv);
    await waitFor(`page action ${action.id}: the smoke page`, (e) => e.slice(pageFrom).some((x) => x.event === "title" && x.detail.startsWith("LJ-SMOKE ready")), 30_000, logFile);
    await sleep(1000);
    const barFrom = events(logFile).length;
    pressShortcut("l", runPid);
    await waitFor(`page action ${action.id}: the command bar`, (e) => count(e.slice(barFrom), "overlay", "true") > 0, 10_000, logFile);
    await sleep(500);
    // The window is in front already (Ctrl/Cmd+L went to it): no second activation.
    typeText(action.typed, runPid, false);
    const typed = await waitFor(
      `page action ${action.id}: typed`,
      (e) => e.slice(barFrom).some((x) => x.event === "suggestions" && x.detail.endsWith(`typed=${action.typed.length}`)),
      15_000,
      logFile,
    );
    if (typed) {
      await sleep(800);
      pressKeys(["Down", "Down", "Return"], runPid, false);
      await waitFor(`page action ${action.id}: up`, (e) => panelUp(e.slice(barFrom), action.id), 20_000, logFile);
    }
    await sleep(1500);
    screenshot(action.shot);
  }
  await forward(["--quit"], runEnv);
  if (!(await exited(runPid, 15_000))) {
    await quit(runChild, runPid);
  }
  return { events: events(logFile), keys };
}

const bundledCheck = await bundledRun();
console.log(`bundled: keys reached ${JSON.stringify(bundledCheck.keys)}`);

mkdirSync(join(out, "theme"), { recursive: true });
const themeRuns: { run: ThemeRun; events: LogEvent[] }[] = [];
for (const run of THEME_RUNS) {
  themeRuns.push({ run, events: await themeRun(run) });
}

/**
 * The fullscreen run (LJ36, smoke-fullscreen.mts): its own launches, driven through clicks only.
 * `name` names the data folder and the log (`smoke-<name>.log`); `extra` adds to the app's
 * environment (the tray run's Wayland session).
 */
function fullscreenLaunch(name: string, settings: Record<string, unknown>, extra: Record<string, string> = {}): Promise<SiteApp> {
  const logFile = join(out, `smoke-${name}.log`);
  rmSync(logFile, { force: true });
  const runDir = dataDir(name, true, settings);
  const runEnv = { ...env, ...extra, LITTLEJOAN_DATA_DIR: runDir, LITTLEJOAN_SMOKE_LOG: logFile };
  const runChild = launch(runEnv);
  return Promise.resolve({
    pid: () => appPid(runChild),
    settings: () => readIfThere(join(runDir, "settings.json")),
    forward: (args) => forward(args, runEnv),
    events: () => events(logFile),
    waitFor: (what, test, timeoutMs) => waitFor(what, test, timeoutMs, logFile),
    quit: async () => {
      const runPid = appPid(runChild);
      await forward(["--quit"], runEnv);
      if (!(await exited(runPid, 15_000))) {
        await quit(runChild, runPid);
      }
    },
  });
}
const fullscreenDriver: FsDriver = {
  os,
  pageUrl: smokePageUrl,
  launch: fullscreenLaunch,
  clickAt,
  screenshot,
  sleep: (ms) => sleep(ms).then(() => undefined),
  fullscreenOff,
};
mkdirSync(join(out, "fs"), { recursive: true });
// With the second launch, whose siteModes put the smoke page's host on the desktop agent.
const fullscreenCheck = await fullscreenRun(fullscreenDriver, true);
// The resize run (LJ39, smoke-resize.mts): its own launch, on the same driver.
const resizeCheck = await resizeRun(fullscreenDriver);
console.log(`resize: ${JSON.stringify(resizeSummary(resizeCheck.results))}`);
// The site-mode run (LJ40, smoke-sitemode.mts): its own launch, typing like the extension run.
const siteDriver: SiteDriver = {
  os,
  pageUrl: smokePageUrl,
  launch: fullscreenLaunch,
  screenshot,
  sleep: (ms) => sleep(ms).then(() => undefined),
  typeText,
  pressKeys,
  pressShortcut,
  agentRequests: () => parseAgentLog(readIfThere(agentLogPath) ?? ""),
};
const siteModeCheck = await siteModeRun(siteDriver);
console.log(`site mode: ${JSON.stringify(siteModeSummary(siteModeCheck, os))}`);
// The quick-commands run (LJ42, smoke-quick.mts): its own launch, typing like the site-mode run.
const quickDriver: QuickDriver = {
  os,
  pageUrl: smokePageUrl,
  launch: fullscreenLaunch,
  screenshot,
  sleep: (ms) => sleep(ms).then(() => undefined),
  typeText,
  pressKeys,
  pressShortcut,
  readClipboard,
};
const quickCheck = await quickRun(quickDriver);
console.log(`quick: ${JSON.stringify(quickSummary(quickCheck, os))}`);
// The keymap run (LJ43, smoke-keymap.mts): its own launch with History on Ctrl/Cmd+Shift+K.
const keymapDriver: KeymapDriver = {
  os,
  pageUrl: smokePageUrl,
  launch: fullscreenLaunch,
  sleep: (ms) => sleep(ms).then(() => undefined),
  pressKeys,
  pressShortcut,
};
const keymapCheck = await keymapRun(keymapDriver);
console.log(`keymap: ${JSON.stringify(keymapSummary(keymapCheck, os))}`);
// Settings → Keyboard with a saved shortcut that can't be used (LJ44): the problem line.
const keyboardProblem = await keyboardProblemRun({ launch: fullscreenLaunch, screenshot, sleep: (ms) => sleep(ms).then(() => undefined) });
if (os === "darwin") {
  // Evidence for reading macOS's own shortcuts (LJ44; informational, read-only): what this
  // runner's com.apple.symbolichotkeys holds — a CI runner holds nothing personal.
  const plist = join(out, "symbolichotkeys.plist");
  run("defaults", ["export", "com.apple.symbolichotkeys", plist]);
  if (existsSync(plist)) {
    run("plutil", ["-convert", "json", "-o", join(out, "symbolichotkeys.json"), plist]);
  }
  console.log(`symbolichotkeys: ${existsSync(join(out, "symbolichotkeys.json")) ? "exported" : "none"}`);
}
const symbolicHotkeys = readIfThere(join(out, "symbolichotkeys.json"));
// The downloads run (LJ15, smoke-downloads.mts): its own launch, the smoke server's attachments.
const downloadCheck = await downloadRun({
  pageUrl: smokePageUrl,
  downloadUrl: `http://127.0.0.1:${port}/${DOWNLOAD_PATH}`,
  slowUrl: `http://127.0.0.1:${port}/${SLOW_PATH}`,
  pdfUrl: `http://127.0.0.1:${port}/${PDF_PATH}`,
  downloadsDir: join(homedir(), "Downloads"),
  elsewhere: [homedir(), process.cwd()],
  launch: fullscreenLaunch,
  screenshot,
  sleep: (ms) => sleep(ms).then(() => undefined),
  slowServed: () => ({ ...slowServe }),
});
console.log(`downloads: ${JSON.stringify(downloadSummary(downloadCheck))}`);
// The tray run (LJ41, smoke-tray.mts): the icon switched off, and every way back still there.
const trayDriver: TrayDriver = {
  os,
  launch: fullscreenLaunch,
  screenshot,
  sleep: (ms) => sleep(ms).then(() => undefined),
  typeText,
  pressKeys,
  exited,
};
const trayCheck = await trayRun(trayDriver);
console.log(`tray: ${JSON.stringify(traySummary(trayCheck, os))}`);

/** Whether a command ran and exited with 0 (the timing run's cache eviction). */
function succeeded(command: string, args: string[]): boolean {
  try {
    execFileSync(command, args, { stdio: ["ignore", "pipe", "pipe"], timeout: 120_000 });
    return true;
  } catch (err) {
    console.log(`(${command} ${args.join(" ")} failed: ${(err as Error).message})`);
    return false;
  }
}

/**
 * The global shortcut (CommandOrControl+Shift+Space) as the OS would get it from a keyboard:
 * xdotool (XTEST) on Linux, SendKeys on Windows, a CGEvent from a Swift script on macOS (best
 * effort). Returns the unix ms just before the key went out (the helper's own clock where it has
 * one), or null.
 */
function pressSummonShortcut(): number | null {
  if (os === "win32") {
    const sent = powershell(`Add-Type -AssemblyName System.Windows.Forms
$t = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
[System.Windows.Forms.SendKeys]::SendWait('^+ ')
Write-Output $t`);
    console.log("pressed Ctrl+Shift+Space");
    return Number(sent.trim().split(/\s+/).at(-1)) || null;
  }
  if (os === "darwin") {
    const swift = join(out, "summon.swift");
    writeFileSync(
      swift,
      `import AppKit
print(Int64(Date().timeIntervalSince1970 * 1000))
for down in [true, false] {
  let event = CGEvent(keyboardEventSource: nil, virtualKey: 49, keyDown: down)
  event?.flags = [.maskCommand, .maskShift]
  event?.post(tap: .cghidEventTap)
  usleep(60000)
}
let release = CGEvent(keyboardEventSource: nil, virtualKey: 55, keyDown: false)
release?.flags = []
release?.post(tap: .cghidEventTap)
`,
    );
    const sent = run("swift", [swift]);
    console.log("pressed Cmd+Shift+Space");
    return Number(sent.trim().split(/\s+/)[0]) || null;
  }
  const sent = Date.now();
  run("xdotool", ["key", "ctrl+shift+space"]);
  console.log("pressed Ctrl+Shift+Space");
  return sent;
}

// The summon-timing run (LJ19.1, smoke-timing.mts): its own launch, without gdb, on a fresh data
// folder with a seeded history.json.
const timingDriver: TimingDriver = {
  os,
  pageUrl: smokePageUrl,
  launch: (name, settings, files) => {
    const logFile = join(out, `smoke-${name}.log`);
    rmSync(logFile, { force: true });
    const runDir = dataDir(name, true, settings);
    for (const [file, text] of Object.entries(files)) {
      writeFileSync(join(runDir, file), text);
    }
    const runEnv = { ...env, LITTLEJOAN_DATA_DIR: runDir, LITTLEJOAN_SMOKE_LOG: logFile };
    const runChild = launch(runEnv, false);
    return Promise.resolve({
      pid: () => appPid(runChild),
      settings: () => readIfThere(join(runDir, "settings.json")),
      forward: (args) => forward(args, runEnv),
      events: () => events(logFile),
      waitFor: (what, test, timeoutMs) => waitFor(what, test, timeoutMs, logFile),
      quit: async () => {
        const runPid = appPid(runChild);
        await forward(["--quit"], runEnv);
        if (!(await exited(runPid, 15_000))) {
          await quit(runChild, runPid);
        }
      },
    });
  },
  sleep: (ms) => sleep(ms).then(() => undefined),
  evict: () => {
    if (os === "darwin") {
      return succeeded("sudo", ["-n", "purge"]) ? "purge" : "failed";
    }
    if (os === "linux") {
      return succeeded("sudo", ["-n", "sh", "-c", "sync; echo 3 > /proc/sys/vm/drop_caches"]) ? "drop_caches" : "failed";
    }
    return "none";
  },
  pressSummonShortcut,
};
const timingCheck = await timingRun(timingDriver);
const timingMd = timingTable(timingCheck, os);
writeFileSync(join(out, "timing.md"), timingMd);
console.log(timingMd);
if (process.env.GITHUB_STEP_SUMMARY) {
  appendFileSync(process.env.GITHUB_STEP_SUMMARY, `\n${timingMd}`);
}
// The over-full-screen run (LJ19.2, smoke-spaces.mts): its own launch, summoned over another app's
// full-screen window.
const spacesCheck = await spacesRun({
  os,
  out,
  launch: fullscreenLaunch,
  screenshot,
  sleep: (ms) => sleep(ms).then(() => undefined),
  clickCentre,
  typeText,
  pressSummonShortcut,
  run,
});
console.log(`spaces: ${JSON.stringify(spacesSummary(spacesCheck, os))}`);
// The rest runs (LJ19.4, smoke-rest.mts): one launch per "After closing" mode.
const restRuns: RestRun[] = [];
for (const mode of ["ready", "hibernate", "ready15"] as const) {
  restRuns.push(
    await restRun(
      {
        os,
        pageUrl: smokePageUrl,
        launch: (name, settings, extra) => {
          const logFile = join(out, `smoke-${name}.log`);
          rmSync(logFile, { force: true });
          const runDir = dataDir(name, true, settings);
          const runEnv = { ...env, ...extra, LITTLEJOAN_DATA_DIR: runDir, LITTLEJOAN_SMOKE_LOG: logFile };
          const runChild = launch(runEnv, false);
          return Promise.resolve({
            pid: () => appPid(runChild),
            settings: () => readIfThere(join(runDir, "settings.json")),
            forward: (args) => forward(args, runEnv),
            events: () => events(logFile),
            waitFor: (what, test, timeoutMs) => waitFor(what, test, timeoutMs, logFile),
            quit: async () => {
              const runPid = appPid(runChild);
              await forward(["--quit"], runEnv);
              if (!(await exited(runPid, 15_000))) {
                await quit(runChild, runPid);
              }
            },
          });
        },
        sleep: (ms) => sleep(ms).then(() => undefined),
        pings: (id) => pingLog.filter((p) => p.id === id).map((p) => p.t),
        engine: engineProcs,
        enginePids: enginePidsNow,
        appRssKb: (runPid) => Math.round(memoryMb(runPid) * 1024),
        kill: killPids,
        pressSummonShortcut,
        typeText,
        screenshot,
      },
      mode,
    ),
  );
}
const restMd = restTable(restRuns, os);
writeFileSync(join(out, "rest.md"), restMd);
console.log(restMd);
if (process.env.GITHUB_STEP_SUMMARY) {
  appendFileSync(process.env.GITHUB_STEP_SUMMARY, `\n${restMd}`);
}
server.close();

// First launch: an empty data folder means a first run — the window opens once with the welcome
// and the default settings are saved; the next launch on the same folder hibernates.
const firstDir = dataDir("first-run", false);
const firstLogPath = join(out, "smoke-first-run.log");
const relaunchLogPath = join(out, "smoke-relaunch.log");
rmSync(firstLogPath, { force: true });
rmSync(relaunchLogPath, { force: true });
const firstEnv = { ...env, LITTLEJOAN_DATA_DIR: firstDir, LITTLEJOAN_SMOKE_LOG: firstLogPath };
const firstChild = launch(firstEnv);
await waitFor(
  "first launch: first-run, window-created, bar-loaded, welcome",
  (e) => ["first-run", "window-created", "bar-loaded", "welcome"].every((event) => count(e, event) > 0),
  30_000,
  firstLogPath,
);
await sleep(1500);
screenshot("welcome.png");
await forward(["--hide"], firstEnv);
await waitFor("first launch: hibernated", (e) => count(e, "hibernated") > 0, 20_000, firstLogPath);
await quit(firstChild, appPid(firstChild));
const settingsSaved = existsSync(join(firstDir, "settings.json"));
const relaunchChild = launch({ ...firstEnv, LITTLEJOAN_SMOKE_LOG: relaunchLogPath });
await waitFor("relaunch: started", (e) => count(e, "started") > 0, 30_000, relaunchLogPath);
// No window may open within 5 s of starting.
await sleep(5000);
const relaunch = events(relaunchLogPath);
await quit(relaunchChild, appPid(relaunchChild));
const firstRun = { first: events(firstLogPath), relaunch, settingsSaved };
console.log(
  `first launch: ${firstRun.first.map((e) => e.event).join(", ")}; relaunch: ${relaunch.map((e) => e.event).join(", ")}; settings saved: ${settingsSaved}`,
);

const log = events();
const persistCheck = { afterClose: persistAfterClose, afterQuit: persistAfterQuit, quitExited, relaunch: afterQuitEvents };
const lastSet = log.findLast((e) => e.event === "title" && e.detail === "LJ-PERSIST set");
const quitLogged = log.findLast((e) => e.event === "quit");
const pageFullscreen = lastTitle(log, "LJ-SMOKE fullscreen=on") !== undefined;
const summary = {
  os,
  app,
  // The main run stays ready (LJ19.4's default): the app's own memory at rest with its bar.
  memoryMb: { readyAtStart: idleMb, withSmokePage: shownMb, readyAfterPage: afterMb },
  ownMemoryMb: { readyAtStart: idleOwnMb, withSmokePage: shownOwnMb, readyAfterPage: afterOwnMb },
  // afterHibernate / afterEachCycle: how many of the helpers that served the page are still alive
  // after a close (atStart: the ready bar's).
  webviewHelpers: {
    atStart: helpersAtStart.length,
    withPage: pageHelpers.length,
    afterHibernate: helperSamples,
    afterEachCycle: helpersPerCycle,
  },
  extraEnv,
  codecs: parseCodecs(lastTitle(log, "LJ-SMOKE ready")),
  fullscreen: { page: pageFullscreen, window: count(log, "fullscreen", "true") > 0 },
  shortcut: log.find((e) => e.event.startsWith("shortcut-"))?.event ?? "none",
  startAtLogin: {
    reported: log.filter((e) => e.event.startsWith("start-at-login")).map((e) => `${e.event} ${e.detail}`),
    entriesAfterOn: loginAfterOn,
    entriesAfterOff: loginAfterOff,
  },
  firstRun: {
    firstLaunch: firstRun.first.map((e) => e.event),
    relaunch: relaunch.map((e) => e.event),
    settingsSaved,
  },
  commandBar: {
    topSite: sawSmokeTopSite(log),
    typedMatch: sawTypedMatch(log),
    typingDriven,
    historyView: sawHistoryView(log),
  },
  settings: settingsTabs(historyOffEvents),
  extensions: {
    hello: lastTitle(extensionCheck.events, "LJ-EXT hello") ?? null,
    probe: lastTitle(extensionCheck.events, "LJ-EXTPROBE") ?? null,
    refused: extensionCheck.events.filter((e) => e.event === "ljext-refused" || e.event === "navigation-refused").map((e) => `${e.event} ${e.detail}`),
    store: parseStore(extensionCheck.store) ?? null,
    answerCopied: clipboard,
  },
  bundled: {
    count: bundledCount,
    up: [...BUNDLED_PANELS, ...PAGE_ACTIONS].filter((panel) => panelUp(bundledCheck.events, panel.id)).map((panel) => panel.id),
    keysReached: bundledCheck.keys,
  },
  fullscreenRun: fullscreenSummary(fullscreenCheck, os, fullscreenOff),
  // LJ14.1: element fullscreen switched off (expected on WebKitGTK older than 2.52 only).
  fullscreenOff: { expected: fullscreenOff, versions: fullscreenOffVersions(log) },
  resizeRun: { steps: resizeSummary(resizeCheck.results), rules: resizeFailures(RESIZE_STEPS, resizeCheck.results) },
  siteModeRun: siteModeSummary(siteModeCheck, os),
  quickRun: quickSummary(quickCheck, os),
  keymapRun: keymapSummary(keymapCheck, os),
  keyboardProblem: {
    events: keyboardProblem.map((e) => `${e.event} ${e.detail}`.trim()).filter((line) => /^(shortcut|settings|system-shortcuts)/.test(line)),
    rules: keyboardProblemFailures(keyboardProblem).concat(systemShortcutFailures(keyboardProblem, symbolicHotkeys, os)),
  },
  trayRun: traySummary(trayCheck, os),
  // LJ19.1: the timing run, and the main run's summons (its first is the runner's first window).
  timingRun: timingSummary(timingCheck, os),
  mainRunTimings: summonTimings(log),
  // LJ19.2: summoned over another app's full-screen window.
  spacesRun: spacesSummary(spacesCheck, os),
  // LJ19.4: what stays at rest per "After closing" mode.
  restRuns: restRuns.map(restSummary),
  downloads: downloadSummary(downloadCheck),
  zoom: zoomSummary(zoomEvents, WIDE, os !== "darwin"),
  themes: themeRuns.map(({ run, events: runEvents }) => ({
    name: run.name,
    theme: run.theme,
    handed: runEvents.map(parseTheme).filter((t) => t !== undefined),
  })),
  history: {
    mainRecords: parseHistory(historyMain)?.length ?? null,
    smokePage: parseHistory(historyMain)?.find((record) => record.url === smokePageUrl) ?? null,
    offRecords: historyOff === undefined ? "no file" : (parseHistory(historyOff)?.length ?? "not history"),
  },
  persist: {
    afterCloseAndSummon: persistAfterClose ?? null,
    afterQuitAndRelaunch: persistAfterQuit ?? null,
    quitExitedByItself: quitExited,
    quitMsAfterSet: lastSet && quitLogged ? quitLogged.millis - lastSet.millis : null,
  },
};
writeFileSync(join(out, "summary.json"), `${JSON.stringify(summary, null, 2)}\n`);
console.log(JSON.stringify(summary, null, 2));

// macOS runners may refuse synthetic clicks (no Accessibility permission), so fullscreen is
// required there only if the click visibly reached the page; Windows and Linux must pass.
const failures = evaluate(
  log,
  { windowFullscreen: windowWanted, pageFullscreen: windowWanted, viewports: [NARROW, WIDE] },
  {
    pageHelpersLeft: Math.max(...helpersPerCycle),
    loginEntry: { afterOn: loginAfterOn, afterOff: loginAfterOff, eventsUntil: loginEventsUntil },
  },
)
  .concat(commandBarFailures(log, { typing: typingDriven }))
  .concat(historyViewFailures(log, afterCtrlY ? { driven: typingDriven, afterCtrlY } : { driven: typingDriven }))
  .concat(persistFailures(log, persistCheck))
  .concat(historyFailures(historyCheck))
  .concat(settingsFailures(historyOffEvents))
  .concat(extensionFailures(extensionCheck.events, parseStore(extensionCheck.store)))
  .concat(raceFailures(extensionCheck.events))
  .concat(barReadyFailures(extensionCheck.events))
  .concat(commandHelloFailures(extensionCheck.events, { driven: typingDriven }))
  .concat(settingsExtensionsFailures(historyOffEvents, bundledCount))
  .concat(bundledFailures(bundledCheck.events, BUNDLED_PANELS))
  .concat(pageActionFailures(bundledCheck.events, PAGE_ACTIONS, { driven: typingDriven }))
  .concat(themeRuns.flatMap(({ run, events: runEvents }) => themeFailures(run, runEvents)))
  .concat(fullscreenFailures(fullscreenCheck, os, fullscreenOff))
  .concat(fullscreenOffFailures(fullscreenOff, { main: log, fullscreen: fullscreenCheck }))
  .concat(resizeFailures(RESIZE_STEPS, resizeCheck.results))
  .concat(siteModeFailures(siteModeCheck, os))
  .concat(quickFailures(quickCheck, os))
  .concat(keymapFailures(keymapCheck, os))
  .concat(keyboardProblemFailures(keyboardProblem))
  .concat(systemShortcutFailures(keyboardProblem, symbolicHotkeys, os))
  .concat(trayFailures(trayCheck, os))
  .concat(timingFailures(timingCheck, os))
  .concat(spacesFailures(spacesCheck, os))
  .concat(restRuns.flatMap((restCheck) => restFailures(restCheck, os)))
  .concat(downloadFailures(downloadCheck, os))
  .concat(zoomFailures(zoomEvents, WIDE, { driven: os !== "darwin" }))
  .concat(firstRunFailures(firstRun));
if (failures.length > 0) {
  console.log("\nSMOKE FAILED:\n- " + failures.join("\n- "));
  const themeLogs = THEME_RUNS.map((run) => join(out, `smoke-${run.name}.log`));
  const extensionLog = join(out, "smoke-extensions.log");
  const bundledLog = join(out, "smoke-bundled.log");
  const fullscreenLogs = [join(out, "smoke-fullscreen.log"), join(out, "smoke-fullscreen-site.log"), join(out, "smoke-resize.log"), join(out, "smoke-site-mode.log")];
  const trayLogs = [
    "quick",
    "keymap",
    "keyboard-problem",
    "downloads",
    "tray",
    "tray-settings",
    "tray-wayland",
    "timing-ready",
    "timing",
    "spaces",
    "rest-ready",
    "rest-hibernate",
    "rest-ready15",
  ].map((name) => join(out, `smoke-${name}.log`));
  for (const path of [logPath, afterQuitLogPath, historyOffLogPath, extensionLog, bundledLog, ...themeLogs, ...fullscreenLogs, ...trayLogs, firstLogPath, relaunchLogPath]) {
    console.log(`\n--- ${path} ---\n${existsSync(path) ? readFileSync(path, "utf8") : "(none)"}`);
  }
  process.exit(1);
}
console.log("\nSMOKE PASSED");
process.exit(0);
