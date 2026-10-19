// The Little Arc-style toolbar's logic (pure, unit-tested): what it shows for the open page —
// the site's icon and domain in the middle (for an extension's panel: its name and icon), a back
// button on the left, "open in default browser" on the right, and between them a small "Phone" /
// "Desktop" pill while the page has the phone version or its site has a mode of its own (LJ40;
// a click toggles it for the site). toolbar.ts draws it.
import { ICON_URI } from "../command/extensions.ts";

/** Domains longer than this are shortened in the middle. */
export const MAX_DOMAIN = 32;

/** Whether `host` (a URL's hostname) is an IP address: shown as is. */
function isIp(host: string): boolean {
  return /^\d{1,3}(\.\d{1,3}){3}$/.test(host) || host.startsWith("[");
}

/** `text` shortened to `max` characters with an ellipsis in the middle. */
export function middleEllipsis(text: string, max = MAX_DOMAIN): string {
  const chars = Array.from(text);
  if (chars.length <= max) {
    return text;
  }
  const keep = max - 1;
  const head = Math.ceil(keep / 2);
  return `${chars.slice(0, head).join("")}…${chars.slice(chars.length - (keep - head)).join("")}`;
}

/** The domain the toolbar shows for `address`: `www.` stripped, IPs as is, long ones shortened. */
export function displayDomain(address: string): string {
  let host: string;
  try {
    host = new URL(address).hostname;
  } catch {
    return "";
  }
  if (!isIp(host)) {
    host = host.replace(/^www\./i, "");
  }
  return middleEllipsis(host);
}

/** The site whose icon the toolbar shows (as the command bar names sites: with a port). */
export function siteFor(address: string): string {
  try {
    const url = new URL(address);
    return isIp(url.hostname) ? url.host : url.host.replace(/^www\./i, "");
  } catch {
    return "";
  }
}

/** What the toolbar shows for an extension's panel (Rust sends it; extensionLabel checks it). */
export interface ExtensionLabel {
  name: string;
  /** A data: URI of an SVG or PNG image, or null. */
  icon: string | null;
}

/** The longest extension name shown. */
export const MAX_NAME = 40;

/** What an extension's panel shows next to its name: never just a name that could pass for a domain. */
export const EXTENSION_BADGE = "Extension";


/** The label Rust sent (anything: checked) — or null for a web page, nothing, or a bad label. */
export function extensionLabel(value: unknown): ExtensionLabel | null {
  if (typeof value !== "object" || value === null) {
    return null;
  }
  const raw = value as Record<string, unknown>;
  const name = typeof raw.name === "string" ? raw.name.trim() : "";
  if (!name || Array.from(name).length > MAX_NAME) {
    return null;
  }
  const icon = typeof raw.icon === "string" && raw.icon.length <= 100_000 && ICON_URI.test(raw.icon) ? raw.icon : null;
  return { name, icon };
}

/** What Rust says about the open web page's site (site_mode_info / setSiteMode). */
export interface SiteModeInfo {
  /** The site a mode is kept for (`youtube.com` for `m.youtube.com`). */
  host: string;
  /** The site's mode: the width decides (auto), or always phone / desktop. */
  mode: "auto" | "phone" | "desktop";
  /** The mode is built in (no entry of the user's applies). */
  builtIn: boolean;
  /** The agent the page has. */
  agent: "phone" | "desktop";
}

const MODES = ["auto", "phone", "desktop"] as const;
const AGENTS = ["phone", "desktop"] as const;

/** Rust's info, checked (anything else, or `null` for no web page, is null). */
export function siteModeInfo(value: unknown): SiteModeInfo | null {
  if (typeof value !== "object" || value === null) {
    return null;
  }
  const raw = value as Record<string, unknown>;
  const host = typeof raw.host === "string" ? raw.host : "";
  const mode = MODES.find((m) => m === raw.mode);
  const agent = AGENTS.find((a) => a === raw.agent);
  if (!host || host.length > 253 || !mode || !agent || typeof raw.builtIn !== "boolean") {
    return null;
  }
  return { host, mode, builtIn: raw.builtIn, agent };
}

/** The toolbar's pill: what it says, what a click asks for, and its tooltip. */
export interface Pill {
  label: "Phone" | "Desktop";
  next: "phone" | "desktop";
  title: string;
}

