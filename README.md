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

## Build from source

Little Joan is built with [Tauri 2](https://tauri.app) (Rust + the operating system's own
webview) and TypeScript. See [docs/architecture.md](docs/architecture.md) for the layout and
build commands.

## License

[MIT](LICENSE)
