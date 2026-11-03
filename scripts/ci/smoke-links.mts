// The smoke test's links run (LJ45) and its rules: Little Joan as a browser the OS can hand links
// to. Its own launch, then:
// - the registration read back as installed — Windows: `reg query` of the per-user keys hooks.nsh
//   writes (StartMenuInternet\LittleJoan, RegisteredApplications, the LittleJoanURL ProgId whose
//   command is `"<app>" --link "%1"`); macOS: the app's Info.plist (http and https as a Viewer, no
//   document types) and Launch Services' handlers for https naming io.github.memoliertorerr.littlejoan
//   (if a fresh runner hasn't indexed the copied app yet, `lsregister -f` registers it as Finder does
//   on install — never a default); Linux: the installed desktop entry with `Exec=little-joan --link
//   %u`, the web MIME types and Categories, accepted by desktop-file-validate, and LJ17's portal
//   entry still hidden — the AppImage's own entry (extracted) must stay plain instead: its AppRun
//   prepends the entry's Exec arguments to every launch, and an AppImage installs nothing anyway;
// - the OS's default untouched — no UserChoice names LittleJoanURL; Launch Services' default for
//   http / https isn't ours; no mimeapps.list / defaults.list names our desktop file (`xdg-mime query
//   default` is recorded too: with no default anywhere it falls back to mimeinfo.cache, which is no
//   default written);
// - a link launched the way the OS would — the ProgId's command line with %1 (Windows, verbatim, as
//   Explorer passes it to CreateProcess), `open -a <app> <url>` (macOS), the desktop entry's Exec
//   with %u (Linux; the AppImage as `<image> --link <url>`) — must log `link-opened` and load the
//   smoke page (`LJ-SMOKE ready`);
// - `--link --quit`, `--link file:///etc/passwd` and `--link javascript:alert(1)` must each log
//   `link-refused` (invalid, file, javascript) and nothing else: no quit, no page, no window change,
//   the app still running. macOS also gets a file URL and a batch of two links through `open -a`
//   (informational: whether AppKit delivered them; nothing may open from the file).
// smoke.mts supplies the driver; the readers are smoke-links-read.mts; the rules are pure and
// unit-tested (smoke-links.test.ts). The Windows uninstaller's removal is ci.yml's last Windows step.
import { count, lastTitle, type LogEvent } from "./smoke-log.mts";
import type { SiteApp } from "./smoke-sitemode.mts";
import {
  CLIENT_KEY,
  appImageEntryFailures,
  DESKTOP_ID,
  MAC_HANDLERS,
  PORTAL_DESKTOP_ID,
  PROGID,
  PROGID_KEY,
  REGISTERED_KEY,
  USER_CHOICE,
  defaultsNamingUs,
  desktopEntryFailures,
  expandExec,
  isOurBundle,
  listOf,
  macUrlTypesFailures,
  parseDesktopEntry,
  parseMacHandlers,
  parseRegTree,
  registeredLaunch,
  windowsRegistrationFailures,
} from "./smoke-links-read.mts";

export interface ExecResult {
  /** The exit code; null when it couldn't start, or ran out of time. */
  code: number | null;
  out: string;
  err: string;
}

export interface ExecOptions {
  cwd?: string;
  env?: Record<string, string | undefined>;
  /** Windows: the command line is `argv0` + " " + the args, as they are (CreateProcess's). */
  verbatim?: { argv0: string };
}

export interface LinksApp extends SiteApp {
  /** The launch's environment (a link launched as the OS would goes to this instance). */
  env(): Record<string, string | undefined>;
}

/** What the run needs from smoke.mts. */
export interface LinksDriver {
  os: string;
  /** The installed binary, and whether it is the AppImage. */
  app: string;
  appImage: boolean;
  /** The repository (scripts/ci/apt-install.sh) and the smoke's output folder. */
  root: string;
  out: string;
  home: string;
  pageUrl: string;
  /** Whether the run may change this machine (a GitHub Actions runner only: smoke-runner.mts). */
  mayChange: boolean;
  launch(name: string, settings: Record<string, unknown>): Promise<LinksApp>;
  exec(command: string, args: string[], options?: ExecOptions): ExecResult;
  readFile(path: string): string | undefined;
  writeFile(path: string, text: string): void;
  listDir(path: string): string[];
  tempDir(name: string): string;
  sleep(ms: number): Promise<void>;
  alive(pid: number): boolean;
}

