# Little Joan — architecture

Little Joan is a Tauri 2 app: a Rust process that owns everything native (tray, global shortcut,
windows, the OS webview), plus a few tiny local pages written in TypeScript and bundled by Vite
(the bar, the settings window, and a plain first page for the page webview).
The plan and its decisions live in [BUILD_PLAN.md](BUILD_PLAN.md).

## Process model

- **At rest — the owner's "After closing" setting (LJ19.4; `settings.afterClose`):** one native
  process with a tray / menu-bar icon, and
  - **Stay ready** (`ready`, the default): the page window, hidden, with only its trusted bar
    webview (`joan-bar`) loaded — no page webview, so no web page's process. Built ~3 s after
    the launch (the prewarm) and kept after every close; its engine helpers (the bar's renderer,
    WebKit's Networking / GPU, WebView2's browser) stay with it.
  - **Stay ready for 15 minutes** (`ready15`): the same, destroyed 15 minutes after the close (or
    the launch's prewarm) unless a summon came: then as Hibernate until the next close.
  - **Hibernate** (`hibernate`): no window and no webview, so no webview helper processes exist
    (WebView2's `msedgewebview2.exe`, WebKit's `WebContent`/`WebKitWebProcess`), as before LJ19.4.

  In every mode **a close destroys the page webview**: no web page's scripts, media, network or
  process survive it (Ground Rule 9 as the owner rewrote it). `tauri.conf.json` declares
  `"windows": []`.
- **Closing the last window never quits.** `RunEvent::ExitRequested` without an exit code is
  prevented; only Quit in the tray menu, the command bar's **Quit Little Joan** and `--quit` (all
  `joan::quit`), and Cmd+Q on macOS (the app menu's standard Quit item), exit.
- **The icon can be hidden** (settings' `showTrayIcon`, Settings → General; LJ41), never so that
  Little Joan is out of reach: `tray::icon_visible` shows it unless the setting is off **and** the
  global shortcut works (`tray::shortcut_works`, from `shortcut/backend.rs`: global-hotkey has one
  registered — the saved one or the default — and this isn't a Linux Wayland session, where the X11
  grab never fires; or the XDG GlobalShortcuts portal holds it and the desktop gave it a key, LJ17;
  a shortcut paused while Settings records one still counts, so the icon doesn't blink).
  `tray::apply_visibility` runs after the shortcut is registered at startup, when the portal
  answers or its trigger changes, and after every settings save (a swap that left no shortcut
  working brings the icon back) and logs `tray-visible <bool>` the first time and on each
  change. It uses the tray's own `set_visible` — macOS removes the status item and builds it again,
  Windows hides the notification icon, Linux sets the AppIndicator's status to Passive /
  Active (tray-icon 0.25.1). Hiding the icon creates nothing; what rests is the same as with it.
  Without the icon: the shortcut, a second launch (no flags = `--show`), and the command bar's
  **Settings** and **Quit Little Joan** rows.
- **macOS:** `ActivationPolicy::Accessory` — a menu-bar icon and no Dock icon.

## Files

```
README.md, LICENSE                 what it is, install, use; MIT
CHANGELOG.md                       what each version contains
CONTRIBUTING.md, SECURITY.md       how to build and check changes; how to report security problems
CODE_OF_CONDUCT.md                 the Contributor Covenant 2.1; reports go privately to the maintainer
package.json, package-lock.json   npm scripts (verify, icons, dev); all JS tooling is local
tsconfig.json                      strict TS for ui/, scripts/, tests/, config files
vite.config.mts                    UI build: root ui/, output dist/ (embedded into the binary)
vitest.config.mts                  TS unit tests: ui/**, scripts/**, tests/**, extensions/** *.test.ts — each
                                   *.test.ts sits next to the module it tests (Rust tests are
                                   #[cfg(test)] modules inside each .rs file)
eslint.config.mjs                  typescript-eslint recommended (type-checked)
rust-toolchain.toml                pinned Rust (minimal profile + rustfmt + clippy)
scripts/env.sh                     source first: keeps caches/toolchains/temp/dev data in .cache/
scripts/setup-rust.sh              installs rustup + the pinned toolchain into .cache/
scripts/png.mts                    dependency-free PNG encoder
scripts/icon-geometry.mts          the original icon artwork as pure shape functions (+ test)
scripts/make-icons.mts             renders the app icon and the macOS tray template
scripts/theme-css.mts              writes ui/lib/theme-defaults.css (`npm run theme:css`)
scripts/copy-icons.mts             copies the desktop icon set from `tauri icon` output
extensions/<id>/                   bundled extensions (embedded at build time by build.rs): an
                                   extension.json and plain web files; hello/ is the example and
                                   the smoke test's target. The others share one layout:
                                   index.html (the SDK, then `<script type="module" src="ui.js">`),
                                   ui.js (DOM glue), logic.js (pure, an ES module the tests import),
                                   logic.test.ts, style.css (the theme's --lj-* variables), icon.svg
extensions/snake/, extensions/2048/  the bundled games: Snake (a canvas; grid, turns without 180°
                                   reversal, growth, walls and self, speed-up, Space pauses) and 2048
                                   (DOM tiles keyed by id so they slide; one merge per tile per move;
                                   the game in progress is saved); best scores in littleJoan.storage
extensions/notes/, extensions/focus/, extensions/clock/  the bundled productivity extensions: Notes
                                   (list + editor, search, autosave with a debounce, copy with the
                                   clipboard permission, delete with Undo), Focus timer (25 / 5 / 15
                                   cycles from a stored end time; a Web Audio chime while open), World
                                   clock (cities.js: ~180 cities → IANA zones; Intl does daylight
                                   saving; add, remove, reorder)
extensions/qr/, clean-link/, wayback/, markdown-link/  the bundled page actions (permission `pageInfo`;
                                   Clean link and Markdown link also `clipboard`): a QR encoder (byte
                                   mode, versions 1–10, level M; reference matrices in
                                   reference.test-data.json), tracking-parameter rules and redirect
                                   unwrapping written for Little Joan, the Wayback Machine's addresses,
                                   Markdown / HTML / Org link escaping
extensions/json/, text-tools/, color/  the bundled tools: a JSON parser with line / column errors and
                                   printers that keep numbers and strings as written; counts, case,
                                   Base64 / URL encoding, line operations; color parsing (hex, rgb(),
                                   hsl(), oklch(), names), conversions and WCAG contrast
extensions/bundled.test.ts         every bundled extension: folder = id, the SDK first and no inline
                                   script, no network API and no `network:` host, text drawn as text,
                                   a pure logic.js with tests
extensions/sdk/v1.js, v1.d.ts      the extension SDK (served to panels at /__lj/sdk/v1.js) and its
                                   types
ui/bar.html, ui/bar.ts             the bar (its own webview): the toolbar above the page, or the
                                   command bar over the whole window; DOM glue only
ui/bar-boot.ts                     the bar's first paint: the theme of its initialization script,
                                   applied before anything awaits; start_info's only if it differs
                                   (pure part unit-tested)
ui/bar-timing.ts                   smoke tests only (start_info's `timing`): how the bar's start
                                   went — its navigation, HTML, script, first answers, first frame,
                                   the command bar's first rows —, reported once, at the end, as
                                   "this long ago" (report_timing; unit-tested, LJ19.1)
ui/bar-logic.ts                    the address field: never clobber an edit (unit-tested)
ui/command/view-state.ts           the bar's two views (toolbar, overlay) and what Enter / Esc do
                                   in each (unit-tested)
ui/command/welcome.ts              StartInfo and the first launch's welcome (unit-tested)
ui/command/panel.ts                the welcome in the command bar, with its start-at-login
                                   checkbox; DOM glue only
ui/command/rows.ts                 the command bar's rows: order, autofill, keys, site icons
                                   (pure, unit-tested)
ui/command/view.ts                 draws the rows, as text only (tested with a stand-in document)
ui/answers/                        quick answers, our own code in the bar: calc.ts (a tokenizer
                                   and precedence-climbing parser, never eval), units.ts, time.ts
                                   (+ cities.ts, ~200 names → IANA zones; Intl with a passed-in
                                   clock), random.ts (web crypto, no modulo bias), timestamp.ts,
                                   index.ts (the built-ins, their order, the answer row) — each
                                   pure and unit-tested
ui/command/quick.ts                the quick commands (LJ42): the fixed list, matching, each row's
                                   state as text, Clear history's second Enter, Rust's
                                   quick_state checked (pure, unit-tested); Copy as Markdown link
                                   uses extensions/markdown-link/logic.js's `link`
ui/command/copy.ts                 copies a quick answer (and the page's link): the async clipboard API on the keypress,
                                   a hidden-textarea fallback
ui/command/extensions.ts           extension commands and page actions as command-bar rows:
                                   matching, "for this page", switched-off ones hidden (pure,
                                   unit-tested)
ui/command/command.ts              the command bar's list: suggest() as you type, autofill, keys,
                                   mouse; DOM glue only
ui/history/logic.ts                the history view: search, grouping by local day (Today,
                                   Yesterday, dates), times, the "Clear history…" choices, keys
                                   (pure, unit-tested)
ui/history/view.ts                 draws the history list by day, as text only (tested with a
                                   stand-in document)
ui/history/history.ts              the history view in the overlay: history_list, search, keys,
                                   mouse, remove, clear; DOM glue only
ui/history/history.css             the history view's styles (linked by bar.html)
ui/toolbar/logic.ts                the toolbar: the domain shown (or an extension's name and
                                   icon), back / open-in-browser enabled, the Phone / Desktop pill
                                   and Rust's site info, the status area (downloads: its text, its
                                   button, how long it stays), macOS from the user agent (pure,
                                   unit-tested)
ui/toolbar/toolbar.ts              the toolbar's buttons and site; DOM glue only
ui/index.html                      the page webview's first page: a plain background, no script
ui/lib/shortcut.ts                 accelerator ↔ key press (recorder) and display text ("⌘⇧Space")
ui/lib/color.ts                    color math: hex ↔ sRGB ↔ OKLab ↔ OKLCH, fitting into sRGB,
                                   mixing in OKLab, WCAG contrast (pure, unit-tested)
ui/lib/theme.ts                    themes: picks → muted backgrounds, text colors, the gradient,
                                   the grain, the --lj-* CSS variables (pure, unit-tested)
ui/lib/presets.ts                  the theme presets; the first is the default theme
ui/lib/theme-css.ts                the default theme as a stylesheet (for theme-defaults.css)
ui/lib/theme-defaults.css          generated by `npm run theme:css`: the default theme's variables,
                                   light and dark (style.css imports it; a test checks it)
ui/lib/apply-theme.ts              puts a theme on a page (custom properties on <html>) and follows
                                   the system's light / dark; DOM glue only
ui/settings.html, ui/settings.ts   the settings window: tabs (General, Keyboard, Appearance,
                                   Extensions, Privacy, About), lists, switches, clear history;
                                   DOM glue only
ui/settings-logic.ts               settings types, tabs, change detection across tabs (the theme
                                   too), the Privacy tab's clear action (unit-tested)
ui/settings/appearance-logic.ts    the Appearance tab: color pads (hue × chroma ↔ color), arrow
                                   keys, adding / removing colors, presets, sliders, the draft
                                   against the saved theme (pure, unit-tested)
ui/settings/appearance.ts          the Appearance tab's controls and live preview; every edit
                                   re-themes the window and goes to preview_theme; DOM glue only
ui/settings/appearance.css         the Appearance tab's styles (linked by settings.html)
ui/settings/extensions-logic.ts    the Extensions tab: groups (Built-in, Bundled, Developer),
                                   permissions in plain words, the switches' draft (pure,
                                   unit-tested)
ui/settings/extensions.ts          the Extensions tab's list, switches, Developer mode, folder
                                   and reload buttons; DOM glue only
ui/settings/extensions.css         the Extensions tab's styles
ui/settings/keyboard-logic.ts      the Keyboard tab: recording a key, the keymap's rules (as Rust's
                                   keymap::validate), reserved-shortcut warnings, the draft, Reset /
                                   Reset all, the launch's shortcut problem (pure, unit-tested)
ui/settings/keyboard.ts            the Keyboard tab: the summon recorder (moved from General) and
                                   every Little Joan key's recorder; DOM glue only
ui/settings/keyboard.css           the Keyboard tab's styles
ui/settings/reserved-shortcuts.ts  the OSes' own shortcuts and the editing keys (curated data,
                                   tested)
ui/settings/sites-logic.ts         General's "Phone or desktop per site": the user's entries and
                                   the built-in ones, Remove / Reset, the draft (pure, unit-tested)
ui/settings/sites.ts               that list in the General tab; DOM glue only
ui/settings/quit.ts                General's "Quit Little Joan" (LJ19.7): the button calls
                                   quit_from_settings once (unit-tested with a fake button)
docs/extensions.md                 the extension authors' guide (experimental)
docs/trademark-check.md            where to check that the name is free before a wider launch or a
                                   store listing (a note for the owner, not legal advice)
docs/upstream/                     reports drafted for other projects, for the owner to file:
                                   webkitgtk-2.50-fullscreen-abort.md (Ubuntu 22.04's WebKitGTK)
docs/images/                       the README's screenshots, cropped from CI's macOS smoke run (its
                                   test page and sample data only — nothing personal)
ui/settings.css                    the settings window's styles (style.css has the shared tokens)
ui/style.css                       shared styles, all colors from the theme's --lj-* variables;
                                   `.themed` = the gradient with the grain on top
src-tauri/Cargo.toml               crate little-joan (lib little_joan_lib + bin)
src-tauri/build.rs                 tauri-build + the app manifest (which commands exist); embeds
                                   extensions/<id>/ ($OUT_DIR/bundled_extensions.rs)
src-tauri/capabilities/bar.json    the bar may call open_input, page_url, focus_page, set_overlay,
                                   go_back, open_in_browser, hibernate, history_list, suggest,
                                   history_remove, history_clear, open_settings, start_info,
                                   set_start_at_login, set_theme_tokens, extensions_list,
                                   open_extension, site_mode_info, set_site_mode, quit_app,
                                   quick_state, quick_command, keymap_info, reveal_download,
                                   report_timing; drag the window
src-tauri/capabilities/settings.json  settings window: get/save settings, suspend shortcut, set
                                   start at login, clear history, preview a theme, list / reload
                                   extensions, show their folder, read the keys, quit
                                   (quit_from_settings, LJ19.7), close
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
src-tauri/src/history/manage.rs    the history view's list (latest visit first) and "Clear
                                   history" by range: hour, day, week, all (pure, tested)
src-tauri/src/history_store.rs     history.json on disk and in AppState while the window exists:
                                   load (on its own thread, waited for by every use), record (page
                                   load / title events), autosave, atomic save,
                                   remove one and clear a range (both rewrite the file at once,
                                   with no window straight on disk), the list, the suggestions
src-tauri/src/suggest.rs           what the command bar suggests: top sites or the starters,
                                   history matches and a search row (pure, tested)
src-tauri/src/welcome.rs           the welcome's per-OS text: where the tray icon is; a Linux Wayland
                                   session (pure)
src-tauri/src/joan.rs              the page window's lifecycle: requests through `toggle`, focus,
                                   resize, quit, window events
src-tauri/src/joan/fullscreen.rs   a summon while the page is in element fullscreen (LJ19.5): the
                                   page asked to leave (its own exit; macOS: WebKit closes it if it
                                   won't), the wait until the page is back in the window
src-tauri/src/joan/rest.rs         what a summon builds and a close leaves (LJ19.4): cold create,
                                   summon from ready, close (rest or destroy), prewarm, release,
                                   Stay ready for 15 minutes' timer, the bar's process ending
src-tauri/src/joan/window.rs       the window itself: macOS's traffic lights, the remembered size
                                   (saved on close, restored when it rests)
src-tauri/src/bar.rs               its two webviews: the bar (built with the window), pushing the
                                   page's address to the bar, focus, navigation; the bar's rest /
                                   summon state (LJ19.4)
src-tauri/src/bar/page.rs          the page webview (LJ19.4): built after the bar (or at once for a
                                   URL; `ensure_page` for whatever needs it), added hidden so it
                                   never flashes, closed with every close (session cookies kept,
                                   downloads ended, macOS unloads it first)
src-tauri/src/bar/view.rs          the view state: toolbar / overlay / element fullscreen,
                                   applied to the webviews — only what changed since the last
                                   layout; macOS autoresizing masks per mode
src-tauri/src/bar/watch.rs         the once-a-second check of the page's address
src-tauri/src/bar/boot.rs          the bar's first paint: its theme script (checked, tested), the
                                   theme pushed to it, whether its script runs yet (`Ready`)
src-tauri/src/bar/background.rs    the bar webview's and the window's own background: the theme's
                                   `--lj-bg-0` it painted last, else the theme's own (theme_color.rs;
                                   pure part tested)
src-tauri/src/theme_color.rs       a theme's `--lj-bg-0` computed as ui/lib/theme.ts does (OKLCH,
                                   pure, tested against the TS values; LJ19.4's theme flash)
src-tauri/src/bar/download.rs      downloads at run time: on_download (WebView2, WKWebView), the
                                   Downloads folder, the toolbar's status, the window closing ends
                                   them (LJ15)
src-tauri/src/bar/status.rs        the toolbar's status area from Rust: a download's state, the zoom
                                   level (`setStatus`), told again when the bar's script runs
src-tauri/src/bar/zoom.rs          page zoom at run time: the window's level, `set_zoom` on the page
src-tauri/src/zoom.rs              page zoom's levels and steps (pure, tested)
src-tauri/src/downloads.rs         downloads: which are saved, the file's name (cleaned, capped,
                                   unique), the window's downloads under way (pure; tests in
                                   downloads/tests.rs)
src-tauri/src/extensions/          extensions (experimental; below): mod.rs (the model),
                                   manifest.rs (extension.json, strict), paths.rs (plain paths,
                                   no traversal, no symlink out), urls.rs (one origin per
                                   extension, per OS), csp.rs (the policy and headers),
                                   registry.rs (bundled + developer folders), api.rs (every
                                   request check; tests in api/tests.rs), web_address.rs (what
                                   `openUrl` may open), protocol.rs (the `ljext` handler),
                                   limits.rs (rate limits, calls in flight), nonce.rs (OS
                                   randomness: nonces, tokens), storage.rs, theme.rs,
                                   clipboard.rs, gate.rs (the navigation gate, the commit rule,
                                   the wait to leave a web page; pure), session.rs (the open
                                   panel, opening and closing, the toolbar label; tests in
                                   session/tests.rs) — pure parts unit-tested
src-tauri/src/page_address.rs      the page's address from its load events; when the engine may
                                   be asked for it (pure, tested)
src-tauri/src/back_stack.rs        whether the toolbar's back button has somewhere to go: this
                                   window's trail of loaded web pages (pure, tested)
src-tauri/src/layout.rs            where the bar and the page go per mode (Toolbar, Overlay,
                                   Fullscreen), from physical size + scale (pure)
src-tauri/src/layout/diff.rs       what a new layout changes (bounds, visibility) and macOS's
                                   autoresizing masks per mode (pure, tested)
src-tauri/src/capability_tests.rs  tests: no capability reaches the page webview, its window, or
                                   remote URLs (Ground Rule 18)
src-tauri/src/phone.rs             phone mode decisions: below 600 px / from 640 px, user agents,
                                   debounce (pure, tested)
src-tauri/src/phone/sites.rs       per site: a site's mode (siteModes + the WebKit built-in list),
                                   the key a site is kept under, changing a mode, the info for the
                                   bar, what the page does after a switch (pure, tested)
src-tauri/src/phone/agent.rs       Windows' phone agent with client hints: CDP
                                   `Emulation.setUserAgentOverride` parameters, the layout-metrics
                                   check (pure, tested; built on Windows and in tests)
src-tauri/src/commands/site.rs     the bar: site_mode_info, set_site_mode; the built-in sites for
                                   Settings
src-tauri/src/commands/app.rs      the bar: quit_app (the command bar's "Quit Little Joan");
                                   Settings: quit_from_settings (General's, LJ19.7)
src-tauri/src/commands/timing.rs   the bar: report_timing (its start's marks; smoke tests only,
                                   LJ19.1)
src-tauri/src/commands/downloads.rs  the bar: reveal_download (the toolbar's "Show in folder")
src-tauri/src/commands/keys.rs     the bar and Settings: keymap_info; the portal's shortcut as they
                                   show it (PortalShortcut)
src-tauri/src/commands/quick.rs    the bar: quick_state, quick_command (a serde enum of fixed
                                   kinds, deny_unknown_fields; tested)
src-tauri/src/settings_live.rs     a settings change at runtime: AppState, the bar's theme, the tray
                                   rule, phone mode, the registry, the atomic save (Save and the
                                   quick commands share it)
src-tauri/src/phone_mode.rs        phone mode at runtime: set the page's user agent for its width
                                   and site, before the app's navigations and on the page's own,
                                   reload (or load the desktop address); a site's mode from the bar,
                                   the info pushed to it
src-tauri/src/placement.rs         where a new window opens (monitor under the cursor)
src-tauri/src/system_shortcuts.rs  macOS's own shortcuts from its com.apple.symbolichotkeys
                                   preferences (as JSON): id, on or off, the key when the entry
                                   gives one (pure; tested on a CI runner's export,
                                   system_shortcuts/macos-runner.json)
src-tauri/src/shortcut.rs          global-shortcut plugin, registration with fallback, live swap
                                   with rollback (Registrar trait), suspend while recording; at
                                   launch the portal first in a Wayland session (LJ17), its answer
                                   and trigger changes (tests in shortcut/tests.rs)
src-tauri/src/shortcut/backend.rs  who holds the summon shortcut (global-hotkey, the portal, or
                                   global-hotkey on XWayland with the --toggle hint), the XDG
                                   shortcuts spec's trigger for an accelerator, the portal's smoke-
                                   log lines (pure, tested)
src-tauri/src/commands.rs          bar: open_input, page_url, focus_page, set_overlay, go_back,
                                   open_in_browser, hibernate, start_info, history_list, suggest,
                                   history_remove, open_settings, set_theme_tokens; settings:
                                   get_settings,
                                   save_settings, suspend_shortcut; set_start_at_login and
                                   history_clear (both)
src-tauri/src/autostart.rs         start at login: tauri-plugin-autostart (Rust API only), the
                                   macOS translocation guard (pure, tested)
src-tauri/src/settings_window.rs   the settings window (on demand, destroyed on close): its tabs
                                   (`SettingsTab`, tested), open on a tab or switch an open one,
                                   links to the default browser
src-tauri/src/smoke.rs             LITTLEJOAN_SMOKE_LOG event log for CI smoke tests
src-tauri/src/summon_timing.rs     smoke mode only: a summon's marks from the request to the command
                                   bar on screen and its field taking keys, one `summon-timing` line
                                   per summon, cold or from ready (pure part tested, LJ19.1, LJ19.4)
src-tauri/src/spaces.rs            where a summoned window shows up (LJ19.2): macOS's collection
                                   behaviour (`summoned`: moveToActiveSpace + fullScreenAuxiliary,
                                   AppKit's exclusive groups kept to one member), the bits' names,
                                   the smoke log's `window-spaces` / `window-front` lines (pure,
                                   tested)
src-tauri/src/fullscreen.rs        FullscreenTracker: element fullscreen ↔ window fullscreen;
                                   WebKitFullscreen (WKWebView's fullscreenState);
                                   needs_fullscreen_off (WebKitGTK older than 2.52); when a summon's
                                   wait for WebKit's fullscreen to end is over (LJ19.5; tested)
src-tauri/src/keys.rs              keys and presses: `Key` (what the recorder records) from
                                   KeyboardEvent.code names, Windows virtual keys and GDK key
                                   names; `Mods`; the app origin (pure, tested)
src-tauri/src/keymap.rs            the keymap: every Little Joan key, the defaults per OS, the
                                   user's overrides, matching, validation, keymap_info's list
                                   (pure; tests in keymap/tests.rs)
src-tauri/src/page_actions.rs      close / open location / reload / back / forward / translate /
                                   open in browser / history / zoom in, out, reset; translate_target
                                   (skips the start page)
src-tauri/src/menu.rs              macOS app menu: Edit (copy/paste in WKWebView) + the page items,
                                   their keys from the keymap (set again when the keys change);
                                   Cmd+Q goes through joan::quit
src-tauri/src/platform/mod.rs      installs the per-OS hooks on the bar and the page webviews
src-tauri/src/platform/win.rs      Windows: WebView2 AcceleratorKeyPressed → keys.rs;
                                   ContainsFullScreenElementChanged → window fullscreen; phone
                                   mode's agent through the DevTools protocol (SetUserAgent as the
                                   fallback), Page.getLayoutMetrics
src-tauri/src/platform/win/rest.rs  Windows (LJ19.4): the bar's ProcessFailed, its memory target
                                   and visibility at rest, the page's downloads noted and cancelled
src-tauri/src/platform/win/front.rs  Windows: the smoke log's check that a summoned window is the
                                   foreground window and on top at its centre (LJ19.2)
src-tauri/src/platform/mac.rs      macOS: WKWebView customUserAgent (phone mode), autoresizing
                                   masks; WebKit's fullscreenState (CI's `wk-fullscreen` log); the
                                   symbolic-hotkeys domain read through NSUserDefaults (read-only)
src-tauri/src/platform/linux.rs    Linux: GTK window key-press-event → keys.rs; the bar's packing
                                   in the window's GtkBox (toolbar / overlay); enter/leave-
                                   fullscreen → hide the bar; element fullscreen off on WebKitGTK
                                   older than 2.52 (the runtime version)
src-tauri/src/platform/linux/downloads.rs  Linux: the page's downloads through WebKitGTK itself (an
                                   emission hook on decide-destination), cancelled on close
src-tauri/src/platform/linux/front.rs      Linux: a summon's raise-and-focus (GTK present) with the X
                                   server's time on X11, and the smoke log's active / top-of-stack
                                   check (LJ19.2)
src-tauri/src/platform/mac/activity.rs     macOS: a summon's NSProcessInfo activity (App Nap off
                                   until the window is up; never at rest) and, for the smoke log,
                                   App Nap's state (the task's suppression policy) (LJ19.1)
src-tauri/src/platform/mac/downloads.rs    macOS: the page's WKDownloads noted (a wrapped
                                   -[WKDownload setDelegate:]) and cancelled when the window closes
src-tauri/src/platform/mac/spaces.rs       macOS: a window's collection behaviour set before it is
                                   first shown (the active Space, over full-screen apps; LJ19.2),
                                   and the smoke log's key / on-top / on-the-active-Space check
src-tauri/src/platform/mac/notes.rs        macOS, smoke log only: AppKit's key-window, native
                                   full-screen and activation notifications with the stacks that
                                   posted them and each window's state, NSWorkspace's Space and
                                   frontmost-app changes (`wk-note`), a window's state at a moment
                                   (`wk-context`; LJ19.4-fix4)
src-tauri/src/platform/portal.rs   Linux: the summon shortcut through the XDG GlobalShortcuts portal
                                   (ashpd): the host registry, a session, BindShortcuts, Activated
                                   → toggle, ShortcutsChanged (LJ17)
src-tauri/linux/io.github.memoliertorerr.littlejoan.desktop  the .deb's second desktop file, under
                                   the app id (NoDisplay): the portals' host registry needs one
src-tauri/permissions/autogenerated/  per-command permissions written by tauri-build
src-tauri/src/tray.rs              tray icon + menu (Open, Translate Page, Open Page in Browser,
                                   Settings…, Quit); page items enabled only while a page is open;
                                   hidden only while the shortcut works (icon_visible,
                                   shortcut_works: pure, tested)
src-tauri/src/input.rs             start-field input → http(s) URL or a search (SearchEngine)
src-tauri/src/navigation.rs        which URLs the page and the bar may navigate to / hand to the
                                   OS browser (never an extension URL)
src-tauri/src/translate.rs         Google Translate proxy URL, language codes, LANGUAGES list
src-tauri/src/settings.rs          Settings (settings.json): tolerant per-field parse, atomic save
src-tauri/src/settings/theme.rs    the `theme` field: mode, 1–3 colors, intensity, grain (tolerant
                                   parse, tested; the default matches ui/lib/presets.ts)
src-tauri/src/settings/keys.rs     the `keys` field: action id → accelerator or "" (tolerant per
                                   entry, tested)
src-tauri/src/settings/change.rs   one settings change (Save's or a quick command's): every part
                                   checked, all or nothing — `apply_change` (pure, tested)
src-tauri/src/settings/site_modes.rs  the `siteModes` field: host → auto | phone | desktop, keys,
                                   the lookup that walks a name up to its last two labels (tested)
src-tauri/src/settings/after_close.rs  the `afterClose` field (LJ19.4): ready | ready15 | hibernate,
                                   tolerant parse (default ready), what each keeps (tested)
src-tauri/src/toggle.rs            the toggle table: (window state — hibernating, ready, open, the
                                   page in fullscreen —, command) → action (pure, tested)
src-tauri/src/cli.rs               --toggle / --show [url] / --hide / --size WxH /
                                   --start-at-login on|off / --settings [tab] / --extension <id> /
                                   --quit parsing
scripts/ci/smoke.mts               CI: drives the installed app (show, page, click, hide, start at
                                   login on/off, cookies and storage across close and quit),
                                   reads the OS login entry, screenshots
scripts/ci/smoke-log.mts           CI: smoke-log parsing and pass/fail rules, unit-tested
scripts/ci/smoke-command.mts       CI: the command bar's and history view's smoke-log rules,
                                   unit-tested
scripts/ci/smoke-theme.mts         CI: the theme runs (themes, screenshots) and their rules,
                                   unit-tested
scripts/ci/smoke-keys.mts          CI: typing on macOS runners — characters as real key codes (US
                                   layout), a Unicode string only for the rest (unit-tested)
scripts/ci/smoke-bundled.mts       CI: the bundled extensions run — which panels open, and when a
                                   panel counts as up (its URL with the nonce, its own theme call,
                                   the toolbar's label, none of its requests refused); unit-tested
scripts/ci/smoke-extensions.mts    CI: the extension run's rules (hello's report, the probe),
                                   unit-tested
scripts/ci/smoke-fullscreen.mts    CI: the fullscreen run (the smoke page's ?fs=both buttons,
                                   clicked where `page-screen` puts them, then the summon during a
                                   fullscreen) and its rules; unit-tested
scripts/ci/smoke-fullscreen-summon.mts  CI: the rules of a summon during the page's fullscreen
                                   (LJ19.5: the fullscreen run's `--toggle`, the main run's
                                   `--show <url>`); unit-tested
scripts/ci/smoke-fullscreen-off.mts  CI: the rules for fullscreen switched off (LJ14.1: the
                                   `linux-22.04` job, and nowhere else); unit-tested
scripts/ci/smoke-resize.mts        CI: the resize run (bursts of `--size`, the phone-mode
                                   hysteresis) and its rules; unit-tested
scripts/ci/smoke-sitemode.mts      CI: the site-mode run (the command bar's site rows, Settings'
                                   list, Windows' client hints from the server's header log) and its
                                   rules; unit-tested
scripts/ci/smoke-quick.mts         CI: the quick-commands run ("mobile", "dark", the theme rows,
                                   Copy link over the smoke page) and its rules; unit-tested
scripts/ci/smoke-keymap.mts        CI: the keymap run (History on Ctrl/Cmd+Shift+K: the new key,
                                   the old one, macOS's menu) and its rules; unit-tested
scripts/ci/smoke-tray.mts          CI: the tray run (the icon switched off; a second launch, Quit
                                   from the command bar, Settings' Quit and a relaunch, a Wayland
                                   session) and its rules; unit-tested
scripts/ci/smoke-downloads.mts     CI: the downloads run (two attachments, one refused while a
                                   panel shows, the window closed during a slow one) and its rules;
                                   unit-tested
scripts/ci/smoke-timing.mts        CI: the summon-timing runs — Stay ready's (from the ready window)
                                   and Hibernate's (first, warm, idle, cold after the file cache was
                                   emptied, the real shortcut; timing.md) —, their rules and
                                   ceilings; unit-tested (LJ19.1, LJ19.4)
scripts/ci/smoke-rest.mts          CI: the rest runs, one per "After closing" mode (LJ19.4): what
                                   stays at rest, a pinging page stopping at the close, memory at
                                   rest, keys typed right after the shortcut, the ready bar's web
                                   process killed (rest.md); unit-tested
scripts/ci/smoke-spaces.mts        CI: the over-full-screen run (Little Joan summoned over another
                                   app's full-screen window: `--show`, the real shortcut,
                                   `--settings`; macOS probes) and its rules; unit-tested (LJ19.2)
scripts/ci/smoke-spaces-helpers.mts  CI: that full-screen window per OS — a Swift app in native full
                                   screen, a borderless WinForms window, an X11 window with
                                   _NET_WM_STATE_FULLSCREEN — each writing its status every 500 ms
scripts/ci/smoke-zoom.mts          CI: the main run's zoom check (Ctrl+= narrows the page's viewport,
                                   Ctrl+0 brings it back) and its rules; unit-tested
scripts/ci/smoke-linux.mts         CI: which process is the app on Linux (under gdb, in the
                                   AppImage), read from /proc (pure parts unit-tested)
scripts/ci/apt-install.sh          CI: every Linux job's apt install — retries, 30 s timeouts, a
                                   deadline on update and download, then the Ubuntu archive first
                                   in the mirror lists (a crawling Azure mirror, LJ19.2)
tests/smoke/smoke.html             CI smoke page: codec report + a big "Go fullscreen" button
                                   (under a desktop agent a 600 px minimum width, like most
                                   desktop pages, unless ?fits=1; its report says `uamobile=`);
                                   with ?persist=set|check, the cookie and localStorage check;
                                   with ?fs=both, "Element fullscreen" and "Phone player"
tests/smoke/fullscreen-logic.mjs   ?fs=both's logic: the buttons' places, the player's route by
                                   user agent, one attempt's report (plain JS, unit-tested)
tests/smoke/smoke-logic.mjs        its logic (plain JS, JSDoc-typed, unit-tested)
tests/smoke/ext-probe.mjs          the smoke page's ?ext=probe: a web page trying an extension's
                                   API, files and panel (unit-tested)
.github/workflows/ci.yml           verify on ubuntu-24.04, windows-latest, macos-latest; package; smoke
.github/ISSUE_TEMPLATE/            issue forms: bug report, feature request; config.yml turns blank
                                   issues off and links private vulnerability reporting
.github/pull_request_template.md   what and why, how it was tested, the checklist
```

## Lifecycle

```
launch ──► tray icon + global shortcut; ~3 s later (Stay ready) the prewarm: the hidden window
           and the bar, no page ──► ready          (Hibernate: nothing ──► hibernating)
              │  shortcut / --toggle / --show [url] / second launch
              ▼
           ready: the window shown (cold: built first), the command bar open over it, its field
           focused; then the page webview, built after the bar (at once for a URL)
              │  type ──Enter──► the page loads; the bar becomes the 40 px toolbar above it
              │  Ctrl/Cmd+L ──► the command bar again (Esc: back to the page)
              │  shortcut while focused / Esc with no page / close button / --hide
              ▼
           the page webview closed (always: nothing of the page survives), size remembered;
           Stay ready: the window hidden, the bar back at its summon state ──► ready
           (for 15 minutes: then destroyed ──► hibernating)
           Hibernate: the window and the bar destroyed ──► hibernating
```

- **First launch:** when `settings.json` doesn't exist as `state::init` runs (a file that exists
  but can't be read or parsed doesn't count), setup registers the shortcut as usual, logs
  `first-run`, saves the default settings at once (so the welcome appears only once, even if the
  user quits straight away), and opens the window with `--show`'s path — in a Linux Wayland
  session once the portal answered (LJ17; `shortcut::welcome_now`), so the welcome names the
  desktop's key or explains `--toggle`. The command bar (in the
  bar webview) asks `start_info`, gets `firstRun: true` once (reading it clears it), and shows
  the welcome under its field: what the shortcut does (on a Linux Wayland session: bind
  `little-joan --toggle`), where the icon is (`welcome::tray_hint`), a "Start Little Joan at
  login" checkbox (`set_start_at_login`), and "Esc or the shortcut closes this window." The field
  keeps the focus. A first launch with flags (`--show <url>`, `--toggle`, `--start-at-login …`) acts on
  them instead, still saves the settings, and shows no welcome. Every later launch hibernates.
- Every request goes through `toggle::decide` (table below) and `joan::handle`, which runs off
  the main thread (`joan::dispatch`): building a webview window from the main thread can
  deadlock on Windows. A lifecycle mutex keeps two quick presses from creating two windows.
- **A cold summon, in order** (`joan/rest.rs` `create`; LJ19.1 measured and shortened it, LJ19.4
  moved the page after the bar): the hidden window, with the theme's own background colour
  (`theme_color.rs`: no flash of white or of the default theme while it opens); history.json
  starts loading on its own thread; the bar webview; the window placed, shown and focused, the
  command bar's field focused; then, once the bar's script runs (`start_info`, at most 3 s) and
  its frame went out (50 ms), the **page webview** (`bar/page.rs`) — and with it, still before any
  web page loads, the kept session cookies (Windows) and the page's user agent for its width. A
  summon with a URL builds the page at once and navigates; anything that needs the page before
  it exists (Enter in the command bar, an extension's panel, a navigation) builds it then
  (`ensure_page`, never on the main thread). The bar asks for its first suggestions as soon as its
  script runs. **macOS:** a summon holds an `NSProcessInfo` activity (user-initiated,
  latency-critical; `platform/mac/activity.rs`) from the request until the window is up, so App
  Nap — which may have throttled the idle app — doesn't slow it; nothing holds one at rest.
- **A summon from ready** (`summon`, LJ19.4): the window and the bar are there. A new extension
  session and registry, history loading on its own thread, the bar's webview shown again with
  its memory target back to normal (Windows), then `window.littleJoan.summoned({theme, timing})`
  — the bar starts over: the theme if it changed meanwhile, the command bar's first rows, its
  field focused — and the window placed by the same rules as a cold summon (the remembered size,
  centred on the monitor under the cursor), joined to the current Space (macOS, LJ19.2), shown
  and brought to front with the keyboard (Linux: the X server's time), the bar's field focused;
  the page webview 50 ms later, in the background (at once for a URL). macOS unhides the app first
  (a close hid it).
- **The prewarm** (`prewarm_after_launch`, Stay ready only): `PREWARM_AFTER` (3 s) after setup,
  after the tray and the shortcut, `Command::Prewarm` builds the hidden window and the bar — no
  page, no history, nothing shown, nothing focused, the bar's memory target low (Windows). Never
  on the first run's welcome (that window stays ready when it closes) nor while quitting; a
  summon that came first makes it do nothing. Switching "After closing" to a ready mode prewarms
  at once; switching to Hibernate destroys a resting window at once (`after_close_changed`).
- **A gone bar:** the engines report the bar's web process ending — WebView2's `ProcessFailed`
  (the renderer exited or hangs, or the browser process exited), WKWebView's
  `webViewWebContentProcessDidTerminate:` (Tauri's `on_web_content_process_terminate`), WebKitGTK's
  `web-process-terminated` — and `bar_process_ended` marks a resting window unusable: the next
  summon destroys it and builds everything anew (`toggle`'s Rebuild; smoke log `bar-gone`, then
  `released bar-gone`), never a blank window. A bar on screen is reloaded instead.
- **Where the window shows up (LJ19.2): where the user is, in front with the keyboard — over
  another app's full-screen window too — and never always-on-top.** Like Little Arc: it opens
  where you are and goes behind other windows once you click them. **macOS:** before the window
  (and Settings) is first shown, `platform/mac/spaces.rs` gives its NSWindow the collection
  behaviour `spaces::summoned` — `MoveToActiveSpace` (made active, it comes to the Space on
  screen instead of taking the user to its own; not `CanJoinAllSpaces`, which would put the open
  window on every desktop and full-screen Space at once) and `FullScreenAuxiliary` (it may show on
  a full-screen window's Space, another app's too for an accessory app like Little Joan); the
  level stays normal, so the summon's activation is what puts it in front, and it drops behind
  like any window after a click elsewhere. AppKit allows one of fullScreenPrimary / Auxiliary /
  None, so Little Joan's own window can't be made a full-screen Space (its green button zooms);
  a page's element fullscreen is WebKit's own window and is untouched (LJ36's guard as before).
  **A summon while the page shows an element fullscreen leaves it first** (LJ19.5, the owner's
  decision; `joan/fullscreen.rs`, every OS): the shortcut, `--show` and the tray's Open take the
  page out of its fullscreen — the page stays, a video keeps playing in the window — and then
  open the command bar with the keyboard in its field; the next press closes as usual (a summon
  with a URL leaves, then loads it). The page is asked as its own Exit button would ask
  (`document.exitFullscreen()` evaluated in it; no IPC). **macOS orders no window until WebKit has
  put the page back in it and that window is on the Space on screen** (`fullscreenState` not in
  fullscreen, the view in the window, `isOnActiveSpace`; LJ19.4-fix4's rule): ordered front during
  WebKit's fullscreen, the window would join WebKit's full-screen Space (`MoveToActiveSpace`), and
  when that Space closed with it inside, CI's Dock afterwards refused every app's full-screen Space
  for minutes (Landmines). A page that hasn't begun leaving after 1.2 s is closed by WebKit itself
  (`closeAllMediaPresentationsWithCompletionHandler:`, macOS 12+); after 4 s the summon gives up
  and only activates the app (the full-screen page with the keyboard), as LJ19.4-fix4 did. Windows
  and Linux wait for the page's fullscreen to end (WebView2's hook, WebKitGTK's
  `leave-fullscreen`) and up to a second for the window to leave its own. Before LJ19.5 the
  shortcut closed the window there (it had the keyboard), and `--show` changed nothing. The accessory activation policy (no Dock icon) stays — it is what lets an app's windows join
  other apps' full-screen Spaces. **Windows:** tao's `set_focus` already does what's needed:
  `SetForegroundWindow`, and when the foreground lock refuses it (a background process), an Alt
  tap through `SendInput` and again; the global shortcut's `WM_HOTKEY` grants the right anyway,
  and a second launch passes it on (`AllowSetForegroundWindow`, tauri-plugin-single-instance). The
  window is never topmost: a full-screen window that is topmost itself stays above it.
  **Linux:** the window manager raises and focuses on GTK's `present` (`_NET_ACTIVE_WINDOW`) —
  if the request isn't older than the user's last input in another window (focus-stealing
  prevention). tao presents with `GDK_CURRENT_TIME`, which GTK turns into the last input event
  Little Joan saw — after the user once typed into it, an old keystroke, so a later summon over a
  full-screen window was refused and left behind it. `platform/linux/front.rs` presents with the X
  server's time now (`gdk_x11_get_server_time`; the shortcut's key goes to global-hotkey's own X
  connection, never to GTK); on Wayland GTK and the compositor decide, as before. A window manager
  may still refuse when the user was typing in another window at that very moment.
- The window opens centred in the work area of the monitor under the mouse cursor, at the last
  size the user left it (logical pixels, saved on close unless fullscreen/maximized/minimized).
- **Every close goes through the same path.** The close button, Cmd+W / Alt+F4
  (`CloseRequested` is prevented and turned into `--hide`), Esc in the command bar with no page,
  the shortcut, and `--hide` all end in `joan/rest.rs` `close`: remember the size, save history,
  then **the page webview is closed** (`bar/page.rs`: its session cookies kept on Windows, a
  download under way ended — WebView2's noted and cancelled, since the window's environment now
  outlives the page —, macOS navigating it to `about:blank` and waiting for that, at most 400 ms,
  then `Webview::close`). **Stay ready:** the window is hidden first, put back to its remembered
  size (not fullscreen, maximized or minimized), its title "Little Joan"; the bar goes back to its
  summon state (`window.littleJoan.rest()`: the command bar, an empty field, no rows, history view,
  welcome or status); Windows hides the bar's controller and sets its memory target low
  (`ICoreWebView2_19`); macOS hides the app (`NSApp.hide`) so the app the user came from gets the
  keyboard back — unless Settings is on screen. For 15 minutes: a timer, cancelled by a summon,
  destroys the window then. **Hibernate:** the bar is unloaded too (macOS) and the window destroyed,
  as before.
- **Quit** (tray, `--quit`, Cmd+Q) is `joan::quit`. Nothing new is built from then on. On macOS
  it first closes an open window (so the page is unloaded) and destroys a ready one (the bar
  unloaded), waiting for that at most 2.5 s: after a plain `app.exit` (`process::exit`, webviews
  never unloaded) CI saw the next launch's WebKit never load a page. Windows and Linux exit at
  once — their stores came through without it.
- **The page's address comes from its load events** (`page_address.rs`, `bar::page_url`): the
  `Started` / `Finished` events carry it, and the app's own navigations set it at once. wry 0.57
  unwraps WKWebView's address, which is unset until the first navigation commits (and again
  after a first navigation that never did), and that panic ends the app, so the engine is asked
  (`Webview::url`) only by the once-a-second check for in-page changes, and only while the last
  load event was `Finished`; an answer that raced with a newer navigation is dropped.
- **Releasing the page on macOS:** WKWebView's page processes were measured outliving a
  destroyed window by minutes (which would also let a video's audio keep playing). On macOS the
  page is navigated to `about:blank` and closed once that loaded (at most 400 ms); a window being
  destroyed gets its bar unloaded too and 400 ms more. CI's rest runs check that a pinging page
  stops within 2 s of the close and that its web processes go, in every mode (LJ19.4). WebView2
  and WebKitGTK release a page's processes when its webview goes without help.
- **Single instance:** `tauri-plugin-single-instance` forwards a second launch's arguments to
  the running app (`--toggle`, `--show [url]`, `--hide`, `--start-at-login on|off`,
  `--settings [tab]`, `--quit`; no flags = `--show`). On Linux it needs a D-Bus session bus; on macOS it uses a socket in `/tmp`.
  `--start-at-login` is handled before the toggle table and never touches the window. `--quit`
  takes the tray's Quit path (`joan::quit`) and wins over any window flag; a `--quit` launch with
  nothing running just ends.
- **Global shortcut:** registered from settings; if it fails to parse or is taken, the default
  (`CommandOrControl+Shift+Space`) is tried; if that fails too, the app still runs. global-hotkey's
  Linux backend grabs keys on X11, so in a Wayland session it hears them only while an X11 window
  has the focus. **In a Linux Wayland session** (`XDG_SESSION_TYPE=wayland` or `WAYLAND_DISPLAY`
  set, `welcome::wayland_session`) `shortcut::register_at_launch` asks the **XDG GlobalShortcuts
  portal** first (LJ17; `platform/portal.rs`, ashpd on tauri's tokio runtime): on its own session-bus
  connection it registers as `io.github.memoliertorerr.littlejoan` with the portal's host registry
  (`org.freedesktop.host.portal.Registry`, x-d-p 1.19.1+; it needs a desktop file of that name,
  which the `.deb` installs; if it refuses, a fresh connection, which the portal identifies by
  itself from the app's systemd unit), creates a session (15 s limit, D-Bus activation included),
  listens for `Activated` and `ShortcutsChanged`, and binds one shortcut — id `toggle`, "Summon or
  dismiss Little Joan", the preferred trigger converted from the settings' accelerator into the
  XDG shortcuts spec (`CommandOrControl+Shift+Space` → `CTRL+SHIFT+space`; no preference for a key
  the spec can't name) — with no time limit, since the desktop may ask the user first. The desktop
  owns the key and may pick another: the trigger it reports (GNOME: "Press <ctrl><shift>space") is
  what the welcome and Settings show. Meanwhile (`PortalState::Asking`) nothing holds the
  shortcut. Registered: `Activated` for `toggle` dispatches `Command::Toggle`, the session lives as
  long as the process, `ShortcutsChanged` updates the trigger, `active_shortcut` stays `None`, and
  Save swaps nothing (the saved accelerator is only the preferred trigger for the next launch).
  Anything else — no session bus, no portal or none with GlobalShortcuts (GNOME 46), a refusal,
  the user declining, a timeout, a panic in ashpd, or the session ending later — logs
  `shortcut-portal-unavailable <why>` and does what v0.1 did: global-hotkey on XWayland, the
  `--toggle` hint (`StartInfo.waylandNoShortcut`, Settings → Keyboard) and the icon back
  (`shortcut::portal_answered`). zbus runs on tokio for this (ashpd's `tokio` feature, shared with
  the single-instance and opener plugins' zbus 5): its blocking API, which those plugins use, runs
  on a tokio runtime of zbus's own. X11 sessions, Windows and macOS never ask the portal.

## Commands and capabilities

**Two trust levels.** The page window holds two webviews. **`joan-bar`** is trusted: every screen
of the app's own lives there — the toolbar, the command bar, the welcome, the history view.
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
| `capabilities/bar.json` | webview `joan-bar` | `allow-open-input`, `allow-page-url`, `allow-focus-page`, `allow-set-overlay`, `allow-go-back`, `allow-open-in-browser`, `allow-hibernate`, `allow-history-list`, `allow-suggest`, `allow-history-remove`, `allow-history-clear`, `allow-open-settings`, `allow-start-info`, `allow-set-start-at-login`, `allow-set-theme-tokens`, `allow-extensions-list`, `allow-open-extension`, `allow-site-mode-info`, `allow-set-site-mode` (only the bar, a `capability_tests.rs` test says), `allow-quit-app` (the command bar's Quit; only the bar, a test says), `allow-quick-state`, `allow-quick-command` (the quick commands; only the bar, a test says), `allow-keymap-info` (the bar and Settings only, a test says), `allow-reveal-download` (the toolbar's "Show in folder"; only the bar, a test says), `core:window:allow-start-dragging` (the toolbar's drag regions), `core:window:allow-internal-toggle-maximize` (a double click on them zooms the window; `capability_tests.rs` allows it to no other capability) |
| `capabilities/settings.json` | window `settings` | `allow-get-settings`, `allow-save-settings`, `allow-suspend-shortcut`, `allow-set-start-at-login`, `allow-history-clear`, `allow-preview-theme`, `allow-extensions-list`, `allow-reload-extensions`, `allow-open-extensions-folder`, `allow-keymap-info`, `allow-quit-from-settings` (General's Quit Little Joan; only Settings, a test says), `core:window:allow-close` |
| — | `joan-page`, window `joan`, remote URLs | nothing |

Capabilities are local unless they list `remote` URLs, and none does. No `core:default`
anywhere; the only core permissions are the bar's `core:window:allow-start-dragging` and
`core:window:allow-internal-toggle-maximize` (and the settings window's `core:window:allow-close`).

| Command | Called by | Does |
|---|---|---|
| `open_input(input)` | the bar, Enter | `input::normalize` with the configured engine → load it in the page webview (the command bar closes), focus the page |
| `page_url()` | the bar, on load | what the bar shows now (`null` on the start page) |
| `focus_page()` | the toolbar, Esc | give the keyboard back to the page |
| `set_overlay(on)` | the bar: a click on the toolbar's domain (on), Esc or × in the command bar with a page (off) | open the command bar with the address selected (as Ctrl/Cmd+L), or close it and give the keyboard to the page |
| `go_back()` | the toolbar's back button | the page action Back (`history.back()` in the page, as Alt+Left / Cmd+[) |
| `open_in_browser()` | the toolbar's "open in default browser" | the page action OpenInBrowser: the page goes to the default browser, the window hibernates |
| `hibernate()` | the bar, Esc with no page | close the window (same path as `--hide`) |
| `start_info()` | the bar, on load | `{ shortcut, platform, firstRun, trayHint, startAtLogin, waylandNoShortcut, portalShortcut, theme }`: the theme the bar applies, and the welcome's facts; `firstRun` is true once; `waylandNoShortcut` is a Wayland session whose portal doesn't hold the shortcut, `portalShortcut` `{ trigger }` while it does (LJ17; `shortcut` is then `null`); logs `theme <mode> colors=<n>` |
| `get_settings()` | settings window, on load | settings + engine and language lists + platform + `shortcutProblem` (the saved shortcut failed at launch, LJ44) + `waylandNoShortcut` + `storage` (`folder`: where settings.json and history.json are; `siteData`: the web engine's folder, or on macOS WebKit's store) + `version` + `builtInSiteModes` (`[{ host, mode }]`: YouTube on desktop on macOS and Linux, none on Windows); logs `settings-loaded` |
| `save_settings(settings)` | settings window, Save | check every part first (`settings::apply_change`: a bad engine id, language, theme mode or color refuses the whole Save, nothing changed); swap the shortcut live (old one restored if the new one is taken) — not while the portal holds or is asked for it (LJ17): the desktop owns that key; then `settings_live::apply` — the path the quick commands take too — and save. Sends `shortcut`, `searchEngine`, `translateTo`, `mobileSites`, `rememberHistory` and optionally `theme` (valid as it stands — `parse_strict`; left out, the saved one stays), `showTrayIcon` and `siteModes` (General's list, only when it was edited — so a pill click while Settings is open isn't undone). A new theme goes at once to the open window's bar (`window.littleJoan.setTheme`) and to the settings window (`window.littleJoanSettings.setTheme`), both by `eval` |
| `suspend_shortcut(suspended)` | settings window, recorder | pause / resume the active shortcut while recording |
| `set_start_at_login(enabled)` | settings window, Save (only when the switch changed); the welcome's checkbox (the bar) | add / remove the OS login entry; returns whether it exists afterwards (read back) |
| `history_list()` | the history view, when it opens and after a change | every record, latest visit first: `[{ url, title, host, lastVisit, visitCount }]` (empty with no window); the view searches, groups and draws them; logs `history-list <n>` |
| `suggest(query)` | the command bar, as you type (60 ms after the last key) | `[{ kind: topSite \| history \| search \| starter, title, url, host, score }]` (`suggest.rs`): nothing typed → up to 8 top sites, leaving out the open page's site, or the 4 starters when there are none; typing → up to 8 history matches, then one `search` row (title = the engine's name, url = its results page) |
| `history_remove(url)` | the command bar (Shift+Delete on a history row), the history view (× or Shift+Delete) | forget that record and rewrite `history.json` at once; returns whether there was one |
| `history_clear(range)` | the history view's "Clear history…"; Settings → Privacy | `range` = `hour` \| `day` \| `week` \| `all`: forget the visits of the last hour, 24 hours, 7 days (rolling windows), or everything; a page left without visits goes. Rewrites `history.json` at once — with no window open, the file itself (`all` always writes an empty one). Returns how many pages went |
| `preview_theme(theme?)` | settings window, the Appearance tab, as you edit (at most once a frame) | show `theme` (checked like a saved one, never saved) in the open page window's bar, and in a page window summoned meanwhile (`AppState::theme_preview`, which `start_info` prefers); `null` = the saved theme again. Closing Settings without saving (Cancel, the close button) puts the saved theme back the same way. Logs `theme-preview <mode> colors=<n>` (or `ended`) |
| `set_theme_tokens(tokens)` | the bar, each time it paints a theme (`apply-theme.ts`'s `onPaint`) | keep `{ scheme, vars }` — the `--lj-*` variables it computed — in memory for extension panels (`littleJoan.theme`); checked first (names `--lj-*`, short CSS values without `;{}<>\`), a bad report is dropped; forgotten when the window closes |
| `extensions_list()` | the command bar, each time it opens; Settings → Extensions | `{ developerMode, folder, extensions: [{ id, name, version, description, author, kind: bundled \| developer, enabled, icon (data: URI), permissions, commands, pageActions }], skipped: [{ name, reason }], disabledBuiltIns }` (`extensions/list.rs`; the last: the quick answers switched off); logs `extensions-list <n> skipped=<m>` |
| `open_extension(id, command, pageAction)` | the command bar, an extension's row | opens that command's panel — or, for a page action, its panel for the open web page (refused without one), handing it the page's address and title only when its manifest asks for `pageInfo`; never a switched-off extension (`session::open`) |
| `reload_extensions()` | Settings → Extensions → Reload extensions | read the registry again; returns the new list |
| `open_extensions_folder()` | Settings → Extensions → Open extensions folder | create `<config dir>/extensions/` if needed and show it in the file manager (`tauri_plugin_opener::reveal_item_in_dir`) |
| `site_mode_info()` | the bar, on load (Rust also pushes it: `window.littleJoan.setSiteMode`, on every address change and agent switch) | `{ host, mode, builtIn, agent }` for the open web page's site — `host` the key a mode is kept under (`youtube.com` for `m.youtube.com`), `mode` `auto` \| `phone` \| `desktop`, `builtIn` whether the built-in list decides, `agent` `phone` \| `desktop`; `null` without a web page (the start page, a panel) |
| `set_site_mode(mode)` | the toolbar's pill, the command bar's site rows ("Desktop site" / "Phone site" / "Automatic", LJ42's quick rows) | keeps `mode` for the open web page's site in `siteModes` — the site is Rust's (`bar::page_url`), the bar sends only the mode; `auto` removes the entry, or keeps `"auto"` when a built-in or a parent's entry would still decide —, saves the settings and applies it at once: the agent switches and the page loads again only when it really changed (going to desktop on a known phone host: its desktop address). Refused without a web page, or with 500 sites already. Logs `site-mode <site> <mode>`; returns the new info |
| `quick_state()` | the command bar, each time it opens (with `extensions_list`), and after a quick command | `{ page: { url, title } \| null (a web page only; the title is the window's), theme, searchEngine, engines: [{ id, name }], mobileSites, rememberHistory, startAtLogin: bool \| null, showTrayIcon, iconForced }` — what the quick rows show (LJ42) |
| `quick_command(command)` | the command bar's quick rows (LJ42) | one of a fixed set, `{ kind, …fields }` (a serde enum with `deny_unknown_fields`: any other kind or field is refused): `translate`, `openInBrowser`, `reload` (the page actions; refused without a web page), `setThemeMode { mode }`, `setTheme { theme }` (a preset's whole theme, checked like Save's), `setSearchEngine { id }`, `setMobileSites { on }`, `setRememberHistory { on }`, `clearHistory { range }` (as `history_clear`; returns how many pages went), `setStartAtLogin { on }`, `openSettings { tab }`, `reloadExtensions`, `setTrayIcon { on }`. Settings ones go through `settings_live::apply` (as Save) and an open settings window rebases its draft (`window.littleJoanSettings.savedChanged(<settings>, <startAtLogin \| null>)`, by `eval`). Never: the shortcut or keys, developer mode, switching extensions, custom colors — Settings only. Logs `quick-command <kind> [<value>]`, never a URL or a title |
| `keymap_info()` | Settings' Keyboard tab (LJ44); the bar may ask too | `{ platform, actions: [{ id, title, binding, defaults }], globalShortcut, shortcutProblem, portalShortcut, systemShortcuts }` — `portalShortcut` `{ trigger }` while the XDG GlobalShortcuts portal holds the summon shortcut (LJ17), else `null` — — the last macOS's own shortcuts as read from its preferences (`[{ id, enabled, accelerator }]`, empty elsewhere; smoke log `system-shortcuts <n> keys=<k>`) — every keymap action in Settings' order, titles as the macOS menu has them, `binding` its changeable key's accelerator (`null`: no key), `defaults` its default accelerators (the first is what Reset brings back), and the saved global shortcut (LJ43) |
| `reveal_download()` | the toolbar's **Show in folder**, after a download was saved (LJ15) | shows the last file saved in this window in the file manager, selected (`tauri_plugin_opener::reveal_item_in_dir`) — which file is Rust's, never the bar's; an error when there is none |
| `quit_app()` | the command bar's **Quit Little Joan** row (LJ41) | `joan::quit`, as the tray's Quit and `--quit`: history saved, on macOS the window closed first; logs `quit` |
| `quit_from_settings()` | Settings → General's **Quit Little Joan** (LJ19.7) | the same `joan::quit`; logs `settings-quit`, then `quit`. A command of its own so the settings window gets no bar command: `capability_tests.rs` allows it to the settings window's capability only (never the bar's, which keeps `quit_app`, never the page). What Settings hasn't saved goes with its window, as with Cancel |
| `open_settings(tab?)` | the command bar's "Settings & extensions" row (tab `extensions`) and its **Settings** row (tab `general`) | open the settings window on `tab`, or focus an open one and switch it to `tab` (`settings_window::open`, the same path as `--settings`); no tab or an unknown one: a new window opens on General, an open one stays where it is |

`get_settings` also reports `startAtLogin` (`null` when the entry can't be read).
`capabilities/settings.json` grants the `settings` window the four settings commands,
`history_clear` and `core:window:allow-close` (its Cancel / Save buttons close it). tauri-plugin-autostart is
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
| Extension storage, developer extensions | `extension-data\<id>.json`, `extensions\<id>\` next to `settings.json` | the same, next to `settings.json` | the same, next to `settings.json` |
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
  in dev builds and CI). Loaded when the page window opens — on a thread of its own while the
  window and its webviews are built (LJ19.1: a 3,000-page file took 20–50 ms off every summon);
  every use of the window's history (suggestions, the history view, recording, removing,
  clearing, closing) waits for that read first, at most 10 s, and a read that ends after its
  window closed keeps nothing —, kept in `AppState` while it exists,
  saved atomically (temp file + rename) when it closes, every 30 s while there are unsaved
  visits, and on Quit; dropped from memory on every close — also while the window stays ready
  (LJ19.4: each summon reads it again, on its own thread), so it is never held at rest.
- **Format:** `{ "version": 1, "entries": [ { "url", "title", "host", "visitCount", "visits":
  [ { "at": <unix ms>, "kind": "typed" | "link" | "reload" } ] } ] }` — one record per URL (the
  fragment dropped, the query kept), `host` lower case without `www.` (with a non-default
  port), the all-time visit count and the last 20 visits. Titles are cut at 300 characters;
  URLs over 4,096 bytes aren't recorded. A bad or missing file is empty history; a bad record
  or visit is skipped alone.
- **Cap:** 5,000 URLs; past that the least frecent go (on a tie, the least recently visited).
- **Removing and clearing** (the history view, Settings → Privacy; `history_remove`,
  `history_clear`) change the window's history and rewrite `history.json` at once; with no
  window open, `history_clear` reads the file, changes it and writes it back, so nothing is held
  in memory. Clearing a range forgets the visits in it (rolling windows: the last hour, 24 hours,
  7 days); a page left without visits goes, one with older visits stays with a lower count;
  "everything" writes an empty file even when the old one couldn't be read. One lock covers every
  read-modify-write of the file and every change of the window's history (open, close,
  autosave, remove, clear), so an autosave's older copy can never land after a clear.
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

**The settings window** (`settings_window.rs`, `ui/settings.*`; about 600×480, not resizable)
opens from the tray (Settings…), Cmd+, on macOS, the command bar's "Settings & extensions" (on
Extensions) and its **Settings** row (on General), and `--settings [tab]` (forwarded by single-instance; it never opens the page
window). A sidebar on the left holds the tabs (↑/↓, Home, End move between them), the tab is on
the right, and **Save / Cancel** along the bottom cover every tab (change detection compares
all fields, whichever tab they are on):

| Tab | Holds |
|---|---|
| General | the shortcut — a line with it and **Change…**, which switches to Keyboard (and the launch's problem line, below) —, **Show the icon in the menu bar** (macOS) / **in the system tray** (Windows, Linux) — `showTrayIcon`, with the note that the shortcut or opening Little Joan again reaches it and that the icon comes back by itself whenever the shortcut can't work —, **After closing** (LJ19.4, `afterClose`: **Stay ready** / **Stay ready for 15 minutes** / **Hibernate**, with a note under the list saying what the chosen one does — "Opens instantly; holds about N MB more memory while closed than Hibernate" with this OS's figure from `READY_MEMORY_MB` (CI's own-memory numbers: macOS 30, Windows 110, Linux 200), "… then hibernates", "Uses the least memory while closed; opens a little slower" — and that closing always ends the page; drawn as text; it takes effect on Save, at once — `after_close_changed`), search engine, translate language, mobile sites, **Phone or desktop per site** (the user's entries and this OS's built-in ones — "youtube.com — Desktop, built in on macOS and Linux" —, each with Remove, or Reset for an overridden built-in; Remove on a built-in keeps `"auto"` for it), start at login, and last **Quit Little Joan** (LJ19.7: a Quit button, `quit_from_settings`; the note under it says that changes not saved are dropped, as with Cancel — the window goes with the app; `ui/settings/quit.ts`, tested) |
| Keyboard | **Summon Little Joan** — the global shortcut's recorder (moved here from General; it pauses the shortcut while it listens, as before), Reset when it isn't the default; on a Linux Wayland session without the portal a note to bind `little-joan --toggle` instead; while the portal holds the shortcut (LJ17) the desktop's key as text instead of the recorder (no Reset, no problems: the desktop's key is never judged, so it can't block Save, and Reset all leaves the preferred trigger alone), with "Change it in your desktop's keyboard settings." — and General's line shows that key with the same note instead of **Change…** — then **Little Joan's keys**: every keymap action (`keymap_info`) with its key as a recorder button (the next key with a modifier, or an F-key; a letter is the one the key types in the user's layout; Esc cancels, Backspace means no key), **No key**, **Reset** when it isn't the default, its problems under it; **Reset all** at the bottom (every key and the summon shortcut). Problems (`keyboard-logic.ts`, the keymap's own rules): errors block Save and say why — "⌘L is already Open Location", "That's the shortcut that summons Little Joan", "Plain keys type text — add ⌘ or ⌃", "On macOS a key needs ⌘ or ⌃", the editing keys ("⌘C is used for editing in pages") —; warnings don't — the OSes' reserved shortcuts ("macOS uses ⌘Space for Spotlight", `reserved-shortcuts.ts`, curated from the OSes' documentation; `Super+*` on Windows and Linux), "Ctrl+Alt is AltGr on many keyboards". **On macOS the user's own changes refine the curated table** (decided by a CI runner's export, LJ44): Rust reads the `com.apple.symbolichotkeys` domain (NSUserDefaults, read-only; `system_shortcuts.rs`), which lists only entries that were written — an entry switched off (or moved to another key) drops that curated warning (curated macOS entries carry their domain id: Spotlight 64, input sources 60 / 61, screenshots 28 / 30 / 184, …), and an entry that is on with a key warns there (under the curated name when its id is known, else "one of its keyboard shortcuts"); entries without a key say nothing, so the curated defaults stay. A saved summon shortcut that isn't one shows as written and is an error until it is recorded again or Reset. On Windows a note says WebView2's own Ctrl+R, F5 and Alt+← / Alt+→ work beside these. The summon recorder gets the same rules (one of Little Joan's keys → error, a reserved one → warning). When the saved shortcut couldn't be registered at launch (`shortcut::settle` keeps `AppState::shortcut_problem`: what was wanted, what is active instead, `taken` or `invalid`; `get_settings` and `keymap_info` report it), Keyboard and General say so: "⌘⇧K couldn't be registered: another app or the system already uses it. Little Joan is using ⌘⇧Space instead." (or "…Little Joan has no shortcut now."); a Save whose shortcut registers clears it. Save sends `keys` only when they were edited |
| Appearance | the theme editor: System / Light / Dark beside the heading; a live preview (a toolbar strip and a command bar card over the theme's gradient); preset swatches (each preset's own colors; the one the theme matches has a ring); up to three color pads — x = hue 0–360°, y = chroma from the muted maximum (0.12, top) to grey, drawn at OKLCH L 0.75 where every hue reaches it in sRGB — with a dot per color: drag it, or focus a pad and use the arrow keys (5° / 5 %, Shift 30° / 20 %); **+** adds a color 45° along from the last (up to 3), × on the selected pad removes it (down to 1); Intensity and Grain sliders. Every edit re-themes this window and calls `preview_theme`; Save sends the theme with the rest |
| Extensions | "Experimental"; **Built-in** (the five quick answers, ids `builtin:<name>`), **Bundled** and **Developer** groups — each extension with its icon, name, version, author, description, its permissions in plain words, and a switch (`disabledExtensions`, saved with Save); **Developer mode** (`developerMode`) with its warning, the folder, **Open extensions folder**, **Reload extensions**, and what couldn't be loaded and why. Saving forgets the registry, so the next use reads it with the new switches |
| Privacy | **Remember history** (`rememberHistory`, saved with Save); **Clear history** — a range (last hour, 24 hours, 7 days, everything), "Clear…", a question, then `history_clear` at once (not a setting, so Cancel doesn't undo it) and what went; **what is stored where** (`get_settings`'s `storage`, plus "session cookies in memory only") |
| About | version, license, the repository link, "Inspired by Little Arc…" as plain text |

A new window gets its first tab from an initialization script (`window.__littleJoanSettingsTab`)
that runs before the page's own; an open window is focused and switched with
`window.littleJoanSettings.showTab(<tab>)`. Tab names come from `SettingsTab`, never from user
text. The window may only navigate within the app's origin; a link (About's) opens in the
default browser (web URLs only), like the page's new-window requests. Smoke log:
`settings-window <tab>` (or `focus`), `settings-loaded`. **A quick command that changes a
setting while Settings is open** (LJ42) reaches it as `window.littleJoanSettings.savedChanged(...)`:
`rebase` (settings-logic.ts, tested) keeps what the window edited and lets every field it didn't
edit follow the change — so a later Save neither undoes the quick command nor loses an edit.

The Keyboard tab (General before LJ44) records the global shortcut by listening to the next key press. While it listens, the active global shortcut is **suspended** (unregistered) so
pressing it doesn't summon or dismiss the page window; it is resumed when recording stops, when
Save runs, and when the window closes for any reason. A new shortcut is registered before the
old one is forgotten: if registration fails, the old one is registered again and the window
shows the error.

## The page

- **Navigation guard** (`on_navigation` → `extensions::session::allow_page_navigation`): web
  schemes, engine-internal schemes, and the app origin (`navigation::is_allowed_navigation`); an
  extension URL only when the app opened it (above). Refusals are logged.
- **Title:** the window title follows the page title (`on_document_title_changed`).
- **New windows** (`target=_blank`, `window.open`): denied inside Little Joan; web URLs are
  handed to the OS default browser (`tauri_plugin_opener::open_url`, used as a plain function —
  the opener plugin and its JS commands are not registered).
- **Keys:** remote pages get no IPC and no injected scripts, so page keys are caught natively
  before the page sees them. **Every one comes from the keymap** (`keymap.rs`, LJ43; Ground Rule
  25): the native hooks (`platform/win.rs`, `platform/linux.rs`) turn a press into a `Key` and
  `Mods` (`keys.rs`: Windows virtual keys, GDK key names) and ask `AppState::keymap`; macOS's menu
  (`menu.rs`) takes its page items' key equivalents from it. These are the **default keys**:

  | Keys | Windows (WebView2 `AcceleratorKeyPressed`) | Linux (GTK `key-press-event`) | macOS (app menu) |
  |---|---|---|---|
  | Close window | Ctrl+W | Ctrl+W | Cmd+W |
  | Edit the address (focus the URL bar, text selected) | Ctrl+L | Ctrl+L | Cmd+L |
  | Reload | Ctrl+R / F5 (WebView2 built-in) | Ctrl+R / F5 | Cmd+R |
  | Back / Forward | Alt+Left / Alt+Right (built-in) | Alt+Left / Alt+Right | Cmd+[ / Cmd+] |
  | Translate page | Ctrl+Shift+T | Ctrl+Shift+T | Cmd+Shift+T |
  | History view (the command bar's overlay) | Ctrl+H | Ctrl+H | Cmd+Y (View menu) |
  | Zoom in (LJ16) | Ctrl+= (also Ctrl+Shift+=, keypad +) | Ctrl+= (also Ctrl+Shift+=, keypad +) | Cmd+= (View → Zoom In) |
  | Zoom out | Ctrl+- (also keypad −) | Ctrl+- (also keypad −) | Cmd+- (View → Zoom Out) |
  | Zoom back to 100 % | Ctrl+0 (also keypad 0) | Ctrl+0 (also keypad 0) | Cmd+0 (View → Actual Size) |
  | Open page in default browser (then hibernate) | tray menu | tray menu | Cmd+Shift+O, tray menu |
  | Copy / paste / select all | built-in | built-in | Edit menu (needed by WKWebView) |

  Matches are exact: Ctrl+Shift+W is not Ctrl+W, and a Ctrl+Alt press (AltGr) never matches a
  Ctrl key. On macOS the menu bar is hidden (Accessory app) but its key equivalents work; Cmd+Q
  quits for real.
- **The keymap** (`keymap.rs`, pure; tests in `keymap/tests.rs` and `keymap/zoom_tests.rs`):
  actions `open-location`, `history`, `translate`, `reload`, `back`, `forward`, `zoom-in`,
  `zoom-out`, `zoom-reset` (LJ16), `open-in-browser`, `close` (LJ49 adds find), each with an
  ordered list of default bindings per OS — the first is the one Settings
  shows and changes, the rest are fixed aliases (Linux's F5 for reload); an action may have none
  (Windows leaves Reload, Back and Forward to WebView2, which handles Ctrl+R, F5 and Alt+← / Alt+→
  itself; Open in browser has no key on Windows and Linux). Bindings are accelerators in the global
  shortcut's syntax with `KeyboardEvent.code` key names (`CommandOrControl+Shift+KeyT`,
  `Alt+ArrowLeft`, `F5`), `CommandOrControl` resolved per OS only when a key is matched; a key is
  matched by what it types on a US layout (a letter key is the key that types that letter in the
  user's layout, as Windows' virtual keys, GDK's key names and macOS's key equivalents all do).
  **settings' `keys`** (`settings/keys.rs`): `{ "<action-id>": "<accelerator>" | "" }` — unknown
  ids and unparsable accelerators dropped on load, `""` = no key; an override replaces the
  action's first binding and drops its aliases (F5 reloads only while reload has its default); one
  equal to the default is the default; a typing key in a hand-edited file is never bound. Typing
  keys can never be bound (no modifier and not an F-key, or Shift alone with a key that types); on
  macOS a key needs ⌘ or ⌃. `keymap::validate` (Settings' Save, `settings::apply_change`) also
  refuses two actions on one key (aliases included) and an action on the global shortcut — and a
  new global shortcut that is one of the keys. A save rebuilds `AppState::keymap` and, on macOS,
  sets the menu again (on the main thread). Not in the keymap (fixed): Esc / Enter / arrows / Tab
  in the bar, the Edit menu, Cmd+Q and Cmd+, on macOS. macOS's Close Window is Little Joan's own
  item now (its key from the keymap): with Settings in front it closes Settings, as the standard
  item did. `keymap_info` hands it all to Settings (LJ44).
- The app origin is `tauri://localhost` on macOS/Linux and `http://tauri.localhost` on Windows
  (`keys::app_origin`).
- **Page zoom** (LJ16; `zoom.rs` pure, `bar/zoom.rs`): the keymap's `zoom-in` / `zoom-out` /
  `zoom-reset` are page actions like the others (`PageAction::ZoomIn` …: WebView2's
  `AcceleratorKeyPressed`, the GTK key hook, the macOS View menu's Zoom In, Zoom Out, Actual Size),
  so the Keyboard tab lists them and they can be changed. They step through 50, 67, 75, 80, 90,
  100, 110, 125, 150, 175, 200, 250 and 300 % — the next level up or down from any value, stopping
  at the ends; reset is 100 % — and set the page webview's zoom with `Webview::set_zoom`
  (WebView2's controller `ZoomFactor`, which "becomes the new default zoom … across navigations";
  WKWebView's `pageZoom` and WebKitGTK's zoom level, properties of the view) — **never Tauri's
  zoom-hotkeys option**, which on macOS and Linux injects a script into the page that needs
  `core:webview:allow-set-webview-zoom` (IPC for remote pages) and on Windows would switch on
  WebView2's own zoom keys and pinch zoom (`IsZoomControlEnabled` stays off); a test reads every
  source file for its name. **Per window:** the level lives next to the window's other state
  (`bar/zoom.rs`), survives the window's navigations, starts at 100 % on every summon and is never
  saved. Each change shows "Zoom 110 %" in the toolbar's status area for 1.5 s ("Zoom 100 %" once,
  then nothing); smoke log `zoom <percent>`. Phone mode is unaffected: its 600 / 640 px are the
  window's logical width, not the page's CSS width.
- **Downloads** (LJ15; `downloads.rs` decides, `bar/download.rs` runs it): a page's download goes
  to the OS Downloads folder (`PathResolver::download_dir`; without one — Linux without XDG user
  dirs — `~/Downloads`, created when missing). Only web downloads are saved: `http` / `https`, and
  the `blob:` / `data:` documents a web page makes — never an extension's URL in any form (`ljext:`,
  Windows' `http://ljext.*`, a blob of either), the app's own pages, `file:` or another scheme
  (`download-refused scheme`) — and **nothing while the page shows an extension's panel** (the
  guard's admitted or the committed one: `download-refused panel`): extensions have no permission
  to drop files there. **The name** is the engine's suggestion (the server's
  `Content-Disposition`, a link's `download`), else the URL's last path segment, else `download`,
  cleaned — path separators and the characters Windows refuses become `_`, control and format
  characters (bidi overrides that could disguise an extension) go, no leading dots (no hidden files,
  no `..`), no trailing dots or spaces, a Windows device name gets a `_` in front (`_nul.txt`) —,
  capped at 200 bytes keeping its extension, and made unique: a name taken on disk or by a
  download still under way gets ` (n)` before its extension (`report (1).pdf`, `archive (1).tar.gz`;
  the extension is the last short plain dot part, with a `.tar` before it). The destination is
  kept by URL until the download ends (WKWebView reports a finished download without its path),
  and the last file saved is what **Show in folder** reveals. **Per engine:** WebView2 and WKWebView
  report through Tauri's `on_download` (wry marks a WebView2 download handled, so WebView2's own
  download flyout doesn't show). **WebKitGTK goes through `platform/linux/downloads.rs`:** wry
  registers its download handler on the web context — which both webviews of every window share
  (and Settings, in installed builds), and which Tauri keeps for the app's lifetime on Linux — once
  per webview built, so the first webview's handler (the bar's built-in "save anywhere") would
  decide every download, and each window would add one more `finished` report. Instead an
  **emission hook** on `WebKitDownload::decide-destination` (added once; it runs for every download
  in the process before any handler, whatever the context and whenever the signal comes — CI's
  WebKitGTK 2.50.4 never ran a handler connected in the context's `download-started`) takes the
  downloads whose `web_view` is the open window's page: it disconnects that download's other
  `decide-destination` handlers (wry's; smoke log `download-deciders-removed <n>`), sets the
  destination with a `file://` URI (WebKitGTK 2.50's legacy API wants one) or cancels a refused
  download, and the class handler keeps that destination (smoke log `download-decide other` for a
  download that isn't the page's). **A navigation that became a download** leaves the page where it
  was: its address goes back to the page's (`PageAddress::download`; WKWebView raises no event for
  the page that stays, WebView2 and WebKitGTK raise a `Finished` for it). **Closing the window ends
  what is under way:** the downloads it started are forgotten (`download-closed <name>`); WebKitGTK's
  are cancelled (and it removes the partial file); WebView2's go with the window's environment (the
  transfer stops, no file is left); **WKWebView's would go on** in WebKit's network process after
  the web view went (CI: still writing the file 8 s after `--hide`), so `platform/mac/downloads.rs`
  wraps `-[WKDownload setDelegate:]` once — wry gives every download its delegate that way, and keeps
  the WKDownload to itself — to note each download (`download-noted`), and on close cancels them
  (`download-cancelled <n>`) and removes the partial files of those under way. Nothing keeps a
  webview alive for a download (Ground Rule 9). **WKWebView shows an attachment it can display** (a
  PDF, an image, text) instead of downloading it: wry's response policy looks only at
  `canShowMIMEType`, not at `Content-Disposition` (CI's `dl-pdf`; Deferred follow-ups).

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
  Ctrl/Cmd+L (page keys and the macOS menu: the address is selected), Ctrl+H / Cmd+Y (in its history
  view, below), and a click on the
  toolbar's domain (`set_overlay(true)`, which goes through the same `focus_bar`). **When it
  closes:** whenever the app shows a page (`bar::navigate`: Enter in the command bar, `--show
  <url>`, Translate; the keyboard moves to the page), Esc or × with a page behind it
  (`set_overlay(false)`, which hands the keyboard to the page), and element fullscreen. Rust
  tells the bar with `window.littleJoan.setOverlay(<bool>)`, and again when the bar finishes
  loading, so it never misses one; the smoke log gets `overlay true|false`.
- **Placing the webviews:** Windows and macOS place child webviews by bounds (re-applied on
  `Resized` / `ScaleFactorChanged`) and hide / show them. Linux packs child webviews into the
  window's vertical GtkBox, where bounds don't apply. The bar sits in a frame (a GtkScrolledWindow
  with external scrolling around a GtkViewport) because a WebKitGTK view's natural height is its
  contents size, never smaller than the view: without the frame the bar kept the whole window
  after the overlay closed. In Toolbar mode the frame is packed with `expand = false` at a 40 px
  height request and the page takes the rest; in Overlay mode (`platform/linux.rs`) the frame is
  packed with `expand = fill = true` (the 40 px request stays a minimum, so the window can still
  shrink) and the page widget is hidden — with `no_show_all` set, because tao shows a window
  with `show_all()`, which would bring it back; hiding the bar (fullscreen) hides the frame with
  it and lets the page fill the window.
- **The toolbar** (`ui/toolbar/`, Little Arc's slim strip): on the left a back button, enabled
  only once the page has somewhere to go back to in this window — counted natively
  (`back_stack.rs`, pure): a trail of the web pages that finished loading, where a load of the
  page before the last one is a step back and a load of the last one again a reload; the app's
  own pages and `about:` don't count; Rust pushes `window.littleJoan.setCanGoBack(<bool>)` on
  every change and when the bar loads. In the middle the site's icon (the command bar's lettered
  stand-in under the favicon) and its domain (`www.` stripped, IPs as they are, long names
  shortened in the middle; `logic.ts`, tested): a click opens the command bar with the full
  address selected. On the right "open in default browser": the existing page action (the page
  goes to the default browser, the window hibernates). Commands `go_back` and
  `open_in_browser` run the page actions Back and OpenInBrowser. **The status area** (LJ15), where
  the pill sits and in its place while it shows: what Rust pushes with
  `window.littleJoan.setStatus(<json>)` — `{ kind: "download", state: downloading | saved | failed
  | blocked, name }` — as text: "Downloading report.pdf…", "report.pdf saved" with a **Show in
  folder** button (`reveal_download`), "Download failed", "Download blocked" (an extension's panel
  showed); long names shortened in the middle; `{ kind: "zoom", percent }` as "Zoom 110 %" for 1.5 s
  (LJ16). A finished download's status stays 8 s of the time the toolbar is visible: while the command bar covers it nothing counts down, so opening the command bar never
  loses one (`ui/toolbar/logic.ts`, tested); Rust tells the bar its last status again when the bar's
  script runs (`bar/status.rs`). Generic — text and an optional action — for LJ46 too. The empty parts of the toolbar
  carry `data-tauri-drag-region`, and the bar's capability has
  `core:window:allow-start-dragging`, so they drag the window (Tauri's own drag script, injected
  in every webview; the page webview's calls are denied). A double click there zooms the window
  and back: the drag script calls `internal_toggle_maximize`, which the bar's
  `core:window:allow-internal-toggle-maximize` allows (LJ37). On Windows and Linux, where the
  toolbar sits under the native title bar, it maximizes like the title bar does. It always zooms:
  macOS's "Double-click a window's title bar to: Minimize" isn't followed (the plan's Deferred
  follow-ups).
- **macOS title bar:** the page window uses `TitleBarStyle::Overlay` with a hidden title, so the
  content view extends under a transparent title bar and the bar webview starts at y = 0 beneath
  the traffic lights (LJ10.1 had used `Transparent` to keep content *below* the title bar). The
  lights are placed inside the 40 px toolbar (`joan::TRAFFIC_LIGHTS`; tauri's `window::WindowBuilder`
  has no setter for that, so the builder starts from a `WindowConfig` with
  `trafficLightPosition`), and the toolbar leaves them 78 px on its left; in the command bar a
  40 px strip above the panel holds them (it drags the window too). The bar knows it is on macOS
  from its user agent (`isMac`). Windows and Linux keep the native title bar.
- **Address:** the bar always shows the page's full address (empty, with the placeholder, on the
  start page). Rust pushes changes with `bar.eval("window.littleJoan.setUrl(<json>)")` on
  every page load (started and finished, so redirects land), every title change, after
  navigations the app starts, and from a once-a-second check while the window exists (for
  `history.pushState` / `#anchor` changes, which raise no event). The bar asks once on load
  (`page_url`) so it can't miss the first address. Changes are de-duplicated.
- **Editing:** click the toolbar's domain or press Ctrl/Cmd+L — the command bar opens with the
  full address selected. Enter opens it in the page (address or search, as before) and moves the
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
  starters); typing → a quick answer, the best history match, "Search <engine> for …", more
  matches, the built-in commands (**History**, **Settings** — for "settings", "preferences",
  "options" —, **Quit Little Joan** — for "quit", "exit" (LJ41), each when what is typed starts
  one of its words, two letters or more, or names it), the **quick commands**, extension commands
  — at most 8, then Settings & extensions. The built-in commands always find room, and so do up to
  three quick commands; history matches make way. **Quick commands** (LJ42, `ui/command/quick.ts`)
  show their state after the title, as text: with a web page open Translate page, Open in browser,
  Copy link, Copy as Markdown link (copied in the bar on the keypress, the Markdown from the
  bundled extension's own `link("markdown", …)`), Reload, and the site rows (Desktop site, Phone
  site, Automatic — "this site", the current ✓); always Theme: System / Light / Dark and Theme:
  <preset> (the current ✓; a preset keeps the user's mode), Search with <engine> (✓), Mobile sites
  — on/off, Remember history — on/off, Clear history — last hour / 24 hours / 7 days / everything
  (the first Enter turns the row into "press Enter again to remove …"; a second Enter on that row
  clears; Esc, another row or typing cancels), Start at login — on/off, Settings → Appearance /
  Extensions / Privacy / About, Reload extensions, Icon — shown / hidden / shown (the shortcut
  can't work). A row matches when every typed word (two letters or more) starts a word of its
  title or keywords; it is chosen with ↓/↑ like any row (never preselected, so Enter on typed text
  still opens it). Settings commands keep the command bar open and the rows show the new state
  (`quick_state` again); page commands close it. Rows come only from this list — nothing a page
  supplies picks a command — and are drawn with `textContent`. **Quick answers** (`ui/answers/`) are
  Little Joan's own TypeScript, asked in order — calculator (`= …` always; bare text only when it
  has a digit and an operator or a function, and isn't a date), units (`<n> <unit> in|to|as
  <unit>`), time zones (`time in <city>`, `<time> <city> to <city>`), random (`uuid`,
  `password [8–128]`, `[roll] NdM`, `flip a coin`, `random [a-b]`), Unix timestamps (10 or 13
  digits, `now unix`); a built-in switched off in Settings (`disabledBuiltIns` in
  `extensions_list`) is skipped. The answer row comes first and is selected, so Enter copies it
  (`copy.ts`: `navigator.clipboard` from the trusted bar on the keypress, else a hidden textarea
  and `execCommand`), and the row says "Copied" for a moment. No third-party code runs per
  keystroke. **Extension rows**
  (`ui/command/extensions.ts`) come from `extensions_list`, read each time the command bar opens: a
  switched-on extension's command matches when every typed word starts a word of its title, its
  keywords or the extension's name, and shows as "Title — Extension" with the extension's icon;
  page actions show only while a web page is open ("Extension, for this page"). Picking one calls
  `open_extension`. **Autofill:** when the best match's site starts with what was typed (a host prefix)
  and its score is at least the matches' average, the field completes it with that address
  (scheme and `www.` left out), the completion selected and its row selected; only right after
  typing at the end of the field, never after a deletion. **Keys:** ↑/↓ move without wrapping (↑
  from the first row goes back to the field), Enter opens the selected row — or with none,
  `open_input` on the field's text, as before —, Tab accepts the autofill, Shift+Delete removes a
  history row; the mouse selects on hover and opens on click. Picking a row goes through
  `open_input`, so the visit counts as typed. Typing asks `suggest` after 60 ms; only the latest
  answer is drawn. While the field shows the open page's address untouched (Ctrl/Cmd+L), the
  command bar shows the top sites.
- **The history view** (`ui/history/`) replaces the command bar's field and rows inside the
  overlay (the body gets the class `history`): a back button, a search field ("Search
  history…") and **Clear history…** at the top, then one row per page at its latest visit,
  grouped by local day — Today, Yesterday, then the date (with the year when it isn't this one) —
  each with the site's icon, "Title — site", the time and a × that removes the page. It opens
  with Ctrl+H (Cmd+Y on macOS; the page action History: `bar::show_history` opens the overlay, then
  `window.littleJoan.showHistory()`) or from the command bar's built-in **History** row (offered
  when what is typed starts the word, two letters or more, or contains it; built-in commands
  always find room among the 8 rows). It asks `history_list` and filters as you type (every word
  in the title or the address), drawing 200 rows at first and 200 more each time the list is
  scrolled to its end. ↑/↓ choose, Enter opens the chosen page (or the first), Shift+Delete
  removes it, Esc closes the clear choices, then returns to the command bar; opening a page
  closes the overlay like any page. **Clear history…** shows four choices (Last hour, Last 24
  hours, Last 7 days, Everything; the smallest is preselected) with Cancel and Clear, then says
  what went ("Removed 3 pages from the last hour.").
- **Text only:** rows are drawn with `textContent` (`view.ts`, `history/view.ts`), never
  `innerHTML`: titles come from web pages (and, later, extensions). **Site icons** are
  `<img src="https://<site>/favicon.ico" referrerpolicy="no-referrer">` straight from the site,
  shown once loaded over the site's first letter in a tinted circle (a stable hue per site),
  which stays when the icon fails. That is why the CSP's `img-src` allows `https:` (and only
  `img-src`); history itself is never sent anywhere.
- **Element fullscreen:** Windows hides the bar in the `ContainsFullScreenElementChanged`
  handler; Linux on WebKitGTK's `enter-fullscreen` / `leave-fullscreen` signals; macOS leaves the
  page alone meanwhile (WebKit shows the element in its own fullscreen window; the guard under
  "Fullscreen video").

## Themes

One to three colors, an intensity, a grain and a mode (`system` | `light` | `dark`) — settings'
`theme` (`settings/theme.rs`) — become every color of the toolbar, the command bar, the history
view and the settings window. **All color math is TypeScript** (`ui/lib/color.ts`,
`ui/lib/theme.ts`), and the result is plain sRGB (`#rrggbb`, `rgba()`): WebView2, WKWebView and
WebKitGTK differ in CSS color-4 support, so no stylesheet uses `oklch()` or `color-mix()`.

- **Muting a pick** (`mute`): the pick's hue is kept, its OKLCH lightness is mapped into a narrow
  band — light 0.86–0.93, dark 0.22–0.32 (lighter picks stay lighter) — and its chroma capped at
  0.12 × intensity, then fitted into sRGB by lowering chroma. Harmony comes from that
  derivation: any picks end up pale in light mode and deep in dark mode, so they blend.
- **Backgrounds** `--lj-bg-0..2`: three picks as they are; two picks and the OKLab point between
  them; one pick and two neighbours (lighter and 24° warmer, darker and 30° cooler).
- **The gradient** (`--lj-gradient`, a `background-image` list): soft radial blobs of `--lj-bg-1`
  (top right) and `--lj-bg-2` (bottom left), whose alpha falls off along a smooth curve, over a
  160° linear gradient from `--lj-bg-0` toward `--lj-bg-1` whose sRGB stops are computed in
  OKLab. **The grain** (`--lj-grain`): a 160 px SVG tile of `feTurbulence` noise, grey and
  stretched to full contrast, at an opacity of 0.14 × grain, as a `data:` URI (the CSP allows
  `data:` images).
- **Text** is near-black (light) or near-white (dark) with a trace of the theme's hue, stepped
  until it reaches **4.5:1 against every color it can sit on** — every gradient stop, each with
  the grain's darkest and lightest speck over it, and the frosted panel over all of those. Muted
  text and the accent (links, focus rings) meet the same bar; the selected row's color is dark
  enough for white text. Tests check every preset in both modes and random picks.
- **Surfaces:** `.themed` (the bar's body, the settings window's body) paints `--lj-bg-0` with
  the grain over the gradient. The command bar's card, the settings window's tab and footer are
  `--lj-panel`, a frosted, translucent tint that lets the gradient through; rows, fields and
  buttons use `--lj-field`, `--lj-row-selected` / `--lj-on-selected`, `--lj-hairline`; favicons
  sit on `--lj-tile` (light in both modes). The built-in rows' symbols are CSS masks painted in
  the muted text color.
- **Applying it** (`apply-theme.ts`): the variables are set on `<html>` as inline custom
  properties (CSSOM, which the CSP's `style-src 'self'` allows), with `color-scheme` so form
  controls match; a `system` theme re-applies when the system switches between light and dark.
  **The bar gets its theme before its first paint** (LJ38): its webview — only the bar's, never
  the page's — is built with an initialization script, `window.__littleJoanTheme = <theme>;`
  (`bar::theme_script`: the shown theme — the preview while Settings edits one, else the saved
  one —, checked again, so the script is one assignment of plain JSON). The engine runs it at
  document start, before the page's own scripts (the bar's CSP doesn't apply to it), and
  `ui/bar-boot.ts`, called first at `bar.ts`'s top level, applies it before anything awaits —
  so the first frame isn't `theme-defaults.css`'s Mist. Its first `set_theme_tokens` report says
  `early: true` (the smoke log's `theme-early`); `start_info` follows that report and its theme is
  applied again only when it differs (a preview may have changed it). Without the script's value
  the bar waits for `start_info`, as before. The settings window gets the theme from
  `get_settings`; after Save, Rust pushes it to both. While Settings' Appearance tab edits a
  theme, every change re-themes the settings window itself and goes to the page window through
  `preview_theme` (not saved); Cancel or closing Settings brings the saved theme back. Until then pages show the default theme:
  `ui/lib/theme-defaults.css` (generated, imported by `style.css`, checked by a test) holds its
  variables for the system's light and dark. `ui/index.html`, the page webview's first page, has
  no IPC and stays a plain neutral white / near-black.
- **Presets** (`presets.ts`): eleven muted palettes of Little Joan's own (Mist — the default —,
  Sand, Sage, Lagoon, Dusk, Ember, Orchid, Glacier, Meadow, Berry, Graphite); a preset sets the
  colors, intensity and grain, never the mode.

## Extensions (experimental)

Extensions are small web apps — an `extension.json` and plain HTML, CSS and JavaScript — that
open as **panels** in the page window. Bundled ones live in the repository's `extensions/<id>/`
and are embedded into the binary at build time (`build.rs` → `$OUT_DIR/bundled_extensions.rs`,
`include_bytes!`); with **Developer mode** on (`developerMode` in settings.json, off by default)
unpacked folders in `<config dir>/extensions/<id>/` load too. `little-joan --extension <id>`
opens one (its first command); while hibernating, the window is built first and the panel opens
once the bar's script runs (its first call, `start_info`, logs `bar-ready`) — on a thread of its
own, at most 5 s, then anyway (`bar-ready timeout`) — so the toolbar is there to name it (LJ38;
on macOS a panel that committed first left the toolbar an empty field). Openings in a window whose
bar already runs (the command bar, `--extension` with the window open) don't wait. Besides hello, twelve are bundled (LJ32–LJ34): Snake, 2048, Notes,
Focus timer, World clock, JSON, Text tools and Color tools (commands), and QR code, Clean link,
Wayback Machine and Copy as Markdown link (page actions) — none declares a network host or asks to
open the default browser (a Rust test checks it), and their pure logic is unit-tested in TypeScript. The registry is read when first needed and forgotten when the
window closes. The authors' guide is LJ30's `docs/extensions.md`.

```
 page window "joan"
 ┌──────────────────────────────────────────────────────────────────────────────┐
 │ joan-bar  (trusted: ui/bar.html, the app's origin, capabilities/bar.json)    │
 │   toolbar · command bar · history view — extension names / icons as TEXT    │
 │   reports the theme's colors ──set_theme_tokens──┐                           │
 ├──────────────────────────────────────────────────┼───────────────────────────┤
 │ joan-page  (NO capability at all: capability_tests.rs)                        │
 │   web pages ── http(s) ──► the web                 │                          │
 │   panels: top-level ljext://<id>.localhost/…       │  (http://ljext.<id>.     │
 │           (one origin per extension; CSP)          │   localhost on Windows)  │
 │      │ fetch /__lj/api/v1/<method> (POST, X-LJ-Api: <token>, same origin)    │
 └──────┼───────────────────────────────────────────┼───────────────────────────┘
        ▼                                            ▼
  Rust `ljext` handler (extensions/protocol.rs) ── checks (api.rs) ── methods: storage
  (extension-data/<id>.json), theme (the bar's colors), pageInfo, openUrl, clipboard, close
```

**What each layer enforces**

| Check | Where | Tests |
|---|---|---|
| The page webview has no IPC; no capability names `joan-page`, its window, or remote URLs | `capabilities/*.json` | `capability_tests.rs` |
| Extension code never runs in the bar or Settings: their navigation guards refuse extension URLs in every form; the handler refuses every webview but `joan-page` | `navigation::is_allowed_bar_navigation`, `settings_window::may_navigate`, `api::route` | `navigation::tests::the_bar_stays_on_the_app_and_never_opens_an_extension`, `settings_window::tests::settings_stays_on_the_app_and_never_opens_an_extension`, `api::tests::only_the_page_webview_may_ask` |
| One origin per extension: `ljext://<id>.localhost` (macOS, Linux), `http://ljext.<id>.localhost` (Windows — wry serves every `http://ljext.*` request from the handler; `.localhost` keeps it off the network and HSTS upgrades) | `urls.rs` | `urls::tests::*` |
| `ljext:` and every `http(s)://ljext.*` URL are never plain web: not allowed by the web rules, not recorded in history, not counted by back, not handed to the default browser, not translated | `urls::is_ext_like`, `navigation.rs`, `history.rs`, `back_stack.rs`, `page_actions.rs` | `navigation::tests::extension_urls_are_never_web_pages`, `history::tests::only_web_pages_are_recorded`, `back_stack::tests::extension_panels_are_not_pages_to_go_back_to`, `page_actions::tests::translate_target_skips_extension_panels` |
| The page webview navigates to an extension URL only when it is exactly the URL the app is opening — which carries a fresh 128-bit single-use nonce (`?lj-open=<hex>`, `extensions/nonce.rs`, OS randomness) that no document can know beforehand — once, fresh for 5 s; everything else is refused (links, `location.href`, and on macOS / Linux iframes, whose navigations the guard sees too; on Windows frames never reach the guard, and their requests are refused by the handler: the page, Referer and commit checks); a shown panel can't turn into a `blob:` / `data:` document; Reload on a panel opens it again with a new nonce (never from the old address); **opening from a web page leaves it first** — the page goes to the app's start page, and only when that committed (or after 1 s) is the panel's URL expected and loaded, so normally no web page is alive while a panel loads (CI on Windows: a racing page's `window.open` during the panel's load left the panel without its files, or showed WebView2's own 404 page). That detour is best effort, for availability — after 1 s the panel opens anyway, and another navigation can supersede it; the security guarantees rest on the nonce and the commit rule, not on it | `gate::NavGate`, `gate::page_may_navigate`, `gate::Leaving`, `session::open`, `session::page_committed`, `session::reopen` | `session::tests::a_racing_document_cant_claim_the_expected_panel`, `an_opening_waits_for_the_page_to_leave_its_web_page`, `only_the_expected_extension_url_is_admitted_once`, `an_expected_url_goes_stale`, `web_pages_can_never_open_extension_urls`, `panel_urls_carry_the_nonce`; smoke: the probe's `location.href` is refused; the race (a web page looping iframes and `window.open` at hello's panel while the app opens it) — the app's panel opens, every extension URL that loads carries the nonce, at least one racer is refused |
| What the user sees follows what **committed**: the toolbar names an extension only after the page's `Started` event — the engines' commit callback (`didCommitNavigation`, WebKitGTK `LOAD_COMMITTED`, WebView2 `ContentLoading`) — brought exactly the admitted panel's URL, nonce included (a late event of an earlier opening doesn't count, and each admission, Reload's too, starts uncommitted) (`Finished` doesn't count: it can come late, and WebKit reports a still-provisional URL); the app's extension navigations (opening, closing, `openUrl` to the page) don't set the page's address up front (`bar::navigate_pending`), so a refused or cancelled navigation leaves the real page named | `session::NavGate::commit`, `session::publish_label`, `bar::navigate_pending`, `page_address::pending` | `session::tests::the_toolbar_follows_what_a_load_event_committed`, `page_address::tests::a_pending_navigation_keeps_the_address_until_its_events_arrive`; smoke: every `toolbar-label <id>` follows that extension's own panel load (`labelFailures`) |
| Never served for a frame, plugin, service or shared worker: `Sec-Fetch-Dest` iframe / frame / embed / object / fencedframe / serviceworker / sharedworker, `Sec-Fetch-Dest` sent twice, or any `Service-Worker` header (a service worker would serve the extension's pages without the handler's policy or checks, and outlive it). **No engine we ship on passes `Sec-Fetch-*` to a custom-scheme handler** (CI's header log: WebView2 gives `Accept`, `Referer`, `User-Agent`, `sec-ch-ua*`, `Origin` on POST; WebKit `Accept`, `User-Agent`, a `Referer` on POST) — the check stays for engines that do; what stops frames is the guard (macOS / Linux), the page, Referer and commit checks, and `X-Frame-Options` / `frame-ancestors` | `api::embedding` | `api::tests::service_and_shared_workers_get_nothing`, `files_are_get_only_never_for_frames_and_only_plain_paths_of_known_types`; smoke, end to end: hello registers its real `sw.js` from its panel and must report `sw=refused` or `sw=unsupported` on every OS, and no `200` for `/sw.js` may appear in the request log (`isHelloReport`, `serviceWorkerFailures`) |
| Files and the SDK are served only to `joan-page`, only while it shows that extension (opened by the app) — "shows" as the navigation guard last let it through (`session::NavGate::showing`: an admitted panel URL sets it, any other allowed navigation clears it), not the page's address from load events, which arrive late (CI on Windows) —, only for `GET`, only plain paths of known types | `api::route` | `api::tests::*` (one test per check); smoke: the probe's `<img>` is blocked |
| **Until the panel commits, only its document:** between the guard admitting a panel and the panel replacing the web page, that web page still runs — CI saw a racing page's iframes served hello's `index.html` in that moment on Windows. Now only the request carrying this opening's nonce (`?lj-open=<hex>`, the app's own navigation) is served then; every other file, the SDK and the API wait — the handler holds them up to 2 s (`api::COMMIT_WAIT`) — until a load event committed the panel (`NavGate::committed`), when the web page is gone. A Referer, when sent, must be a page of the extension's own origin (racers on Windows carry the web page's) | `api::route`, `api::referer_ok`, `protocol::respond`, `session::nonce` | `api::tests::before_the_panel_commits_only_its_document_is_served`, `a_referer_must_be_the_extensions_own`; smoke: `servedToOthers` (no `status=200` for a request whose Referer is another page) |
| Paths: plain names only (letters, digits, `. _ -`), no `..`, no absolute paths or drive letters, no backslashes, no percent-encoding (so no encoded traversal), no dotfiles or Windows device names, `__lj/` reserved; on disk the real path (symlinks followed) must stay in the folder | `paths.rs` | `paths::tests::*`, `registry::tests::developer_files_never_come_from_outside_their_folder`, `api::tests::dot_segments_resolve_inside_the_extension` |
| The CSP: `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; media-src 'self' data: blob:; font-src 'self' data:; connect-src 'self' <declared https hosts>; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'`, plus `nosniff`, `X-Frame-Options: DENY`, `Cross-Origin-Resource-Policy: same-origin`, `Referrer-Policy: same-origin`, a `Permissions-Policy` denying camera, microphone, geolocation, clipboard-read …, `Cache-Control: no-store`, `X-DNS-Prefetch-Control: off`, never a CORS header (the CSP doesn't cover WebRTC, DNS prefetch or preconnect: `network:` is not a confidentiality boundary — extensions.md says so) | `csp.rs` | `csp::tests::*`; smoke: hello's fetch to example.com reports `net=blocked` (a CSP violation) |
| The API: `POST`, `X-LJ-Api` exactly once and equal (constant-time) to the opening's secret token — 128 random bits made when the panel is opened and written only into the SDK served to it (`session::token`, `protocol::TOKEN_PLACEHOLDER`) — (on Chromium the custom header also forces a refused CORS preflight for a web page), the extension's own origin (`api::origin_ok`: an `Origin` sent must be exactly the extension's, once — `null` refused; Windows must send one; WebKit sends none with a same-origin POST to a custom scheme, so on macOS and Linux a missing `Origin` is accepted when no Referer from another origin comes with it — every cross-origin request carries an `Origin` there, as CI's probe showed), `Sec-Fetch-Site: same-origin` when sent, the page showing that extension's committed panel, a method on the allowlist (`storage.get/set/remove/keys`, `theme`, `pageInfo`, `openUrl`, `clipboard.writeText`, `close`), and the method's manifest permission | `api::route`, `api::origin_ok` | `api::tests::*` (incl. `the_origin_must_be_the_extensions_own`, `webkit_sends_no_origin_from_the_extension_itself`); smoke: the probe's API calls are refused and nothing reaches hello's storage |
| Storage: keys 1–128 characters, 1 MB of JSON per extension, a refused write changes nothing, one file per extension, atomic writes | `storage.rs` | `storage::tests::*` |
| Resource limits: at most 8 API calls in flight per extension, 20 `storage.set` a second, one browser `openUrl` every 2 s (429 `busy`); a body over 1 MB + 64 KB is refused by its `Content-Length` before it is parsed (wry has already read it — every engine hands the handler the whole body); `theme` answers at once (the default theme until the bar reports) | `limits.rs`, `protocol::answer_call`, `theme::current` | `limits::tests::*`, `theme::tests::the_default_theme_answers_at_once_until_the_bar_reports`, `api::tests::bodies_are_json_objects_of_limited_size` |
| `openUrl` opens only public http(s) addresses — never an extension URL, this computer or the local network (127/8, 10/8, 172.16/12, 192.168/16, 169.254/16, 0/8, `::1`, `::`, fc00::/7, fe80::/10, IPv4-mapped, `localhost`, `*.localhost`, `*.local`); to the default browser only with `openBrowser` | `api::openable`, `api::is_private_host`, `protocol::call` | `api::tests::extensions_open_web_addresses_only`, `manifest::tests::open_browser_is_a_permission_of_its_own` |
| Manifests: strict (unknown fields refused), `id` `^[a-z0-9][a-z0-9-]{1,31}$` and not a Windows device name, `apiVersion` 1, permissions from `storage`, `clipboard`, `pageInfo`, `openBrowser`, `network:<host>` (lower-case DNS names, optional `*.`, at least two labels, no IPs or ports), texts without control or format (Cf: bidi, zero-width) characters, a name that doesn't read as a web address, panels `.html` inside the folder, an icon `.svg` / `.png` | `manifest.rs` | `manifest::tests::*` (incl. `ids_are_never_windows_device_names`, `texts_refuse_invisible_and_bidi_characters`, `names_never_read_as_web_addresses`), `registry::tests::the_shipped_extensions_are_all_valid` |
| What extensions supply is drawn as text: the toolbar's name through `textContent`, next to an "Extension" badge (so a name never passes for a domain), the icon only as a checked `data:image/(svg+xml|png);base64` URI in an `<img>` | `ui/toolbar/`, `session::publish_label`, `ui/command/extensions.ts` + `view.ts`, `ui/settings/extensions.ts` | `ui/toolbar/logic.test.ts`, `ui/command/extensions.test.ts` (a title with markup renders as text; only image data: URIs) |
| Switched-off extensions can't be opened, served or called; only Settings lists them | `registry::Registry::get` / `find` | `registry::tests::switched_off_extensions_are_listed_but_never_found`, `api::tests::switched_off_extensions_are_unknown`, `ui/command/extensions.test.ts` |

**The API** (SDK: `extensions/sdk/v1.js`, a global `littleJoan`). Every call is a same-origin
`POST /__lj/api/v1/<method>` with a JSON object body (at most 1 MB + 64 KB) and answers
`{ "ok": true, "value": … }` or `{ "ok": false, "error": "<code>", "message": … }`:

| Method | Permission | Does |
|---|---|---|
| `storage.get {key}` / `storage.set {key, value}` / `storage.remove {key}` / `storage.keys` | `storage` | the extension's own store, `<config dir>/extension-data/<id>.json`, read per call, written atomically; 1 MB, keys 1–128 characters |
| `theme` | — | `{ scheme, vars }`: the `--lj-*` variables the bar painted last (`set_theme_tokens`); the default theme's light ones at once until the bar has reported |
| `pageInfo` | `pageInfo` | the URL and title the panel was opened for (page actions, LJ30), or null — checked again inside the call against the opening's extension |
| `openUrl {url, where}` | — for `page`; `openBrowser` for `browser` | a public http(s) URL (never an extension URL, this computer or the local network) in the page (`page`, the default; the panel goes away — the page's address changes when the new page's own events arrive) or the default browser (`browser`, one every 2 s) |
| `clipboard.writeText {text}` | `clipboard` | the system clipboard, natively (NSPasteboard, GTK, Win32), at most 1 MB |
| `close` | — | back to the web page the panel was opened from, or to the command bar |

**Known limits** (documented, not enforced): `network:` limits what a panel can *fetch*, not where
it can *go* — a panel can navigate the page to a web address (or `openUrl`), which the user sees.
A panel can use the web's own clipboard API (after a click), like any page. Tauri's IPC object is
visible in panels as in web pages, and every call is refused.

## Page size and phone mode

- **The page always fills the window minus the bar, in logical pixels.** `bar::apply_layout`
  turns the window's physical inner size and scale factor into logical bounds
  (`layout::for_physical_size`); Tauri and wry convert logical bounds with the display's own
  scale (WKWebView `backingScaleFactor`, WebView2 per-window DPI), so a page's `innerWidth` is
  the same on a Retina screen as at scale 1. Linux sizes the views in a GtkBox, which is
  logical by nature. Unit tests cover scale 1, 1.5 and 2; the CI smoke test resizes the window
  with `--size` and checks the page's `innerWidth × innerHeight` (also at GDK scale 2 on Linux).
- **Resizing is live, like Little Arc** (LJ39): pages reflow to the window as it is dragged — no
  scaling, no zoom to fit, no viewport emulation (the desktop engines ignore `<meta
  name="viewport">`). To keep that smooth:
  - **Only what changed.** `bar/view.rs` keeps the layout it last applied (per window; forgotten
    when WebKit's fullscreen had the page, and at another scale factor, since the engines turn
    logical bounds into pixels at the scale of the moment) and `layout::changes` gives what
    differs: a resize sets only the bounds that changed — at most the two webviews' — and shows
    or hides nothing; a mode change shows or hides what it must. Before, every `Resized` set the
    bounds of both webviews and showed both (on Windows `ShowWindow` + `SetIsVisible` twice per
    event, on Linux `show_all()`). Smoke log: `bounds-set <n>` per layout. Linux's GtkBox packing
    is GTK's layout and changes only with the mode, as before.
  - **macOS: autoresizing masks per mode** (`layout::mask_for`, set by `platform/mac.rs` when a
    mode is first laid out): Toolbar — the bar follows the width with a flexible bottom margin
    (it keeps its 40 px at the top), the page follows width and height; Overlay — the bar follows
    both, and so does the hidden page. AppKit then resizes the webviews in its own layout pass
    while the window is dragged, where wry's fixed `NSViewMinYMargin` and tao's `Resized` (a frame
    late, from a run-loop observer) left gaps at the edges. The layout `Resized` still triggers
    sets the same frames AppKit already gave them (so a late or off-thread layout can never leave
    a view stale). Nothing touches the page's mask or bounds while WebKit has it in its
    fullscreen window; afterwards the layout and the masks are applied again in full.
  - **The bar webview's background is the theme's** (`bar/background.rs`): the `--lj-bg-0` the bar
    painted last (kept in `AppState` across windows, memory only; forgotten when Settings saves
    or previews another theme), set when the window is built and again with
    `Webview::set_background_color` whenever the bar paints a new theme; with nothing painted yet
    (or painted in the other scheme), white or near-black — the start page's colors. It shows
    before the bar paints and, on Windows, in the strip a fast drag uncovers before WebView2
    repaints. **The page webview keeps the engine's white:** a page without a background of its
    own is drawn over the webview's color on every engine (WebView2's `DefaultBackgroundColor`,
    WebKitGTK's background color, WKWebView with `drawsBackground` off), so a theme color there
    would turn plain pages dark on dark.
  - Never Tauri's `auto_resize`: it scales child webviews proportionally, so the 40 px bar would
    grow.
- **Phone mode** (`phone.rs`, `phone_mode.rs`, setting `mobileSites`, on by default): many sites
  (Google, YouTube) choose their mobile version by **user agent**, not by viewport width, so a
  narrow window alone still gets a cropped desktop page. Below **600 logical px** of page width
  the page webview gets a phone user agent, and the desktop one again only from **640 px**
  (hysteresis: in between, the agent the page has stays; a new window decides by 600; no tablet
  range): **iPhone Safari** on macOS and Linux (WebKit
  engines) and **Chrome on Android** on Windows (Chromium; the Chrome version is copied from
  WebView2's own agent), so the mobile pages that come back are built for the same engine.
  - Set per OS at runtime: WKWebView `customUserAgent` (`platform/mac.rs`), WebView2
    `ICoreWebView2Settings2::SetUserAgent` (`platform/win.rs`, the engine's own agent is read
    once to restore it), WebKitGTK's `user-agent` setting (`platform/linux.rs`).
  - A new window gets the right agent before any web page loads. A resize that crosses a
    threshold (600 going narrower, 640 going wider) switches the agent once the resize has been
    still for 400 ms (`Debounce`), then reloads the current web page once (its URL is kept; the
    bar keeps working) — only when the agent really changes. Nothing switches
    during fullscreen — a fullscreen video makes the window wide, and a reload would end it
    (macOS: also while WebKit's own fullscreen window shows the page).
  - Changing the setting applies at once.
  - **Per site** (`siteModes` in settings.json, `settings/site_modes.rs`; LJ36): `{ "<host>":
    "auto" | "phone" | "desktop" }` — lower-case host names without `www.` or a port, IP
    addresses as they are; an entry covers its subdomains (the lookup walks a name up to its last
    two labels, never an IP address). `phone` and `desktop` hold at any width and whatever
    `mobileSites` says; `auto` follows the width. **Built in, on the WebKit engines (macOS,
    Linux):** `youtube.com` and its subdomains get the desktop agent — its phone site's player
    waits for iOS's video fullscreen (`webkitbeginfullscreen`, an orientation lock), which a
    desktop WebKit never gives it, so its fullscreen button dropped out at once on macOS; a user
    entry (`"auto"` too) wins over it. `phone::site_mode` decides; Windows has no built-ins.
  - **When the agent follows the site:** the app's own navigations (`bar::navigate`: typed
    addresses, picked rows, `--show <url>`) set the target's agent first (`phone_mode::prepare`);
    a navigation the page starts itself gets it on its `Started` event, and when the agent really
    changes the page loads once more (never twice in a row for the same URL; the visit counts as
    a reload). Going to the desktop agent on a known phone host loads its desktop address instead
    — `m.youtube.com` → `www.youtube.com`, `<lang>.m.wikipedia.org` → `<lang>.wikipedia.org`
    (`phone::desktop_url`) —, and the app's navigation to such an address for a site on `desktop`
    goes to the desktop one directly.
  - **A site's phone address is the site** (LJ40, `phone::mode_site`): a mode is kept under the
    desktop name — `m.youtube.com` → `youtube.com`, `en.m.wikipedia.org` → `en.wikipedia.org`,
    and an `m.` / `mobile.` label in front of a name of two labels or more dropped
    (`m.facebook.com` → `facebook.com`) — and a lookup asks the page's own host first, then that
    site. Otherwise "Request desktop site" on a phone address would hold for that host alone, and
    a site that sends a desktop agent from `m.` to `www.` (and a phone agent back) would bounce.
  - **The UI** (LJ40): the toolbar's **pill** — "Phone" while the page has the phone agent, or
    "Desktop" while its site has a mode of its own (a built-in one too); nothing for a desktop page
    that follows the width (most of the time), the start page or a panel; it fits beside the site
    at 320 px (the domain shortens first). A click asks `set_site_mode` for the other version. The
    command bar's quick rows (LJ42; LJ40's built-in rows moved there) "Desktop site", "Phone
    site", "Automatic" — each "— this site", the current one marked ✓ — while a web page is open
    (typed words must start the row's words: "desktop", "phone", "mobile", "site", "request",
    "auto…"); picking one calls `set_site_mode` and closes the command bar. Settings → General's list (above). Rust pushes the
    site info to the bar (`setSiteMode`) on every address change and agent switch.
  - **Windows' phone agent with client hints** (LJ40, `platform/win.rs`, `phone::agent`): a bare
    custom agent (`ICoreWebView2Settings2::SetUserAgent`) left Chromium's user-agent client hints
    saying desktop (or nothing), so sites saw an Android agent with `Sec-CH-UA-Mobile: ?0`. The
    phone agent now goes through WebView2's own DevTools protocol, called from Rust
    (`ICoreWebView2::CallDevToolsProtocolMethod`, no script in any page; DevTools stay off for the
    user): `Emulation.setUserAgentOverride` with the agent and its metadata — platform Android,
    mobile, brands and full versions from WebView2's own version (`BrowserVersionString`) —, and
    the desktop agent clears that override (`{ "userAgent": "" }`: WebView2's own agent and
    metadata, exactly). The call waits (off the main thread, at most 2 s) for the engine, so the
    reload right after goes out with the new agent; smoke log `ua-override <phone|desktop>`. If the
    protocol refuses, `SetUserAgent` as before from then on (`ua-override fallback <reason>`).
  - **Windows skips a reload for a page that already fits:** before reloading for the phone agent
    (a resize going narrow), CDP's `Page.getLayoutMetrics` is asked; when the page's content
    (`cssContentSize` width) is no wider than the viewport, it is responsive and stays as it is
    (`ua-reload skipped fits`; `layout-metrics content=<w> viewport=<w>`). Only going narrow: a
    phone page always "fits" a wider window, and going wide must bring the desktop version back.
    macOS and Linux would need a script in the page to measure it (Ground Rule 10): they reload.

## Fullscreen video

HTML5 element fullscreen (a video's fullscreen button) must fill the screen:

| Engine | What happens | Code |
|---|---|---|
| WebView2 (Windows) | The engine only fills the webview's bounds and raises `ContainsFullScreenElementChanged`; the app makes the window fullscreen and back. The change is queued off the callback because resizing re-enters WebView2. | `platform/win.rs` + `fullscreen.rs` |
| WKWebView (macOS 12.3+) | wry enables `elementFullscreenEnabled`; WebKit moves the element into its own fullscreen window (its own Space). **The guard:** meanwhile nothing touches the page — `bar/view.rs`'s layout sets no bounds on, shows or hides the page (wry's `set_bounds` would set the WKWebView's frame inside WebKit's fullscreen window and shrink the video), phone mode doesn't switch (a reload would end it), and the command bar (Cmd+L, Cmd+Y, a click on the toolbar's site) doesn't open. "Meanwhile" is `fullscreenState` isn't "not in fullscreen" (macOS 13+), or the page's WKWebView isn't in the Little Joan window (macOS 12's only sign) — `fullscreen::page_layout_allowed`, read on the main thread where each decision is made. A skipped layout is applied once, after a 100 ms check finds the fullscreen ended (nothing runs otherwise). | `platform/mac.rs` + `fullscreen.rs` + `bar/view.rs` |
| WebKitGTK (Linux) | The default `enter-fullscreen` handler fullscreens the toplevel GTK window. | none |
| WebKitGTK older than 2.52 (Ubuntu 22.04's 2.50.4, the `.deb` there) | **Element fullscreen is switched off** (LJ14.1): this engine aborts the process (SIGABRT inside WebKitGTK) as a page enters element fullscreen — with WebKitGTK's own handling, and also when an `enter-fullscreen` handler returns TRUE to decline it, with or without fullscreening the window itself (LJ14's experiment), so no stopgap through that signal exists. With WebKitSettings' `enable-fullscreen` off the bare window survives the same click (LJ14.1's experiment), so the page webview gets it off whenever the runtime WebKitGTK (`webkit_get_major/minor_version`: the library actually loaded) is older than 2.52 — `fullscreen::needs_fullscreen_off`, smoke log `fullscreen-off <version>`: a page's request is refused, a video stays in the window, nothing crashes. 2.52 and newer (24.04, the AppImage's bundled 2.52) keep fullscreen. A report for Ubuntu is drafted in `docs/upstream/`. | `platform/linux.rs` + `fullscreen.rs` |

`FullscreenTracker` only undoes a fullscreen it caused, so a window the user made fullscreen
stays fullscreen when a video leaves it. The shortcut (or `--show`, the tray's Open) during an
element fullscreen takes the page out of it first, on every engine, then opens the command bar
(LJ19.5; Lifecycle, "Where the window shows up").

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
  on hibernate), the macOS File menu, and (translate only) the page keys; Open Page in Browser
  is also the toolbar's right-hand button.

## Smoke log

When `LITTLEJOAN_SMOKE_LOG` names a file, the app appends `<unix millis> <event>[ <detail>]`
lines: `started`, `shortcut-registered <accel>` (global-hotkey; `shortcut-registered <trigger>
portal` when the XDG GlobalShortcuts portal took it, `none` for a trigger the desktop didn't give,
LJ17), `shortcut-portal-unavailable <no-bus | no-portal | refused | cancelled | timeout |
no-shortcut | ended | failed> [<error>]` (then global-hotkey takes it), `shortcut-portal-host
registered | unregistered <error>` (the portal's host registry and Little Joan's app id),
`shortcut-portal-changed <trigger | none>` (the desktop changed the key), `shortcut-failed <reason>`, `tray-visible <bool>`
(the icon shown or hidden: at startup and on each change, LJ41),
`suggestions <rows> first=<site> typed=<characters>` (each `suggest` call:
counts and the first row's site, never the typed text), `overlay <true|false>` (the command bar opened or closed; once right after
`window-created` for the state a new window starts in), `title <page title>`, `page-loaded <url>`, `fullscreen <true|false>` (any
engine: the window's fullscreen state changed), `element-fullscreen <true|false>` (Windows: the
app changed it for a fullscreen element), `bar-loaded` (the URL bar's page finished loading),
`bar-url <address | start>` (what the bar was told to show), `closing <mode>` (a close began,
LJ19.4), `hibernated [ready | ready15]` (the window closed: destroyed, or with the mode it rests in),
`prewarmed <mode>` (the ready window was built after the launch, or when the setting switched),
`released <timer | setting | bar-gone | rest-over>` (a resting window was destroyed) and `release
<why>` (asked for), `page-built` / `page-closed` (the page webview, built after the bar and closed
with every close), `page-wait bar-timeout` (a cold summon's page didn't wait for the bar's script
past 3 s), `page-unload timeout` (macOS: its `about:blank` didn't load within 400 ms),
`bar-gone <why>` (the bar's web process ended; `… reload` on screen), `bar-process-failed kind=<n>
(restarts)` (Windows: a GPU or utility process WebView2 restarts), `bar-rest <low | normal |
plain>` (Windows: the bar's controller at rest), `theme-summon <mode> colors=<n>` (a summon from
ready handed the bar its theme), `after-close <mode>` (the setting changed), `window-created
<cold | ready>` (a summon brought the window up), 
`start-at-login <true|false>` (the login entry read back after a change),
`start-at-login-failed <reason>`, `settings-window <tab | focus>` (Settings opened on a tab, or
was focused / switched), `settings-loaded` (its page asked for the settings), `first-run` (no
settings.json at launch), `welcome` (the command
bar was handed the first run), `quit` (Quit or `--quit` is about to exit),
`session-cookies-kept <n>` / `session-cookies-restored <n>` (Windows: session cookies kept on
close, set again in the new window; counts only), `history-visit <kind> <url>` (a visit was
recorded), `history-saved <records>` (history.json written), `history-view` (Ctrl+H / Cmd+Y opened
the history view), `history-list <pages>` (the history view asked for the pages; a count only),
`theme <mode> colors=<n>` (the bar asked for its theme, or a settings change handed it a new one;
never the colors), `quick-command <kind> [<value>]` (a quick command ran, LJ42; never a URL or a
title), `menu-keys <action>=<key | none>` (macOS: the menu's page items were built from the
keymap, LJ43; one line per action), `system-shortcuts <n> keys=<k>` (macOS: what keymap_info read
of the system's own shortcuts, LJ44), `theme-early <mode>
colors=<n>` (the bar painted its initialization script's theme first), `bar-ready` (the bar's
script runs: its first call) and `bar-ready timeout` (a panel opened with a new window stopped
waiting for it), `theme-preview <mode>
colors=<n> | ended` (Settings previewed a theme; the preview ended when it closed unsaved),
`extension-opened <id>`, `extension-missing <id>`, `extension-skipped <folder> <reason>`,
`ext-api <id> <method> <ok | error code>` (an answered API call), `ljext-refused <reason>
<webview> <url>` (a refused `ljext` request), `navigation-refused <url>` (the page's guard),
`page-started <url>` (the page's `Started` load event), `toolbar-label <id | none>` (what the
toolbar was told to name), `new-window-refused <url>` (a popup that isn't a web page),
`ljext-headers <webview> <method> <url without query> status=<n> referer=<own | other | - |
twice> names=<header names> dest= site= mode= origin= service-worker=` (every `ljext` request:
its answer and what the engine sent — never the token, nor the Referer's value, which can carry a
nonce), `page-bounds <x,y,w,h>` and `page-visible <bool>` (every layout of the page webview:
logical bounds in the window), `page-screen <x>,<y>,<w>,<h> scale=<s>` (where the page is on the
screen, physical pixels, for CI's clicks), and on macOS `wk-fullscreen <not|entering|in|exiting>`
(WKWebView's `fullscreenState`, polled every 100 ms while a page window exists — smoke mode only),
`wk-window <resized|focused|moved> <detail>` (the window's events while that isn't `not`), and
`layout-deferred` / `layout-resumed` (macOS: a layout waited for WebKit's fullscreen to end),
`user-agent <phone|desktop>` (phone mode switched the page's agent), `bounds-set <n>` (every
layout: how many webviews' bounds it set, LJ39), `autoresizing <webview> <mask> flipped=<bool>
follows=<bool>` (macOS: a webview's autoresizing mask was set), `bar-background <light|dark>`
(the bar's webview background followed a newly painted theme), `site-mode <site> <mode>`
(`set_site_mode` kept a mode, LJ40), `ua-override <phone|desktop | fallback <reason> | no answer>`
(Windows: the agent through the DevTools protocol), `layout-metrics content=<w> viewport=<w>` and
`ua-reload skipped fits` (Windows: a page that fits wasn't reloaded for the phone agent),
`download-folder <dir>` / `download-started <name>` / `download-finished <name> ok|failed` /
`download-refused <panel|scheme|folder|name>` / `download-closed <name>` (a download under way
when the window closed) / `download-deciders-removed <n>` (Linux: wry's handlers taken off a
download, LJ15) / `download-decide other|no-url` (Linux: a download that isn't the page's) /
`download-noted` / `download-cancelled <n>` (macOS: a WKDownload noted; cancelled on close),
`zoom <percent>` (a zoom key set the page's level, LJ16), `fullscreen-off <version>`
(Linux: element fullscreen switched off for a new page window, the runtime WebKitGTK being older than 2.52, LJ14.1),
`window-spaces <label> behavior=<names> was=<names> level=<n>` (macOS: a window's collection
behaviour was set before it was first shown, and what it had before; LJ19.2), `summon-fullscreen
leaving | forcing | left <ms> | gave-up <ms> | window-stays-fullscreen <ms> | activated` (a summon
found the page in element fullscreen and asked it to leave; macOS asked WebKit itself; the page
was back in the window after `<ms>`, or the summon gave up; Windows / Linux: the window kept its
own fullscreen; macOS: it then only activated the app; LJ19.5, LJ19.4-fix4), `window-front
<label> focused=<bool> top=<bool>[ space=<bool>]` or `window-front <label> unknown` (about a second
after a summon brought a window to the front: the key window of the active app / the foreground
window / GTK's active window; nothing covering its centre, or the top of the window manager's
stack; macOS: on the active Space), `window-present <label> x11 server-time | current-time`
(Linux: the summon's raise-and-focus and its timestamp), `wk-app active=<bool> menubar=<bool>
key=<bool> space=<bool> front=<own|other>` (macOS, while a page window exists, on each change: the
app active, the menu bar shown, the Little Joan window key and on the active Space, the frontmost
app — around WebKit's element fullscreen; with `LITTLEJOAN_SMOKE_SPACES_BITS=<n>` in a smoke run
the windows get those collection-behaviour bits instead, for CI experiments only), `wk-note <name>
<object's class> [key=<bool> visible=<bool> space=<bool> behavior=<n> level=<n> active=<bool>]
[app=<pid|own>] [stack=<module:symbol < …>]` (macOS: AppKit's window did become / resign key, will /
did enter / exit full screen, the app did become / resign active or hide, NSWorkspace's active
Space did change and app did activate — a window's state with it, the key, exit and resign ones
with the call stack that posted them; LJ19.4-fix4), `wk-context <page-adding|page-added|shown>
<window's state>` (macOS: the page window just before and after the page's webview is added, and
once a summon's show and raise ran on the main thread; LJ19.4-fix4), and next to the logs
`unified.log` (macOS: the unified log of the whole smoke run — the Dock and Stage Manager without
their per-frame chatter, WebKit's Fullscreen category, AppKit's full-screen lines) and, with
`LJ_DOCK_PROBE=1` in a run's `extra_env` (CI experiments only), `dock-probe.jsonl` (before each
launch a Swift probe's window enters and leaves native full screen: whether the Dock gave it a
Space),
`summon-timing {"source":…,"kind":"cold|ready","mode":"ready|ready15|hibernate","url":…,"at":<unix ms>,"nap":…,"partial":…,"marks":{…}}`
(LJ19.1, LJ19.4: one line per summon — `cold` built the window, `ready` showed the ready one —,
each mark in ms after the request on the monotonic clock — the window code's `dispatched`,
`windowBuilt` and `webviewsAdded` (cold only: the bar's webview), `shown`, `created`;
`historyLoaded` (its own thread); the page builder's `pageBuilt`, `cookiesRestored`, `agentSet`;
the bar's `barNavigation`, `barHtml`, `barLoaded`, `barModule`, `barScript`, `barThemeAnswered`,
`barReady`, `barStartInfo` (cold only), `barPainted` (a ready summon: its first frame after the
summon), `barFocused` (its field had the keyboard), `barSuggestions`, then `commandBar`, or with
a URL `pageStarted` — the bar's own reported once per summon through `report_timing`; `nap` macOS's App Nap state at the request
(`active=<0|1> lowpri= timer= disk=`, the task's suppression policy), else `null`. The line
comes once every mark the summon waits for is in, in whatever order (on Windows the bar loads,
runs and paints inside `add_child`, before the window code's later marks); `partial` when one is
still missing as the window closes, or 20 s after the request; summon_timing.rs). Without the
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
- **settings** — `{version, shortcut, searchEngine, translateTo, windowWidth, windowHeight,
  mobileSites, rememberHistory, theme, developerMode, disabledExtensions, siteModes,
  showTrayIcon, keys, afterClose}` (camelCase
  JSON; `disabledExtensions` keeps valid ids and `builtin:<name>`s, each once; `siteModes` keeps
  at most 500 valid `host: auto|phone|desktop` entries). Each field falls back to its default on its own; sizes are clamped to
  320–4096 × 300–4096; `theme` = `{mode, colors, intensity, grain}`, each part on its own too
  (bad colors dropped, 1–3 kept as `#rrggbb`, numbers clamped to 0–1); saves go through a temp
  file + rename.
- **toggle** — what a shortcut press, a command-line request or Little Joan itself does (LJ19.4
  added the ready state, `Prewarm` and `Release`; LJ19.5 the page in fullscreen):

  | State                  | Toggle          | Show(None)      | Show(url)        | Hide    | Prewarm | Release |
  |------------------------|-----------------|-----------------|------------------|---------|---------|---------|
  | Hibernating            | Create          | Create          | Create(url)      | Nothing | Prewarm | Nothing |
  | Ready, bar alive       | Summon          | Summon          | Summon(url)      | Nothing | Nothing | Release |
  | Ready, bar gone        | Rebuild         | Rebuild         | Rebuild(url)     | Nothing | Nothing | Release |
  | Open, page full screen | LeaveFullscreen | LeaveFullscreen | FocusAndNavigate | Close   | Nothing | Nothing |
  | Open, focused          | Close           | Focus           | FocusAndNavigate | Close   | Nothing | Nothing |
  | Open, not focused      | Focus           | Focus           | FocusAndNavigate | Close   | Nothing | Nothing |

  `Close` leaves the window ready or destroys it as "After closing" says. "Page full screen" is
  the page showing an element fullscreen, focused or not: `LeaveFullscreen` takes the page out of
  it, then focuses the window with the command bar open; every focus leaves it first
  (joan/fullscreen.rs), so a URL loads after it.
- **after_close** — `afterClose`: `ready` (default) | `ready15` | `hibernate`, anything else the
  default; whether a close keeps the window and the bar, and for how long.

- **spaces** — macOS's collection behaviour for Little Joan's windows: `summoned(current)` sets
  `MoveToActiveSpace` and `FullScreenAuxiliary`, clears the other members of their exclusive
  groups (`CanJoinAllSpaces`; fullScreenPrimary / None) and keeps every other bit; idempotent.
- **cli** — `--toggle`, `--show [http(s) url]`, `--hide`, `--size WxH` (1–10000 each; resizes
  the open window); the last flag wins; unknown arguments are ignored; `--show` accepts only
  absolute web URLs. `--start-at-login on|off` (any case; other values are ignored) is a
  separate result next to the window command, so it never maps to one; a second launch whose
  only flag is `--start-at-login` leaves the window alone. `--settings [tab]` (general,
  keyboard, appearance, extensions, privacy, about, any case; General without one or for a name
  it doesn't know) opens Settings and leaves the page window alone too. `--quit` is a third result; it
  replaces the window command (a quitting launch never opens the window, nor Settings).

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
| Linux (`linux`) | ubuntu-22.04 | `tauri build --bundles deb` | `.deb` — depends on the system's WebKitGTK and the GStreamer plugins for video; built on 22.04, its binary needs glibc 2.34, so it installs on Ubuntu 22.04+ and Debian 12+. Two desktop files: Tauri's `Little Joan.desktop` (named after the product, for the app list) and, from `bundle.linux.deb.files`, `io.github.memoliertorerr.littlejoan.desktop` (`NoDisplay=true`), which the XDG portals' host registry needs to accept the app id (LJ17) |
| Linux (`linux-appimage`) | ubuntu-24.04 | `tauri build --bundles appimage` | `.AppImage` — bundles the WebKitGTK of the machine that built it (24.04's 2.52: 22.04's 2.50 aborts on element fullscreen) and the media framework; glibc floor 2.39 (Ubuntu 24.04+, Debian 13, Fedora 40+) |
| Linux ARM64 (`linux-arm64`) | ubuntu-22.04-arm | `tauri build --bundles deb` | `arm64` `.deb` (`Little Joan_<v>_arm64.deb`, ~3 MB) — the `linux` package built natively on GitHub's arm64 runner: the system's WebKitGTK, glibc floor 2.34 (Ubuntu 22.04+, Debian 12+), both desktop files (LJ18.1) |
| Linux ARM64 (`linux-appimage-arm64`) | ubuntu-24.04-arm | `tauri build --bundles appimage` | `aarch64` `.AppImage` (`Little Joan_<v>_aarch64.AppImage`, ~149 MB) — the `linux-appimage` package built natively on arm64 (Tauri doesn't cross-build AppImages): bundles 24.04's arm64 WebKitGTK 2.52 and media framework; glibc floor 2.39 (LJ18.1) |

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
- **package** — the installers above, uploaded as artifacts `little-joan-<name>` (14 days:
  `windows`, `macos`, `linux` — the `.deb` —, `linux-appimage`, and the ARM64 builds
  `linux-arm64` and `linux-appimage-arm64`), with their sizes in the job summary; each entry has
  its own cache key (`package-<name>-…`, so one per architecture). The Linux jobs also write the
  glibc floor (the newest `GLIBC_` symbol version the `.deb`'s binary, or any file in the AppImage,
  needs) and check that the package is built for the runner's architecture (the `.deb`'s
  `Architecture`, the AppImage's app per `file`), and the AppImage jobs the WebKitGTK it bundles —
  the bundled library asked for its own version through `webkit_get_*_version`; older than 2.52
  fails the job — and how it starts (its `AppRun`). The `.deb` and AppImage checks run for every
  build of their kind (keyed on `bundles`). A runner image without rustup (the arm64 runners are
  partner images) gets it in `.cache/` from `scripts/setup-rust.sh`.
- **smoke** — installs each installer the way a user would (NSIS `/S`; copy the `.app` out of
  the mounted `.dmg`; `apt install ./….deb`; the AppImage made executable) and runs `scripts/ci/smoke.mts` against the
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
  on Linux, SendKeys on Windows, key events on macOS, best effort — finds it first; then
  Ctrl+H (Cmd+Y on macOS, best effort) must log `history-view` and a `history-list` with at
  least one page, and on Windows and Linux, with the view closed again (Esc), Ctrl+Y must open
  nothing within 2 s; `smoke-command.mts`, screenshots `command-bar.png`, `command-typing.png`,
  `history.png`), that the page saw a **390×560 viewport with a phone user agent and a 720×560 one
  with the desktop agent** (it reports `innerWidth`, `innerHeight`, the agent kind and
  `devicePixelRatio` in its title), that **Ctrl+= then Ctrl+0** (xdotool `ctrl+equal`, SendKeys
  `^=`; macOS a key event, required only once its `zoom` line shows it arrived) zoom the 720 px page
  to 110 % — a narrower viewport, about 655 px; `zoom.png` shows "Zoom 110 %" in the toolbar — and
  back to 720 (`smoke-zoom.mts`), and that none of the webview helper processes that served the page outlive the
  window (followed by process id, so other apps' WebKit helpers on the runner don't count).
  Screenshots (`hibernating.png`, `start.png`, `page.png`, `fullscreen.png`, `narrow.png`,
  `wide.png`, `command-bar.png`, `command-typing.png`, `history.png`, `settings-general.png`,
  `settings-privacy.png`, `settings-appearance.png`, `welcome.png`, and the theme runs'
  `theme/*.png`), the logs, and
  The app runs on a fresh `LITTLEJOAN_DATA_DIR` that already holds the default `settings.json`,
  so it starts at rest — since LJ19.4 Stay ready: the run waits for `prewarmed`, and measures
  memory at rest with the ready bar before any window opens; the page's helpers are the ones that
  came after the bar's, and none may outlive the close. Then the
  **login-persistence check**: the smoke page's `?persist=set&t=<token>` sets a persistent
  cookie (`Max-Age` a year), a session cookie and a localStorage value; `--hide` and `--show
  ?persist=check` must report `LJ-PERSIST p=1 s=1 l=1`; new values, `--quit` within 1 s of
  them (from the log's clock), and a relaunch on the same data folder (log
  `smoke-after-quit.log`) must report `p=1 s=0 l=1` — session cookies end with the app run. The
  smoke server keeps its port across the relaunch, because localStorage is per origin. Then the
  **history check**: the main run's `history.json` must hold the smoke page's address with its
  `LJ-SMOKE …` title, its host and a visit, and none of the app's own pages; a run whose
  settings say `"rememberHistory": false` loads the smoke page, hibernates and quits, and must
  leave no records (log `smoke-history-off.log`); before it quits it runs the **settings
  check**: `--settings` must open Settings on General and its page load (`settings-window
  general`, `settings-loaded`; screenshot `settings-general.png`), `--settings privacy` must
  switch it to Privacy (`settings-privacy.png`, where Remember history is off), `--settings
  appearance` to the theme editor (`settings-appearance.png`), and `--settings extensions` to the
  extensions (after `--settings keyboard` switches it to Keyboard, `settings-keyboard.png`, LJ44;
  this run has Developer mode on, with a copy of hello as `dev-hello` and a broken
  manifest in its extensions folder: `extensions-list 2 skipped=1`, `settings-extensions.png`), and
  the page window must not open. Then the **extension run** (`smoke-extensions.mts`), its own launch: `--extension hello`
  must log `bar-ready` before hello's panel commits (from hibernation the opening waits for the
  bar, LJ38), load hello's panel and report `LJ-EXT hello storage=ok theme=ok ipc=denied net=blocked
  sw=refused` (or `sw=unsupported`: hello tries to register its `sw.js` as a service worker, and
  the request log must never show that script served) (screenshot `extension-hello.png`), with its storage file counting the visit; a new window's
  command bar, typing `hello`, must offer "Say hello" (`command-hello.png`), and ↓ ↓ Enter — pressed
  only once the typing showed up in the log — must open hello again (Windows, Linux; best effort on
  macOS); a third window's command bar, typing
  `= 12*7`, shows the answer 84 first (`answer.png`), and Enter copies it (the clipboard is read
  back where the runner has a tool for it, informational: `summary.json`); then the smoke page
  with `?ext=probe&base=<hello's origin>` tries hello's API (as the SDK would, and as a simple
  request), one of its files (as an image) and `location.href` to its panel, and must report
  `LJ-EXTPROBE ljext-api=blocked ljext-file=blocked ljext-nav=refused`, with no API call
  answered, no extension URL loaded, a `navigation-refused` in the log and nothing written to
  hello's storage (log `smoke-extensions.log`, `extension-probe.png`); last the **race**: the smoke
  page with `?ext=race` keeps putting hello's panel URL (no nonce) into iframes and `window.open`
  while `--extension hello` opens it — hello must report again, every extension URL that loads
  must carry the app's nonce, at least one racer must be refused, and the toolbar may name hello
  only right after hello's own panel loaded (`extension-race.png`). Then the **bundled extensions
  run** (`smoke-bundled.mts`), its own launch on a data folder whose `extension-data/` already
  holds sample content (three notes, a focus session 7½ minutes in, four cities): the window first
  (`--show`; hello's run covers opening from hibernation), then `--extension <id>` for each
  bundled panel in turn (Snake, 2048, Notes, Focus timer, World clock, JSON, Text tools, Color
  tools — the tools with sample drafts too) — each must
  come up (its URL with the app's nonce, its script's own `theme` call, the toolbar naming it, none
  of its requests refused, no failed call) and is screenshotted (`snake.png`, `2048.png`,
  `notes.png`, `focus.png`, `clock.png`, `json.png`, `text-tools.png`, `color.png`); then the
  **page actions** on the smoke page: Ctrl/Cmd+L, typing `qr`, ↓ ↓ Enter must open QR code for the
  page (it must come up like any panel and have `pageInfo` answered; required on Windows and Linux,
  best effort on macOS) → `qr.png`; the focus timer must show the session running from its
  stored end time (its title counts down, `1x:xx · Focus`); 2048 gets four arrow keys first
  (whether they reached it — its own save — goes into `summary.json`, informational) (log
  `smoke-bundled.log`). Then the **theme runs**
  (`smoke-theme.mts`), each its own launch on a
  fresh data folder whose `settings.json` carries a theme (so the main run's memory numbers
  don't move): a two-color theme with grain 0.4 in light (`theme/page.png`,
  `theme/command-bar.png`, `theme/history.png`, `theme/settings.png` — Settings on Appearance)
  and in dark (`theme/page-dark.png`, `theme/command-bar-dark.png`, `theme/settings-dark.png`),
  and the
  default theme in dark (`theme/command-bar-default-dark.png`); every `theme` event must name the
  run's mode and number of colors, and come after a `theme-early` of the same theme (the bar
  painted it before its first frame, LJ38). Then the **fullscreen run** (`smoke-fullscreen.mts`, LJ36),
  its own launch: the smoke page in `?fs=both` mode at 390×600 (phone agent) and 720×600 (desktop
  agent); at each size the page's "Element fullscreen" (`requestFullscreen` on a container, as
  desktop YouTube) and "Phone player" (m.youtube.com's way under an iPhone agent:
  `video.webkitEnterFullscreen()`, the iOS events, an orientation lock; element fullscreen of its
  container otherwise) are clicked where the app's latest `page-screen` line of the page's own width
  puts them (a late layout from the fullscreen size can log a stale one after leaving it), and each one's
  Exit (in fullscreen, at the screen's centre) 2 s later — clicks only, never keys. Each attempt
  reports `LJ-FS <element|phone> width= ua= entered= held_ms= exited_by_page= events=`
  (screenshots `fs/element-390.png`, `fs/phone-390.png`, `fs/element-720.png`, 1 s after each
  click; log `smoke-fullscreen.log`; what happened in `summary.json`'s `fullscreenRun`). A second
  launch whose settings say `"siteModes": { "127.0.0.1": "desktop" }` must report the desktop
  agent at 390 px, where the player takes element fullscreen (`fs/desktop-site-390.png`, log
  `smoke-fullscreen-site.log`). The rules: every element-fullscreen path enters, holds until the
  page's own exit and leaves when asked (required on Windows and Linux; on macOS whenever the
  click reached the page), the player's iOS route under an iPhone agent is informational, and on
  macOS no `page-bounds` / `page-visible` line may come while `wk-fullscreen` isn't `not`. Last in
  the first launch (LJ19.5), the **summon during the fullscreen**: the page (it pings the smoke
  server, `?ping=fs-fullscreen`) enters "Element fullscreen" at 720, then `--toggle` — the
  shortcut's own command — must not close the window but leave the fullscreen (`summon-fullscreen
  left`, after `wk-fullscreen not` on macOS, `element-fullscreen false` elsewhere), then open the
  command bar (`overlay true`); the page goes on (no `page-started`, pings a second later); a
  letter typed then reaches the command bar's field (`typed=1`; Windows and Linux, macOS recorded);
  a second `--toggle` closes the window (`fs/summon-command-bar.png`; `summary.json`'s
  `fullscreenRun.summon`). The main run's `--show <url>` during its page's fullscreen must leave
  the fullscreen first too (`summon-fullscreen left`, every OS). Then the
  **resize run** (`smoke-resize.mts`, LJ39), its own launch in a dark two-color theme with the
  window created 820×600 (so every width stays on the runner's screen): the smoke page, `--size
  380x600`, a burst of 12 `--size` steps from 380 to 820 px wide sent through single-instance 30 ms
  apart (`resize-820.png`), the same burst back (`resize-380.png`), then 620 px coming from phone,
  660, and 620 coming from desktop. Per step the page must end with `innerWidth` = the window's
  width and `innerHeight` = 560 and the right agent (phone at 380 and at 620 from phone, desktop at
  820, 660 and 620 from desktop), with exactly one `user-agent` switch and one reload per crossing
  (none for the two 620 steps), every `bounds-set` at most 2, and no `page-visible` line (a resize
  shows or hides nothing); what each step saw goes into `summary.json`'s `resizeRun` (log
  `smoke-resize.log`). Then the **site-mode run** (`smoke-sitemode.mts`, LJ40), its own launch: the
  smoke page at 390 px on the phone agent (`phone-pill.png`: the toolbar's "Phone"); Ctrl/Cmd+L,
  typing `request desktop`, ↓ ↓ Enter must log `site-mode 127.0.0.1 desktop`, leave
  `"127.0.0.1": "desktop"` in settings.json and load the page exactly once more, now on the desktop
  agent; `automatic` the same way must log `site-mode 127.0.0.1 auto`, remove the entry and bring
  the phone agent back with one load (typed on Windows and Linux; best effort on macOS). The smoke
  server writes every request's `User-Agent`, `Sec-CH-UA-Mobile`, `Sec-CH-UA-Platform` and
  `Sec-CH-UA` to `agent-headers.jsonl`, and the smoke page's report says `uamobile=` (what
  `navigator.userAgentData.mobile` says): on Windows the phone agent's request must carry Android,
  `?1` and `"Android"` and the page `uamobile=true`, the desktop agent's `?0`, `"Windows"` and
  `uamobile=false` — informational if the app logged `ua-override fallback`. Then, informational,
  `?fits=1` (the smoke page without the 600 px minimum width it has under a desktop agent) at 720
  and then 390 px: Windows skips the reload, macOS and Linux reload. Last, "Desktop site"
  once more and `--settings general` (`settings-site-modes.png`: the list). Log
  `smoke-site-mode.log`, `summary.json`'s `siteModeRun`. Then the **quick-commands run**
  (`smoke-quick.mts`, LJ42), its own launch on a fresh data folder: a new command bar, `mobile`,
  ↓ ↓ Enter (the search row, then "Mobile sites — on") must log `quick-command setMobileSites
  false` and leave `"mobileSites": false` in settings.json; a new command bar, `dark`, ↓ ↓ Enter
  must log `quick-command setThemeMode dark`, hand the bar `theme dark colors=1` and save the mode
  `dark` (typed on Windows and Linux; best effort on macOS); typing `theme` shows the theme rows
  (`quick-theme.png`); over the smoke page Ctrl/Cmd+L, `copy` (`quick-page.png`), ` link`, ↓ ↓
  Enter copies its address — read back from the clipboard where the runner can (informational).
  Log `smoke-quick.log`, `summary.json`'s `quickRun`. Then the **keymap run** (`smoke-keymap.mts`,
  LJ43), its own launch whose settings.json says `"keys": { "history":
  "CommandOrControl+Shift+KeyK" }`: over the smoke page Ctrl+Shift+K (SendKeys `^+k`, xdotool
  `ctrl+shift+k`) must open the history view, and the old key (Ctrl+H), pressed with the view
  closed again, must open nothing within 2 s; on macOS the log's `menu-keys history=⌘⇧K` (and
  `open-location=⌘L`) shows the menu was built from the keymap (required; the presses are best
  effort there). Log `smoke-keymap.log`, `summary.json`'s `keymapRun`. Then a short launch whose
  settings.json holds a shortcut that doesn't parse (`Hyper+Nope`, LJ44): it must log
  `shortcut-failed`, register the default instead, and `--settings keyboard` must open Keyboard
  (`settings-keyboard-problem.png`: the problem line; log `smoke-keyboard-problem.log`). On macOS
  the runner's `com.apple.symbolichotkeys` is exported into the artifacts (`symbolichotkeys.json`,
  read-only), and what Keyboard's `keymap_info` read (`system-shortcuts <n> keys=<k>`) must match
  it — every entry and its keys. Then the **downloads run** (`smoke-downloads.mts`, LJ15), its own
  launch: over the smoke page, `--show` on a smoke-server address (`dl-attachment`) that answers a
  fixed 1 KB body as `application/octet-stream` with `Content-Disposition: attachment;
  filename="lj-smoke.bin"` must log `download-started lj-smoke.bin`, then `download-finished
  lj-smoke.bin ok`, in the OS's Downloads folder (`download-folder` = `~/Downloads` on every runner),
  the file holding exactly those bytes (`download.png`: the toolbar's "lj-smoke.bin saved" with
  Show in folder); the same again lands as `lj-smoke (1).bin`; with hello's panel shown the same
  download must log `download-refused panel` and no file may appear. Then a slow download
  (`lj-slow.bin`, 1 KB every half second; `download-progress.png`: "Downloading lj-slow.bin…") with
  the window closed after 1.5 s: the server must see the engine go away before its last chunk, and
  nothing of it may be left in Downloads (`closeFailures`; `summary.json`'s
  `downloads.closeDuringDownload`). Informational: a PDF sent as an attachment (`dl-pdf`) — saved,
  or shown in the page (macOS) —, and files of the run found in the home or working folder. Every
  file is removed afterwards (log `smoke-downloads.log`).
  Then the **tray run**
  (`smoke-tray.mts`, LJ41), its own launches with `"showTrayIcon": false`: the log must have
  `tray-visible false` after `shortcut-registered` (and never `true`); a second launch without
  flags must log `window-created`; typing `quit` in its command bar, ↓ ↓ Enter (the search row,
  then "Quit Little Joan"; `command-quit.png`) must log `quit` and the process must end by itself
  (Windows, Linux; best effort on macOS); a relaunch sets a login on the smoke page
  (`?persist=set`) and opens Settings → General with the switch off (`settings-general-tray.png`),
  and 4 s after Settings loaded its **Quit Little Joan** button is clicked (LJ19.7; a smoke-only
  click, `LITTLEJOAN_SMOKE_SETTINGS_QUIT=1` — CI has no driver for Settings' controls — through the
  page's own handler: `settings-quit-click`): the app must log `settings-quit` and `quit` and end by
  itself within 15 s, and a relaunch on the same data folder (`smoke-tray-settings-after.log`) must
  be a later launch whose page still has the login (persistent cookie and localStorage, no session
  cookie; every OS); and on Linux a launch with `XDG_SESSION_TYPE=wayland` in its
  environment (still Xvfb's X11 underneath, but a Wayland session to Little Joan, where the X11
  shortcut can't fire) must log `tray-visible true` despite the setting (logs `smoke-tray.log`,
  `smoke-tray-settings.log`, `smoke-tray-wayland.log`; `summary.json`'s `trayRun`). Then the
  **summon-timing runs** (`smoke-timing.mts`, LJ19.1, LJ19.4), each its own launch — without gdb on
  Linux, which slows every thread and library the app starts — on a data folder holding a 3,000-page
  `history.json` (300 sites under `.test`, which never resolve). **Stay ready's** (`timing-ready`,
  LJ19.4): after the prewarm, the first summon, three warm ones, the real shortcut, one after 60 s
  of quiet and a `--show <url>` — every one a `ready` summon with its complete marks (the native
  ones and the page builder's in order), its rows and its field taking the keyboard under the
  `ready` ceiling. **Hibernate's** (`timing`, LJ19.1's run with `"afterClose": "hibernate"`): the
  first summon after the launch,
  three warm ones 3 s apart, one through the real global shortcut (xdotool's XTEST, SendKeys, a
  CGEvent; best effort), one after 60 s of quiet, one after the OS's file cache was emptied (macOS
  `sudo -n purge`, Linux `drop_caches`; Windows has no such command, so a 180 s gap), the same with
  `--show <url>`, the shortcut again after an eviction and 60 s, and a warm `--show <url>`. Every
  forwarded summon must log a complete `summon-timing` line with every mark, the native marks in
  order, ready (the command bar's rows; with a URL the later of the bar's frame and the page's
  start) under its ceiling — per OS, several times what LJ19.1 measured (Linux 2 / 3 / 5 s for
  warm / first / cold, macOS 8 / 10 / 15 s, Windows 4 / 5 / 6 s: loose on purpose, CI's runners
  vary) —; the shortcut summons count only when the key arrived (`sent→request` is the key's
  latency to the app). The table, with macOS's App Nap state per summon under it, goes into
  `timing.md` and the job's summary (log
  `smoke-timing.log` and `smoke-timing-ready.log`, `summary.json`'s `timingRun` per mode; the main
  run's own lines are its `mainRunTimings`). Then the **over-full-screen run** (`smoke-spaces.mts`, LJ19.2): another
  app's full-screen window first (`smoke-spaces-helpers.mts` — macOS: a Swift app in native full
  screen, its own Space; Windows: a borderless WinForms window covering the screen, not topmost;
  Linux: an X11 window with `_NET_WM_STATE_FULLSCREEN`, through Python's ctypes and libX11, under
  openbox), made active (a click on it; macOS: its own activation), then Little Joan on its own
  launch summoned over it three times — `--show`; after a letter typed into its command bar
  (Windows, Linux), `--hide` and a click back, the real global shortcut (`--show` if its key didn't
  arrive); `--hide`, `--settings`. Windows and Linux: the full-screen window was active before
  each summon, and each summoned window logs `window-front … focused=true top=true` while the
  active window belongs to the app (the helper's GetForegroundWindow; xdotool). macOS: every
  window's `window-spaces` shows moveToActiveSpace + fullScreenAuxiliary at level 0, and once the
  helper is full screen its Space is still the one on screen after each summon and the window is
  on it (`space=true`); in front and the keyboard are recorded (CI's summons are synthetic, which
  macOS 14+ may not activate an app for), as are three probes — an accessory window with AppKit's
  default behaviour, with Little Joan's, and with Little Joan's + canJoinAllApplications, ordered
  front over the full-screen window. Screenshots `over-fullscreen.png`,
  `over-fullscreen-shortcut.png`, `over-fullscreen-settings.png` (log `smoke-spaces.log`, the
  helper's `fs-helper.jsonl`; `summary.json`'s `spacesRun`). Then the **rest runs**
  (`smoke-rest.mts`, LJ19.4), one launch per "After closing" mode. Each summons the smoke page's
  `?ping=<id>` (a ping to the smoke server every 250 ms while it runs), closes it, and requires no
  ping later than 2 s after the app's `closing` line and none of the web processes that came with
  the page left 30 s after (sampled at 2, 10, 30 s); memory at rest — the resident memory of the
  app and the engine processes it started, summed (Windows: its msedgewebview2 tree, Linux: its
  WebKitGTK children, macOS: the WebKit services that weren't running before the launch), and their
  own memory, which counts shared libraries once (private bytes, Pss, the physical footprint) —
  before and after. **ready:** `prewarmed ready` after the launch and a web process for the bar at rest,
  no more web processes at rest after the close than the bar's; the real shortcut, then 150 ms
  later `ljkeys` typed without activating anything — the command bar must see `typed=6` (Windows
  and Linux; macOS's synthetic keys are informational); then the bar's web processes killed at
  rest (SIGKILL / taskkill): the engine must say so (`bar-gone`) and the next `--show` must log
  `released bar-gone`, `window-created cold` and a complete `summon-timing` line (screenshots
  `rest-ready-page.png`, `rest-ready-typed.png`, `rest-ready-rebuilt.png`). **hibernate:** no
  prewarm, no engine process of ours after the launch, and no web process after a close (LJ9's
  rule — WebKitGTK's network process and WebKit's Networking / GPU services stay after a window was
  destroyed, as they always did: they run no page). **ready15:** with
  `LITTLEJOAN_SMOKE_READY_MS=6000` (read only with the smoke log) the ready window must go by
  itself twice (`released timer`: after the launch's prewarm and after the close), the summon
  between them cold. Logs `smoke-rest-<mode>.log`; the table goes into `rest.md` and the job's
  summary, `summary.json`'s `restRuns`. Last comes the
  **first-launch check**: quit, launch on a new empty data folder (it must log `first-run`,
  `window-created`, `bar-loaded`, `welcome`; screenshot `welcome.png`), `--hide`, quit, relaunch
  on the same folder: `settings.json` must exist and no window may open within 5 s (the prewarm,
  which builds a hidden one, logs `prewarmed`, never `window-created`; logs `smoke-first-run.log`,
  `smoke-relaunch.log`).
  `summary.json` (memory — resident and the app's own: private bytes / private dirty /
  physical footprint —, helper counts sampled for 60 s and after three summon cycles, codecs,
  the login entries found)
  are uploaded as `smoke-<name>` (the job's limit is 75 minutes: GitHub's Azure apt mirror has
  crawled for 45, so every Linux job installs through `scripts/ci/apt-install.sh`, which stops a
  download that passes 8 minutes and goes on with the Ubuntu archive first). Linux runs under `xvfb-run` + `dbus-run-session`
  (single-instance needs a session bus) with openbox, and under gdb so a crash leaves a
  backtrace. A second Linux job (`linux-scale-2`) runs the
  same test at **scale factor 2** (`GDK_SCALE=2` on a 2560×1600 Xvfb screen); the Windows and
  macOS runners can't change their display scale. A third (`linux-appimage`, ubuntu-24.04) runs
  the whole test against the **AppImage**, under the release's file name: the runners have no
  FUSE, so it extracts itself and runs from there (`APPIMAGE_EXTRACT_AND_RUN=1`), every launch —
  the second instances too — sharing that folder (`NO_CLEANUP=1`, or a second instance would
  delete it as it exits while the first still runs from it). Its runtime starts the app through
  `AppRun`, a shell script, which gdb can't follow, so `smoke.mts` attaches gdb to the app once it
  runs (`smoke-linux.mts` finds it in `/proc`: the newest `little-joan` in the launch's process
  tree; the job allows ptrace beyond a process's own children). A fourth (`linux-22.04`,
  ubuntu-22.04) runs the whole test against the `.deb` on 22.04's own WebKitGTK 2.50.4, where
  element fullscreen is switched off (LJ14.1; `LJ_SMOKE_FULLSCREEN=off`): every launch that opens a
  page must log `fullscreen-off` with a version older than 2.52, the main run's click and every
  fullscreen-run path must change nothing — no `fullscreen true`, no `element-fullscreen true`,
  each path's report `entered=0` — while everything else passes as on 24.04
  (`smoke-fullscreen-off.mts`); every other job must never log `fullscreen-off`. **ARM64** (LJ18.1):
  three more jobs run the same whole test on GitHub's arm64 runners, each like its x86_64 twin —
  `linux-arm64` (the arm64 `.deb` on ubuntu-24.04-arm), `linux-22.04-arm64` (the arm64 `.deb` on
  ubuntu-22.04-arm, fullscreen off as `linux-22.04`) and `linux-appimage-arm64` (the aarch64
  AppImage on ubuntu-24.04-arm, as `linux-appimage`); there is no scale-2 arm64 job. Each Linux
  entry names the installer it takes (`installer: deb` / `appimage`), and the AppImage's release
  name and library check follow `uname -m`.
- **Smoke-only reruns** (manual `workflow_dispatch`, for diagnosing without rebuilding):
  `installers_from_run` (smoke the installers of an earlier run; skips verify and package),
  `extra_env` (e.g. `WEBKIT_DISABLE_DMABUF_RENDERER=1`), `linux_runner` (`ubuntu-24.04`; only for
  `linux` and `linux-scale-2` — the AppImage and 22.04 entries and the ARM64 ones keep their
  runners), `linux_window_manager` (`none`), `linux_plain_webkit_check`
  (before the app's smoke, wherever a `.deb` runs: the fullscreen click in a bare WebKitGTK window
  built with PyGObject — no Tauri — to tell engine crashes from ours, in four variants: WebKitGTK's
  own handling, an `enter-fullscreen` handler returning TRUE, that handler fullscreening the
  window (LJ14), and WebKitSettings' `enable-fullscreen` off (LJ14.1); each logs `BARE WEBKITGTK
  (<variant>): still running after the click` or its exit status).

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
