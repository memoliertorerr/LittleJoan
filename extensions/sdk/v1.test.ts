// The SDK (extensions/sdk/v1.js) run as a panel would run it, in a context of its own with a
// stand-in for `fetch` and the panel's window: littleJoan.theme.onChange (LJ56) asks `theme` again
// on Little Joan's `littlejoan:themechange` event — which carries no data — and calls back.
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { runInNewContext } from "node:vm";
import { describe, expect, it } from "vitest";

const SDK = readFileSync(join(import.meta.dirname, "v1.js"), "utf8");
const EVENT = "littlejoan:themechange";

interface Theme {
  scheme: "light" | "dark";
  vars: Record<string, string>;
}

interface Sdk {
  theme: {
    get(): Promise<Theme>;
    onChange(callback: (theme: Theme) => unknown): () => void;
  };
}

const LIGHT: Theme = { scheme: "light", vars: { "--lj-bg-0": "#dae1ee" } };
const DARK: Theme = { scheme: "dark", vars: { "--lj-bg-0": "#1f2430" } };

/** A call the stand-in `fetch` holds until the test answers it. */
interface Pending {
  url: string;
  init: { method?: string; headers?: Record<string, string> };
  answer(value: unknown, ok?: boolean): void;
}

/** The SDK in a fresh context: its `littleJoan`, the window it listens on, its calls. */
function load() {
  const window = new EventTarget();
  const calls: Pending[] = [];
  const reported: unknown[] = [];
  const fetch = (url: string, init: Pending["init"]) =>
    new Promise((resolve) => {
      calls.push({
        url,
        init,
        answer: (value, ok = true) =>
          resolve({ ok, status: ok ? 200 : 403, json: () => Promise.resolve(ok ? { ok: true, value } : { ok: false, error: "permission", message: "No." }) }),
      });
    });
  // An error thrown in a callback is reported like one in an event listener (a timer's throw).
  const setTimeout = (run: () => void) => {
    try {
      run();
    } catch (error) {
      reported.push(error);
    }
  };
  const context: Record<string, unknown> = { window, fetch, setTimeout };
  runInNewContext(SDK, context);
  return { sdk: context.littleJoan as Sdk, window, calls, reported };
}

/** Lets the SDK's promises run. */
async function settle(): Promise<void> {
  for (let i = 0; i < 10; i += 1) {
    await new Promise((done) => setImmediate(done));
  }
}

describe("the SDK's theme.onChange", () => {
  it("asks for the theme again on Little Joan's event and calls back with it", async () => {
    const { sdk, window, calls } = load();
    const seen: Theme[] = [];
    sdk.theme.onChange((theme) => {
      seen.push(theme);
    });
    await settle();
    expect(calls).toEqual([]); // following asks nothing until a change
    window.dispatchEvent(new Event(EVENT));
    await settle();
    expect(calls.map((c) => c.url)).toEqual(["/__lj/api/v1/theme"]);
    expect(calls[0]?.init.method).toBe("POST");
    expect(calls[0]?.init.headers?.["X-LJ-Api"]).toBe("__LJ_SESSION_TOKEN__");
    calls[0]?.answer(DARK);
    await settle();
    expect(seen).toEqual([DARK]);
  });

  it("takes nothing from the event: the colors are always asked for", async () => {
    const { sdk, window, calls } = load();
    const seen: Theme[] = [];
    sdk.theme.onChange((theme) => {
      seen.push(theme);
    });
    window.dispatchEvent(new CustomEvent(EVENT, { detail: { scheme: "dark", vars: { "--lj-bg-0": "red" } } }));
    await settle();
    calls[0]?.answer(LIGHT);
    await settle();
    expect(seen).toEqual([LIGHT]);
    // Other events are ignored.
    window.dispatchEvent(new Event("themechange"));
    await settle();
    expect(calls).toHaveLength(1);
  });

  it("asks one at a time, and once more for changes that came meanwhile", async () => {
    const { sdk, window, calls } = load();
    const seen: Theme[] = [];
    sdk.theme.onChange((theme) => {
      seen.push(theme);
    });
    for (let i = 0; i < 5; i += 1) {
      window.dispatchEvent(new Event(EVENT));
    }
    await settle();
    expect(calls).toHaveLength(1);
    calls[0]?.answer(LIGHT);
    await settle();
    expect(calls).toHaveLength(2);
    calls[1]?.answer(DARK);
    await settle();
    expect(calls).toHaveLength(2);
    expect(seen).toEqual([LIGHT, DARK]);
  });

  it("waits for a callback's promise before asking again", async () => {
    const { sdk, window, calls } = load();
    let release: () => void = () => undefined;
    const seen: Theme[] = [];
    sdk.theme.onChange(async (theme) => {
      seen.push(theme);
      await new Promise<void>((done) => {
        release = done;
      });
    });
    window.dispatchEvent(new Event(EVENT));
    await settle();
    calls[0]?.answer(LIGHT);
    await settle();
    window.dispatchEvent(new Event(EVENT));
    await settle();
    expect(calls).toHaveLength(1);
    release();
    await settle();
    expect(calls).toHaveLength(2);
    calls[1]?.answer(DARK);
    await settle();
    expect(seen).toEqual([LIGHT, DARK]);
  });

  it("keeps following after a refused call or a callback that throws", async () => {
    const { sdk, window, calls, reported } = load();
    const seen: Theme[] = [];
    sdk.theme.onChange((theme) => {
      seen.push(theme);
      throw new Error("the panel's own bug");
    });
    window.dispatchEvent(new Event(EVENT));
    await settle();
    calls[0]?.answer(null, false);
    await settle();
    expect(seen).toEqual([]);
    window.dispatchEvent(new Event(EVENT));
    await settle();
    calls[1]?.answer(DARK);
    await settle();
    expect(seen).toEqual([DARK]);
    expect(reported).toHaveLength(1);
    window.dispatchEvent(new Event(EVENT));
    await settle();
    expect(calls).toHaveLength(3);
  });

  it("stops when asked", async () => {
    const { sdk, window, calls } = load();
    const stop = sdk.theme.onChange(() => undefined);
    stop();
    window.dispatchEvent(new Event(EVENT));
    await settle();
    expect(calls).toEqual([]);
  });

  it("leaves theme.get a snapshot", async () => {
    const { sdk, calls } = load();
    const got = sdk.theme.get();
    await settle();
    calls[0]?.answer(LIGHT);
    await expect(got).resolves.toEqual(LIGHT);
  });
});
