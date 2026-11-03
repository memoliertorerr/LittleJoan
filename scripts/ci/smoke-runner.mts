// What the smoke harness may change on the machine it runs on. A few runs change the machine
// itself — the macOS runner's own Reduce Transparency around the Liquid Glass runs (LJ19.9) — which
// is fine on a disposable CI runner and never fine on a developer's computer, whose settings are
// theirs. So every such write asks `mayChangeThisMachine` first, and only a GitHub Actions runner
// (GITHUB_ACTIONS=true, which GitHub sets in every job) says yes; elsewhere the run logs that it
// skipped the write and goes on without it. Pure, unit-tested (smoke-runner.test.ts).

/** Whether the harness runs on a GitHub Actions runner, the only machine it may change. */
export function mayChangeThisMachine(env: Readonly<Record<string, string | undefined>>): boolean {
  return env.GITHUB_ACTIONS === "true";
}

/** The line a run logs instead of `what` when it may not change this machine. */
export function skippedChange(what: string): string {
  return `not on a GitHub Actions runner: skipped ${what} (the harness never changes a developer's own settings)`;
}
