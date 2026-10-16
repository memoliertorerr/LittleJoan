# Little Joan

A tiny pop-up browser for Windows, Linux, and macOS that stays out of the way until you call it.

Little Joan hibernates by default: after launch there is no window and no loaded page, just an
icon in the tray or menu bar. Press a global keyboard shortcut and a small, resizable window
appears with a single field. Type an address or a search, press Enter, and the page fills the
window. Close it and Little Joan goes back to sleep.

Inspired by Little Arc in the Arc browser. Little Joan is an independent project and is not
affiliated with or endorsed by The Browser Company.

> **Status:** under construction. The build plan and progress tracker live in
> [docs/BUILD_PLAN.md](docs/BUILD_PLAN.md).

## Install

Installers are built by CI for every change: open the latest successful
[ci run](https://github.com/memoliertorerr/LittleJoan/actions/workflows/ci.yml), scroll to
**Artifacts**, and download the one for your system. (There are no signed releases yet.)

- **Windows 10/11** — `little-joan-windows` → run `Little Joan_…_x64-setup.exe`. It installs
  for your user only (no admin). Windows SmartScreen may warn about an unknown publisher: choose
  **More info → Run anyway**. Windows 10 without the WebView2 runtime gets it during setup.
- **macOS 12.3 or later** (Intel or Apple silicon) — `little-joan-macos` → open the `.dmg` and
  drag Little Joan to Applications. The app isn't notarized, so the first time, right-click it
  and choose **Open**, or run
  `xattr -dr com.apple.quarantine "/Applications/Little Joan.app"`. It lives in the menu bar
  (no Dock icon).
- **Linux** — `little-joan-linux`:
  - Debian/Ubuntu: `sudo apt install ./little-joan_*.deb` (pulls in the GStreamer plugins
    video needs).
  - Elsewhere: `chmod +x Little*.AppImage` and run it.
  - The tray icon needs an AppIndicator host (on GNOME, the "AppIndicator and
    KStatusNotifierItem Support" extension).

Little Joan doesn't start at login on its own yet; add it to your system's startup apps if you
want the shortcut to always be ready.

## Using it

- **Summon:** press **Ctrl+Shift+Space** (Windows, Linux) or **Cmd+Shift+Space** (macOS). Type an
  address or a search and press Enter.
- **Dismiss:** press the shortcut again, press Esc in the empty field, or close the window.
  The page is discarded and Little Joan goes back to sleep.
- **Keys on a page:**

  | | Windows / Linux | macOS |
  |---|---|---|
  | Close (hibernate) | Ctrl+W | Cmd+W |
  | Go somewhere else | Ctrl+L | Cmd+L |
  | Reload | Ctrl+R, F5 | Cmd+R |
  | Back / Forward | Alt+Left / Alt+Right | Cmd+[ / Cmd+] |
  | Translate the page | Ctrl+Shift+T | Cmd+Shift+T |
  | Open the page in your browser | tray menu | Cmd+Shift+O |

- **Fullscreen video:** a video's fullscreen button fills the screen; Esc brings it back. On
  Linux, video needs the GStreamer plugins (`gstreamer1.0-plugins-good`, `-plugins-bad`,
  `gstreamer1.0-libav` on Debian/Ubuntu); the `.deb` installs them.
- **Links that open a new window** (`target=_blank`) open in your default browser.
- **Command line:** `little-joan --toggle`, `little-joan --show [https://…]`, `little-joan --hide`
  talk to the running app. On Linux under Wayland, where global shortcuts can't be registered,
  bind `little-joan --toggle` to a key in your desktop's keyboard settings.

## Translate

**Translate Page** (tray menu, or Ctrl/Cmd+Shift+T) shows the current page through Google
Translate's web proxy, in the language chosen in Settings (English by default). This sends the
page's address to Google, and the translated page is served from `translate.goog`. Pages that
need you to be signed in usually can't be translated this way.

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

## Build from source

Little Joan is built with [Tauri 2](https://tauri.app) (Rust + the operating system's own
webview) and TypeScript. See [docs/architecture.md](docs/architecture.md) for the layout and
build commands.

## License

[MIT](LICENSE)
