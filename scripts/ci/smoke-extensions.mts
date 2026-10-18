// The smoke test's extension run and its rules (pure, unit-tested; scripts/ci/smoke.mts drives
// it): a launch of its own on a fresh data folder runs `--extension hello` — the bundled example
// checks storage, the theme, that Tauri's IPC is refused and that its CSP blocks the network, and
// titles itself with the results — and then shows the smoke page with ?ext=probe, which tries to
// call hello's API, show one of its files, and navigate the page webview to its panel, all of
// which must fail. The app logs `extension-opened <id>`, `ext-api <id> <method> <ok | error>`,
// `ljext-refused <reason> <webview> <url>` and `navigation-refused <url>`.
import type { LogEvent } from "./smoke-log.mts";

/** What hello reports when all is well (`sw=unsupported` is fine too: see `isHelloReport`). */
export const HELLO_TITLE = "LJ-EXT hello storage=ok theme=ok ipc=denied net=blocked sw=refused";

/**
 * Whether `title` is hello's good report: storage and the theme work, Tauri's IPC is refused, the
 * CSP blocks the network, and a service worker couldn't be registered — Little Joan refused its
 * script, or the engine has none for extensions.
 */
export function isHelloReport(title: string): boolean {
  return /^LJ-EXT hello storage=ok theme=ok ipc=denied net=blocked sw=(refused|unsupported)$/.test(title);
}
/** What the smoke page's probe must report. */
export const PROBE_TITLE = "LJ-EXTPROBE ljext-api=blocked ljext-file=blocked ljext-nav=refused";

/** hello's origin as the platform's engine names it (WebView2 serves custom schemes over http). */
export function helloOrigin(os: NodeJS.Platform): string {
  return os === "win32" ? "http://ljext.hello.localhost" : "ljext://hello.localhost";
}

/** An extension URL in either form (`ljext:`, or WebView2's `http(s)://ljext.*`). */
export function isExtUrl(url: string): boolean {
  return /^(ljext:|https?:\/\/ljext\.)/i.test(url);
}

/** What the run left in hello's storage file (extension-data/hello.json), if it was there. */
export interface HelloStore {
  opened: unknown;
  keys: string[];
}

/** Reads extension-data/hello.json; undefined when it is missing or not a store. */
export function parseStore(text: string | undefined): HelloStore | undefined {
  if (text === undefined) {
    return undefined;
  }
  try {
    const value = JSON.parse(text) as { items?: unknown };
    if (typeof value.items !== "object" || value.items === null) {
      return undefined;
    }
    const items = value.items as Record<string, unknown>;
    return { opened: items.opened, keys: Object.keys(items) };
  } catch {
    return undefined;
  }
}

function titles(events: LogEvent[], prefix: string): string[] {
  return events.filter((e) => e.event === "title" && e.detail.startsWith(prefix)).map((e) => e.detail);
}

/**
 * The extension run's rules: every bundled extension loaded; hello opened (its panel's URL loaded
 * in the page), called storage and theme with success, and reported HELLO_TITLE — the first open
 * (`--extension hello`, from hibernation) on its own, before anything else opened it again, with
 * none of hello's own requests refused meanwhile; then, from the
 * probe page on, no API call succeeded, no extension URL loaded, the guard refused the navigation
 * to the panel, and the probe reported PROBE_TITLE. hello's store holds its count and nothing the
 * probe tried to write.
 */
