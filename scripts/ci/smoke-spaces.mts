// The smoke test's over-full-screen run (LJ19.2) and its rules. Another app's full-screen window
// comes up first (smoke-spaces-helpers.mts: macOS — a small Swift app in native full screen, its
// own Space; Windows — a borderless WinForms window covering the screen, not topmost; Linux — an
// X11 window with _NET_WM_STATE_FULLSCREEN under openbox) and is made the active window (a click
// on it; on macOS its own activation). Then Little Joan, on a launch of its own, is summoned over
// it three times, each after the full-screen window had the focus again:
//   (a) `--show`;
//   (b) after a letter typed into its command bar (Windows, Linux: the app has seen a keystroke —
//       an old timestamp Linux window managers would refuse), `--hide`, the real global shortcut
//       (`--show` when its key didn't arrive);
//   (c) after `--hide`, `--settings`.
// Each must bring the window up in front with the keyboard: Windows and Linux — the app's own
// `window-front <label> focused=true top=true` (smoke mode, ~1 s after the summon), and from
// outside the active window belonging to the app (the helper's GetForegroundWindow; xdotool),
// with the full-screen window active just before. macOS — every window's `window-spaces` line
// shows moveToActiveSpace and fullScreenAuxiliary at the normal level, and where the helper
// reached its full-screen Space, that Space is still the one on screen after each summon (no Space
// switch) and Little Joan's window is on it (`space=true`); whether it is in front and has the
// keyboard is recorded — CI's summons are synthetic, and macOS 14+ may decline to activate an app
// for them —, as are three probes after the launch (an accessory window with AppKit's default
// behaviour, with Little Joan's, and with Little Joan's + canJoinAllApplications, each ordered
// front over the full-screen window): what this runner does without the setting. Screenshots
// over-fullscreen.png, over-fullscreen-shortcut.png, over-fullscreen-settings.png. smoke.mts
// supplies the driver; the rules are pure and unit-tested (smoke-spaces.test.ts).
//
// CI experiments (LJ19.6): `LJ_SPACES_REPEAT=<n>` in the workflow's `extra_env` runs the launch n
// times in one job (`spacesRepeat`), each a fresh helper in full screen and a fresh Little Joan
// launch (`smoke-spaces-<i>.log`, no probes or screenshots after the first), and reports each
// one's rules (`repeatSummary`): a failure seen about once in 20 macOS runs becomes countable.
// Off by default (one launch).
import { spawn, type ChildProcess } from "node:child_process";
import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { LogEvent } from "./smoke-log.mts";
import type { SiteApp } from "./smoke-sitemode.mts";
import { LINUX_HELPER, MAC_HELPER, WINDOWS_HELPER } from "./smoke-spaces-helpers.mts";

/** What the run needs from smoke.mts. */
export interface SpacesDriver {
  os: string;
  /** Where the helper's script and status file go (the smoke's output folder). */
  out: string;
  launch(name: string, settings: Record<string, unknown>): Promise<SiteApp>;
  screenshot(name: string): void;
  sleep(ms: number): Promise<void>;
  /** Clicks the centre of the primary screen, which the full-screen window covers. */
  clickCentre(): void;
  typeText(text: string, pid: number, activate: boolean): void;
  /** Presses the global shortcut as a keyboard would. */
  pressSummonShortcut(): number | null;
  /** Runs a command and returns its output ("" when it failed). */
  run(command: string, args: string[]): string;
}

/** One line of the helper's status file (fields per OS: smoke-spaces-helpers.mts). */
export interface HelperStatus {
  t: number;
  state: string;
  pid?: number;
  fullscreen?: boolean;
  onActiveSpace?: boolean;
  active?: boolean;
  key?: boolean;
  top?: boolean;
  frontPid?: number;
  activeWindow?: number;
  xid?: number;
  bits?: number;
  behavior?: number;
  level?: number;
}

