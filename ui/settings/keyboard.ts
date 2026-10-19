// Settings → Keyboard (LJ44; DOM glue): the shortcut that summons Little Joan — its recorder moved
// here from General, pausing the shortcut while it listens as before — and every Little Joan key
// from Rust's keymap_info(), each a recorder button with "No key" and "Reset", its problems under
// it (keyboard-logic.ts: errors block Save, warnings don't), "Reset all" at the bottom. General
// keeps a line with the shortcut and a "Change…" button. While the XDG GlobalShortcuts portal holds
// the shortcut (a Wayland session, LJ17), both show the desktop's key instead of the recorder and
// say to change it in the desktop's settings. Everything is drawn as text.
import { invoke } from "@tauri-apps/api/core";
import { acceleratorFromEvent } from "../lib/shortcut.ts";
import {
  DEFAULT_SUMMON,
  PORTAL_NOTE,
  allProblems,
  bindingOf,
  canResetAll,
  hasErrors,
  isCustom,
  portalText,
  problemText,
  recordAction,
  resetAll,
  resetKey,
  sameOverrides,
  setKey,
  show,
  type KeyActionInfo,
  type KeyboardDraft,
  type Os,
  type PortalShortcut,
  type Problem,
  type ShortcutProblem,
  type SystemShortcut,
} from "./keyboard-logic.ts";

/** What Rust's keymap_info() says. */
export interface KeymapInfo {
  platform: string;
  actions: KeyActionInfo[];
  globalShortcut: string;
  shortcutProblem: ShortcutProblem | null;
  /** The portal holds the shortcut (LJ17): the desktop's key, or null. */
  portalShortcut: PortalShortcut | null;
  /** macOS's own shortcuts as read from its preferences (empty elsewhere). */
  systemShortcuts: SystemShortcut[];
}

export interface KeyboardTab {
  /** The saved shortcut and keys, the actions, the launch's problem, a Wayland session. */
  load(info: KeymapInfo, saved: { shortcut: string; keys: Record<string, string> }, waylandNoShortcut: boolean): void;
  /** The summon shortcut as edited. */
  shortcut(): string;
  /** The keys as edited, or undefined when they are the saved ones (Rust keeps those). */
  keys(): Record<string, string> | undefined;
  /** The keys as edited, always. */
  overrides(): Record<string, string>;
  /** The portal holds the summon shortcut (the desktop owns its key). */
  portal(): boolean;
  /** A row has an error: Save is refused. */
  hasErrors(): boolean;
  /** Stops listening (Save, Cancel, another tab). */
  stopRecording(): void;
  /** Settings saved: editing starts again from what was saved. */
  saved(shortcut: string, keys: Record<string, string>): void;
}

function find<T extends HTMLElement>(selector: string): T {
  const found = document.querySelector<T>(selector);
  if (!found) {
    throw new Error(`missing ${selector}`);
  }
  return found;
}

const NOTE = "Click, then press the keys you want to use.";

