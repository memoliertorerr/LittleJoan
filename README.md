# Little Joan

A tiny pop-up browser for Windows, Linux, and macOS that stays out of the way until you call it.

Little Joan hibernates by default: after launch there is no window and no loaded page, just an
icon in the tray or menu bar. Press a global keyboard shortcut and a small, resizable window
appears with a single field. Type an address or a search, press Enter, and the page fills the
window. Close it and Little Joan goes back to sleep — the page is thrown away, so nothing keeps
using memory in the background.

It uses the web engine your system already has (WebView2 on Windows, WebKit on macOS,
WebKitGTK on Linux), which keeps the download to a few megabytes.

Inspired by Little Arc in the Arc browser. Little Joan is an independent project and is not
affiliated with or endorsed by The Browser Company.

> **Status:** [v0.1.0](https://github.com/memoliertorerr/LittleJoan/releases/tag/v0.1.0) is the
> latest release; the
> [v0.2.0-beta.1 pre-release](https://github.com/memoliertorerr/LittleJoan/releases/tag/v0.2.0-beta.1)
> adds start at login and a first-launch welcome. Every build is smoke-tested in CI on all three
> systems (Linux on Ubuntu 24.04); none is code-signed yet. See the known issue under Fullscreen
> video. The build plan and progress tracker live in [docs/BUILD_PLAN.md](docs/BUILD_PLAN.md).

## Install

Download the installer for your system from the
**[latest release](https://github.com/memoliertorerr/LittleJoan/releases/latest)**. The builds
aren't code-signed yet, so the first launch shows a warning; the steps below get past it.
(Development builds of every change are also kept as artifacts of the
[CI workflow](https://github.com/memoliertorerr/LittleJoan/actions/workflows/ci.yml).)

- **Windows 10/11 (x64):** `LittleJoan-<version>-windows-x64-setup.exe` (about 1.3 MB). It
  installs for your user only, no admin needed. Windows SmartScreen may warn about an unknown
  publisher: choose **More info → Run anyway**. Windows 10 without the WebView2 runtime gets it
  during setup.
- **macOS 12.3 or later (Intel or Apple silicon):** `LittleJoan-<version>-macos-universal.dmg`
  (about 3.4 MB). Open it and drag Little Joan to Applications. It isn't notarized, so macOS
  blocks the first launch. Either run this once:

  ```bash
  xattr -dr com.apple.quarantine "/Applications/Little Joan.app"
  ```

  or try to open the app, then click **Open Anyway** in **System Settings → Privacy &
  Security**. (Right-click → Open no longer gets past this on macOS 15 and later.) Little Joan
  lives in the menu bar, with no Dock icon.
- **Linux (x86_64):**
  - Debian/Ubuntu: `sudo apt install ./LittleJoan-<version>-linux-amd64.deb` (about 2 MB; it
    pulls in WebKitGTK and the GStreamer plugins that video needs). Ubuntu 24.04 or later is
    recommended: on 22.04 the system WebKitGTK crashes when a page goes fullscreen.
  - Elsewhere: `chmod +x LittleJoan-<version>-linux-x86_64.AppImage` and run it (about 140 MB;
    it carries its own WebKitGTK and media framework). It currently bundles the WebKitGTK version
    with the fullscreen crash, so prefer the `.deb` where you can.
  - The tray icon needs an AppIndicator host (on GNOME, the "AppIndicator and
    KStatusNotifierItem Support" extension).
- **Check a download (optional):** each release includes `SHA256SUMS.txt`. Run
  `shasum -a 256 -c SHA256SUMS.txt --ignore-missing` in the download folder on macOS or Linux, or
  compare against `Get-FileHash <file>` on Windows.

The very first launch opens the window once with a short welcome: the shortcut, where the icon
is, and a "Start Little Joan at login" checkbox. Every launch after that only shows the tray /
menu-bar icon — that's hibernation. To have the shortcut ready after every restart, turn on
**Start Little Joan at login** (in the welcome or in Settings; it's off until you do). On macOS,
move Little Joan to Applications first: an app opened straight from the disk image or from
Downloads runs from a temporary copy, so Little Joan won't add a login item for it.

## Using it

- **Summon:** press **Ctrl+Shift+Space** (Windows, Linux) or **Cmd+Shift+Space** (macOS), or
  choose **Open Little Joan** in the tray menu. Type an address or a search and press Enter.
- **The address bar** stays at the top of the window and always shows the page you're on,
  following links and redirects. Click it (or press Ctrl/Cmd+L) to go somewhere else: Enter
  opens the address or searches for the text, Esc puts the current address back. It hides
  while a video is fullscreen.
- **Phone mode:** pages always fit the window and reflow as you resize it. When the window is
  narrower than 600 px, Little Joan also tells websites it is a phone browser (iPhone Safari on
  macOS and Linux, Chrome on Android on Windows), so Google, YouTube and others serve their
  mobile versions; widen it past 600 px and the page reloads in its desktop version once you
  let go. Switch it off in Settings ("Mobile sites in a narrow window").
- **Dismiss:** press the shortcut again, press Ctrl/Cmd+W, close the window, or press Esc in the
  empty address bar before opening anything. The page is discarded and Little Joan goes back to sleep. The window remembers its
  size.
- **Keys on a page:**

  | | Windows / Linux | macOS |
  |---|---|---|
  | Close (hibernate) | Ctrl+W | Cmd+W |
  | Edit the address | Ctrl+L | Cmd+L |
  | Reload | Ctrl+R, F5 | Cmd+R |
  | Back / Forward | Alt+Left / Alt+Right | Cmd+[ / Cmd+] |
  | Translate the page | Ctrl+Shift+T | Cmd+Shift+T |
  | Open the page in your browser | tray menu | Cmd+Shift+O |

- **Fullscreen video:** a video's fullscreen button fills the screen; Esc brings it back. On
  Linux, video needs the GStreamer plugins (`gstreamer1.0-plugins-good`, `-plugins-bad`,
  `gstreamer1.0-libav` on Debian/Ubuntu); the `.deb` installs them.
  **Known issue:** the WebKitGTK shipped with Ubuntu 22.04 at the moment (2.50.4) crashes when
  any page goes fullscreen — reproduced in testing without Little Joan. Ubuntu 24.04's
  (2.52) works. The AppImage currently bundles the 22.04 version.
- **Links that open a new window** (`target=_blank`) open in your default browser.
- **Command line:** `little-joan --toggle`, `little-joan --show [https://…]`, `little-joan --hide`,
  `little-joan --size 390x844` (resize the open window, in logical pixels),
  `little-joan --start-at-login on|off`, and `little-joan --quit` talk to the running app. On Linux under Wayland, where global shortcuts can't be registered,
  bind `little-joan --toggle` to a key in your desktop's keyboard settings.
- **Quit:** tray menu → **Quit Little Joan**, `little-joan --quit`, or Cmd+Q on macOS while
  it's in front.

## Translate

**Translate Page** (tray menu, or Ctrl/Cmd+Shift+T) shows the current page through Google
Translate's web proxy, in the language chosen in Settings (English by default). Pages that need
you to be signed in usually can't be translated this way.

## Settings

Open **Settings…** from the tray / menu-bar icon (or press Cmd+, on macOS while Little Joan is in
front) to change:

- **Shortcut** — click it, then press the new combination. It needs a modifier (Ctrl, Alt, Cmd,
  or Super; Shift alone doesn't count). If another app already owns it, Little Joan says so and
  keeps the old one.
- **Start Little Joan at login** — adds Little Joan to your system's login items: a LaunchAgent
  on macOS (`~/Library/LaunchAgents/Little Joan.plist`), the startup apps on Windows (the
  registry's Run key), `~/.config/autostart/Little Joan.desktop` on Linux. Little Joan keeps no
  copy of this setting, so if you remove it in your system's settings, it shows as off here too.
- **Search with** — Google, DuckDuckGo, Bing, or Brave.
- **Translate pages to** — the language for "Translate Page".

The default shortcut, Cmd/Ctrl+Shift+Space, can clash with app shortcuts (VS Code's parameter
hints, Word's non-breaking space); change it here if it does.

## Privacy

- Little Joan has no telemetry and no account. It only talks to the sites you open.
- What you type in the field goes to the search engine you picked when it isn't an address.
- **Translate Page** sends the page's address to Google, and the translated page is served
  through Google's `translate.goog` proxy.
- Cookies and site data (so logins survive) are kept by the system web engine in Little Joan's
  own data store, separate from your other browsers: `%LOCALAPPDATA%\io.github.memoliertorerr.littlejoan`
  on Windows, `~/.local/share/io.github.memoliertorerr.littlejoan` on Linux, WebKit's store for
  Little Joan on macOS (under `~/Library/WebKit` and `~/Library/HTTPStorages`). Closing the
  window discards the page, not the cookies; logins also survive quitting and restarting.
- Session cookies (the "until you close the browser" kind) last while Little Joan runs, across
  closing and reopening the window. On Windows, Little Joan holds them in memory meanwhile —
  never on disk — and quitting forgets them, as quitting a browser does.
- Web pages get no access to Little Joan's internals: only its own two local pages (the start
  field and Settings) can call into the app.

## Build from source

Little Joan is built with [Tauri 2](https://tauri.app) (Rust + the operating system's own
webview) and TypeScript. See [docs/architecture.md](docs/architecture.md) for the layout, the
build commands, and how CI builds and smoke-tests the installers.

## Contributing

Bug reports and ideas are welcome in [issues](https://github.com/memoliertorerr/LittleJoan/issues).
See [CONTRIBUTING.md](CONTRIBUTING.md) for how to build and check changes, and
[CHANGELOG.md](CHANGELOG.md) for what's in each version. Please report security problems
privately as described in [SECURITY.md](SECURITY.md).

## License

[MIT](LICENSE) © 2026 Mehmet Ali Ertorer