/** The smoke page as the OS's launch opens it. */
export const LINK_QUERY = "?link=os";
/** What `--link` must refuse, and the first word of each `link-refused`. */
export const REFUSED_LINKS: readonly (readonly [string, string])[] = [
  ["--quit", "invalid"],
  ["file:///etc/passwd", "file"],
  ["javascript:alert(1)", "javascript"],
];
const LSREGISTER = "/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister";

/** What the registration and the defaults looked like, read back. */
export interface Registration {
  /** What's wrong with the registration. */
  failures: string[];
  /** Any OS default naming Little Joan. */
  defaults: string[];
  /** What was read, for the summary. */
  facts: string[];
  /** How the OS would start Little Joan for a URL ({URL} in place of it), when known. */
  launch?: (url: string) => { command: string; args: string[]; options?: ExecOptions } | undefined;
}

export interface LinksRun {
  registration: Registration;
  /** How the link was launched, or why it couldn't be. */
  launched: string;
  afterLaunch: LogEvent[];
  afterRefusals: LogEvent[];
  aliveAfterRefusals: boolean;
  /** macOS: a file URL and two links through `open -a` (informational). */
  macExtras: LogEvent[];
  events: LogEvent[];
}

function readWindows(d: LinksDriver): Registration {
  const query = (key: string, ...more: string[]) => parseRegTree(d.exec("reg", ["query", key, ...more]).out);
  const tree = { ...query(CLIENT_KEY, "/s"), ...query(REGISTERED_KEY, "/v", "Little Joan"), ...query(PROGID_KEY, "/s") };
  const defaults: string[] = [];
  const facts: string[] = [];
  for (const scheme of ["http", "https"]) {
    const choice = query(USER_CHOICE(scheme), "/v", "ProgId")[USER_CHOICE(scheme)]?.ProgId ?? null;
    facts.push(`UserChoice ${scheme} = ${choice ?? "none"}`);
    if (choice === PROGID) {
      defaults.push(`links: Windows' UserChoice for ${scheme} names ${PROGID}`);
    }
    const own = d.exec("reg", ["query", `HKEY_CURRENT_USER\\Software\\Classes\\${scheme}`, "/s"]).out;
    if (/little-joan/i.test(own)) {
      defaults.push(`links: HKCU\\Software\\Classes\\${scheme} names Little Joan (only the ProgId may)`);
    }
  }
  const command = tree[`${PROGID_KEY}\\shell\\open\\command`]?.["(Default)"];
  return {
    failures: windowsRegistrationFailures(tree, d.app),
    defaults,
    facts,
    launch: (url) => {
      const line = command === undefined ? undefined : registeredLaunch(command, url);
      return line ? { command: line.program, args: [line.rest], options: { verbatim: { argv0: line.argv0 } } } : undefined;
    },
  };
}

function readMac(d: LinksDriver): Registration {
  const bundle = d.app.replace(/\/Contents\/MacOS\/[^/]+$/, "");
  const plist = `${bundle}/Contents/Info.plist`;
  const types = d.exec("plutil", ["-extract", "CFBundleURLTypes", "json", "-o", "-", plist]);
  const docs = d.exec("plutil", ["-extract", "CFBundleDocumentTypes", "json", "-o", "-", plist]);
  const failures = macUrlTypesFailures(types.out, docs.code === 0);
  const facts: string[] = [];
  const script = `${d.out}/links-handlers.swift`;
  d.writeFile(script, MAC_HANDLERS);
  let handlers = parseMacHandlers(d.exec("swift", [script]).out);
  if (!handlers?.https.all.some(isOurBundle) && d.mayChange) {
    // A copied app is indexed when Finder sees it; a fresh runner may not have yet.
    d.exec(LSREGISTER, ["-f", bundle]);
    facts.push("registered with lsregister -f (as Finder does when an app is installed)");
    handlers = parseMacHandlers(d.exec("swift", [script]).out);
  }
  if (!handlers) {
    failures.push("links: Launch Services' handlers couldn't be read");
  } else if (!handlers.https.all.some(isOurBundle) || !handlers.http.all.some(isOurBundle)) {
    failures.push(`links: Launch Services doesn't list Little Joan for http and https (https: ${handlers.https.all.join(", ") || "none"})`);
  }
  const defaults: string[] = [];
  for (const scheme of ["http", "https"] as const) {
    facts.push(`default ${scheme} = ${handlers?.[scheme].default ?? "none"}`);
    if (isOurBundle(handlers?.[scheme].default)) {
      defaults.push(`links: Launch Services' default for ${scheme} is Little Joan`);
    }
  }
  facts.push(`NSWorkspace apps for https: ${handlers?.workspace.join(", ") || "none"}`);
  return { failures, defaults, facts, launch: (url) => ({ command: "open", args: ["-a", bundle, url] }) };
}

