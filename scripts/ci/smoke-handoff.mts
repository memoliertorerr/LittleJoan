// The smoke test's handoff run (LJ46) and its rules: pages Little Joan hands on go to the main
// browser, never back to Little Joan. Its own launches:
// - Windows and Linux (on a GitHub-hosted runner only — smoke-runner.mts): a fake browser is
//   installed for the run — a `StartMenuInternet\LJFakeBrowser` entry under HKCU whose Capabilities
//   give https to the ProgId `LJFakeBrowserURL` (Little Joan launches through that ProgId's command,
//   %1 = the URL), or `lj-fake-browser.desktop` in ~/.local/share/applications — whose command is a
//   node script that writes the URL it gets to a file. The `handoff` launch has it as `mainBrowser`:
//   the smoke page's `?blank=1` (one big target=_blank link) is clicked at the screen's centre, and
//   the app must log `handoff browser=<fake>`, the fake must receive `…smoke.html?from=blank`, and no
//   `link-opened` may follow. Then the **flood** (phase 21's security review): `?flood=1` asks for
//   five new windows per click, clicked three times — every `new-window` must be handed on or
//   dropped (`handoff-throttled`), at most one handoff a second over the flood's span, and the fake
//   may receive no more than were handed on.
// - Every OS: Settings → General scrolled to its Browser section (a smoke-only scroll,
//   `LITTLEJOAN_SMOKE_SETTINGS_SCROLL=browser`) for `settings-general-browser.png`; the app must log
//   `browsers-listed <n> default=<self|other|none>`.
// - Linux: the loop check — the smoke itself makes Little Joan the runner's default for http and
//   https (`xdg-mime default`, on the disposable runner; the app never does), and the
//   `handoff-default` launch, with no main browser, clicks the same link: it must go to the first
//   other browser (the fake: ~/.local/share comes first), with `handoff-self-refused`, and no
//   `link-opened` may follow (no loop). The runner's mimeapps.list is put back after.
// Never on macOS or Windows does the smoke change the default; on macOS the handoff is covered by
// unit tests and the owner's check. The fake browsers are removed at the end.
import { count, lastTitle, type LogEvent } from "./smoke-log.mts";
import type { ExecOptions, ExecResult, LinksApp } from "./smoke-links.mts";
import { DESKTOP_ID } from "./smoke-links-read.mts";

export const FAKE_WINDOWS_ID = "LJFakeBrowser";
export const FAKE_DESKTOP_ID = "lj-fake-browser.desktop";
const FAKE_KEY = `HKEY_CURRENT_USER\\Software\\Clients\\StartMenuInternet\\${FAKE_WINDOWS_ID}`;
const FAKE_PROGID = "LJFakeBrowserURL";
const FAKE_PROGID_KEY = `HKEY_CURRENT_USER\\Software\\Classes\\${FAKE_PROGID}`;
/** The `?flood=1` page's targets, and its windows per click (tests/smoke/smoke-logic.mjs). */
export const FLOOD_TARGET = "smoke.html?from=flood";
const FLOOD_CLICKS = 3;
/** The fake browser: every URL it is started with, one JSON line each. */
export const FAKE_BROWSER_SCRIPT = `import { appendFileSync } from "node:fs";
appendFileSync(process.argv[2], JSON.stringify(process.argv.slice(3)) + "\\n");
`;
/** The `?blank=1` page's link target (tests/smoke/smoke-logic.mjs). */
export const BLANK_TARGET = "smoke.html?from=blank";

export interface HandoffDriver {
  os: string;
  out: string;
  home: string;
  pageUrl: string;
  mayChange: boolean;
  /** This node, which runs the fake browser. */
  nodePath: string;
  launch(name: string, settings: Record<string, unknown>, extra?: Record<string, string>): Promise<LinksApp>;
  exec(command: string, args: string[], options?: ExecOptions): ExecResult;
  readFile(path: string): string | undefined;
  writeFile(path: string, text: string): void;
  removeFile(path: string): void;
  mkdirs(path: string): void;
  sleep(ms: number): Promise<void>;
  screenshot(name: string): void;
  clickCentre(): void;
}

