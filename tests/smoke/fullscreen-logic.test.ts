import { describe, expect, it } from "vitest";
import {
  agentKind,
  centre,
  ENTER_WAIT_MS,
  FS_BUTTONS,
  FsAttempt,
  fsReadyTitle,
  fsRequest,
  HOLD_LIMIT_MS,
  playerPath,
  unavailableTitle,
} from "./fullscreen-logic.mjs";

const IPHONE =
  "Mozilla/5.0 (iPhone; CPU iPhone OS 18_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Mobile/15E148 Safari/604.1";
const ANDROID = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Mobile Safari/537.36";
const MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)";

type Box = { x: number; y: number; width: number; height: number };
const inside = (box: Box, point: { x: number; y: number }) =>
  point.x > box.x && point.x < box.x + box.width && point.y > box.y && point.y < box.y + box.height;

describe("the buttons", () => {
  it("sit inside the viewport", () => {
    for (const box of Object.values(FS_BUTTONS)) {
      expect(box.x).toBeGreaterThanOrEqual(0);
      expect(box.y).toBeGreaterThanOrEqual(0);
      expect(box.x + box.width).toBeLessThanOrEqual(1);
      expect(box.y + box.height).toBeLessThanOrEqual(1);
    }
  });

  it("leave a wide gap around the middle of the window, where a stray click does nothing", () => {
    const { element, phone } = FS_BUTTONS;
    expect(element.y + element.height).toBeLessThanOrEqual(0.4);
    expect(phone.y).toBeGreaterThanOrEqual(0.6);
    expect(inside(element, { x: 0.5, y: 0.5 })).toBe(false);
    expect(inside(phone, { x: 0.5, y: 0.5 })).toBe(false);
  });

  it("put the exit in the middle of the screen, with room around the centre", () => {
    const { exit } = FS_BUTTONS;
    expect(centre(exit)).toEqual({ x: 0.5, y: 0.5 });
    // A click 10 % off the screen's centre (a taskbar, a menu bar) still lands on it.
    for (const point of [
      { x: 0.5, y: 0.4 },
      { x: 0.5, y: 0.6 },
      { x: 0.4, y: 0.5 },
    ]) {
      expect(inside(exit, point)).toBe(true);
    }
  });

  it("are clicked at their centres", () => {
    expect(centre(FS_BUTTONS.element).y).toBeCloseTo(0.215);
    expect(centre(FS_BUTTONS.phone).y).toBeCloseTo(0.785);
    expect(centre(FS_BUTTONS.element).x).toBeCloseTo(0.5);
  });
});

describe("the request and the route", () => {
  it("is ?fs=both", () => {
    expect(fsRequest("?fs=both")).toBe(true);
    expect(fsRequest("?fs=element")).toBe(false);
    expect(fsRequest("")).toBe(false);
  });

  it("takes the iOS route under an iPhone agent, element fullscreen otherwise", () => {
    expect(playerPath(IPHONE)).toBe("ios");
    expect(playerPath(ANDROID)).toBe("element");
    expect(playerPath(MAC)).toBe("element");
  });

  it("names the agent's kind", () => {
    expect(agentKind(IPHONE)).toBe("phone");
    expect(agentKind(ANDROID)).toBe("phone");
    expect(agentKind(MAC)).toBe("desktop");
  });
});

describe("titles", () => {
  it("say what the page is ready for", () => {
    expect(fsReadyTitle(390.4, IPHONE)).toBe("LJ-FS ready width=390 ua=phone");
    expect(fsReadyTitle(720, MAC)).toBe("LJ-FS ready width=720 ua=desktop");
  });

  it("say why the phone player can't play", () => {
    expect(unavailableTitle(390, IPHONE, "no-capture-stream")).toBe("LJ-FS phone width=390 ua=phone unavailable=no-capture-stream");
    expect(unavailableTitle(390, IPHONE, "Bad Reason!")).toBe("LJ-FS phone width=390 ua=phone unavailable=bad-reason-");
  });
});

describe("an attempt", () => {
  it("that holds until the page's own exit", () => {
    const attempt = new FsAttempt("element", 1000);
    attempt.note("request");
    expect(attempt.observe(true, 1300, "fullscreenchange")).toBe(false);
    expect(attempt.observe(true, 1400, "poll")).toBe(false);
    attempt.askExit(3300);
    expect(attempt.observe(false, 3500, "fullscreenchange")).toBe(true);
    expect(attempt.done).toBe(true);
    expect(attempt.report(390, MAC)).toBe(
      "LJ-FS element width=390 ua=desktop entered=1 held_ms=2200 exited_by_page=1 events=click,request,on:fullscreenchange,exit,off:fullscreenchange",
    );
  });

  it("that drops out at once", () => {
    const attempt = new FsAttempt("phone", 0);
    attempt.note("webkit-enter");
    attempt.observe(true, 200, "poll");
    expect(attempt.observe(false, 450, "poll")).toBe(true);
    // The exit click that comes later changes nothing.
    attempt.askExit(2000);
    expect(attempt.report(390, IPHONE)).toBe(
      "LJ-FS phone width=390 ua=phone entered=1 held_ms=250 exited_by_page=0 events=click,webkit-enter,on:poll,off:poll",
    );
  });

  it("that never enters", () => {
    const attempt = new FsAttempt("element", 0);
    attempt.note("request");
    attempt.note("request-refused");
    expect(attempt.expire(ENTER_WAIT_MS - 1)).toBe(false);
    expect(attempt.expire(ENTER_WAIT_MS)).toBe(true);
    expect(attempt.report(720, MAC)).toBe(
      "LJ-FS element width=720 ua=desktop entered=0 held_ms=0 exited_by_page=0 events=click,request,request-refused,timeout",
    );
  });

  it("whose exit does nothing", () => {
    const attempt = new FsAttempt("element", 0);
    attempt.observe(true, 100, "fullscreenchange");
    attempt.askExit(2100);
    expect(attempt.expire(100 + HOLD_LIMIT_MS - 1)).toBe(false);
    expect(attempt.expire(100 + HOLD_LIMIT_MS)).toBe(true);
    expect(attempt.report(390, ANDROID)).toBe(
      `LJ-FS element width=390 ua=phone entered=1 held_ms=${HOLD_LIMIT_MS} exited_by_page=0 events=click,on:fullscreenchange,exit,timeout`,
    );
    // Over: nothing else is recorded.
    expect(attempt.observe(false, 9000, "poll")).toBe(false);
  });

  it("notes each thing once in a row, as plain names", () => {
    const attempt = new FsAttempt("element", 0);
    attempt.note("fullscreenchange");
    attempt.note("fullscreenchange");
    attempt.note("lock:NotSupportedError");
    expect(attempt.events).toEqual(["click", "fullscreenchange", "lock:notsupportederror"]);
  });
});
