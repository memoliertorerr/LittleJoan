# Contributing to Little Joan

Thanks for helping. Little Joan is small on purpose — it should stay light at rest and minimal
on screen — so please open an issue to talk about a feature before building it.

## Build and check

Everything (toolchains, caches, temp files) stays inside the repository's `.cache/` folder:

```bash
source scripts/env.sh && bash scripts/setup-rust.sh   # once: Rust toolchain into .cache/
source scripts/env.sh && npm ci
source scripts/env.sh && npm run verify               # must pass before every commit
```

`npm run verify` runs the TypeScript typecheck, ESLint, the Vitest tests, the UI build,
`cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`. On Linux you also need the
WebKitGTK and AppIndicator development packages (see `.github/workflows/ci.yml`).
`source scripts/env.sh && npm run dev` runs the app from source.

CI runs the same checks on Windows, macOS, and Linux, builds the installers, then installs each
one and drives the installed app (`scripts/ci/smoke.mts`). A pull request needs all of it green.

## Ground rules

- **Web pages never get IPC.** Only Little Joan's own screens may call app commands — the bar
  webview (`joan-bar`) and the settings window — each through a capability that lists exactly
  what it needs (`src-tauri/capabilities/`). The page webview (`joan-page`) gets no capability at
  all, not even for local pages (`capability_tests.rs` enforces it). Don't add `remote` URLs to a
  capability or inject scripts into web pages.
- **Nothing runs at rest.** No window or webview may exist while Little Joan hibernates.
- Keep decisions in small, tested, pure modules (`src-tauri/src/*.rs` without Tauri types,
  `ui/*-logic.ts`); keep the native glue thin.
- TypeScript in `strict` mode; Rust with clippy warnings as errors. Match the style of the code
  around your change.
- Update `README.md`, `docs/architecture.md`, and `CHANGELOG.md` when behavior changes.

## Commits and pull requests

One logical change per commit, with a message that says what changed and why. Open the pull
request against `main` unless a maintainer asks otherwise, and describe how you checked it —
including anything that needs a real desktop to try.

## Reporting a security problem

Please don't open a public issue; see [SECURITY.md](SECURITY.md).
