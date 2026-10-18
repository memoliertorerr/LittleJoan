// The settings window: tabs in a sidebar (General, Appearance, Extensions, Privacy, About), a
// shortcut recorder, lists and switches, and Save / Cancel for every tab. All validation happens
// in Rust; this page shows what it says. Rust picks the first tab (window.__littleJoanSettingsTab,
// set before this script runs) and switches an open window with window.littleJoanSettings.showTab.
// The window wears the saved theme; the Appearance tab (settings/appearance.ts) edits it, the
// window and the open page window previewing every change, and Save keeps it. The Extensions tab
// (settings/extensions.ts) lists them with their switches and Developer mode.
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { themeFrom } from "./lib/theme.ts";
import { acceleratorFromEvent, displayShortcut } from "./lib/shortcut.ts";
import { appearance } from "./settings/appearance.ts";
import { extensionsTab } from "./settings/extensions.ts";
import { sitesList } from "./settings/sites.ts";
import { builtInsFrom, siteModesFrom } from "./settings/sites-logic.ts";
import { BUILT_INS } from "./answers/index.ts";
import {
  CLEAR_CHOICES,
  CLEAR_START,
  TABS,
  changes,
  clearQuestion,
  clearStep,
  hasChanges,
  tabForKey,
  tabFrom,
  toInput,
  type Changes,
  type Choice,
  type ClearEvent,
  type Settings,
  type SettingsInput,
  type SettingsView,
  type Tab,
} from "./settings-logic.ts";

declare global {
  interface Window {
    __littleJoanSettingsTab?: string;
    littleJoanSettings?: { showTab(name: string): void; setTheme(theme: unknown): void };
  }
}

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
const remember = element<HTMLInputElement>("#remember");
const clearRange = element<HTMLSelectElement>("#clear-range");
const clearAsk = element<HTMLButtonElement>("#clear-ask");
const clearConfirm = element<HTMLElement>("#clear-confirm");
const clearQuestionText = element<HTMLElement>("#clear-question");
const clearCancel = element<HTMLButtonElement>("#clear-cancel");
const clearGo = element<HTMLButtonElement>("#clear-go");
const clearStatus = element<HTMLElement>("#clear-status");
const storeFolder = element<HTMLElement>("#store-folder");
const storeSite = element<HTMLElement>("#store-site");
const aboutVersion = element<HTMLElement>("#about-version");
const error = element<HTMLElement>("#error");
const save = element<HTMLButtonElement>("#save");
const cancel = element<HTMLButtonElement>("#cancel");

const NOTE = "Click, then press the keys you want to use.";

const look = appearance(element<HTMLElement>("#panel-appearance"), refresh);
const extensionsPanel = extensionsTab(element<HTMLElement>("#panel-extensions"), refresh, BUILT_INS);
const sites = sitesList(element<HTMLElement>("#site-modes"), element<HTMLElement>("#site-modes-empty"), refresh);

let saved: Settings | null = null;
let savedLogin: boolean | null = null;
let isMac = false;
let shortcut = "";
let recording = false;
let tab: Tab = tabFrom(window.__littleJoanSettingsTab);
let clearing = CLEAR_START;