/**
 * The pill for the open page: shown only on a web page (never the start page or a panel) while
 * the page has the phone version or its site has a mode of its own (a built-in one too); a
 * desktop page in a wide window shows none. A click asks for the other version for the site.
 */
export function pillFor(info: SiteModeInfo | null, address: string, extension: ExtensionLabel | null): Pill | null {
  if (!info || extension || !/^https?:\/\//i.test(address)) {
    return null;
  }
  if (info.agent === "desktop" && info.mode === "auto") {
    return null;
  }
  const site = middleEllipsis(info.host, MAX_DOMAIN);
  return info.agent === "phone"
    ? { label: "Phone", next: "desktop", title: `Phone version of ${site}: click for the desktop version` }
    : { label: "Desktop", next: "phone", title: `Desktop version of ${site}: click for the phone version` };
}

export interface ToolbarView {
  /** Shown in the middle; "" with no page. For an extension, its name (drawn as text). */
  domain: string;
  /** Whose icon to show; "" for none. */
  site: string;
  /** An extension's own icon (a data: URI), or null. */
  icon: string | null;
  /** "Extension" for an extension's panel (shown as a badge next to its name), else "". */
  badge: string;
  /** The back button works: the page navigated within this window. */
  back: boolean;
  /** "Open in default browser" works: a web page is open. */
  openInBrowser: boolean;
  /** The phone / desktop pill, or null for none. */
  pill: Pill | null;
}

/**
 * The toolbar for the page at `address` ("" = none), or for an extension's panel (`extension`).
 * `canGoBack` comes from Rust, which counts this window's page loads and backs (back_stack.rs):
 * it is false in every new window. A panel can't go to the default browser.
 */
export function toolbarView(
  address: string,
  canGoBack: boolean,
  extension: ExtensionLabel | null = null,
  info: SiteModeInfo | null = null,
): ToolbarView {
  if (extension) {
    return {
      domain: middleEllipsis(extension.name),
      site: "",
      icon: extension.icon,
      badge: EXTENSION_BADGE,
      back: canGoBack,
      openInBrowser: false,
      pill: null,
    };
  }
  const web = /^https?:\/\//i.test(address);
  return {
    domain: web ? displayDomain(address) : "",
    site: web ? siteFor(address) : "",
    icon: null,
    badge: "",
    back: web && canGoBack,
    openInBrowser: web,
    pill: pillFor(info, address, null),
  };
}

/** The platform the bar runs on, from its user agent (macOS puts the traffic lights in the bar). */
export function isMac(userAgent: string): boolean {
  return /Macintosh|Mac OS X/.test(userAgent);
}

// The status area (LJ15): a short line where the pill sits — "Downloading report.pdf…", then
// "report.pdf saved" with "Show in folder", or "Download failed"; the page's zoom level, "Zoom
// 110 %" (LJ16) — pushed by Rust (window.littleJoan.setStatus). While it shows, the pill makes way.
// A finished download stays STATUS_LINGER_MS of the time the toolbar is visible, a zoom level
// ZOOM_LINGER_MS: opening the command bar never loses one, it shows when the toolbar comes back.
// Generic — text and an optional action — so later steps add their own kinds.

/** How long a finished status stays, in milliseconds the toolbar is visible. */
export const STATUS_LINGER_MS = 8_000;
/** How long a zoom level shows (LJ16). */
export const ZOOM_LINGER_MS = 1_500;
/** The longest file name shown in full (longer ones are shortened in the middle). */
export const MAX_STATUS_NAME = 40;

export type DownloadState = "downloading" | "saved" | "failed" | "blocked";

/** What Rust says happened (checked by `statusMessage`). */
export type StatusMessage = { kind: "download"; state: DownloadState; name: string } | { kind: "zoom"; percent: number };

/** What a status action asks Rust for: the only one is "Show in folder". */
export interface StatusAction {
  label: string;
  command: "reveal_download";
}

/** What the status area shows: its text (as text), a tooltip, and maybe a button. */
export interface StatusView {
  text: string;
  title: string;
  action: StatusAction | null;
}

const DOWNLOAD_STATES = ["downloading", "saved", "failed", "blocked"] as const;

