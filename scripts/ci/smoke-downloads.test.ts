import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import {
  DOWNLOAD_BYTES,
  PANEL_NAME,
  SLOW_CHUNKS,
  attachmentHeaders,
  downloadBody,
  downloadFailures,
  downloadSummary,
  type DownloadRun,
} from "./smoke-downloads.mts";

const log = (rows: string[]) => parseLog(rows.map((row, i) => `${1000 + i} ${row}`).join("\n") + "\n");

const DIR = "/home/runner/Downloads";

const WEB = [
  "started",
  "window-created",
  "title LJ-SMOKE ready h264=mse",
  `download-folder ${DIR}`,
  "download-started lj-smoke.bin",
  "download-finished lj-smoke.bin ok",
  `download-folder ${DIR}`,
  "download-started lj-smoke (1).bin",
  "download-finished lj-smoke (1).bin ok",
  "extension-opened hello",
  "title LJ-EXT hello storage=ok theme=ok ipc=denied net=blocked sw=refused",
];
const PANEL = ["download-refused panel", "title LJ-SMOKE ready h264=mse", "download-started lj-slow.bin", "hibernated", "download-closed lj-slow.bin"];

function good(over: Partial<DownloadRun> = {}, rows: string[] = [...WEB, ...PANEL]): DownloadRun {
  return {
    events: log(rows),
    downloadsDir: DIR,
    first: downloadBody(),
    second: downloadBody(),
    panelFrom: WEB.length,
    panelFile: false,
    pdf: { downloaded: true, shownInPage: false },
    slow: { started: true, served: { started: true, chunksSent: 4, aborted: true, completed: false }, leftInDownloads: [], afterHide: [] },
    ...over,
  };
}

describe("the downloads run's rules (LJ15)", () => {
  it("passes two saved downloads and a refusal while hello's panel shows", () => {
    expect(downloadFailures(good(), "linux")).toEqual([]);
  });

  it("serves a fixed 1 KB body as an attachment", () => {
    const body = downloadBody();
    expect(body).toHaveLength(DOWNLOAD_BYTES);
    expect(new Set(body).size).toBe(256);
    expect(attachmentHeaders("lj-smoke.bin", body.length)).toEqual({
      "content-type": "application/octet-stream",
      "content-disposition": 'attachment; filename="lj-smoke.bin"',
      "content-length": "1024",
      "cache-control": "no-store",
    });
  });

  it("wants each name started, then finished ok", () => {
    const failed = WEB.map((row) => (row === "download-finished lj-smoke (1).bin ok" ? "download-finished lj-smoke (1).bin failed" : row));
    expect(downloadFailures(good({}, [...failed, ...PANEL]), "linux")).toEqual([
      'downloads: no "download-started lj-smoke (1).bin" followed by "download-finished lj-smoke (1).bin ok"',
    ]);
    // The engine's own name (the address names "dl-attachment") means the header was ignored.
    const named = WEB.map((row) => row.replace("lj-smoke.bin", "dl-attachment"));
    expect(downloadFailures(good({}, [...named, ...PANEL]), "linux")[0]).toContain('"download-started lj-smoke.bin"');
  });

  it("wants the OS's Downloads folder, compared as the OS does", () => {
    const elsewhere = WEB.map((row) => row.replace(`download-folder ${DIR}`, "download-folder /tmp"));
    expect(downloadFailures(good({}, [...elsewhere, ...PANEL]), "linux")[0]).toContain("not the Downloads folder");
    const windows = WEB.map((row) => row.replace(`download-folder ${DIR}`, "download-folder C:\\Users\\runneradmin\\Downloads"));
    expect(downloadFailures(good({ downloadsDir: "C:\\Users\\RUNNERADMIN\\Downloads\\" }, [...windows, ...PANEL]), "win32")).toEqual([]);
    expect(downloadFailures(good({ downloadsDir: "/home/Runner/Downloads" }), "linux")[0]).toContain("not the Downloads folder");
  });

  it("wants the exact bytes in both files", () => {
    const short = downloadBody().slice(0, 1000);
    const changed = downloadBody();
    changed[10] = (changed[10] ?? 0) ^ 1;
    expect(downloadFailures(good({ first: null }), "darwin")).toEqual(["downloads: lj-smoke.bin isn't the server's 1024 bytes (missing)"]);
    expect(downloadFailures(good({ second: short }), "darwin")).toEqual(["downloads: lj-smoke (1).bin isn't the server's 1024 bytes (1000 bytes)"]);
    expect(downloadFailures(good({ first: changed }), "darwin")).toEqual(["downloads: lj-smoke.bin isn't the server's 1024 bytes (1024 bytes)"]);
  });

  it("refuses with the panel shown — and nothing else", () => {
    expect(downloadFailures(good({}, [...WEB, ...PANEL.slice(1)]), "win32")).toContain(
      'downloads: with hello\'s panel shown, no "download-refused panel" (nothing refused)',
    );
    const scheme = [...WEB, "download-refused scheme", ...PANEL.slice(1)];
    expect(downloadFailures(good({}, scheme), "win32")[0]).toContain("(scheme)");
    const started = [...WEB, "download-refused panel", "download-started lj-smoke (2).bin", ...PANEL.slice(1)];
    expect(downloadFailures(good({}, started), "win32")).toEqual(["downloads: a download started while hello's panel showed"]);
    expect(downloadFailures(good({ panelFile: true }), "win32")).toEqual([`downloads: ${PANEL_NAME} appeared although the download was refused`]);
    const refusedWeb = [...WEB.slice(0, 4), "download-refused scheme", ...WEB.slice(4)];
    expect(downloadFailures(good({ panelFrom: refusedWeb.length }, [...refusedWeb, ...PANEL]), "linux")).toContain(
      "downloads: a web page's download was refused",
    );
  });

  it("records the close during a download, informational", () => {
    const summary = downloadSummary(good());
    expect(summary.folder).toBe(DIR);
    expect(summary.firstBytes).toBe(DOWNLOAD_BYTES);
    expect(summary.closeDuringDownload).toEqual({
      started: true,
      serverSentChunks: 4,
      of: SLOW_CHUNKS,
      engineStopped: true,
      completed: false,
      leftInDownloads: [],
      afterHide: [],
    });
    // Whatever the engine did, the rules don't depend on it.
    const kept = good({ slow: { started: true, served: { started: true, chunksSent: 40, aborted: false, completed: true }, leftInDownloads: [{ name: "lj-slow.bin", bytes: 40960 }], afterHide: [] } });
    expect(downloadFailures(kept, "darwin")).toEqual([]);
  });
});
