// Settings → Extensions (pure, unit-tested): the list in groups — Built-in (Little Joan's own
// quick answers), Bundled, Developer (unpacked folders, while Developer mode is on) — each with a
// switch, its details and its permissions in plain words, and the draft of what Save sends
// (`disabledExtensions`, `developerMode`) against the saved settings.
import type { ExtensionInfo, ExtensionsView } from "../command/extensions.ts";

/** A built-in feature that can be switched off like an extension (its id: `builtin:<name>`). */
export interface BuiltIn {
  id: string;
  name: string;
  description: string;
}

/** What one row of the list shows. */
export interface ListedExtension {
  id: string;
  name: string;
  /** "Version 1.0 · by Someone" (what there is of it). */
  details: string;
  description: string;
  /** The permissions, in plain words. */
  permissions: string[];
  icon: string | null;
  on: boolean;
}

export interface Groups {
  builtIn: ListedExtension[];
  bundled: ListedExtension[];
  developer: ListedExtension[];
}

/** A permission from the manifest, in plain words. */
export function permissionWords(permission: string): string {
  if (permission.startsWith("network:")) {
    return `Connect to ${permission.slice("network:".length)}`;
  }
  switch (permission) {
    case "storage":
      return "Keep its own data on this computer (up to 1 MB)";
    case "clipboard":
      return "Copy text to your clipboard";
    case "pageInfo":
      return "See the address and title of the page you use it on";
    default:
      return permission;
  }
}

/** "Version 1.0 · by Someone", leaving out what isn't there. */
export function details(version: string, author: string): string {
  return [version ? `Version ${version}` : "", author ? `by ${author}` : ""].filter(Boolean).join(" · ");
}

/** The draft: which ids are switched off, and Developer mode. */
export interface Draft {
  disabled: string[];
  developerMode: boolean;
}

export function isOn(draft: Draft, id: string): boolean {
  return !draft.disabled.includes(id);
}

/** Switches `id` on or off in the draft (kept sorted, without repeats). */
export function toggle(draft: Draft, id: string, on: boolean): Draft {
  const rest = draft.disabled.filter((other) => other !== id);
  return { ...draft, disabled: (on ? rest : [...rest, id]).sort() };
}

export function withDeveloperMode(draft: Draft, on: boolean): Draft {
  return { ...draft, developerMode: on };
}

/** Whether the draft differs from what was saved. */
export function isDirty(saved: Draft, draft: Draft): boolean {
  const a = [...saved.disabled].sort();
  const b = [...draft.disabled].sort();
  return saved.developerMode !== draft.developerMode || a.length !== b.length || a.some((id, i) => id !== b[i]);
}

function listed(extension: ExtensionInfo, draft: Draft): ListedExtension {
  return {
    id: extension.id,
    name: extension.name,
    details: details(extension.version, extension.author),
    description: extension.description,
    permissions: extension.permissions.map(permissionWords),
    icon: extension.icon,
    on: isOn(draft, extension.id),
  };
}

/** The list's groups for the draft (the switches show the draft, not what Rust last loaded). */
export function groups(view: ExtensionsView | null, builtIns: readonly BuiltIn[], draft: Draft): Groups {
  const extensions = view?.extensions ?? [];
  return {
    builtIn: builtIns.map((builtIn) => ({
      id: builtIn.id,
      name: builtIn.name,
      details: "Built in",
      description: builtIn.description,
      permissions: [],
      icon: null,
      on: isOn(draft, builtIn.id),
    })),
    bundled: extensions.filter((e) => e.kind === "bundled").map((e) => listed(e, draft)),
    developer: extensions.filter((e) => e.kind === "developer").map((e) => listed(e, draft)),
  };
}

/** What the Developer group says when it has nothing to list. */
export function developerNote(view: ExtensionsView | null, draft: Draft): string {
  if (!draft.developerMode) {
    return "Developer mode is off.";
  }
  if (view && !view.developerMode) {
    return "The folder below loads once you save.";
  }
  return "No extensions in the folder below yet.";
}
