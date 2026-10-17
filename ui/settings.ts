// The settings window: a shortcut recorder and two lists. All validation happens in Rust; this
// page shows what it says.
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { acceleratorFromEvent, displayShortcut } from "./lib/shortcut.ts";
import {
  changes,
  hasChanges,
  toInput,
  type Changes,
  type Choice,
  type Settings,
  type SettingsInput,
  type SettingsView,
} from "./settings-logic.ts";

function element<T extends HTMLElement>(selector: string): T {
  const found = document.querySelector<T>(selector);
  if (!found) {
    throw new Error(`missing ${selector}`);
  }
  return found;
}

const recorder = element<HTMLButtonElement>("#shortcut");
const note = element<HTMLElement>("#shortcut-note");
const login = element<HTMLInputElement>("#login");
const loginNote = element<HTMLElement>("#login-note");
const engine = element<HTMLSelectElement>("#engine");
const language = element<HTMLSelectElement>("#language");
const mobile = element<HTMLInputElement>("#mobile");
const error = element<HTMLElement>("#error");
const save = element<HTMLButtonElement>("#save");
const cancel = element<HTMLButtonElement>("#cancel");

const NOTE = "Click, then press the keys you want to use.";

let saved: Settings | null = null;
let savedLogin: boolean | null = null;
let isMac = false;
let shortcut = "";
let recording = false;

function fill(select: HTMLSelectElement, choices: Choice[], selected: string): void {
  select.replaceChildren(
    ...choices.map((choice) => {
      const option = document.createElement("option");
      option.value = choice.id;
      option.textContent = choice.name;
      option.selected = choice.id === selected;
      return option;
    }),
  );
}

function showError(message: string): void {
  error.textContent = message;
  error.hidden = message.length === 0;
}

function refresh(): void {
  recorder.textContent = recording ? "Press keys…" : displayShortcut(shortcut, isMac) || "None";
  recorder.classList.toggle("recording", recording);
  note.textContent = recording ? "Esc cancels." : NOTE;
  save.disabled = saved === null || !hasChanges(pending(saved));
}

function input(): SettingsInput {
  return toInput(shortcut, engine.value, language.value, mobile.checked);
}

function pending(from: Settings): Changes {
  return changes(from, savedLogin, input(), login.checked);
}

/** Applies what changed: settings first, so a taken shortcut stops Save before anything else. */
async function apply(todo: Changes, wanted: SettingsInput, wantedLogin: boolean): Promise<void> {
  if (todo.settings) {
    saved = await invoke<Settings>("save_settings", { settings: wanted });
  }
  if (todo.startAtLogin) {
    savedLogin = await invoke<boolean>("set_start_at_login", { enabled: wantedLogin });
    login.checked = savedLogin;
    if (savedLogin !== wantedLogin) {
      throw new Error("Your system didn't keep the change. Try again, or check its login items.");
    }
  }
}

function stopRecording(): void {
  if (!recording) {
    return;
  }
  recording = false;
  void invoke("suspend_shortcut", { suspended: false });
  refresh();
}

recorder.addEventListener("click", () => {
  if (recording) {
    stopRecording();
    return;
  }
  recording = true;
  showError("");
  void invoke("suspend_shortcut", { suspended: true });
  refresh();
});

recorder.addEventListener("blur", stopRecording);

window.addEventListener(
  "keydown",
  (event) => {
    if (!recording) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    if (event.code === "Escape" && !event.ctrlKey && !event.metaKey && !event.altKey && !event.shiftKey) {
      stopRecording();
      return;
    }
    const accelerator = acceleratorFromEvent(event, isMac);
    if (accelerator) {
      shortcut = accelerator;
      stopRecording();
    }
  },
  true,
);

engine.addEventListener("change", refresh);
language.addEventListener("change", refresh);
mobile.addEventListener("change", refresh);
login.addEventListener("change", refresh);

save.addEventListener("click", () => {
  stopRecording();
  showError("");
  if (saved === null) {
    return;
  }
  apply(pending(saved), input(), login.checked)
    .then(async () => {
      await getCurrentWindow().close();
    })
    .catch((err: unknown) => {
      showError(err instanceof Error ? err.message : String(err));
      refresh();
    });
});

cancel.addEventListener("click", () => {
  stopRecording();
  void getCurrentWindow().close();
});

invoke<SettingsView>("get_settings")
  .then((view) => {
    saved = view.settings;
    isMac = view.platform === "macos";
    shortcut = view.settings.shortcut;
    fill(engine, view.engines, view.settings.searchEngine);
    fill(language, view.languages, view.settings.translateTo);
    mobile.checked = view.settings.mobileSites;
    savedLogin = view.startAtLogin;
    login.checked = view.startAtLogin ?? false;
    login.disabled = view.startAtLogin === null;
    if (view.startAtLogin === null) {
      loginNote.textContent = "Couldn't read your system's login items.";
    }
    refresh();
  })
  .catch((err: unknown) => showError(String(err)));
