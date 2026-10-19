// The smoke test's keymap run (LJ43) and its rules. Its own launch whose settings.json says
// `"keys": { "history": "CommandOrControl+Shift+KeyK" }`: over the smoke page, Ctrl+Shift+K
// (SendKeys `^+k`, xdotool `ctrl+shift+k`; ⌘⇧K on macOS) must open the history view, and the old
// key — Ctrl+H (⌘Y on macOS), pressed with the view closed again — must open nothing within 2 s.
// On macOS the log's `menu-keys history=⌘⇧K` shows the menu was built from the keymap (required;
// the key presses there are best effort, as all synthetic keys on CI's macOS runners). smoke.mts
// supplies the driver; the rules are pure and unit-tested (smoke-keymap.test.ts).
import type { LogEvent } from "./smoke-log.mts";
import type { SiteApp } from "./smoke-sitemode.mts";

/** The binding the run sets, and how the menu and Settings show it. */
export const HISTORY_BINDING = "CommandOrControl+Shift+KeyK";
export const HISTORY_SHOWN_MAC = "⌘⇧K";

export interface KeymapDriver {
  os: string;
  pageUrl: string;
  launch(name: string, settings: Record<string, unknown>): Promise<SiteApp>;
  sleep(ms: number): Promise<void>;
  pressKeys(keys: string[], pid: number, activate: boolean): void;
  /** Ctrl+<letter> (Cmd on macOS), with Shift when asked. */
  pressShortcut(letter: string, pid: number, shift?: boolean): void;
}

export interface KeymapRun {
  events: LogEvent[];
  /** The new key opened the history view. */
  newKeyOpened: boolean;
  /** What the log said in the 2 s after the old key (undefined: not pressed). */
  afterOldKey?: LogEvent[];
}

/** The `menu-keys` lines: action id → what the menu shows. */
export function menuKeys(events: LogEvent[]): Record<string, string> {
  const keys: Record<string, string> = {};
  for (const event of events) {
    if (event.event === "menu-keys") {
      const at = event.detail.indexOf("=");
      if (at > 0) {
        keys[event.detail.slice(0, at)] = event.detail.slice(at + 1);
      }
    }
  }
  return keys;
}

/**
 * The rules: Windows and Linux — the new key opens the history view and the old one nothing;
 * macOS — the menu shows ⌘⇧K for History (and ⌘L still for Open Location), and when the new
 * key's press reached the app, the old key opens nothing.
 */
export function keymapFailures(run: KeymapRun, os: string): string[] {
  const failures: string[] = [];
  const mac = os === "darwin";
  if (mac) {
    const menu = menuKeys(run.events);
    if (menu["history"] !== HISTORY_SHOWN_MAC) {
      failures.push(`keymap: the menu showed History as ${menu["history"] ?? "nothing"}, not ${HISTORY_SHOWN_MAC}`);
    }
    if (menu["open-location"] !== "⌘L") {
      failures.push(`keymap: the menu showed Open Location as ${menu["open-location"] ?? "nothing"}, not ⌘L`);
    }
  } else if (!run.newKeyOpened) {
    failures.push("keymap: Ctrl+Shift+K didn't open the history view");
  }
  if ((!mac || run.newKeyOpened) && run.afterOldKey?.some((e) => e.event === "history-view")) {
    failures.push(`keymap: the old history key (${mac ? "⌘Y" : "Ctrl+H"}) still opened the history view`);
  }
  if (!mac && run.newKeyOpened && run.afterOldKey === undefined) {
    failures.push("keymap: the old history key was never pressed");
  }
  return failures;
}

/** Runs the keymap launch. */
export async function keymapRun(d: KeymapDriver): Promise<KeymapRun> {
  const app = await d.launch("keymap", { keys: { history: HISTORY_BINDING } });
  await app.waitFor("keymap: started", (e) => e.some((x) => x.event === "started"), 30_000);
  await d.sleep(1500);
  const pageFrom = app.events().length;
  await app.forward(["--show", `${d.pageUrl}?keymap=1`]);
  await app.waitFor("keymap: the smoke page", (e) => e.slice(pageFrom).some((x) => x.event === "title" && x.detail.startsWith("LJ-SMOKE")), 30_000);
  await d.sleep(1500);
  const pid = app.pid();
  const keyFrom = app.events().length;
  d.pressShortcut("k", pid, true);
  const newKeyOpened = await app.waitFor(
    "keymap: Ctrl+Shift+K (⌘⇧K) opened the history view",
    (e) => e.slice(keyFrom).some((x) => x.event === "history-view"),
    10_000,
  );
  let afterOldKey: LogEvent[] | undefined;
  if (newKeyOpened || d.os !== "darwin") {
    // Back to the command bar, then the old key: it must open nothing now.
    d.pressKeys(["Escape"], pid, false);
    await d.sleep(1000);
    const oldFrom = app.events().length;
    d.pressShortcut(d.os === "darwin" ? "y" : "h", pid);
    await d.sleep(2000);
    afterOldKey = app.events().slice(oldFrom);
  }
  await app.quit();
  return { events: app.events(), newKeyOpened, ...(afterOldKey ? { afterOldKey } : {}) };
}

/** What summary.json records of the run. */
export function keymapSummary(run: KeymapRun, os: string) {
  return {
    menuKeys: menuKeys(run.events),
    newKeyOpened: run.newKeyOpened,
    afterOldKey: run.afterOldKey?.map((e) => e.event) ?? null,
    rules: keymapFailures(run, os),
  };
}
