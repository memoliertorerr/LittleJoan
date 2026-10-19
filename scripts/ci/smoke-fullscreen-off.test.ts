import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { fullscreenOffFailures, fullscreenOffVersions, olderThan252 } from "./smoke-fullscreen-off.mts";

const log = (rows: string[]) => parseLog(rows.map((row, i) => `${1000 + i} ${row}`).join("\n") + "\n");

/** A launch on Ubuntu 22.04's WebKitGTK: fullscreen off, the click changes nothing. */
const offLaunch = (extra: string[] = []) =>
  log(["started", "window-created", "fullscreen-off 2.50.4", "title LJ-SMOKE ready h264=mse", "title LJ-SMOKE viewport=1280x800 ua=phone dpr=1", ...extra, "hibernated"]);
/** The same on 24.04: fullscreen on, the click fullscreens the window. */
const onLaunch = (extra: string[] = []) =>
  log(["started", "window-created", "title LJ-SMOKE ready h264=mse", "fullscreen true", "title LJ-SMOKE fullscreen=on", "fullscreen false", ...extra, "hibernated"]);

describe("fullscreen switched off (LJ14.1)", () => {
  it("reads the versions and knows which are older than 2.52", () => {
    expect(fullscreenOffVersions(offLaunch())).toEqual(["2.50.4"]);
    expect(olderThan252("2.50.4")).toBe(true);
    expect(olderThan252("2.44.0")).toBe(true);
    expect(olderThan252("1.99.9")).toBe(true);
    expect(olderThan252("2.52.6")).toBe(false);
    expect(olderThan252("2.54.0")).toBe(false);
    expect(olderThan252("3.0.0")).toBe(false);
    expect(olderThan252("2.50")).toBe(false);
    expect(olderThan252("")).toBe(false);
  });

  it("passes a 22.04 run: fullscreen off in every launch, nothing went fullscreen", () => {
    expect(fullscreenOffFailures(true, { main: offLaunch(), fullscreen: { main: offLaunch(), site: offLaunch() } })).toEqual([]);
  });

  it("fails a 22.04 run where fullscreen wasn't switched off, or the window went fullscreen anyway", () => {
    expect(fullscreenOffFailures(true, { main: onLaunch(), fullscreen: { main: offLaunch() } })).toEqual([
      'main: no "fullscreen-off" — element fullscreen must be off on this WebKitGTK',
      'main: "fullscreen true" — something went fullscreen though fullscreen is off',
      "main: the page reported element fullscreen though fullscreen is off",
    ]);
    expect(fullscreenOffFailures(true, { main: offLaunch(), fullscreen: { main: offLaunch(["element-fullscreen true"]) } })).toEqual([
      'fullscreen: "element-fullscreen true" — something went fullscreen though fullscreen is off',
    ]);
  });

  it("fails a version that isn't older than 2.52 (the switch must follow the engine)", () => {
    const wrong = log(["started", "fullscreen-off 2.52.6", "hibernated"]);
    expect(fullscreenOffFailures(true, { main: wrong, fullscreen: { main: offLaunch() } })).toEqual([
      'main: "fullscreen-off 2.52.6", but that WebKitGTK isn\'t older than 2.52',
    ]);
  });

  it("fails any fullscreen-off where fullscreen must stay on (24.04, the AppImage, Windows, macOS)", () => {
    expect(fullscreenOffFailures(false, { main: onLaunch(), fullscreen: { main: onLaunch(), site: onLaunch() } })).toEqual([]);
    expect(fullscreenOffFailures(false, { main: onLaunch(), fullscreen: { main: onLaunch(), site: offLaunch() } })).toEqual([
      'siteModes: logged "fullscreen-off 2.50.4" on an engine whose fullscreen must stay on',
    ]);
  });
});