/** The desktop entry the deb installed, or the AppImage's own (extracted); and where it was. */
function linuxEntry(d: LinksDriver): { path: string; text: string } | undefined {
  if (!d.appImage) {
    const path = `/usr/share/applications/${DESKTOP_ID}`;
    const text = d.readFile(path);
    return text === undefined ? undefined : { path, text };
  }
  const dir = d.tempDir("links-appimage");
  d.exec(d.app, ["--appimage-extract", "usr/share/applications/*"], { cwd: dir });
  const apps = `${dir}/squashfs-root/usr/share/applications`;
  for (const name of d.listDir(apps)) {
    const text = d.readFile(`${apps}/${name}`);
    const entry = parseDesktopEntry(text ?? "");
    if (text !== undefined && entry.Name === "Little Joan" && entry.NoDisplay !== "true") {
      return { path: `${apps}/${name}`, text };
    }
  }
  return undefined;
}

function readLinux(d: LinksDriver): Registration {
  const failures: string[] = [];
  const facts: string[] = [];
  const found = linuxEntry(d);
  const entry = parseDesktopEntry(found?.text ?? "");
  if (!found) {
    failures.push(`links: no desktop entry for Little Joan (${d.appImage ? "in the AppImage" : `/usr/share/applications/${DESKTOP_ID}`})`);
  } else {
    failures.push(...(d.appImage ? appImageEntryFailures(entry, found.path) : desktopEntryFailures(entry, found.path)));
    let valid = d.exec("desktop-file-validate", [found.path]);
    if (valid.code === null && d.mayChange) {
      d.exec("bash", [`${d.root}/scripts/ci/apt-install.sh`, "desktop-file-utils"]);
      facts.push("installed desktop-file-utils for desktop-file-validate");
      valid = d.exec("desktop-file-validate", [found.path]);
    }
    facts.push(`desktop-file-validate: ${valid.code === 0 ? "ok" : `code ${valid.code}`} ${(valid.out + valid.err).trim()}`.trim());
    if (valid.code !== 0) {
      failures.push(`links: desktop-file-validate refused ${found.path}: ${(valid.out + valid.err).trim() || `code ${valid.code}`}`);
    }
  }
  if (!d.appImage) {
    const portal = parseDesktopEntry(d.readFile(`/usr/share/applications/${PORTAL_DESKTOP_ID}`) ?? "");
    if (portal.NoDisplay !== "true" || listOf(portal.MimeType).some((type) => type.startsWith("x-scheme-handler/http"))) {
      failures.push(`links: LJ17's ${PORTAL_DESKTOP_ID} must stay hidden and no browser`);
    }
  }
  const lists = [
    `${d.home}/.config/mimeapps.list`,
    "/etc/xdg/mimeapps.list",
    `${d.home}/.local/share/applications/mimeapps.list`,
    `${d.home}/.local/share/applications/defaults.list`,
    "/usr/local/share/applications/mimeapps.list",
    "/usr/local/share/applications/defaults.list",
    "/usr/share/applications/mimeapps.list",
    "/usr/share/applications/defaults.list",
  ].flatMap((path) => {
    const text = d.readFile(path);
    return text === undefined ? [] : [{ path, text }];
  });
  const defaults = defaultsNamingUs(lists).map((where) => `links: ${where} makes Little Joan the default`);
  const answer = d.exec("xdg-mime", ["query", "default", "x-scheme-handler/https"]);
  facts.push(`xdg-mime query default x-scheme-handler/https: ${answer.code === null ? "no xdg-mime" : answer.out.trim() || "none"}`);
  if (answer.out.trim() === DESKTOP_ID && defaults.length === 0) {
    facts.push("(xdg-mime fell back to mimeinfo.cache: no default is written anywhere)");
  }
  return {
    failures,
    defaults,
    facts,
    launch: (url) => {
      // An AppImage installs nothing for the OS to start: a link reaches it as `<image> --link <url>`.
      if (d.appImage) {
        return { command: d.app, args: ["--link", url] };
      }
      const args = expandExec(entry.Exec ?? "", url);
      return args?.[0] ? { command: args[0], args: args.slice(1) } : undefined;
    },
  };
}

/** The events since index `from` in `events`. */
const since = (events: LogEvent[], from: number) => events.slice(from);

