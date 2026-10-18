// Settings → General's "Phone or desktop per site" (LJ40; pure, unit-tested): the sites with a
// mode of their own — the user's entries (settings.json's siteModes) and the ones built in on this
// OS (Rust's get_settings: YouTube on the desktop site on macOS and Linux) — each with Remove, or
// Reset for a built-in the user overrode. The list is a draft that Save sends.

export type SiteMode = "auto" | "phone" | "desktop";

/** A site built in on this OS (Rust's `builtInSiteModes`). */
export interface BuiltInSite {
  host: string;
  mode: SiteMode;
}

/** One line of the list. */
export interface SiteRow {
  host: string;
  /** "Desktop", "Phone" or "Automatic". */
  mode: string;
  /** What else it says: "built in on macOS and Linux", "built in: Desktop", or "". */
  note: string;
  /** Remove (an entry, or a built-in not overridden: it becomes "auto") or Reset (back to the built-in). */
  action: "remove" | "reset";
}

const NAMES: Record<SiteMode, string> = { auto: "Automatic", phone: "Phone", desktop: "Desktop" };

/** Where the built-ins apply (they are only sent where they do). */
export const BUILT_IN_WHERE = "built in on macOS and Linux";

function isMode(value: unknown): value is SiteMode {
  return value === "auto" || value === "phone" || value === "desktop";
}

/** settings.json's siteModes as the window got it (anything odd dropped; Rust checks it again). */
export function siteModesFrom(value: unknown): Record<string, SiteMode> {
  const out: Record<string, SiteMode> = {};
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return out;
  }
  for (const [host, mode] of Object.entries(value as Record<string, unknown>)) {
    if (host && isMode(mode)) {
      out[host] = mode;
    }
  }
  return out;
}

/** Rust's built-in list, checked. */
export function builtInsFrom(value: unknown): BuiltInSite[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.flatMap((item: unknown) => {
    const raw = typeof item === "object" && item !== null ? (item as Record<string, unknown>) : {};
    return typeof raw.host === "string" && raw.host && isMode(raw.mode) ? [{ host: raw.host, mode: raw.mode }] : [];
  });
}

/**
 * The list: every entry (sorted by site) and every built-in without one. A built-in the user
 * overrode shows the user's mode with "built in: <mode>" and Reset; one not overridden says where
 * it is built in and has Remove.
 */
export function siteRows(entries: Readonly<Record<string, SiteMode>>, builtIns: readonly BuiltInSite[]): SiteRow[] {
  const builtIn = new Map(builtIns.map((site) => [site.host, site.mode]));
  const rows: SiteRow[] = Object.keys(entries)
    .sort()
    .map((host) => {
      const mode = entries[host] ?? "auto";
      const base = builtIn.get(host);
      return base === undefined
        ? { host, mode: NAMES[mode], note: "", action: "remove" }
        : { host, mode: NAMES[mode], note: `built in: ${NAMES[base]}`, action: "reset" };
    });
  for (const site of builtIns) {
    if (!(site.host in entries)) {
      rows.push({ host: site.host, mode: NAMES[site.mode], note: BUILT_IN_WHERE, action: "remove" });
    }
  }
  return rows;
}

/**
 * The draft after a row's button: Reset or Remove of an entry deletes it; Remove of a built-in
 * that has no entry keeps "auto" for it, so the width decides there again.
 */
export function afterAction(
  entries: Readonly<Record<string, SiteMode>>,
  builtIns: readonly BuiltInSite[],
  host: string,
): Record<string, SiteMode> {
  const next = { ...entries };
  if (host in next) {
    delete next[host];
  } else if (builtIns.some((site) => site.host === host)) {
    next[host] = "auto";
  }
  return next;
}

/** Whether two tables say the same. */
export function sameSites(a: Readonly<Record<string, SiteMode>>, b: Readonly<Record<string, SiteMode>>): boolean {
  const left = Object.keys(a).sort();
  const right = Object.keys(b).sort();
  return left.length === right.length && left.every((host, i) => host === right[i] && a[host] === b[host]);
}

/** A row as text: "youtube.com — Desktop, built in on macOS and Linux". */
export function rowText(row: SiteRow): string {
  return `${row.host} — ${row.mode}${row.note ? `, ${row.note}` : ""}`;
}

/** What the empty list says. */
export const NO_SITES = "No site has its own mode yet. With a page open, type “desktop” or “phone” in the command bar.";
