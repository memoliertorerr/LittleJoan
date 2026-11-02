// The smoke test's Liquid Glass runs (LJ19.9) and their rules; pure parts unit-tested
// (smoke-glass.test.ts), smoke.mts drives them. The app logs, at launch, `glass-support
// available=<bool> os=<major|-> reduce=<bool> contrast=<bool>` (src-tauri/src/glass.rs); with glass,
// `glass-view <clear|regular|none> back= fills= corner= tint= dark= opaque= window-alpha= views=`
// each time the native glass changes (what AppKit reports back: platform/mac/glass.rs), the bar's
// `bar-surface <glass> html=<css> body=<css> image=<none|set>` (what it draws under itself), and a
// theme line's ` glass=<clear|tinted>`; `glass-accessibility reduce= contrast=` and `glass
// <off|clear|tinted>` when Reduce Transparency / Increase Contrast change — in CI through the smoke
// stand-in file (LITTLEJOAN_SMOKE_A11Y): a runner's own settings can't be switched.
//
// - The main run (every OS, glass Off — the default): the support line says what the OS has (macOS
//   26+ only), and Off changes nothing: no glass-view, no bar-surface, no theme line with glass.
// - The glass runs (macOS 26+ only): their own launches over a striped backdrop window
//   (MAC_BACKDROP), Clear in light and Tinted in dark, each with the smoke page under the toolbar and
//   the command bar, screenshotted into <out>/glass/; one of them also switches Reduce Transparency
//   and Increase Contrast on and off while the command bar is open.
// - The glass timing (smoke-timing.mts's `ready-glass` / `hibernate-glass` runs): summons with glass
//   no slower than without (glassTimingFailures).
import type { LogEvent } from "./smoke-log.mts";
import { count, lastTitle } from "./smoke-log.mts";
import { sawSmokeTopSite } from "./smoke-command.mts";
import type { SmokeTheme } from "./smoke-theme.mts";
import type { SiteApp } from "./smoke-sitemode.mts";
import { median, readyMs, type Measured, type RunMode } from "./smoke-timing.mts";

export type GlassName = "clear" | "tinted";

/** The `glass-support` line. */
export interface GlassSupport {
  available: boolean;
  /** The macOS major version, or null off macOS. */
  os: number | null;
  reduce: boolean;
  contrast: boolean;
}

export function parseGlassSupport(events: LogEvent[]): GlassSupport | undefined {
  const line = events.find((e) => e.event === "glass-support");
  const match = line ? /^available=(true|false) os=(\d+|-) reduce=(true|false) contrast=(true|false)$/.exec(line.detail) : null;
  if (!match) {
    return undefined;
  }
  return { available: match[1] === "true", os: match[2] === "-" ? null : Number(match[2]), reduce: match[3] === "true", contrast: match[4] === "true" };
}

/** Whether a macOS version has Liquid Glass (as settings/glass.rs: 26+, and 16 — Tahoe's compat number). */
export function versionHasGlass(major: number): boolean {
  return major >= 26 || major === 16;
}

/** A `glass-view` line: the style AppKit reports, then its key=value parts. */
export interface GlassView {
  style: "clear" | "regular" | "none";
  fields: Record<string, string>;
}

export function parseGlassView(detail: string): GlassView | undefined {
  const [style, ...parts] = detail.trim().split(/\s+/);
  if (style !== "clear" && style !== "regular" && style !== "none") {
    return undefined;
  }
  const fields: Record<string, string> = {};
  for (const part of parts) {
    const at = part.indexOf("=");
    if (at > 0) {
      fields[part.slice(0, at)] = part.slice(at + 1);
    }
  }
  return { style, fields };
}

/** The main run (glass Off): the OS reports what it has, and nothing of glass shows up. */
export function glassOffFailures(events: LogEvent[], os: string): string[] {
  const failures: string[] = [];
  const support = parseGlassSupport(events);
  if (!support) {
    return ["glass: no glass-support line at launch"];
  }
  if (os === "darwin") {
    const expected = support.os !== null && versionHasGlass(support.os);
    if (support.available !== expected) {
      failures.push(`glass: macOS ${support.os ?? "?"} reports available=${support.available}, not ${expected}`);
    }
  } else if (support.available || support.os !== null) {
    failures.push(`glass: ${os} reports glass (available=${support.available} os=${support.os ?? "-"})`);
  }
  if (events.some((e) => e.event === "glass-view")) {
    failures.push("glass: Off put a glass view into the window");
  }
  if (events.some((e) => e.event === "bar-surface")) {
    failures.push("glass: Off reported a glass surface from the bar");
  }
  const themed = events.filter((e) => /^theme(-early|-summon)?$/.test(e.event) && e.detail.includes("glass="));
  if (themed.length > 0) {
    failures.push(`glass: Off handed the bar glass (${themed.map((e) => `${e.event} ${e.detail}`).join("; ")})`);
  }
  return failures;
}

