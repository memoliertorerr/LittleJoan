# Security policy

## Supported versions

Little Joan is young: only the latest version (currently 0.2.x, from the `main` branch) gets
fixes.

## Reporting a vulnerability

Please report security problems **privately**, not in a public issue or pull request:

1. Open the repository's **Security** tab on GitHub.
2. Choose **Report a vulnerability** (GitHub's private vulnerability reporting).
3. Describe the problem, the system and version it affects, and how to reproduce it.

The report is visible only to the maintainer. You'll get an answer as soon as possible; once a
fix is ready, the advisory can be published with credit to you if you'd like.

Things that matter most here: anything that lets a web page call into Little Joan (pages are
meant to get no IPC at all), open non-web URLs or local files, or keep running after the window
is closed — in every "After closing" mode a close discards the page; only Little Joan's own
hidden window and command bar may stay ("Stay ready"), never a web page's scripts, media,
network or process.

Links from other apps are untrusted input too (since v0.3: Little Joan can be the system's
browser for http and https links). They arrive only as `little-joan --link <url>` — exactly one
argument after the flag, never read as a flag itself, with every other flag on that command line
ignored — or as macOS's open-URL event, and only absolute http and https addresses open, through
the same rules as typed addresses: never `file:`, `javascript:`, `data:`, `about:`, an extension's
or the app's own pages. Little Joan registers itself only as a candidate and never changes the
system's default browser. Report anything that lets a link do more than open one web page — run
a flag or command, change a setting, open a file or a non-web URL — or that changes a system
default. Pages Little Joan hands on ("Open in browser", new-window links, extensions' `openUrl`
to the browser) go to the user's main browser through one place, never to Little Joan itself —
report anything that makes a page loop back, or that starts a browser through a shell or with
arguments a page chose.

Extensions (experimental) have their own model — one origin each, a strict Content-Security-Policy,
no IPC, and a small API that checks every request (see `docs/architecture.md`, "Extensions"). Also
report anything that lets a web page call an extension's API, load its files or open it; lets an
extension reach another extension's data, Little Joan's own screens or commands, files outside its
folder, or the network beyond the hosts its manifest declares.
