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

- **Links that open a new window** (`target=_blank`) open in your default browser.
- **Command line:** `little-joan --toggle`, `little-joan --show [https://…]`, `little-joan --hide`
  talk to the running app. On Linux under Wayland, where global shortcuts can't be registered,
  bind `little-joan --toggle` to a key in your desktop's keyboard settings.

## Build from source

Little Joan is built with [Tauri 2](https://tauri.app) (Rust + the operating system's own
webview) and TypeScript. See [docs/architecture.md](docs/architecture.md) for the layout and
build commands.

## License

[MIT](LICENSE)
