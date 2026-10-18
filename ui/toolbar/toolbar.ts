// The toolbar (DOM glue): back on the left, the site's icon and domain in the middle — for an
// extension's panel its icon and name, as text — (a click opens the command bar with the address
// selected), "open in default browser" on the right.
// The empty parts carry data-tauri-drag-region, so they drag the window. What it shows comes
// from logic.ts.
import { invoke } from "@tauri-apps/api/core";
import { siteIcon } from "../command/view.ts";
import { toolbarView, type ExtensionLabel } from "./logic.ts";

export interface Toolbar {
  /** The page's address changed ("" = none), whether it can go back, or the extension shown. */
  update(address: string, canGoBack: boolean, extension: ExtensionLabel | null): void;
}

export function toolbar(root: HTMLElement): Toolbar {
  const back = root.querySelector<HTMLButtonElement>("#back");
  const site = root.querySelector<HTMLButtonElement>("#site");
  const domain = root.querySelector<HTMLElement>("#domain");
  const openInBrowser = root.querySelector<HTMLButtonElement>("#open-browser");
  /** What the icon shows now: a site, or `ext:<name>`. */
  let shownSite: string | null = null;

  back?.addEventListener("click", () => void invoke("go_back"));
  site?.addEventListener("click", () => void invoke("set_overlay", { on: true }));
  openInBrowser?.addEventListener("click", () => void invoke("open_in_browser"));

  return {
    update(address: string, canGoBack: boolean, extension: ExtensionLabel | null) {
      const view = toolbarView(address, canGoBack, extension);
      if (domain) {
        domain.textContent = view.domain;
      }
      if (site) {
        site.title = extension ? extension.name : address;
        // A new icon only for a new site, so the favicon isn't fetched again for every page.
        const key = extension ? `ext:${extension.name}` : view.site;
        if (key !== shownSite) {
          shownSite = key;
          site.querySelector(".row-icon")?.remove();
          const box = siteIcon<HTMLElement>(document, view.site, "site");
          if (view.icon) {
            const img = document.createElement("img");
            img.className = "row-favicon loaded";
            img.alt = "";
            img.src = view.icon;
            box.append(img);
          }
          site.prepend(box);
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
