# Little Joan — architecture

Little Joan is a Tauri 2 app: a Rust process that owns everything native (tray, global shortcut,
windows, the OS webview), plus a few tiny local pages written in TypeScript and bundled by Vite
(the bar, the settings window, and a plain first page for the page webview).
The plan and its decisions live in [BUILD_PLAN.md](BUILD_PLAN.md).

## Process model

- **At rest (hibernating):** one native process with a tray / menu-bar icon. There is no
  window and no webview, so no webview helper processes exist (WebView2's `msedgewebview2.exe`,
  WebKit's `WebContent`/`WebKitWebProcess`). `tauri.conf.json` declares `"windows": []`.
- **Closing the last window never quits.** `RunEvent::ExitRequested` without an exit code is
  prevented; only Quit in the tray menu and `--quit` (both `joan::quit`), and Cmd+Q on macOS
  (the app menu's standard Quit item), exit.
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
ui/bar.html, ui/bar.ts             the bar (its own webview): the toolbar above the page, or the
                                   command bar over the whole window; DOM glue only
ui/bar-logic.ts                    the address field: never clobber an edit (unit-tested)
ui/command/view-state.ts           the bar's two views (toolbar, overlay) and what Enter / Esc do
                                   in each (unit-tested)
ui/command/welcome.ts              StartInfo and the first launch's welcome (unit-tested)
ui/command/panel.ts                the welcome in the command bar, with its start-at-login
                                   checkbox; DOM glue only
ui/command/rows.ts                 the command bar's rows: order, autofill, keys, site icons
                                   (pure, unit-tested)
ui/command/view.ts                 draws the rows, as text only (tested with a stand-in document)
ui/command/command.ts              the command bar's list: suggest() as you type, autofill, keys,
                                   mouse; DOM glue only
ui/index.html                      the page webview's first page: a plain background, no script
ui/lib/shortcut.ts                 accelerator ↔ key press (recorder) and display text ("⌘⇧Space")
ui/settings.html, ui/settings.ts   the settings window: shortcut recorder, search engine, language
ui/settings-logic.ts               settings types + change detection, unit-tested
ui/style.css                       shared styles; light/dark via prefers-color-scheme
src-tauri/Cargo.toml               crate little-joan (lib little_joan_lib + bin)
src-tauri/build.rs                 tauri-build + the app manifest (which commands exist)
src-tauri/capabilities/bar.json    the bar may call open_input, page_url, focus_page, set_overlay,
                                   hibernate, history_top, history_search, suggest,
                                   history_remove, open_settings, start_info, set_start_at_login
src-tauri/capabilities/settings.json  settings window: get/save settings, suspend shortcut, set
                                   start at login, close
src-tauri/tauri.conf.json          product config: no windows at startup, CSP, bundle icons
src-tauri/icons/                   generated icons (npm run icons); tray-template*.png for macOS
src-tauri/src/main.rs              calls little_joan_lib::run()
src-tauri/src/lib.rs               builder: plugins, commands, setup, run loop
src-tauri/src/state.rs             AppState (settings, data dir, active shortcut, first run, kept
                                   session cookies) behind a Mutex; first-run detection (tested)
src-tauri/src/cookies.rs           session cookies kept in memory across close → summon where the
                                   engine drops them (WebView2): which, how many (pure, tested)
src-tauri/src/history.rs           history: what is recorded, frecency, top sites, typed matching,
                                   the 5,000-URL cap, history.json parse/serialize (pure; tests
                                   in history/tests.rs, with a fixed clock)
src-tauri/src/history_store.rs     history.json on disk and in AppState while the window exists:
                                   load, record (page load / title events), autosave, atomic save,
                                   remove one, the command bar's suggestions
src-tauri/src/suggest.rs           what the command bar suggests: top sites or the starters,
                                   history matches and a search row (pure, tested)
src-tauri/src/welcome.rs           the welcome's per-OS text: where the tray icon is, Wayland (pure)
src-tauri/src/joan.rs              the page window: create / focus / destroy, window events
src-tauri/src/bar.rs               its two webviews (bar + page): build, pushing the page's
                                   address to the bar, focus, navigation
src-tauri/src/bar/view.rs          the view state: toolbar / overlay / element fullscreen,
                                   applied to the webviews (layout, show / hide, Linux packing)
src-tauri/src/page_address.rs      the page's address from its load events; when the engine may
                                   be asked for it (pure, tested)
src-tauri/src/layout.rs            where the bar and the page go per mode (Toolbar, Overlay,
                                   Fullscreen), from physical size + scale (pure)
src-tauri/src/capability_tests.rs  tests: no capability reaches the page webview, its window, or
                                   remote URLs (Ground Rule 18)
src-tauri/src/phone.rs             phone mode decisions: 600 px threshold, user agents, debounce
src-tauri/src/phone_mode.rs        phone mode at runtime: set the page's user agent, reload
src-tauri/src/placement.rs         where a new window opens (monitor under the cursor)
src-tauri/src/shortcut.rs          global-shortcut plugin, registration with fallback, live swap
                                   with rollback (Registrar trait), suspend while recording
src-tauri/src/commands.rs          bar: open_input, page_url, focus_page, set_overlay, hibernate,
                                   start_info, history_top, history_search, suggest,
                                   history_remove, open_settings; settings:
                                   get_settings, save_settings, suspend_shortcut;
                                   set_start_at_login (both)
src-tauri/src/autostart.rs         start at login: tauri-plugin-autostart (Rust API only), the
                                   macOS translocation guard (pure, tested)
src-tauri/src/settings_window.rs   the settings window (on demand, destroyed on close)
src-tauri/src/smoke.rs             LITTLEJOAN_SMOKE_LOG event log for CI smoke tests
src-tauri/src/fullscreen.rs        FullscreenTracker: element fullscreen ↔ window fullscreen
src-tauri/src/keys.rs              page keys per OS (pure), app origin
src-tauri/src/page_actions.rs      close / open location / reload / back / forward / translate /
                                   open in browser; translate_target (skips the start page)
src-tauri/src/menu.rs              macOS app menu: Edit (copy/paste in WKWebView) + page keys;
                                   Cmd+Q goes through joan::quit
src-tauri/src/platform/mod.rs      installs the per-OS hooks on the bar and the page webviews
src-tauri/src/platform/win.rs      Windows: WebView2 AcceleratorKeyPressed → keys.rs;
                                   ContainsFullScreenElementChanged → window fullscreen
src-tauri/src/platform/mac.rs      macOS: WKWebView customUserAgent (phone mode)
src-tauri/src/platform/linux.rs    Linux: GTK window key-press-event → keys.rs; the bar's packing
                                   in the window's GtkBox (toolbar / overlay); enter/leave-
                                   fullscreen → hide the bar
src-tauri/permissions/autogenerated/  per-command permissions written by tauri-build
src-tauri/src/tray.rs              tray icon + menu (Open, Translate Page, Open Page in Browser,
                                   Settings…, Quit); page items enabled only while a page is open
src-tauri/src/input.rs             start-field input → http(s) URL or a search (SearchEngine)
src-tauri/src/navigation.rs        which URLs the page may navigate to / hand to the OS browser
src-tauri/src/translate.rs         Google Translate proxy URL, language codes, LANGUAGES list
src-tauri/src/settings.rs          Settings (settings.json): tolerant per-field parse, atomic save
src-tauri/src/toggle.rs            the toggle table: (window state, command) → action
src-tauri/src/cli.rs               --toggle / --show [url] / --hide / --size WxH /
                                   --start-at-login on|off / --quit parsing
scripts/ci/smoke.mts               CI: drives the installed app (show, page, click, hide, start at
                                   login on/off, cookies and storage across close and quit),
                                   reads the OS login entry, screenshots
scripts/ci/smoke-log.mts           CI: smoke-log parsing and pass/fail rules, unit-tested
scripts/ci/smoke-command.mts       CI: the command bar's smoke-log rules, unit-tested
tests/smoke/smoke.html             CI smoke page: codec report + a big "Go fullscreen" button;
                                   with ?persist=set|check, the cookie and localStorage check
tests/smoke/smoke-logic.mjs        its logic (plain JS, JSDoc-typed, unit-tested)
.github/workflows/ci.yml           verify on ubuntu-24.04, windows-latest, macos-latest; package; smoke
```

## Lifecycle

```
launch ──► hibernating: tray icon + global shortcut, no window, no webview
              │  shortcut / --toggle / --show [url] / second launch
              ▼
           joan window created with the command bar open (the bar covers the window, focused)
              │  type ──Enter──► the page loads; the bar becomes the 40 px toolbar above it
              │  Ctrl/Cmd+L ──► the command bar again (Esc: back to the page)
              │  shortcut while focused / Esc with no page / close button / --hide
              ▼
           window + webview destroyed (page discarded, size remembered) ──► hibernating
```

- **First launch:** when `settings.json` doesn't exist as `state::init` runs (a file that exists
  but can't be read or parsed doesn't count), setup registers the shortcut as usual, logs
  `first-run`, saves the default settings at once (so the welcome appears only once, even if the
  user quits straight away), and opens the window with `--show`'s path. The command bar (in the
  bar webview) asks `start_info`, gets `firstRun: true` once (reading it clears it), and shows
  the welcome under its field: what the shortcut does (on a Linux Wayland session: bind
  `little-joan --toggle`), where the icon is (`welcome::tray_hint`), a "Start Little Joan at
  login" checkbox (`set_start_at_login`), and "Esc or the shortcut closes this window." The field
  keeps the focus. A first launch with flags (`--show <url>`, `--toggle`, `--start-at-login …`) acts on
  them instead, still saves the settings, and shows no welcome. Every later launch hibernates.
- Every request goes through `toggle::decide` (table below) and `joan::handle`, which runs off
  the main thread (`joan::dispatch`): building a webview window from the main thread can
  deadlock on Windows. A lifecycle mutex keeps two quick presses from creating two windows.
- The window opens centred in the work area of the monitor under the mouse cursor, at the last
  size the user left it (logical pixels, saved on close unless fullscreen/maximized/minimized).
- **Every close goes through the same path.** The close button, Cmd+W / Alt+F4
  (`CloseRequested` is prevented and turned into `--hide`), Esc in the command bar with no page,
  the shortcut, and `--hide` all end in `joan::destroy`: remember the size, keep the session cookies (Windows,
  below), release the page, destroy the window.
- **Quit** (tray, `--quit`, Cmd+Q) is `joan::quit`. On macOS it first closes an open window
  through `destroy` (so the page is unloaded) and waits for that at most 1.5 s: after a plain
  `app.exit` (`process::exit`, webviews never unloaded) CI saw the next launch's WebKit never
  load a page. Windows and Linux exit at once — their stores came through without it.
- **The page's address comes from its load events** (`page_address.rs`, `bar::page_url`): the
  `Started` / `Finished` events carry it, and the app's own navigations set it at once. wry 0.57
  unwraps WKWebView's address, which is unset until the first navigation commits (and again
  after a first navigation that never did), and that panic ends the app, so the engine is asked
  (`Webview::url`) only by the once-a-second check for in-page changes, and only while the last
  load event was `Finished`; an answer that raced with a newer navigation is dropped.
- **Releasing the page on macOS:** WKWebView's page processes were measured outliving a
  destroyed window by minutes (which would also let a video's audio keep playing). On macOS
  the window is hidden at once, navigated to `about:blank`, and destroyed 400 ms later; CI then
  sees the `WebContent` processes gone within 30 s and no growth across summon cycles. WebView2
  and WebKitGTK release theirs on destroy without help.
- **Single instance:** `tauri-plugin-single-instance` forwards a second launch's arguments to
  the running app (`--toggle`, `--show [url]`, `--hide`, `--start-at-login on|off`, `--quit`; no
  flags = `--show`). On Linux it needs a D-Bus session bus; on macOS it uses a socket in `/tmp`.
  `--start-at-login` is handled before the toggle table and never touches the window. `--quit`
  takes the tray's Quit path (`joan::quit`) and wins over any window flag; a `--quit` launch with
  nothing running just ends.
- **Global shortcut:** registered from settings; if it fails to parse or is taken, the default
  (`CommandOrControl+Shift+Space`) is tried; if that fails too, the app still runs. Linux
  global shortcuts work on X11 only — on Wayland, bind `little-joan --toggle` in the desktop's
  keyboard settings.

## Commands and capabilities

**Two trust levels.** The page window holds two webviews. **`joan-bar`** is trusted: every screen
of the app's own lives there — the toolbar, the command bar, the welcome (and, later, history).
**`joan-page`** shows web pages (and, from LJ29, extension panels) and has **no IPC at all**: no
capability names it or the page window (`joan`), so nothing loaded there can call a command —
not a remote page, and not a local one either. That matters because Tauri treats every
custom-scheme page as *local* (`is_local_url`), so an `ljext:` extension page would get every
local capability of its webview. `capability_tests.rs` reads every capability file (and inline
ones in `tauri.conf.json`) and fails if any names `joan-page`, scopes a window pattern covering
`joan`, or lists `remote` URLs. The page webview's first page, `ui/index.html`, is a plain
background with no script.

`build.rs` declares the app's commands in an app manifest, which makes each one require a
permission, and capabilities grant them per webview label or window:

| Capability | Scope | Grants |
|---|---|---|
| `capabilities/bar.json` | webview `joan-bar` | `allow-open-input`, `allow-page-url`, `allow-focus-page`, `allow-set-overlay`, `allow-hibernate`, `allow-history-top`, `allow-history-search`, `allow-suggest`, `allow-history-remove`, `allow-open-settings`, `allow-start-info`, `allow-set-start-at-login` |
| `capabilities/settings.json` | window `settings` | `allow-get-settings`, `allow-save-settings`, `allow-suspend-shortcut`, `allow-set-start-at-login`, `core:window:allow-close` |
| — | `joan-page`, window `joan`, remote URLs | nothing |

Capabilities are local unless they list `remote` URLs, and none does. No `core:default`
anywhere.

| Command | Called by | Does |
|---|---|---|
| `open_input(input)` | the bar, Enter | `input::normalize` with the configured engine → load it in the page webview (the command bar closes), focus the page |
| `page_url()` | the bar, on load | what the bar shows now (`null` on the start page) |
| `focus_page()` | the toolbar, Esc | give the keyboard back to the page |
| `set_overlay(on)` | the bar: a click on the toolbar's address (on), Esc in the command bar with a page (off) | open or close the command bar's overlay; closing gives the keyboard to the page |
| `hibernate()` | the bar, Esc with no page | close the window (same path as `--hide`) |
| `start_info()` | the command bar, on load | `{ shortcut, platform, firstRun, trayHint, startAtLogin, waylandNoShortcut }` for the hint line and the welcome; `firstRun` is true once |
| `get_settings()` | settings window | settings + engine and language lists + platform |
| `save_settings(settings)` | settings window, Save | validate; swap the shortcut live (old one restored if the new one is taken); save |
| `suspend_shortcut(suspended)` | settings window, recorder | pause / resume the active shortcut while recording |
| `set_start_at_login(enabled)` | settings window, Save (only when the switch changed); the welcome's checkbox (the bar) | add / remove the OS login entry; returns whether it exists afterwards (read back) |
| `history_top(n)` | nothing yet (the command bar uses `suggest`; left for the history view) | up to `n` (≤ 50) top sites: `[{ url, title, host, score }]`; read-only |
| `history_search(query, limit)` | nothing yet (as above) | up to `limit` (≤ 50) records matching every word of `query`, same rows; read-only |
| `suggest(query)` | the command bar, as you type (60 ms after the last key) | `[{ kind: topSite \| history \| search \| starter, title, url, host, score }]` (`suggest.rs`): nothing typed → up to 8 top sites, leaving out the open page's site, or the 4 starters when there are none; typing → up to 8 history matches, then one `search` row (title = the engine's name, url = its results page) |
| `history_remove(url)` | the command bar, Shift+Delete on a history row | forget that record (saved with the rest of history); returns whether there was one |
| `open_settings(tab?)` | the command bar's "Settings & extensions" row | open (or focus) the settings window; `tab` is for LJ26's tabs |

`get_settings` also reports `startAtLogin` (`null` when the entry can't be read).
`capabilities/settings.json` grants the `settings` window the last four plus
`core:window:allow-close` (its Cancel / Save buttons close it). tauri-plugin-autostart is
registered for its Rust API only: no capability grants any `autostart:*` permission, so its own JS
commands are denied everywhere.

## Start at login

`autostart.rs` wraps tauri-plugin-autostart (which uses the `auto-launch` crate). **The OS login
entry is the only state** — `settings.json` has no field for it — so removing the entry in the
OS's own settings shows as off the next time Settings opens. Off by default; turned on only in
Settings or with `--start-at-login on`.

| OS | Entry (named after the product name, "Little Joan") | Runs |
|---|---|---|
| Windows | value `Little Joan` under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` (and an "enabled" flag under `…\Explorer\StartupApproved\Run`) | the running `little-joan.exe`, unquoted |
| macOS | `~/Library/LaunchAgents/Little Joan.plist`, `Label` "Little Joan", `RunAtLoad` (never AppleScript: no Automation prompt) | the canonical path of the running executable |
| Linux | `~/.config/autostart/Little Joan.desktop` | the running binary, or `$APPIMAGE` for the AppImage |

- No launch arguments: a plain launch already hibernates.
- Only a change is applied (Windows' disable fails when there is no value), and the result is
  read back from the OS.
- **macOS translocation guard:** an app opened straight from a disk image or from Downloads runs
  from a random `/AppTranslocation/` copy, and a LaunchAgent made then would point at nothing
  after a reboot, so turning it on there fails with "Move Little Joan to Applications first, then
  turn this on." (`autostart::refusal`, unit-tested).

## Where data lives

Little Joan's own file is `settings.json`; cookies, localStorage, IndexedDB and the HTTP cache
belong to the web engine, in its own store (read from the vendored tauri 2.12.4 / wry 0.57
sources):

| | Windows (WebView2) | macOS (WKWebView) | Linux (WebKitGTK) |
|---|---|---|---|
| `settings.json` | `%APPDATA%\io.github.memoliertorerr.littlejoan\` | `~/Library/Application Support/io.github.memoliertorerr.littlejoan/` | `~/.config/io.github.memoliertorerr.littlejoan/` |
| Cookies and site data | `%LOCALAPPDATA%\io.github.memoliertorerr.littlejoan\` (the WebView2 user data folder, `EBWebView\` inside): Tauri forces `app_local_data_dir` when the app sets no `data_directory` | WebKit's `defaultDataStore`, keyed by the bundle id (under `~/Library/WebKit/` and `~/Library/HTTPStorages/`); `data_directory` does nothing there | `~/.local/share/io.github.memoliertorerr.littlejoan/` (Tauri forces `app_local_data_dir`); cookies are a text file, `cookies`, there |
| With `LITTLEJOAN_DATA_DIR` (dev builds, CI) | `<dir>\settings.json`, `<dir>\webview\` | `<dir>/settings.json`; site data stays in WebKit's default store | `<dir>/settings.json`, `<dir>/webview/` |

Switching between a dev build and the installed app therefore looks like a logout on Windows and
Linux. Both webviews of the window share one data folder (WebView2 requires it). Closing the
window discards the page, not the store.

**Logins across close and quit** (proved per OS by the CI smoke test, LJ20):

| | Close → summon | Quit → relaunch |
|---|---|---|
| Persistent cookies, localStorage | kept by the engine (all three) | on disk (all three; macOS closes the window before quitting, above) |
| Session cookies (no expiry) | macOS and Linux keep them while the app runs; WebView2 drops them with the window, so on Windows `joan::destroy` reads them (`Webview::cookies`) and keeps up to 500 in memory, and `joan::create` sets them again before the first page (`cookies.rs`) | gone, as when a browser quits — never written to disk by Little Joan |

## History

Little Joan keeps a local history for the command bar's suggestions (LJ23) and the history view
(LJ25). It never leaves the machine: no sync, no remote suggestions; favicons come straight
from the sites visited.

- **Where:** `history.json` next to `settings.json` (the table above; `LITTLEJOAN_DATA_DIR`
  in dev builds and CI). Loaded when the page window opens, kept in `AppState` while it exists,
  saved atomically (temp file + rename) when it closes, every 30 s while there are unsaved
  visits, and on Quit; dropped from memory on hibernate, so nothing extra is held at rest.
- **Format:** `{ "version": 1, "entries": [ { "url", "title", "host", "visitCount", "visits":
  [ { "at": <unix ms>, "kind": "typed" | "link" | "reload" } ] } ] }` — one record per URL (the
  fragment dropped, the query kept), `host` lower case without `www.` (with a non-default
  port), the all-time visit count and the last 20 visits. Titles are cut at 300 characters;
  URLs over 4,096 bytes aren't recorded. A bad or missing file is empty history; a bad record
  or visit is skipped alone.
- **Cap:** 5,000 URLs; past that the least frecent go (on a tie, the least recently visited).
- **Recording** is native, from the page webview's events (`bar.rs`), so pages need no IPC: a
  finished page load records a visit, a title change updates that URL's record. The visit's
  kind comes from the app: `typed` after `open_input` (a picked suggestion goes through it too),
  `reload` after the Reload page action, `link` otherwise (WebView2's own F5 / Ctrl+R reload
  counts as `link`).
- **Never recorded:** anything but `http`/`https` (`about:`, `ljext:`, `data:`, `file:` …), the
  app's own pages (`keys::is_app_url`), Tauri's custom-scheme hosts on Windows
  (`tauri.localhost`, `ipc.localhost`, `ljext.localhost`), and nothing at all while the setting
  `rememberHistory` is off — existing history then stays until it is cleared (LJ25).
- **Frecency** (per host for top sites, per URL for matching): Σ w · 2^(−age / 14 days) over the
  last 20 visits within 90 days; w = 2 typed, 1 link, 0 reload.
- **Top sites** (`top_sites(n, exclude)`): hosts by frecency (sites without a recent visit left
  out), ties to the latest visit; each opens the host's most-visited URL with at most one path
  segment and no query, else its origin.
- **Matching** (`search(query, limit)`): every word must match; per word the best of host
  prefix 10, host word boundary 8, title word prefix 6, path word boundary 4, substring 1;
  score = points × the URL's frecency, ties to the latest visit (old matches still come, last).

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

## The bar: toolbar and command bar

The page window is a plain `tauri::Window` with two child webviews (Tauri 2 multi-webview,
which needs Tauri's `unstable` feature in 2.12): **`joan-bar`**, the trusted local page
(`ui/bar.html`), and **`joan-page`**, where web pages load. The bar is never injected into a
page; it is a separate webview with its own capability, and the page webview has none.

- **Modes** (`layout::Mode`, `bar::apply_layout`): **Toolbar** — the bar is 40 logical px tall at
  the top and the page fills the rest; **Overlay** — the command bar: the bar covers the whole
  window and the page is hidden; **Fullscreen** — element fullscreen: the bar is hidden and the
  page fills the window. Fullscreen always wins: starting it closes the overlay, and the overlay
  doesn't open meanwhile.
- **When the overlay opens:** every summon (a new window is built with it, before it is shown),
  Ctrl/Cmd+L (page keys and the macOS menu: the address is selected), and a click on the
  toolbar's address (`set_overlay(true)`). **When it closes:** whenever the app shows a page
  (`bar::navigate`: Enter in the command bar, `--show <url>`, Translate), Esc with a page behind
  it (`set_overlay(false)`, which hands the keyboard to the page), and element fullscreen. Rust
  tells the bar with `window.littleJoan.setOverlay(<bool>)`, and again when the bar finishes
  loading, so it never misses one; the smoke log gets `overlay true|false`.
- **Placing the webviews:** Windows and macOS place child webviews by bounds (re-applied on
  `Resized` / `ScaleFactorChanged`) and hide / show them. Linux packs child webviews into the
  window's vertical GtkBox, where bounds don't apply: in Toolbar mode the bar is packed with
  `expand = false` at a 40 px height request and the page takes the rest; in Overlay mode
  (`platform/linux.rs`) the bar is packed with `expand = fill = true` (the 40 px request stays a
  minimum, so the window can still shrink) and the page widget is hidden — with `no_show_all`
  set, because tao shows a window with `show_all()`, which would bring it back; hiding the bar
  (fullscreen) lets the page fill the window.
- **Address:** the bar always shows the page's full address (empty, with the placeholder, on the
  start page). Rust pushes changes with `bar.eval("window.littleJoan.setUrl(<json>)")` on
  every page load (started and finished, so redirects land), every title change, after
  navigations the app starts, and from a once-a-second check while the window exists (for
  `history.pushState` / `#anchor` changes, which raise no event). The bar asks once on load
  (`page_url`) so it can't miss the first address. Changes are de-duplicated.
- **Editing:** click the toolbar's address or press Ctrl/Cmd+L — the command bar opens with the
  text selected. Enter opens it in the page (address or search, as before) and moves the
  keyboard to the page. Esc reverts an edit; with nothing to revert it closes the command bar and
  returns to the page, or with no page closes the window — also after a click elsewhere in the
  command bar (the welcome's checkbox), since the keys stay in the bar webview. Leaving the field
  without Enter drops the edit. An address update never overwrites an edit in progress.
  (`ui/command/view-state.ts` holds these rules.)
- **The command bar** (`ui/command/`): a rounded panel with a search icon, the field
  ("Search or enter URL…") and a × button (back to the page, or with no page, close the window);
  under it the welcome on the first run, then the rows — a site icon, then "Title — domain" with
  the domain muted; the selected row is filled with the accent color; **"Settings & extensions"**
  (a gear) is always the last row. Rows (`rows.ts`): nothing typed → the top sites (or the
  starters); typing → a quick answer (LJ31's hook, empty now), the best history match, "Search
  <engine> for …", more matches, extension commands (LJ30's hook, empty now) — at most 8, then
  Settings. **Autofill:** when the best match's site starts with what was typed (a host prefix)
  and its score is at least the matches' average, the field completes it with that address
  (scheme and `www.` left out), the completion selected and its row selected; only right after
  typing at the end of the field, never after a deletion. **Keys:** ↑/↓ move without wrapping (↑
  from the first row goes back to the field), Enter opens the selected row — or with none,
  `open_input` on the field's text, as before —, Tab accepts the autofill, Shift+Delete removes a
  history row; the mouse selects on hover and opens on click. Picking a row goes through
  `open_input`, so the visit counts as typed. Typing asks `suggest` after 60 ms; only the latest
  answer is drawn. While the field shows the open page's address untouched (Ctrl/Cmd+L), the
  command bar shows the top sites.
- **Text only:** rows are drawn with `textContent` (`view.ts`), never `innerHTML`: titles come
  from web pages (and, later, extensions). **Site icons** are
  `<img src="https://<site>/favicon.ico" referrerpolicy="no-referrer">` straight from the site,
  shown once loaded over the site's first letter in a tinted circle (a stable hue per site),
  which stays when the icon fails. That is why the CSP's `img-src` allows `https:` (and only
  `img-src`); history itself is never sent anywhere.
- **Element fullscreen:** Windows hides the bar in the `ContainsFullScreenElementChanged`
  handler; Linux on WebKitGTK's `enter-fullscreen` / `leave-fullscreen` signals; macOS needs
  nothing (WebKit shows the element in its own fullscreen window).

## Page size and phone mode

- **The page always fills the window minus the bar, in logical pixels.** `bar::apply_layout`
  turns the window's physical inner size and scale factor into logical bounds
  (`layout::for_physical_size`); Tauri and wry convert logical bounds with the display's own
  scale (WKWebView `backingScaleFactor`, WebView2 per-window DPI), so a page's `innerWidth` is
  the same on a Retina screen as at scale 1. Linux sizes the views in a GtkBox, which is
  logical by nature. Unit tests cover scale 1, 1.5 and 2; the CI smoke test resizes the window
  with `--size` and checks the page's `innerWidth × innerHeight` (also at GDK scale 2 on Linux).
- **Phone mode** (`phone.rs`, `phone_mode.rs`, setting `mobileSites`, on by default): many sites
  (Google, YouTube) choose their mobile version by **user agent**, not by viewport width, so a
  narrow window alone still gets a cropped desktop page. Below **600 logical px** of page width
  the page webview gets a phone user agent: **iPhone Safari** on macOS and Linux (WebKit
  engines) and **Chrome on Android** on Windows (Chromium; the Chrome version is copied from
  WebView2's own agent), so the mobile pages that come back are built for the same engine.
  - Set per OS at runtime: WKWebView `customUserAgent` (`platform/mac.rs`), WebView2
    `ICoreWebView2Settings2::SetUserAgent` (`platform/win.rs`, the engine's own agent is read
    once to restore it), WebKitGTK's `user-agent` setting (`platform/linux.rs`).
  - A new window gets the right agent before any web page loads. A resize that crosses the
    threshold switches the agent once the resize has been still for 400 ms (`Debounce`), then
    reloads the current web page (its URL is kept; the bar keeps working). Nothing switches
    during fullscreen — a fullscreen video makes the window wide, and a reload would end it.
  - Changing the setting applies at once.

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
`window-created`, `suggestions <rows> first=<site> typed=<characters>` (each `suggest` call:
counts and the first row's site, never the typed text), `overlay <true|false>` (the command bar opened or closed; once right after
`window-created` for the state a new window starts in), `title <page title>`, `page-loaded <url>`, `fullscreen <true|false>` (any
engine: the window's fullscreen state changed), `element-fullscreen <true|false>` (Windows: the
app changed it for a fullscreen element), `bar-loaded` (the URL bar's page finished loading),
`bar-url <address | start>` (what the bar was told to show), `hibernated`,
`start-at-login <true|false>` (the login entry read back after a change),
`start-at-login-failed <reason>`, `first-run` (no settings.json at launch), `welcome` (the command
bar was handed the first run), `quit` (Quit or `--quit` is about to exit),
`session-cookies-kept <n>` / `session-cookies-restored <n>` (Windows: session cookies kept on
close, set again in the new window; counts only), `history-visit <kind> <url>` (a visit was
recorded), `history-saved <records>` (history.json written). Without the variable nothing is
logged.

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
- **settings** — `{version, shortcut, searchEngine, translateTo, windowWidth, windowHeight,
  mobileSites, rememberHistory}` (camelCase JSON). Each field falls back to its default on its own; sizes are clamped to
  320–4096 × 300–4096; saves go through a temp file + rename.
- **toggle** — what a shortcut press or a command-line request does:

  | State             | Toggle  | Show(None) | Show(url)          | Hide    |
  |-------------------|---------|------------|--------------------|---------|
  | Hibernating       | Create  | Create     | Create(url)        | Nothing |
  | Open, focused     | Destroy | Focus      | FocusAndNavigate   | Destroy |
  | Open, not focused | Focus   | Focus      | FocusAndNavigate   | Destroy |

- **cli** — `--toggle`, `--show [http(s) url]`, `--hide`, `--size WxH` (1–10000 each; resizes
  the open window); the last flag wins; unknown arguments are ignored; `--show` accepts only
  absolute web URLs. `--start-at-login on|off` (any case; other values are ignored) is a
  separate result next to the window command, so it never maps to one; a second launch whose
  only flag is `--start-at-login` leaves the window alone. `--quit` is a third result; it
  replaces the window command (a quitting launch never opens the window).

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

The release profile optimizes for size (`opt-level = "s"`, LTO, one codegen unit, stripped)
but keeps `panic = "unwind"`: on macOS, wry catches the Objective-C exception WebKit raises when
Tauri answers a `tauri://` request (from a tokio thread) whose navigation was cancelled
meanwhile, and that catch cannot work under `panic = "abort"` — CI saw the app abort. Builds are not notarized or Authenticode-signed (see the plan's Later
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
  installed binary: launch with `LITTLEJOAN_SMOKE_LOG`, `--show` (the command bar covers the
  window), `--show http://127.0.0.1:<port>/smoke.html` (it closes: the toolbar above the page),
  click the screen centre (the page's fullscreen button),
  load the page again (leaving fullscreen), `--size 390x600` then `--size 720x600`, `--hide`,
  summon and hide again, `--start-at-login on` then `off` (reading the OS entry each time: the
  Run key's values, the LaunchAgent plists' `ProgramArguments`, the autostart files' `Exec=` —
  it must point at the installed binary, then be gone, and no window may open), then check the
  log order, single-instance forwarding, window
  fullscreen (Windows, Linux), the bar (`bar-loaded`, and `bar-url` with the smoke page's
  address), the command bar (`overlay true` first after every `window-created`, and `overlay
  false` before the smoke page reports ready; the starters first with no history; after the
  smoke page, a new window's empty command bar offers its site first, and typing `127` — xdotool
  on Linux, SendKeys on Windows, key events on macOS, best effort — finds it first;
  `smoke-command.mts`, screenshots `command-bar.png`, `command-typing.png`), that the page saw a **390×560 viewport with a phone user agent and a 720×560 one
  with the desktop agent** (it reports `innerWidth`, `innerHeight`, the agent kind and
  `devicePixelRatio` in its title), and that none of the webview helper processes that served the page outlive the
  window (followed by process id, so other apps' WebKit helpers on the runner don't count).
  Screenshots (`hibernating.png`, `start.png`, `page.png`, `fullscreen.png`, `narrow.png`,
  `wide.png`, `welcome.png`), the logs, and
  The app runs on a fresh `LITTLEJOAN_DATA_DIR` that already holds the default `settings.json`,
  so it starts hibernating (memory at rest is measured before any window opens). Then the
  **login-persistence check**: the smoke page's `?persist=set&t=<token>` sets a persistent
  cookie (`Max-Age` a year), a session cookie and a localStorage value; `--hide` and `--show
  ?persist=check` must report `LJ-PERSIST p=1 s=1 l=1`; new values, `--quit` within 1 s of
  them (from the log's clock), and a relaunch on the same data folder (log
  `smoke-after-quit.log`) must report `p=1 s=0 l=1` — session cookies end with the app run. The
  smoke server keeps its port across the relaunch, because localStorage is per origin. Then the
  **history check**: the main run's `history.json` must hold the smoke page's address with its
  `LJ-SMOKE …` title, its host and a visit, and none of the app's own pages; a run whose
  settings say `"rememberHistory": false` loads the smoke page, hibernates and quits, and must
  leave no records (log `smoke-history-off.log`). Last comes the **first-launch check**: quit, launch on a new empty data folder (it must log `first-run`,
  `window-created`, `bar-loaded`, `welcome`; screenshot `welcome.png`), `--hide`, quit, relaunch
  on the same folder: `settings.json` must exist and no window may open within 5 s (logs
  `smoke-first-run.log`, `smoke-relaunch.log`).
  `summary.json` (memory — resident and the app's own: private bytes / private dirty /
  physical footprint —, helper counts sampled for 60 s and after three summon cycles, codecs,
  the login entries found)
  are uploaded as `smoke-<os>`. Linux runs under `xvfb-run` + `dbus-run-session`
  (single-instance needs a session bus) with openbox, and under gdb so a crash leaves a
  backtrace; the AppImage gets a launch check. A second Linux job (`linux-scale-2`) runs the
  same test at **scale factor 2** (`GDK_SCALE=2` on a 2560×1600 Xvfb screen); the Windows and
  macOS runners can't change their display scale.
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