/** One click on the `?blank=1` page: what the app logged after it, and what the fake received. */
export interface Click {
  afterClick: LogEvent[];
  received: string[];
}

/** The flood: what reached the app, and what was handed on. */
export interface Flood {
  afterClicks: LogEvent[];
  received: string[];
}

export interface HandoffRun {
  /** The fake browser's id, or why there was none. */
  fake: string | null;
  skipped: string | null;
  listed: string[];
  scrolled: boolean;
  main: Click | null;
  flood: Flood | null;
  /** Linux: with Little Joan the default and no main browser. */
  loop: (Click & { listed: string[] }) | null;
}

/** The URLs the fake browser received (its JSON lines). */
export function fakeReceived(text: string | undefined): string[] {
  return (text ?? "").split("\n").flatMap((line) => {
    try {
      const args = JSON.parse(line) as unknown;
      return Array.isArray(args) ? args.filter((a): a is string => typeof a === "string") : [];
    } catch {
      return [];
    }
  });
}

/** A desktop file for the fake browser (Linux). */
export function fakeDesktopFile(node: string, script: string, record: string): string {
  return [
    "[Desktop Entry]",
    "Type=Application",
    "Name=LJ Fake Browser",
    `Exec="${node}" "${script}" "${record}" %u`,
    "MimeType=x-scheme-handler/http;x-scheme-handler/https;",
    "Categories=Network;WebBrowser;",
    "Terminal=false",
    "",
  ].join("\n");
}

/** mimeapps.list's defaults making Little Joan the browser (the loop check's own write, Linux). */
export function ourMimeapps(existing: string | undefined): string {
  const kept = (existing ?? "").split("\n").filter((line) => !/^x-scheme-handler\/https?=/.test(line.trim()) && line.trim() !== "[Default Applications]");
  return [...kept.filter((line) => line.trim() !== ""), "[Default Applications]", `x-scheme-handler/http=${DESKTOP_ID}`, `x-scheme-handler/https=${DESKTOP_ID}`, ""].join("\n");
}

async function clickBlank(d: HandoffDriver, app: LinksApp, record: string): Promise<Click> {
  await app.forward(["--show", `${d.pageUrl}?blank=1`]);
  await app.waitFor("handoff: the new-window page is up", (e) => lastTitle(e, "LJ-BLANK ready") !== undefined, 30_000);
  await d.sleep(1500);
  const from = app.events().length;
  const before = fakeReceived(d.readFile(record)).length;
  d.clickCentre();
  await app.waitFor("handoff: the page was handed on", (e) => count(e.slice(from), "handoff") + count(e.slice(from), "handoff-none") > 0, 20_000);
  for (let waited = 0; waited < 15_000 && fakeReceived(d.readFile(record)).length <= before; waited += 500) {
    await d.sleep(500);
  }
  // Long enough for a page bounced back to the app to show up (it mustn't).
  await d.sleep(4000);
  return { afterClick: app.events().slice(from), received: fakeReceived(d.readFile(record)).slice(before) };
}

async function flood(d: HandoffDriver, app: LinksApp, record: string): Promise<Flood> {
  await app.forward(["--show", `${d.pageUrl}?flood=1`]);
  await app.waitFor("handoff: the flood page is up", (e) => lastTitle(e, "LJ-FLOOD ready") !== undefined, 30_000);
  await d.sleep(2500);
  const from = app.events().length;
  const before = fakeReceived(d.readFile(record)).length;
  for (let n = 0; n < FLOOD_CLICKS; n += 1) {
    d.clickCentre();
  }
  // Anything still under way lands, and nothing more may come.
  await d.sleep(6000);
  return { afterClicks: app.events().slice(from), received: fakeReceived(d.readFile(record)).slice(before) };
}

const listedLines = (events: LogEvent[]) => events.filter((e) => e.event === "browsers-listed").map((e) => e.detail);