/** A switch of the stand-in accessibility file, and the glass it must leave. */
export interface AccessibilityStep {
  write: "reduce" | "contrast" | "none";
  /** What the window shows afterwards. */
  shows: "off" | GlassName;
  shot?: string;
}

export interface GlassRun {
  /** Names the data folder and the log (`smoke-<name>.log`). */
  name: string;
  theme: SmokeTheme & { mode: "light" | "dark" };
  glass: GlassName;
  /** Reduce Transparency / Increase Contrast switched while the command bar is open. */
  accessibility: readonly AccessibilityStep[];
}

const TWO_COLORS = ["#9d8fd0", "#e0a2a0"];

export const GLASS_RUNS: readonly GlassRun[] = [
  { name: "glass-clear-light", theme: { mode: "light", colors: TWO_COLORS, intensity: 0.6, grain: 0.4 }, glass: "clear", accessibility: [] },
  {
    name: "glass-tinted-dark",
    theme: { mode: "dark", colors: TWO_COLORS, intensity: 0.6, grain: 0.4 },
    glass: "tinted",
    accessibility: [
      { write: "reduce", shows: "off", shot: "reduce-transparency" },
      { write: "none", shows: "tinted", shot: "restored" },
      { write: "contrast", shows: "off" },
      { write: "none", shows: "tinted" },
    ],
  },
];

/** What a glass run's glass view must say (Clear: the clear style; Tinted: the regular one). */
export function wantedView(run: GlassRun): Record<string, string> {
  return {
    back: "true",
    fills: "true",
    tint: run.glass === "clear" ? "0.25" : "0.65",
    dark: run.theme.mode === "dark" ? "true" : "false",
    opaque: "false",
    "window-alpha": "0.00",
  };
}

function viewProblems(view: GlassView | undefined, style: GlassView["style"], wanted: Record<string, string>): string[] {
  if (!view) {
    return ["no glass-view line"];
  }
  const problems = view.style === style ? [] : [`style ${view.style}, not ${style}`];
  for (const [key, value] of Object.entries(wanted)) {
    if (view.fields[key] !== value) {
      problems.push(`${key}=${view.fields[key] ?? "?"}, not ${value}`);
    }
  }
  return problems;
}

/**
 * A glass run's rules: glass available; before any accessibility switch, every theme line handed
 * the bar its glass, the native glass is at the back of the window, filling it, in the run's style,
 * tint and scheme, with the window clear and non-opaque, and the bar draws nothing under itself;
 * after each switch the window shows what the step says (the solid theme: no glass view, the window
 * opaque again).
 */
