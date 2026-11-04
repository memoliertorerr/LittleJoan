// What the smoke harness may change on the machine it runs on. A few runs change the machine
// itself — the macOS runner's own Reduce Transparency around the Liquid Glass runs (LJ19.9) — which
// is fine on a disposable CI runner and never fine on a developer's computer, whose settings are
// theirs. So every such write asks `mayChangeThisMachine` first, and only a GitHub-hosted runner
// says yes: GITHUB_ACTIONS=true (GitHub sets it in every job — but `act` and self-hosted runners set
// it too) and RUNNER_ENVIRONMENT=github-hosted (GitHub's own machines; `self-hosted` elsewhere).
// Elsewhere the run logs that it skipped the write and goes on without it. Pure, unit-tested
// (smoke-runner.test.ts).

/** Whether the harness runs on a GitHub-hosted Actions runner, the only machine it may change. */
export function mayChangeThisMachine(env: Readonly<Record<string, string | undefined>>): boolean {
  return env.GITHUB_ACTIONS === "true" && env.RUNNER_ENVIRONMENT === "github-hosted";
}

/** The line the smoke logs once: what the runner says it is. */
export function runnerLine(env: Readonly<Record<string, string | undefined>>): string {
  return `runner: GITHUB_ACTIONS=${env.GITHUB_ACTIONS ?? "-"} RUNNER_ENVIRONMENT=${env.RUNNER_ENVIRONMENT ?? "-"} → ${mayChangeThisMachine(env) ? "may" : "may not"} change this machine`;
}

/**
 * A GitHub Actions job that doesn't say what runner it is would skip every machine change — and
 * with them the handoff run's checks — without a word: that fails the smoke instead.
 */
export function runnerFailures(env: Readonly<Record<string, string | undefined>>): string[] {
  return env.GITHUB_ACTIONS === "true" && env.RUNNER_ENVIRONMENT === undefined
    ? ["runner: a GitHub Actions job without RUNNER_ENVIRONMENT — the runner-only checks would be skipped"]
    : [];
}

/** The line a run logs instead of `what` when it may not change this machine. */
export function skippedChange(what: string): string {
  return `not on a GitHub-hosted runner: skipped ${what} (the harness never changes a developer's own settings)`;
}