export async function handoffRun(d: HandoffDriver): Promise<HandoffRun> {
  const withFake = d.os !== "darwin" && d.mayChange;
  const fake = withFake ? (d.os === "win32" ? FAKE_WINDOWS_ID : FAKE_DESKTOP_ID) : null;
  const skipped = d.os === "darwin" ? "macOS: the unit tests and the owner's check" : d.mayChange ? null : "not on a GitHub runner";
  const script = `${d.out}/fake-browser.mjs`;
  const record = `${d.out}/fake-browser.jsonl`;
  const desktop = `${d.home}/.local/share/applications/${FAKE_DESKTOP_ID}`;
  const mimeapps = `${d.home}/.config/mimeapps.list`;
  let run: HandoffRun = { fake, skipped, listed: [], scrolled: false, main: null, flood: null, loop: null };
  if (fake) {
    d.writeFile(script, FAKE_BROWSER_SCRIPT);
    d.writeFile(record, "");
    if (d.os === "win32") {
      d.exec("reg", ["add", FAKE_KEY, "/ve", "/d", "LJ Fake Browser", "/f"]);
      d.exec("reg", ["add", `${FAKE_KEY}\\shell\\open\\command`, "/ve", "/d", `"${d.nodePath}"`, "/f"]);
      d.exec("reg", ["add", `${FAKE_KEY}\\Capabilities\\URLAssociations`, "/v", "https", "/d", FAKE_PROGID, "/f"]);
      d.exec("reg", ["add", `${FAKE_PROGID_KEY}\\shell\\open\\command`, "/ve", "/d", `"${d.nodePath}" "${script}" "${record}" "%1"`, "/f"]);
    } else {
      d.mkdirs(`${d.home}/.local/share/applications`);
      d.writeFile(desktop, fakeDesktopFile(d.nodePath, script, record));
    }
  }
  const savedMimeapps = d.readFile(mimeapps);
  try {
    const app = await d.launch("handoff", fake ? { mainBrowser: fake } : {}, { LITTLEJOAN_SMOKE_SETTINGS_SCROLL: "browser" });
    await app.waitFor("handoff: started and prewarmed", (e) => count(e, "prewarmed") > 0, 30_000);
    const main = fake ? await clickBlank(d, app, record) : null;
    const flooded = fake ? await flood(d, app, record) : null;
    await app.forward(["--settings", "general"]);
    const scrolled = await app.waitFor("handoff: Settings at its Browser section", (e) => count(e, "settings-scrolled", "browser") > 0 && count(e, "browsers-listed") > 0, 30_000);
    await d.sleep(1500);
    d.screenshot("settings-general-browser.png");
    run = { ...run, main, flood: flooded, scrolled, listed: listedLines(app.events()) };
    await app.quit();
    if (fake && d.os === "linux") {
      // The loop check: Little Joan the runner's default — the smoke's own write, never the app's.
      const set = d.exec("xdg-mime", ["default", DESKTOP_ID, "x-scheme-handler/http", "x-scheme-handler/https"]);
      if (set.code !== 0) {
        d.mkdirs(`${d.home}/.config`);
        d.writeFile(mimeapps, ourMimeapps(savedMimeapps));
      }
      const loopApp = await d.launch("handoff-default", {});
      await loopApp.waitFor("handoff: started (Little Joan the default)", (e) => count(e, "prewarmed") > 0, 30_000);
      const click = await clickBlank(d, loopApp, record);
      run = { ...run, loop: { ...click, listed: listedLines(click.afterClick) } };
      await loopApp.quit();
    }
  } finally {
    if (fake && d.os === "linux") {
      if (savedMimeapps === undefined) {
        d.removeFile(mimeapps);
      } else {
        d.writeFile(mimeapps, savedMimeapps);
      }
      d.removeFile(desktop);
    }
    if (fake && d.os === "win32") {
      d.exec("reg", ["delete", FAKE_KEY, "/f"]);
      d.exec("reg", ["delete", FAKE_PROGID_KEY, "/f"]);
    }
  }
  return run;
}

