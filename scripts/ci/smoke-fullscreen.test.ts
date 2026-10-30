import { describe, expect, it } from "vitest";
import { FS_BUTTONS } from "../../tests/smoke/fullscreen-logic.mjs";
import { parseLog } from "./smoke-log.mts";
import {
  FS_SIZES,
  SITE_SIZE,
  clickPoint,
  fsReports,
  fullscreenFailures,
  lastPageScreen,
  pageScreenFor,
  layoutDuringWebKitFullscreen,
  parseFsReport,
  parsePageScreen,
  readyIndex,
  reportAfter,
  route,
  webkitStates,
} from "./smoke-fullscreen.mts";

const held = (path: string, width: number, ua: string) =>
  `title LJ-FS ${path} width=${width} ua=${ua} entered=1 held_ms=2104 exited_by_page=1 events=click,request,on:fullscreenchange,exit,exit-call,off:fullscreenchange`;
const dropped = (path: string, width: number, ua: string) =>
  `title LJ-FS ${path} width=${width} ua=${ua} entered=1 held_ms=180 exited_by_page=0 events=click,request,on:poll,off:poll`;
const ios = (width: number) =>
  `title LJ-FS phone width=${width} ua=phone entered=1 held_ms=120 exited_by_page=0 events=click,webkit-enter,on:poll,off:poll,no-begin,lock:notsupportederror`;

/** A whole fullscreen launch's log: every path at 390 (phone agent) and 720 (desktop agent). */
function mainLog(lines: { element390?: string; phone390?: string; element720?: string; phone720?: string; extra?: string[] }): string {
  const rows = [
    "started",
    "window-created",
    "page-screen 720,224,390,560 scale=1",
    "title LJ-FS ready width=390 ua=phone",
    lines.element390 ?? held("element", 390, "phone"),
    lines.phone390 ?? ios(390),
    "page-started http://127.0.0.1:5000/smoke.html?fs=both",
    "title LJ-FS ready width=720 ua=desktop",
    lines.element720 ?? held("element", 720, "desktop"),
    lines.phone720 ?? held("phone", 720, "desktop"),
    ...(lines.extra ?? []),
  ];
  return rows.map((row, i) => `${1000 + i} ${row}`).join("\n") + "\n";
}

const siteLog = (ua = "desktop") =>
  parseLog(`1 started\n2 title LJ-FS ready width=390 ua=${ua}\n3 ${held("phone", 390, ua)}\n`);

describe("reports", () => {
  it("parse every field", () => {
    expect(parseFsReport("LJ-FS element width=390 ua=phone entered=1 held_ms=2104 exited_by_page=1 events=click,request")).toEqual({
      path: "element",
      width: 390,
      ua: "phone",
      entered: true,
      heldMs: 2104,
      exitedByPage: true,
      events: ["click", "request"],
    });
    expect(parseFsReport("LJ-FS phone width=390 ua=phone unavailable=no-capture-stream")).toMatchObject({
      path: "phone",
      unavailable: "no-capture-stream",
      entered: false,
    });
    for (const bad of ["LJ-FS ready width=390 ua=phone", "LJ-FS element width=390 ua=phone entered=2", "LJ-SMOKE ready"]) {
      expect(parseFsReport(bad)).toBeUndefined();
    }
  });

  it("know their route", () => {
    const [phone] = fsReports(parseLog(`1 ${ios(390)}\n`));
    const [element] = fsReports(parseLog(`1 ${held("phone", 720, "desktop")}\n`));
    expect(phone && route(phone)).toBe("ios");
    expect(element && route(element)).toBe("element");
  });

  it("are found after a point in the log", () => {
    const events = parseLog(mainLog({}));
    expect(reportAfter(events, 0, "phone")?.width).toBe(390);
    expect(reportAfter(events, 6, "phone")?.width).toBe(720);
  });
});

