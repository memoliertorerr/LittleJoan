// The links run's readers (LJ45): what each OS's registration of Little Joan as a browser says,
// parsed from the tools' own output — `reg query /s` (Windows), `plutil` and Launch Services
// through a Swift snippet (macOS), desktop entries and mimeapps.list files (Linux) — and how the OS
// would launch a link from it: the ProgId's command with `%1`, a desktop entry's `Exec` with `%u`.
// Pure and unit-tested (smoke-links-read.test.ts); smoke-links.mts runs the tools.

/** Little Joan's names in the registrations (src-tauri/windows/hooks.nsh, Info.plist, the template). */
export const PROGID = "LittleJoanURL";
export const CLIENT_KEY = "HKEY_CURRENT_USER\\Software\\Clients\\StartMenuInternet\\LittleJoan";
export const PROGID_KEY = `HKEY_CURRENT_USER\\Software\\Classes\\${PROGID}`;
export const REGISTERED_KEY = "HKEY_CURRENT_USER\\Software\\RegisteredApplications";
export const USER_CHOICE = (scheme: string) =>
  `HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\Shell\\Associations\\UrlAssociations\\${scheme}\\UserChoice`;
export const BUNDLE_ID = "io.github.memoliertorerr.littlejoan";
/** Tauri's desktop file (named after the product), which the template makes a web browser. */
export const DESKTOP_ID = "Little Joan.desktop";
/** LJ17's desktop file for the portals: hidden, and no browser. */
export const PORTAL_DESKTOP_ID = `${BUNDLE_ID}.desktop`;

/** `reg query <key> /s` → key → value name → data (`(Default)` for a key's default value). */
export function parseRegTree(output: string): Record<string, Record<string, string>> {
  const tree: Record<string, Record<string, string>> = {};
  let key: string | undefined;
  for (const line of output.split(/\r?\n/)) {
    if (/^HKEY_/.test(line)) {
      key = line.trim();
      tree[key] ??= {};
      continue;
    }
    const value = /^ {4}(.+?) {4}(REG_\w+)(?: {4}(.*))?$/.exec(line);
    if (key && value?.[1]) {
      (tree[key] ??= {})[value[1]] = value[3] ?? "";
    }
  }
  return tree;
}

/** A Windows path for comparing: backslashes, lower case. */
const winPath = (path: string) => path.replace(/\//g, "\\").toLowerCase();

/** What the Windows registration lacks (hooks.nsh), against the installed `app`. */
export function windowsRegistrationFailures(tree: Record<string, Record<string, string>>, app: string): string[] {
  const failures: string[] = [];
  const want = (key: string, name: string, test: (data: string) => boolean, what: string) => {
    const data = tree[key]?.[name];
    if (data === undefined || !test(data)) {
      failures.push(`links: ${key} ${name} is ${data === undefined ? "missing" : JSON.stringify(data)}, not ${what}`);
    }
  };
  const exe = winPath(app);
  want(CLIENT_KEY, "(Default)", (d) => d === "Little Joan", '"Little Joan"');
  want(`${CLIENT_KEY}\\Capabilities`, "ApplicationName", (d) => d === "Little Joan", '"Little Joan"');
  want(`${CLIENT_KEY}\\Capabilities`, "ApplicationIcon", (d) => winPath(d) === `${exe},0`, "the app's icon");
  for (const scheme of ["http", "https"]) {
    want(`${CLIENT_KEY}\\Capabilities\\URLAssociations`, scheme, (d) => d === PROGID, PROGID);
  }
  want(`${CLIENT_KEY}\\shell\\open\\command`, "(Default)", (d) => winPath(d) === `"${exe}"`, "the quoted app");
  want(REGISTERED_KEY, "Little Joan", (d) => d === "Software\\Clients\\StartMenuInternet\\LittleJoan\\Capabilities", "its Capabilities key");
  want(PROGID_KEY, "(Default)", (d) => d === "Little Joan URL", '"Little Joan URL"');
  want(`${PROGID_KEY}\\shell\\open\\command`, "(Default)", (d) => winPath(d) === `"${exe}" --link "%1"`, '"<app>" --link "%1"');
  return failures;
}

/**
 * How Explorer runs a ProgId's command for `url`: `%1` replaced, then the program (quoted or not),
 * the program as written (`argv0`) and the rest of the line — the caller hands the line to
 * CreateProcess as it is (node's windowsVerbatimArguments: `argv0` + " " + `rest`).
 */
export function registeredLaunch(command: string, url: string): { program: string; argv0: string; rest: string } | undefined {
  const expanded = command.replace(/%1/g, url).trim();
  const quoted = /^"([^"]+)"/.exec(expanded);
  const argv0 = quoted ? quoted[0] : (expanded.split(/\s+/)[0] ?? "");
  const program = quoted ? quoted[1] : argv0;
  return program ? { program, argv0, rest: expanded.slice(argv0.length).trim() } : undefined;
}

