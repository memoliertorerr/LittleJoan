// The smoke test's run of the bundled extensions (pure rules, unit-tested; scripts/ci/smoke.mts
// drives it): a launch of its own opens each bundled extension with a command through
// `--extension <id>` and screenshots its panel. A panel counts as up when the app opened it, its
// URL (with the app's nonce) started loading, its own script asked for the theme (`ext-api <id>
// theme ok` — so its scripts ran), the toolbar named it, and none of its own requests were
// refused (the engine's favicon request aside).
import type { LogEvent } from "./smoke-log.mts";

/** A bundled extension the run opens, and the screenshot of its panel. */
export interface BundledPanel {
  id: string;
  shot: string;
  /** Keys pressed once it is up (Windows, Linux; best effort on macOS), for a livelier shot. */
  keys?: string[];
}

/** The bundled extensions opened with `--extension <id>`, in order. */
export const BUNDLED_PANELS: BundledPanel[] = [
  { id: "snake", shot: "snake.png" },
  { id: "2048", shot: "2048.png", keys: ["Left", "Up", "Right", "Down"] },
];

/** An extension URL in either form (`ljext://<id>.localhost/…`, WebView2's `http://ljext.<id>.localhost/…`). */
export function isPanelOf(url: string, id: string): boolean {
  return url.startsWith(`ljext://${id}.localhost/`) || url.startsWith(`http://ljext.${id}.localhost/`);
}

/** The events from `extension-opened <id>` (the last one) up to the next extension's opening. */
function stretch(events: LogEvent[], id: string): LogEvent[] | undefined {
  const at = events.findLastIndex((e) => e.event === "extension-opened" && e.detail === id);
  if (at < 0) {
    return undefined;
  }
  const next = events.findIndex((e, i) => i > at && e.event === "extension-opened");
  return events.slice(at, next < 0 ? undefined : next);
}

/** Whether `id`'s panel is up: opened, loading its own URL, its script asked for the theme. */
export function panelUp(events: LogEvent[], id: string): boolean {
  const own = stretch(events, id);
  return (
    own !== undefined &&
    own.some((e) => e.event === "page-started" && isPanelOf(e.detail, id) && e.detail.includes("lj-open=")) &&
    own.some((e) => e.event === "ext-api" && e.detail === `${id} theme ok`)
  );
}

/** What went wrong with each bundled panel the run opened (empty when all is well). */
export function bundledFailures(events: LogEvent[], panels: readonly BundledPanel[]): string[] {
  const failures: string[] = [];
  for (const missing of events.filter((e) => e.event === "extension-missing" || e.event === "extension-skipped")) {
    failures.push(`bundled: ${missing.event} ${missing.detail}`);
  }
  for (const { id } of panels) {
    const own = stretch(events, id);
    if (!own) {
      failures.push(`bundled: --extension ${id} never opened it`);
      continue;
    }
    if (!panelUp(events, id)) {
      failures.push(`bundled: ${id}'s panel never came up (its page and its theme call)`);
    }
    if (!own.some((e) => e.event === "toolbar-label" && e.detail === id)) {
      failures.push(`bundled: the toolbar never named ${id}`);
    }
    for (const refused of own.filter((e) => e.event === "ljext-refused" && isPanelOf(e.detail.split(" ").at(-1) ?? "", id))) {
      if (!refused.detail.includes("favicon.ico")) {
        failures.push(`bundled: a request of ${id}'s own panel was refused: ${refused.detail}`);
      }
    }
    for (const call of own.filter((e) => e.event === "ext-api" && e.detail.startsWith(`${id} `) && !e.detail.endsWith(" ok"))) {
      failures.push(`bundled: ${id}'s API call failed: ${call.detail}`);
    }
  }
  return failures;
}

/** Whether keys reached `id`'s panel: a storage write of its own after its theme call (2048 saves each move). */
export function keysReached(events: LogEvent[], id: string): boolean {
  const own = stretch(events, id) ?? [];
  const up = own.findIndex((e) => e.event === "ext-api" && e.detail === `${id} theme ok`);
  return up >= 0 && own.slice(up).some((e) => e.event === "ext-api" && e.detail === `${id} storage.set ok`);
}
