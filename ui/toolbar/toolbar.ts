// The toolbar (DOM glue): back on the left, the site's icon and domain in the middle — for an
// extension's panel its icon and name, as text — (a click opens the command bar with the address
// selected), "open in default browser" on the right, and before it the "Phone" / "Desktop" pill
// (logic.ts says when; a click asks Rust for the other version for this site: set_site_mode).
// The empty parts carry data-tauri-drag-region, so they drag the window. The status area (LJ15)
// takes the pill's place while it shows: Rust's message ("Downloading …", "… saved" with "Show in
// folder"), timed by logic.ts against the time the toolbar is visible. What it shows comes from
// logic.ts.
import { invoke } from "@tauri-apps/api/core";
import { siteIcon } from "../command/view.ts";
import {
  initialStatus,
  nextStatusChange,
  shownStatus,
  statusMessage,
  toolbarView,
  withStatus,
  withToolbarVisible,
  type ExtensionLabel,
  type Pill,
  type SiteModeInfo,
  type StatusView,
} from "./logic.ts";

export interface Toolbar {
  /**
   * The page's address changed ("" = none), whether it can go back, the extension shown, or what
   * its site is on.
   */
  update(address: string, canGoBack: boolean, extension: ExtensionLabel | null, info: SiteModeInfo | null): void;
  /** Rust's status message (anything: checked). */
  setStatus(message: unknown): void;
  /** The toolbar is shown (true) or the command bar covers it. */
  setVisible(visible: boolean): void;
}

export function toolbar(root: HTMLElement): Toolbar {
  const back = root.querySelector<HTMLButtonElement>("#back");
  const site = root.querySelector<HTMLButtonElement>("#site");
  const domain = root.querySelector<HTMLElement>("#domain");
  const badge = root.querySelector<HTMLElement>("#site-badge");
  const openInBrowser = root.querySelector<HTMLButtonElement>("#open-browser");
  const pill = root.querySelector<HTMLButtonElement>("#site-pill");
  const status = root.querySelector<HTMLElement>("#toolbar-status");
  const statusText = root.querySelector<HTMLElement>("#toolbar-status-text");
  const statusAction = root.querySelector<HTMLButtonElement>("#toolbar-status-action");
  /** What the icon shows now: a site, or `ext:<name>`. */
  let shownSite: string | null = null;
  /** What the pill shows now (its click asks for `next`). */
  let shownPill: Pill | null = null;
  /** The statuses held, and the one shown (a new window opens with the command bar over it). */
  let statuses = initialStatus(false, performance.now());
  let shown: StatusView | null = null;
  let timer: ReturnType<typeof setTimeout> | undefined;

  back?.addEventListener("click", () => void invoke("go_back"));
  site?.addEventListener("click", () => void invoke("set_overlay", { on: true }));
  openInBrowser?.addEventListener("click", () => void invoke("open_in_browser"));
  pill?.addEventListener("click", () => {
    if (shownPill) {
      void invoke("set_site_mode", { mode: shownPill.next });
    }
  });
  statusAction?.addEventListener("click", () => {
    if (shown?.action) {
      // Which file is Rust's (the last one saved); a failure leaves the status as it is.
      invoke(shown.action.command).catch(() => undefined);
    }
  });

  /** The pill shows only while no status does. */
  const showPill = (): void => {
    if (pill) {
      pill.hidden = shownPill === null || shown !== null;
    }
  };

  /** Draws the status that is due now (as text), and wakes up when it runs out. */
  const refresh = (): void => {
    const now = performance.now();
    shown = shownStatus(statuses, now);
    if (status && statusText && statusAction) {
      status.hidden = shown === null;
      statusText.textContent = shown?.text ?? "";
      status.title = shown?.title ?? "";
      statusAction.hidden = !shown?.action;
      statusAction.textContent = shown?.action?.label ?? "";
    }
    showPill();
    clearTimeout(timer);
    const next = nextStatusChange(statuses, now);
    if (next !== null) {
      timer = setTimeout(refresh, next + 20);
    }
  };

  return {
    update(address: string, canGoBack: boolean, extension: ExtensionLabel | null, info: SiteModeInfo | null) {
      const view = toolbarView(address, canGoBack, extension, info);
      shownPill = view.pill;
      showPill();
      if (pill) {
        pill.textContent = view.pill?.label ?? "";
        pill.title = view.pill?.title ?? "";
        pill.setAttribute("aria-label", view.pill?.title ?? "");
      }
      if (domain) {
        domain.textContent = view.domain;
      }
      if (badge) {
        badge.textContent = view.badge;
        badge.hidden = view.badge === "";
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
    setStatus(value: unknown) {
      const message = statusMessage(value);
      if (message) {
        statuses = withStatus(statuses, message, performance.now());
        refresh();
      }
    },
    setVisible(visible: boolean) {
      statuses = withToolbarVisible(statuses, visible, performance.now());
      refresh();
    },
  };
}
