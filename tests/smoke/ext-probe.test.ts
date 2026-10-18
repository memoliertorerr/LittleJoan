import { describe, expect, it } from "vitest";
import { probeApi, probeFile, probeRequest, probeTitle } from "./ext-probe.mjs";

describe("the extension probe", () => {
  it("takes an extension origin in either platform's form, nothing else", () => {
    expect(probeRequest("?ext=probe&base=ljext%3A%2F%2Fhello.localhost")).toEqual({ base: "ljext://hello.localhost" });
    expect(probeRequest("?ext=probe&base=http%3A%2F%2Fljext.hello.localhost")).toEqual({ base: "http://ljext.hello.localhost" });
    for (const search of [
      "?ext=probe&base=https%3A%2F%2Fexample.com",
      "?ext=probe&base=ljext%3A%2F%2Fhello.localhost%2Fx",
      "?ext=probe",
      "?base=ljext%3A%2F%2Fhello.localhost",
      "",
    ]) {
      expect(probeRequest(search)).toBeUndefined();
    }
  });

  it("calls the API twice — as the SDK does, and as a simple request — and is blocked only if both fail", async () => {
    const seen: { url: string; init: RequestInit | undefined }[] = [];
    const refuse = (url: string, init?: RequestInit) => {
      seen.push({ url, init });
      return Promise.reject(new TypeError("refused"));
    };
    expect(await probeApi(refuse, "ljext://hello.localhost")).toBe("blocked");
    expect(seen.map((s) => s.url)).toEqual([
      "ljext://hello.localhost/__lj/api/v1/storage.set",
      "ljext://hello.localhost/__lj/api/v1/storage.set",
    ]);
    expect(seen[0]?.init?.headers).toMatchObject({ "X-LJ-Api": "1" });
    expect(seen[1]?.init?.headers).toEqual({ "Content-Type": "text/plain" });
    expect(await probeApi(() => Promise.resolve({ ok: false }), "ljext://hello.localhost")).toBe("blocked");
    expect(await probeApi(() => Promise.resolve({ ok: true }), "ljext://hello.localhost")).toBe("open");
    let first = true;
    const secondOnly = () => {
      const ok = !first;
      first = false;
      return Promise.resolve({ ok });
    };
    expect(await probeApi(secondOnly, "ljext://hello.localhost")).toBe("open");
  });

  it("reports whether an extension's file showed", async () => {
    expect(await probeFile(() => Promise.resolve(false), "http://ljext.hello.localhost")).toBe("blocked");
    expect(await probeFile((src) => Promise.resolve(src === "http://ljext.hello.localhost/icon.svg"), "http://ljext.hello.localhost")).toBe("open");
    expect(probeTitle({ api: "blocked", file: "blocked", nav: "refused" })).toBe(
      "LJ-EXTPROBE ljext-api=blocked ljext-file=blocked ljext-nav=refused",
    );
  });
});
