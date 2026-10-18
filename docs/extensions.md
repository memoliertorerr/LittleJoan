# Little Joan extensions (experimental)

> **Experimental.** The extension API is version 1 and may still change — while it is
> experimental, a new Little Joan can change or remove parts of it. Extensions declare
> `"apiVersion": 1`; anything else is refused, so an old extension never runs against an API it
> wasn't written for.

An extension is a small web app — an `extension.json` and plain HTML, CSS and JavaScript — that
opens as a **panel** in the Little Joan window: from the command bar (type its name), or with
`little-joan --extension <id>`. Little Joan ships a few (**bundled** extensions); with
**Developer mode** on, it also loads your own from a folder on your computer.

## What an extension is — and isn't

- **It is** a page of its own, with its own origin, storage and colors from Little Joan's theme. It
  can keep data (1 MB), copy text, open web addresses, see the page it was opened on (page
  actions), and fetch from the https hosts its manifest names.
- **It isn't** a browser extension: it can't read or change the web pages you open, inject
  scripts, see your history, cookies or other tabs, or run in the background. It runs only while
  its panel is open — closing the panel or the window ends it, and nothing runs while Little Joan
  hibernates.
- It has **no access to Little Joan's own commands** (IPC), and can't open other extensions or
  Little Joan's own screens.
- No build step is needed: what is in the folder is what runs.

## Quick start

1. Copy [`extensions/hello/`](../extensions/hello/) into your **extensions folder** under a new
   name, say `my-hello`:

   | | Extensions folder |
   |---|---|
   | macOS | `~/Library/Application Support/io.github.memoliertorerr.littlejoan/extensions/` |
   | Windows | `%APPDATA%\io.github.memoliertorerr.littlejoan\extensions\` |
   | Linux | `~/.config/io.github.memoliertorerr.littlejoan/extensions/` |

   (Settings → Extensions → **Open extensions folder** creates it and shows it.)
2. In `my-hello/extension.json`, set `"id": "my-hello"` — the folder's name must be the id, and an
   id that a bundled extension already has (like `hello`) is refused.
3. Settings → Extensions: turn on **Developer mode**, then **Save**.
4. Run `little-joan --extension my-hello`, or summon Little Joan and type "hello".

After editing files, reopen the panel (or close and summon the window) to see the change;
**Reload extensions** in Settings reads the folder again (new folders, changed manifests) and
lists what couldn't be loaded, and why.

## The manifest: `extension.json`

```json
{
  "id": "my-hello",
  "name": "My hello",
  "version": "1.0.0",
  "description": "Says hello.",
  "author": "You",
  "icon": "icon.svg",
  "apiVersion": 1,
  "permissions": ["storage"],
  "contributes": {
    "commands": [
      { "id": "open", "title": "Say hello", "keywords": ["greeting"], "panel": "index.html" }
    ],
    "pageActions": [
      { "id": "page", "title": "Say hello to this page", "panel": "page.html" }
    ]
  }
}
```

| Field | Required | Rules |
|---|---|---|
| `id` | yes | 2–32 characters: `a-z`, `0-9` and `-`, not starting with `-` (`^[a-z0-9][a-z0-9-]{1,31}$`); the folder's name |
| `name` | yes | 1–40 characters, shown in the command bar, the toolbar and Settings (as plain text) |
| `version` | yes | 1–32 characters: letters, digits, `.`, `-`, `+` |
| `description` | no | up to 200 characters (Settings) |
| `author` | no | up to 80 characters (Settings) |
| `icon` | no | a `.svg` or `.png` file in the folder (shown at 20–24 px; up to 64 KB) |
| `apiVersion` | yes | `1` |
| `permissions` | no | up to 16, from the list below, each once |
| `contributes.commands` | no | up to 16; each opens a panel |
| `contributes.pageActions` | no | up to 16; each opens a panel for the web page that is open |
| `$schema` | no | ignored |

Each command or page action has an `id` (1–32 characters, `a-z`, `0-9`, `-`, unique within its
list), a `title` (1–60 characters), up to 8 `keywords` (1–32 characters each) and a `panel` — an
`.html` file in the folder. Texts may not contain control characters. **Unknown fields are
refused** (a misspelled `permisions` fails loudly instead of being ignored), and so is a manifest
over 64 KB. The command bar matches what you type against the title, the keywords and the
extension's name: every typed word must start one of their words. Page actions show only while a
web page is open, marked "for this page".

## Permissions

| Permission | Lets the panel |
|---|---|
| `storage` | use `littleJoan.storage` (1 MB) |
| `clipboard` | use `littleJoan.clipboard.writeText` |
| `pageInfo` | read the address and title of the page a page action was opened on (`littleJoan.pageInfo()`) |
| `network:<host>` | fetch from `https://<host>` — e.g. `network:api.example.com`, or `network:*.example.com` for its subdomains. Lower-case DNS names only, with at least two labels after `*.` (no `*.com`), no IP addresses, ports, paths or schemes; https only |

