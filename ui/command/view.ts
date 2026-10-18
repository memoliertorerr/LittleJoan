// Draws the command bar's rows. Text only: every title and domain goes in through textContent,
// never as HTML — titles come from web pages (and, later, extensions). Written against a small
// slice of the DOM so the tests can draw into a stand-in document.
import { faviconUrl, hueFor, letterFor, type Row } from "./rows.ts";

/** The part of an element this file uses (an HTMLElement has all of it). */
export interface ElementLike {
  className: string;
  textContent: string | null;
  setAttribute(name: string, value: string): void;
  append(...nodes: this[]): void;
  addEventListener(type: string, listener: () => void): void;
  remove(): void;
  style: { setProperty(name: string, value: string): void };
}

/** Makes elements: `document`, or a stand-in in tests. */
export interface DocumentLike<E extends ElementLike> {
  createElement(tag: string): E;
}

function element<E extends ElementLike>(doc: DocumentLike<E>, tag: string, className: string, text?: string): E {
  const el = doc.createElement(tag);
  el.className = className;
  if (text !== undefined) {
    el.textContent = text;
  }
  return el;
}

/**
 * A site's icon: its first letter in a tinted circle, with the favicon (straight from the site)
 * drawn over it once it loads; a favicon that fails to load is removed, leaving the letter. The
 * toolbar uses it too. `kind` names the box's extra class.
 */
export function siteIcon<E extends ElementLike>(doc: DocumentLike<E>, host: string, kind: string): E {
  const box = element(doc, "span", `row-icon row-icon-${kind}`);
  box.setAttribute("aria-hidden", "true");
  if (!host) {
    return box;
  }
  const letter = element(doc, "span", "row-letter", letterFor(host));
  letter.style.setProperty("--hue", String(hueFor(host)));
  box.append(letter);
  const src = faviconUrl(host);
  if (src) {
    const img = element(doc, "img", "row-favicon");
    img.setAttribute("alt", "");
    img.setAttribute("referrerpolicy", "no-referrer");
    img.setAttribute("src", src);
    // Shown once it loaded (until then the letter shows); one that fails goes away.
    img.addEventListener("load", () => {
      img.className = "row-favicon loaded";
    });
    img.addEventListener("error", () => img.remove());
    box.append(img);
  }
  return box;
}

/**
 * A row's icon: its site's, or for Settings and built-in commands their own symbol (style.css);
 * an extension's command shows its extension's icon (a data: URI) over the symbol.
 */
function icon<E extends ElementLike>(doc: DocumentLike<E>, row: Row): E {
  if (row.icon !== undefined) {
    const box = siteIcon(doc, "", row.icon);
    if (row.image) {
      const img = element(doc, "img", "row-favicon loaded");
      img.setAttribute("alt", "");
      img.setAttribute("src", row.image);
      box.append(img);
    }
    return box;
  }
  return siteIcon(doc, row.kind === "settings" ? "" : row.host, row.kind);
}

/** One row: icon, then "Title — domain" with the domain muted. */
export function renderRow<E extends ElementLike>(doc: DocumentLike<E>, row: Row, index: number, selected: boolean): E {
  const item = element(doc, "li", `row row-${row.kind}${selected ? " selected" : ""}`);
  item.setAttribute("role", "option");
  item.setAttribute("aria-selected", selected ? "true" : "false");
  item.setAttribute("data-index", String(index));
  const text = element(doc, "span", "row-text");
  text.append(element(doc, "span", "row-title", row.title));
  if (row.domain) {
    text.append(element(doc, "span", "row-domain", ` — ${row.domain}`));
  }
  item.append(icon(doc, row), text);
  return item;
}

/** All rows, the selected one marked. */
export function renderRows<E extends ElementLike>(doc: DocumentLike<E>, rows: readonly Row[], selected: number | null): E[] {
  return rows.map((row, index) => renderRow(doc, row, index, index === selected));
}
