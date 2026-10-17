// CI smoke test of the INSTALLED app (what users download, not the build folder).
//
//   node scripts/ci/smoke.mts --app <installed binary> --out <dir>            full run
//   node scripts/ci/smoke.mts --app <binary> --out <dir> --started-only      launch check only
//
// The full run: serve tests/smoke/ on 127.0.0.1, launch the app with LITTLEJOAN_SMOKE_LOG,
// summon it with `--show` (single-instance forwarding), load the smoke page, click its big
// fullscreen button, hibernate with `--hide`, summon and hibernate once more, then check the
// log (scripts/ci/smoke-log.mts) and the webview helper processes. Screenshots and a summary
// land in <out>. Linux runs inside xvfb-run + dbus-run-session with a window manager.
import { spawn, execFileSync, type ChildProcess } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { extname, join, resolve } from "node:path";
import {
  count,
  evaluate,
  lastTitle,
  newPids,
  parseCodecs,
  parseLog,
  sawViewport,
  survivors,
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
const env = { ...process.env, ...extraEnv, LITTLEJOAN_SMOKE_LOG: logPath };

const sleep = (ms: number) => new Promise((done) => setTimeout(done, ms));

function events(): LogEvent[] {
  return existsSync(logPath) ? parseLog(readFileSync(logPath, "utf8")) : [];
}

async function waitFor(what: string, test: (e: LogEvent[]) => boolean, timeoutMs = 30_000): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (test(events())) {
      console.log(`ok: ${what}`);
      return true;
    }
    await sleep(250);
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

/** Runs a second instance with `args`; single-instance forwards them and it exits. */
async function forward(args: string[]): Promise<void> {
  console.log(`forward: ${args.join(" ")}`);
  const child = spawn(app as string, args, { env, stdio: "ignore" });
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
function launch(): ChildProcess {
  const underGdb = os === "linux" && process.env.LJ_SMOKE_GDB === "1";
  const [command, args] = underGdb
    ? [
        "gdb",
        ["-q", "-batch", "-ex", "run", "-ex", "bt 40", "-ex", "thread apply all bt 8", "--args", app as string],
      ]
    : [app as string, []];
  const child = spawn(command, args, { env, stdio: ["ignore", "pipe", "pipe"] });
  child.stdout?.on("data", (chunk: Buffer) => process.stdout.write(`[app] ${chunk.toString()}`));
  child.stderr?.on("data", (chunk: Buffer) => process.stdout.write(`[app] ${chunk.toString()}`));
  child.on("exit", (code, signal) => console.log(`app exited with code ${code} signal ${signal}`));
  return child;
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

child.kill();
server.close();

const log = events();
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
};
writeFileSync(join(out, "summary.json"), `${JSON.stringify(summary, null, 2)}\n`);
console.log(JSON.stringify(summary, null, 2));

// macOS runners may refuse synthetic clicks (no Accessibility permission), so fullscreen is
// required there only if the click visibly reached the page; Windows and Linux must pass.
const failures = evaluate(
  log,
  { windowFullscreen: windowWanted, pageFullscreen: windowWanted, viewports: [NARROW, WIDE] },
  { pageHelpersLeft: Math.max(...helpersPerCycle) },
);
if (failures.length > 0) {
  console.log("\nSMOKE FAILED:\n- " + failures.join("\n- "));
  console.log(`\n--- smoke log ---\n${existsSync(logPath) ? readFileSync(logPath, "utf8") : "(none)"}`);
  process.exit(1);
}
console.log("\nSMOKE PASSED");
process.exit(0);
