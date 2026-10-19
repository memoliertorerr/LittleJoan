# Changelog

All notable changes to Little Joan are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- **Hide the tray / menu-bar icon** (Settings → General → "Show the icon in the menu bar" on
  macOS, "… in the system tray" on Windows and Linux). Little Joan then lives on its shortcut;
  opening it again from your apps also opens the window. The icon comes back by itself whenever
  the shortcut can't work (none could be registered, or a Linux Wayland session), so Little Joan
  is never out of reach.
- **Settings and Quit in the command bar:** type "settings" (or "preferences", "options") for
  **Settings**, "quit" (or "exit") for **Quit Little Joan**.
- **Quick commands in the command bar**, each showing its state: Translate page, Open in browser,
  Copy link, Copy as Markdown link and Reload on a page; the theme's mode and presets, the search
  engine, Mobile sites, Remember history, Start at login and the icon (✓ or on/off); Clear history
  (it asks for a second Enter); Settings → each tab; Reload extensions. Settings changed this way
  are saved at once, and an open Settings window keeps up.
- **Little Joan's own keys can be changed** — Ctrl/Cmd+L, history, translate, reload, back,
  forward, open in browser, close — in `settings.json` for now (`"keys": { "history":
  "CommandOrControl+Shift+KeyK" }`; `""` for no key). Keys that type text, two actions on one
  key and the shortcut that summons Little Joan are refused. On Windows, WebView2's own Ctrl+R,
  F5 and Alt+← / Alt+→ keep working beside your keys.
- **Settings → Keyboard:** the shortcut that summons Little Joan (moved here from General, which
  keeps a line with it and **Change…**) and every one of Little Joan's own keys, each recorded
  with a click, with **Reset** and **Reset all**. Clashes are refused; keys the system uses —
  Spotlight's ⌘Space, Windows' Win+…, screenshots, input sources — and Ctrl+Alt (AltGr) are
  warned about; on macOS the system shortcuts you turned off or moved in System Settings are taken
  into account (read only, never changed). When the saved shortcut couldn't be registered (another app has it), Keyboard and
  General say so, and which one Little Joan uses instead.

### Changed
- The command bar's rows for phone or desktop per site are now **Desktop site**, **Phone site**
  and **Automatic**, "this site", the current one marked ✓ (they were "Request desktop site",
  "Request phone site" and "Automatic for this site").
- **The AppImage is built on Ubuntu 24.04** and carries WebKitGTK 2.52, so it now needs glibc
  2.39 or later (Ubuntu 24.04+, Debian 13, Fedora 40+). On Ubuntu 22.04 and Debian 12, use the
  `.deb`, which is still built on 22.04 and uses the system's WebKitGTK.

### Fixed
- **Fullscreen video in the AppImage:** it bundled Ubuntu 22.04's WebKitGTK 2.50, which crashes
  when a page goes fullscreen, on every distribution it ran on.