/** `changed` runs after every edit (Save's state); `showKeyboard` switches to this tab. */
export function keyboardTab(changed: () => void, showKeyboard: () => void): KeyboardTab {
  const summon = find<HTMLButtonElement>("#shortcut");
  const summonPortal = find<HTMLElement>("#shortcut-portal");
  const summonReset = find<HTMLButtonElement>("#shortcut-reset");
  const note = find<HTMLElement>("#shortcut-note");
  const summonProblems = find<HTMLElement>("#summon-problems");
  const list = find<HTMLElement>("#kb-actions");
  const resetAllButton = find<HTMLButtonElement>("#kb-reset-all");
  const problemLine = find<HTMLElement>("#kb-problem");
  const wayland = find<HTMLElement>("#kb-wayland");
  const current = find<HTMLElement>("#shortcut-current");
  const generalProblem = find<HTMLElement>("#general-problem");
  const change = find<HTMLButtonElement>("#shortcut-change");
  const generalPortal = find<HTMLElement>("#general-portal-note");
  const engineNote = find<HTMLElement>("#kb-engine-note");

  let os: Os = "linux";
  let draft: KeyboardDraft = { os, actions: [], overrides: {}, summon: DEFAULT_SUMMON };
  let savedKeys: Record<string, string> = {};
  /** What is listening: the summon shortcut, an action's id, or nothing. */
  let recording: string | null = null;

  const problemsInto = (target: HTMLElement, problems: readonly Problem[]): void => {
    target.replaceChildren(
      ...problems.map((problem) => {
        const item = document.createElement("li");
        item.className = problem.level === "error" ? "kb-error" : "kb-warning";
        item.textContent = problem.message;
        return item;
      }),
    );
  };

  const button = (label: string, onClick: () => void): HTMLButtonElement => {
    const b = document.createElement("button");
    b.type = "button";
    b.textContent = label;
    b.addEventListener("click", onClick);
    return b;
  };

  /** The recorder buttons' text and look (without rebuilding the rows, so focus stays put). */
  const drawRecording = (): void => {
    summon.textContent = recording === "summon" ? "Press keys…" : show(draft.summon, os) || "None";
    summon.classList.toggle("recording", recording === "summon");
    note.textContent = draft.portal ? PORTAL_NOTE : recording === null ? NOTE : "Esc cancels.";
    for (const recorder of Array.from(list.querySelectorAll<HTMLButtonElement>("button[data-action]"))) {
      const id = recorder.dataset["action"] ?? "";
      const action = draft.actions.find((a) => a.id === id);
      const binding = action ? bindingOf(action, draft.overrides) : null;
      recorder.textContent = recording === id ? "Press keys…" : binding ? show(binding, os) : "No key";
      recorder.classList.toggle("recording", recording === id);
    }
  };

  const startRecording = (what: string): void => {
    stop();
    recording = what;
    if (what === "summon") {
      // The shortcut is paused while a new one is recorded, so pressing it does nothing else.
      void invoke("suspend_shortcut", { suspended: true });
    }
    drawRecording();
  };

  const stop = (): void => {
    if (recording === null) {
      return;
    }
    if (recording === "summon") {
      void invoke("suspend_shortcut", { suspended: false });
    }
    recording = null;
    drawRecording();
  };

  const edit = (next: KeyboardDraft): void => {
    draft = next;
    draw();
    changed();
  };

  const draw = (): void => {
    const rows = allProblems(draft);
    const portal = draft.portal ?? null;
    summon.hidden = portal !== null;
    summonPortal.hidden = portal === null;
    summonPortal.textContent = portal ? portalText(portal) : "";
    summonReset.hidden = portal !== null || draft.summon === DEFAULT_SUMMON;
    change.hidden = portal !== null;
    generalPortal.hidden = portal === null;
    generalPortal.textContent = PORTAL_NOTE;
    problemsInto(summonProblems, rows["summon"] ?? []);
    current.textContent = portal ? portalText(portal) : show(draft.summon, os) || "None";
    list.replaceChildren(
      ...draft.actions.map((action) => {
        const item = document.createElement("li");
        item.className = "kb-action";
        const line = document.createElement("div");
        line.className = "kb-action-line";
        const title = document.createElement("span");
        title.className = "kb-action-title";
        title.textContent = action.title.replace(/…$/, "");
        const buttons = document.createElement("span");
        buttons.className = "kb-action-buttons";
        const binding = bindingOf(action, draft.overrides);
        const recorder = button("", () => (recording === action.id ? stop() : startRecording(action.id)));
        recorder.className = "recorder";
        recorder.dataset["action"] = action.id;
        recorder.setAttribute("aria-label", `Key for ${title.textContent}`);
        recorder.addEventListener("blur", () => {
          if (recording === action.id) {
            stop();
          }
        });
        buttons.append(recorder);
        if (binding !== null) {
          buttons.append(button("No key", () => edit(setKey(draft, action.id, null))));
        }
        if (isCustom(draft, action.id)) {
          buttons.append(button("Reset", () => edit(resetKey(draft, action.id))));
        }
        line.append(title, buttons);
        const problems = document.createElement("ul");
        problems.className = "kb-problems";
        problemsInto(problems, rows[action.id] ?? []);
        item.append(line, problems);
        return item;
      }),
    );
    resetAllButton.disabled = !canResetAll(draft);
    drawRecording();
  };

  summon.addEventListener("click", () => (recording === "summon" ? stop() : startRecording("summon")));
  summon.addEventListener("blur", () => {
    if (recording === "summon") {
      stop();
    }
  });
  summonReset.addEventListener("click", () => edit({ ...draft, summon: DEFAULT_SUMMON }));
  resetAllButton.addEventListener("click", () => edit(resetAll(draft)));
  change.addEventListener("click", () => {
    showKeyboard();
    summon.focus();
  });

  window.addEventListener(
    "keydown",
    (event) => {
      if (recording === null) {
        return;
      }
      event.preventDefault();
      event.stopPropagation();
      if (recording === "summon") {
        if (event.code === "Escape" && !event.ctrlKey && !event.metaKey && !event.altKey && !event.shiftKey) {
          stop();
          return;
        }
        const accelerator = acceleratorFromEvent(event, os === "macos");
        if (accelerator) {
          stop();
          edit({ ...draft, summon: accelerator });
        }
        return;
      }
      const which = recording;
      const outcome = recordAction(event, os);
      if (outcome.kind === "wait") {
        return;
      }
      stop();
      if (outcome.kind === "clear") {
        edit(setKey(draft, which, null));
      } else if (outcome.kind === "set") {
        edit(setKey(draft, which, outcome.accelerator));
      }
    },
    true,
  );

  return {
    load(info, saved, waylandNoShortcut) {
      os = info.platform === "macos" ? "macos" : info.platform === "windows" ? "windows" : "linux";
      savedKeys = { ...saved.keys };
      draft = {
        os,
        actions: info.actions,
        overrides: { ...saved.keys },
        summon: saved.shortcut,
        system: info.systemShortcuts,
        portal: info.portalShortcut,
      };
      engineNote.hidden = os !== "windows";
      const problem = info.shortcutProblem ? problemText(info.shortcutProblem, os) : "";
      for (const line of [problemLine, generalProblem]) {
        line.textContent = problem;
        line.hidden = problem === "";
      }
      wayland.hidden = !waylandNoShortcut;
      draw();
    },
    shortcut: () => draft.summon,
    keys: () => (sameOverrides(savedKeys, draft.overrides) ? undefined : { ...draft.overrides }),
    overrides: () => ({ ...draft.overrides }),
    portal: () => Boolean(draft.portal),
    hasErrors: () => hasErrors(draft),
    stopRecording: stop,
    saved(shortcut, keys) {
      savedKeys = { ...keys };
      draft = { ...draft, overrides: { ...keys }, summon: shortcut };
      draw();
    },
  };
}
