import { describe, expect, it } from "vitest";
import type { ExtensionInfo, ExtensionsView } from "../command/extensions.ts";
import { details, developerNote, groups, isDirty, isOn, permissionWords, toggle, withDeveloperMode, type Draft } from "./extensions-logic.ts";

function extension(id: string, kind: "bundled" | "developer", overrides: Partial<ExtensionInfo> = {}): ExtensionInfo {
  return {
    id,
    name: id.toUpperCase(),
    version: "1.0",
    description: `${id} does things`,
    author: "Someone",
    kind,
    enabled: true,
    icon: null,
    permissions: [],
    commands: [],
    pageActions: [],
    ...overrides,
  };
}

const VIEW: ExtensionsView = {
  developerMode: true,
  folder: "/config/extensions",
  extensions: [
    extension("hello", "bundled", { permissions: ["storage", "clipboard", "pageInfo", "network:api.example.com"] }),
    extension("mine", "developer", { author: "" }),
  ],
  skipped: [],
};

const SAVED: Draft = { disabled: ["builtin:calc"], developerMode: true };

describe("Settings → Extensions", () => {
  it("groups built-in, bundled and developer extensions", () => {
    const listed = groups(VIEW, [{ id: "builtin:calc", name: "Calculator", description: "= 12*7" }], SAVED);
    expect(listed.builtIn).toEqual([
      { id: "builtin:calc", name: "Calculator", details: "Built in", description: "= 12*7", permissions: [], icon: null, on: false },
    ]);
    expect(listed.bundled.map((e) => e.id)).toEqual(["hello"]);
    expect(listed.developer.map((e) => e.id)).toEqual(["mine"]);
    expect(listed.developer[0]?.details).toBe("Version 1.0");
    expect(groups(null, [], SAVED)).toEqual({ builtIn: [], bundled: [], developer: [] });
  });

  it("words permissions plainly", () => {
    const [hello] = groups(VIEW, [], SAVED).bundled;
    expect(hello?.permissions).toEqual([
      "Keep its own data on this computer (up to 1 MB)",
      "Copy text to your clipboard",
      "See the address and title of the page you use it on",
      "Connect to api.example.com",
    ]);
    expect(permissionWords("network:*.example.org")).toBe("Connect to *.example.org");
    expect(details("2.0", "Little Joan")).toBe("Version 2.0 · by Little Joan");
    expect(details("", "")).toBe("");
  });

  it("switches show the draft and count as changes until saved", () => {
    let draft = toggle(SAVED, "hello", false);
    expect(isOn(draft, "hello")).toBe(false);
    expect(groups(VIEW, [], draft).bundled[0]?.on).toBe(false);
    expect(isDirty(SAVED, draft)).toBe(true);
    draft = toggle(draft, "hello", true);
    expect(draft.disabled).toEqual(["builtin:calc"]);
    expect(isDirty(SAVED, draft)).toBe(false);
    expect(toggle(toggle(SAVED, "a", false), "a", false).disabled).toEqual(["a", "builtin:calc"]);
    expect(isDirty(SAVED, withDeveloperMode(SAVED, false))).toBe(true);
    expect(isDirty({ disabled: ["b", "a"], developerMode: false }, { disabled: ["a", "b"], developerMode: false })).toBe(false);
  });

  it("says why the developer group is empty", () => {
    expect(developerNote(VIEW, { disabled: [], developerMode: false })).toBe("Developer mode is off.");
    expect(developerNote({ ...VIEW, developerMode: false }, { disabled: [], developerMode: true })).toMatch(/once you save/);
    expect(developerNote(VIEW, { disabled: [], developerMode: true })).toMatch(/No extensions/);
  });
});
