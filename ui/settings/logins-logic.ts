// Settings → Privacy's saved passwords and form data (LJ51; pure, tested). On Windows, WebView2 —
// the web engine Little Joan runs on there — can save passwords and fill in forms itself: two
// switches (Save / Cancel cover them like every setting) and "Remove saved passwords and form
// data…", which asks first and acts at once. macOS and Linux have no such switch in Little Joan: a
// note says why. Every text says where the data lives (the engine's own store, on this computer)
// and that Little Joan itself never reads it (Ground Rule 26).

/** The two switches, as settings.json names them. */
export interface LoginsInput {
  savePasswords: boolean;
  autofillForms: boolean;
}

/** The engine's own defaults (settings/logins.rs): no password saving, forms filled in. */
export const LOGINS_DEFAULTS: LoginsInput = { savePasswords: false, autofillForms: true };

/** The command the Remove button calls (capabilities/settings.json grants it to Settings only). */
export const CLEAR_LOGINS_COMMAND = "clear_saved_logins";

/** Whether this platform (get_settings' `platform`) has the switches: Windows only. */
export function loginsAvailable(platform: string): boolean {
  return platform === "windows";
}

/** The switches' labels and notes (drawn as text). */
export const SAVE_PASSWORDS_LABEL = "Offer to save passwords (kept by Windows' web engine, on this computer)";
export const SAVE_PASSWORDS_NOTE =
  "Windows' web engine offers to save a password when you sign in, and fills it in next time. Off: nothing new is saved — passwords saved earlier still fill in until you remove them.";
export const AUTOFILL_LABEL = "Fill in forms (addresses and the like)";
export const AUTOFILL_NOTE = "Windows' web engine suggests what you typed in forms before. Off: no suggestions, and nothing new is kept.";
export const REMOVE_LABEL = "Remove saved passwords and form data…";
export const REMOVE_NOTE =
  "Removes every password and form entry Windows' web engine saved for Little Joan. Passkeys (Windows Hello keeps them), cookies, site data and history stay. Little Joan itself never reads, exports or syncs them.";

/** Why there is nothing to switch on `platform`: the first line, then one line why. */
export function unavailableNote(platform: string): { title: string; why: string } {
  const title = "Saving passwords isn't available in Little Joan on this system yet.";
  if (platform === "macos") {
    return { title, why: "Passkeys need an entitlement Apple grants to browsers, and password AutoFill isn't dependable in an app's web view." };
  }
  if (platform === "linux") {
    return { title, why: "WebKitGTK has no password manager." };
  }
  return { title, why: "This system's web engine offers no password saving to Little Joan." };
}

/** "What is stored where": where saved passwords and form data are on `platform`. */
export function storedWhere(platform: string): string {
  return loginsAvailable(platform)
    ? "In Windows' web engine's own store, inside the site-data folder above, on this computer. Little Joan never reads them."
    : "Nowhere: Little Joan saves no passwords or form data on this system.";
}

/** The switches as settings.json has them (defaults for anything missing or of another type). */
export function loginsFrom(settings: { savePasswords?: unknown; autofillForms?: unknown }): LoginsInput {
  return {
    savePasswords: typeof settings.savePasswords === "boolean" ? settings.savePasswords : LOGINS_DEFAULTS.savePasswords,
    autofillForms: typeof settings.autofillForms === "boolean" ? settings.autofillForms : LOGINS_DEFAULTS.autofillForms,
  };
}

/** "Remove saved passwords and form data…": ask, then the engine's answer. Not a setting. */
export interface RemoveState {
  /** The question and its buttons show. */
  asking: boolean;
  /** Waiting for the engine. */
  busy: boolean;
  /** What happened last ("" for nothing). */
  message: string;
}

export type RemoveEvent = { kind: "ask" } | { kind: "cancel" } | { kind: "confirm" } | { kind: "done" } | { kind: "failed"; error: string };

export const REMOVE_START: RemoveState = { asking: false, busy: false, message: "" };

/** The question before removing. */
export const REMOVE_QUESTION = "Remove every saved password and form entry? This can't be undone.";

/** What the status line says once the engine answered. */
export const REMOVED_TEXT = "Saved passwords and form data removed.";

export function removeStep(state: RemoveState, event: RemoveEvent): RemoveState {
  switch (event.kind) {
    case "ask":
      return state.busy ? state : { ...state, asking: true, message: "" };
    case "cancel":
      return state.busy ? state : { ...state, asking: false };
    case "confirm":
      return state.asking && !state.busy ? { ...state, busy: true } : state;
    case "done":
      return { asking: false, busy: false, message: REMOVED_TEXT };
    case "failed":
      return { asking: false, busy: false, message: event.error };
  }
}
