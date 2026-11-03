import { describe, expect, it } from "vitest";
import { mayChangeThisMachine, skippedChange } from "./smoke-runner.mts";

describe("what the smoke harness may change on its machine", () => {
  it("changes only a GitHub Actions runner", () => {
    expect(mayChangeThisMachine({ GITHUB_ACTIONS: "true" })).toBe(true);
  });

  it("leaves every other machine alone", () => {
    for (const env of [{}, { GITHUB_ACTIONS: undefined }, { GITHUB_ACTIONS: "" }, { GITHUB_ACTIONS: "false" }, { GITHUB_ACTIONS: "1" }, { GITHUB_ACTIONS: "TRUE" }, { CI: "true" }]) {
      expect(mayChangeThisMachine(env), JSON.stringify(env)).toBe(false);
    }
  });

  it("says what it skipped", () => {
    expect(skippedChange("writing reduceTransparency")).toContain("skipped writing reduceTransparency");
  });
});