/** The status file's lines; a line cut short by a write under way is skipped. */
export function parseStatus(text: string): HelperStatus[] {
  const lines: HelperStatus[] = [];
  for (const line of text.split(/\r?\n/)) {
    try {
      const value = JSON.parse(line.replace(/^\uFEFF/, "")) as Partial<HelperStatus>;
      if (typeof value.t === "number" && typeof value.state === "string") {
        lines.push(value as HelperStatus);
      }
    } catch {
      // Not a whole line.
    }
  }
  return lines;
}

/** A `window-front` line (platform/mod.rs report_front). */
export interface FrontLine {
  label: string;
  /** The window was gone when the app looked. */
  unknown: boolean;
  focused?: boolean;
  top?: boolean;
  space?: boolean;
}

export function parseFront(detail: string): FrontLine | undefined {
  const [label, ...fields] = detail.trim().split(/\s+/);
  if (!label) {
    return undefined;
  }
  const line: FrontLine = { label, unknown: fields[0] === "unknown" };
  for (const field of fields) {
    const match = /^(focused|top|space)=(true|false)$/.exec(field);
    if (match?.[1]) {
      line[match[1] as "focused" | "top" | "space"] = match[2] === "true";
    }
  }
  return line;
}

/** The last `window-front` line for `label` among `events`. */
export function frontOf(events: LogEvent[], label: string): FrontLine | undefined {
  return events
    .filter((e) => e.event === "window-front")
    .map((e) => parseFront(e.detail))
    .findLast((line) => line?.label === label);
}

/** A `window-spaces` line (macOS: platform/mac/spaces.rs). */
export interface SpacesLine {
  label: string;
  behavior: string[];
  was: string[];
  level: number;
}

export function parseSpaces(detail: string): SpacesLine | undefined {
  const match = /^(\S+) behavior=(\S+) was=(\S+) level=(-?\d+)$/.exec(detail.trim());
  if (!match?.[1] || !match[2] || !match[3]) {
    return undefined;
  }
  return { label: match[1], behavior: match[2].split("|"), was: match[3].split("|"), level: Number(match[4]) };
}

/** One summon over the full-screen window. */
export interface Summon {
  name: "show" | "shortcut" | "settings";
  /** `--show` / `--settings`; the real shortcut; `--show` because the shortcut's key didn't arrive. */
  trigger: "cli" | "shortcut" | "cli-fallback";
  label: "joan" | "settings";
  /** The full-screen window was the active one just before (from outside the app). */
  helperActiveBefore: boolean;
  /** The app's log from just before the summon. */
  events: LogEvent[];
  /** The helper's last status line, about 2 s after the summon. */
  helperAfter: HelperStatus | null;
  /** The process of the active (foreground) window then: the helper's word, or xdotool's. */
  activePid: number | null;
}

/** macOS: an accessory window with collection behaviour `bits`, ordered front over the helper. */
export interface Probe {
  bits: number;
  probe: HelperStatus | null;
  /** The helper's status while the probe was up. */
  helperDuring: HelperStatus | null;
}

/** What the run showed. */
export interface SpacesRun {
  /** The helper's line saying its window is full screen (none: it never got there). */
  helperUp: HelperStatus | null;
  appPid: number;
  summons: Summon[];
  /** The launch's `window-spaces` lines (macOS). */
  spaces: LogEvent[];
  probes: Probe[];
}

/** The collection behaviour bits the macOS probes try: AppKit's default, Little Joan's, + canJoinAllApplications. */
export const PROBE_BITS = [0, (1 << 1) | (1 << 8), (1 << 1) | (1 << 8) | (1 << 18)];

/**
 * The rules. Windows and Linux: the helper came up full screen and was active before every summon,
 * and every summon's window is focused and on top, the active window being the app's. macOS: each
 * window's `window-spaces` (moveToActiveSpace + fullScreenAuxiliary, never canJoinAllSpaces, level
 * 0); once the helper is full screen, its Space stays on screen through every summon and the
 * summoned window is on it.
 */
