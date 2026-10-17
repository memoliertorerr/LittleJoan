import { describe, expect, it } from "vitest";
import {
  hasPage,
  initialState,
  onCloseButton,
  onFieldPress,
  onKey,
  withOverlay,
  withShown,
  type BarState,
} from "./view-state.ts";

const PAGE = "https://example.com/a";
const overlayNoPage: BarState = initialState();
const overlayWithPage: BarState = withShown(initialState(), PAGE);
const toolbarWithPage: BarState = withOverlay(overlayWithPage, false);

describe("views", () => {
  it("a new window opens the command bar with no page", () => {
    expect(initialState()).toEqual({ view: "overlay", shown: "" });
    expect(hasPage(initialState())).toBe(false);
  });

  it("follows Rust between the toolbar and the overlay, keeping the address", () => {
    expect(toolbarWithPage).toEqual({ view: "toolbar", shown: PAGE });
    expect(withOverlay(toolbarWithPage, true)).toEqual({ view: "overlay", shown: PAGE });
    expect(withOverlay(withOverlay(toolbarWithPage, true), false)).toEqual(toolbarWithPage);
    expect(hasPage(toolbarWithPage)).toBe(true);
  });

  it("a press on the field opens the command bar from the toolbar only", () => {
    expect(onFieldPress(toolbarWithPage)).toBe("openOverlay");
    expect(onFieldPress(overlayWithPage)).toBe("none");
    expect(onFieldPress(overlayNoPage)).toBe("none");
  });
});

describe("Enter", () => {
  it("opens the trimmed text in either view", () => {
    for (const state of [overlayNoPage, overlayWithPage, toolbarWithPage]) {
      expect(onKey(state, "Enter", "  rust lang ", false)).toEqual({ kind: "open", input: "rust lang" });
    }
    expect(onKey(overlayWithPage, "Enter", PAGE, false)).toEqual({ kind: "open", input: PAGE });
  });

  it("does nothing on an empty field or while composing", () => {
    expect(onKey(overlayNoPage, "Enter", "   ", false)).toEqual({ kind: "none" });
    expect(onKey(overlayNoPage, "Enter", "日本", true)).toEqual({ kind: "none" });
  });
});

describe("Esc", () => {
  it("first reverts an edit", () => {
    expect(onKey(overlayWithPage, "Escape", "https://exam", false)).toEqual({ kind: "revert" });
    expect(onKey(overlayNoPage, "Escape", "typed", false)).toEqual({ kind: "revert" });
    expect(onKey(toolbarWithPage, "Escape", "x", false)).toEqual({ kind: "revert" });
  });

  it("with no page, closes the window", () => {
    expect(onKey(overlayNoPage, "Escape", "", false)).toEqual({ kind: "hibernate" });
    expect(onKey(withOverlay(overlayNoPage, false), "Escape", "", false)).toEqual({ kind: "hibernate" });
  });

  it("with a page, closes the overlay and returns to the page", () => {
    expect(onKey(overlayWithPage, "Escape", PAGE, false)).toEqual({ kind: "closeOverlay" });
    expect(onKey(toolbarWithPage, "Escape", PAGE, false)).toEqual({ kind: "focusPage" });
  });

  it("does nothing while composing", () => {
    expect(onKey(overlayNoPage, "Escape", "", true)).toEqual({ kind: "none" });
  });
});

describe("other keys", () => {
  it("are ignored", () => {
    expect(onKey(overlayWithPage, "a", "a", false)).toEqual({ kind: "none" });
  });
});

describe("the × button", () => {
  it("returns to the page, or with no page closes the window", () => {
    expect(onCloseButton(overlayWithPage)).toBe("closeOverlay");
    expect(onCloseButton(overlayNoPage)).toBe("hibernate");
  });
});
