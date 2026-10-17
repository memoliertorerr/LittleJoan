import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { CODECS, codecSupport, fullscreenTitle, isFullscreen, readyTitle, requestFullscreen } from "./smoke-logic.mjs";

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

describe("smoke.html", () => {
  it("loads the tested module and reports through the title", () => {
    const html = readFileSync(resolve(import.meta.dirname, "smoke.html"), "utf8");
    expect(html).toContain('from "./smoke-logic.mjs"');
    expect(html).toContain("readyTitle(codecSupport(");
    expect(html).toContain('id="go"');
  });
});
