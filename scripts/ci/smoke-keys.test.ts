import { describe, expect, it } from "vitest";
import { macKeyToken } from "./smoke-keys.mts";

describe("typing on macOS", () => {
  it("sends key codes for what the US layout has, with Shift where needed", () => {
    expect([..."qr"].map(macKeyToken)).toEqual(["k12", "k15"]);
    expect([..."hello"].map(macKeyToken)).toEqual(["k4", "k14", "k37", "k37", "k31"]);
    expect([..."= 12*7"].map(macKeyToken)).toEqual(["k24", "k49", "k18", "k19", "k28s", "k26"]);
    expect(macKeyToken("Q")).toBe("k12s");
    expect(macKeyToken("a")).toBe("k0");
  });

  it("falls back to a Unicode string for anything else", () => {
    expect(macKeyToken("é")).toBe("ué");
    expect(macKeyToken("☕")).toBe("u☕");
    expect(macKeyToken("~")).toBe("u~");
  });
});
