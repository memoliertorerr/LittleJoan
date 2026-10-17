// The bar's view classes ("overlay", "toolbar", "history") live on <body> and switch the whole
// page; reusing one on an element inside it, or styling it without the body's class, hides or
// shows the wrong thing (LJ24 shipped a `.toolbar { display: none }` that hid the whole toolbar
// view).
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const ui = import.meta.dirname;
const html = readFileSync(resolve(ui, "bar.html"), "utf8");
// Every stylesheet the bar links. Comments may name the classes; only rules count.
const sheets = [...html.matchAll(/<link rel="stylesheet" href="\.\/([^"]+)"/g)].map((match) => match[1] ?? "");
const css = sheets
  .map((sheet) => readFileSync(resolve(ui, sheet), "utf8"))
  .join("\n")
  .replace(/\/\*[\s\S]*?\*\//g, "");
const VIEWS = ["overlay", "toolbar", "history"];

describe("the bar's view classes", () => {
  it("are checked in every stylesheet the bar links", () => {
    expect(sheets).toEqual(["style.css", "history/history.css"]);
  });

  it("are only on <body>", () => {
    for (const match of html.matchAll(/<(\w+)[^>]*\sclass="([^"]*)"/g)) {
      const [, tag, classes] = match;
      const used = (classes ?? "").split(/\s+/).filter((name) => VIEWS.includes(name));
      if (tag !== "body") {
        expect(used, `<${tag} class="${classes}">`).toEqual([]);
      }
    }
  });

  it("are only styled together with the body's own class", () => {
    for (const view of VIEWS) {
      // Every selector naming `.overlay` / `.toolbar` / `.history` must be `.bar-body.<view>`.
      const loose = new RegExp(`(^|[^\\w-])(?<!\\.bar-body)\\.${view}(?![\\w-])`, "gm");
      expect(css.match(loose) ?? [], view).toEqual([]);
    }
  });
});
