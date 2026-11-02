import { describe, expect, it } from "vitest";
import { glassControl } from "./glass.ts";

/** A button with its data-glass-choice, aria-pressed and click listener. */
function fakeButton(choice: string) {
  const attributes: Record<string, string> = {};
  let listener: (() => void) | undefined;
  return {
    dataset: { glassChoice: choice },
    attributes,
    setAttribute(name: string, value: string) {
      attributes[name] = value;
    },
    addEventListener(_type: string, next: () => void) {
      listener = next;
    },
    click() {
      listener?.();
    },
  };
}

function fakeRoot() {
  const buttons = ["off", "clear", "tinted"].map(fakeButton);
  const note = { textContent: "" };
  const root = {
    hidden: true,
    querySelectorAll: () => buttons,
    querySelector: () => note,
  };
  return { root, buttons, note, element: root as unknown as HTMLElement };
}

const pressed = (buttons: ReturnType<typeof fakeButton>[]) => buttons.filter((b) => b.attributes["aria-pressed"] === "true").map((b) => b.dataset.glassChoice);

describe("Settings → Appearance's Liquid Glass control (LJ19.9)", () => {
  it("stays hidden and gives no draft where this system has no glass", () => {
    const fake = fakeRoot();
    const control = glassControl(fake.element, () => undefined);
    control.load("clear", { available: false, blockedBy: null });
    expect(fake.root.hidden).toBe(true);
    // Save then leaves the saved setting as it is.
    expect(control.draft()).toBeUndefined();
  });

  it("shows on macOS 26+, marks the saved choice, and a click is a draft that previews", () => {
    const fake = fakeRoot();
    let changes = 0;
    const control = glassControl(fake.element, () => (changes += 1));
    control.load("off", { available: true, blockedBy: null });
    expect(fake.root.hidden).toBe(false);
    expect(pressed(fake.buttons)).toEqual(["off"]);
    expect(control.draft()).toBe("off");
    fake.buttons[2]?.click();
    expect(control.draft()).toBe("tinted");
    expect(pressed(fake.buttons)).toEqual(["tinted"]);
    expect(changes).toBe(1);
    expect(fake.note.textContent).toMatch(/^More opaque/);
    // The same choice again changes nothing.
    fake.buttons[2]?.click();
    expect(changes).toBe(1);
  });

  it("says why the solid theme shows under Reduce Transparency", () => {
    const fake = fakeRoot();
    const control = glassControl(fake.element, () => undefined);
    control.load("clear", { available: true, blockedBy: "reduceTransparency" });
    expect(fake.root.hidden).toBe(false);
    expect(fake.note.textContent).toContain("Reduce Transparency is on");
    expect(control.draft()).toBe("clear");
  });
});
