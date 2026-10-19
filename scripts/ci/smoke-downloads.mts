// The smoke test's downloads run (LJ15) and its rules. Its own launch: the smoke page first, then
// `--show` on a smoke-server address that answers a fixed 1 KB body as
// `application/octet-stream` with `Content-Disposition: attachment; filename="lj-smoke.bin"` —
// the address itself names something else (`dl-attachment`), so the file's name must come from
// the header. The log must show `download-started lj-smoke.bin` and `download-finished
// lj-smoke.bin ok`, the file must be in the OS's Downloads folder with exactly those bytes
// (`download.png`: the toolbar's "lj-smoke.bin saved" with "Show in folder"), and the same
// download again lands as `lj-smoke (1).bin`. With an extension's panel shown (hello), the same
// download is refused (`download-refused panel`) and no file appears. Then a slow download
// (`lj-slow.bin`, 1 KB every half second) is closed with the window: the server must see the
// engine stop before the end, and no file may be left in Downloads (closing the window ends a
// download, LJ15). Every file is removed afterwards. smoke.mts supplies the driver and the server; the rules are pure
// and unit-tested (smoke-downloads.test.ts).
import { existsSync, readdirSync, readFileSync, rmSync, statSync } from "node:fs";
import { join } from "node:path";
import { count, type LogEvent } from "./smoke-log.mts";
import type { SiteApp } from "./smoke-sitemode.mts";

/** The smoke server's paths (no leading slash) and the names their headers give. */
export const DOWNLOAD_PATH = "dl-attachment";
export const SLOW_PATH = "dl-slow";
export const PDF_PATH = "dl-pdf";
export const DOWNLOAD_NAME = "lj-smoke.bin";
export const SECOND_NAME = "lj-smoke (1).bin";
/** The name a download refused while a panel shows would have got: it must never appear. */
export const PANEL_NAME = "lj-smoke (2).bin";
export const SLOW_NAME = "lj-slow.bin";
/** A PDF sent as an attachment: whether each engine downloads it or shows it is informational. */
export const PDF_NAME = "lj-smoke.pdf";
export const PDF_BODY = "%PDF-1.4\n1 0 obj << /Type /Catalog >> endobj\ntrailer << /Root 1 0 R >>\n%%EOF\n";
export const DOWNLOAD_BYTES = 1024;
/** The slow download: this many 1 KB chunks, one every SLOW_EVERY_MS (20 s in all). */
export const SLOW_CHUNKS = 40;
export const SLOW_CHUNK = 1024;
export const SLOW_EVERY_MS = 500;

/** The fixed body of lj-smoke.bin: every byte value, in a pattern a truncation or a re-encoding breaks. */
export function downloadBody(): Uint8Array {
  return Uint8Array.from({ length: DOWNLOAD_BYTES }, (_, i) => (i * 31 + 7) & 0xff);
}

/** The headers both download routes answer with. */
export function attachmentHeaders(name: string, length: number): Record<string, string> {
  return {
    "content-type": "application/octet-stream",
    "content-disposition": `attachment; filename="${name}"`,
    "content-length": String(length),
    "cache-control": "no-store",
  };
}

/** What the smoke server saw of the slow download. */
export interface SlowServe {
  started: boolean;
  chunksSent: number;
  /** The client went away before the end. */
  aborted: boolean;
  completed: boolean;
}

/** What the run needs from smoke.mts. */
export interface DownloadDriver {
  /** The smoke page, the download, the slow download and the PDF attachment (full addresses). */
  pageUrl: string;
  downloadUrl: string;
  slowUrl: string;
  pdfUrl: string;
  /** The OS's Downloads folder, where the files must land. */
  downloadsDir: string;
  /** Other folders an engine's own default could have used (home, the working folder): listed if
   * a file of this run turns up there, and tidied. */
  elsewhere: string[];
  launch(name: string, settings: Record<string, unknown>): Promise<SiteApp>;
  screenshot(name: string): void;
  sleep(ms: number): Promise<void>;
  /** What the server saw of the slow download so far. */
  slowServed(): SlowServe;
}