export function extensionFailures(events: LogEvent[], store: HelloStore | undefined): string[] {
  const failures: string[] = [];
  for (const skipped of events.filter((e) => e.event === "extension-skipped")) {
    failures.push(`extensions: a bundled extension was skipped: ${skipped.detail}`);
  }
  if (!events.some((e) => e.event === "extension-opened" && e.detail === "hello")) {
    failures.push("extensions: --extension hello never opened it");
  }
  if (!events.some((e) => e.event === "page-loaded" && isExtUrl(e.detail) && e.detail.includes("hello"))) {
    failures.push("extensions: hello's panel never loaded in the page");
  }
  const reported = titles(events, "LJ-EXT hello");
  if (!reported.some(isHelloReport)) {
    failures.push(`extensions: hello reported ${JSON.stringify(reported.at(-1) ?? "nothing")}, not "${HELLO_TITLE.replace("sw=refused", "sw=refused|unsupported")}"`);
  }
  failures.push(...serviceWorkerFailures(events));
  const firstOpen = events.findIndex((e) => e.event === "extension-opened" && e.detail === "hello");
  if (firstOpen >= 0) {
    const next = events.findIndex(
      (e, i) => i > firstOpen && ((e.event === "extension-opened") || (e.event === "page-loaded" && e.detail.includes("ext=probe"))),
    );
    const firstPanel = events.slice(firstOpen, next < 0 ? undefined : next);
    if (!firstPanel.some((e) => e.event === "title" && isHelloReport(e.detail))) {
      failures.push("extensions: --extension hello's own panel never reported its checks (before anything opened it again)");
    }
    // The engine's favicon request and hello's service worker are refused on purpose.
    const expected = (detail: string) => detail.includes("favicon.ico") || /\/sw\.js$/.test(detail);
    for (const refused of firstPanel.filter((e) => e.event === "ljext-refused" && /\/\/hello\.localhost\//.test(e.detail) && !expected(e.detail))) {
      failures.push(`extensions: a request of hello's own panel was refused: ${refused.detail}`);
    }
  }
  for (const method of ["storage.set", "storage.get", "theme"]) {
    if (!events.some((e) => e.event === "ext-api" && e.detail === `hello ${method} ok`)) {
      failures.push(`extensions: hello's ${method} call never succeeded`);
    }
  }
  const probeAt = events.findIndex((e) => e.event === "page-loaded" && e.detail.includes("ext=probe"));
  if (probeAt < 0) {
    failures.push("extensions: the smoke page's probe never loaded");
    return failures;
  }
  // The probe's stretch: until the race (below) starts, where the app opens hello again.
  const raceAt = events.findIndex((e, i) => i > probeAt && e.event === "page-loaded" && e.detail.includes("ext=race"));
  const after = events.slice(probeAt, raceAt < 0 ? undefined : raceAt);
  for (const call of after.filter((e) => e.event === "ext-api" && e.detail.endsWith(" ok"))) {
    failures.push(`extensions: an API call succeeded after the probe page loaded: ${call.detail}`);
  }
  for (const load of after.filter((e) => e.event === "page-loaded" && isExtUrl(e.detail))) {
    failures.push(`extensions: the probe page took the page webview to an extension: ${load.detail}`);
  }
  if (!after.some((e) => e.event === "navigation-refused" && isExtUrl(e.detail))) {
    failures.push("extensions: the guard never refused the probe's navigation to hello's panel");
  }
  const probe = titles(after, "LJ-EXTPROBE ljext-api");
  if (!probe.includes(PROBE_TITLE)) {
    failures.push(`extensions: the probe reported ${JSON.stringify(probe.at(-1) ?? "nothing")}, not "${PROBE_TITLE}"`);
  }
  if (!store) {
    failures.push("extensions: hello's storage file is missing");
  } else {
    if (typeof store.opened !== "number" || store.opened < 1) {
      failures.push(`extensions: hello's storage has no count (${JSON.stringify(store.opened)})`);
    }
    if (store.keys.includes("smoke-probe")) {
      failures.push("extensions: the probe page wrote into hello's storage");
    }
  }
  return failures;
}

/** Whether `url` is the app's own start page (`tauri://localhost/…`, `http://tauri.localhost/…`). */
function isStartPage(url: string): boolean {
  return /^(tauri:\/\/localhost|https?:\/\/tauri\.localhost)\//.test(url);
}

/** Whether `url` is hello's panel as the app opens it: with its single-use nonce. */
function isOpenedPanel(url: string): boolean {
  return isExtUrl(url) && url.includes("hello") && /[?&]lj-open=[0-9a-f]{32}(?:$|&)/.test(url);
}

/**
 * The race (the security review's case): while the smoke page keeps putting hello's panel URL —
 * without the app's nonce — into iframes and window.open, `--extension hello` must still open the
 * panel (it reports again), every extension URL that loads in the page must carry the nonce, at
 * least one of the page's attempts must have been refused (the guard on macOS / Linux, the
 * handler on Windows, where the guard doesn't see frames), the app must leave the web page for
 * its start page before the panel loads (so the page can't race it), nothing may be served to a request
 * whose Referer is another page (`servedToOthers` — CI saw the racers' frames served on Windows
 * in the moment before the panel committed), and the toolbar may name hello only right after
 * hello's own panel loaded — never while a web page is shown.
 */
export function raceFailures(events: LogEvent[]): string[] {
  const failures: string[] = [];
  const raceAt = events.findIndex((e) => e.event === "page-loaded" && e.detail.includes("ext=race"));
  if (raceAt < 0) {
    return ["race: the smoke page's race never loaded"];
  }
  const after = events.slice(raceAt);
  if (!after.some((e) => e.event === "title" && isHelloReport(e.detail))) {
    failures.push("race: --extension hello didn't open the panel while the page raced it");
  }
  for (const load of events.filter((e) => (e.event === "page-loaded" || e.event === "page-started") && isExtUrl(e.detail))) {
    if (!isOpenedPanel(load.detail)) {
      failures.push(`race: an extension URL without the app's nonce loaded in the page: ${load.detail}`);
    }
  }
  const openedAt = after.findIndex((e) => e.event === "extension-opened");
  const nextPages = after.slice(openedAt + 1).filter((e) => e.event === "page-started").map((e) => e.detail);
  if (openedAt >= 0 && (!isStartPage(nextPages[0] ?? "") || !isOpenedPanel(nextPages[1] ?? ""))) {
    failures.push(`race: the app didn't leave the web page for its start page before the panel (then: ${nextPages.slice(0, 2).join(", ") || "nothing"})`);
  }
  const refusedRacer = after.some(
    (e) =>
      (e.event === "navigation-refused" || e.event === "ljext-refused") &&
      e.detail.includes("hello.localhost/index.html") &&
      !e.detail.includes("lj-open="),
  );
  if (!refusedRacer) {
    failures.push("race: none of the page's iframe attempts reached the guard or the handler to be refused");
  }
  failures.push(...servedToOthers(events));
  failures.push(...labelFailures(events));
  return failures;
}

/**
 * hello's service worker script (`/sw.js`) must never be served — whatever the engine reports, a
 * 200 for it would mean a worker could have been installed (the request log has every answer).
 */
export function serviceWorkerFailures(events: LogEvent[]): string[] {
  return events
    .filter((e) => e.event === "ljext-headers" && /\/sw\.js status=200\b/.test(e.detail))
    .map((e) => `extensions: a service worker's script was served: ${e.detail.split(" names=")[0]}`);
}

/**
 * Requests the handler answered 200 although their Referer was another page (`ljext-headers …
 * status=200 referer=other`, logged for every `ljext` request).
 */
export function servedToOthers(events: LogEvent[]): string[] {
  return events
    .filter((e) => e.event === "ljext-headers" && /\bstatus=200\b/.test(e.detail) && /\breferer=other\b/.test(e.detail))
    .map((e) => `served to another page: ${e.detail.split(" names=")[0]}`);
}

/** The toolbar names an extension only right after that extension's panel (with its nonce) loaded. */
export function labelFailures(events: LogEvent[]): string[] {
  const failures: string[] = [];
  let lastPage = "";
  for (const e of events) {
    if (e.event === "page-started" || e.event === "page-loaded") {
      lastPage = e.detail;
    } else if (e.event === "toolbar-label" && e.detail !== "none") {
      const itsOwn = lastPage.includes(`//${e.detail}.localhost/`) || lastPage.includes(`//ljext.${e.detail}.localhost/`);
      if (!isOpenedPanel(lastPage) || !itsOwn) {
        failures.push(`the toolbar named ${e.detail} while the page was ${JSON.stringify(lastPage || "nothing")}`);
      }
    }
  }
  if (!events.some((e) => e.event === "toolbar-label" && e.detail === "hello")) {
    failures.push("the toolbar never named hello on its panel");
  }
  return failures;
}

/** The developer extensions the settings run puts in its folder: one that loads, one that can't. */
export const DEV_EXTENSION = "dev-hello";
export const BROKEN_EXTENSION = "broken";

/**
 * Settings → Extensions (the run with Developer mode on): `--settings extensions` switched Settings
 * to the tab, the settings window listed hello and the developer copy with the broken one skipped
 * (`extensions-list 2 skipped=1` — the tab asks once, when the window loads, whichever tab it
 * opens on), and the broken one's reason was logged.
 */
export function settingsExtensionsFailures(events: LogEvent[]): string[] {
  const failures: string[] = [];
  if (!events.some((e) => e.event === "settings-window" && e.detail === "extensions")) {
    failures.push("--settings extensions never switched Settings to Extensions");
    return failures;
  }
  const opened = events.findIndex((e) => e.event === "settings-window");
  const listed = events.slice(opened).filter((e) => e.event === "extensions-list");
  if (!listed.some((e) => e.detail === "2 skipped=1")) {
    failures.push(
      `Settings → Extensions didn't list hello and ${DEV_EXTENSION} with ${BROKEN_EXTENSION} skipped (${JSON.stringify(listed.map((e) => e.detail))})`,
    );
  }
  if (!events.some((e) => e.event === "extension-skipped" && e.detail.startsWith(`${BROKEN_EXTENSION} `))) {
    failures.push(`the broken developer extension wasn't reported as skipped`);
  }
  return failures;
}

/**
 * The command bar's extension rows (the extension run): typing "hello" and picking "Say hello"
 * (↓ ↓ Enter) opened hello a second time. Required where keys can be driven (Windows, Linux),
 * best effort on macOS; the bar asked for the extensions either way.
 */
export function commandHelloFailures(events: LogEvent[], options: { driven: boolean }): string[] {
  const failures: string[] = [];
  if (!events.some((e) => e.event === "extensions-list")) {
    failures.push("the command bar never asked for the extensions");
  }
  const opened = events.filter((e) => e.event === "extension-opened" && e.detail === "hello").length;
  if (options.driven && opened < 2) {
    failures.push(`picking "Say hello" in the command bar didn't open hello (opened ${opened} time(s) in all)`);
  }
  return failures;
}
