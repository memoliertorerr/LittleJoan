# Changelog

All notable changes to Little Joan are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/memoliertorerr/LittleJoan/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/memoliertorerr/LittleJoan/releases/tag/v0.1.0
