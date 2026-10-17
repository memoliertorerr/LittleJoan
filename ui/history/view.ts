// Draws the history view's list: a heading per day, then a row per page — the site's icon,
// "Title — site", the time, and a remove (×) button. Text only: titles come from web pages, so
// everything goes in through textContent, never as HTML (the command bar's rule, command/view.ts).
import { siteIcon, type DocumentLike, type ElementLike } from "../command/view.ts";
import { rowText, timeText, type DayGroup } from "./logic.ts";

function element<E extends ElementLike>(doc: DocumentLike<E>, tag: string, className: string, text?: string): E {
  const el = doc.createElement(tag);
  el.className = className;
  if (text !== undefined) {
    el.textContent = text;
  }
  return el;
}

/**
 * The list's items: per day a heading, then its pages, numbered across the days (`data-index`,
 * which the remove button carries as `data-remove`). `selected` is a row number.
 */
export function renderHistory<E extends ElementLike>(
  doc: DocumentLike<E>,
  groups: readonly DayGroup[],
  selected: number | null,
  locale?: string,
): E[] {
  const items: E[] = [];
  let index = 0;
  for (const group of groups) {
    const heading = element(doc, "li", "history-day", group.label);
    heading.setAttribute("role", "presentation");
    items.push(heading);
    for (const page of group.pages) {
      const on = index === selected;
      const item = element(doc, "li", `row history-row${on ? " selected" : ""}`);
      item.setAttribute("role", "option");
      item.setAttribute("aria-selected", on ? "true" : "false");
      item.setAttribute("data-index", String(index));
      const { title, domain } = rowText(page);
      const text = element(doc, "span", "row-text");
      text.append(element(doc, "span", "row-title", title));
      if (domain) {
        text.append(element(doc, "span", "row-domain", ` — ${domain}`));
      }
      const time = element(doc, "span", "history-time", timeText(page.lastVisit, locale));
      const remove = element(doc, "button", "history-remove", "×");
      remove.setAttribute("type", "button");
      remove.setAttribute("aria-label", `Remove ${title} from history`);
      remove.setAttribute("title", "Remove from history");
      remove.setAttribute("data-remove", String(index));
      item.append(siteIcon(doc, page.host, "page"), text, time, remove);
      items.push(item);
      index++;
    }
  }
  return items;
}