Settings → Extensions shows them in plain words. `theme`, `openUrl` and `close` need no
permission.

## The SDK

Every panel can load the SDK from Little Joan:

```html
<script src="/__lj/sdk/v1.js"></script>
<script src="app.js" defer></script>
```

It defines a global `littleJoan` (types for editors: [`extensions/sdk/v1.d.ts`](../extensions/sdk/v1.d.ts)).
Every call returns a promise and rejects with a `littleJoan.LittleJoanError` whose `code` says
why (`permission`, `quota`, `bad-key`, `bad-url`, …) when Little Joan refuses or can't do it.

```js
// Storage (permission "storage"): keys of 1–128 characters, any JSON value, 1 MB in all.
const count = ((await littleJoan.storage.get("count")) ?? 0) + 1; // undefined when not set
await littleJoan.storage.set("count", count);
await littleJoan.storage.remove("draft"); // true if there was one
const keys = await littleJoan.storage.keys();

// Little Joan's colors: { scheme: "light" | "dark", vars: { "--lj-bg-0": "#…", … } }.
await littleJoan.theme.apply(); // sets the --lj-* variables and color-scheme on <html>
const { scheme, vars } = await littleJoan.theme.get();

// The page a page action was opened on (permission "pageInfo"), or null.
const page = await littleJoan.pageInfo(); // { url, title }

// Open a web address in the window (the panel goes away) or in the default browser.
await littleJoan.openUrl("https://example.com/", { where: "browser" });

// Copy text (permission "clipboard").
await littleJoan.clipboard.writeText("Hello!");

// Close the panel: back to the page it was opened from, or to the command bar.
await littleJoan.close();
```

The variables `theme` hands out are the ones Little Joan's own screens use: `--lj-bg-0`,
`--lj-bg-1`, `--lj-bg-2` (the backgrounds), `--lj-gradient` and `--lj-grain` (background images),
`--lj-fg`, `--lj-fg-muted` (text), `--lj-accent`, `--lj-panel`, `--lj-surface`, `--lj-field`,
`--lj-row-hover`, `--lj-row-selected`, `--lj-on-selected`, `--lj-tile`, `--lj-hairline`,
`--lj-shadow`. Text in `--lj-fg` stays readable (4.5:1) on every background. `get()` is a
snapshot: ask again to follow a theme change.

Under the hood each call is a `POST` to the panel's own origin, `/__lj/api/v1/<method>`, with a
JSON body and the header `X-LJ-Api: 1`, answered `{ "ok": true, "value": … }` or
`{ "ok": false, "error": "<code>", "message": "…" }` — the SDK does this for you.

## Panels, URLs and origins

A panel is the top-level page of the Little Joan window while it is open; the toolbar shows the
extension's name and icon instead of a domain. Each extension has an **origin of its own**:

| | A panel's URL | Its origin |
|---|---|---|
| macOS, Linux | `ljext://<id>.localhost/<path>` | `ljext://<id>.localhost` |
| Windows | `http://ljext.<id>.localhost/<path>` | `http://ljext.<id>.localhost` |

So `localStorage`, IndexedDB and cookies are per extension too — but use `littleJoan.storage`,
which is the documented, size-limited store. Only Little Joan opens a panel: a page (or a panel)
can't navigate the window to an extension's URL. A panel is one page: links to other pages of the
same extension don't open (build your UI in the page; that includes `#anchor` links and
`location.reload()` — scroll with `scrollIntoView()`, re-render instead of reloading), and a panel
can't become a `blob:` or `data:` document. Following a link to the web leaves the panel, like `openUrl`. Popups
(`target=_blank`, `window.open`) open web addresses in the default browser.

**Files** are served by plain paths only: names of letters, digits, `.`, `_` and `-` (no spaces,
no names starting with `.`), at most 8 folders deep; `__lj/` is Little Joan's. Served types:
`.html`, `.js`/`.mjs`, `.css`, `.json`, `.txt`/`.md`, `.svg`, `.png`, `.jpg`/`.jpeg`, `.gif`,
`.webp`, `.ico`, `.woff2`/`.woff`/`.ttf`/`.otf`, `.mp3`/`.ogg`/`.wav`, `.mp4`/`.webm`; anything
else is not served. A file may be up to 16 MB; symlinks that lead out of the folder are refused.

## What the Content-Security-Policy blocks

Every response carries:

```
default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';
img-src 'self' data: blob:; media-src 'self' data: blob:; font-src 'self' data:;
connect-src 'self' <https://each declared host>; frame-src 'none'; object-src 'none';
base-uri 'none'; form-action 'none'; frame-ancestors 'none'
```

- **Scripts** only from the extension's own files: no inline `<script>`, no `onclick="…"`, no
  `eval` or `new Function`, no scripts from the web. Inline styles are fine.
