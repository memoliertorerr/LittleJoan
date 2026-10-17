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

> **Status:** v0.1, built and smoke-tested in CI on all three systems (Linux on Ubuntu 24.04);
> not yet released as signed installers. See the known issue under Fullscreen video. The build plan and progress tracker live in
> [docs/BUILD_PLAN.md](docs/BUILD_PLAN.md).

## Install

Installers are built by CI for every change: open the latest successful
[ci run](https://github.com/memoliertorerr/LittleJoan/actions/workflows/ci.yml), scroll to
**Artifacts**, and download the one for your system. (There are no signed releases yet.)

- **Windows 10/11** — `little-joan-windows` → run `Little Joan_0.1.0_x64-setup.exe` (about
  1.2 MB). It installs for your user only (no admin). Windows SmartScreen may warn about an
  unknown publisher: choose **More info → Run anyway**. Windows 10 without the WebView2 runtime
  gets it during setup.
- **macOS 12.3 or later** (Intel or Apple silicon) — `little-joan-macos` → open the `.dmg`
  (about 3.3 MB) and drag Little Joan to Applications. The app isn't notarized, so the first
  time, right-click it and choose **Open**, or run
  `xattr -dr com.apple.quarantine "/Applications/Little Joan.app"`. It lives in the menu bar
  (no Dock icon).
- **Linux** — `little-joan-linux`:
  - Debian/Ubuntu: `sudo apt install "./Little Joan_0.1.0_amd64.deb"` (about 2 MB; pulls in
    WebKitGTK and the GStreamer plugins video needs).
  - Elsewhere: `chmod +x "Little Joan_0.1.0_amd64.AppImage"` and run it (about 140 MB: it
    carries its own WebKitGTK and media framework).
  - The tray icon needs an AppIndicator host (on GNOME, the "AppIndicator and
    KStatusNotifierItem Support" extension).

After launch you only see the tray / menu-bar icon — that's hibernation. Little Joan doesn't
start at login on its own yet; add it to your system's startup apps if you want the shortcut to
always be ready.

## Using it

- **Summon:** press **Ctrl+Shift+Space** (Windows, Linux) or **Cmd+Shift+Space** (macOS), or
  choose **Open Little Joan** in the tray menu. Type an address or a search and press Enter.
- **The address bar** stays at the top of the window and always shows the page you're on,
  following links and redirects. Click it (or press Ctrl/Cmd+L) to go somewhere else: Enter
  opens the address or searches for the text, Esc puts the current address back. It hides
  while a video is fullscreen.
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
- **Command line:** `little-joan --toggle`, `little-joan --show [https://…]`, `little-joan --hide`
  talk to the running app. On Linux under Wayland, where global shortcuts can't be registered,
  bind `little-joan --toggle` to a key in your desktop's keyboard settings.
- **Quit:** tray menu → **Quit Little Joan** (or Cmd+Q on macOS while it's in front).

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
  own data folder, separate from your other browsers. Closing the window discards the page, not
  the cookies.
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
