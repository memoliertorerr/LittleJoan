import { describe, expect, it } from "vitest";
import { mayChangeThisMachine, runnerFailures, runnerLine, skippedChange } from "./smoke-runner.mts";

const hosted = { GITHUB_ACTIONS: "true", RUNNER_ENVIRONMENT: "github-hosted" };

describe("what the smoke harness may change on its machine", () => {
  it("changes only a GitHub-hosted Actions runner", () => {
    expect(mayChangeThisMachine(hosted)).toBe(true);
  });

  it("leaves every other machine alone — act and self-hosted runners too", () => {
    for (const env of [
      {},
      { GITHUB_ACTIONS: undefined },
      { GITHUB_ACTIONS: "" },
      { GITHUB_ACTIONS: "false" },
      { GITHUB_ACTIONS: "1" },
      { GITHUB_ACTIONS: "TRUE" },
      { CI: "true" },
      { GITHUB_ACTIONS: "true" },
      { GITHUB_ACTIONS: "true", RUNNER_ENVIRONMENT: "self-hosted" },
      { GITHUB_ACTIONS: "true", RUNNER_ENVIRONMENT: "GitHub-Hosted" },
      { RUNNER_ENVIRONMENT: "github-hosted" },
    ]) {
      expect(mayChangeThisMachine(env), JSON.stringify(env)).toBe(false);
    }
  });

  it("fails a GitHub job that doesn't say what runner it is", () => {
    expect(runnerFailures(hosted)).toEqual([]);
    expect(runnerFailures({ GITHUB_ACTIONS: "true", RUNNER_ENVIRONMENT: "self-hosted" })).toEqual([]);
    expect(runnerFailures({})).toEqual([]);
    expect(runnerFailures({ GITHUB_ACTIONS: "true" })[0]).toContain("RUNNER_ENVIRONMENT");
  });

  it("says what it skipped, and what the runner is", () => {
    expect(skippedChange("writing reduceTransparency")).toContain("skipped writing reduceTransparency");
    expect(runnerLine(hosted)).toBe("runner: GITHUB_ACTIONS=true RUNNER_ENVIRONMENT=github-hosted → may change this machine");
    expect(runnerLine({})).toContain("RUNNER_ENVIRONMENT=- → may not");
  });
});