- **Network** (`fetch`, XHR, WebSocket, EventSource) only to the panel itself (the API) and the
  hosts from `network:` permissions; images, media and fonts only from the extension's files and
  `data:` / `blob:` URLs.
- **No frames, plugins or forms**, and no page may frame a panel.
- Also sent: `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`,
  `Cross-Origin-Resource-Policy: same-origin`, `Referrer-Policy: same-origin`,
  `Cache-Control: no-store`, and a `Permissions-Policy` that denies camera, microphone,
  geolocation, reading the clipboard, screen capture, USB, serial, HID, Bluetooth and payments.

`network:` limits what a panel can fetch, not where it can go: a panel can still take the window
to a web page (a link, `openUrl`), which you see.

## Storage limits

`littleJoan.storage` keeps an extension's items in `extension-data/<id>.json` next to Little
Joan's `settings.json`. Keys are 1–128 characters; values are any JSON; the whole store, as JSON,
is at most **1 MB** — a write that would pass that fails with `quota` and changes nothing. Writes
are atomic. Each extension sees only its own store.

## Testing

- **Keep the logic pure.** Put what can be tested (game rules, parsers, formatting) in a plain
  JavaScript module with no DOM, and test it with any test runner (bundled extensions bring their
  tests along, and CI runs them).
- **Try it in Little Joan** with Developer mode on; Settings → Extensions lists what couldn't be
  loaded and why (a manifest error names the field).
- **See what Little Joan decided:** start it with the environment variable
  `LITTLEJOAN_SMOKE_LOG=<file>` and it writes events there — `extension-opened <id>`,
  `ext-api <id> <method> ok|<error>`, `ljext-refused <reason> …` and `navigation-refused <url>`.
- `extensions/hello/` is the reference: CI opens it on Windows, macOS and Linux and checks that
  storage and the theme work, that Little Joan's IPC is refused, and that the CSP blocks a fetch it
  didn't declare.

## The security model

Extensions run in the same web view as web pages — **the page web view, which has no IPC at all**:
no capability names it, so neither a web page nor a panel can call any of Little Joan's commands (a
test reads every capability file to keep it that way). Little Joan's own screens — the toolbar, the
command bar, history, Settings — live in other web views, never load an extension URL, and show
what an extension supplies (names, titles, keywords, its icon) as plain text and a checked image.

An extension reaches Little Joan only through the `ljext` API, served by Little Joan itself on the
extension's own origin. Every request is checked from scratch:

1. it comes from the page web view (never the toolbar or Settings);
2. the extension exists and is switched on;
3. the page web view is showing a panel of **that** extension, which Little Joan opened;
4. files and the SDK: a `GET`, not for a frame, a plain path of a served type;
5. the API: a `POST` with `X-LJ-Api: 1`, sent from the extension's own origin — an `Origin`
   header that is exactly the extension's origin (on Windows it must be there; macOS and Linux
   send none with a request to the panel's own origin, while every request from another origin
   carries one), a method on the list above, and the permission that method needs.

Little Joan never sends CORS headers, so no other page can read an answer. The window navigates to
an extension URL only when Little Joan itself opens that exact URL; web pages can't, in any form
(Windows' `http://ljext.…` form is never treated as a web address). Paths can't leave the
extension's folder (no `..`, absolute paths, percent-encoding or symlinks out). The details, and
the test for each check, are in [architecture.md](architecture.md#extensions-experimental).

Report a hole privately: see [SECURITY.md](../SECURITY.md).

## Built-in quick answers

Little Joan's own **quick answers** — calculator, unit conversion, time zones, random values and
Unix timestamps — answer in the command bar's first row as you type (Enter copies the answer).
They are part of Little Joan, not extensions: an extension can't add one, because a quick answer
runs on every keystroke, and extensions run only as panels (third-party quick answers are designed
for later). Settings → Extensions lists them under **Built-in**, each with a switch; switched off,
their ids (`builtin:calc`, `builtin:units`, `builtin:time`, `builtin:random`, `builtin:timestamp`)
go into `disabledExtensions` in settings.json.

## Getting a bundled extension into Little Joan

Bundled extensions live in the repository's [`extensions/`](../extensions/) folder, one folder per
extension, and are built into Little Joan. To add one, open a pull request with:

- the folder (`extensions/<id>/`) with its manifest, plain JavaScript (no build step) and an
  original icon (simple geometry; no other product's artwork or name);
- tests for its logic (`*.test.ts` next to it — they aren't built into the app);
- the fewest permissions it needs — no `network:` unless the extension can't work without it;
- an MIT license (the repository's) for everything in it.

CI checks that every bundled extension loads (a Rust test reads all of them) and runs the whole
test suite. See [CONTRIBUTING.md](../CONTRIBUTING.md) for building and checking changes.
