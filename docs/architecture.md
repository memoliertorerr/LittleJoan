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
README.md, LICENSE                 what it is, install, use; MIT
CHANGELOG.md                       what each version contains
CONTRIBUTING.md, SECURITY.md       how to build and check changes; how to report security problems
package.json, package-lock.json   npm scripts (verify, icons, dev); all JS tooling is local
tsconfig.json                      strict TS for ui/, scripts/, tests/, config files
vite.config.mts                    UI build: root ui/, output dist/ (embedded into the binary)
vitest.config.mts                  TS unit tests: ui/**, scripts/**, tests/** *.test.ts — each
                                   *.test.ts sits next to the module it tests (Rust tests are
                                   #[cfg(test)] modules inside each .rs file)
eslint.config.mjs                  typescript-eslint recommended (type-checked)
rust-toolchain.toml                pinned Rust (minimal profile + rustfmt + clippy)
scripts/env.sh                     source first: keeps caches/toolchains/temp/dev data in .cache/
scripts/setup-rust.sh              installs rustup + the pinned toolchain into .cache/
scripts/png.mts                    dependency-free PNG encoder
scripts/icon-geometry.mts          the original icon artwork as pure shape functions (+ test)
scripts/make-icons.mts             renders the app icon and the macOS tray template
scripts/copy-icons.mts             copies the desktop icon set from `tauri icon` output
ui/bar.html, ui/bar.ts             the URL bar (its own webview above the page); DOM glue only
ui/bar-logic.ts                    URL bar logic (Enter / Esc / never clobber an edit), unit-tested
ui/index.html, ui/start.ts         the start page under the bar: a hint, nothing to type into
ui/start-logic.ts                  the start page's StartInfo type
ui/lib/shortcut.ts                 accelerator ↔ key press (recorder) and display text ("⌘⇧Space")
ui/settings.html, ui/settings.ts   the settings window: shortcut recorder, search engine, language
ui/settings-logic.ts               settings types + change detection, unit-tested
ui/style.css                       shared styles; light/dark via prefers-color-scheme
src-tauri/Cargo.toml               crate little-joan (lib little_joan_lib + bin)
src-tauri/build.rs                 tauri-build + the app manifest (which commands exist)
src-tauri/capabilities/bar.json    the URL bar may call open_input, page_url, focus_page, hibernate
src-tauri/capabilities/joan.json   the page webview's local start page may call start_info
src-tauri/capabilities/settings.json  settings window: get/save settings, suspend shortcut, close
src-tauri/tauri.conf.json          product config: no windows at startup, CSP, bundle icons
src-tauri/icons/                   generated icons (npm run icons); tray-template*.png for macOS
src-tauri/src/main.rs              calls little_joan_lib::run()
src-tauri/src/lib.rs               builder: plugins, commands, setup, run loop
src-tauri/src/state.rs             AppState (settings, data dir, active shortcut) behind a Mutex
src-tauri/src/joan.rs              the page window: create / focus / destroy, window events
src-tauri/src/bar.rs               its two webviews (URL bar + page): build, layout, fullscreen,
                                   pushing the page's address to the bar, focus
src-tauri/src/layout.rs            where the bar and the page go (pure, tested)
src-tauri/src/placement.rs         where a new window opens (monitor under the cursor)
src-tauri/src/shortcut.rs          global-shortcut plugin, registration with fallback, live swap
                                   with rollback (Registrar trait), suspend while recording
src-tauri/src/commands.rs          start page: open_input, hibernate, start_info;
                                   settings: get_settings, save_settings, suspend_shortcut
src-tauri/src/settings_window.rs   the settings window (on demand, destroyed on close)
src-tauri/src/smoke.rs             LITTLEJOAN_SMOKE_LOG event log for CI smoke tests
src-tauri/src/fullscreen.rs        FullscreenTracker: element fullscreen ↔ window fullscreen
src-tauri/src/keys.rs              page keys per OS (pure), app origin
src-tauri/src/page_actions.rs      close / open location / reload / back / forward / translate /
                                   open in browser; translate_target (skips the start page)
src-tauri/src/menu.rs              macOS app menu: Edit (copy/paste in WKWebView) + page keys
src-tauri/src/platform/mod.rs      installs the per-OS hooks on the bar and the page webviews
src-tauri/src/platform/win.rs      Windows: WebView2 AcceleratorKeyPressed → keys.rs;
                                   ContainsFullScreenElementChanged → window fullscreen
src-tauri/src/platform/linux.rs    Linux: GTK window key-press-event → keys.rs; the bar's packing
                                   in the window's GtkBox; enter/leave-fullscreen → hide the bar
src-tauri/permissions/autogenerated/  per-command permissions written by tauri-build
src-tauri/src/tray.rs              tray icon + menu (Open, Translate Page, Open Page in Browser,
                                   Settings…, Quit); page items enabled only while a page is open
src-tauri/src/input.rs             start-field input → http(s) URL or a search (SearchEngine)
src-tauri/src/navigation.rs        which URLs the page may navigate to / hand to the OS browser
src-tauri/src/translate.rs         Google Translate proxy URL, language codes, LANGUAGES list
src-tauri/src/settings.rs          Settings (settings.json): tolerant per-field parse, atomic save
src-tauri/src/toggle.rs            the toggle table: (window state, command) → action
src-tauri/src/cli.rs               --toggle / --show [url] / --hide parsing
scripts/ci/smoke.mts               CI: drives the installed app (show, page, click, hide), screenshots
scripts/ci/smoke-log.mts           CI: smoke-log parsing and pass/fail rules, unit-tested
tests/smoke/smoke.html             CI smoke page: codec report + a big "Go fullscreen" button
tests/smoke/smoke-logic.mjs        its logic (plain JS, JSDoc-typed, unit-tested)
.github/workflows/ci.yml           verify on ubuntu-latest, windows-latest, macos-latest
```

## Lifecycle

```
launch ──► hibernating: tray icon + global shortcut, no window, no webview
              │  shortcut / --toggle / --show [url] / second launch
              ▼
           joan window created: URL bar on top (focused), start page below
              │  type in the bar ──Enter──► the page loads under the bar
              │  shortcut while focused / Esc on start page / close button / --hide
              ▼
           window + webview destroyed (page discarded, size remembered) ──► hibernating
```

- Every request goes through `toggle::decide` (table below) and `joan::handle`, which runs off
  the main thread (`joan::dispatch`): building a webview window from the main thread can
  deadlock on Windows. A lifecycle mutex keeps two quick presses from creating two windows.
- The window opens centred in the work area of the monitor under the mouse cursor, at the last
  size the user left it (logical pixels, saved on close unless fullscreen/maximized/minimized).
- **Every close goes through the same path.** The close button, Cmd+W / Alt+F4
  (`CloseRequested` is prevented and turned into `--hide`), Esc on the start page, the shortcut,
  and `--hide` all end in `joan::destroy`: remember the size, release the page, destroy the
  window.
- **Releasing the page on macOS:** WKWebView's page processes were measured outliving a
  destroyed window by minutes (which would also let a video's audio keep playing). On macOS
  the window is hidden at once, navigated to `about:blank`, and destroyed 400 ms later; CI then
  sees the `WebContent` processes gone within 30 s and no growth across summon cycles. WebView2
  and WebKitGTK release theirs on destroy without help.
- **Single instance:** `tauri-plugin-single-instance` forwards a second launch's arguments to
  the running app (`--toggle`, `--show [url]`, `--hide`; no flags = `--show`). On Linux it
  needs a D-Bus session bus; on macOS it uses a socket in `/tmp`.
- **Global shortcut:** registered from settings; if it fails to parse or is taken, the default
  (`CommandOrControl+Shift+Space`) is tried; if that fails too, the app still runs. Linux
  global shortcuts work on X11 only — on Wayland, bind `little-joan --toggle` in the desktop's
  keyboard settings.

## Commands and capabilities

Remote pages get no IPC. `build.rs` declares the app's commands in an app manifest, which makes
each one require a permission, and capabilities grant them **per webview label**:
`capabilities/bar.json` gives the URL bar (`joan-bar`) `allow-open-input`, `allow-page-url`,
`allow-focus-page`, `allow-hibernate`; `capabilities/joan.json` gives the page webview
(`joan-page`) only `allow-start-info`, and only for its local start page — capabilities are
local unless they list `remote` URLs, and none does. No `core:default` anywhere.

| Command | Called by | Does |
|---|---|---|
| `open_input(input)` | URL bar, Enter | `input::normalize` with the configured engine → load it in the page webview, focus the page |
| `page_url()` | URL bar, on load | what the bar shows now (`null` on the start page) |
| `focus_page()` | URL bar, Esc | give the keyboard back to the page |
| `hibernate()` | URL bar, Esc on the start page | close the window (same path as `--hide`) |
| `start_info()` | start page, on load | `{ shortcut, platform }` for the hint line |
| `get_settings()` | settings window | settings + engine and language lists + platform |
| `save_settings(settings)` | settings window, Save | validate; swap the shortcut live (old one restored if the new one is taken); save |
| `suspend_shortcut(suspended)` | settings window, recorder | pause / resume the active shortcut while recording |

`capabilities/settings.json` grants the `settings` window the last three plus
`core:window:allow-close` (its Cancel / Save buttons close it).

## Settings

`settings.json` lives in the app's config folder:

| OS | Path |
|---|---|
| Windows | `%APPDATA%\io.github.memoliertorerr.littlejoan\settings.json` |
| macOS | `~/Library/Application Support/io.github.memoliertorerr.littlejoan/settings.json` |
| Linux | `~/.config/io.github.memoliertorerr.littlejoan/settings.json` |

The settings window (tray → Settings…, or Cmd+, on macOS) records a shortcut by listening to the
next key press. While it listens, the active global shortcut is **suspended** (unregistered) so
pressing it doesn't summon or dismiss the page window; it is resumed when recording stops, when
Save runs, and when the window closes for any reason. A new shortcut is registered before the
old one is forgotten: if registration fails, the old one is registered again and the window
shows the error.

## The page

- **Navigation guard** (`on_navigation` → `navigation::is_allowed_navigation`): web schemes,
  engine-internal schemes, and the app origin only.
- **Title:** the window title follows the page title (`on_document_title_changed`).
- **New windows** (`target=_blank`, `window.open`): denied inside Little Joan; web URLs are
  handed to the OS default browser (`tauri_plugin_opener::open_url`, used as a plain function —
  the opener plugin and its JS commands are not registered).
- **Keys:** remote pages get no IPC and no injected scripts, so page keys are caught natively
  before the page sees them:

  | Keys | Windows (WebView2 `AcceleratorKeyPressed`) | Linux (GTK `key-press-event`) | macOS (app menu) |
  |---|---|---|---|
  | Close window | Ctrl+W | Ctrl+W | Cmd+W |
  | Edit the address (focus the URL bar, text selected) | Ctrl+L | Ctrl+L | Cmd+L |
  | Reload | Ctrl+R / F5 (WebView2 built-in) | Ctrl+R / F5 | Cmd+R |
  | Back / Forward | Alt+Left / Alt+Right (built-in) | Alt+Left / Alt+Right | Cmd+[ / Cmd+] |
  | Translate page | Ctrl+Shift+T | Ctrl+Shift+T | Cmd+Shift+T |
  | Open page in default browser (then hibernate) | tray menu | tray menu | Cmd+Shift+O, tray menu |
  | Copy / paste / select all | built-in | built-in | Edit menu (needed by WKWebView) |

  Matches are exact: Ctrl+Shift+W is not Ctrl+W, and Ctrl+Alt (AltGr) never matches. On macOS
  the menu bar is hidden (Accessory app) but its key equivalents work; Cmd+Q quits for real.
- The app origin is `tauri://localhost` on macOS/Linux and `http://tauri.localhost` on Windows
  (`keys::app_origin`).

## The URL bar

The page window is a plain `tauri::Window` with two child webviews (Tauri 2 multi-webview,
which needs Tauri's `unstable` feature in 2.12): **`joan-bar`**, a trusted local page
(`ui/bar.html`) 40 logical px tall at the top, and **`joan-page`**, where web pages load. The bar
is never injected into a page; it is a separate webview with its own capability, and the page
webview still gives remote content no IPC.

- **Layout** (`layout.rs`, `bar::apply_layout`): bar at the top, page below; during element
  fullscreen the bar is hidden and the page fills the window. Windows and macOS place child
  webviews by bounds, re-applied on `Resized` / `ScaleFactorChanged`. Linux packs child
  webviews into the window's vertical GtkBox, where bounds don't apply: the bar is packed with
  `expand = false` at a 40 px height request and the page takes the rest; hiding the bar lets the
  page fill the window.
- **Address:** the bar always shows the page's full address (empty, with the placeholder, on the
  start page). Rust pushes changes with `bar.eval("window.littleJoan.setUrl(<json>)")` on
  every page load (started and finished, so redirects land), every title change, after
  navigations the app starts, and from a once-a-second check while the window exists (for
  `history.pushState` / `#anchor` changes, which raise no event). The bar asks once on load
  (`page_url`) so it can't miss the first address. Changes are de-duplicated.
- **Editing:** click the bar or press Ctrl/Cmd+L — the text is selected. Enter opens it in the
  page (address or search, as before) and moves the keyboard to the page. Esc reverts an edit;
  with nothing to revert it returns to the page, or on the start page closes the window.
  Leaving the field without Enter drops the edit. An address update never overwrites an edit in
  progress.
- **Element fullscreen:** Windows hides the bar in the `ContainsFullScreenElementChanged`
  handler; Linux on WebKitGTK's `enter-fullscreen` / `leave-fullscreen` signals; macOS needs
  nothing (WebKit shows the element in its own fullscreen window).

## Fullscreen video

HTML5 element fullscreen (a video's fullscreen button) must fill the screen:

| Engine | What happens | Code |
|---|---|---|
| WebView2 (Windows) | The engine only fills the webview's bounds and raises `ContainsFullScreenElementChanged`; the app makes the window fullscreen and back. The change is queued off the callback because resizing re-enters WebView2. | `platform/win.rs` + `fullscreen.rs` |
| WKWebView (macOS 12.3+) | wry enables `elementFullscreenEnabled`; WebKit moves the element into its own fullscreen window (its own Space). | none |
| WebKitGTK (Linux) | The default `enter-fullscreen` handler fullscreens the toplevel GTK window. | none |

`FullscreenTracker` only undoes a fullscreen it caused, so a window the user made fullscreen
stays fullscreen when a video leaves it.

**Codecs** come from the engine: WebView2 has Edge's (H.264, AAC, VP9, AV1); WKWebView has
Safari's (H.264, HEVC, VP9 on recent macOS); WebKitGTK uses GStreamer, so Linux needs
`gstreamer1.0-plugins-good`, `gstreamer1.0-plugins-bad`, and `gstreamer1.0-libav` (the `.deb`
depends on them; the AppImage bundles them). `tests/smoke/smoke.html` reports what each engine
supports (`LJ-SMOKE ready h264=… vp9=… av1=…` in the title).

## Translate and open in browser

- **Translate Page** navigates the page window to Google Translate's proxy for the current
  page (`translate::translate_url`, language from settings). It does nothing on the start page,
  on non-web pages, and on pages that are already translated.
- **Open Page in Browser** hands the current web page to the OS default browser and then
  hibernates, like Little Arc's "open in the main browser".
- Both live in the tray menu (enabled only while a web page is open — updated on page load and
  on hibernate), the macOS File menu, and (translate only) the page keys.

## Smoke log

When `LITTLEJOAN_SMOKE_LOG` names a file, the app appends `<unix millis> <event>[ <detail>]`
lines: `started`, `shortcut-registered <accel>`, `shortcut-failed <reason>`,
`window-created`, `title <page title>`, `page-loaded <url>`, `fullscreen <true|false>` (any
engine: the window's fullscreen state changed), `element-fullscreen <true|false>` (Windows: the
app changed it for a fullscreen element), `bar-loaded` (the URL bar's page finished loading),
`bar-url <address | start>` (what the bar was told to show), `hibernated`. Without the
variable nothing is logged.

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

## Packaging

Installers are built in CI only (`package` job), never on a developer machine:

| OS | Runner | Command | Artifact |
|---|---|---|---|
| Windows | windows-latest | `tauri build --bundles nsis` | `Little Joan_<v>_x64-setup.exe` — per-user install (no admin), downloads the WebView2 runtime if missing |
| macOS | macos-latest | `tauri build --target universal-apple-darwin --bundles app,dmg` | `Little Joan_<v>_universal.dmg` — Intel + Apple silicon, ad-hoc signed (`signingIdentity: "-"`), macOS 12.3+ |
| Linux | ubuntu-22.04 | `tauri build --bundles deb,appimage` | `.deb` (depends on the GStreamer plugins for video) and `.AppImage` (bundles the media framework) |

The release profile optimizes for size (`opt-level = "s"`, LTO, one codegen unit, stripped,
`panic = "abort"`). Builds are not notarized or Authenticode-signed (see the plan's Later
section), so macOS Gatekeeper and Windows SmartScreen warn on first launch.

## CI

`.github/workflows/ci.yml`, for every pull request (any base branch — the phase PRs are stacked)
and pushes to `main`:

- **verify** — `npm run verify` on Ubuntu, Windows, and macOS. It is the only compiler for the
  Windows- and Linux-specific code paths.
- **package** — the installers above, uploaded as artifacts `little-joan-<os>` (14 days), with
  their sizes in the job summary.
- **smoke** — installs each installer the way a user would (NSIS `/S`; copy the `.app` out of
  the mounted `.dmg`; `apt install ./….deb`) and runs `scripts/ci/smoke.mts` against the
  installed binary: launch with `LITTLEJOAN_SMOKE_LOG`, `--show` (start page), `--show
  http://127.0.0.1:<port>/smoke.html`, click the screen centre (the page's fullscreen button),
  `--hide`, summon and hide again, then check the log order, single-instance forwarding, window
  fullscreen (Windows, Linux), the URL bar (`bar-loaded`, and `bar-url` with the smoke page's
  address), and that none of the webview helper processes that served the page outlive the
  window (followed by process id, so other apps' WebKit helpers on the runner don't count).
  Screenshots (`hibernating.png`, `start.png`, `page.png`, `fullscreen.png`), the log, and
  `summary.json` (memory — resident and the app's own: private bytes / private dirty /
  physical footprint —, helper counts sampled for 60 s and after three summon cycles, codecs)
  are uploaded as `smoke-<os>`. Linux runs under `xvfb-run` + `dbus-run-session`
  (single-instance needs a session bus) with openbox, and under gdb so a crash leaves a
  backtrace; the AppImage gets a launch check.
- **Smoke-only reruns** (manual `workflow_dispatch`, for diagnosing without rebuilding):
  `installers_from_run` (smoke the installers of an earlier run; skips verify and package),
  `extra_env` (e.g. `WEBKIT_DISABLE_DMABUF_RENDERER=1`), `linux_runner` (`ubuntu-24.04`),
  `linux_window_manager` (`none`), `linux_plain_webkit_check` (repeat the fullscreen click in a
  bare WebKitGTK window built with PyGObject — no Tauri — to tell engine crashes from ours).

  ```bash
  gh workflow run ci.yml --repo memoliertorerr/LittleJoan --ref <branch> \
    -f installers_from_run=<run id> -f linux_plain_webkit_check=true
  ```

### Measured in CI (v0.1, run 38936095643)

| | Windows (WebView2) | macOS (WKWebView) | Linux (Ubuntu 24.04, WebKitGTK 2.52.6, Xvfb) |
|---|---|---|---|
| Hibernating at start: resident / own | 12.8 MB / 2.2 MB private | 66 MB / 11 MB footprint | 97 MB / 19.5 MB private dirty |
| Hibernating again after a page: own | 4.5 MB | 24 MB | 44.6 MB |
| Webview helpers while a page is open | 6 `msedgewebview2.exe` | 3 `WebContent` (+ GPU, Networking) | 1 `WebKitWebProcess` |
| Helpers after hibernating (2 s … 60 s, and after 3 cycles) | 0 | 0 | 0 |
| Element fullscreen after a click | window fullscreen ✔ | own Space ✔ | window fullscreen ✔ (Ubuntu 22.04's WebKitGTK 2.50.4 aborts — see the plan's Landmines) |
| Codecs (H.264 / VP9 / AV1 via MSE) | ✔ / ✔ / ✔ | ✔ / ✔ / ✔ | ✔ / ✔ / ✔ |

Resident memory counts shared system libraries (WebKit is large); the "own" column is what the
app itself holds.
