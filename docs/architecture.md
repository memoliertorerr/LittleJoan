# Little Joan — architecture

Little Joan is a Tauri 2 app: a Rust process that owns everything native (tray, global shortcut,
windows, the OS webview), plus two tiny local pages written in TypeScript and bundled by Vite.
The plan and its decisions live in [BUILD_PLAN.md](BUILD_PLAN.md).

## Process model

- **At rest (hibernating):** one native process with a tray / menu-bar icon. There is no
  window and no webview, so no webview helper processes exist (WebView2's `msedgewebview2.exe`,
  WebKit's `WebContent`/`WebKitWebProcess`). `tauri.conf.json` declares `"windows": []`.
- **Closing the last window never quits.** `RunEvent::ExitRequested` without an exit code is
  prevented; only Quit in the tray menu exits.
- **macOS:** `ActivationPolicy::Accessory` — a menu-bar icon and no Dock icon.

## Files

```
package.json, package-lock.json   npm scripts (verify, icons, dev); all JS tooling is local
tsconfig.json                      strict TS for ui/, scripts/, tests/, config files
vite.config.mts                    UI build: root ui/, output dist/ (embedded into the binary)
vitest.config.mts                  TS unit tests: ui/**, scripts/**, tests/** *.test.ts
eslint.config.mjs                  typescript-eslint recommended (type-checked)
rust-toolchain.toml                pinned Rust (minimal profile + rustfmt + clippy)
scripts/env.sh                     source first: keeps caches/toolchains/temp/dev data in .cache/
scripts/setup-rust.sh              installs rustup + the pinned toolchain into .cache/
scripts/png.mts                    dependency-free PNG encoder
scripts/icon-geometry.mts          the original icon artwork as pure shape functions (+ test)
scripts/make-icons.mts             renders the app icon and the macOS tray template
scripts/copy-icons.mts             copies the desktop icon set from `tauri icon` output
ui/index.html, ui/start.ts         the start page (placeholder until LJ3)
ui/style.css                       shared styles; light/dark via prefers-color-scheme
src-tauri/Cargo.toml               crate little-joan (lib little_joan_lib + bin)
src-tauri/build.rs                 tauri-build
src-tauri/tauri.conf.json          product config: no windows at startup, CSP, bundle icons
src-tauri/icons/                   generated icons (npm run icons); tray-template*.png for macOS
src-tauri/src/main.rs              calls little_joan_lib::run()
src-tauri/src/lib.rs               builder, setup, run loop (prevent exit on last window)
src-tauri/src/tray.rs              tray icon + menu (Quit)
src-tauri/src/input.rs             start-field input → http(s) URL or a search (SearchEngine)
src-tauri/src/navigation.rs        which URLs the page may navigate to / hand to the OS browser
src-tauri/src/translate.rs         Google Translate proxy URL, language codes, LANGUAGES list
src-tauri/src/settings.rs          Settings (settings.json): tolerant per-field parse, atomic save
src-tauri/src/toggle.rs            the toggle table: (window state, command) → action
src-tauri/src/cli.rs               --toggle / --show [url] / --hide parsing
.github/workflows/ci.yml           verify on ubuntu-latest, windows-latest, macos-latest
```

## Pure core (no Tauri types, unit-tested)

The modules below hold every decision the app makes; the native glue only calls them.

- **input** — trims the text; an absolute `http`/`https` URL is used as is; `host[:port][/…]`
  gets `https://` (`http://` for `localhost`, `*.localhost`, and IP literals); everything else,
  including `javascript:`, `file:`, `data:`, `about:`, is searched for with the chosen engine
  (Google, DuckDuckGo, Bing, Brave). The start field can never open a non-web URL.
- **navigation** — the page may go to `http`, `https`, `about`, `blob`, `data`, and the app's own
  origin; `file:`, `javascript:`, and app-launching custom schemes are refused. Only `http(s)`
  URLs are ever handed to the OS default browser.
- **translate** — `https://translate.google.com/translate?sl=auto&tl=<lang>&hl=<lang>&u=<page>`,
  which redirects to `<host>.translate.goog`; skipped for non-web pages and pages that are
  already translated.
- **settings** — `{version, shortcut, searchEngine, translateTo, windowWidth, windowHeight}`
  (camelCase JSON). Each field falls back to its default on its own; sizes are clamped to
  320–4096 × 300–4096; saves go through a temp file + rename.
- **toggle** — what a shortcut press or a command-line request does:

  | State             | Toggle  | Show(None) | Show(url)          | Hide    |
  |-------------------|---------|------------|--------------------|---------|
  | Hibernating       | Create  | Create     | Create(url)        | Nothing |
  | Open, focused     | Destroy | Focus      | FocusAndNavigate   | Destroy |
  | Open, not focused | Focus   | Focus      | FocusAndNavigate   | Destroy |

- **cli** — `--toggle`, `--show [http(s) url]`, `--hide`; the last flag wins; unknown arguments
  are ignored; `--show` accepts only absolute web URLs.

## Building

Everything runs from the repo root with the environment from `scripts/env.sh`, which points the
npm cache, `TMPDIR`, `RUSTUP_HOME`, `CARGO_HOME`, and `CARGO_TARGET_DIR` at `.cache/`
(gitignored), so a build never writes to the rest of the machine:

```bash
source scripts/env.sh && bash scripts/setup-rust.sh   # once: rustup + pinned toolchain
source scripts/env.sh && npm ci
source scripts/env.sh && npm run verify               # tsc, eslint, vitest, vite build,
                                                      # rustfmt, clippy -D warnings, cargo test
```

`npm run verify` builds the UI before any cargo command because `tauri::generate_context!`
embeds `dist/` at compile time.

On Linux, Tauri needs the WebKitGTK and AppIndicator development packages (see the CI workflow
for the list).

### Why the GUI is not launched on the development machine

A running Tauri app writes outside the repo (WebKit data under `~/Library` on macOS, the
single-instance socket in `/tmp`) and registers a system-wide shortcut. Development stays
inside the repo, so the app is exercised in CI (and by people on their own machines); local
checks are the compiler, clippy, and unit tests.

## CI

`.github/workflows/ci.yml` runs `npm run verify` on Ubuntu, Windows, and macOS for every pull
request (any base branch — the phase PRs are stacked) and on pushes to `main`. It is the only
compiler for the Windows- and Linux-specific code paths.