### Known issues
- On Ubuntu 22.04 (the `.deb`, with the system's WebKitGTK 2.50.4), a video's fullscreen button
  still crashes Little Joan. WebKitGTK aborts even when the app declines the page's request, so
  there is no workaround in Little Joan; Ubuntu 24.04 and the AppImage are fine.

## [0.2.0-beta.3] — 2026-10-18

A third pre-release of v0.2: smoother resizing, phone or desktop per site, and fixes — YouTube's
fullscreen button on macOS among them.

### Changed
- **History is Ctrl+H on Windows and Linux** (Chrome's key), so Ctrl+Y is redo again in text
  fields; macOS keeps Cmd+Y.
- **Smoother resizing.** Pages reflow live as you drag, as in Little Arc. A resize now moves only
  what changed (before, every step of a drag also re-showed both views); on macOS the toolbar and
  the page follow the window's edge in the same frame; the toolbar's own background is your
  theme's, so a fast drag on Windows no longer shows a white strip beside a dark toolbar.
- **Phone mode switches less:** below 600 px the phone version, and back to the desktop version
  only from 640 px — a window dragged back and forth around 600 px no longer flips the page
  between the two.

### Added
- **Phone or desktop per site.** A small **Phone** / **Desktop** label in the toolbar (while a
  page shows its phone version, or its site has a mode of its own) switches the site to the other
  version and remembers it; the command bar has **Request desktop site**, **Request phone site**
  and **Automatic for this site** (type "desktop", "phone" or "site"), the current one marked;
  **Settings → General** lists the sites, with Remove (or Reset for YouTube's built-in desktop
  site on macOS and Linux). The page reloads only when its agent really changes.
- **Windows: the phone version with client hints.** The phone agent now comes with matching
  client hints (`Sec-CH-UA-Mobile: ?1`, platform Android, `navigator.userAgentData.mobile`), set
  through WebView2's own DevTools protocol from Little Joan itself — no script in the page,
  DevTools stay off — so sites no longer see an Android agent that says desktop everywhere else;
  and a page that already fits a narrower window isn't reloaded.
- **Double-click the toolbar to zoom the window** (and again to bring it back), on its empty
  parts — on Windows and Linux too, like the title bar.

### Fixed
- **YouTube's fullscreen button on macOS.** In a narrow window phone mode gave YouTube an iPhone
  agent, and its phone site's player waits for an iPhone's video fullscreen, which a Mac never
  gives it: the video dropped out of fullscreen at once, and its exit button did nothing. On
  macOS and Linux (WebKit) YouTube now always gets its desktop site — `m.youtube.com` addresses
  open as `www.youtube.com` — and `"siteModes"` in settings.json chooses the phone or desktop
  site per host (`"auto"` undoes the built-in). On macOS nothing resizes, moves or reloads the
  page while WebKit shows it fullscreen any more, nor opens the command bar over it.
- **Less flash of the default theme on summon.** The bar gets your theme before its first frame
  (only the bar: web pages never get a script from Little Joan). Part of the toolbar can still
  show the default colours for a split second while the window opens.
- **macOS: an extension opened while Little Joan was asleep** (`little-joan --extension snake`)
  could leave the toolbar empty; the panel now opens once the toolbar is ready.

### Known issues
- With a non-default theme, part of the toolbar can show the default colours for a split
  second while the window opens.
- "Request desktop site" changes nothing on sites that serve the same page to every device.
- On Ubuntu 22.04, and in the AppImage, a video's fullscreen button can crash WebKitGTK 2.50
  (unchanged since v0.1.0).

## [0.2.0-beta.2] — 2026-10-18

A second pre-release of v0.2: Little Joan now looks and works like Little Arc — a command bar
with your most visited sites, a slim toolbar, history, themes, and extensions (experimental).

### Changed
- **The window opens with a command bar.** Every summon shows the command bar over the whole
  window — the field at the top, and on the first launch the welcome under it; Enter opens the
  page and the bar shrinks to the address bar above it. Ctrl/Cmd+L (or a click on the address)
  brings the command bar back with the address selected; Esc returns to the page, or with no
  page closes the window. Esc now also works after clicking the welcome's checkbox.
- **The web view that shows pages has no access to the app at all.** The welcome moved out of
  it into the bar, so no page loaded there — not even Little Joan's own start page — can call
  into Little Joan; a test checks that no permission ever reaches it.

### Added
- **Page actions and tools,** bundled: **QR code** (the page's address as a QR code, made locally —
  byte mode, versions 1–10), **Clean link** (copies the address without tracking parameters and
  out of redirect wrappers; Little Joan's own rules, keeping the parameters that make a link work),
  **Wayback Machine** (opens the Internet Archive's snapshots of the page), **Copy as Markdown
  link** (also HTML, plain text or Org mode; the title is editable) — these four appear in the
  command bar only while a web page is open —, and the tools **JSON** (format, minify, sort keys,
  validate with the error's line and column, without changing a number or string), **Text tools**
  (counts, case, Base64 / URL encoding, line sorting) and **Color tools** (hex / RGB / HSL / OKLCH,
  WCAG contrast). None of them fetches anything from the network.
- **Notes, Focus timer and World clock,** bundled productivity extensions. **Notes:** several
  notes that save as you type, search, copy a note, delete with Undo; N for a new note, `/` to
  search. **Focus timer:** 25-minute focus sessions, 5-minute breaks and a 15-minute break after
  four, pause and resume, a soft chime when a phase ends while the panel is open; it keeps the end
  time, so reopening the panel shows the time left (it says so when a session ended while it was
  closed — there are no background alarms while Little Joan hibernates). **World clock:** your
  time plus the cities you pick, with the difference and the day there, right across daylight
  saving time; Delete removes a city, Alt+↑/↓ moves it. All three keep their data in the
  extension's own storage on this computer.
- **Snake and 2048,** bundled games: type "snake" or "2048" in the command bar (or run
  `little-joan --extension snake` / `--extension 2048`). Both are played with the keyboard (arrows
  or WASD; Snake pauses on Space; Esc closes), take the theme's colors, fit any window size down to
  phone width, and keep your best score — 2048 also keeps the game in progress — in the extension's
  own storage.
- **Quick answers in the command bar:** type `= 12*7`, `10 km in mi`, `72f to c`, `time in
  Tokyo`, `3pm london to new york`, `uuid`, `password 20`, `roll 2d6`, `flip a coin`,
  `random 1-100`, `1700000000` or `now unix`, and the answer is the first row — Enter copies it.
  The calculator is a real parser (never `eval`); time zones follow daylight saving time; random
  values come from the system's secure randomness. Each can be switched off in Settings →
  Extensions → Built-in.
- **Extensions in the command bar and Settings (experimental):** type an extension's name (or
  what it does) in the command bar to open it; page actions — extensions for the page you're on —
  show only while a web page is open. **Settings → Extensions** lists every extension with a
  switch to turn it off, its version, author and what it may do in plain words, plus **Developer
  mode** (load your own unpacked extensions from the extensions folder), **Open extensions
  folder** and **Reload extensions**, and what couldn't be loaded and why. How to write one:
  [docs/extensions.md](docs/extensions.md).
- **Extensions (experimental), the runtime:** an extension is a folder with an `extension.json`
  and plain web files that opens as a panel in the Little Joan window. Each runs on an origin of
  its own with a strict Content-Security-Policy (its own files only; network only to the https
  hosts its manifest declares), in the web view that has no access to the app, and reaches Little
  Joan only through a small checked API: storage (1 MB), the current theme's colors, the page it
  was opened for, opening a public web address (in the default browser only with the `openBrowser`
  permission), the clipboard, and closing itself; it can't install service workers. Web pages
  can't call that API, load an extension's files, or open an extension; the toolbar marks an open
  extension with an **Extension** badge. `little-joan --extension <id>` opens one; **Hello**, the
  bundled example, shows what an extension can and can't do. With Developer mode on, unpacked
  extensions in the `extensions` folder next to settings.json load too.
- **Themes:** one to three colors, an intensity and a grain become a muted gradient on the
  toolbar, the command bar, the history view and Settings, in light, dark or following the
  system (and switching with it). Picks are kept pale in light mode and deep in dark mode, so any
  colors blend, and text is chosen to stay readable (at least 4.5:1) on every part of the
  gradient. The default is a quiet blue-grey; saving a theme changes the open window at once.
  Kept in `settings.json` as `theme`.
- **Settings → Appearance, the theme editor:** System / Light / Dark, eleven presets, up to three
  color pads (a dot on a hue × color grid: drag it, or use the arrow keys; + adds a color, ×
  removes one), Intensity and Grain sliders, and a live preview of the toolbar and the command
  bar. Settings and the open Little Joan window show the theme while you edit; Save keeps it,
  Cancel or closing Settings puts the saved one back. `little-joan --settings appearance` opens
  it.
- **Settings has tabs:** General (shortcut, search engine, translate language, mobile sites,
  start at login), Appearance and Extensions (coming next), Privacy (**Remember history**,
  **Clear history**, and what is kept where), and About (version, license, the project's page),
  in a sidebar, with Save and Cancel for all of them. **Settings & extensions** in the command
  bar opens it on Extensions, and `little-joan --settings [tab]` on any tab. Links in Settings
  open in your default browser.
- **History view (Ctrl+Y, Cmd+Y on macOS):** the pages you opened, by day — Today, Yesterday,
  then dates — with their times, a search field, × to remove a page, and **Clear history…** for
  the last hour, the last 24 hours, the last 7 days, or everything. It opens over the command
  bar (also from its "History" row, when you type "history"); a click or Enter opens a page, Esc
  returns to the command bar. Removing and clearing rewrite `history.json` at once.
- **A Little Arc-style toolbar** above the page: back (enabled once the page has somewhere to
  go back to in this window), the site's icon and domain in the middle — click it to edit the
  address in the command bar —, and "open in default browser" on the right. On macOS the
  traffic lights sit inside the toolbar, and its empty parts drag the window.
- **Suggestions in the command bar,** like Little Arc's: the sites you open most when nothing is
  typed (Wikipedia, YouTube, GitHub and Google Maps until there is history), and while typing
  the best history match — with the address completed in the field when you type the start of a
  site (Tab accepts it) —, "Search <engine> for …", and more matches. Each row shows the site's
  icon and "Title — site"; ↑/↓, Enter, and Shift+Delete to forget a page. The last row,
  **Settings & extensions**, opens Settings. Site icons come straight from the sites.
- `little-joan --quit` quits the running app, the same way as **Quit Little Joan** in the tray
  menu (handy for scripts; CI uses it to quit gracefully).
- **History, kept locally:** the pages you open (address, title, when and how) are recorded in
  `history.json` next to the settings — at most 5,000 addresses, ranked by how often and how
  recently you visit them — for the command bar's suggestions to come. It never leaves your
  computer; Little Joan's own pages are never recorded, and the `rememberHistory` setting (on by
  default) turns recording off.
- CI now proves that logins survive: on every system the smoke test checks that a persistent
  cookie, a session cookie and localStorage outlive closing the window, and that the persistent
  cookie and localStorage outlive quitting and relaunching.

### Fixed
- **macOS: an occasional crash when a page was opened as the window appeared** (e.g.
  `little-joan --show <url>` while hibernating). Little Joan's own start page could still be
  loading when the page replaced it, and the web engine's complaint about the cancelled load
  ended the app; release builds now let the web view layer handle it. The page's address is also
  taken from the page's own load events instead of being asked from the engine at risky moments.
- **Windows: "until you close the browser" logins no longer end when the window closes.**
  WebView2 dropped session cookies with every window; Little Joan now keeps them in memory
  while it runs (never on disk) and sets them again in the next window.
- **macOS: Quit closes the window first.** Quitting with a page open ended the app without
  unloading the page, and the next launch's web engine could fail to load anything (and the app
  then crashed reading the page's address). Quit, `--quit` and Cmd+Q now close the window the
  normal way first (at most 1.5 s), and the page's address is no longer read before the page has
  started loading.

### Known issues
- On macOS, YouTube's own fullscreen button drops back to the normal size at once; its F key
  works (Esc leaves fullscreen).
- On Ubuntu 22.04, and in the AppImage, a video's fullscreen button can crash WebKitGTK 2.50
  (unchanged since v0.1.0).

## [0.2.0-beta.1] — 2026-10-17

A pre-release of v0.2: the first two v0.2 features, for trying out before v0.2.0.

### Added
- **Start at login**, off until you turn it on: a "Start Little Joan at login" switch in
  Settings and `little-joan --start-at-login on|off` add or remove the system's own login item
  (a LaunchAgent on macOS, the registry Run key on Windows, an XDG autostart file on Linux).
  The login item is the only record, so removing it in the system's settings shows as off in
  Little Joan. On macOS it refuses to start from a temporary copy ("Move Little Joan to
  Applications first").
- **A one-time welcome on the first launch:** when there are no settings yet, the window opens
  once on the start page with the shortcut, where the tray / menu-bar icon is (and, on Linux,
  the GNOME AppIndicator note), a "Start Little Joan at login" checkbox, and under Wayland how
  to bind `little-joan --toggle`. The default settings are saved at once, so it never shows
  again; every later launch hibernates as before.

## [0.1.0] — 2026-10-17

The first version: a tiny pop-up browser for Windows, macOS, and Linux that hibernates until a
global shortcut calls it.

### Added
- **Hibernation by default.** After launch there is only a tray / menu-bar icon: no window, no
  webview. Closing the window destroys it and its page, so the web engine's processes exit.
- **Global shortcut** (Cmd/Ctrl+Shift+Space by default) that summons and dismisses a small,
  resizable window on the monitor under the mouse; the window remembers its size.
- **Always-visible address bar** at the top of the window: it shows the current page's address
  (following links and redirects), Ctrl/Cmd+L or a click to edit, Enter to open an address or
  search, Esc to revert. It hides while a video is fullscreen.
- **Phone mode:** in a window narrower than 600 px, pages get a phone browser's user agent
  (iPhone Safari on macOS and Linux, Chrome on Android on Windows), so sites serve their mobile
  versions; crossing 600 px reloads the page with the other agent once the resize settles.
  On by default; "Mobile sites in a narrow window" in Settings turns it off.
- **Search** with Google, DuckDuckGo, Bing, or Brave; anything that isn't a web address is
  searched for, so `file:`, `javascript:` and other schemes can never be opened.
- **Fullscreen video** (a page's fullscreen button fills the screen) on all three systems.
- **Translate Page** through Google Translate's web proxy, in a language chosen in Settings.
- **Open Page in Browser**, and links that open a new window go to the default browser.
- **Page keys:** close (Ctrl/Cmd+W), edit the address (Ctrl/Cmd+L), reload, back / forward,
  translate; copy and paste on macOS.
- **Settings window:** shortcut recorder (the old shortcut is kept if the new one is taken),
  search engine, translate language.
- **Command line:** `--toggle`, `--show [url]`, `--hide`, `--size WxH`, forwarded to the
  running app (bind `--toggle` on Linux Wayland, where global shortcuts are unavailable).
- **Installers built in CI:** Windows NSIS (per-user), macOS universal DMG (ad-hoc signed),
  Linux `.deb` and AppImage — each installed and smoke-tested in CI.

### Security
- Web pages get no IPC: only Little Joan's own local pages (the address bar, the start page,
  Settings) can call into the app, each limited to the commands it needs.

### Known issues
- On Ubuntu 22.04, the system WebKitGTK (2.50.4 at the time of writing) aborts when a page
  enters fullscreen — reproduced without Little Joan. Ubuntu 24.04 (WebKitGTK 2.52) works. The
  AppImage currently bundles the 22.04 version.
- Builds are not notarized (macOS) or Authenticode-signed (Windows), so the first launch shows
  a Gatekeeper or SmartScreen warning.
- Linux global shortcuts work on X11 only; on Wayland, bind `little-joan --toggle` instead.

[Unreleased]: https://github.com/memoliertorerr/LittleJoan/compare/v0.2.0-beta.3...HEAD
[0.2.0-beta.3]: https://github.com/memoliertorerr/LittleJoan/releases/tag/v0.2.0-beta.3
[0.2.0-beta.2]: https://github.com/memoliertorerr/LittleJoan/releases/tag/v0.2.0-beta.2
[0.2.0-beta.1]: https://github.com/memoliertorerr/LittleJoan/releases/tag/v0.2.0-beta.1
[0.1.0]: https://github.com/memoliertorerr/LittleJoan/releases/tag/v0.1.0
