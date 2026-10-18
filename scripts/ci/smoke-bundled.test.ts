import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { BUNDLED_PANELS, bundledFailures, isPanelOf, keysReached, panelUp, seedFile } from "./smoke-bundled.mts";

const NONCE = "0123456789abcdef0123456789abcdef";

/** A panel's good stretch of the log, as CI writes it (macOS / Linux URLs, or Windows'). */
function opened(id: string, at: number, windows = false): string {
  const origin = windows ? `http://ljext.${id}.localhost` : `ljext://${id}.localhost`;
  return [
    `${at} extension-opened ${id}`,
    `${at + 1} page-started ${origin}/index.html?lj-open=${NONCE}`,
    `${at + 2} toolbar-label ${id}`,
    `${at + 3} ljext-refused page joan-page ${origin}/favicon.ico`,
    `${at + 4} ext-api ${id} theme ok`,
    `${at + 5} ext-api ${id} storage.get ok`,
  ].join("\n");
}

const PANELS = [
  { id: "snake", shot: "snake.png" },
  { id: "2048", shot: "2048.png" },
];

describe("the bundled extensions run", () => {
  it("knows each panel's URL in both forms", () => {
    expect(isPanelOf(`ljext://snake.localhost/index.html?lj-open=${NONCE}`, "snake")).toBe(true);
    expect(isPanelOf("http://ljext.2048.localhost/ui.js", "2048")).toBe(true);
    expect(isPanelOf("ljext://snake.localhost.evil/", "snake")).toBe(false);
    expect(isPanelOf("ljext://hello.localhost/index.html", "snake")).toBe(false);
    expect(BUNDLED_PANELS.map((p) => p.id)).toEqual(expect.arrayContaining(["snake", "2048"]));
  });

  it("passes when every panel came up on its own", () => {
    for (const windows of [false, true]) {
      const log = parseLog(`1 started\n${opened("snake", 10, windows)}\n${opened("2048", 20, windows)}\n`);
      expect(bundledFailures(log, PANELS)).toEqual([]);
      expect(panelUp(log, "snake")).toBe(true);
    }
  });

  it("fails for a panel that never opened, never ran its script, or wasn't named", () => {
    const missing = parseLog(`${opened("snake", 10)}\n`);
    expect(bundledFailures(missing, PANELS).join()).toMatch(/--extension 2048 never opened/);
    const noScript = parseLog(`${opened("snake", 10).replace("ext-api snake theme ok", "ext-api other theme ok")}\n${opened("2048", 20)}`);
    expect(panelUp(noScript, "snake")).toBe(false);
    expect(bundledFailures(noScript, PANELS).join()).toMatch(/snake's panel never came up/);
    const noNonce = parseLog(`${opened("snake", 10).replace(`?lj-open=${NONCE}`, "")}\n${opened("2048", 20)}`);
    expect(bundledFailures(noNonce, PANELS).join()).toMatch(/snake's panel never came up/);
    const unnamed = parseLog(`${opened("snake", 10).replace("toolbar-label snake", "toolbar-label none")}\n${opened("2048", 20)}`);
    expect(bundledFailures(unnamed, PANELS).join()).toMatch(/never named snake/);
    // Another extension's theme call doesn't count for this one.
    const late = parseLog(`${opened("snake", 10).replace("ext-api snake theme ok", "")}\n${opened("2048", 20)}\n30 ext-api snake theme ok`);
    expect(panelUp(late, "snake")).toBe(false);
  });

  it("fails when the panel's own requests are refused or its calls fail", () => {
    const refused = parseLog(`${opened("snake", 10)}\n16 ljext-refused page joan-page ljext://snake.localhost/ui.js\n${opened("2048", 20)}`);
    expect(bundledFailures(refused, PANELS).join()).toMatch(/snake's own panel was refused: page joan-page ljext:\/\/snake.localhost\/ui.js/);
    const failed = parseLog(`${opened("snake", 10)}\n16 ext-api snake storage.set quota\n${opened("2048", 20)}`);
    expect(bundledFailures(failed, PANELS).join()).toMatch(/snake's API call failed: snake storage.set quota/);
    const skipped = parseLog(`1 extension-skipped snake \`apiVersion\` must be 1\n${opened("snake", 10)}\n${opened("2048", 20)}`);
    expect(bundledFailures(skipped, PANELS).join()).toMatch(/extension-skipped snake/);
  });

  it("wants a panel's title when it names one (the focus timer counting down from its stored end)", () => {
    const panels = [{ id: "focus", shot: "focus.png", title: /^1[0-7]:\d\d · Focus$/ }];
    expect(bundledFailures(parseLog(`${opened("focus", 10)}\n17 title 17:29 · Focus`), panels)).toEqual([]);
    expect(bundledFailures(parseLog(`${opened("focus", 10)}\n17 title Focus timer`), panels).join()).toMatch(/never showed a title/);
    const focus = BUNDLED_PANELS.find((p) => p.id === "focus");
    expect(focus?.title?.test("17:29 · Focus")).toBe(true);
    expect(focus?.title?.test("25:00 · Focus")).toBe(false);
  });

  it("seeds storage the way Little Joan writes it", () => {
    const now = Date.UTC(2026, 9, 18, 12, 0);
    for (const panel of BUNDLED_PANELS.filter((p) => p.seed)) {
      const file = JSON.parse(seedFile(panel.seed?.(now) ?? {})) as { version: number; items: Record<string, unknown> };
      expect(file.version).toBe(1);
      expect(Object.keys(file.items).length).toBeGreaterThan(0);
    }
    const focus = BUNDLED_PANELS.find((p) => p.id === "focus")?.seed?.(now) as { timer: { endsAt: number } };
    expect(focus.timer.endsAt - now).toBe(17.5 * 60_000);
  });

  it("sees keys arrive by the panel's own writes after it came up", () => {
    const log = parseLog(`${opened("2048", 20)}\n30 ext-api 2048 storage.set ok`);
    expect(keysReached(log, "2048")).toBe(true);
    expect(keysReached(parseLog(opened("2048", 20)), "2048")).toBe(false);
  });
});
