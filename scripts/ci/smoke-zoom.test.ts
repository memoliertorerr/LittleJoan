import { describe, expect, it } from "vitest";
import { parseLog, type Viewport } from "./smoke-log.mts";
import { viewportIn, zoomFailures, zoomReset, zoomSummary, zoomedViewport } from "./smoke-zoom.mts";

const log = (rows: string[]) => parseLog(rows.map((row, i) => `${1000 + i} ${row}`).join("\n") + "\n");
const WIDE: Viewport = { width: 720, height: 560, ua: "desktop" };

const GOOD = [
  "zoom 110",
  "title LJ-SMOKE viewport=655x509 ua=desktop dpr=1.1 uamobile=false",
  "zoom 100",
  "title LJ-SMOKE viewport=720x560 ua=desktop dpr=1 uamobile=false",
];

describe("the zoom check's rules (LJ16)", () => {
  it("reads the page's viewport reports", () => {
    expect(viewportIn("LJ-SMOKE viewport=655x509 ua=desktop dpr=1.1 uamobile=false")).toEqual({ width: 655, height: 509, ua: "desktop" });
    expect(viewportIn("LJ-SMOKE ready h264=mse")).toBeUndefined();
  });

  it("passes 110 % narrowing the page and Ctrl+0 bringing 720 back", () => {
    const events = log(GOOD);
    expect(zoomFailures(events, WIDE, { driven: true })).toEqual([]);
    expect(zoomedViewport(events, WIDE)).toEqual({ width: 655, height: 509, ua: "desktop" });
    expect(zoomReset(events, WIDE)).toBe(true);
    expect(zoomSummary(events, WIDE, true)).toEqual({ driven: true, levels: [110, 100], zoomedViewport: "655x509", reset: true });
  });

  it("needs the zoom, the narrower page, and the reset", () => {
    expect(zoomFailures(log([]), WIDE, { driven: true })).toEqual([
      'zoom: Ctrl/Cmd+= didn\'t zoom the page to 110 % (no "zoom 110")',
      "zoom: Ctrl/Cmd+0 didn't bring the page back to 720x560",
    ]);
    const unchanged = log(["zoom 110", "title LJ-SMOKE viewport=720x560 ua=desktop dpr=1 uamobile=false", "zoom 100"]);
    expect(zoomFailures(unchanged, WIDE, { driven: true })).toEqual([
      'zoom: after "zoom 110" the page never reported a viewport narrower than 720 px',
      "zoom: Ctrl/Cmd+0 didn't bring the page back to 720x560",
    ]);
    // A report from before the zoom doesn't count, nor one before the reset.
    const early = log(["title LJ-SMOKE viewport=655x509 ua=desktop dpr=1", "zoom 110", "title LJ-SMOKE viewport=720x560 ua=desktop dpr=1", "zoom 100"]);
    expect(zoomFailures(early, WIDE, { driven: true })).toHaveLength(2);
    const noReset = log(GOOD.slice(0, 2));
    expect(zoomFailures(noReset, WIDE, { driven: true })).toEqual(["zoom: Ctrl/Cmd+0 didn't bring the page back to 720x560"]);
    // The phone agent would mean the window, not the page, got narrower.
    const phone = log(["zoom 110", "title LJ-SMOKE viewport=390x560 ua=phone dpr=1", ...GOOD.slice(2)]);
    expect(zoomFailures(phone, WIDE, { driven: true })[0]).toContain("never reported a viewport narrower");
  });

  it("is informational on macOS until a key arrives", () => {
    expect(zoomFailures(log([]), WIDE, { driven: false })).toEqual([]);
    expect(zoomFailures(log(["zoom 110"]), WIDE, { driven: false })).toHaveLength(2);
    expect(zoomFailures(log(GOOD), WIDE, { driven: false })).toEqual([]);
  });
});