/** A desktop entry's `[Desktop Entry]` group: key → value (later keys win, other groups ignored). */
export function parseDesktopEntry(text: string): Record<string, string> {
  const entry: Record<string, string> = {};
  let inGroup = false;
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (line.startsWith("[")) {
      inGroup = line === "[Desktop Entry]";
    } else if (inGroup && line && !line.startsWith("#")) {
      const at = line.indexOf("=");
      if (at > 0) {
        entry[line.slice(0, at).trim()] = line.slice(at + 1).trim();
      }
    }
  }
  return entry;
}

/** A `;`-separated desktop entry list (MimeType, Categories). */
export const listOf = (value: string | undefined): string[] => (value ?? "").split(";").filter(Boolean);

/**
 * A desktop entry's `Exec` as arguments for `url` (the Desktop Entry spec): double quotes group,
 * a backslash escapes `"`, `` ` ``, `$` and `\` inside them; `%u` / `%U` / `%f` / `%F` become the
 * URL, `%%` a percent sign, the other field codes go. Undefined for an unbalanced quote or a URL
 * code inside a quoted argument (the spec forbids it).
 */
export function expandExec(exec: string, url: string): string[] | undefined {
  const words: { text: string; quoted: boolean }[] = [];
  let current: { text: string; quoted: boolean } | null = null;
  let quoting = false;
  for (let i = 0; i < exec.length; i += 1) {
    const c = exec[i] as string;
    if (quoting) {
      if (c === "\\" && i + 1 < exec.length && '"`$\\'.includes(exec[i + 1] as string)) {
        current = current ?? { text: "", quoted: true };
        current.text += exec[i + 1];
        i += 1;
      } else if (c === '"') {
        quoting = false;
      } else {
        current = current ?? { text: "", quoted: true };
        current.text += c;
      }
    } else if (c === '"') {
      quoting = true;
      current = current ?? { text: "", quoted: true };
      current.quoted = true;
    } else if (c === " " || c === "\t") {
      if (current) {
        words.push(current);
        current = null;
      }
    } else {
      current = current ?? { text: "", quoted: false };
      current.text += c;
    }
  }
  if (quoting) {
    return undefined;
  }
  if (current) {
    words.push(current);
  }
  const args: string[] = [];
  for (const word of words) {
    if (word.quoted) {
      if (/%[uUfF]/.test(word.text)) {
        return undefined;
      }
      args.push(word.text);
    } else if (/^%[uUfF]$/.test(word.text)) {
      args.push(url);
    } else if (/^%[ick]$/.test(word.text)) {
      continue;
    } else {
      args.push(word.text.replace(/%%/g, "%").replace(/%[a-zA-Z]/g, ""));
    }
  }
  return args.length > 0 ? args : undefined;
}

/** What Little Joan's desktop entry lacks as a web browser (the template). */
export function desktopEntryFailures(entry: Record<string, string>, where: string): string[] {
  const failures: string[] = [];
  const exec = expandExec(entry.Exec ?? "", "URL");
  if (entry.Type !== "Application") {
    failures.push(`links: ${where} Type is ${entry.Type ?? "missing"}`);
  }
  if (!exec || exec.length !== 3 || !/(^|\/)little-joan$/.test(exec[0] ?? "") || exec[1] !== "--link" || exec[2] !== "URL") {
    failures.push(`links: ${where} Exec is ${JSON.stringify(entry.Exec ?? "")}, not "little-joan --link %u"`);
  }
  const mime = listOf(entry.MimeType);
  for (const type of ["x-scheme-handler/http", "x-scheme-handler/https"]) {
    if (!mime.includes(type)) {
      failures.push(`links: ${where} MimeType lacks ${type}`);
    }
  }
  const categories = listOf(entry.Categories);
  if (!categories.includes("Network") || !categories.includes("WebBrowser")) {
    failures.push(`links: ${where} Categories is ${JSON.stringify(entry.Categories ?? "")}, not Network;WebBrowser;`);
  }
  if (entry.NoDisplay === "true" || entry.Hidden === "true") {
    failures.push(`links: ${where} is hidden (NoDisplay/Hidden)`);
  }
  return failures;
}

/**
 * What the AppImage's own desktop entry must stay (src-tauri/linux/little-joan-appimage.desktop):
 * a plain `Exec=little-joan` and no web MIME types. Its AppRun puts the entry's Exec arguments in
 * front of every launch's own, so `--link %u` there would turn `--show` into a refused link.
 */
