import { describe, expect, it } from "vitest";
import type { DocumentLike, ElementLike } from "../command/view.ts";
import { groupByDay, type HistoryPage } from "./logic.ts";
import { renderHistory } from "./view.ts";

/** A stand-in element that records what was done to it and refuses HTML. */
class FakeElement implements ElementLike {
  className = "";
  textContent: string | null = null;
  readonly attributes = new Map<string, string>();
  readonly children: FakeElement[] = [];
  readonly style = { setProperty: () => undefined };
  readonly tag: string;

  constructor(tag: string) {
    this.tag = tag;
  }

  set innerHTML(_html: string) {
    throw new Error("innerHTML must never be used for history");
  }

  setAttribute(name: string, value: string): void {
    this.attributes.set(name, value);
  }

  append(...nodes: FakeElement[]): void {
    this.children.push(...nodes);
  }

  addEventListener(): void {}

  remove(): void {}

  all(): FakeElement[] {
    return [this, ...this.children.flatMap((child) => child.all())];
  }

  find(className: string): FakeElement | undefined {
    return this.all().find((el) => el.className === className);
  }

  text(): string {
    return this.textContent ?? this.children.map((child) => child.text()).join("");
  }
}

const doc: DocumentLike<FakeElement> = { createElement: (tag) => new FakeElement(tag) };
const at = (day: number, hour: number, minute = 0) => new Date(2026, 9, day, hour, minute).getTime();
const NOW = at(18, 16);
const page = (url: string, title: string, lastVisit: number): HistoryPage => ({
  url,
  title,
  host: new URL(url).host,
  lastVisit,
  visitCount: 1,
});

describe("drawing the history", () => {
  const groups = groupByDay(
    [
      page("http://127.0.0.1:5000/smoke.html", "LJ-SMOKE ready", at(18, 15, 2)),
      page("https://example.com/", "Example", at(18, 9)),
      page("https://old.test/", "Old", at(17, 9)),
    ],
    NOW,
  );

  it("draws a heading per day, then numbered rows: icon, Title — site, time, ×", () => {
    const items = renderHistory(doc, groups, 1, "en-GB");
    expect(items.map((item) => item.className)).toEqual([
      "history-day",
      "row history-row",
      "row history-row selected",
      "history-day",
      "row history-row",
    ]);
    expect(items[0]?.text()).toBe("Today");
    expect(items[3]?.text()).toBe("Yesterday");
    const [, smoke, example, , old] = items;
    expect(smoke?.find("row-text")?.text()).toBe("LJ-SMOKE ready — 127.0.0.1:5000");
    expect(smoke?.find("history-time")?.text()).toBe("15:02");
    expect(smoke?.attributes.get("data-index")).toBe("0");
    expect(example?.attributes.get("aria-selected")).toBe("true");
    expect(old?.attributes.get("data-index")).toBe("2");
    const remove = old?.find("history-remove");
    expect(remove?.tag).toBe("button");
    expect(remove?.attributes.get("data-remove")).toBe("2");
    expect(remove?.attributes.get("aria-label")).toBe("Remove Old from history");
    const img = smoke?.all().find((el) => el.tag === "img");
    expect(img?.attributes.get("src")).toBe("https://127.0.0.1:5000/favicon.ico");
  });

  it("renders a title full of HTML as text", () => {
    const title = '<img src=x onerror="alert(1)">';
    const [, row] = renderHistory(doc, groupByDay([page("https://evil.test/", title, at(18, 10))], NOW), null);
    const titleEl = row?.find("row-title");
    expect(titleEl?.textContent).toBe(title);
    expect(titleEl?.children).toHaveLength(0);
    const images = row?.all().filter((el) => el.tag === "img") ?? [];
    expect(images.map((img) => img.attributes.get("src"))).toEqual(["https://evil.test/favicon.ico"]);
  });

  it("draws nothing for no history", () => {
    expect(renderHistory(doc, [], null)).toEqual([]);
  });
});