export function glassRunFailures(run: GlassRun, events: LogEvent[]): string[] {
  const failures: string[] = [];
  const name = run.name;
  const support = parseGlassSupport(events);
  if (!support?.available) {
    return [`${name}: no Liquid Glass on this runner (glass-support ${support ? "available=false" : "missing"})`];
  }
  const firstSwitch = events.findIndex((e) => e.event === "glass-accessibility");
  const before = firstSwitch < 0 ? events : events.slice(0, firstSwitch);
  const themed = before.filter((e) => /^theme(-early|-summon)?$/.test(e.event));
  if (!themed.some((e) => e.event === "theme-early")) {
    failures.push(`${name}: the bar never painted its theme early`);
  }
  for (const line of themed) {
    if (!line.detail.endsWith(` glass=${run.glass}`)) {
      failures.push(`${name}: ${line.event} ${line.detail} — not glass=${run.glass}`);
    }
  }
  const style = run.glass === "clear" ? "clear" : "regular";
  const views = before.filter((e) => e.event === "glass-view").map((e) => parseGlassView(e.detail));
  if (views.length === 0) {
    failures.push(`${name}: no glass-view line`);
  }
  for (const view of views) {
    const problems = viewProblems(view, style, wantedView(run));
    if (problems.length > 0) {
      failures.push(`${name}: the glass view — ${problems.join(", ")}`);
    }
  }
  const surfaces = before.filter((e) => e.event === "bar-surface");
  if (surfaces.length === 0) {
    failures.push(`${name}: the bar never reported its surface over glass`);
  }
  for (const surface of surfaces) {
    if (surface.detail !== `${run.glass} html=rgba(0, 0, 0, 0) body=rgba(0, 0, 0, 0) image=none`) {
      failures.push(`${name}: the bar draws under itself over glass: ${surface.detail}`);
    }
  }
  if (!sawSmokeTopSite(events)) {
    failures.push(`${name}: the command bar never offered the smoke page's site`);
  }
  if (!lastTitle(events, "LJ-SMOKE ready")) {
    failures.push(`${name}: the smoke page never loaded`);
  }
  // Each switch: what the window shows afterwards, and the native side to match.
  const shown = events.filter((e) => e.event === "glass").map((e) => e.detail);
  const wanted = run.accessibility.map((step) => step.shows);
  if (shown.join(",") !== wanted.join(",")) {
    failures.push(`${name}: after the accessibility switches the window showed [${shown.join(", ")}], not [${wanted.join(", ")}]`);
  }
  let from = firstSwitch < 0 ? events.length : firstSwitch;
  for (const step of run.accessibility) {
    const at = events.findIndex((e, i) => i >= from && e.event === "glass");
    if (at < 0) {
      break;
    }
    const view = events.slice(at).find((e) => e.event === "glass-view");
    const problems =
      step.shows === "off"
        ? viewProblems(view ? parseGlassView(view.detail) : undefined, "none", { opaque: "true", "window-alpha": "1.00" })
        : viewProblems(view ? parseGlassView(view.detail) : undefined, style, wantedView(run));
    if (problems.length > 0) {
      failures.push(`${name}: after "${step.write}" — ${problems.join(", ")}`);
    }
    from = at + 1;
  }
  return failures;
}

/** How much slower a summon with glass may be than without (ms on top of 1.5×): CI's macOS VMs are noisy. */
export const GLASS_SLACK_MS: Partial<Record<RunMode, number>> = { "ready-glass": 150, "hibernate-glass": 750 };
const OFF_RUN: Partial<Record<RunMode, RunMode>> = { "ready-glass": "ready", "hibernate-glass": "hibernate" };

/** The warm summons' median time to usable, per run. */
export function warmMedian(run: Measured[], mode: RunMode): number | undefined {
  return median(run.flatMap((m) => (m.run === mode && m.kind === "warm" && m.timing && !m.timing.partial ? (readyMs(m.timing) ?? []) : [])));
}

/** Summons with glass are no slower than without: their warm median within 1.5× + the slack. */
export function glassTimingFailures(run: Measured[]): string[] {
  const failures: string[] = [];
  for (const [mode, slack] of Object.entries(GLASS_SLACK_MS) as [RunMode, number][]) {
    if (!run.some((m) => m.run === mode)) {
      continue;
    }
    const off = OFF_RUN[mode];
    const glass = warmMedian(run, mode);
    const without = off === undefined ? undefined : warmMedian(run, off);
    if (glass === undefined || without === undefined) {
      failures.push(`summon timing (${mode}): no warm median to compare (glass ${glass ?? "–"}, off ${without ?? "–"})`);
    } else if (glass > without * 1.5 + slack) {
      failures.push(`summon timing (${mode}): warm median ${glass} ms with glass, ${without} ms without — over 1.5× + ${slack} ms`);
    }
  }
  return failures;
}

/** What a glass run needs from smoke.mts. */
export interface GlassDriver {
  out: string;
  pageUrl: string;
  /** Launches the app on a fresh data folder with these settings and extra environment. */
  launch(name: string, settings: Record<string, unknown>, extra: Record<string, string>): Promise<SiteApp>;
  screenshot(name: string): void;
  sleep(ms: number): Promise<void>;
  /** Writes the accessibility stand-in file. */
  write(path: string, text: string): void;
}

