// The smoke test's zoom check (LJ16) and its rules — part of the main run, right after the wide
// (720×600) viewport: Ctrl+= (⌘= on macOS) zooms the page to 110 %, so the page's next viewport
// report is narrower than 720 px (about 655), and Ctrl+0 (⌘0) brings 720 back. The keys go
// through the keymap like every page key (WebView2's AcceleratorKeyPressed, the GTK key hook, the
// macOS View menu). Required on Windows and Linux; on macOS only when the synthetic key reached
// the app (its `zoom` line), the same rule as the fullscreen click. Pure and unit-tested
// (smoke-zoom.test.ts).
import type { LogEvent, Viewport } from "./smoke-log.mts";

/** The level one Ctrl+= gives from 100 %. */
export const ZOOMED = 110;

/** A viewport report in a title: "LJ-SMOKE viewport=654x509 ua=desktop dpr=1.1 …". */
export function viewportIn(title: string): { width: number; height: number; ua: string } | undefined {
  const match = /^LJ-SMOKE viewport=(\d+)x(\d+) ua=(\w+)/.exec(title);
  return match ? { width: Number(match[1]), height: Number(match[2]), ua: match[3] ?? "" } : undefined;
}

function viewports(events: LogEvent[]): { width: number; height: number; ua: string }[] {
  return events.flatMap((e) => {
    const report = e.event === "title" ? viewportIn(e.detail) : undefined;
    return report ? [report] : [];
  });
}

/** The page's first viewport after the `zoom 110` line, if it is narrower than `wide`. */
export function zoomedViewport(events: LogEvent[], wide: Viewport): { width: number; height: number } | undefined {
  const at = events.findIndex((e) => e.event === "zoom" && e.detail === String(ZOOMED));
  if (at < 0) {
    return undefined;
  }
  return viewports(events.slice(at + 1)).find((v) => v.width > 0 && v.width < wide.width && v.ua === wide.ua);
}

/** After the `zoom 100` that follows 110 %, the page reported `wide` again. */
export function zoomReset(events: LogEvent[], wide: Viewport): boolean {
  const zoomed = events.findIndex((e) => e.event === "zoom" && e.detail === String(ZOOMED));
  const reset = events.findIndex((e, i) => i > zoomed && zoomed >= 0 && e.event === "zoom" && e.detail === "100");
  return reset >= 0 && viewports(events.slice(reset + 1)).some((v) => v.width === wide.width && v.height === wide.height && v.ua === wide.ua);
}

/**
 * The rules, over the main log from the zoom check on: `zoom 110`, then a narrower viewport;
 * `zoom 100`, then the wide one again. Where the keys aren't driven (macOS), only when a `zoom`
 * line shows they arrived.
 */
export function zoomFailures(events: LogEvent[], wide: Viewport, options: { driven: boolean }): string[] {
  const arrived = events.some((e) => e.event === "zoom");
  if (!options.driven && !arrived) {
    return [];
  }
  const failures: string[] = [];
  if (!events.some((e) => e.event === "zoom" && e.detail === String(ZOOMED))) {
    failures.push(`zoom: Ctrl/Cmd+= didn't zoom the page to ${ZOOMED} % (no "zoom ${ZOOMED}")`);
  } else if (!zoomedViewport(events, wide)) {
    failures.push(`zoom: after "zoom ${ZOOMED}" the page never reported a viewport narrower than ${wide.width} px`);
  }
  if (!zoomReset(events, wide)) {
    failures.push(`zoom: Ctrl/Cmd+0 didn't bring the page back to ${wide.width}x${wide.height}`);
  }
  return failures;
}

/** What summary.json records. */
export function zoomSummary(events: LogEvent[], wide: Viewport, driven: boolean): Record<string, unknown> {
  const zoomed = zoomedViewport(events, wide);
  return {
    driven,
    levels: events.filter((e) => e.event === "zoom").map((e) => Number(e.detail)),
    zoomedViewport: zoomed ? `${zoomed.width}x${zoomed.height}` : null,
    reset: zoomReset(events, wide),
  };
}