/** What the run showed. */
export interface DownloadRun {
  events: LogEvent[];
  downloadsDir: string;
  /** The bytes of lj-smoke.bin and lj-smoke (1).bin, or null when missing. */
  first: Uint8Array | null;
  second: Uint8Array | null;
  /** The log length when hello's panel showed (the refusal comes after it). */
  panelFrom: number;
  /** A file appeared for the download refused while the panel showed. */
  panelFile: boolean;
  /** Informational: a PDF sent as an attachment was downloaded (else the page showed it). */
  pdf: { downloaded: boolean; shownInPage: boolean };
  /** This run's files found outside the Downloads folder (diagnostics). */
  strays: string[];
  /** Closing the window during the slow download. */
  slow: { started: boolean; served: SlowServe; leftInDownloads: { name: string; bytes: number }[]; afterHide: string[] };
}

/** Files of this run in the Downloads folder, partial ones an engine may leave included
 * (`lj-slow.bin.crdownload`, Chromium's `Unconfirmed 123.crdownload`). */
function isOurs(name: string): boolean {
  return /lj-(smoke|slow)/.test(name) || /^Unconfirmed \d+\.crdownload$/.test(name);
}

function listOurs(dir: string): { name: string; bytes: number }[] {
  try {
    return readdirSync(dir)
      .filter(isOurs)
      .map((name) => ({ name, bytes: statSync(join(dir, name)).size }));
  } catch {
    return [];
  }
}

function readFile(dir: string, name: string): Uint8Array | null {
  const path = join(dir, name);
  return existsSync(path) ? new Uint8Array(readFileSync(path)) : null;
}

/** Removes this run's files from the Downloads folder (before, so the names are free; and after). */
function tidy(dir: string): void {
  for (const { name } of listOurs(dir)) {
    rmSync(join(dir, name), { force: true });
  }
}