export function spacesFailures(run: SpacesRun, os: string): string[] {
  const failures: string[] = [];
  const mac = os === "darwin";
  if (!mac && !run.helperUp) {
    failures.push("over-fullscreen: the full-screen helper window never came up full screen");
  }
  for (const summon of run.summons) {
    const what = `over-fullscreen (${summon.name})`;
    const front = frontOf(summon.events, summon.label);
    if (!front || front.unknown) {
      failures.push(`${what}: no window-front line for ${summon.label}`);
      continue;
    }
    if (mac) {
      if (run.helperUp && summon.helperAfter?.onActiveSpace !== true) {
        failures.push(`${what}: the full-screen app's Space was left (the summon switched Spaces)`);
      }
      if (run.helperUp && front.space !== true) {
        failures.push(`${what}: ${summon.label} isn't on the Space on screen (space=${String(front.space)})`);
      }
      continue;
    }
    if (!summon.helperActiveBefore) {
      failures.push(`${what}: the full-screen window wasn't the active window before the summon`);
    }
    if (front.focused !== true || front.top !== true) {
      failures.push(`${what}: ${summon.label} came up focused=${String(front.focused)} top=${String(front.top)}, not in front with the keyboard`);
    }
    if (summon.activePid !== run.appPid) {
      failures.push(`${what}: the active window belongs to pid ${String(summon.activePid)}, not the app (${run.appPid})`);
    }
  }
  if (mac) {
    for (const label of new Set(run.summons.map((summon) => summon.label))) {
      const line = run.spaces.map((e) => parseSpaces(e.detail)).findLast((spaces) => spaces?.label === label);
      if (!line) {
        failures.push(`over-fullscreen: no window-spaces line for ${label}`);
      } else if (!line.behavior.includes("moveToActiveSpace") || !line.behavior.includes("fullScreenAuxiliary") || line.behavior.includes("canJoinAllSpaces")) {
        failures.push(`over-fullscreen: ${label}'s collection behaviour is ${line.behavior.join("|")}, not moveToActiveSpace + fullScreenAuxiliary`);
      } else if (line.level !== 0) {
        failures.push(`over-fullscreen: ${label}'s window level is ${line.level}, not the normal level (0)`);
      }
    }
  }
  return failures;
}

/** What summary.json records of the run. */
export function spacesSummary(run: SpacesRun, os: string) {
  const helper = (status: HelperStatus | null) =>
    status && { active: status.active, fullscreen: status.fullscreen, onActiveSpace: status.onActiveSpace, frontPid: status.frontPid, top: status.top };
  return {
    helperUp: run.helperUp !== null,
    appPid: run.appPid,
    summons: run.summons.map((summon) => ({
      name: summon.name,
      trigger: summon.trigger,
      helperActiveBefore: summon.helperActiveBefore,
      front: frontOf(summon.events, summon.label) ?? null,
      present: summon.events.filter((e) => e.event === "window-present").map((e) => e.detail),
      helperAfter: helper(summon.helperAfter),
      activePid: summon.activePid,
    })),
    spaces: run.spaces.map((e) => e.detail),
    probes: run.probes.map((probe) => ({
      bits: probe.bits,
      window: probe.probe && { onActiveSpace: probe.probe.onActiveSpace, key: probe.probe.key, active: probe.probe.active, top: probe.probe.top, behavior: probe.probe.behavior, level: probe.probe.level },
      helperDuring: helper(probe.helperDuring),
    })),
    rules: spacesFailures(run, os),
  };
}

/** The helper process and its status file. */
interface Helper {
  statusPath: string;
  child: ChildProcess;
  statuses(): HelperStatus[];
  latest(): HelperStatus | null;
  command(name: string): void;
}

