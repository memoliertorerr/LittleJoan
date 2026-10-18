# Hello — the example extension

Hello is the extension to copy when you write your own (see
[docs/extensions.md](../../docs/extensions.md), "Quick start"). It says hello, counts how often
it was opened, and shows what an extension can and can't do. Little Joan's CI opens it on
Windows, macOS and Linux and reads its checks from the page title.

| File | What it is |
|---|---|
| `extension.json` | The manifest: the id (the folder's name), the name and version, the icon, `"apiVersion": 1`, the permissions it needs (`storage` only), and one command, "Say hello", whose panel is `index.html`. |
| `index.html` | The panel. It loads the SDK from Little Joan (`/__lj/sdk/v1.js`, which defines `littleJoan`), its stylesheet and its script — all from its own folder, as the Content-Security-Policy requires (no inline scripts). |
| `hello.js` | The script. It applies Little Joan's colors (`littleJoan.theme.apply()`), counts the openings in `littleJoan.storage`, then tries two things an extension can't do — call Little Joan's own commands, and fetch a host its manifest doesn't declare — and lists the results. The title, `LJ-EXT hello storage=ok theme=ok ipc=denied net=blocked`, is what CI checks. The Close button calls `littleJoan.close()`. |
| `hello.css` | The styles, using the theme's `--lj-*` variables (with fallbacks for the moment before the theme arrives). |
| `icon.svg` | The icon: original, simple geometry. Shown in the command bar, the toolbar and Settings. |
| `README.md` | This file (Little Joan serves `.md` as text; nothing links to it). |

To start your own: copy the folder under a new name into your extensions folder, change `"id"`
in `extension.json` to that name, and replace the checks in `hello.js` with your own panel.
