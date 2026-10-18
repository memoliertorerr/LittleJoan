// Settings → General's "Phone or desktop per site" (DOM glue, LJ40): the list from sites-logic.ts,
// each site as text with its Remove or Reset button. The list is a draft that Save sends
// (settings' siteModes); Cancel forgets it.
import { NO_SITES, afterAction, rowText, sameSites, siteRows, type BuiltInSite, type SiteMode } from "./sites-logic.ts";

export interface SitesList {
  /** The saved table and this OS's built-ins arrived (or Save happened): the draft starts there. */
  load(saved: Record<string, SiteMode>, builtIns: BuiltInSite[]): void;
  /** What Save sends, or undefined when the list wasn't changed (Rust then keeps the saved one). */
  draft(): Record<string, SiteMode> | undefined;
}

export function sitesList(list: HTMLElement, empty: HTMLElement, changed: () => void): SitesList {
  let saved: Record<string, SiteMode> = {};
  let draft: Record<string, SiteMode> = {};
  let builtIns: BuiltInSite[] = [];

  const draw = (): void => {
    const rows = siteRows(draft, builtIns);
    list.replaceChildren(
      ...rows.map((row) => {
        const li = document.createElement("li");
        li.className = "site-mode";
        const label = document.createElement("span");
        label.className = "site-mode-text";
        label.textContent = rowText(row);
        const button = document.createElement("button");
        button.type = "button";
        button.className = "site-mode-button";
        button.textContent = row.action === "reset" ? "Reset" : "Remove";
        button.setAttribute("aria-label", `${button.textContent} ${row.host}`);
        button.addEventListener("click", () => {
          draft = afterAction(draft, builtIns, row.host);
          draw();
          changed();
        });
        li.append(label, button);
        return li;
      }),
    );
    list.hidden = rows.length === 0;
    empty.textContent = NO_SITES;
    empty.hidden = rows.length > 0;
  };

  return {
    load(table: Record<string, SiteMode>, sites: BuiltInSite[]) {
      saved = { ...table };
      draft = { ...table };
      builtIns = sites;
      draw();
    },
    draft() {
      return sameSites(saved, draft) ? undefined : { ...draft };
    },
  };
}
