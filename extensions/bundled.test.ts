// Checks every bundled extension the way docs/extensions.md asks of one ("Getting a bundled
// extension into Little Joan"): the folder is named after its id, it loads the SDK before its own
// scripts and has no inline scripts (the CSP would block them), it fetches nothing from the
// network and declares no hosts, it draws text as text, and its logic is tested. (The app's own
// manifest checks run in Rust: registry::tests::the_shipped_extensions_are_all_valid.)
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const root = import.meta.dirname;
const bundled = readdirSync(root).filter((name) => existsSync(join(root, name, "extension.json")));

interface Manifest {
  id: string;
  icon?: string;
  permissions?: string[];
  contributes?: { commands?: { panel: string }[]; pageActions?: { panel: string }[] };
}

/** Every file of an extension (relative paths), tests left out as build.rs leaves them out. */
function files(dir: string, prefix = ""): string[] {
  return readdirSync(join(dir, prefix)).flatMap((name) => {
    const path = prefix ? `${prefix}/${name}` : name;
    return statSync(join(dir, path)).isDirectory() ? files(dir, path) : name.includes(".test.") ? [] : [path];
  });
}

describe("bundled extensions", () => {
  it("are all here", () => {
    expect(bundled).toEqual(
      expect.arrayContaining(["hello", "snake", "2048", "notes", "focus", "clock", "qr", "clean-link", "wayback", "markdown-link", "json", "text-tools", "color"]),
    );
  });

  for (const id of bundled) {
    describe(id, () => {
      const dir = join(root, id);
      const manifest = JSON.parse(readFileSync(join(dir, "extension.json"), "utf8")) as Manifest;
      const panels = [...(manifest.contributes?.commands ?? []), ...(manifest.contributes?.pageActions ?? [])].map((c) => c.panel);
      const sources = files(dir).filter((path) => /\.(html|js|css)$/.test(path));

      it("is named after its id, with an icon and panels that exist", () => {
        expect(manifest.id).toBe(id);
        expect(manifest.icon && existsSync(join(dir, manifest.icon))).toBe(true);
        expect(panels.length).toBeGreaterThan(0);
        for (const panel of panels) {
          expect(existsSync(join(dir, panel)), panel).toBe(true);
        }
      });

      it("loads the SDK first and runs no inline script", () => {
        for (const panel of new Set(panels)) {
          const html = readFileSync(join(dir, panel), "utf8");
          const scripts = [...html.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/g)];
          expect(scripts[0]?.[1], `${panel}: the SDK comes first`).toContain('src="/__lj/sdk/v1.js"');
          for (const [, attributes, body] of scripts) {
            expect(attributes, `${panel}: scripts come from files`).toMatch(/\bsrc="/);
            expect(body?.trim(), `${panel}: no inline script`).toBe("");
          }
          expect(html, `${panel}: no inline event handlers`).not.toMatch(/\son[a-z]+\s*=/i);
        }
      });

      it("reaches no network and declares no hosts", () => {
        expect((manifest.permissions ?? []).filter((p) => p.startsWith("network:"))).toEqual([]);
        if (id === "hello") {
          return; // hello tries a fetch on purpose: the CSP must block it (the smoke test checks).
        }
        for (const path of sources) {
          const text = readFileSync(join(dir, path), "utf8");
          expect(text, path).not.toMatch(/\b(fetch|XMLHttpRequest|WebSocket|EventSource|sendBeacon|importScripts)\s*\(/);
          expect(text, path).not.toMatch(/url\(\s*["']?https?:/i);
          expect(text, path).not.toMatch(/<(img|link|script|iframe|audio|video|source)\b[^>]*\b(src|href)="https?:/i);
        }
      });

      it("draws text as text", () => {
        for (const path of sources.filter((p) => p.endsWith(".js"))) {
          const text = readFileSync(join(dir, path), "utf8");
          expect(text, path).not.toMatch(/\.(innerHTML|outerHTML)\s*=|insertAdjacentHTML|document\.write/);
          expect(text, path).not.toMatch(/\beval\s*\(|new Function\s*\(/);
        }
      });

      it("keeps its logic pure and tested", () => {
        if (id === "hello") {
          return; // The example has nothing to test but what CI checks in the app.
        }
        expect(existsSync(join(dir, "logic.js"))).toBe(true);
        expect(existsSync(join(dir, "logic.test.ts"))).toBe(true);
        const logic = readFileSync(join(dir, "logic.js"), "utf8");
        expect(logic).not.toMatch(/\b(document|window|littleJoan|localStorage|navigator)\.[A-Za-z_$]/);
      });
    });
  }
});