describe("clicks", () => {
  it("read where the page is on the screen", () => {
    const events = parseLog("1 page-screen 10,20,390,560 scale=1\n2 page-screen -1440,52,780,1120 scale=2\n3 page-bounds 0,40,390,560\n");
    expect(events[0] && parsePageScreen(events[0])).toEqual({ x: 10, y: 20, width: 390, height: 560, scale: 1 });
    expect(lastPageScreen(events)).toEqual({ x: -1440, y: 52, width: 780, height: 1120, scale: 2 });
    expect(lastPageScreen(parseLog("1 started\n"))).toBeUndefined();
  });

  it("go by the page's own width when a late layout logged the fullscreen size (linux-scale-2)", () => {
    const events = parseLog(
      [
        "1 page-screen 0,80,2560,1520 scale=2",
        "2 page-screen 800,260,1440,1120 scale=2",
        "3 title LJ-FS element width=720 ua=desktop entered=1 held_ms=2340 exited_by_page=1 events=click",
        "4 page-screen 800,260,2560,1520 scale=2",
        "",
      ].join("\n"),
    );
    expect(lastPageScreen(events)).toEqual({ x: 800, y: 260, width: 2560, height: 1520, scale: 2 });
    expect(pageScreenFor(events, 720)).toEqual({ x: 800, y: 260, width: 1440, height: 1120, scale: 2 });
    // Nothing of that width: the latest, as before.
    expect(pageScreenFor(events, 390)).toEqual({ x: 800, y: 260, width: 2560, height: 1520, scale: 2 });
    expect(pageScreenFor(parseLog("1 page-screen 10,20,390,560 scale=1\n"), 390)).toEqual({ x: 10, y: 20, width: 390, height: 560, scale: 1 });
  });

  it("land in the middle of a button: pixels on Windows and Linux, points on macOS", () => {
    const screen = { x: 720, y: 224, width: 390, height: 560, scale: 1 };
    expect(clickPoint(screen, FS_BUTTONS.element, "win32")).toEqual({ x: 915, y: 344 });
    expect(clickPoint(screen, FS_BUTTONS.phone, "linux")).toEqual({ x: 915, y: 664 });
    const retina = { x: 1440, y: 448, width: 780, height: 1120, scale: 2 };
    expect(clickPoint(retina, FS_BUTTONS.element, "darwin")).toEqual({ x: 915, y: 344 });
    expect(clickPoint(retina, FS_BUTTONS.element, "linux")).toEqual({ x: 1830, y: 689 });
  });
});

describe("ready", () => {
  it("counts only after the page's last start", () => {
    const stale = parseLog("1 title LJ-FS ready width=720 ua=desktop\n2 page-started http://x/\n");
    expect(readyIndex(stale, 720, "desktop")).toBe(-1);
    const fresh = parseLog("1 title LJ-FS ready width=720 ua=phone\n2 page-started http://x/\n3 title LJ-FS ready width=720 ua=desktop\n");
    expect(readyIndex(fresh, 720, "desktop")).toBe(2);
    expect(readyIndex(fresh, 720, "phone")).toBe(-1);
    // A report after the ready title: the page isn't waiting for a click any more.
    const busy = parseLog(`1 title LJ-FS ready width=390 ua=phone\n2 ${held("element", 390, "phone")}\n`);
    expect(readyIndex(busy, 390, "phone")).toBe(-1);
  });
});