function startHelper(d: SpacesDriver): Helper {
  const statusPath = join(d.out, "fs-helper.jsonl");
  rmSync(statusPath, { force: true });
  const spec: { file: string; source: string; command: string; args: (path: string) => string[] } =
    d.os === "darwin"
      ? { file: "fs-helper.swift", source: MAC_HELPER, command: "swift", args: (path) => [path, "fullscreen", statusPath, "300"] }
      : d.os === "win32"
        ? {
            file: "fs-helper.ps1",
            source: WINDOWS_HELPER,
            command: "powershell",
            args: (path) => ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", path, "-Status", statusPath, "-Seconds", "300"],
          }
        : { file: "fs-helper.py", source: LINUX_HELPER, command: "python3", args: (path) => [path, statusPath, "300"] };
  const path = join(d.out, spec.file);
  writeFileSync(path, spec.source);
  const child = spawn(spec.command, spec.args(path), { stdio: ["ignore", "pipe", "pipe"] });
  child.stdout?.on("data", (chunk: Buffer) => process.stdout.write(`[fs-helper] ${chunk.toString()}`));
  child.stderr?.on("data", (chunk: Buffer) => process.stdout.write(`[fs-helper] ${chunk.toString()}`));
  child.on("exit", (code, signal) => console.log(`fs-helper exited with code ${code} signal ${signal}`));
  child.on("error", (err) => console.log(`(fs-helper: ${err.message})`));
  let read: HelperStatus[] = [];
  const statuses = () => {
    try {
      if (existsSync(statusPath)) {
        read = parseStatus(readFileSync(statusPath, "utf8"));
      }
    } catch {
      // Windows: PowerShell's Add-Content holds the file without sharing it while it writes
      // (EBUSY, seen in CI); the last read stands until the next one.
    }
    return read;
  };
  return {
    statusPath,
    child,
    statuses,
    latest: () => statuses().findLast((s) => s.state === "status") ?? null,
    command: (name) => writeFileSync(`${statusPath}.cmd`, name),
  };
}

/** The helper's line saying its window is full screen, waiting up to `timeoutMs`. */
async function helperUp(d: SpacesDriver, helper: Helper, timeoutMs: number): Promise<HelperStatus | null> {
  for (let waited = 0; waited < timeoutMs; waited += 500) {
    const up = helper.statuses().find((s) => (d.os === "darwin" ? s.state === "fullscreen" : s.state === "status" && s.fullscreen === true));
    if (up) {
      console.log(`ok: the full-screen helper is up (${waited} ms)`);
      return up;
    }
    if (helper.child.exitCode !== null) {
      break;
    }
    await d.sleep(500);
  }
  console.log("(the full-screen helper never got to full screen)");
  return null;
}

/** Hands the focus back to the full-screen window, as the user would. */
function focusHelper(d: SpacesDriver, helper: Helper): void {
  if (d.os === "darwin") {
    helper.command("activate");
  } else {
    d.clickCentre();
  }
}

/** The active window's process now: xdotool on Linux, the helper's last word elsewhere. */
function activePid(d: SpacesDriver, helper: Helper): number | null {
  if (d.os === "linux") {
    const pid = Number(d.run("xdotool", ["getactivewindow", "getwindowpid"]).trim());
    return Number.isInteger(pid) && pid > 0 ? pid : null;
  }
  return helper.latest()?.frontPid ?? null;
}

/** How many over-full-screen launches `LJ_SPACES_REPEAT` asks for: 1 (the default) to 200. */
export function spacesRepeat(value: string | undefined): number {
  const n = Number((value ?? "").trim());
  return Number.isInteger(n) && n >= 1 ? Math.min(n, 200) : 1;
}

/** One launch of the run: its log's name (`smoke-<name>.log`), and whether it probes and screenshots. */
export interface SpacesLaunch {
  name: string;
  probes: boolean;
  shots: boolean;
}

/** The normal launch: `spaces`, with the probes and the screenshots. */
export const FIRST_LAUNCH: SpacesLaunch = { name: "spaces", probes: true, shots: true };

/** The `index`-th launch (2, 3, …) of a repeated run (LJ19.6): its own log, nothing else. */
export function repeatLaunch(index: number): SpacesLaunch {
  return { name: `spaces-${index}`, probes: false, shots: false };
}

/** What summary.json records of a repeated run: how many launches failed, and each one's rules. */
export function repeatSummary(runs: { name: string; run: SpacesRun }[], os: string) {
  const results = runs.map(({ name, run }) => ({ name, helperUp: run.helperUp !== null, rules: spacesFailures(run, os) }));
  return { launches: results.length, failed: results.filter((r) => r.rules.length > 0).length, results };
}

