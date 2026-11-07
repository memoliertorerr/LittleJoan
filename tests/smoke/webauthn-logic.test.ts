import { describe, expect, it } from "vitest";
import { ASK_TIMEOUT_MS, parseWebauthnTitle, webauthnReport, webauthnRequest, webauthnTitle } from "./webauthn-logic.mjs";

/** A `PublicKeyCredential` stand-in whose static questions answer as given. */
function credential(platform: () => unknown, conditional?: () => unknown): unknown {
  const fake = function PublicKeyCredential() {} as unknown as Record<string, unknown>;
  fake.isUserVerifyingPlatformAuthenticatorAvailable = platform;
  if (conditional) {
    fake.isConditionalMediationAvailable = conditional;
  }
  return fake;
}

describe("the smoke page's passkey report (LJ52)", () => {
  it("is asked for with ?webauthn=1 only", () => {
    expect(webauthnRequest("?webauthn=1")).toBe(true);
    expect(webauthnRequest("?find=1&webauthn=1")).toBe(true);
    for (const search of ["", "?webauthn=0", "?webauthn", "?webauthn=true"]) {
      expect(webauthnRequest(search)).toBe(false);
    }
  });

  it("says what the engine offers, in one title, and reads it back", () => {
    const title = webauthnTitle({ api: true, platform: true, conditional: "error", secure: true });
    expect(title).toBe("LJ-WEBAUTHN api=1 platform=true conditional=error secure=true");
    expect(parseWebauthnTitle(title)).toEqual({ api: true, platform: true, conditional: "error", secure: true });
    expect(parseWebauthnTitle("LJ-WEBAUTHN api=0 platform=false conditional=false")).toEqual({ api: false, platform: false, conditional: false, secure: false });
    for (const other of [undefined, "", "LJ-SMOKE ready", "LJ-WEBAUTHN api=2 platform=true conditional=true", "LJ-WEBAUTHN api=1 platform=yes conditional=true"]) {
      expect(parseWebauthnTitle(other)).toBeNull();
    }
  });

  it("reports no API when the engine has no PublicKeyCredential", async () => {
    expect(await webauthnReport({ isSecureContext: true })).toEqual({ api: false, platform: false, conditional: false, secure: true });
    expect(await webauthnReport({ PublicKeyCredential: {}, isSecureContext: false })).toEqual({ api: false, platform: false, conditional: false, secure: false });
  });

  it("asks both questions, and a missing one is false", async () => {
    expect(await webauthnReport({ PublicKeyCredential: credential(() => Promise.resolve(true), () => Promise.resolve(false)), isSecureContext: true })).toEqual({
      api: true,
      platform: true,
      conditional: false,
      secure: true,
    });
    expect(await webauthnReport({ PublicKeyCredential: credential(() => Promise.resolve(false)), isSecureContext: true })).toMatchObject({ platform: false, conditional: false });
  });

  it("counts a throw, a rejection, an odd answer or a hang as an error", async () => {
    const report = await webauthnReport({
      PublicKeyCredential: credential(
        () => {
          throw new Error("NotAllowedError");
        },
        () => Promise.reject(new Error("SecurityError")),
      ),
    });
    expect(report).toEqual({ api: true, platform: "error", conditional: "error", secure: false });
    expect((await webauthnReport({ PublicKeyCredential: credential(() => Promise.resolve("yes")) })).platform).toBe("error");
    const hung = await webauthnReport({ PublicKeyCredential: credential(() => new Promise(() => undefined)) }, 20);
    expect(hung.platform).toBe("error");
    expect(ASK_TIMEOUT_MS).toBe(5000);
  });
});
