import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import {
  POPUP_FEATURES,
  POPUP_NAME,
  POPUP_READY_TITLE,
  SIGNED_IN,
  SIGNIN_DELAY_MS,
  isSignedIn,
  openedTitle,
  popupRequest,
  popupTitle,
  signinStays,
  signinUrl,
} from "./popup-logic.mjs";

const AT = "http://127.0.0.1:8124";

describe("the popup opener (LJ47)", () => {
  it("reads its mode, the sign-in origin and whether the sign-in page stays", () => {
    expect(popupRequest(`?popup=oauth&at=${encodeURIComponent(AT)}`)).toEqual({ mode: "oauth", at: AT, stay: false });
    expect(popupRequest(`?popup=blank&at=${AT}&stay=1`)).toEqual({ mode: "blank", at: AT, stay: true });
  });

  it("refuses any other mode or sign-in origin", () => {
    for (const bad of [
      "",
      "?popup=oauth",
      `?popup=other&at=${AT}`,
      "?popup=oauth&at=https://evil.example",
      "?popup=oauth&at=http://127.0.0.1:8124/x",
      "?popup=oauth&at=http://127.0.0.1",
      "?popup=oauth&at=javascript:alert(1)",
    ]) {
      expect(popupRequest(bad), bad).toBeUndefined();
    }
  });

  it("opens the sign-in page on the other origin, sized like a sign-in library's", () => {
    expect(signinUrl(AT, false)).toBe(`${AT}/signin.html`);
    expect(signinUrl(AT, true)).toBe(`${AT}/signin.html?stay=1`);
    expect(POPUP_FEATURES).toMatch(/^width=\d+,height=\d+$/);
    expect(POPUP_NAME).not.toMatch(/^_/);
  });

  it("counts only the sign-in page's own message", () => {
    expect(isSignedIn(SIGNED_IN, AT, AT)).toBe(true);
    expect(isSignedIn(SIGNED_IN, "http://127.0.0.1:8123", AT)).toBe(false);
    expect(isSignedIn("other", AT, AT)).toBe(false);
    expect(isSignedIn({ type: SIGNED_IN }, AT, AT)).toBe(false);
  });

  it("reports what the smoke reads", () => {
    expect(POPUP_READY_TITLE).toBe("LJ-POPUP ready");
    expect(popupTitle(1, true)).toBe("LJ-POPUP received=1 closed=1");
    expect(popupTitle(0, false)).toBe("LJ-POPUP received=0 closed=0");
    expect(openedTitle(true)).toBe("LJ-POPUP opened=1");
  });
});

describe("the fake sign-in page (LJ47)", () => {
  it("stays only with ?stay=1", () => {
    expect(signinStays("?stay=1")).toBe(true);
    for (const other of ["", "?stay=0", "?stay=", "?popup=oauth"]) {
      expect(signinStays(other), other).toBe(false);
    }
  });

  it("posts back, then closes, after a moment for the screenshot", () => {
    const html = readFileSync(resolve(import.meta.dirname, "signin.html"), "utf8");
    expect(html).toContain('from "./popup-logic.mjs"');
    expect(html).toContain("window.opener?.postMessage(SIGNED_IN");
    expect(html).toContain("window.close()");
    expect(SIGNIN_DELAY_MS).toBeGreaterThanOrEqual(1000);
  });

  it("is wired into smoke.html", () => {
    const html = readFileSync(resolve(import.meta.dirname, "smoke.html"), "utf8");
    expect(html).toContain('from "./popup-logic.mjs"');
    expect(html).toContain("popupRequest(location.search)");
    expect(html).toContain("window.open(\"about:blank\", POPUP_NAME, POPUP_FEATURES)");
  });
});
