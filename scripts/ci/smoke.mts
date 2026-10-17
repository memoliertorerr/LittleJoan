// CI smoke test of the INSTALLED app (what users download, not the build folder).
//
//   node scripts/ci/smoke.mts --app <installed binary> --out <dir>            full run
//   node scripts/ci/smoke.mts --app <binary> --out <dir> --started-only      launch check only
//
// The full run: serve tests/smoke/ on 127.0.0.1, launch the app with LITTLEJOAN_SMOKE_LOG,
// summon it with `--show` (single-instance forwarding), load the smoke page, click its big
// fullscreen button, hibernate with `--hide`, summon and hibernate once more, turn start at
// login on and off (reading the OS's own login entry each time), check that cookies and
// localStorage outlive closing the window and quitting (`--quit`, then a relaunch on the same
// data folder), that history.json recorded the smoke page (and that a run with history off
// records nothing), then check the logs (scripts/ci/smoke-log.mts) and the webview helper
// processes. The app runs on a fresh
// LITTLEJOAN_DATA_DIR that already holds the default settings.json, so it starts hibernating
// as every launch after the first does. Last, the first-launch check: quit, launch on an empty
// data folder (the window opens once with the welcome), --hide, quit, relaunch on the same
// folder (it must hibernate). Screenshots and a summary land in <out>. Linux runs inside
// xvfb-run + dbus-run-session with a window manager.
import { spawn, execFileSync, type ChildProcess } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { createServer, type Server } from "node:http";
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

const root = resolve(import.meta.dirname, "..", "..");
const pageDir = join(root, "tests", "smoke");
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

