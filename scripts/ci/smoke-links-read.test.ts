import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  CLIENT_KEY,
  DESKTOP_ID,
  PROGID_KEY,
  REGISTERED_KEY,
  appImageEntryFailures,
  defaultsNamingUs,
  desktopEntryFailures,
  expandExec,
  isOurBundle,
  macUrlTypesFailures,
  parseDesktopEntry,
  parseMacHandlers,
  parseRegTree,
  registeredLaunch,
  webDefaults,
  windowsRegistrationFailures,
} from "./smoke-links-read.mts";

const root = join(import.meta.dirname, "..", "..");
const APP = "C:\\Users\\runneradmin\\AppData\\Local\\Little Joan\\little-joan.exe";

/** `reg query /s` as Windows prints the keys hooks.nsh writes. */
const REG = [
  "",
  CLIENT_KEY,
  "    (Default)    REG_SZ    Little Joan",
  "",
  `${CLIENT_KEY}\\Capabilities`,
  "    ApplicationName    REG_SZ    Little Joan",
  "    ApplicationDescription    REG_SZ    A tiny pop-up browser that hibernates until you call it.",
  `    ApplicationIcon    REG_SZ    ${APP},0`,
  "",
  `${CLIENT_KEY}\\Capabilities\\URLAssociations`,
  "    http    REG_SZ    LittleJoanURL",
  "    https    REG_SZ    LittleJoanURL",
  "",
  `${CLIENT_KEY}\\DefaultIcon`,
  `    (Default)    REG_SZ    ${APP},0`,
  "",
  `${CLIENT_KEY}\\shell\\open\\command`,
  `    (Default)    REG_SZ    "${APP}"`,
  "",
  REGISTERED_KEY,
  "    Little Joan    REG_SZ    Software\\Clients\\StartMenuInternet\\LittleJoan\\Capabilities",
  "",
  PROGID_KEY,
  "    (Default)    REG_SZ    Little Joan URL",
  "",
  `${PROGID_KEY}\\shell\\open\\command`,
  `    (Default)    REG_SZ    "${APP}" --link "%1"`,
  "",
].join("\r\n");

