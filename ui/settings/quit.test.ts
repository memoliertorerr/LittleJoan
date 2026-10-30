import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { QUIT_COMMAND, quitButton, type QuitControl } from "./quit.ts";

/** A button that remembers its click listener. */
function fakeButton(): QuitControl & { click(): void } {
  let listener: (() => void) | undefined;
  return {
    disabled: false,
    addEventListener(_type, next) {
      listener = next;
    },
    click() {
      listener?.();
    },
  };
}

const flush = () => new Promise((done) => setTimeout(done, 0));

describe("Settings → General's Quit Little Joan (LJ19.7)", () => {
  it("calls the Settings-only command once, after stopping a key recording", async () => {
    const button = fakeButton();
    const steps: string[] = [];
    quitButton(
      button,
      (command) => {
        steps.push(`call ${command}`);
        return Promise.resolve();
      },
      () => steps.push("before"),
      (message) => steps.push(`error ${message}`),
    );
    expect(steps).toEqual([]);
    button.click();
    expect(button.disabled).toBe(true);
    // A second click while quitting does nothing.
    button.click();
    await flush();
    expect(steps).toEqual(["before", "call quit_from_settings"]);
    expect(QUIT_COMMAND).toBe("quit_from_settings");
  });

  it("works again and shows why when the call fails", async () => {
    const button = fakeButton();
    const errors: string[] = [];
    quitButton(
      button,
      () => Promise.reject(new Error("not allowed")),
      () => undefined,
      (message) => errors.push(message),
    );
    button.click();
    await flush();
    expect(button.disabled).toBe(false);
    expect(errors).toEqual(["not allowed"]);
  });

  it("is the last control of the General tab, with its note under it", () => {
    const html = readFileSync(resolve(import.meta.dirname, "..", "settings.html"), "utf8");
    const start = html.indexOf('<section id="panel-general"');
    const general = html.slice(start, html.indexOf("</section>", start));
    expect(start).toBeGreaterThan(-1);
    const controls = [...general.matchAll(/<(button|input|select)\b[^>]*\bid="([^"]+)"/g)].map((match) => match[2]);
    expect(controls.at(-1)).toBe("quit");
    expect(general).toMatch(/<button id="quit" type="button"[^>]*>Quit<\/button>/);
    const note = /<p class="setting-note" id="quit-note">([\s\S]*?)<\/p>/.exec(general)?.[1]?.replace(/\s+/g, " ").trim();
    expect(note).toContain("haven't saved are dropped");
    expect(general.indexOf('id="quit-note"')).toBeGreaterThan(general.indexOf('id="quit"'));
  });
});
