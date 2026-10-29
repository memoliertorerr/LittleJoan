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

Extensions (experimental) have their own model — one origin each, a strict Content-Security-Policy,
no IPC, and a small API that checks every request (see `docs/architecture.md`, "Extensions"). Also
report anything that lets a web page call an extension's API, load its files or open it; lets an
extension reach another extension's data, Little Joan's own screens or commands, files outside its
folder, or the network beyond the hosts its manifest declares.
