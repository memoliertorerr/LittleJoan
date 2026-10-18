import { describe, expect, it } from "vitest";
import { buildRows } from "../command/rows.ts";
import { BUILT_INS, findAnswer, quickAnswer } from "./index.ts";

const NOW = Date.UTC(2026, 9, 18, 12, 0, 0);
const context = { now: NOW, disabled: [] as string[], random: () => 0, locale: "en-US" };

describe("quick answers", () => {
  it("ask each built-in in turn", () => {
    expect(findAnswer("= 12*7", context)?.title).toBe("84");
    expect(findAnswer("10 km in mi", context)?.title).toBe("6.213711922 mi");
    expect(findAnswer("time in Tokyo", context)?.title).toBe("9:00 PM in Tokyo");
    expect(findAnswer("flip a coin", context)?.title).toBe("Heads");
    expect(findAnswer("1700000000", context)?.title).toBe("2023-11-14 22:13:20 UTC");
    expect(findAnswer("little joan", context)).toBeNull();
  });

  it("skip the built-ins switched off in Settings", () => {
    expect(findAnswer("= 12*7", { ...context, disabled: ["builtin:calc"] })).toBeNull();
    expect(findAnswer("10 km in mi", { ...context, disabled: ["builtin:calc"] })?.title).toBe("6.213711922 mi");
    expect(findAnswer("1700000000", { ...context, disabled: ["builtin:timestamp"] })).toBeNull();
    expect(BUILT_INS.map((b) => b.id)).toEqual(["builtin:calc", "builtin:units", "builtin:time", "builtin:random", "builtin:timestamp"]);
  });

  it("come first in the command bar, and Enter copies the answer", () => {
    const { rows } = buildRows("= 12*7", [], { quickAnswer: (q) => quickAnswer(q, context) });
    expect(rows[0]).toEqual({
      kind: "answer",
      title: "84",
      domain: "= 12*7",
      host: "",
      icon: "answer",
      action: { kind: "copy", text: "84" },
    });
    expect(rows.map((row) => row.kind)).toEqual(["answer", "search", "settings"]);
  });
});