export async function downloadRun(driver: DownloadDriver): Promise<DownloadRun> {
  const dir = driver.downloadsDir;
  tidy(dir);
  for (const other of driver.elsewhere) {
    tidy(other);
  }
  const app = await driver.launch("downloads", {});
  await app.waitFor("downloads: started", (e) => count(e, "started") > 0, 30_000);
  await driver.sleep(1500);
  await app.forward(["--show", driver.pageUrl]);
  await app.waitFor("downloads: the smoke page", (e) => e.some((x) => x.event === "title" && x.detail.startsWith("LJ-SMOKE ready")), 30_000);
  await driver.sleep(1000);

  await app.forward(["--show", driver.downloadUrl]);
  await app.waitFor("downloads: the first finished", (e) => count(e, "download-finished") >= 1, 30_000);
  await driver.sleep(1200);
  driver.screenshot("download.png");
  await app.forward(["--show", driver.downloadUrl]);
  await app.waitFor("downloads: the second finished", (e) => count(e, "download-finished") >= 2, 30_000);
  await driver.sleep(500);
  const first = readFile(dir, DOWNLOAD_NAME);
  const second = readFile(dir, SECOND_NAME);

  // Informational: a PDF as an attachment — WKWebView's response policy (wry) looks at whether it
  // can show the type, not at Content-Disposition.
  const pdfFrom = app.events().length;
  await app.forward(["--show", driver.pdfUrl]);
  await app.waitFor(
    "downloads: the PDF attachment",
    (e) => e.slice(pdfFrom).some((x) => (x.event === "download-finished" && x.detail.startsWith(PDF_NAME)) || (x.event === "page-loaded" && x.detail.includes(PDF_PATH))),
    15_000,
  );
  await driver.sleep(500);
  const pdfEvents = app.events().slice(pdfFrom);
  const pdf = {
    downloaded: pdfEvents.some((e) => e.event === "download-started" && e.detail === PDF_NAME),
    shownInPage: pdfEvents.some((e) => e.event === "page-loaded" && e.detail.includes(PDF_PATH)),
  };

  // With an extension's panel shown, nothing is saved.
  const helloFrom = app.events().length;
  await app.forward(["--extension", "hello"]);
  await app.waitFor("downloads: hello's panel", (e) => e.slice(helloFrom).some((x) => x.event === "title" && x.detail.startsWith("LJ-EXT hello")), 30_000);
  await driver.sleep(800);
  const panelFrom = app.events().length;
  await app.forward(["--show", driver.downloadUrl]);
  await app.waitFor("downloads: refused while the panel shows", (e) => e.slice(panelFrom).some((x) => x.event === "download-refused"), 15_000);
  await driver.sleep(2000);
  const panelFile = existsSync(join(dir, PANEL_NAME));

  // Informational: the window closes while a download is under way.
  const pageFrom = app.events().length;
  await app.forward(["--show", driver.pageUrl]);
  await app.waitFor("downloads: back to the smoke page", (e) => e.slice(pageFrom).some((x) => x.event === "title" && x.detail.startsWith("LJ-SMOKE ready")), 30_000);
  await driver.sleep(800);
  const slowFrom = app.events().length;
  await app.forward(["--show", driver.slowUrl]);
  const started = await app.waitFor(
    "downloads: the slow one started",
    (e) => e.slice(slowFrom).some((x) => x.event === "download-started" && x.detail === SLOW_NAME),
    20_000,
  );
  await driver.sleep(1500);
  driver.screenshot("download-progress.png");
  const hideFrom = app.events().length;
  await app.forward(["--hide"]);
  await app.waitFor("downloads: hibernated", (e) => e.slice(hideFrom).some((x) => x.event === "hibernated"), 20_000);
  // Long enough for the slow download to have gone on, short of its 20 s.
  await driver.sleep(8000);
  const served = { ...driver.slowServed() };
  const leftInDownloads = listOurs(dir).filter((file) => !file.name.startsWith("lj-smoke"));
  const afterHide = app
    .events()
    .slice(hideFrom)
    .filter((e) => e.event.startsWith("download"))
    .map((e) => `${e.event} ${e.detail}`.trim());
  await app.quit();
  const events = app.events();
  const strays = driver.elsewhere.flatMap((other) => listOurs(other).map((file) => `${join(other, file.name)} (${file.bytes} bytes)`));
  tidy(dir);
  for (const other of driver.elsewhere) {
    tidy(other);
  }
  return { events, downloadsDir: dir, first, second, panelFrom, panelFile, pdf, strays, slow: { started, served, leftInDownloads, afterHide } };
}

/** Whether `event` with exactly `detail` comes after the first `before` event with `beforeDetail`. */
function inOrder(events: LogEvent[], before: [string, string], after: [string, string]): boolean {
  const at = events.findIndex((e) => e.event === before[0] && e.detail === before[1]);
  return at >= 0 && events.slice(at + 1).some((e) => e.event === after[0] && e.detail === after[1]);
}

function sameBytes(got: Uint8Array | null, want: Uint8Array): boolean {
  return got !== null && got.length === want.length && got.every((byte, i) => byte === want[i]);
}

/** Folder paths compared as the OS does (Windows and macOS ignore case; separators either way). */
function sameFolder(a: string, b: string, os: string): boolean {
  const norm = (path: string) => {
    const plain = path.replace(/[\\/]+$/, "").replace(/\\/g, "/");
    return os === "linux" ? plain : plain.toLowerCase();
  };
  return norm(a) === norm(b);
}

/**
 * The rules: both downloads started and finished `ok` under their names, in the OS's Downloads
 * folder, with the exact bytes; nothing was refused before the panel showed; with the panel
 * shown the download was refused as `panel`, nothing started, and no file appeared.
 */
