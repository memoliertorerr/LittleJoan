import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { LINK_QUERY, REFUSED_LINKS, linkLoaded, linksFailures, linksSummary, type LinksRun } from "./smoke-links.mts";

const log = (lines: string[]) => parseLog(lines.map((line, i) => `${i + 1} ${line}`).join("\n") + "\n");
const PAGE = `http://127.0.0.1:8123/smoke.html${LINK_QUERY}`;

function run(over: Partial<LinksRun> = {}): LinksRun {
  return {
    registration: { failures: [], defaults: [], facts: ["UserChoice https = MSEdgeHTM"] },
    launched: '"C:\\app.exe" --link "…" → code 0',
    afterLaunch: log(["link-opened", `page-started ${PAGE}`, `page-loaded ${PAGE}`, "title LJ-SMOKE ready h264=mse"]),
    afterRefusals: log(["link-refused invalid", "link-refused file", "link-refused javascript"]),
    aliveAfterRefusals: true,
    macExtras: [],
    events: [],
    ...over,
  };
}

describe("the links run", () => {
  it("passes a registered, untouched, working, refusing app", () => {
    expect(linksFailures(run())).toEqual([]);
    expect(linksSummary(run()).afterRefusals).toEqual(["link-refused invalid", "link-refused file", "link-refused javascript"]);
  });

  it("refuses exactly the three bad links, in order", () => {
    expect(REFUSED_LINKS.map(([text]) => text)).toEqual(["--quit", "file:///etc/passwd", "javascript:alert(1)"]);
    const two = run({ afterRefusals: log(["link-refused invalid", "link-refused javascript"]) });
    expect(linksFailures(two)[0]).toContain("[invalid, javascript]");
  });

  it("fails when a refused link did anything", () => {
    for (const event of ["quit", "link-opened", "page-started file:///etc/passwd", "window-created", "settings-window general", "hibernated"]) {
      const caused = run({ afterRefusals: log(["link-refused invalid", event, "link-refused file", "link-refused javascript"]) });
      expect(linksFailures(caused).join("\n"), event).toContain("a refused link caused");
    }
    expect(linksFailures(run({ aliveAfterRefusals: false })).join("\n")).toContain("--link --quit must not quit");
  });

  it("needs the OS's launch to open the smoke page", () => {
    expect(linkLoaded(run().afterLaunch)).toBe(true);
    expect(linkLoaded(log([`page-loaded ${PAGE}`, "title LJ-SMOKE ready"]))).toBe(false);
    expect(linkLoaded(log(["link-opened", "page-loaded http://127.0.0.1:8123/smoke.html", "title LJ-SMOKE ready"]))).toBe(false);
    expect(linkLoaded(log(["link-opened", `page-loaded ${PAGE}`, "title LJ-SMOKE loading"]))).toBe(false);
    expect(linksFailures(run({ afterLaunch: [] }))[0]).toContain("didn't log link-opened");
  });

  it("keeps the registration's and the defaults' findings", () => {
    const bad = run({ registration: { failures: ["links: x"], defaults: ["links: y"], facts: [] } });
    expect(linksFailures(bad)).toEqual(["links: x", "links: y"]);
  });

  it("never lets a file URL open on macOS", () => {
    expect(linksFailures(run({ macExtras: log(["link-refused file"]) }))).toEqual([]);
    expect(linksFailures(run({ macExtras: log(["page-started file:///etc/passwd"]) }))[0]).toContain("file URL");
  });
});
