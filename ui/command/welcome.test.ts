import { describe, expect, it } from "vitest";
import { DEFAULT_THEME } from "../lib/theme.ts";
import { lineText, welcome, type StartInfo } from "./welcome.ts";

const mac: StartInfo = {
  shortcut: "CommandOrControl+Shift+Space",
  platform: "macos",
  firstRun: true,
  trayHint: "Little Joan's icon is in the menu bar at the top of the screen.",
  startAtLogin: false,
  waylandNoShortcut: false,
  portalShortcut: null,
  theme: DEFAULT_THEME,
};

describe("the welcome", () => {
  it("appears only on the first launch's summon", () => {
    expect(welcome({ ...mac, firstRun: false })).toBeNull();
    expect(welcome(mac)?.title).toBe("Little Joan is ready.");
  });

  it("says what the shortcut does, in this OS's notation", () => {
    const line = welcome(mac)?.shortcut;
    expect(line?.kind).toBe("keys");
    expect(line && lineText(line)).toBe("⌘⇧Space summons and dismisses this window.");
    const linux = welcome({ ...mac, platform: "linux" })?.shortcut;
    expect(linux && lineText(linux)).toBe("Ctrl+Shift+Space summons and dismisses this window.");
  });

  it("asks for a desktop binding under Wayland instead", () => {
    const line = welcome({ ...mac, platform: "linux", waylandNoShortcut: true })?.shortcut;
    expect(line?.kind).toBe("command");
    expect(line?.code).toBe("little-joan --toggle");
    expect(line && lineText(line)).toContain("Bind little-joan --toggle to a key in your keyboard settings");
  });

  it("names the desktop's key when the portal holds the shortcut (LJ17)", () => {
    const portal = { ...mac, platform: "linux", shortcut: null, portalShortcut: { trigger: "Press <ctrl><shift>space" } };
    const line = welcome(portal)?.shortcut;
    expect(line?.kind).toBe("keys");
    expect(line?.code).toBe("Press <ctrl><shift>space");
    expect(line && lineText(line)).toBe("Your desktop's shortcut summons and dismisses this window: Press <ctrl><shift>space.");
    const keyless = welcome({ ...portal, portalShortcut: { trigger: null } })?.shortcut;
    expect(keyless?.kind).toBe("none");
    expect(keyless && lineText(keyless)).toContain("in your desktop's keyboard settings");
    expect(keyless && lineText(keyless)).not.toContain("--toggle");
  });

  it("points at Settings when no shortcut could be registered", () => {
    const line = welcome({ ...mac, shortcut: null })?.shortcut;
    expect(line?.kind).toBe("none");
    expect(line && lineText(line)).toContain("Settings");
  });

  it("passes on the tray hint and the login entry, and says how to close", () => {
    const shown = welcome({ ...mac, startAtLogin: true });
    expect(shown?.tray).toBe(mac.trayHint);
    expect(shown?.startAtLogin).toBe(true);
    expect(shown?.close).toBe("Esc or the shortcut closes this window.");
    expect(welcome({ ...mac, startAtLogin: null })?.startAtLogin).toBeNull();
  });
});
