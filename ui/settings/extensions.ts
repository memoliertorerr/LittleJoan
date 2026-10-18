// Settings → Extensions (DOM glue): the list in its groups (extensions-logic.ts) with a switch
// each, Developer mode with its warning, "Open extensions folder" and "Reload extensions", and
// what was skipped and why. Everything an extension supplies goes in as text (textContent); its
// icon only as a checked data: URI. The switches and Developer mode are a draft that Save sends.
import { invoke } from "@tauri-apps/api/core";
import { ICON_URI, type ExtensionsView } from "../command/extensions.ts";
import type { ExtensionsInput } from "../settings-logic.ts";
import { developerNote, groups, isDirty, toggle, withDeveloperMode, type BuiltIn, type Draft, type ListedExtension } from "./extensions-logic.ts";

export interface ExtensionsTab {
  /** The saved settings arrived (or Save happened): the draft starts from them. */
  load(saved: ExtensionsInput): void;
  /** What Save sends for this tab. */
  draft(): ExtensionsInput;
  /** Whether the draft differs from the saved settings. */
  dirty(): boolean;
}

function find<T extends HTMLElement>(root: HTMLElement, selector: string): T {
  const found = root.querySelector<T>(selector);
  if (!found) {
    throw new Error(`missing ${selector}`);
  }
  return found;
}

function text(tag: string, className: string, value: string): HTMLElement {
  const el = document.createElement(tag);
  el.className = className;
  el.textContent = value;
  return el;
}

export function extensionsTab(root: HTMLElement, changed: () => void, builtIns: readonly BuiltIn[]): ExtensionsTab {
  const builtInGroup = find<HTMLElement>(root, "#ext-builtin-group");
  const builtInList = find<HTMLElement>(root, "#ext-builtin");
  const bundledList = find<HTMLElement>(root, "#ext-bundled");
  const developerList = find<HTMLElement>(root, "#ext-developer");
  const developerNoteText = find<HTMLElement>(root, "#ext-developer-note");
  const skippedList = find<HTMLElement>(root, "#ext-skipped");
  const devMode = find<HTMLInputElement>(root, "#dev-mode");
  const folder = find<HTMLElement>(root, "#ext-folder");
  const openFolder = find<HTMLButtonElement>(root, "#ext-open-folder");
  const reload = find<HTMLButtonElement>(root, "#ext-reload");
  const error = find<HTMLElement>(root, "#ext-error");

  let view: ExtensionsView | null = null;
  let saved: Draft = { disabled: [], developerMode: false };
  let draft: Draft = saved;

  const showError = (message: string): void => {
    error.textContent = message;
    error.hidden = message.length === 0;
  };

  const item = (extension: ListedExtension): HTMLElement => {
    const li = document.createElement("li");
    li.className = "ext-item";
    const icon = document.createElement("span");
    icon.className = "ext-icon";
    if (extension.icon !== null && ICON_URI.test(extension.icon)) {
      const img = document.createElement("img");
      img.alt = "";
      img.src = extension.icon;
      icon.append(img);
    }
    const body = document.createElement("div");
    body.className = "ext-text";
    const name = text("div", "ext-name", extension.name);
    if (extension.details) {
      name.append(text("span", "ext-details", ` ${extension.details}`));
    }
    body.append(name);
    if (extension.description) {
      body.append(text("div", "ext-description", extension.description));
    }
    if (extension.permissions.length > 0) {
      const permissions = document.createElement("ul");
      permissions.className = "ext-permissions";
      permissions.setAttribute("aria-label", `What ${extension.name} may do`);
      permissions.append(...extension.permissions.map((words) => text("li", "", words)));
      body.append(permissions);
    }
    const toggleBox = document.createElement("input");
    toggleBox.type = "checkbox";
    toggleBox.className = "ext-switch";
    toggleBox.checked = extension.on;
    toggleBox.setAttribute("aria-label", `${extension.name} on`);
    toggleBox.addEventListener("change", () => {
      draft = toggle(draft, extension.id, toggleBox.checked);
      changed();
    });
    li.append(icon, body, toggleBox);
    return li;
  };

  const draw = (): void => {
    const listed = groups(view, builtIns, draft);
    builtInGroup.hidden = listed.builtIn.length === 0;
    builtInList.replaceChildren(...listed.builtIn.map(item));
    bundledList.replaceChildren(...listed.bundled.map(item));
    developerList.replaceChildren(...listed.developer.map(item));
    developerNoteText.textContent = listed.developer.length === 0 ? developerNote(view, draft) : "";
    developerNoteText.hidden = listed.developer.length > 0;
    skippedList.replaceChildren(
      ...(view?.skipped ?? []).map((skipped) => text("li", "", `${skipped.name} wasn't loaded: ${skipped.reason}`)),
    );
    devMode.checked = draft.developerMode;
    folder.textContent = view ? view.folder : "";
  };

  const show = (next: ExtensionsView): void => {
    view = next;
    draw();
  };

  devMode.addEventListener("change", () => {
    draft = withDeveloperMode(draft, devMode.checked);
    draw();
    changed();
  });
  openFolder.addEventListener("click", () => {
    showError("");
    invoke("open_extensions_folder").catch((err: unknown) => showError(String(err)));
  });
  reload.addEventListener("click", () => {
    showError("");
    invoke<ExtensionsView>("reload_extensions")
      .then(show)
      .catch((err: unknown) => showError(String(err)));
  });

  invoke<ExtensionsView>("extensions_list")
    .then(show)
    .catch((err: unknown) => showError(String(err)));

  return {
    load(settings: ExtensionsInput) {
      saved = { disabled: [...settings.disabledExtensions].sort(), developerMode: settings.developerMode };
      draft = saved;
      draw();
    },
    draft() {
      return { disabledExtensions: draft.disabled, developerMode: draft.developerMode };
    },
    dirty() {
      return isDirty(saved, draft);
    },
  };
}