describe("the Windows registration", () => {
  it("reads reg query's keys and values", () => {
    const tree = parseRegTree(REG);
    expect(tree[CLIENT_KEY]?.["(Default)"]).toBe("Little Joan");
    expect(tree[`${CLIENT_KEY}\\Capabilities\\URLAssociations`]).toEqual({ http: "LittleJoanURL", https: "LittleJoanURL" });
    expect(parseRegTree("    Orphan    REG_SZ    x\r\n")).toEqual({});
    expect(parseRegTree(`${CLIENT_KEY}\r\n    Empty    REG_SZ\r\n`)[CLIENT_KEY]).toEqual({ Empty: "" });
  });

  it("passes what hooks.nsh writes, whatever the path's case", () => {
    expect(windowsRegistrationFailures(parseRegTree(REG), APP)).toEqual([]);
    expect(windowsRegistrationFailures(parseRegTree(REG), APP.toUpperCase().replace(/\\/g, "/"))).toEqual([]);
  });

  it("names what is missing or wrong", () => {
    const unquoted = REG.replace(`"${APP}" --link "%1"`, `"${APP}" "%1"`);
    expect(windowsRegistrationFailures(parseRegTree(unquoted), APP).join("\n")).toContain('--link "%1"');
    const noHttps = REG.replace("    https    REG_SZ    LittleJoanURL\r\n", "");
    expect(windowsRegistrationFailures(parseRegTree(noHttps), APP).join("\n")).toContain("https is missing");
    expect(windowsRegistrationFailures({}, APP)).toHaveLength(9);
  });

  it("matches hooks.nsh itself", () => {
    const hooks = readFileSync(join(root, "src-tauri", "windows", "hooks.nsh"), "utf8");
    expect(hooks).toContain('--link $\\"%1$\\""');
    expect(hooks).toContain('"Software\\Clients\\StartMenuInternet\\LittleJoan"');
    expect(hooks).toContain('"Software\\Classes\\LittleJoanURL"');
    // Never the schemes' own keys, never the user's choice.
    expect(hooks).not.toMatch(/Classes\\https?"/);
    expect(hooks).not.toMatch(/UserChoice"|UserChoice\\/);
    // The uninstaller removes all three.
    const uninstall = hooks.slice(hooks.indexOf("NSIS_HOOK_POSTUNINSTALL"));
    // Only when the ProgId is still this install's own (phase 21's security review).
    expect(uninstall).toContain('ReadRegStr $R7 HKCU "${LJ_PROGID_KEY}\\shell\\open\\command" ""');
    expect(uninstall).toContain('${If} $R7 == "$\\"$INSTDIR\\${MAINBINARYNAME}.exe$\\" --link $\\"%1$\\""');
    for (const what of ["DeleteRegKey HKCU \"${LJ_CLIENT_KEY}\"", 'DeleteRegValue HKCU "${LJ_REGISTERED_KEY}" "${PRODUCTNAME}"', 'DeleteRegKey HKCU "${LJ_PROGID_KEY}"']) {
      expect(uninstall).toContain(what);
    }
  });

  it("launches a link with the registered command line, as Explorer would", () => {
    const launch = registeredLaunch(`"${APP}" --link "%1"`, "http://127.0.0.1:8123/smoke.html?link=os");
    expect(launch).toEqual({ program: APP, argv0: `"${APP}"`, rest: '--link "http://127.0.0.1:8123/smoke.html?link=os"' });
    expect(registeredLaunch("C:\\x.exe --link %1", "u")).toEqual({ program: "C:\\x.exe", argv0: "C:\\x.exe", rest: "--link u" });
    expect(registeredLaunch("", "u")).toBeUndefined();
  });
});

describe("desktop entries", () => {
  // Windows' checkout may turn the file's line ends into CRLF.
  const template = readFileSync(join(root, "src-tauri", "linux", "little-joan.desktop.hbs"), "utf8").replace(/\r\n/g, "\n");
  // As Tauri renders the template (name, exec, icon, comment).
  const rendered = template
    .replace("{{name}}", "Little Joan")
    .replace(/\{\{#if comment\}\}\n(.*)\n\{\{\/if\}\}\n/, "$1\n")
    .replace("{{comment}}", "A tiny pop-up browser that hibernates until you call it.")
    .replace(/\{\{exec\}\}/g, "little-joan")
    .replace("{{icon}}", "little-joan");

  it("reads the [Desktop Entry] group only", () => {
    expect(parseDesktopEntry("[Desktop Entry]\nName=A\n# c\n[Desktop Action x]\nName=B\n")).toEqual({ Name: "A" });
  });

  it("passes the template as Tauri renders it", () => {
    const entry = parseDesktopEntry(rendered);
    expect(entry.Exec).toBe("little-joan --link %u");
    expect(desktopEntryFailures(entry, "x")).toEqual([]);
    expect(rendered).not.toContain("{{");
  });

  it("names a desktop entry that isn't a browser", () => {
    const plain = parseDesktopEntry("[Desktop Entry]\nType=Application\nExec=little-joan\nCategories=Utility;\n");
    const text = desktopEntryFailures(plain, "x").join("\n");
    expect(text).toContain("Exec");
    expect(text).toContain("x-scheme-handler/https");
    expect(text).toContain("Categories");
    expect(desktopEntryFailures({ ...parseDesktopEntry(rendered), NoDisplay: "true" }, "x").join("\n")).toContain("hidden");
  });

  it("keeps the AppImage's own entry plain (its AppRun prepends Exec's arguments to every launch)", () => {
    const appImage = parseDesktopEntry(readFileSync(join(root, "src-tauri", "linux", "little-joan-appimage.desktop"), "utf8").replace(/\r\n/g, "\n"));
    expect(appImage.Exec).toBe("little-joan");
    expect(appImageEntryFailures(appImage, "x")).toEqual([]);
    const both = appImageEntryFailures(parseDesktopEntry(rendered), "x").join("\n");
    expect(both).toContain("plain");
    expect(both).toContain("claims web links");
  });

  it("expands Exec as the spec says", () => {
    expect(expandExec("little-joan --link %u", "https://x.test/a b")).toEqual(["little-joan", "--link", "https://x.test/a b"]);
    expect(expandExec('"/opt/my app/run" %U --x', "u")).toEqual(["/opt/my app/run", "u", "--x"]);
    expect(expandExec('sh -c "echo \\"hi\\" \\$HOME" %f', "u")).toEqual(["sh", "-c", 'echo "hi" $HOME', "u"]);
    expect(expandExec("app %i %c %k --pct=%%", "u")).toEqual(["app", "--pct=%"]);
    expect(expandExec('app "%u"', "u")).toBeUndefined();
    expect(expandExec('app "open', "u")).toBeUndefined();
    expect(expandExec("", "u")).toBeUndefined();
  });
});

describe("Linux defaults", () => {
  it("reads [Default Applications] for web links only", () => {
    const list = "[Added Associations]\nx-scheme-handler/https=a.desktop;\n[Default Applications]\nx-scheme-handler/https=firefox.desktop;b.desktop\ntext/html=c.desktop\n";
    expect(webDefaults(list)).toEqual({ "x-scheme-handler/https": ["firefox.desktop", "b.desktop"] });
  });

  it("finds a list that makes Little Joan the default", () => {
    const ours = `[Default Applications]\nx-scheme-handler/http=${DESKTOP_ID}\n`;
    expect(defaultsNamingUs([{ path: "/a", text: ours }, { path: "/b", text: "[Default Applications]\nx-scheme-handler/https=firefox.desktop\n" }])).toEqual([
      "/a: x-scheme-handler/http",
    ]);
    expect(defaultsNamingUs([{ path: "/c", text: `[Added Associations]\nx-scheme-handler/https=${DESKTOP_ID}\n` }])).toEqual([]);
  });
});

describe("macOS", () => {
  const plist = readFileSync(join(root, "src-tauri", "Info.plist"), "utf8");

  it("declares http and https as a viewer, and no documents", () => {
    expect(plist).toMatch(/<string>http<\/string>\s*<string>https<\/string>/);
    expect(plist).toContain("<string>Viewer</string>");
    expect(plist).not.toContain("CFBundleDocumentTypes");
    const json = JSON.stringify([{ CFBundleURLName: "Web site URL", CFBundleURLSchemes: ["http", "https"], CFBundleTypeRole: "Viewer" }]);
    expect(macUrlTypesFailures(json, false)).toEqual([]);
    expect(macUrlTypesFailures(json, true)[0]).toContain("CFBundleDocumentTypes");
    expect(macUrlTypesFailures("", false)[0]).toContain("no CFBundleURLTypes");
    expect(macUrlTypesFailures(json.replace("Viewer", "Editor"), false)[0]).toContain("Viewer");
  });

  it("reads Launch Services' answer", () => {
    const answer = '{"http":{"all":["com.apple.Safari","io.github.memoliertorerr.littlejoan"],"default":"com.apple.safari"},"https":{"all":["IO.GITHUB.MEMOLIERTORERR.LITTLEJOAN"],"default":null},"workspace":["com.apple.Safari"]}';
    const handlers = parseMacHandlers(`warning: deprecated\n${answer}\n`);
    expect(handlers?.http.default).toBe("com.apple.safari");
    expect(handlers?.https.default).toBeNull();
    expect(handlers?.https.all.some(isOurBundle)).toBe(true);
    expect(isOurBundle(handlers?.http.default)).toBe(false);
    expect(parseMacHandlers("not json")).toBeUndefined();
  });
});