/** Rust's message, checked — anything else is null. A file name is text from a web server. */
export function statusMessage(value: unknown): StatusMessage | null {
  if (typeof value !== "object" || value === null) {
    return null;
  }
  const raw = value as Record<string, unknown>;
  if (raw.kind === "download") {
    const state = DOWNLOAD_STATES.find((s) => s === raw.state);
    const name = typeof raw.name === "string" ? raw.name : null;
    if (!state || name === null || name.length > 1_000 || ((state === "downloading" || state === "saved") && !name)) {
      return null;
    }
    return { kind: "download", state, name };
  }
  if (raw.kind === "zoom") {
    const percent = raw.percent;
    return typeof percent === "number" && Number.isInteger(percent) && percent >= 1 && percent <= 1_000 ? { kind: "zoom", percent } : null;
  }
  return null;
}

/** The text, tooltip and button for `message`. */
export function statusView(message: StatusMessage): StatusView {
  if (message.kind === "zoom") {
    return { text: `Zoom ${message.percent} %`, title: `The page at ${message.percent} %`, action: null };
  }
  const name = middleEllipsis(message.name, MAX_STATUS_NAME);
  switch (message.state) {
    case "downloading":
      return { text: `Downloading ${name}…`, title: `Downloading ${message.name}…`, action: null };
    case "saved":
      return {
        text: `${name} saved`,
        title: `${message.name} saved in Downloads`,
        action: { label: "Show in folder", command: "reveal_download" },
      };
    case "failed":
      return { text: "Download failed", title: message.name ? `Download failed: ${message.name}` : "Download failed", action: null };
    case "blocked":
      return { text: "Download blocked", title: "Extensions can't save files to Downloads", action: null };
  }
}

/** How long `message` stays once the toolbar shows it (null: until another replaces it). */
export function lingerFor(message: StatusMessage): number | null {
  if (message.kind === "zoom") {
    return ZOOM_LINGER_MS;
  }
  return message.state === "downloading" ? null : STATUS_LINGER_MS;
}

interface StatusEntry {
  message: StatusMessage;
  /** Visible milliseconds left; null = until replaced. */
  remaining: number | null;
}

/** The statuses held (one per kind, latest last) and since when the toolbar is visible. */
export interface StatusState {
  entries: readonly StatusEntry[];
  /** When the toolbar became visible; null while it isn't (the command bar is open). */
  visibleSince: number | null;
}

/** No status yet; the toolbar visible or not (a new window opens with the command bar). */
export function initialStatus(visible: boolean, now: number): StatusState {
  return { entries: [], visibleSince: visible ? now : null };
}

/** The visible time since `visibleSince` taken off every entry, expired ones dropped. */
function settle(state: StatusState, now: number): StatusState {
  if (state.visibleSince === null) {
    return state;
  }
  const elapsed = Math.max(0, now - state.visibleSince);
  const entries = state.entries
    .map((entry) => (entry.remaining === null ? entry : { ...entry, remaining: entry.remaining - elapsed }))
    .filter((entry) => entry.remaining === null || entry.remaining > 0);
  return { entries, visibleSince: now };
}

/** A new message replaces the one of its kind and shows first. */
export function withStatus(state: StatusState, message: StatusMessage, now: number): StatusState {
  const settled = settle(state, now);
  const entries = settled.entries.filter((entry) => entry.message.kind !== message.kind);
  return { ...settled, entries: [...entries, { message, remaining: lingerFor(message) }] };
}

/** The toolbar was shown or hidden (the command bar closed or opened). */
export function withToolbarVisible(state: StatusState, visible: boolean, now: number): StatusState {
  const settled = settle(state, now);
  return { ...settled, visibleSince: visible ? now : null };
}

/** What the status area shows at `now`: the latest status that hasn't run out, or null. */
export function shownStatus(state: StatusState, now: number): StatusView | null {
  const latest = settle(state, now).entries.at(-1);
  return latest ? statusView(latest.message) : null;
}

/** Milliseconds until what is shown changes by itself (a status runs out), or null for never. */
export function nextStatusChange(state: StatusState, now: number): number | null {
  if (state.visibleSince === null) {
    return null;
  }
  const left = settle(state, now)
    .entries.map((entry) => entry.remaining)
    .filter((remaining): remaining is number => remaining !== null);
  return left.length > 0 ? Math.min(...left) : null;
}