/** A click handed on to the fake browser, which got the link's page, and nothing came back. */
function clickFailures(click: Click | null, fake: string, what: string): string[] {
  if (!click) {
    return [`handoff: ${what}: no click was made`];
  }
  const failures: string[] = [];
  const handoffs = click.afterClick.filter((e) => e.event === "handoff").map((e) => e.detail);
  if (!handoffs.includes(`browser=${fake}`)) {
    failures.push(`handoff: ${what}: the app logged [${handoffs.join(", ") || (count(click.afterClick, "handoff-none") > 0 ? "handoff-none" : "nothing")}], not handoff browser=${fake}`);
  }
  if (!click.received.some((url) => url.endsWith(BLANK_TARGET))) {
    failures.push(`handoff: ${what}: the fake browser received [${click.received.join(", ")}], not the link's ${BLANK_TARGET}`);
  }
  if (count(click.afterClick, "link-opened") > 0) {
    failures.push(`handoff: ${what}: the page came back to Little Joan (link-opened): a loop`);
  }
  return failures;
}

/** The flood's rule: every new window handed on or dropped, at most one a second, no more received. */
export function floodFailures(flood: Flood | null): string[] {
  if (!flood) {
    return ["handoff: the flood never ran"];
  }
  const failures: string[] = [];
  const windows = flood.afterClicks.filter((e) => e.event === "new-window");
  const handed = count(flood.afterClicks, "handoff");
  const throttled = count(flood.afterClicks, "handoff-throttled");
  if (windows.length === 0) {
    failures.push("handoff: the flood's clicks asked for no new window at all");
    return failures;
  }
  if (handed + throttled !== windows.length) {
    failures.push(`handoff: of ${windows.length} new windows ${handed} were handed on and ${throttled} dropped — every one must be one or the other`);
  }
  const span = (windows.at(-1)?.millis ?? 0) - (windows[0]?.millis ?? 0);
  const allowed = Math.floor(span / 1000) + 1;
  if (handed > allowed) {
    failures.push(`handoff: ${handed} handoffs over a ${span} ms flood (at most ${allowed}: one a second)`);
  }
  const floods = flood.received.filter((url) => url.includes(FLOOD_TARGET));
  if (floods.length > handed) {
    failures.push(`handoff: the fake browser received ${floods.length} of the flood's pages, more than the ${handed} handed on`);
  }
  return failures;
}

export function handoffFailures(run: HandoffRun, os: string): string[] {
  const failures: string[] = [];
  if (!run.listed.some((line) => /^\d+ default=(self|other|none)$/.test(line))) {
    failures.push(`handoff: no browsers-listed <n> default=<self|other|none> line (got [${run.listed.join(", ")}])`);
  }
  if (!run.scrolled) {
    failures.push("handoff: Settings never scrolled to its Browser section (settings-scrolled browser)");
  }
  if (run.fake) {
    failures.push(...clickFailures(run.main, run.fake, "with the fake as the main browser"));
    failures.push(...floodFailures(run.flood));
    if (os === "linux") {
      failures.push(...clickFailures(run.loop, run.fake, "with Little Joan the default and no main browser"));
      if (run.loop && count(run.loop.afterClick, "handoff-self-refused") === 0) {
        failures.push("handoff: with Little Joan the default, the handoff didn't say it passed over itself (handoff-self-refused)");
      }
      if (run.loop && !run.loop.listed.some((line) => line.endsWith("default=self"))) {
        failures.push(`handoff: with Little Joan the default, the app read [${run.loop.listed.join(", ")}], not default=self`);
      }
    }
  }
  return failures;
}

export function handoffSummary(run: HandoffRun, os: string): Record<string, unknown> {
  const lines = (events: LogEvent[] | undefined) => (events ?? []).filter((e) => e.event.startsWith("handoff") || e.event.startsWith("link-")).map((e) => `${e.event} ${e.detail}`.trim());
  return {
    fake: run.fake,
    skipped: run.skipped,
    listed: run.listed,
    main: run.main ? { log: lines(run.main.afterClick), received: run.main.received } : null,
    flood: run.flood
      ? {
          newWindows: count(run.flood.afterClicks, "new-window"),
          handedOn: count(run.flood.afterClicks, "handoff"),
          throttled: count(run.flood.afterClicks, "handoff-throttled"),
          received: run.flood.received.length,
        }
      : null,
    loop: run.loop ? { log: lines(run.loop.afterClick), received: run.loop.received, listed: run.loop.listed } : null,
    rules: handoffFailures(run, os),
  };
}
