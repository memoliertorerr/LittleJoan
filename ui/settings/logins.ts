// Settings → Privacy's saved passwords and form data (LJ51; DOM glue). logins-logic.ts writes every
// text. Windows: "Offer to save passwords" and "Fill in forms" are a draft like every setting (Save
// keeps them, Cancel drops them); "Remove saved passwords and form data…" asks first, then calls
// `clear_saved_logins` at once — Windows' web engine removes what it saved; Little Joan never sees
// it. macOS and Linux: the note why it isn't available, and no switch.
import {
  AUTOFILL_LABEL,
  AUTOFILL_NOTE,
  CLEAR_LOGINS_COMMAND,
  REMOVE_LABEL,
  REMOVE_NOTE,
  REMOVE_QUESTION,
  REMOVE_START,
  SAVE_PASSWORDS_LABEL,
  SAVE_PASSWORDS_NOTE,
  loginsAvailable,
  removeStep,
  storedWhere,
  unavailableNote,
  type LoginsInput,
  type RemoveEvent,
} from "./logins-logic.ts";

export interface LoginsSection {
  /** get_settings answered: what this platform shows, the switches at the saved values. */
  load(platform: string, saved: LoginsInput): void;
  /** What Save sends; `undefined` where there are no switches (Rust keeps the saved ones). */
  draft(): LoginsInput | undefined;
}

export function loginsSection(root: HTMLElement, call: (command: string) => Promise<unknown>, onChange: () => void): LoginsSection {
  const find = <T extends HTMLElement>(id: string): T | null => root.querySelector<T>(`#${id}`);
  const windows = find<HTMLElement>("logins-windows");
  const unavailable = find<HTMLElement>("logins-unavailable");
  const save = find<HTMLInputElement>("save-passwords");
  const autofill = find<HTMLInputElement>("autofill-forms");
  const remove = find<HTMLButtonElement>("logins-remove");
  const confirm = find<HTMLElement>("logins-confirm");
  const cancel = find<HTMLButtonElement>("logins-cancel");
  const go = find<HTMLButtonElement>("logins-go");
  const status = find<HTMLElement>("logins-status");
  const stored = document.getElementById("store-logins");
  const text = (id: string, value: string): void => {
    const element = find<HTMLElement>(id);
    if (element) {
      element.textContent = value;
    }
  };
  let available = false;
  let removing = REMOVE_START;

  text("save-passwords-label", SAVE_PASSWORDS_LABEL);
  text("save-passwords-note", SAVE_PASSWORDS_NOTE);
  text("autofill-forms-label", AUTOFILL_LABEL);
  text("autofill-forms-note", AUTOFILL_NOTE);
  text("logins-remove-note", REMOVE_NOTE);
  text("logins-question", REMOVE_QUESTION);
  if (remove) {
    remove.textContent = REMOVE_LABEL;
  }

  function drawRemoving(): void {
    if (remove) {
      remove.disabled = removing.busy || removing.asking;
    }
    if (confirm) {
      confirm.hidden = !removing.asking;
    }
    if (go) {
      go.disabled = removing.busy;
    }
    if (status) {
      status.textContent = removing.message;
      status.hidden = removing.message.length === 0;
    }
  }

  function onRemove(event: RemoveEvent): void {
    const before = removing;
    removing = removeStep(removing, event);
    drawRemoving();
    if (event.kind === "confirm" && removing.busy && !before.busy) {
      call(CLEAR_LOGINS_COMMAND)
        .then(() => onRemove({ kind: "done" }))
        .catch((err: unknown) => onRemove({ kind: "failed", error: err instanceof Error ? err.message : String(err) }));
    } else if (event.kind === "ask") {
      go?.focus();
    }
  }

  save?.addEventListener("change", onChange);
  autofill?.addEventListener("change", onChange);
  remove?.addEventListener("click", () => onRemove({ kind: "ask" }));
  cancel?.addEventListener("click", () => onRemove({ kind: "cancel" }));
  go?.addEventListener("click", () => onRemove({ kind: "confirm" }));
  drawRemoving();

  return {
    load(platform: string, saved: LoginsInput) {
      available = loginsAvailable(platform);
      if (windows) {
        windows.hidden = !available;
      }
      if (unavailable) {
        unavailable.hidden = available;
      }
      const why = unavailableNote(platform);
      text("logins-unavailable-title", why.title);
      text("logins-unavailable-why", why.why);
      if (stored) {
        stored.textContent = storedWhere(platform);
      }
      if (save && autofill) {
        save.checked = saved.savePasswords;
        autofill.checked = saved.autofillForms;
      }
    },
    draft: () => (available && save && autofill ? { savePasswords: save.checked, autofillForms: autofill.checked } : undefined),
  };
}