describe("fullscreenFailures", () => {
  it("pass when every element path holds until the page's own exit", () => {
    const run = { main: parseLog(mainLog({})), site: siteLog() };
    for (const os of ["win32", "linux", "darwin"]) {
      expect(fullscreenFailures(run, os)).toEqual([]);
    }
  });

  it("fail when fullscreen ends within a second of entering", () => {
    const run = { main: parseLog(mainLog({ element390: dropped("element", 390, "phone") })) };
    expect(fullscreenFailures(run, "darwin")).toEqual([
      "fullscreen: element at 390: left fullscreen by itself after 180 ms (events click,request,on:poll,off:poll)",
    ]);
  });

  it("fail when the exit does nothing, or fullscreen never starts", () => {
    const stuck =
      "title LJ-FS element width=720 ua=desktop entered=1 held_ms=6000 exited_by_page=0 events=click,request,on:fullscreenchange,exit,timeout";
    const never = "title LJ-FS phone width=720 ua=desktop entered=0 held_ms=0 exited_by_page=0 events=click,request,timeout";
    expect(fullscreenFailures({ main: parseLog(mainLog({ element720: stuck, phone720: never })) }, "linux")).toEqual([
      "fullscreen: element at 720: still fullscreen 6000 ms after entering, though its Exit was clicked",
      "fullscreen: phone at 720: never entered fullscreen (events click,request,timeout)",
    ]);
  });

  it("leave the iOS route under an iPhone agent informational, but not Windows' Android agent", () => {
    const phone390 = dropped("phone", 390, "phone");
    expect(fullscreenFailures({ main: parseLog(mainLog({ phone390: ios(390) })) }, "linux")).toEqual([]);
    expect(fullscreenFailures({ main: parseLog(mainLog({ phone390 })) }, "win32")).toEqual([
      "fullscreen: phone at 390: left fullscreen by itself after 180 ms (events click,request,on:poll,off:poll)",
    ]);
  });

  it("need every report on Windows and Linux, but only the clicks that arrived on macOS", () => {
    const missing = mainLog({}).replace(/^.*LJ-FS element width=720.*\n/m, "");
    expect(fullscreenFailures({ main: parseLog(missing) }, "darwin")).toEqual([]);
    expect(fullscreenFailures({ main: parseLog(missing) }, "win32")).toEqual(["fullscreen: element at 720: the page never reported (the click didn't reach it)"]);
  });

  it("with fullscreen off (WebKitGTK < 2.52, LJ14.1): every element path reports, none entered", () => {
    const refused = (path: string, width: number, ua: string) =>
      `title LJ-FS ${path} width=${width} ua=${ua} entered=0 held_ms=0 exited_by_page=0 events=click,request,no-fullscreen-api,timeout`;
    const off = mainLog({
      element390: refused("element", 390, "phone"),
      element720: refused("element", 720, "desktop"),
      phone720: refused("phone", 720, "desktop"),
    });
    const site = parseLog(`1 started\n2 title LJ-FS ready width=390 ua=desktop\n3 ${refused("phone", 390, "desktop")}\n`);
    expect(fullscreenFailures({ main: parseLog(off), site }, "linux", true)).toEqual([]);
    // The same reports where fullscreen must work: every path failed.
    expect(fullscreenFailures({ main: parseLog(off), site }, "linux")).toHaveLength(4);
    // A path that entered anyway, or never reported (the click didn't land, or the app died).
    expect(fullscreenFailures({ main: parseLog(mainLog({ element720: refused("element", 720, "desktop") })) }, "linux", true)).toEqual([
      "fullscreen: element at 390: entered fullscreen, which is off on this WebKitGTK (events click,request,on:fullscreenchange,exit,exit-call,off:fullscreenchange)",
      "fullscreen: phone at 720: entered fullscreen, which is off on this WebKitGTK (events click,request,on:fullscreenchange,exit,exit-call,off:fullscreenchange)",
    ]);
    const missing = off.replace(/^.*LJ-FS element width=720.*\n/m, "");
    expect(fullscreenFailures({ main: parseLog(missing) }, "linux", true)).toEqual([
      "fullscreen: element at 720: the page never reported (the click didn't reach it)",
    ]);
  });

  it("need the desktop agent at 390 in the siteModes launch", () => {
    const run = { main: parseLog(mainLog({})), site: siteLog("phone") };
    expect(fullscreenFailures(run, "win32")).toContain(`siteModes: the page never reported the desktop agent at ${SITE_SIZE.width}`);
  });

  it("fail on macOS when the page is laid out during WebKit's fullscreen", () => {
    const extra = ["wk-fullscreen entering", "wk-fullscreen in", "page-bounds 0,40,720,560", "page-visible true", "wk-fullscreen not", "page-bounds 0,40,720,560"];
    const events = parseLog(mainLog({ extra }));
    expect(layoutDuringWebKitFullscreen(events).map((e) => e.event)).toEqual(["page-bounds", "page-visible"]);
    expect(webkitStates(events)).toEqual(["entering", "in", "not"]);
    expect(fullscreenFailures({ main: events }, "darwin")).toEqual([
      "fullscreen: the page was laid out during WebKit's fullscreen (page-bounds 0,40,720,560; page-visible true)",
    ]);
    // Only macOS has WebKit's fullscreen window.
    expect(fullscreenFailures({ main: events }, "linux")).toEqual([]);
  });

  it("cover both widths with both paths, and the siteModes launch at a phone width", () => {
    expect(FS_SIZES.map((size) => [size.width, size.ua, size.paths.join("+")])).toEqual([
      [390, "phone", "element+phone"],
      [720, "desktop", "element+phone"],
    ]);
    expect([SITE_SIZE.width, SITE_SIZE.ua]).toEqual([390, "desktop"]);
    expect([...FS_SIZES.flatMap((size) => Object.values(size.shots)), ...Object.values(SITE_SIZE.shots)]).toEqual([
      "fs/element-390.png",
      "fs/phone-390.png",
      "fs/element-720.png",
      "fs/desktop-site-390.png",
    ]);
  });
});
