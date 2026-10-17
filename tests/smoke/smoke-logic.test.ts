import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import {
  CODECS,
  codecSupport,
  fullscreenTitle,
  isFullscreen,
  isPhoneAgent,
  readyTitle,
  requestFullscreen,
  viewportTitle,
} from "./smoke-logic.mjs";

describe("codecSupport", () => {
  it("prefers MSE, then canPlayType, then no", () => {
    const video = { canPlayType: (type: string) => (type === CODECS.vp9 ? "maybe" : "") };
    const mediaSource = { isTypeSupported: (type: string) => type === CODECS.h264 };
    expect(codecSupport(video, mediaSource)).toEqual({ h264: "mse", vp9: "maybe", av1: "no" });
  });

  it("works without MediaSource", () => {
    const video = { canPlayType: () => "probably" };
    expect(codecSupport(video, undefined)).toEqual({ h264: "probably", vp9: "probably", av1: "probably" });
  });
});

describe("titles", () => {
  it("have the shape the CI smoke test looks for", () => {
    expect(readyTitle({ h264: "mse", vp9: "no", av1: "maybe" })).toBe("LJ-SMOKE ready h264=mse vp9=no av1=maybe");
    expect(fullscreenTitle(true)).toBe("LJ-SMOKE fullscreen=on");
    expect(fullscreenTitle(false)).toBe("LJ-SMOKE fullscreen=off");
  });
});

describe("fullscreen helpers", () => {
  it("reads standard and prefixed state", () => {
    const element = {} as Element;
    expect(isFullscreen({ fullscreenElement: element })).toBe(true);
    expect(isFullscreen({ webkitFullscreenElement: element })).toBe(true);
    expect(isFullscreen({ fullscreenElement: null })).toBe(false);
  });

  it("falls back to the prefixed request", () => {
    let prefixed = 0;
    requestFullscreen({ webkitRequestFullscreen: () => void prefixed++ });
    expect(prefixed).toBe(1);
    let standard = 0;
    requestFullscreen({ requestFullscreen: () => Promise.resolve(void standard++) });
    expect(standard).toBe(1);
  });
});

describe("viewport report", () => {
  const IPHONE =
    "Mozilla/5.0 (iPhone; CPU iPhone OS 18_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Mobile/15E148 Safari/604.1";
  const ANDROID =
    "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Mobile Safari/537.36";
  const WEBVIEW2 =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36 Edg/142.0.0.0";
  const WKWEBVIEW = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)";
  const WEBKITGTK =
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15";

  it("tells phone agents from desktop ones", () => {
    expect(isPhoneAgent(IPHONE)).toBe(true);
    expect(isPhoneAgent(ANDROID)).toBe(true);
    for (const desktop of [WEBVIEW2, WKWEBVIEW, WEBKITGTK]) {
      expect(isPhoneAgent(desktop)).toBe(false);
    }
  });

  it("reports the CSS viewport, the agent kind, and the pixel ratio", () => {
    expect(viewportTitle(390, 560, IPHONE, 2)).toBe("LJ-SMOKE viewport=390x560 ua=phone dpr=2");
    expect(viewportTitle(900.4, 559.6, WEBVIEW2, 1)).toBe("LJ-SMOKE viewport=900x560 ua=desktop dpr=1");
  });
});

describe("smoke.html", () => {
  it("loads the tested module and reports through the title", () => {
    const html = readFileSync(resolve(import.meta.dirname, "smoke.html"), "utf8");
    expect(html).toContain('from "./smoke-logic.mjs"');
    expect(html).toContain("readyTitle(codecSupport(");
    expect(html).toContain("viewportTitle(window.innerWidth, window.innerHeight, navigator.userAgent");
    expect(html).toContain('id="go"');
  });
});