export async function linksRun(d: LinksDriver): Promise<LinksRun> {
  const app = await d.launch("links", {});
  await app.waitFor("links: started and prewarmed", (e) => count(e, "prewarmed") > 0, 30_000);
  const registration = d.os === "win32" ? readWindows(d) : d.os === "darwin" ? readMac(d) : readLinux(d);
  for (const fact of registration.facts) {
    console.log(`links: ${fact}`);
  }
  const url = `${d.pageUrl}${LINK_QUERY}`;
  const launch = registration.launch?.(url);
  let launched = "no launch command could be read";
  const beforeLaunch = app.events().length;
  if (launch) {
    const result = d.exec(launch.command, launch.args, { env: app.env(), ...launch.options });
    launched = `${launch.options?.verbatim?.argv0 ?? launch.command} ${launch.args.join(" ")} → code ${result.code}`;
    console.log(`links: launched as the OS would: ${launched}`);
    await app.waitFor("links: the link opened", (e) => linkLoaded(since(e, beforeLaunch)), 45_000);
    // The summon's own late lines (`window-front` about a second after it) come before the refusals.
    await d.sleep(2500);
  }
  const afterLaunch = since(app.events(), beforeLaunch);
  const beforeRefusals = app.events().length;
  for (const [text] of REFUSED_LINKS) {
    await app.forward(["--link", text]);
  }
  await app.waitFor("links: every refusal logged", (e) => count(since(e, beforeRefusals), "link-refused") >= REFUSED_LINKS.length, 20_000);
  await d.sleep(2000);
  const afterRefusals = since(app.events(), beforeRefusals);
  const aliveAfterRefusals = d.alive(app.pid());
  const beforeExtras = app.events().length;
  if (d.os === "darwin" && launch) {
    const bundle = launch.args[1] as string;
    d.exec("open", ["-a", bundle, "file:///etc/passwd"]);
    await d.sleep(3000);
    d.exec("open", ["-a", bundle, `${d.pageUrl}?link=batch-1`, `${d.pageUrl}?link=batch-2`]);
    await d.sleep(5000);
  }
  const macExtras = since(app.events(), beforeExtras);
  const events = app.events();
  await app.quit();
  return { registration, launched, afterLaunch, afterRefusals, aliveAfterRefusals, macExtras, events };
}

/** `link-opened`, then the smoke page from the link loaded and reporting (`LJ-SMOKE ready`). */
export function linkLoaded(events: LogEvent[]): boolean {
  const opened = events.findIndex((e) => e.event === "link-opened");
  if (opened < 0) {
    return false;
  }
  const after = events.slice(opened);
  return after.some((e) => e.event === "page-loaded" && e.detail.includes(LINK_QUERY)) && lastTitle(after, "LJ-SMOKE ready") !== undefined;
}

/** Events a refused link must never cause. */
const NEVER_AFTER_REFUSAL = ["link-opened", "quit", "page-started", "window-created", "closing", "hibernated", "settings-window", "extension-opened"];

export function linksFailures(run: LinksRun): string[] {
  const failures = [...run.registration.failures, ...run.registration.defaults];
  if (!linkLoaded(run.afterLaunch)) {
    failures.push(`links: a link launched as the OS would (${run.launched}) didn't log link-opened and load the smoke page`);
  }
  const refused = run.afterRefusals.filter((e) => e.event === "link-refused").map((e) => e.detail.split(" ")[0]);
  const wanted = REFUSED_LINKS.map(([, why]) => why);
  if (refused.join(",") !== wanted.join(",")) {
    failures.push(`links: --link refusals logged [${refused.join(", ")}], not [${wanted.join(", ")}]`);
  }
  const caused = run.afterRefusals.filter((e) => NEVER_AFTER_REFUSAL.includes(e.event)).map((e) => e.event);
  if (caused.length > 0) {
    failures.push(`links: a refused link caused ${[...new Set(caused)].join(", ")}`);
  }
  if (!run.aliveAfterRefusals) {
    failures.push("links: the app wasn't running after the refused links (--link --quit must not quit)");
  }
  const fileOpened = run.macExtras.some((e) => e.event === "page-started" && e.detail.startsWith("file:"));
  if (fileOpened) {
    failures.push("links: a file URL from `open -a` opened in the page");
  }
  return failures;
}

export function linksSummary(run: LinksRun): Record<string, unknown> {
  const lines = (events: LogEvent[]) => events.filter((e) => e.event.startsWith("link-")).map((e) => `${e.event} ${e.detail}`.trim());
  return {
    facts: run.registration.facts,
    launched: run.launched,
    afterLaunch: lines(run.afterLaunch),
    afterRefusals: lines(run.afterRefusals),
    aliveAfterRefusals: run.aliveAfterRefusals,
    macExtras: lines(run.macExtras),
    rules: linksFailures(run),
  };
}
