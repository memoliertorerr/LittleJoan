import { describe, expect, it } from "vitest";
import { SYSTEM_DEFAULT, browserChoices, browsersInfoFrom, defaultText, goesToNote, settingsNote, settingsPlace, showsSettingsButton, type BrowsersInfo } from "./browser-logic.ts";

const info = (over: Partial<BrowsersInfo> = {}): BrowsersInfo => ({
  platform: "macos",
  defaultKind: "other",
  defaultName: "Safari",
  browsers: [
    { id: "com.apple.Safari", name: "Safari" },
    { id: "org.mozilla.firefox", name: "Firefox" },
  ],
  mainBrowser: null,
  firstOther: "Safari",
  canOpenSettings: true,
  candidate: true,
  ...over,
});

describe("Settings' Browser section", () => {
  it("reads Rust's answer, field by field", () => {
    const raw = { platform: "macos", defaultKind: "self", defaultName: null, browsers: [{ id: "a", name: "A" }, { id: 3 }, "x"], mainBrowser: "a", firstOther: "A", canOpenSettings: true };
    expect(browsersInfoFrom(raw)).toEqual({ platform: "macos", defaultKind: "self", defaultName: null, browsers: [{ id: "a", name: "A" }], mainBrowser: "a", firstOther: "A", canOpenSettings: true, candidate: true });
    expect(browsersInfoFrom(null)).toEqual({ platform: "", defaultKind: "none", defaultName: null, browsers: [], mainBrowser: null, firstOther: null, canOpenSettings: false, candidate: true });
    expect(browsersInfoFrom({ candidate: false }).candidate).toBe(false);
    expect(browsersInfoFrom({ defaultKind: "maybe" }).defaultKind).toBe("none");
  });

  it("says whether Little Joan is the default", () => {
    expect(defaultText(info({ defaultKind: "self" }))).toBe("Little Joan is your default browser");
    expect(defaultText(info())).toBe("Little Joan isn't your default browser (Safari is)");
    expect(defaultText(info({ defaultName: null }))).toBe("Little Joan isn't your default browser");
    expect(defaultText(info({ defaultKind: "none" }))).toContain("couldn't be read");
  });

  it("names where each system chooses it, and never promises to change it", () => {
    expect(settingsPlace("macos")).toContain("Desktop & Dock");
    expect(settingsPlace("windows")).toContain("Default apps");
    expect(settingsNote(info())).toContain("never changes it by itself");
    const noSettings = settingsNote(info({ platform: "linux", canOpenSettings: false }));
    expect(noSettings).toContain('xdg-settings set default-web-browser "Little Joan.desktop"');
    expect(noSettings).toContain("yourself");
    expect(showsSettingsButton(info())).toBe(true);
    expect(showsSettingsButton(info({ canOpenSettings: false }))).toBe(false);
  });

  it("tells an AppImage user it can't be the system's browser, without the button", () => {
    const appImage = info({ platform: "linux", candidate: false });
    expect(settingsNote(appImage)).toContain("The AppImage can't be your system's browser");
    expect(showsSettingsButton(appImage)).toBe(false);
  });

  it("lists System default first, and keeps a saved browser that is gone", () => {
    expect(browserChoices(info()).map((c) => c.id)).toEqual([SYSTEM_DEFAULT, "com.apple.Safari", "org.mozilla.firefox"]);
    expect(browserChoices(info({ mainBrowser: "gone" })).at(-1)).toEqual({ id: "gone", name: "gone (not installed)" });
  });

  it("says where pages go with the choice shown", () => {
    expect(goesToNote(info(), "org.mozilla.firefox")).toContain("go to Firefox");
    expect(goesToNote(info(), SYSTEM_DEFAULT)).toContain("go to Safari");
    expect(goesToNote(info({ defaultKind: "none", defaultName: null }), SYSTEM_DEFAULT)).toContain("your default browser");
    expect(goesToNote(info({ defaultKind: "self" }), SYSTEM_DEFAULT)).toBe("Opening pages in Safari, because Little Joan is your default browser.");
    expect(goesToNote(info({ defaultKind: "self", browsers: [], firstOther: null }), SYSTEM_DEFAULT)).toContain("stay in Little Joan");
    expect(goesToNote(info({ defaultKind: "self" }), "gone")).toBe("That browser isn't installed any more. Opening pages in Safari, because Little Joan is your default browser.");
  });
});