/** Clicks the centre of the primary screen, where the smoke page's big button sits. */
function clickCentre(): void {
  if (os === "win32") {
    powershell(`Add-Type -AssemblyName System.Windows.Forms
Add-Type -Namespace LJ -Name Mouse -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y); [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint x, uint y, uint d, System.UIntPtr e);'
$a = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea
[LJ.Mouse]::SetCursorPos($a.Left + [int]($a.Width / 2), $a.Top + [int]($a.Height / 2)) | Out-Null
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
let p = CGPoint(x: f.midX, y: f.midY)
for t in [CGEventType.leftMouseDown, CGEventType.leftMouseUp] {
  CGEvent(mouseEventSource: nil, mouseType: t, mouseCursorPosition: p, mouseButton: .left)?.post(tap: .cghidEventTap)
  usleep(80000)
}
`,
    );
    run("swift", [swift]);
  } else {
    const [width, height] = run("xdotool", ["getdisplaygeometry"]).trim().split(/\s+/).map(Number);
    run("xdotool", ["mousemove", String(Math.round((width ?? 1280) / 2)), String(Math.round((height ?? 800) / 2)), "click", "1"]);
  }
  console.log("clicked the screen centre");
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

function serve(): Promise<Server> {
  const types: Record<string, string> = { ".html": "text/html; charset=utf-8", ".mjs": "text/javascript" };
  const server = createServer((req, res) => {
    const name = (req.url ?? "/").split("?")[0]?.replace(/^\//, "") ?? "";
    const file = join(pageDir, name);
    if (!/^[\w.-]+$/.test(name) || !types[extname(name)] || !existsSync(file)) {
      res.writeHead(404).end();
      return;
    }
    res.writeHead(200, { "content-type": types[extname(name)] ?? "text/plain" }).end(readFileSync(file));
  });
  return new Promise((done) => server.listen(0, "127.0.0.1", () => done(server)));
}

/** Launches the app; with LJ_SMOKE_GDB=1 (Linux) it runs under gdb so a crash prints a backtrace. */
function launch(withEnv: NodeJS.ProcessEnv = env): ChildProcess {
  const underGdb = os === "linux" && process.env.LJ_SMOKE_GDB === "1";
  const [command, args] = underGdb
    ? [
        "gdb",
        ["-q", "-batch", "-ex", "run", "-ex", "bt 40", "-ex", "thread apply all bt 8", "--args", app as string],
      ]
    : [app as string, []];
  const child = spawn(command, args, { env: withEnv, stdio: ["ignore", "pipe", "pipe"] });
  child.stdout?.on("data", (chunk: Buffer) => process.stdout.write(`[app] ${chunk.toString()}`));
  child.stderr?.on("data", (chunk: Buffer) => process.stdout.write(`[app] ${chunk.toString()}`));
  child.on("exit", (code, signal) => console.log(`app exited with code ${code} signal ${signal}`));
  return child;
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

/** The pid of the app itself (under gdb, the debuggee is gdb's child). */
function appPid(child: ChildProcess): number {
  if (os === "linux" && process.env.LJ_SMOKE_GDB === "1") {
    const found = run("pgrep", ["-n", "-x", "little-joan"]).trim();
    return Number(found) || -1;
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
await sleep(3000);
const pid = appPid(child);
const idleMb = memoryMb(pid);
const idleOwnMb = ownMemoryMb(pid);
const helpersAtStart = helperPids();
console.log(`hibernating at start: ${idleMb} MB resident, ${idleOwnMb} MB own, ${helpersAtStart.length} webview helpers`);
screenshot("hibernating.png");

await forward(["--show"]);
await waitFor("window-created", (e) => count(e, "window-created") > 0, 20_000);
await waitFor("start page title", (e) => count(e, "title", "Little Joan") > 0, 20_000);
await sleep(1500);
screenshot("start.png");

await forward(["--show", `http://127.0.0.1:${port}/smoke.html`]);
await waitFor("smoke page ready", (e) => lastTitle(e, "LJ-SMOKE ready") !== undefined, 30_000);
await sleep(1500);
const shownMb = memoryMb(pid);
const shownOwnMb = ownMemoryMb(pid);
const pageHelpers = newPids(helpersAtStart, helperPids());
writeFileSync(join(out, "helpers-with-page.txt"), helperListing());
screenshot("page.png");

clickCentre();
const windowWanted = os !== "darwin";
await waitFor(
  "fullscreen",
  (e) => lastTitle(e, "LJ-SMOKE fullscreen=on") !== undefined && (!windowWanted || count(e, "fullscreen", "true") > 0),
  15_000,
);
await sleep(2000);
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
console.log(`hibernating after a page: ${afterMb} MB resident, ${afterOwnMb} MB own, helpers ${JSON.stringify(helperSamples)}`);

// Two more summon/hibernate cycles: helper counts that grow per cycle would mean webviews leak.
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
const historyOffDir = dataDir("history-off", true, { rememberHistory: false });
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
await forward(["--quit"], historyOffEnv);
if (!(await exited(historyOffPid, 15_000))) {
  await quit(historyOffChild, historyOffPid);
}
const historyOff = readIfThere(join(historyOffDir, "history.json"));
const historyCheck = { main: historyMain, pageUrl: smokePageUrl, off: historyOff };
console.log(`history: main ${historyMain === undefined ? "no file" : `${historyMain.length} bytes`}, off ${historyOff ?? "no file"}`);
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
  memoryMb: { hibernatingAtStart: idleMb, withSmokePage: shownMb, hibernatingAfterPage: afterMb },
  ownMemoryMb: { hibernatingAtStart: idleOwnMb, withSmokePage: shownOwnMb, hibernatingAfterPage: afterOwnMb },
  // afterHibernate / afterEachCycle: how many of the helpers that served the page are still alive.
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
  .concat(persistFailures(log, persistCheck))
  .concat(historyFailures(historyCheck))
  .concat(firstRunFailures(firstRun));
if (failures.length > 0) {
  console.log("\nSMOKE FAILED:\n- " + failures.join("\n- "));
  for (const path of [logPath, afterQuitLogPath, historyOffLogPath, firstLogPath, relaunchLogPath]) {
    console.log(`\n--- ${path} ---\n${existsSync(path) ? readFileSync(path, "utf8") : "(none)"}`);
  }
  process.exit(1);
}
console.log("\nSMOKE PASSED");
process.exit(0);
