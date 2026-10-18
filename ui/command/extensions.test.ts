import { describe, expect, it } from "vitest";
import { commandRows, iconFor, isWebPage, matches, type ExtensionInfo } from "./extensions.ts";
import { buildRows } from "./rows.ts";
import { renderRows } from "./view.ts";

const ICON = "data:image/svg+xml;base64,PHN2Zy8+";

function extension(overrides: Partial<ExtensionInfo>): ExtensionInfo {
  return {
    id: "hello",
    name: "Hello",
    version: "1.0.0",
    description: "",
    author: "",
    kind: "bundled",
    enabled: true,
    icon: ICON,
    permissions: ["storage"],
    commands: [{ id: "open", title: "Say hello", keywords: ["example", "greeting"] }],
    pageActions: [],
    ...overrides,
  };
}

const QR = extension({
  id: "qr",
  name: "QR code",
  icon: null,
  commands: [],
  pageActions: [{ id: "page", title: "Show a QR code", keywords: ["share"] }],
});

describe("extension commands in the command bar", () => {
  it("match by title, keywords and the extension's name, every word a prefix", () => {
    const hello = [extension({})];
    for (const query of ["hello", "say", "SAY HEL", "exam", "greet", "  hello  "]) {
      expect(commandRows(query, hello, false).map((row) => row.title), query).toEqual(["Say hello"]);
    }
    for (const query of ["", "  ", "ello", "say goodbye", "xample", "h e l l o x"]) {
      expect(commandRows(query, hello, false), query).toEqual([]);
    }
    expect(matches("snake game", ["Play Snake", "game"])).toBe(true);
    expect(matches("2048", ["Play 2048"])).toBe(true);
  });

  it("show the extension's name as the muted part and open that command", () => {
    const [row] = commandRows("hello", [extension({})], false);
    expect(row).toMatchObject({
      kind: "command",
      title: "Say hello",
      domain: "Hello",
      icon: "extension",
      image: ICON,
      action: { kind: "command", extension: "hello", command: "open", pageAction: false },
    });
  });

  it("show page actions only while a web page is open, and say so", () => {
    expect(commandRows("qr", [QR], false)).toEqual([]);
    const [row] = commandRows("qr", [QR], true);
    expect(row).toMatchObject({
      title: "Show a QR code",
      domain: "QR code, for this page",
      action: { kind: "command", extension: "qr", command: "page", pageAction: true },
    });
    expect(row && "image" in row).toBe(false);
    expect(isWebPage("https://example.com/")).toBe(true);
    expect(isWebPage("")).toBe(false);
    expect(isWebPage("ljext://qr.localhost/page.html")).toBe(false);
  });

  it("never show switched-off extensions", () => {
    expect(commandRows("hello", [extension({ enabled: false })], true)).toEqual([]);
    expect(commandRows("qr", [{ ...QR, enabled: false }], true)).toEqual([]);
  });

  it("take only image data: URIs as icons", () => {
    for (const bad of ["https://example.com/i.svg", "data:text/html;base64,PGI+", "data:image/svg+xml,<svg onload=x>", `${ICON}"x`]) {
      expect(iconFor(extension({ icon: bad }))).toBeUndefined();
    }
    expect(iconFor(extension({}))).toBe(ICON);
  });

  it("render as text, markup and all, after the built-in rows", () => {
    const sneaky = extension({
      name: "<b>Bold</b>",
      commands: [{ id: "open", title: "<img src=x onerror=alert(1)> hello", keywords: [] }],
    });
    const { rows } = buildRows("hello", [], { commands: (q) => commandRows(q, [sneaky], false) });
    expect(rows.map((row) => row.kind)).toEqual(["search", "command", "settings"]);
    const doc = standIn();
    const [, drawn] = renderRows(doc, rows, null);
    expect(drawn?.text()).toBe("<img src=x onerror=alert(1)> hello — <b>Bold</b>");
    expect(doc.made.filter((tag) => tag === "img")).toHaveLength(1);
  });
});

/** A stand-in document: elements keep their text, and setting innerHTML throws. */
interface Fake {
  className: string;
  textContent: string | null;
  children: Fake[];
  attrs: Record<string, string>;
  style: { setProperty(name: string, value: string): void };
  setAttribute(name: string, value: string): void;
  append(...nodes: Fake[]): void;
  addEventListener(): void;
  remove(): void;
  text(): string;
}

function standIn(): { made: string[]; createElement(tag: string): Fake } {
  const made: string[] = [];
  return {
    made,
    createElement(tag: string): Fake {
      made.push(tag);
      const el: Fake = {
        className: "",
        textContent: null,
        children: [],
        attrs: {},
        style: { setProperty: () => undefined },
        setAttribute(name, value) {
          el.attrs[name] = value;
        },
        append(...nodes) {
          el.children.push(...nodes);
        },
        addEventListener: () => undefined,
        remove: () => undefined,
        text() {
          return (el.textContent ?? "") + el.children.map((child) => child.text()).join("");
        },
      };
      Object.defineProperty(el, "innerHTML", {
        set() {
          throw new Error("innerHTML is never used");
        },
      });
      return el;
    },
  };
}
