import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import {
  HELLO_TITLE,
  PROBE_TITLE,
  commandHelloFailures,
  extensionFailures,
  helloOrigin,
  labelFailures,
  raceFailures,
  servedToOthers,
  isExtUrl,
  parseStore,
  settingsExtensionsFailures,
  type HelloStore,
} from "./smoke-extensions.mts";

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

  it("fails when --extension hello's own panel was refused (the Windows race CI saw)", () => {
    const raced = GOOD.replace(
      "4 page-loaded ljext://hello.localhost/index.html",
      "4 ljext-refused page joan-page ljext://hello.localhost/index.html\n4 page-loaded ljext://hello.localhost/index.html",
    );
    expect(extensionFailures(parseLog(raced), STORE).join()).toMatch(/was refused/);
    // Its title only after a second open (from the command bar) doesn't count for the first.
    const late = GOOD.replace(`8 title ${HELLO_TITLE}`, `8 extension-opened hello\n8 title ${HELLO_TITLE}`);
    expect(extensionFailures(parseLog(late), STORE).join()).toMatch(/own panel never reported/);
    // WebView2's own favicon request isn't hello's.
    const favicon = GOOD.replace("4 page-loaded", "4 ljext-refused not-found joan-page ljext://hello.localhost/favicon.ico\n4 page-loaded");
    expect(extensionFailures(parseLog(favicon), STORE)).toEqual([]);
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

describe("Settings → Extensions in the developer run", () => {
  // As CI logs it: the window lists the extensions as it loads (on General), before the switch.
  const GOOD_SETTINGS = `1 extensions-list 1 skipped=0
2 settings-window general
3 extension-skipped broken \`apiVersion\` must be 1
4 extensions-list 2 skipped=1
5 settings-window privacy
6 settings-window appearance
7 settings-window extensions
`;

  it("passes when the tab listed hello and the developer copy, the broken one skipped", () => {
    expect(settingsExtensionsFailures(parseLog(GOOD_SETTINGS))).toEqual([]);
  });

  it("fails without the tab, the right counts or the skipped reason", () => {
    expect(settingsExtensionsFailures(parseLog("1 settings-window appearance\n")).join()).toMatch(/never switched/);
    // The command bar's list before Settings opened doesn't count.
    const beforeSettings = "1 extensions-list 2 skipped=1\n2 settings-window general\n3 settings-window extensions\n";
    expect(settingsExtensionsFailures(parseLog(beforeSettings)).join()).toMatch(/didn't list/);
    expect(settingsExtensionsFailures(parseLog(GOOD_SETTINGS.replace("2 skipped=1", "1 skipped=0"))).join()).toMatch(/didn't list/);
    expect(settingsExtensionsFailures(parseLog(GOOD_SETTINGS.replace("3 extension-skipped broken", "3 extension-skipped other"))).join()).toMatch(
      /wasn't reported/,
    );
  });
});

describe("the command bar's extension rows", () => {
  it("want hello opened twice where keys are driven", () => {
    const once = parseLog("1 extension-opened hello\n2 extensions-list 1 skipped=0\n");
    const twice = parseLog("1 extension-opened hello\n2 extensions-list 1 skipped=0\n3 extension-opened hello\n");
    expect(commandHelloFailures(twice, { driven: true })).toEqual([]);
    expect(commandHelloFailures(once, { driven: true }).join()).toMatch(/didn't open hello/);
    expect(commandHelloFailures(once, { driven: false })).toEqual([]);
    expect(commandHelloFailures(parseLog("1 extension-opened hello\n"), { driven: false }).join()).toMatch(/never asked/);
  });
});

describe("the race and the toolbar", () => {
  const N = "0123456789abcdef0123456789abcdef";
  const RACE_URL = "http://127.0.0.1:5000/smoke.html?ext=race&base=ljext%3A%2F%2Fhello.localhost";
  const GOOD_RACE = `1 page-started ljext://hello.localhost/index.html?lj-open=${N}
2 toolbar-label hello
3 page-loaded ljext://hello.localhost/index.html?lj-open=${N}
4 page-started ${RACE_URL}
5 toolbar-label none
6 page-loaded ${RACE_URL}
7 title LJ-EXTRACE running
8 navigation-refused ljext://hello.localhost/index.html
9 extension-opened hello
10 navigation-refused ljext://hello.localhost/index.html
10 ljext-headers joan-page GET ljext://hello.localhost/index.html status=403 referer=other names=accept,referer dest=- site=- mode=- origin=- service-worker=-
10 ljext-headers joan-page GET ljext://hello.localhost/index.html status=200 referer=- names=accept dest=- site=- mode=- origin=- service-worker=-
11 page-started ljext://hello.localhost/index.html?lj-open=${N.replace("0", "f")}
12 toolbar-label hello
13 title ${HELLO_TITLE}
`;

  it("passes when the app's panel wins and every racer is refused", () => {
    expect(raceFailures(parseLog(GOOD_RACE))).toEqual([]);
  });

  it("fails when a racer's URL loaded, nothing was refused, or the app's panel didn't open", () => {
    const loaded = GOOD_RACE.replace("10 navigation-refused ljext://hello.localhost/index.html", "10 page-started ljext://hello.localhost/index.html");
    expect(raceFailures(parseLog(loaded)).join()).toMatch(/without the app's nonce loaded/);
    const quiet = GOOD_RACE.replaceAll(/\d+ navigation-refused .*\n/g, "");
    expect(raceFailures(parseLog(quiet)).join()).toMatch(/none of the page's iframe attempts/);
    const lost = GOOD_RACE.replace(`13 title ${HELLO_TITLE}`, "13 title LJ-EXTRACE done opened=0");
    expect(raceFailures(parseLog(lost)).join()).toMatch(/didn't open the panel/);
    expect(raceFailures(parseLog("1 started\n"))).toEqual(["race: the smoke page's race never loaded"]);
  });

  it("fails when the handler served a request from another page", () => {
    const served = GOOD_RACE.replace("status=403 referer=other", "status=200 referer=other");
    expect(raceFailures(parseLog(served)).join()).toMatch(/served to another page: joan-page GET ljext:\/\/hello.localhost\/index.html status=200/);
    expect(servedToOthers(parseLog(GOOD_RACE))).toEqual([]);
    const own = "1 ljext-headers joan-page GET ljext://hello.localhost/hello.css status=200 referer=own names=accept,referer\n";
    expect(servedToOthers(parseLog(own))).toEqual([]);
  });

  it("never lets the toolbar name an extension over a web page", () => {
    const spoofed = GOOD_RACE.replace("8 navigation-refused", "8 toolbar-label hello\n8 navigation-refused");
    expect(labelFailures(parseLog(spoofed)).join()).toMatch(/named hello while the page was/);
    const otherName = GOOD_RACE.replace("2 toolbar-label hello", "2 toolbar-label other");
    expect(labelFailures(parseLog(otherName)).join()).toMatch(/named other/);
    expect(labelFailures(parseLog("1 toolbar-label none\n")).join()).toMatch(/never named hello/);
    const windows = `1 page-started http://ljext.hello.localhost/index.html?lj-open=${N}\n2 toolbar-label hello\n`;
    expect(labelFailures(parseLog(windows))).toEqual([]);
  });
});
