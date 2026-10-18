// Extension commands in the command bar (pure, unit-tested): the rows for what is typed, from
// Rust's extensions_list(). A command matches when every typed word starts a word of its title,
// its keywords or its extension's name; it shows as "Title — Extension" (text only, view.ts). Page
// actions show only while a web page is open, and say "for this page". Switched-off extensions
// never show.
import type { Row } from "./rows.ts";

export interface ExtensionItem {
  id: string;
  title: string;
  keywords: string[];
}

/** One extension as extensions_list() describes it (src-tauri/src/extensions/list.rs). */
export interface ExtensionInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  kind: "bundled" | "developer";
  enabled: boolean;
  icon: string | null;
  permissions: string[];
  commands: ExtensionItem[];
  pageActions: ExtensionItem[];
}

export interface ExtensionsView {
  developerMode: boolean;
  folder: string;
  /** The built-in quick answers switched off (`builtin:<name>`), for the command bar. */
  disabledBuiltIns: string[];
  extensions: ExtensionInfo[];
  skipped: { name: string; reason: string }[];
}

/** Icons the bar shows: Rust's base64 data: URIs of SVG or PNG images, nothing else. */
export const ICON_URI = /^data:image\/(svg\+xml|png);base64,[A-Za-z0-9+/]+={0,2}$/;

/** The extension's icon if it is one the bar may show, else undefined. */
export function iconFor(extension: ExtensionInfo): string | undefined {
  return extension.icon !== null && extension.icon.length <= 100_000 && ICON_URI.test(extension.icon) ? extension.icon : undefined;
}

/** Lower-case words of `text` (letters and digits). */
function words(text: string): string[] {
  return text.toLowerCase().split(/[^\p{L}\p{N}]+/u).filter(Boolean);
}

/** Every typed word starts one of `haystack`'s words. */
export function matches(query: string, haystack: readonly string[]): boolean {
  const typed = words(query);
  if (typed.length === 0) {
    return false;
  }
  const have = haystack.flatMap(words);
  return typed.every((word) => have.some((w) => w.startsWith(word)));
}

/**
 * The extension rows for `query`: commands of switched-on extensions that match, then (only when
 * `webPage`) their page actions, in the extensions' order.
 */
export function commandRows(query: string, extensions: readonly ExtensionInfo[], webPage: boolean): Row[] {
  const rows: Row[] = [];
  for (const extension of extensions) {
    if (!extension.enabled) {
      continue;
    }
    const image = iconFor(extension);
    const add = (item: ExtensionItem, pageAction: boolean): void => {
      if (!matches(query, [item.title, ...item.keywords, extension.name])) {
        return;
      }
      rows.push({
        kind: "command",
        title: item.title,
        domain: pageAction ? `${extension.name}, for this page` : extension.name,
        host: "",
        icon: "extension",
        ...(image ? { image } : {}),
        action: { kind: "command", extension: extension.id, command: item.id, pageAction },
      });
    };
    for (const command of extension.commands) {
      add(command, false);
    }
    if (webPage) {
      for (const action of extension.pageActions) {
        add(action, true);
      }
    }
  }
  return rows;
}

/** Whether the bar's address is a web page (page actions need one). */
export function isWebPage(address: string): boolean {
  return /^https?:\/\//i.test(address);
}
