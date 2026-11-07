import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import {
  AUTOFILL_LABEL,
  AUTOFILL_NOTE,
  CLEAR_LOGINS_COMMAND,
  LOGINS_DEFAULTS,
  REMOVE_LABEL,
  REMOVE_NOTE,
  REMOVE_QUESTION,
  REMOVE_START,
  REMOVED_TEXT,
  SAVE_PASSWORDS_LABEL,
  SAVE_PASSWORDS_NOTE,
  loginsAvailable,
  loginsFrom,
  removeStep,
  storedWhere,
  unavailableNote,
} from "./logins-logic.ts";

describe("Settings → Privacy's saved passwords and form data (LJ51)", () => {
  it("offers the switches on Windows only", () => {
    expect(loginsAvailable("windows")).toBe(true);
    for (const platform of ["macos", "linux", "", "Windows"]) {
      expect(loginsAvailable(platform)).toBe(false);
    }
  });

  it("starts from the engine's own defaults, and reads only booleans", () => {
    expect(LOGINS_DEFAULTS).toEqual({ savePasswords: false, autofillForms: true });
    expect(loginsFrom({})).toEqual(LOGINS_DEFAULTS);
    expect(loginsFrom({ savePasswords: true, autofillForms: false })).toEqual({ savePasswords: true, autofillForms: false });
    expect(loginsFrom({ savePasswords: "yes", autofillForms: 0 })).toEqual(LOGINS_DEFAULTS);
  });

  it("is honest about where the data lives and who reads it", () => {
    expect(SAVE_PASSWORDS_LABEL).toBe("Offer to save passwords (kept by Windows' web engine, on this computer)");
    expect(AUTOFILL_LABEL).toBe("Fill in forms (addresses and the like)");
    expect(REMOVE_LABEL).toBe("Remove saved passwords and form data…");
    // Off stops new saving only: what was saved still fills in until it is removed (WebView2's rule).
    expect(SAVE_PASSWORDS_NOTE).toContain("passwords saved earlier still fill in until you remove them");
    expect(AUTOFILL_NOTE).toContain("nothing new is kept");
    expect(REMOVE_NOTE).toContain("Little Joan itself never reads, exports or syncs them");
    expect(REMOVE_NOTE).toContain("Cookies, site data and history stay");
    expect(storedWhere("windows")).toContain("Windows' web engine's own store");
    expect(storedWhere("windows")).toContain("Little Joan never reads them");
    expect(storedWhere("macos")).toContain("saves no passwords or form data");
    expect(storedWhere("linux")).toBe(storedWhere("macos"));
  });

  it("says why there is nothing to switch on macOS and Linux", () => {
    const title = "Saving passwords isn't available in Little Joan on this system yet.";
    expect(unavailableNote("macos")).toEqual({ title, why: "Apple lets only browsers with a special entitlement use Passwords and passkeys." });
    expect(unavailableNote("linux")).toEqual({ title, why: "WebKitGTK has no password manager." });
    expect(unavailableNote("freebsd").title).toBe(title);
  });

  it("asks before removing, and removes once", () => {
    let state = removeStep(REMOVE_START, { kind: "confirm" });
    expect(state).toEqual(REMOVE_START);
    state = removeStep(state, { kind: "ask" });
    expect(state).toEqual({ asking: true, busy: false, message: "" });
    expect(removeStep(state, { kind: "cancel" })).toEqual(REMOVE_START);
    state = removeStep(state, { kind: "confirm" });
    expect(state.busy).toBe(true);
    // A second confirm, an ask or a cancel while the engine works changes nothing.
    expect(removeStep(state, { kind: "confirm" })).toBe(state);
    expect(removeStep(state, { kind: "ask" })).toBe(state);
    expect(removeStep(state, { kind: "cancel" })).toBe(state);
    expect(removeStep(state, { kind: "done" })).toEqual({ asking: false, busy: false, message: REMOVED_TEXT });
    expect(removeStep(state, { kind: "failed", error: "Couldn't remove them: busy" })).toEqual({
      asking: false,
      busy: false,
      message: "Couldn't remove them: busy",
    });
    expect(REMOVE_QUESTION).toContain("can't be undone");
    expect(CLEAR_LOGINS_COMMAND).toBe("clear_saved_logins");
  });

  it("sits in the Privacy tab: the switches and the button for Windows, the note for the rest", () => {
    const html = readFileSync(resolve(import.meta.dirname, "..", "settings.html"), "utf8");
    const start = html.indexOf('<section id="panel-privacy"');
    const privacy = html.slice(start, html.indexOf("</section>", start));
    expect(start).toBeGreaterThan(-1);
    for (const id of ["logins-label", "logins-section", "save-passwords", "autofill-forms", "logins-remove", "logins-confirm", "logins-go", "logins-cancel", "logins-unavailable", "store-logins"]) {
      expect(privacy).toContain(`id="${id}"`);
    }
    // Both parts start hidden: get_settings' platform decides which shows.
    expect(privacy).toMatch(/<div id="logins-windows" hidden>/);
    expect(privacy).toMatch(/<div id="logins-unavailable" hidden>/);
    expect(privacy).toMatch(/<input id="save-passwords" type="checkbox"/);
    expect(privacy.indexOf('id="logins-go"')).toBeGreaterThan(privacy.indexOf('id="logins-remove"'));
  });
});
