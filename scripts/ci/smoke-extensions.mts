// The smoke test's extension run and its rules (pure, unit-tested; scripts/ci/smoke.mts drives
// it): a launch of its own on a fresh data folder runs `--extension hello` — the bundled example
// checks storage, the theme, that Tauri's IPC is refused and that its CSP blocks the network, and
// titles itself with the results — and then shows the smoke page with ?ext=probe, which tries to
// call hello's API, show one of its files, and navigate the page webview to its panel, all of
// which must fail. The app logs `extension-opened <id>`, `ext-api <id> <method> <ok | error>`,
// `ljext-refused <reason> <webview> <url>` and `navigation-refused <url>`.
import type { LogEvent } from "./smoke-log.mts";

/** What hello must report. */
export const HELLO_TITLE = "LJ-EXT hello storage=ok theme=ok ipc=denied net=blocked";
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
 * in the page), called storage and theme with success, and reported HELLO_TITLE; then, from the
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
  if (!reported.includes(HELLO_TITLE)) {
    failures.push(`extensions: hello reported ${JSON.stringify(reported.at(-1) ?? "nothing")}, not "${HELLO_TITLE}"`);
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
  const after = events.slice(probeAt);
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

/** The developer extensions the settings run puts in its folder: one that loads, one that can't. */
export const DEV_EXTENSION = "dev-hello";
export const BROKEN_EXTENSION = "broken";

/**
 * Settings → Extensions (the run with Developer mode on): `--settings extensions` opened the tab
 * after Appearance, it listed hello and the developer copy with the broken one skipped
 * (`extensions-list 2 skipped=1`), and the broken one's reason was logged.
 */
export function settingsExtensionsFailures(events: LogEvent[]): string[] {
  const failures: string[] = [];
  const at = events.findIndex((e) => e.event === "settings-window" && e.detail === "extensions");
  if (at < 0) {
    failures.push("--settings extensions never switched Settings to Extensions");
    return failures;
  }
  const listed = events.slice(at).filter((e) => e.event === "extensions-list");
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
