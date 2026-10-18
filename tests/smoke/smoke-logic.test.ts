import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import {
  CODECS,
  codecSupport,
  fullscreenTitle,
  isFullscreen,
  isPhoneAgent,
  parseCookies,
  PERSIST_SET_TITLE,
  PERSIST_STORAGE_KEY,
  persistCookies,
  persistRequest,
  persistSet,
  persistTitle,
  readyTitle,
  requestFullscreen,
  viewportTitle,
  desktopMinWidth,
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
    expect(viewportTitle(390, 560, IPHONE, 2)).toBe("LJ-SMOKE viewport=390x560 ua=phone dpr=2 uamobile=undefined");
    expect(viewportTitle(900.4, 559.6, WEBVIEW2, 1, false)).toBe("LJ-SMOKE viewport=900x560 ua=desktop dpr=1 uamobile=false");
    expect(viewportTitle(390, 560, IPHONE, 1, true)).toBe("LJ-SMOKE viewport=390x560 ua=phone dpr=1 uamobile=true");
  });

  it("lays out like a desktop page with a minimum width under a desktop agent, unless ?fits=1", () => {
    expect(desktopMinWidth(WEBVIEW2, "")).toBe(true);
    expect(desktopMinWidth(WEBVIEW2, "?fits=1&t=x")).toBe(false);
    expect(desktopMinWidth(IPHONE, "")).toBe(false);
  });
});

describe("smoke.html", () => {
  it("loads the tested module and reports through the title", () => {
    const html = readFileSync(resolve(import.meta.dirname, "smoke.html"), "utf8");
    expect(html).toContain('from "./smoke-logic.mjs"');
    expect(html).toContain("readyTitle(codecSupport(");
    expect(html).toContain("viewportTitle(window.innerWidth, window.innerHeight, navigator.userAgent");
    expect(html).toContain("navigator.userAgentData?.mobile");
    expect(html).toContain("desktopMinWidth(navigator.userAgent, location.search)");
    expect(html).toContain('id="go"');
  });

  it("runs the persistence check instead of the other reports when the URL asks for it", () => {
    const html = readFileSync(resolve(import.meta.dirname, "smoke.html"), "utf8");
    expect(html).toContain("persistRequest(location.search)");
    expect(html).toContain("persistSet(document, localStorage, persist.token)");
    expect(html).toContain("persistTitle(document.cookie, stored(), persist.token)");
  });
});

describe("persistence check", () => {
  it("reads the mode and a cookie-safe token from the query", () => {
    expect(persistRequest("?persist=set&t=abc-1_2")).toEqual({ mode: "set", token: "abc-1_2" });
    expect(persistRequest("?t=x9&persist=check")).toEqual({ mode: "check", token: "x9" });
    for (const bad of ["", "?persist=set", "?persist=set&t=", "?persist=drop&t=a", "?persist=set&t=a;b", "?persist=set&t=a%20b"]) {
      expect(persistRequest(bad), bad).toBeUndefined();
    }
    expect(persistRequest(`?persist=set&t=${"a".repeat(65)}`)).toBeUndefined();
  });

  it("writes a persistent cookie, a session cookie and localStorage", () => {
    expect(persistCookies("tok")).toEqual(["lj_p=tok; Max-Age=31536000; Path=/; SameSite=Lax", "lj_s=tok; Path=/"]);
    const written: string[] = [];
    const doc = {
      set cookie(value: string) {
        written.push(value);
      },
      get cookie() {
        return "";
      },
    };
    const storage = new Map<string, string>();
    const title = persistSet(doc, { setItem: (key, value) => void storage.set(key, value) }, "tok");
    expect(title).toBe(PERSIST_SET_TITLE);
    expect(title).toBe("LJ-PERSIST set");
    expect(written).toEqual(persistCookies("tok"));
    expect(storage.get(PERSIST_STORAGE_KEY)).toBe("tok");
  });

  it("parses document.cookie", () => {
    expect([...parseCookies("a=1; lj_p=x;lj_s = y ;junk; =z; a=2")]).toEqual([
      ["a", "1"],
      ["lj_p", "x"],
      ["lj_s", "y"],
    ]);
    expect(parseCookies("").size).toBe(0);
  });

  it("reports which values still hold the token", () => {
    expect(persistTitle("lj_p=t1; lj_s=t1", "t1", "t1")).toBe("LJ-PERSIST p=1 s=1 l=1");
    expect(persistTitle("lj_p=t1", "t1", "t1")).toBe("LJ-PERSIST p=1 s=0 l=1");
    // An older token doesn't count: the check is about the value set last.
    expect(persistTitle("lj_p=t0; lj_s=t1", "t0", "t1")).toBe("LJ-PERSIST p=0 s=1 l=0");
    expect(persistTitle("", null, "t1")).toBe("LJ-PERSIST p=0 s=0 l=0");
  });
});