function fill(select: HTMLSelectElement, choices: readonly Choice[], selected: string): void {
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

/** Shows `next` in the sidebar and its panel; `focus` moves the keyboard to its tab button. */
function showTab(next: Tab, focus = false): void {
  tab = next;
  for (const { id } of TABS) {
    const button = element<HTMLButtonElement>(`#tab-${id}`);
    const panel = element<HTMLElement>(`#panel-${id}`);
    const on = id === next;
    button.setAttribute("aria-selected", on ? "true" : "false");
    button.tabIndex = on ? 0 : -1;
    button.classList.toggle("selected", on);
    panel.hidden = !on;
    if (on && focus) {
      button.focus();
    }
  }
}

function refresh(): void {
  recorder.textContent = recording ? "Press keys…" : displayShortcut(shortcut, isMac) || "None";
  recorder.classList.toggle("recording", recording);
  note.textContent = recording ? "Esc cancels." : NOTE;
  save.disabled = saved === null || !hasChanges(pending(saved));
}

function input(): SettingsInput {
  return toInput(
    shortcut,
    engine.value,
    language.value,
    mobile.checked,
    remember.checked,
    look.draft(),
    extensionsPanel.draft(),
    sites.draft(),
  );
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

/** The Privacy tab's "Clear history" (it acts at once; Save and Cancel don't cover it). */
function drawClearing(): void {
  clearRange.value = clearing.range;
  clearRange.disabled = clearing.busy;
  clearAsk.disabled = clearing.busy || clearing.asking;
  clearConfirm.hidden = !clearing.asking;
  clearQuestionText.textContent = clearQuestion(clearing.range);
  clearGo.disabled = clearing.busy;
  clearStatus.textContent = clearing.message;
  clearStatus.hidden = clearing.message.length === 0;
}

function onClear(event: ClearEvent): void {
  const before = clearing;
  clearing = clearStep(clearing, event);
  drawClearing();
  if (event.kind === "confirm" && clearing.busy && !before.busy) {
    invoke<number>("history_clear", { range: clearing.range })
      .then((removed) => onClear({ kind: "done", removed }))
      .catch((err: unknown) => onClear({ kind: "failed", error: String(err) }));
  } else if (event.kind === "ask") {
    clearGo.focus();
  }
}

for (const { id } of TABS) {
  const button = element<HTMLButtonElement>(`#tab-${id}`);
  button.addEventListener("click", () => showTab(id));
  button.addEventListener("keydown", (event) => {
    const next = tabForKey(tab, event.key);
    if (next) {
      event.preventDefault();
      showTab(next, true);
    }
  });
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

for (const control of [engine, language, mobile, login, remember]) {
  control.addEventListener("change", refresh);
}

fill(
  clearRange,
  CLEAR_CHOICES.map((choice) => ({ id: choice.range, name: choice.label })),
  clearing.range,
);
clearRange.addEventListener("change", () => {
  const choice = CLEAR_CHOICES.find((c) => c.range === clearRange.value);
  if (choice) {
    onClear({ kind: "pick", range: choice.range });
  }
});
clearAsk.addEventListener("click", () => onClear({ kind: "ask" }));
clearCancel.addEventListener("click", () => onClear({ kind: "cancel" }));
clearGo.addEventListener("click", () => onClear({ kind: "confirm" }));

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

window.littleJoanSettings = {
  showTab(name: string) {
    showTab(tabFrom(name));
  },
  setTheme(theme: unknown) {
    look.load(themeFrom(theme));
  },
};

showTab(tab);
drawClearing();

invoke<SettingsView>("get_settings")
  .then((view) => {
    saved = view.settings;
    look.load(themeFrom(view.settings.theme));
    isMac = view.platform === "macos";
    shortcut = view.settings.shortcut;
    fill(engine, view.engines, view.settings.searchEngine);
    fill(language, view.languages, view.settings.translateTo);
    mobile.checked = view.settings.mobileSites;
    remember.checked = view.settings.rememberHistory;
    extensionsPanel.load({ disabledExtensions: view.settings.disabledExtensions, developerMode: view.settings.developerMode });
    sites.load(siteModesFrom(view.settings.siteModes), builtInsFrom(view.builtInSiteModes));
    savedLogin = view.startAtLogin;
    login.checked = view.startAtLogin ?? false;
    login.disabled = view.startAtLogin === null;
    if (view.startAtLogin === null) {
      loginNote.textContent = "Couldn't read your system's login items.";
    }
    storeFolder.textContent = view.storage.folder;
    storeSite.textContent = view.storage.siteData;
    aboutVersion.textContent = `Version ${view.version}`;
    refresh();
  })
  .catch((err: unknown) => showError(String(err)));