export function downloadFailures(run: DownloadRun, os: string): string[] {
  const failures: string[] = [];
  const web = run.events.slice(0, run.panelFrom);
  for (const name of [DOWNLOAD_NAME, SECOND_NAME]) {
    if (!inOrder(web, ["download-started", name], ["download-finished", `${name} ok`])) {
      failures.push(`downloads: no "download-started ${name}" followed by "download-finished ${name} ok"`);
    }
  }
  const folders = web.filter((e) => e.event === "download-folder").map((e) => e.detail);
  if (folders.length === 0 || !folders.every((folder) => sameFolder(folder, run.downloadsDir, os))) {
    failures.push(`downloads: saved to ${JSON.stringify(folders)}, not the Downloads folder ${run.downloadsDir}`);
  }
  const body = downloadBody();
  if (!sameBytes(run.first, body)) {
    failures.push(`downloads: ${DOWNLOAD_NAME} isn't the server's ${DOWNLOAD_BYTES} bytes (${run.first === null ? "missing" : `${run.first.length} bytes`})`);
  }
  if (!sameBytes(run.second, body)) {
    failures.push(`downloads: ${SECOND_NAME} isn't the server's ${DOWNLOAD_BYTES} bytes (${run.second === null ? "missing" : `${run.second.length} bytes`})`);
  }
  if (web.some((e) => e.event === "download-refused")) {
    failures.push("downloads: a web page's download was refused");
  }
  const panel = run.events.slice(run.panelFrom);
  const refusedAt = panel.findIndex((e) => e.event === "download-refused");
  if (refusedAt < 0 || panel[refusedAt]?.detail !== "panel") {
    failures.push(`downloads: with hello's panel shown, no "download-refused panel" (${panel[refusedAt]?.detail ?? "nothing refused"})`);
  }
  const nextPage = panel.findIndex((e) => e.event === "title" && e.detail.startsWith("LJ-SMOKE ready"));
  const panelPart = nextPage < 0 ? panel : panel.slice(0, nextPage);
  if (panelPart.some((e) => e.event === "download-started")) {
    failures.push("downloads: a download started while hello's panel showed");
  }
  if (run.panelFile) {
    failures.push(`downloads: ${PANEL_NAME} appeared although the download was refused`);
  }
  return failures.concat(closeFailures(run));
}

/**
 * Closing the window ends a download under way (LJ15): the slow one started, the server saw the
 * engine go away before its last chunk, and nothing of it is left in Downloads.
 */
export function closeFailures(run: DownloadRun): string[] {
  if (!run.slow.started) {
    return [`downloads: the slow download (${SLOW_NAME}) never started`];
  }
  const failures: string[] = [];
  if (!run.slow.served.aborted || run.slow.served.completed) {
    failures.push(`downloads: closing the window didn't end the download under way (the server sent ${run.slow.served.chunksSent} of ${SLOW_CHUNKS} chunks${run.slow.served.completed ? ", all of them" : ""})`);
  }
  const left = run.slow.leftInDownloads.filter((file) => file.name.includes("lj-slow"));
  if (left.length > 0) {
    failures.push(`downloads: closing the window left ${left.map((file) => `${file.name} (${file.bytes} bytes)`).join(", ")} in Downloads`);
  }
  return failures;
}

/** What summary.json records. */
export function downloadSummary(run: DownloadRun): Record<string, unknown> {
  return {
    folder: run.events.find((e) => e.event === "download-folder")?.detail ?? null,
    log: run.events.filter((e) => e.event.startsWith("download")).map((e) => `${e.event} ${e.detail}`.trim()),
    firstBytes: run.first?.length ?? null,
    secondBytes: run.second?.length ?? null,
    panelFile: run.panelFile,
    pdfAttachment: run.pdf,
    elsewhere: run.strays,
    closeDuringDownload: {
      started: run.slow.started,
      serverSentChunks: run.slow.served.chunksSent,
      of: SLOW_CHUNKS,
      engineStopped: run.slow.served.aborted,
      completed: run.slow.served.completed,
      leftInDownloads: run.slow.leftInDownloads,
      afterHide: run.slow.afterHide,
    },
  };
}
