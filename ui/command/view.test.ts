import { describe, expect, it } from "vitest";
import { SETTINGS_ROW, buildRows, type Row } from "./rows.ts";
import { renderRows, type DocumentLike, type ElementLike } from "./view.ts";

/** A stand-in element that records what was done to it and refuses HTML. */
class FakeElement implements ElementLike {
  className = "";
  textContent: string | null = null;
  readonly attributes = new Map<string, string>();
  readonly children: FakeElement[] = [];
  readonly listeners = new Map<string, () => void>();
  readonly styleProperties = new Map<string, string>();
  removed = false;
  readonly style = { setProperty: (name: string, value: string) => void this.styleProperties.set(name, value) };

  readonly tag: string;

  constructor(tag: string) {
    this.tag = tag;
  }

  set innerHTML(_html: string) {
    throw new Error("innerHTML must never be used for rows");
  }

  setAttribute(name: string, value: string): void {
    this.attributes.set(name, value);
  }

  append(...nodes: FakeElement[]): void {
    this.children.push(...nodes);
  }

  addEventListener(type: string, listener: () => void): void {
    this.listeners.set(type, listener);
  }

  remove(): void {
    this.removed = true;
  }

  all(): FakeElement[] {
    return [this, ...this.children.flatMap((child) => child.all())];
  }

  text(): string {
    return this.textContent ?? this.children.map((child) => child.text()).join("");
  }
}

const doc: DocumentLike<FakeElement> = { createElement: (tag) => new FakeElement(tag) };
const draw = (rows: Row[], selected: number | null) => renderRows(doc, rows, selected);

describe("drawing rows", () => {
  it("draws Title — domain with the favicon from the site and a lettered stand-in", () => {
    const { rows } = buildRows("", [
      { kind: "topSite", title: "Example", url: "https://example.com/", host: "example.com", score: 1 },
    ]);
    const [item, settings] = draw(rows, 0);
    expect(item?.all().find((el) => el.className === "row-text")?.text()).toBe("Example — example.com");
    expect(item?.attributes.get("aria-selected")).toBe("true");
    expect(item?.className).toContain("selected");
    const img = item?.all().find((el) => el.tag === "img");
    expect(img?.attributes.get("src")).toBe("https://example.com/favicon.ico");
    expect(img?.attributes.get("referrerpolicy")).toBe("no-referrer");
    const letter = item?.all().find((el) => el.className === "row-letter");
    expect(letter?.textContent).toBe("E");
    expect(letter?.styleProperties.get("--hue")).toMatch(/^\d+$/);
    // It shows once loaded; a favicon that fails to load leaves the letter.
    img?.listeners.get("load")?.();
    expect(img?.className).toBe("row-favicon loaded");
    img?.listeners.get("error")?.();
    expect(img?.removed).toBe(true);
    expect(settings?.text()).toBe(SETTINGS_ROW.title);
    expect(settings?.attributes.get("aria-selected")).toBe("false");
  });

  it("renders a title full of HTML as text", () => {
    const title = '<img src=x onerror="alert(1)">';
    const { rows } = buildRows("x", [
      { kind: "history", title, url: "https://evil.test/", host: "evil.test", score: 1 },
    ]);
    const [item] = draw(rows, null);
    const titleEl = item?.all().find((el) => el.className === "row-title");
    expect(titleEl?.textContent).toBe(title);
    expect(titleEl?.children).toHaveLength(0);
    const images = item?.all().filter((el) => el.tag === "img") ?? [];
    expect(images.map((img) => img.attributes.get("src"))).toEqual(["https://evil.test/favicon.ico"]);
  });

  it("gives settings its own icon and no favicon", () => {
    const [settings] = draw([SETTINGS_ROW], null);
    expect(settings?.all().some((el) => el.tag === "img")).toBe(false);
    expect(settings?.all().some((el) => el.className.includes("row-icon-settings"))).toBe(true);
  });

  it("numbers the rows for the mouse", () => {
    const { rows } = buildRows("", []);
    expect(draw(rows, null).map((item) => item.attributes.get("data-index"))).toEqual(["0"]);
  });
});