/** One glass run (macOS 26+): the page, the command bar, then its accessibility switches. */
export async function glassRun(d: GlassDriver, run: GlassRun): Promise<LogEvent[]> {
  const a11y = `${d.out}/a11y-${run.name}.txt`;
  const app = await d.launch(run.name, { theme: run.theme, glass: run.glass }, { LITTLEJOAN_SMOKE_A11Y: a11y });
  const shot = (what: string) => d.screenshot(`glass/${run.name}-${what}.png`);
  await app.waitFor(`${run.name}: started`, (e) => count(e, "started") > 0, 30_000);
  await d.sleep(1500);
  await app.forward(["--show", d.pageUrl]);
  await app.waitFor(`${run.name}: smoke page`, (e) => lastTitle(e, "LJ-SMOKE ready") !== undefined && count(e, "overlay", "false") > 0, 30_000);
  await d.sleep(1500);
  shot("page");
  await app.forward(["--hide"]);
  await app.waitFor(`${run.name}: closed`, (e) => count(e, "hibernated") > 0, 20_000);
  const from = app.events().length;
  await app.forward(["--show"]);
  await app.waitFor(`${run.name}: command bar`, (e) => sawSmokeTopSite(e.slice(from)), 20_000);
  await d.sleep(1500);
  shot("command-bar");
  for (const step of run.accessibility) {
    const at = app.events().length;
    d.write(a11y, step.write);
    await app.waitFor(
      `${run.name}: ${step.write} → ${step.shows}`,
      (e) => e.slice(at).some((x) => x.event === "glass" && x.detail === step.shows) && e.slice(at).some((x) => x.event === "glass-view"),
      10_000,
    );
    await d.sleep(1200);
    if (step.shot) {
      shot(step.shot);
    }
  }
  const closed = count(app.events(), "hibernated");
  await app.forward(["--hide"]);
  await app.waitFor(`${run.name}: closed again`, (e) => count(e, "hibernated") > closed, 20_000);
  await app.quit();
  return app.events();
}

/**
 * macOS: `swift glass-backdrop.swift <status> <seconds>` — a borderless window of bright diagonal
 * stripes and large text over the whole screen, at the normal level, behind Little Joan's window
 * (an accessory app that never activates or takes the mouse), so the screenshots show what the glass
 * does to what is behind the window. It writes `shown` to <status> once on screen, and quits when
 * <status>.stop appears or after <seconds>.
 */
export const MAC_BACKDROP = `import AppKit

let args = CommandLine.arguments
let statusPath = args.count > 1 ? args[1] : NSTemporaryDirectory() + "lj-glass-backdrop.txt"
let seconds = Double(args.count > 2 ? args[2] : "600") ?? 600
let app = NSApplication.shared
app.setActivationPolicy(.accessory)

final class Stripes: NSView {
  override func draw(_ dirty: NSRect) {
    let colors: [NSColor] = [.systemRed, .systemOrange, .systemYellow, .systemGreen, .systemTeal, .systemBlue, .systemPurple, .systemPink]
    let width: CGFloat = 70
    var x: CGFloat = -bounds.height
    var index = 0
    while x < bounds.width {
      let path = NSBezierPath()
      path.move(to: NSPoint(x: x, y: 0))
      path.line(to: NSPoint(x: x + width, y: 0))
      path.line(to: NSPoint(x: x + width + bounds.height, y: bounds.height))
      path.line(to: NSPoint(x: x + bounds.height, y: bounds.height))
      path.close()
      colors[index % colors.count].setFill()
      path.fill()
      x += width
      index += 1
    }
    let text = "LITTLE JOAN GLASS" as NSString
    let attributes: [NSAttributedString.Key: Any] = [.font: NSFont.boldSystemFont(ofSize: 72), .foregroundColor: NSColor.white]
    var y: CGFloat = 30
    while y < bounds.height {
      text.draw(at: NSPoint(x: 40, y: y), withAttributes: attributes)
      y += 150
    }
  }
}

let frame = NSScreen.main?.frame ?? NSRect(x: 0, y: 0, width: 1280, height: 800)
let window = NSWindow(contentRect: frame, styleMask: [.borderless], backing: .buffered, defer: false)
window.contentView = Stripes(frame: NSRect(origin: .zero, size: frame.size))
window.level = .normal
window.ignoresMouseEvents = true
window.isReleasedWhenClosed = false
window.orderFrontRegardless()
DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
  FileManager.default.createFile(atPath: statusPath, contents: "shown\\n".data(using: .utf8))
}
DispatchQueue.main.asyncAfter(deadline: .now() + seconds) { app.terminate(nil) }
Timer.scheduledTimer(withTimeInterval: 0.25, repeats: true) { _ in
  if FileManager.default.fileExists(atPath: statusPath + ".stop") { app.terminate(nil) }
}
app.run()
`;