export function appImageEntryFailures(entry: Record<string, string>, where: string): string[] {
  const failures: string[] = [];
  const exec = expandExec(entry.Exec ?? "", "URL");
  if (!exec || exec.length !== 1 || !/(^|\/)little-joan$/.test(exec[0] ?? "")) {
    failures.push(`links: ${where} Exec is ${JSON.stringify(entry.Exec ?? "")}, not a plain "little-joan" (the AppImage's AppRun prepends it to every launch)`);
  }
  if (listOf(entry.MimeType).some((type) => type.startsWith("x-scheme-handler/http"))) {
    failures.push(`links: ${where} claims web links, but an AppImage installs nothing to receive them`);
  }
  return failures;
}

/**
 * The defaults a mimeapps.list (or defaults.list) sets for web links: its `[Default Applications]`
 * entries for x-scheme-handler/http and /https.
 */
export function webDefaults(text: string): Record<string, string[]> {
  const found: Record<string, string[]> = {};
  let inGroup = false;
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (line.startsWith("[")) {
      inGroup = line === "[Default Applications]";
    } else if (inGroup) {
      const match = /^(x-scheme-handler\/https?)\s*=\s*(.*)$/.exec(line);
      if (match?.[1]) {
        found[match[1]] = listOf(match[2]);
      }
    }
  }
  return found;
}

/** The files among `lists` whose defaults name Little Joan for web links ("<path>: <type>"). */
export function defaultsNamingUs(lists: readonly { path: string; text: string }[]): string[] {
  return lists.flatMap(({ path, text }) =>
    Object.entries(webDefaults(text))
      .filter(([, ids]) => ids.includes(DESKTOP_ID) || ids.includes(PORTAL_DESKTOP_ID))
      .map(([type]) => `${path}: ${type}`),
  );
}

/** macOS: the URL types in the app's Info.plist (`plutil -extract CFBundleURLTypes json`). */
export function macUrlTypesFailures(urlTypes: string, documentTypesExtracted: boolean): string[] {
  const failures: string[] = [];
  let types: unknown;
  try {
    types = JSON.parse(urlTypes);
  } catch {
    types = undefined;
  }
  const entries = Array.isArray(types) ? (types as Record<string, unknown>[]) : [];
  const web = entries.find((entry) => {
    const schemes = Array.isArray(entry.CFBundleURLSchemes) ? entry.CFBundleURLSchemes : [];
    return schemes.includes("http") && schemes.includes("https");
  });
  if (!web) {
    failures.push(`links: the app's Info.plist has no CFBundleURLTypes for http and https (${urlTypes.trim() || "none"})`);
  } else if (web.CFBundleTypeRole !== "Viewer") {
    failures.push(`links: the web URL types' role is ${String(web.CFBundleTypeRole)}, not Viewer`);
  }
  if (documentTypesExtracted) {
    failures.push("links: the app's Info.plist declares CFBundleDocumentTypes (Little Joan never opens files)");
  }
  return failures;
}

/** What Launch Services says (MAC_HANDLERS' JSON): per scheme, every handler and the default. */
export interface MacHandlers {
  http: { all: string[]; default: string | null };
  https: { all: string[]; default: string | null };
  workspace: string[];
}

export function parseMacHandlers(text: string): MacHandlers | undefined {
  try {
    const value = JSON.parse(text.trim().split("\n").at(-1) ?? "") as Partial<MacHandlers>;
    const scheme = (s: unknown) => {
      const v = s as { all?: unknown; default?: unknown } | undefined;
      return {
        all: Array.isArray(v?.all) ? v.all.filter((x): x is string => typeof x === "string") : [],
        default: typeof v?.default === "string" ? v.default : null,
      };
    };
    return { http: scheme(value.http), https: scheme(value.https), workspace: Array.isArray(value.workspace) ? value.workspace.filter((x): x is string => typeof x === "string") : [] };
  } catch {
    return undefined;
  }
}

/** Whether a bundle id is Little Joan's (Launch Services may lower-case them). */
export const isOurBundle = (id: string | null | undefined): boolean => (id ?? "").toLowerCase() === BUNDLE_ID;

/** Read-only: Launch Services' handlers for http and https, and NSWorkspace's apps for a web URL. */
export const MAC_HANDLERS = `import AppKit
import CoreServices
func all(_ scheme: String) -> [String] {
  (LSCopyAllHandlersForURLScheme(scheme as CFString)?.takeRetainedValue() as? [String]) ?? []
}
func current(_ scheme: String) -> Any {
  (LSCopyDefaultHandlerForURLScheme(scheme as CFString)?.takeRetainedValue() as String?) ?? NSNull()
}
var out: [String: Any] = [:]
for scheme in ["http", "https"] { out[scheme] = ["all": all(scheme), "default": current(scheme)] }
out["workspace"] = NSWorkspace.shared.urlsForApplications(toOpen: URL(string: "https://example.com")!).compactMap { Bundle(url: $0)?.bundleIdentifier }
print(String(data: try! JSONSerialization.data(withJSONObject: out), encoding: .utf8)!)
`;