/** Runs the over-full-screen launch (the normal one, or one launch of a repeated run). */
export async function spacesRun(d: SpacesDriver, launch: SpacesLaunch = FIRST_LAUNCH): Promise<SpacesRun> {
  const helper = startHelper(d);
  const up = await helperUp(d, helper, d.os === "darwin" ? 45_000 : 20_000);
  const app = await d.launch(launch.name, {});
  await app.waitFor("spaces: started, shortcut", (e) => e.some((x) => x.event === "started") && e.some((x) => x.event.startsWith("shortcut-")), 30_000);
  await d.sleep(1500);
  const pid = app.pid();
  const summons: Summon[] = [];
  const hide = async () => {
    const hibernated = app.events().filter((e) => e.event === "hibernated").length;
    await app.forward(["--hide"]);
    await app.waitFor("spaces: hibernated", (e) => e.filter((x) => x.event === "hibernated").length > hibernated, 15_000);
  };
  const summon = async (name: Summon["name"], label: Summon["label"], ask: (from: number) => Promise<Summon["trigger"]>, shot: string) => {
    focusHelper(d, helper);
    await d.sleep(2500);
    const helperActiveBefore = helper.latest()?.active === true;
    const from = app.events().length;
    const trigger = await ask(from);
    await app.waitFor(`spaces: ${name}: window-front ${label}`, (e) => e.slice(from).some((x) => x.event === "window-front" && x.detail.startsWith(`${label} `)), 20_000);
    await d.sleep(800);
    if (launch.shots) {
      d.screenshot(shot);
    }
    summons.push({ name, trigger, label, helperActiveBefore, events: app.events().slice(from), helperAfter: helper.latest(), activePid: activePid(d, helper) });
  };

  await summon("show", "joan", async () => {
    await app.forward(["--show"]);
    return "cli";
  }, "over-fullscreen.png");
  if (d.os !== "darwin") {
    d.typeText("a", pid, false);
    await app.waitFor("spaces: typed into the command bar", (e) => e.some((x) => x.event === "suggestions" && x.detail.endsWith("typed=1")), 5_000);
  }
  await hide();
  await summon("shortcut", "joan", async (from) => {
    d.pressSummonShortcut();
    if (await app.waitFor("spaces: the shortcut's window", (e) => e.slice(from).some((x) => x.event === "window-created"), 10_000)) {
      return "shortcut";
    }
    await app.forward(["--show"]);
    return "cli-fallback";
  }, "over-fullscreen-shortcut.png");
  await hide();
  await summon("settings", "settings", async () => {
    await app.forward(["--settings"]);
    return "cli";
  }, "over-fullscreen-settings.png");
  const spaces = app.events().filter((e) => e.event === "window-spaces");
  await app.quit();

  const probes: Probe[] = [];
  if (d.os === "darwin" && up && launch.probes) {
    for (const bits of PROBE_BITS) {
      focusHelper(d, helper);
      await d.sleep(2000);
      const probePath = `${helper.statusPath}.probe-${bits}`;
      rmSync(probePath, { force: true });
      const child = spawn("swift", [join(d.out, "fs-helper.swift"), "probe", probePath, String(bits)], { stdio: "ignore" });
      await new Promise<void>((done) => {
        const timer = setTimeout(() => {
          child.kill();
          done();
        }, 30_000);
        child.on("exit", () => {
          clearTimeout(timer);
          done();
        });
      });
      const probe = existsSync(probePath) ? (parseStatus(readFileSync(probePath, "utf8")).find((s) => s.state === "probe") ?? null) : null;
      const helperDuring = probe ? (helper.statuses().findLast((s) => s.state === "status" && s.t <= probe.t + 250) ?? null) : null;
      console.log(`probe ${bits}: ${JSON.stringify(probe)}; helper: ${JSON.stringify(helperDuring)}`);
      probes.push({ bits, probe, helperDuring });
    }
  }

  if (d.os === "darwin") {
    helper.command("stop");
    for (let waited = 0; waited < 6_000 && helper.child.exitCode === null; waited += 250) {
      await d.sleep(250);
    }
  }
  helper.child.kill();
  await d.sleep(2000);
  return { helperUp: up, appPid: pid, summons, spaces, probes };
}
