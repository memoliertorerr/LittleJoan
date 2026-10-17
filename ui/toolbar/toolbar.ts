// The toolbar (DOM glue): back on the left, the site's icon and domain in the middle (a click
// opens the command bar with the address selected), "open in default browser" on the right.
// The empty parts carry data-tauri-drag-region, so they drag the window. What it shows comes
// from logic.ts.
import { invoke } from "@tauri-apps/api/core";
import { siteIcon } from "../command/view.ts";
import { toolbarView } from "./logic.ts";

export interface Toolbar {
  /** The page's address changed ("" = none) or whether it can go back. */
  update(address: string, canGoBack: boolean): void;
}

export function toolbar(root: HTMLElement): Toolbar {
  const back = root.querySelector<HTMLButtonElement>("#back");
  const site = root.querySelector<HTMLButtonElement>("#site");
  const domain = root.querySelector<HTMLElement>("#domain");
  const openInBrowser = root.querySelector<HTMLButtonElement>("#open-browser");
  let shownSite: string | null = null;

  back?.addEventListener("click", () => void invoke("go_back"));
  site?.addEventListener("click", () => void invoke("set_overlay", { on: true }));
  openInBrowser?.addEventListener("click", () => void invoke("open_in_browser"));

  return {
    update(address: string, canGoBack: boolean) {
      const view = toolbarView(address, canGoBack);
      if (domain) {
        domain.textContent = view.domain;
      }
      if (site) {
        site.title = address;
        // A new icon only for a new site, so the favicon isn't fetched again for every page.
        if (view.site !== shownSite) {
          shownSite = view.site;
          site.querySelector(".row-icon")?.remove();
          site.prepend(siteIcon<HTMLElement>(document, view.site, "site"));
        }
      }
      if (back) {
        back.disabled = !view.back;
      }
      if (openInBrowser) {
        openInBrowser.disabled = !view.openInBrowser;
      }
    },
  };
}
