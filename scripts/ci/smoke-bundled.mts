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
  /**
   * What its storage holds before the app starts (`extension-data/<id>.json`'s items), made at
   * `now` (unix ms) — sample content for the screenshot, and a check that it reads storage.
   */
  seed?: (now: number) => Record<string, unknown>;
  /** A page title the panel must show (its script read the seed and acted on it). */
  title?: RegExp;
}

/**
 * A page action the run opens from the command bar while the smoke page is shown (Ctrl/Cmd+L,
 * typing `typed`, ↓ ↓ Enter — required where keys can be driven, best effort on macOS); its
 * panel must get the page (`ext-api <id> pageInfo ok`).
 */
export interface PageActionPanel {
  id: string;
  shot: string;
  typed: string;
}

/** The page actions opened on the smoke page. */
export const PAGE_ACTIONS: PageActionPanel[] = [{ id: "qr", shot: "qr.png", typed: "qr" }];

const MINUTE = 60_000;

/** The bundled extensions opened with `--extension <id>`, in order. */
export const BUNDLED_PANELS: BundledPanel[] = [
  { id: "snake", shot: "snake.png" },
  { id: "2048", shot: "2048.png", keys: ["Left", "Up", "Right", "Down"] },
  {
    id: "notes",
    shot: "notes.png",
    seed: (now) => ({
      notes: {
        version: 1,
        notes: [
          { id: "n1", text: "Groceries\nmilk, eggs, coffee beans\nbread", created: now - 90 * MINUTE, updated: now - 4 * MINUTE },
          { id: "n2", text: "Ideas for the weekend\nhike up the hill\nthe new museum", created: now - 26 * 60 * MINUTE, updated: now - 25 * 60 * MINUTE },
          { id: "n3", text: "Little Joan\nThe command bar opens on every summon; Esc goes back to the page.", created: now - 9 * 24 * 60 * MINUTE, updated: now - 9 * 24 * 60 * MINUTE },
        ],
      },
    }),
  },
  {
    id: "focus",
    shot: "focus.png",
    // A focus session that started 7½ minutes before the app did: the panel must show it running
    // from its stored end time (the window title counts down).
    seed: (now) => ({ timer: { phase: "focus", round: 2, endsAt: now + 17.5 * MINUTE, remaining: 25 * MINUTE, completed: 1 } }),
    title: /^1[0-7]:\d\d · Focus$/,
  },
  { id: "clock", shot: "clock.png", seed: () => ({ cities: ["london", "new-york", "tokyo", "sydney"] }) },
  {
    id: "json",
    shot: "json.png",
    seed: () => ({
      draft: JSON.stringify({ name: "Little Joan", version: "0.2.0", tiny: true, window: { width: 480, height: 640 }, extensions: ["snake", "2048", "notes"], theme: null }),
      style: "two",
    }),
  },
  { id: "text-tools", shot: "text-tools.png", seed: () => ({ draft: "The quick brown fox jumps over the lazy dog.\nLittle Joan hibernates until you call it." }) },
  { id: "color", shot: "color.png", seed: () => ({ colors: { color: "#e0675a", against: "#ffffff" } }) },
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

/** What went wrong with an opened panel: it came up, the toolbar named it, none of its requests or calls failed. */
function panelFailures(events: LogEvent[], id: string): string[] {
  const own = stretch(events, id) ?? [];
  const failures: string[] = [];
  if (!panelUp(events, id)) {
    failures.push(`${id}'s panel never came up (its page and its theme call)`);
  }
  if (!own.some((e) => e.event === "toolbar-label" && e.detail === id)) {
    failures.push(`the toolbar never named ${id}`);
  }
  for (const refused of own.filter((e) => e.event === "ljext-refused" && isPanelOf(e.detail.split(" ").at(-1) ?? "", id))) {
    if (!refused.detail.includes("favicon.ico")) {
      failures.push(`a request of ${id}'s own panel was refused: ${refused.detail}`);
    }
  }
  for (const call of own.filter((e) => e.event === "ext-api" && e.detail.startsWith(`${id} `) && !e.detail.endsWith(" ok"))) {
    failures.push(`${id}'s API call failed: ${call.detail}`);
  }
  return failures;
}

/** What went wrong with each bundled panel the run opened (empty when all is well). */
export function bundledFailures(events: LogEvent[], panels: readonly BundledPanel[]): string[] {
  const failures: string[] = [];
  for (const missing of events.filter((e) => e.event === "extension-missing" || e.event === "extension-skipped")) {
    failures.push(`bundled: ${missing.event} ${missing.detail}`);
  }
  for (const { id } of panels) {
    if (!stretch(events, id)) {
      failures.push(`bundled: --extension ${id} never opened it`);
      continue;
    }
    failures.push(...panelFailures(events, id).map((failure) => `bundled: ${failure}`));
  }
  for (const { id, title } of panels) {
    const own = stretch(events, id) ?? [];
    if (title && !own.some((e) => e.event === "title" && title.test(e.detail))) {
      const seen = own.filter((e) => e.event === "title").map((e) => e.detail);
      failures.push(`bundled: ${id}'s panel never showed a title like ${String(title)} (${JSON.stringify(seen.slice(-3))})`);
    }
  }
  return failures;
}

/**
 * What went wrong with a page action opened from the command bar on the smoke page: it must have
 * come up like any panel, and have been handed the page. Where keys can't be driven (macOS) an
 * opening that never happened isn't a failure.
 */
export function pageActionFailures(events: LogEvent[], actions: readonly PageActionPanel[], options: { driven: boolean }): string[] {
  const failures: string[] = [];
  for (const { id } of actions) {
    const own = stretch(events, id);
    if (!own) {
      if (options.driven) {
        failures.push(`page action: picking ${id} in the command bar never opened it`);
      }
      continue;
    }
    failures.push(...panelFailures(events, id).map((failure) => `page action: ${failure}`));
    if (!own.some((e) => e.event === "ext-api" && e.detail === `${id} pageInfo ok`)) {
      failures.push(`page action: ${id} wasn't handed the page (no pageInfo call answered)`);
    }
  }
  return failures;
}

/** The storage file a seed becomes (`extension-data/<id>.json`, as Little Joan writes it). */
export function seedFile(items: Record<string, unknown>): string {
  return `${JSON.stringify({ version: 1, items })}\n`;
}

/** Whether keys reached `id`'s panel: a storage write of its own after its theme call (2048 saves each move). */
export function keysReached(events: LogEvent[], id: string): boolean {
  const own = stretch(events, id) ?? [];
  const up = own.findIndex((e) => e.event === "ext-api" && e.detail === `${id} theme ok`);
  return up >= 0 && own.slice(up).some((e) => e.event === "ext-api" && e.detail === `${id} storage.set ok`);
}
