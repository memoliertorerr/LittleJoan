import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { HELLO_TITLE, PROBE_TITLE, extensionFailures, helloOrigin, isExtUrl, parseStore, type HelloStore } from "./smoke-extensions.mts";

const PROBE_URL = "http://127.0.0.1:5000/smoke.html?ext=probe&base=ljext%3A%2F%2Fhello.localhost";

const GOOD = `1 started
2 window-created
3 extension-opened hello
4 page-loaded ljext://hello.localhost/index.html
5 ext-api hello storage.get ok
6 ext-api hello storage.set ok
7 ext-api hello theme ok
8 title ${HELLO_TITLE}
9 page-loaded ${PROBE_URL}
10 ljext-refused page joan-page ljext://hello.localhost/icon.svg
11 navigation-refused ljext://hello.localhost/index.html
12 title ${PROBE_TITLE}
13 quit
`;

const STORE: HelloStore = { opened: 1, keys: ["opened"] };

function without(text: string, line: string): string {
  return text
    .split("\n")
    .filter((l) => !l.includes(line))
    .join("\n");
}

describe("the extension run", () => {
  it("passes when hello works and the probe is refused everything", () => {
    expect(extensionFailures(parseLog(GOOD), STORE)).toEqual([]);
  });

  it("fails when hello didn't open, load or report", () => {
    expect(extensionFailures(parseLog(without(GOOD, "extension-opened")), STORE).join()).toMatch(/never opened/);
    expect(extensionFailures(parseLog(without(GOOD, "4 page-loaded")), STORE).join()).toMatch(/never loaded in the page/);
    const wrong = GOOD.replace(HELLO_TITLE, "LJ-EXT hello storage=ok theme=ok ipc=open net=blocked");
    expect(extensionFailures(parseLog(wrong), STORE).join()).toMatch(/ipc=open/);
    expect(extensionFailures(parseLog(without(GOOD, "theme ok")), STORE).join()).toMatch(/theme call never succeeded/);
    const skipped = GOOD.replace("3 extension", "2 extension-skipped hello `index.html` is missing\n3 extension");
    expect(extensionFailures(parseLog(skipped), STORE).join()).toMatch(/was skipped/);
  });

  it("fails when the probe reached hello in any way", () => {
    const called = GOOD.replace("10 ljext-refused", "10 ext-api hello storage.set ok\n10 ljext-refused");
    expect(extensionFailures(parseLog(called), STORE).join()).toMatch(/succeeded after the probe/);
    const navigated = GOOD.replace("11 navigation-refused ljext://hello.localhost/index.html", "11 page-loaded http://ljext.hello.localhost/index.html");
    const failures = extensionFailures(parseLog(navigated), STORE).join();
    expect(failures).toMatch(/took the page webview to an extension/);
    expect(failures).toMatch(/never refused/);
    const open = GOOD.replace(PROBE_TITLE, "LJ-EXTPROBE ljext-api=open ljext-file=blocked ljext-nav=refused");
    expect(extensionFailures(parseLog(open), STORE).join()).toMatch(/ljext-api=open/);
    expect(extensionFailures(parseLog(GOOD), { opened: 1, keys: ["opened", "smoke-probe"] }).join()).toMatch(/wrote into/);
    expect(extensionFailures(parseLog(without(GOOD, "ext=probe")), STORE).join()).toMatch(/probe never loaded/);
  });

  it("fails without hello's stored count", () => {
    expect(extensionFailures(parseLog(GOOD), undefined).join()).toMatch(/storage file is missing/);
    expect(extensionFailures(parseLog(GOOD), { opened: "x", keys: ["opened"] }).join()).toMatch(/no count/);
  });

  it("knows both forms of extension URLs and reads the store", () => {
    expect(isExtUrl("ljext://hello.localhost/index.html")).toBe(true);
    expect(isExtUrl("http://ljext.hello.localhost/index.html")).toBe(true);
    expect(isExtUrl("HTTPS://LJEXT.hello.localhost/")).toBe(true);
    expect(isExtUrl(PROBE_URL)).toBe(false);
    expect(helloOrigin("win32")).toBe("http://ljext.hello.localhost");
    expect(helloOrigin("darwin")).toBe("ljext://hello.localhost");
    expect(parseStore('{"version":1,"items":{"opened":2}}')).toEqual({ opened: 2, keys: ["opened"] });
    expect(parseStore("nope")).toBeUndefined();
    expect(parseStore(undefined)).toBeUndefined();
  });
});
