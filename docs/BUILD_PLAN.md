# Little Joan — Build Plan (v0.1, v0.2, v0.3)

> **Little Joan** is a tiny pop-up browser for Windows, Linux, and macOS, inspired by Little Arc.
> It **hibernates by default**: after launch it shows no window and loads no page, and sits in
> the tray / menu bar using as little memory and CPU as it can. A **configurable global keyboard
> shortcut** brings up a small, resizable window with one URL/search field; press Enter and the
> page fills the window. Closing the window **discards the page** and returns Little Joan to
> hibernation.
>
> **In scope for v0.1:** hibernation (no webview at rest), global shortcut (configurable),
> start field (URL or search), the page in a minimal native window with a way to close it,
> HTML5 element fullscreen for video, translate the current page through Google Translate's web
> proxy, a small settings window, tray / menu-bar menu, installers built and smoke-tested in CI
> for all three OSes.
>
> **Explicitly NOT in scope** (designed, deferred — see **Later**): tabs, bookmarks, history UI,
> extensions, downloads UI, find-in-page, zoom controls, start at login, keeping the page after
> close, popups that stay inside the app (OAuth windows), code signing / notarization, auto-update,
> Wayland-native global shortcut.
>
> **v0.2 (planned 2026-10-17, steps LJ11–LJ19, phases 6–11):** start at login and a one-time
> welcome on the first launch; Linux fullscreen (the AppImage built on Ubuntu 24.04, a stopgap
> for WebKitGTK older than 2.52); downloads and page zoom; a Wayland-native global shortcut (XDG
> portal); Developer ID signing and notarization for macOS. **Not in v0.2:** Windows code signing
> (waiting on the user's SignPath Foundation application), find in page, auto-update, popups
> inside the app, keeping the page after close, tabs, bookmarks, history. **v0.2 must not
> disturb:** hibernation (no window or webview at rest), no IPC for remote pages, the URL bar,
> phone mode, fullscreen video on Windows / macOS / Ubuntu 24.04, single-instance CLI flags, and
> the page size at scale 1, 1.5 and 2 — every phase PR re-checks them.
>
> **Round 3 (planned 2026-10-18, steps LJ36–LJ57, phases 18–26):** v0.2.0 is finished by fixes
> (YouTube's fullscreen on macOS, Ctrl+H for history, the bar's first paint, double-click to zoom),
> responsive resizing (live resize like Little Arc, phone mode per site), control (hide the icon,
> quick commands with their state, every key configurable in a Keyboard tab), then round 1's phases
> 7–11 (Linux fullscreen, downloads + zoom, the Wayland shortcut, macOS signing if the owner's
> secrets exist, the v0.2.0 release). **v0.3:** Little Joan as the system's browser for links,
> sign-in popups inside the app, find in page and pinned sites, saved logins on Windows, nine more
> bundled extensions. **Not in round 3:** macOS saved passwords and passkeys (deferred by the
> owner; Later has the research), Linux saved logins, Windows code signing (waiting on SignPath),
> auto-update, tabs, keeping the page after close. **Round 3 must not disturb** the list above, nor
> the command bar, toolbar, history, themes, logins surviving close and quit, or the extension
> runtime's security checks. Checkpoints stop for the owner's own testing; security-sensitive
> phases end ready and green, not merged (Ground Rule 24).
>
> **Never use Arc's name, logo, or artwork as branding.** "Inspired by Little Arc" in the README
> is the only mention. The icon is original.
>
> **Autopilot:** one step = one commit, made by the agent only when verify is green. One
> phase = one pull request. Since round 2 the agent merges a phase PR once its CI is green (the
> owner's standing exception) — never a security-sensitive phase's, never a tag or a release
> (Ground Rule 24). The agent stops at checkpoints, at the end of each phase it was asked to run,
> and at any stop condition.

---

## Progress Tracker

> Each session flips its own row to ✅ in the same commit as the step's change. Read this table
> at the start of a session to know where to resume. `auto` steps run back to back;
> `checkpoint` steps end with a stop for a human check. *(Round 3)* A `review` step ends a
> security-sensitive phase: the PR is marked ready and green, and the agent stops without merging
> (Ground Rule 24). `⏭ skipped` marks a step whose precondition didn't hold, with the reason.

| Step | Scope | Model | Gate | Status |
|------|-------|-------|------|--------|
| LJ1 | Scaffold: env + toolchain, Tauri 2 + Vite + TS strict, icons, hibernating tray app, CI verify on 3 OSes | strong | auto | ✅ done |
| LJ2 | Pure core modules: input → URL, navigation policy, translate URL, settings model, toggle state machine, CLI args | strong | auto | ✅ done |
| LJ3 | Summon / hibernate: global shortcut, the Joan window, start page, single-instance CLI, smoke log | strong | auto | ✅ done |
| LJ4 | Page behavior: navigation guard, titles, new windows → default browser, page keys (Win / Linux / macOS menu) | strong | auto | ✅ done |
| LJ5 | Fullscreen video: WebView2 handler, engine checks, smoke test page | strong | auto | ✅ done |
| LJ6 | Settings window: shortcut recorder, search engine, translate language; live shortcut re-register | standard | auto | ✅ done |
| LJ7 | Translate page + open in default browser (tray, macOS menu, keys) | standard | auto | ✅ done |
| LJ8 | Packaging in CI: NSIS (Windows), universal DMG (macOS), deb + AppImage (Linux), artifacts uploaded | strong | auto | ✅ done |
| LJ9 | Install-and-run smoke in CI: install what users download, drive it, screenshots, memory check | strong | auto | ✅ done (Linux smoke on 24.04 — see notes) |
| LJ10 | Docs + human checkpoint: README, architecture, checklist in the last PR | standard | checkpoint | ✅ done — waiting for the human check |
| LJ10.1 | *(added mid-plan 2026-10-17)* Always-visible URL bar: a trusted local webview above the page | strong | auto | ✅ done |
| LJ10.2 | *(added mid-plan 2026-10-17)* Project files: CHANGELOG, CONTRIBUTING, SECURITY, repo metadata | standard | auto | ✅ done |
| LJ10.3 | *(added mid-plan 2026-10-17)* Page size at every scale factor; phone mode for narrow windows | strong | auto | ✅ done |
| LJ11 | *(v0.2)* Start at login: tauri-plugin-autostart, a Settings switch, `--start-at-login on\|off`, the OS entry checked in smoke | strong | auto | ✅ done |
| LJ12 | *(v0.2)* First launch: the window opens once with a welcome (shortcut, where the icon is, start at login) | standard | auto | ✅ done |
| LJ13 | *(v0.2)* AppImage built on Ubuntu 24.04 (WebKitGTK 2.52); the AppImage gets the full smoke | strong | auto | ✅ done |
| LJ14 | *(v0.2)* WebKitGTK older than 2.52: fullscreen the window instead of aborting; an Ubuntu 22.04 smoke job | strong | auto | ⏭ stopped at its experiment: 2.50.4 aborts even when `enter-fullscreen` is handled — no app change; the Launchpad draft written (see notes) → LJ14.1 |
| LJ14.1 | *(added mid-plan 2026-10-19)* WebKitGTK older than 2.52: element fullscreen switched off instead of the abort; an Ubuntu 22.04 smoke job | strong | auto | ✅ done |
| LJ15 | *(v0.2)* Downloads: save to the Downloads folder; status and "Show in folder" in the URL bar | strong | auto | ✅ done |
| LJ16 | *(v0.2)* Page zoom: Ctrl/Cmd + / − / 0 through native keys and `Webview::set_zoom` | standard | auto | ✅ done |
| LJ17 | *(v0.2)* Wayland-native global shortcut through the XDG GlobalShortcuts portal | strong | auto | ✅ done |
| LJ18 | *(v0.2)* macOS Developer ID signing and notarization in CI (runs only if the owner's secrets exist; else ⏭ skipped and recorded) | strong | auto | ⏭ skipped — no Apple signing secrets on 2026-10-19; runs when the owner adds them (LJ18's pre-step) |
| LJ18.1 | *(added mid-plan 2026-10-19)* Linux ARM64 builds: an `arm64` `.deb` (ubuntu-22.04-arm) and an `aarch64` AppImage (ubuntu-24.04-arm), smoke-tested on ARM runners like their x86_64 twins | strong | auto | ✅ done |
| LJ19 | *(v0.2)* Release v0.2.0: versions, CHANGELOG, README; the owner's checklist; the green PR may be merged, the tag waits for the owner's word | standard | checkpoint | ⬜ todo |
| LJ20 | *(v0.2, round 2)* Logins survive: CI proves cookies and storage outlive close, quit and relaunch; fix what doesn't | strong | auto | ✅ done (checks first, then lj-step-20-fix — see notes) |
| LJ21 | *(v0.2, round 2)* History: a native recorder, a local store, frecency, top sites and matching (pure, tested) | strong | auto | ✅ done |
| LJ22 | *(v0.2, round 2)* All trusted UI moves into the bar webview; the page webview gets no IPC at all; the overlay layout mode | strong | auto | ✅ done |
| LJ23 | *(v0.2, round 2)* The command bar: suggestions when empty, history + search + commands while typing, keyboard, favicons | strong | auto | ✅ done |
| LJ24 | *(v0.2, round 2)* The Little Arc-style toolbar: favicon + domain, back, open in browser; macOS traffic lights inside it | strong | auto | ✅ done |
| LJ25 | *(v0.2, round 2)* History view (Ctrl/Cmd+Y): by day, search, delete, clear | standard | auto | ✅ done |
| LJ26 | *(v0.2, round 2)* Settings window with tabs (General, Appearance, Extensions, Privacy, About), reached from the command bar | standard | auto | ✅ done |
| LJ27 | *(v0.2, round 2)* Theme engine: 1–3 colors, intensity, grain, light/dark/system → muted gradients everywhere we draw | strong | auto | ✅ done |
| LJ28 | *(v0.2, round 2)* Appearance editor: color pads, presets, live preview | standard | auto | ✅ done |
| LJ29 | *(v0.2, round 2)* Extension runtime: manifest, registry, `ljext` protocol with per-extension CSP, the narrow API, `--extension <id>` | strong | auto | ✅ done |
| LJ30 | *(v0.2, round 2)* Extensions in the command bar and Settings; developer mode; `docs/extensions.md` (experimental) and a template | strong | auto | ✅ done |
| LJ31 | *(v0.2, round 2)* Built-in quick answers: calculator, units, time zones, random / UUID / password, timestamps | standard | auto | ✅ done |
| LJ32 | *(v0.2, round 2)* Bundled games: Snake and 2048 | standard | auto | ✅ done |
| LJ33 | *(v0.2, round 2)* Bundled productivity: Notes, Focus timer, World clock | standard | auto | ✅ done |
| LJ34 | *(v0.2, round 2)* Bundled page actions and tools: QR code, Clean link, Wayback, Markdown link, JSON + text + color tools | standard | auto | ✅ done |
| LJ35 | *(v0.2, round 2)* Docs, screenshots, the user's checklist; v0.2.0-beta.2 on the user's word | standard | checkpoint | ✅ done — the owner's checklist passed 2026-10-18 (one YouTube fullscreen bug, see notes); v0.2.0-beta.2 released |
| LJ36 | *(round 3)* YouTube's fullscreen on macOS (the phone site, confirmed by the owner): reproduce in CI; guard WebKit's fullscreen; youtube.com on the desktop agent on WebKit (a minimal per-site table) | strong | auto | ✅ done (reproduced, then lj-step-36-fix — see notes) |
| LJ37 | *(round 3)* Ctrl+H for history on Windows and Linux (Cmd+Y stays); double-clicking the toolbar zooms the window | standard | auto | ✅ done |
| LJ38 | *(round 3)* The bar's first paint: its theme before it paints; panels opened from hibernation wait for the bar | strong | auto | ✅ done |
| LJ39 | *(round 3)* Live resize like Little Arc: only changed layout, macOS autoresizing masks, themed webview backgrounds, phone-mode hysteresis (600 / 640 px) | strong | auto | ✅ done (the bar's background only — see notes) |
| LJ40 | *(round 3)* Phone mode per site: a command, a toolbar pill, a Settings list; Windows' phone agent with client hints; the owner's check | strong | checkpoint | ✅ done — the owner's check passed 2026-10-18 (see LJ40's notes); `v0.2.0-beta.3` released |
| LJ41 | *(round 3)* Hide the tray / menu-bar icon (back by itself when the shortcut can't work); Settings and Quit in the command bar | standard | auto | ✅ done |
| LJ42 | *(round 3)* Quick commands in the command bar, each showing its current state | standard | auto | ✅ done |
| LJ43 | *(round 3)* One keymap: every Little Joan key configurable (engine side; macOS menu from the keymap) | strong | auto | ✅ done |
| LJ44 | *(round 3)* Settings → Keyboard: record keys, clashes, the OS's reserved shortcuts, "another app holds it" | standard | auto | ✅ done |
| LJ45 | *(v0.3)* Links from other apps: register as a browser candidate per OS, receive links safely | strong | auto | ⬜ todo |
| LJ46 | *(v0.3)* The main browser: handing pages on without loops; the default-apps button | strong | review | ⬜ todo |
| LJ47 | *(v0.3)* Sign-in popups in a small IPC-free window that shares the store and the opener | strong | auto | ⬜ todo |
| LJ48 | *(v0.3)* Popups close with their page; Ctrl/Cmd+W; docs | strong | review | ⬜ todo |
| LJ49 | *(v0.3)* Find in page: Ctrl/Cmd+F, the engines' own find, a find strip in the toolbar | strong | auto | ⬜ todo |
| LJ50 | *(v0.3)* Pinned sites at the top of the command bar | standard | auto | ⬜ todo |
| LJ51 | *(v0.3)* Windows: WebView2's password saving and autofill as Privacy settings; removing them | strong | auto | ⬜ todo |
| LJ52 | *(v0.3)* Passkeys and saved logins: what each engine offers; docs; the macOS research | standard | review | ⬜ todo |
| LJ53 | *(v0.3)* Bundled: Reading list, To-do | standard | auto | ⬜ todo |
| LJ54 | *(v0.3)* Bundled: Stopwatch & countdown, Breathing timer, Minesweeper | standard | auto | ⬜ todo |
| LJ55 | *(v0.3)* Bundled: Password generator, Regex tester, Hash & Base64, Emoji picker | standard | auto | ⬜ todo |
| LJ56 | *(v0.3)* Panels follow theme changes; `--extension` opens a page action | standard | auto | ⬜ todo |
| LJ57 | *(v0.3)* Docs, screenshots, the owner's checklist; a v0.3 pre-release only on the owner's word | standard | checkpoint | ⬜ todo |

**Overall status: v0.1.0 (2026-10-17). LJ1–LJ10 passed the user's own check on macOS. Added
mid-plan: LJ10.1–LJ10.2 (the always-visible URL bar and the project files) and LJ10.3 (phone mode
for narrow windows, and layout tests at scale factors 1, 1.5 and 2). PRs #1–#7 were merged into
`main` at the user's request. The GitHub Release `v0.1.0` carries the Windows, macOS and Linux
installers CI built from the tagged commit. Linux fullscreen: the AppImage is fixed by LJ13; on
Ubuntu 22.04's `.deb` (WebKitGTK 2.50.4, which aborts on element fullscreen even when the app
declines the request) fullscreen is switched off since LJ14.1 — no crash, no fullscreen video; see
Deferred follow-ups.**

**v0.2 status: planned 2026-10-17 with the user (scope and decisions under "Design Decisions
Already Made → v0.2"). PR #8 (housekeeping: `.claude/` ignored, the Linux verify runner pinned
to `ubuntu-24.04`) and PR #9 (this plan) are merged. Phase 6 (LJ11 → LJ12) is in progress on
`lj/p6-first-run` (PR #10): LJ11 and LJ12 done, waiting for the user's review and checks. Next:
phase 7 (LJ13 → LJ14). LJ18 also waits for the user to add the Apple signing secrets (its prompt
lists them).**

**v0.2 round 2 (planned 2026-10-17): the user checked phase 6 on their Mac (all passed), PR #10
was merged, and `v0.2.0-beta.1` was published as a GitHub pre-release. The user then asked for
Little Joan to look and feel like Little Arc: a command bar with suggestions, history, logins
that survive, themes, and an extension system with bundled extensions (decisions: "Design
Decisions Already Made → v0.2 round 2"). These phases (12–17, LJ20–LJ35) run NEXT, before phases
7–10; LJ19 stays the final v0.2.0 release. The user allowed the agent to merge each phase PR
once its CI is green (CLAUDE.md, standing exception). Phase 12 (LJ20 → LJ21: logins survive
close and quit on all three OSes; local history with frecency) was PR #13, merged. Phase 13
(LJ22 → LJ24: every trusted screen in the bar webview and no IPC at all for the page webview, the
command bar with suggestions, the Little Arc-style toolbar with the macOS traffic lights inside
it; it opened with `lj-step-20-fix2` for the macOS abort `main`'s CI had shown) was PR #14,
merged. Phase 14 (LJ25 → LJ26: the history view with Ctrl/Cmd+Y, removing and clearing history
on disk at once, the tabbed Settings window with `--settings [tab]`) is PR #15 on
`lj/p14-history-settings`, merged. Phase 15 (LJ27 → LJ28: the theme engine — muted gradients from
1–3 colors on every surface we draw — and the Appearance editor; it also fixed the bar missing a
view change while its page loaded) is PR #16 on `lj/p15-themes`, merged once verify, package and
smoke are green. Phase 16 (LJ29 → LJ31: the extension platform — the `ljext` runtime with one
origin per extension, a CSP from the manifest and a checked API, extensions in the command bar and
Settings with developer mode and `docs/extensions.md`, and the built-in quick answers) is PR #17 on
`lj/p16-extensions`. It is security-critical, so it is **not** merged by the agent. An
independent security review (2026-10-18) found no Critical/High issues; its findings, and what the
new request log showed while fixing them, are fixed in `lj-step-29-fix4` → `fix8` (LJ29's notes,
"Security review"); the merge decision is the owner's; it was merged on 2026-10-18, and `main`'s CI for the merge
commit is green (run 39116954733). Phase 17 (LJ32 → LJ35: the
bundled extensions — games, productivity, page actions and tools — then the docs and the owner's
checklist) is PR #18 on `lj/p17-bundled-extensions`, merged by the agent once verify, package and
smoke were green on every OS (the standing exception); LJ35 is a checkpoint: the owner's checklist
is in PR #18. **2026-10-18: the owner's checklist passed ("all works amazingly"; one YouTube
fullscreen bug, see LJ35's notes — the phone site, as the owner confirmed later that day), and on
their word `v0.2.0-beta.2` was released as a pre-release.
Next: the owner's new requests ("Requested for the next version" under LJ35) and phases 7–10 —
nothing starts until the owner says yes.**

**Round 3 (planned 2026-10-18; decisions under "Design Decisions Already Made → round 3"): the
owner said yes to the next version.** It finishes **v0.2.0** — phase 18 (LJ36–LJ38: YouTube's
fullscreen on macOS reproduced in CI and fixed — the owner confirmed it was the phone site —,
Ctrl+H, the bar's first paint, double-click to zoom), phase 19 (LJ39–LJ40: live resize like Little
Arc and phone mode per site, designed from the resize research), phase 20 (LJ41–LJ44: hide the icon safely, quick
commands, a Keyboard tab for every key), then round 1's phases 7–11 (LJ13–LJ19, their prompts
adjusted to round 2's ground: the toolbar, the keymap, merging into `main`) — and plans **v0.3**:
phases 21–26 (LJ45–LJ57: Little Joan as the system's browser for links, sign-in popups, find in page
and pinned sites, saved logins on Windows, nine more bundled extensions, the docs and the owner's
checklist). **Execution order: 18 → 19 → 20 → 7 → 8 → 9 → 10 → 11 → 21 → 22 → 23 → 24 → 25 → 26. **2026-10-18: phases 18 and 19 are merged (PRs #21, #22); the owner's checkpoint passed, and on their word `v0.2.0-beta.3` was released (pre-release). 2026-10-19: phase 20 (PR #24) merged. Phase 7 (PR #25): LJ13 done; LJ14 stopped at its own experiment; on the owner's call LJ14.1 (fullscreen off on WebKitGTK < 2.52) was added and done. Phase 8 (PR #26: downloads and page zoom) merged. Phase 9 (PR #27: the Wayland shortcut through the XDG GlobalShortcuts portal) merged. Phase 10 (LJ18) ⏭ skipped: `gh secret list --repo memoliertorerr/LittleJoan` shows no Apple signing secrets (checked 2026-10-19), so the macOS build stays ad-hoc signed and LJ19's checklist keeps the README's Gatekeeper step; LJ18 runs as written once the owner adds the secrets (its pre-step). On the owner's decision (2026-10-19) phase 10 instead carries **LJ18.1, Linux ARM64 builds** (added mid-plan: the owner tests Ubuntu 22.04 ARM in Parallels on an Apple silicon Mac at LJ19's checkpoint): an `arm64` `.deb` built on ubuntu-22.04-arm and an `aarch64` AppImage built on ubuntu-24.04-arm, both smoke-tested on ARM runners like their x86_64 twins (branch `lj/p10-arm64`). Next: LJ19.****
**The owner's update (2026-10-18): checkpoints stop for the owner's own testing and review** — the
end of phase 19 (LJ40: the resizing feel), LJ19 (v0.2.0 release prep) and v0.3's last step (LJ57)
are `checkpoint` gates: the step fills the PR with a precise checklist, the agent may merge the
green PR, and then STOPS; the next phase starts only when the owner says go. **The agent's merge
permission is no wider than before:** it merges a phase PR once verify, package and smoke are green
on every OS job; the security-sensitive phases 21, 22 and 24 end in a `review` step — ready, green,
NOT merged — for an independent review and the owner's decision; tags and releases happen only on
the owner's word. LJ18 runs only if the owner's Apple signing secrets exist, and is otherwise
recorded as skipped. CLAUDE.md's standing exception covers the approved plan's v0.2 and v0.3
phases (confirmed 2026-10-18 for round 3), so v0.3's `auto` phases (23, 25) merge when green like
v0.2's (Ground Rule 24).

**Phase 18 (LJ36 → LJ38, 2026-10-18)** is PR #21 on `lj/p18-fixes`, merged by the agent once
verify, package and smoke were green on every OS job (the standing exception). CI reproduced
YouTube's macOS fullscreen bug — our layout and phone mode acting inside WebKit's fullscreen window —
and the guard fixed it on every element path; YouTube gets the desktop agent on WebKit; Ctrl+H
opens history on Windows and Linux; double-clicking the toolbar zooms; the bar paints its theme
first, and panels from hibernation wait for it. Human checks are in PR #21. **Next: phase 19
(LJ39 → LJ40)**, which ends in a checkpoint.

**Phase 10 (LJ18 ⏭ skipped, LJ18.1, 2026-10-19)** is PR #28 on `lj/p10-arm64`, merged by the agent
once verify, package and smoke were green on every job (the standing exception). LJ18 is skipped —
no Apple signing secrets —, and LJ18.1 adds Linux ARM64: an `arm64` `.deb` built on
ubuntu-22.04-arm (3.03 MiB, glibc 2.34) and an `aarch64` AppImage built on ubuntu-24.04-arm (148.62
MiB, WebKitGTK 2.52.6 bundled, glibc 2.39), each smoke-tested on ARM runners like its x86_64 twin
(`linux-arm64`, `linux-22.04-arm64` with fullscreen off on 2.50.4, `linux-appimage-arm64`); all
eighteen jobs green on the first try. The owner's check — the arm64 `.deb` in Ubuntu 22.04 ARM in
Parallels — is in PR #28 and LJ19's checklist.

**Phase 9 (LJ17, 2026-10-19)** is PR #27 on `lj/p9-wayland-shortcut`, merged by the agent once
verify, package and smoke were green on every job (the standing exception). In a Linux Wayland
session Little Joan asks the XDG GlobalShortcuts portal for its shortcut first (ashpd, sharing the
one zbus 5), registered with the portal's host registry under its app id — the `.deb` now also
installs `io.github.memoliertorerr.littlejoan.desktop` for it, since Tauri's is "Little
Joan.desktop" —; the desktop owns the key, which the welcome and Settings → Keyboard show; a
portal shortcut counts as working for the icon rule; without the portal everything is as before
(`--toggle`). X11, Windows and macOS are untouched. CI can't run a portal backend: its Wayland
launch showed the fallback. The portal path is the owner's check (PR #27, LJ19's checklist).

**Phase 8 (LJ15 → LJ16, 2026-10-19)** is PR #26 on `lj/p8-downloads-zoom`, merged by the agent once
verify, package and smoke were green on every job (the standing exception). Downloads go to the OS
Downloads folder under a cleaned, unique name on every engine, with "Downloading …" / "… saved" +
Show in folder / "Download failed" in the toolbar's status area; none while an extension's panel
shows; closing the window ends a download under way (WebKitGTK cancelled through an emission hook
— 2.50.4 needed it —, WKDownloads noted through a wrapped `-[WKDownload setDelegate:]` and
cancelled, WebView2's end with the window). Page zoom: Ctrl/Cmd + / − / 0 as keymap actions (the
Keyboard tab lists them, the macOS View menu has them), 50–300 %, per window, "Zoom 110 %" in the
status area. Human checks are in PR #26; they also go into LJ19's checklist.

**Phase 7 (LJ13 → LJ14.1, 2026-10-19)** is PR #25 on `lj/p7-linux-fullscreen`, merged by the agent
once verify, package and smoke were green on every job (the standing exception). LJ13: the AppImage
is built on ubuntu-24.04 (WebKitGTK 2.52.6 bundled, glibc 2.39, 156.79 MiB) and passes the whole
smoke as `linux-appimage`; the `.deb` stays on ubuntu-22.04. LJ14 stopped at its own experiment:
WebKitGTK 2.50.4 aborts even when `enter-fullscreen` declines the request, so the window stopgap
can't work; the Launchpad report is drafted in `docs/upstream/`. On the owner's call ("a crash is
worse than no fullscreen") **LJ14.1** was added: with `enable-fullscreen` off the bare window
survives on 22.04, so the page webview gets it off below WebKitGTK 2.52, and a permanent
`linux-22.04` smoke job runs the whole smoke on 22.04's engine. Human checks are in PR #25.

**Phase 20 (LJ41 → LJ44, 2026-10-19)** is PR #24 on `lj/p20-control`, merged by the agent once
verify, package and smoke were green on every OS job (the standing exception; LJ36's fullscreen
run missed its click on linux-scale-2 three times — a stale `page-screen` line after leaving
fullscreen —, fixed in the harness by `lj-step-44-fix2`; the app's layout race is under Deferred
follow-ups). The tray / menu-bar
icon can be hidden and comes back by itself whenever the shortcut can't work; the command bar has
Settings, Quit and quick commands showing their state (settings changes share one checked path
with Save); every Little Joan key comes from one keymap and can be changed in Settings → Keyboard,
with clashes refused and the OSes' reserved shortcuts warned about (on macOS refined by its own
preferences, read-only). Human checks are in PR #24. **Next: phase 7 (LJ13 → LJ14)** — on the
owner's go.

**Phase 19 (LJ39 → LJ40, 2026-10-18)** is PR #22 on `lj/p19-resizing`, merged by the agent once
verify, package and smoke were green on every OS job (the standing exception). Resizing applies
only what changed, macOS's webviews follow the window through autoresizing masks, the toolbar's
webview background is the theme's, and phone mode switches below 600 px and back only from 640 px
(CI's resize run: one switch and one reload per crossing, at most two bounds per resize); phone or
desktop per site has a toolbar pill, command-bar rows and a Settings list; Windows' phone agent
carries client hints through WebView2's own DevTools protocol (CI: `?1` / `"Android"` /
`userAgentData.mobile` true, in the release build). **LJ40 is a checkpoint: waiting for the
owner's check (the checklist in PR #22); phase 20 starts only on the owner's go.**

**Model note:** phases 1, 2, and 4 need the strongest model available — phase 1 sets the shape
every later step inherits, phase 2 is native per-OS code the local Mac cannot compile for
Windows or Linux (CI is the only compiler for those paths), and phase 4 is CI packaging, where
mistakes only show up after a 15-minute round trip. Phase 3 is ordinary UI + glue on top of
settled contracts and can run on a standard model. In v0.2 the strong steps are the ones that
touch three OSes' native APIs or CI packaging (LJ11, LJ13–LJ15, LJ17, LJ18); LJ12, LJ16 and
LJ19 are UI, key tables and release chores on settled code. *(Round 3)* Phases 18–22 and 24 run
on the strong model: macOS fullscreen and first-paint timing (18), per-OS resize and agent code
(19), the keymap across three native key paths (20), and the security-sensitive phases (21, 22,
24); phase 23's find glue is native too, so it runs strong as well. Phases 25 and 26 (bundled
extensions in an established layout, docs) can run on a standard model.

**Phases** — one pull request each, stacked (each phase branches from the previous one):
1. **LJ1 → LJ2** · strong — `lj/p1-foundation` → PR into `main`. Review: the toolchain/caching
   layout, the pure-core contracts (URL normalization rules, settings shape, toggle table).
2. **LJ3 → LJ5** · strong — `lj/p2-summon` → PR into `lj/p1-foundation`. Review: window
   lifecycle (nothing alive at rest), the no-IPC-for-remote-pages rule, per-OS native glue.
3. **LJ6 → LJ7** · standard — `lj/p3-settings` → PR into `lj/p2-summon`. Review: settings UI,
   shortcut re-registration with rollback, translate.
4. **LJ8 → LJ10** · strong — `lj/p4-ship` → PR into `lj/p3-settings`. Review: installers, the
   CI smoke screenshots, and the **human checklist**.
5. **LJ10.1 → LJ10.2** · strong — `lj/p5-url-bar` → PR into `lj/p4-ship` *(added mid-plan
   2026-10-17)*. Review: the two-webview window (trusted bar, IPC-free page), the `unstable`
   feature, layout and fullscreen per OS, the project files. No human checkpoint (the user
   asked for none).

**v0.2 phases** — stacked the same way: each PR goes into the previous phase's branch, or into
`main` once that phase is merged (`gh pr edit <n> --base main --repo memoliertorerr/LittleJoan`).
Every phase PR's "Needs human check" names the v0.2 must-not-disturb list above.

6. **LJ11 → LJ12** · strong — `lj/p6-first-run` → PR into `main`. Review: the login entry per
   OS, the welcome, the new `set_start_at_login` command and who may call it (local pages only).
7. **LJ13 → LJ14** · strong — `lj/p7-linux-fullscreen` → PR into `lj/p6-first-run`. Review: the
   two Linux package jobs and their glibc floors, the 22.04 stopgap and its smoke job.
8. **LJ15 → LJ16** · strong — `lj/p8-downloads-zoom` → PR into `lj/p7-linux-fullscreen`.
   Review: where files land and how they're named, the bar's status line, zoom keys per OS.
9. **LJ17** · strong — `lj/p9-wayland-shortcut` → PR into `lj/p8-downloads-zoom`. Review: the
   new `ashpd` dependency, the fallback without a portal. Human check on a Wayland desktop.
10. **LJ18** · strong — `lj/p10-macos-signing` → PR into `lj/p9-wayland-shortcut`. Starts only
    when the user's signing secrets exist. Review: how secrets reach one step only, the ad-hoc
    fallback, the Gatekeeper evidence.
11. **LJ19** · standard — `lj/release-0.2.0` → PR into `lj/p10-macos-signing` (or `main`).
    Checkpoint: the user's checklist; merge, tag and release only on their word.

**v0.2 round 2 phases** — **execution order: 6 (done) → 12 → 13 → 14 → 15 → 16 → 17 → 7 → 8
→ 9 → 10 → 11.** The user lets the agent merge each phase PR into `main` once CI is green, so
every phase branches from the latest `main` and its PR goes into `main` (no stacking). The
phases after 17 then branch from `main` too; LJ15 and LJ16 use the toolbar that LJ24 builds.

12. **LJ20 → LJ21** · strong — `lj/p12-logins-history`. Review: what the cookie checks found per
    OS and what was fixed; the history store's format, size cap and what it never records.
13. **LJ22 → LJ24** · strong — `lj/p13-command-bar`. Review: no capability names `joan-page`
    any more; the overlay layout on Linux; the command bar and toolbar screenshots per OS.
14. **LJ25 → LJ26** · standard — `lj/p14-history-settings`. Review: deleting and clearing
    history really removes it from disk; the tabbed Settings window.
15. **LJ27 → LJ28** · standard (LJ27 strong) — `lj/p15-themes`. Review: theme math tests, the
    screenshots in light and dark, contrast of text on every preset.
16. **LJ29 → LJ31** · strong — `lj/p16-extensions`. Review: the security model (extensions run
    only in the IPC-free page webview, the API's request checks, CSP, path traversal), the docs.
17. **LJ32 → LJ35** · standard — `lj/p17-bundled-extensions`. Ends in a checkpoint: the user's
    checklist, then `v0.2.0-beta.2` only on their word.

**Round 3 phases** — **execution order: 18 → 19 → 20 → 7 → 8 → 9 → 10 → 11 → 21 → 22 → 23 → 24
→ 25 → 26.** Every phase branches from the latest `main` and its PR goes into `main`. Round 1's
phases 7–11 keep their numbers and steps (LJ13–LJ19), adjusted where round 2 changed the ground.

18. **LJ36 → LJ38** · strong — `lj/p18-fixes`. Review: the fullscreen reproduction's log per OS,
    the macOS fullscreen guard, the minimal per-site table (YouTube on the desktop agent on
    WebKit); the history key; the bar's initialization script (bar only). Merged by the agent when
    green.
19. **LJ39 → LJ40** · strong — `lj/p19-resizing`. Review: the layout diffing and macOS masks, the
    phone-mode hysteresis, the per-site UI, Windows' agent override through CDP; the resize smoke
    numbers. **Ends in a checkpoint** (the owner's eyes on the resizing feel).
20. **LJ41 → LJ44** · strong — `lj/p20-control`. Review: the icon's safety rule, the quick
    commands' fixed command set, the keymap replacing every hard-coded key. Merged when green.
7. **LJ13 → LJ14** · strong — `lj/p7-linux-fullscreen`. As planned; merged when green.
8. **LJ15 → LJ16** · strong — `lj/p8-downloads-zoom`. Downloads' status in the toolbar; zoom keys in
   the keymap. Merged when green.
9. **LJ17** · strong — `lj/p9-wayland-shortcut`. The portal's trigger in the Keyboard tab; a portal
   shortcut counts as working for the icon rule. Merged when green.
10. **LJ18** · strong — `lj/p10-macos-signing`, only if the owner's Apple secrets exist (`gh secret
    list` names only); otherwise ⏭ skipped and recorded in LJ19's PR. Merged when green. *(⏭ skipped
    2026-10-19: no secrets; recorded in phase 10's PR below, as the owner decided.)*
    **LJ18.1** · strong — `lj/p10-arm64` *(added mid-plan 2026-10-19)*: Linux ARM64 packages and
    their smoke jobs, with LJ18's skip as its first commit. Review: the four new jobs, the arm64
    glibc floor and bundled WebKitGTK, the release names. Merged when green.
11. **LJ19** · standard — `lj/release-0.2.0`. **Checkpoint:** the owner's checklist; the
    version-bump PR may be merged when green; tag and release only on the owner's word.
21. **LJ45 → LJ46** · strong — `lj/p21-links`. **Security-sensitive (`review`):** ready and green,
    not merged. Review: argument injection, the OS registrations and their removal, no defaults
    changed, the handoff loop guard.
22. **LJ47 → LJ48** · strong — `lj/p22-popups`, after phase 21 is merged. **Security-sensitive
    (`review`).** Review: popups have no IPC, what decides a popup, its lifecycle.
23. **LJ49 → LJ50** · strong — `lj/p23-find-pins`. Review: the per-engine find glue. Merged when
    green.
24. **LJ51 → LJ52** · strong — `lj/p24-saved-logins`. **Security-sensitive (`review`).** Review:
    password saving off by default, removal clears the right profile, no credential handling.
25. **LJ53 → LJ56** · standard — `lj/p25-more-extensions`. Review: the bundled extensions' rules
    (no network, text as text), the two platform touch-ups. Merged when green.
26. **LJ57** · standard — `lj/p26-v0.3-docs`. **Checkpoint:** the owner's v0.3 checklist; a
    pre-release only on their word.

Never start a phase with uncommitted changes in the tree.

---

## How To Run This

1. Open a **new session** in `LittleJoan/` on the model the next phase names. The local,
   gitignored `CLAUDE.md` (if present) loads automatically; this plan is the source of truth.
2. **Paste the Opening prompt.** It reports which phase is next and waits.
3. **Paste the Phase prompt** with that phase's step range, and say go.
4. The session runs each step: does the work, runs `npm run verify`, ticks its tracker row,
   and commits — one commit per step, only when green. It pushes after each commit so CI
   checks the Windows and Linux code paths the Mac cannot compile, and fixes CI-only failures
   in follow-up commits (Ground Rule 3). It stops at a checkpoint, at the end of the phase, or
   at any stop condition, and reports.
5. At the end of the phase it pushes and marks the PR ready. You review the PR, do the checks
   listed under **Needs human check**, and merge — or reply with fixes.

The first build (v0.1) was run end to end by one autonomous session at the user's request:
all four phases back to back, stopping only at LJ10's human checkpoint. PRs stay open for the
user to review and merge in order (phase 1 first).

v0.2 (phases 6–11) uses the same prompts. The user says per run how far to go: one phase, or
several back to back. LJ18 needs the user's signing secrets before it starts (see its prompt).
`CLAUDE.md` lets the agent commit and push on branches; merging, tagging and releasing still need
the user's word for each one.

Round 3 uses the same prompts. The owner says per run how far to go. Within that range a phase
whose last step is `auto` is merged by the agent once CI is green, and the next phase starts from
the latest `main`; a `checkpoint` (LJ40, LJ19, LJ57) or a `review` step (LJ46, LJ48, LJ52) ends
the run whatever the range says (Ground Rule 24).

### Opening prompt — every session

```text
You are building Little Joan, a tiny cross-platform pop-up browser (Tauri 2 + TypeScript) that
hibernates until a global shortcut summons it, in LittleJoan/ (its own git repo).
Before any work:
1. Read docs/BUILD_PLAN.md and docs/architecture.md in full, and CLAUDE.md if it exists
   (local-only, gitignored).
2. Run `git status`, `git branch --show-current`, and `git log -1 --oneline`. Confirm the
   tree is clean and you are on the branch of the next phase. If that branch doesn't exist
   yet, create it from the previous phase's branch as "0. Before Starting" shows, and show me
   the output.
3. Run `source scripts/env.sh && rustc --version && node --version && gh auth status` and
   `df -h .`. Free disk must be above 3 GB.
4. Restate, in one line each, the rules you will not break this session:
   (a) you commit only when `npm run verify` is green, one commit per step, never on main;
   (b) you never force-push, rewrite pushed history, or delete a branch, and you merge only
       what Ground Rule 24 allows (a phase PR whose CI is green under CLAUDE.md's standing
       exception — never a `review` phase's PR; tags and releases only on the owner's word);
   (c) you stop at checkpoints and `review` steps, at the end of the range you were given, and
       at every stop condition in Ground Rules — list them;
   (d) you write only inside LittleJoan/, source scripts/env.sh before every npm/node/cargo
       command, and never launch the GUI app on this machine;
   (e) remote web pages never get IPC, and at rest no webview exists.
5. Read the Progress Tracker and tell me which phase is next and which steps it covers.
Then wait for me to say go.
```

### Phase prompt

```text
Execute steps LJa → LJb from docs/BUILD_PLAN.md, in order.

For each step:
1. Run `df -h .` and note the free space. Read the step's prompt in the plan and follow it
   exactly. Do not start work the prompt doesn't ask for; record anything out of scope under
   "Deferred follow-ups".
2. Run its verify. If green: flip its tracker row to ✅, add implementation notes under the
   step (always including the free-disk numbers), and commit everything as one commit with
   the step's commit message.
3. Push. If the phase has no PR yet, open it as a draft (base = previous phase's branch; main
   for phase 1, phase 6, or when the previous phase is already merged) so CI runs. Watch CI with
   `gh run watch <id> --repo memoliertorerr/LittleJoan`. If CI is red on code the Mac cannot
   compile, fix it in a follow-up commit "lj-step-N-fix: ..." (Ground Rule 3).
4. Paste a short report: the verify output, `git log -1 --stat`, and the CI result per OS.
5. If the step's gate is `checkpoint`: fill the PR with the step's checklist, merge it once CI
   is green if the step says so, tell me exactly what to check, and stop. If it is `review`:
   fill the PR (the review focus first), mark it ready once CI is green, and stop — never merge
   it. If it is `auto`, continue to the next step.

Stop immediately at any stop condition in Ground Rules and tell me which one.
At the end of each phase in the range: push the branch, fill the PR description from the
template, and once CI is green either merge it (Ground Rule 24's `auto` case) and go on with the
next phase from the latest `main`, or — after the range's last phase — mark it ready / merge it as
its last step says, then stop.
```

### PR description template

```markdown
## Little Joan — phase <n>: <phase name>

Steps: LJa → LJb (plan: docs/BUILD_PLAN.md). Base: <base branch> (stacked PRs — merge in order).

| Step | Commit | Verify evidence |
|---|---|---|
| LJa | <short sha> <message> | `npm run verify` → <n> TS tests, <n> Rust tests, clippy clean; CI ubuntu/windows/macos green |

### Implementation notes
- <what differed from the prompts, and why>

### Added to Deferred follow-ups
- <item, or "none">

### Needs human check
- [ ] <exactly what to look at, and where — or "nothing beyond the diff">
- [ ] Nothing regressed: <the specific earlier features to re-check>
```

### Who verifies what

| Check | Who | "Green" looks like |
|---|---|---|
| `npm run verify` (tsc, eslint, vitest, UI build, cargo fmt, clippy `-D warnings`, cargo test) | Agent, on the Mac | Clean exit; test counts in the report |
| Windows + Linux code paths (`#[cfg(windows)]`, `#[cfg(target_os = "linux")]`) | CI (`verify` job on ubuntu-24.04, windows-latest, macos-latest) | All three jobs green on the pushed commit |
| Installers build | CI `package` job (LJ8) | NSIS `.exe`, universal `.dmg`, `.deb`, `.AppImage` uploaded as artifacts (*LJ18.1:* also the `arm64` `.deb` and the `aarch64` AppImage) |
| Installed app launches, summons, loads a page, element fullscreen toggles the window, closing frees the webview | CI `smoke` job (LJ9), agent reads the logs and looks at the screenshots | Smoke log shows the expected events; screenshots show the start page and a loaded page |
| **Nothing regressed** | you, in each PR | Earlier phases' behaviors in the PR's checklist still hold |
| Global shortcut on real desktops (Windows, macOS, Linux X11; Wayland fallback) | **you** — LJ10 checkpoint | The shortcut summons and dismisses the window; focus lands in the field |
| Resizing feel, window placement on multi-monitor, theme | **you** — LJ10 | The window opens on the monitor with the cursor, resizes smoothly, remembers its size |
| Fullscreen video on real hardware (YouTube) | **you** — LJ10 | Video fills the screen, Esc returns to the window, audio/video play |
| Memory at rest in Task Manager / Activity Monitor / `ps` | **you** — LJ10 (CI gives a first number) | No webview helper processes while hibernating |
| Gatekeeper (unsigned macOS), SmartScreen (unsigned Windows), tray on GNOME | **you** — LJ10 | The documented workaround opens the app |
| *(v0.2)* Login entry written and removed per OS | CI `smoke` (LJ11) | The registry value / LaunchAgent plist / autostart `.desktop` appears after `--start-at-login on`, points at the installed binary, and is gone after `off` |
| *(v0.2)* Start at login across a real logout or reboot; the first-run welcome | **you** — phase 6 PR | The icon is there after logging in and the shortcut works; a fresh profile shows the welcome once |
| *(v0.2)* Fullscreen on a real Ubuntu 22.04 desktop (the `.deb`), and the AppImage on 24.04 | **you** — phase 7 PR, if you have the machines | 22.04: no crash, the video stays in the window (fullscreen is off there, LJ14.1); 24.04 AppImage: the video fills the screen |
| *(v0.2)* A real download (a PDF from a site), "Show in folder", zoom keys | **you** — phase 8 PR | The file is in Downloads under its own name; the folder opens with it selected; zoom steps and resets |
| *(v0.2)* Wayland-native shortcut (GNOME 48+ or KDE Plasma, Wayland session) | **you** — LJ17 (CI has no portal backend) | The desktop asks once to allow the shortcut; it then summons and dismisses the window |
| *(v0.2)* Gatekeeper on a DMG downloaded with a browser (notarized build) | **you** — LJ18 / LJ19 | It opens without any warning or `xattr` step |
| *(LJ18.1)* Linux ARM64 on a real ARM machine | CI `smoke` on GitHub's arm64 runners (`linux-arm64`, `linux-22.04-arm64`, `linux-appimage-arm64`), then **you** — LJ19 (Ubuntu 22.04 ARM in Parallels) | The arm64 `.deb` installs; summon, a page and phone mode work; a video's fullscreen button does nothing and nothing crashes |
| *(round 2)* Cookies and storage survive close, quit and relaunch | CI `smoke` (LJ20) | The smoke page reports its persistent cookie, session cookie and localStorage value back after each step, on all three OSes |
| *(round 2)* No IPC for the page webview, ever | `cargo test` (LJ22) + smoke (LJ29) | A Rust test reads every capability file and finds no `joan-page`; the test extension reports `ipc=denied` |
| *(round 2)* How the command bar, toolbar, history view, themes and extensions look | the agent from CI screenshots on all three OSes (each step names its screenshots), then **you** — LJ35 | Screenshots match the step's description in light and dark; at LJ35 it feels like Little Arc on your Mac |
| *(round 2)* Suggestions feel right with real history | **you** — LJ35 | After a few days of use, the empty command bar shows the sites you actually open most, and typing finds them fast |
| *(round 3)* Element fullscreen at narrow and wide widths, entered and left by clicks; nothing touches the page during WebKit's fullscreen | CI `smoke` (LJ36's fullscreen run; macOS where the click reaches the page) | Every element path holds until the page's own exit and leaves when asked; with `siteModes` the page gets the desktop agent at 390 px; no `page-bounds` during `wk-fullscreen` on macOS |
| *(round 3)* YouTube's own player: button, F, exit button | **you** — phase 18 PR, LJ19 | The desktop site at any width on macOS; fullscreen stays; the exit button works |
| *(round 3)* Resizing: smoothness, the 600 / 640 px switch, one reload per crossing | CI `smoke` (LJ39's resize run: the final viewport, switches and reloads counted, `bounds-set` ≤ 2 per event; LJ40's agent headers per request) | The numbers in LJ39 / LJ40's Verify |
| *(round 3)* The resizing feel | **you** — LJ40 checkpoint | Dragging on a Retina Mac: no edge gaps, no lag; Windows: no white strips; Google / YouTube / Wikipedia right on both sides of 600 px; still signed in after switches |
| *(round 3)* The icon hidden, and back by itself when the shortcut can't work | CI `smoke` (LJ41's tray run logs) + **you** (seeing the menu bar / tray) | `tray-visible false` with the setting off; `true` on a Wayland session; on your Mac the icon is gone and Little Joan still opens |
| *(round 3)* Custom keys on real keyboards and layouts; the reserved-shortcut warnings | CI `smoke` (one rebinding, LJ43) + **you** — phase 20 PR | A rebound key works and the old one doesn't; the warnings name what the OS uses |
| *(v0.3)* Little Joan as the OS's browser for links | CI `smoke` (LJ45/46: the registration read back, a link launched as the OS would, the defaults untouched, the Linux loop check) + **you** — phase 21 PR | It appears in the OS's default-apps list; once chosen, links from other apps open in it, and "Open in browser" goes to your main browser |
| *(v0.3)* Sign-in popups with real providers | CI `smoke` (a cross-origin fake sign-in, LJ47) + **you** — phase 22 PR | Google / Apple / GitHub sign-ins finish in the popup, which shows the provider's host and closes |
| *(v0.3)* Find in page | CI `smoke` (counts on Windows / Linux, found on macOS) + **you** | Real pages: Enter, Shift+Enter, Esc behave |
| *(v0.3)* Saved passwords and passkeys on Windows | CI `smoke` (the settings read back, removal) + **you** — phase 24 PR | WebView2 offers to save, fills in, forgets after removal; a passkey with Windows Hello |
| *(v0.3)* Security-sensitive phases (21, 22, 24) | an independent read-only review, then the owner | No unaddressed Critical / High findings; the owner merges |

The agent self-verifies everything a compiler, a test suite, and CI (including headless
screenshots of the installed app) can show. Everything that needs a real desktop, real input
devices, real GPUs, or a real user session is the human's, collected in LJ10's checkpoint.

---

## Ground Rules

1. **Branches:** one per phase, `lj/p<n>-<name>`, each cut from the previous phase's branch
   (phase 1 from `main`). PRs are stacked: phase 1 → `main`, phase n → phase n-1's branch.
   After the bootstrap commit, **nothing is ever committed to `main`** by the agent. *(Since
   round 2: every phase branches from the latest `main` and its PR goes into `main`.)*
2. **One step = one commit**, message `lj-step-N: <message>`, made **only when
   `npm run verify` is green**. Never commit red.
3. **CI-only failures:** the Mac cannot compile the Windows/Linux code paths, so CI is part of
   each step's verification. If CI goes red after a step was pushed, fix it in a follow-up
   commit `lj-step-N-fix: <what>` (local verify green first) and note it in the step's
   implementation notes. Never amend or rebase a pushed commit.
4. **Push and PR:** the agent pushes after every step commit and opens the phase PR as a draft
   after the first one. It never force-pushes, never rewrites pushed history, never merges any
   PR, and never deletes a branch. `gh` commands always pass `--repo memoliertorerr/LittleJoan`.
   *(Since round 2 the owner's standing exception lets the agent merge green phase PRs; Ground
   Rule 24 says exactly what it may merge.)*
5. **Never weaken verification** to get green — no skipped or deleted tests, loosened
   assertions, `#[allow(...)]` / `eslint-disable` added to silence a failure, or swallowed
   errors.
6. **Stay inside the repo.** Write only inside `LittleJoan/`. Source `scripts/env.sh` before
   every npm, node, cargo, rustup, or tauri command (it points npm cache, TMPDIR, RUSTUP_HOME,
   CARGO_HOME, CARGO_TARGET_DIR, and the dev app's data at `.cache/`). No global installs, no
   sudo, no shell-profile or system-settings changes.
7. **Never launch the GUI app on the development Mac.** A running Tauri app writes outside the
   repo (WebKit data under `~/Library`, the single-instance socket in `/tmp`) and grabs a
   global shortcut on the user's desktop. The app runs only in CI (LJ9) and on the human's
   machine (LJ10). `cargo test` never opens a window, so it is fine.
8. **Disk:** run `df -h .` at the start of every step and after any big install or build; log
   the numbers in the step's implementation notes. Build debug only (never a local release
   build, never another OS's target). Run `cargo clean` when `.cache/target` passes ~2 GB.
   Stop if free space would fall below 3 GB.
9. **Hibernation is the product.** At rest there is no window and no webview: nothing is
   created at startup, and closing the window destroys it (the page is discarded). Nothing may
   pre-create, hide-instead-of-close, or keep a webview alive "for speed".
10. **Remote pages get no IPC.** Capabilities list only local windows (`joan`, `settings`) and
    never a `remote` URL. App commands are restricted per window through `build.rs`'s
    `AppManifest`. Native code, not page scripts, handles in-page keys.
11. **Navigation stays on the web.** Only `http`/`https` URLs are opened from user input, the
    command line, or new-window requests. `file:`, `javascript:`, and custom schemes are
    refused (see `navigation.rs`, LJ2).
12. **Branding:** the product is "Little Joan". No Arc name, logo, or artwork as branding; the
    icon is generated by `scripts/make-icons.mts` from original geometry.
13. **Git identity and attribution:** commits use the existing global git config
    (Mehmet Ali Ertorer). No `Co-Authored-By`, no "Generated with", no session links in
    commits or PRs. `CLAUDE.md` / `AGENTS.md` are gitignored and never committed.
14. **Code:** TypeScript `strict`; Rust 2024 edition, `cargo clippy -- -D warnings`, `rustfmt`.
    Pure logic lives in modules with no Tauri types and is unit-tested; native glue is thin.
    Files stay under ~400 lines. No `unwrap()` on anything that can fail at runtime outside
    tests — log and degrade.
15. **Every step:** update `docs/architecture.md` (file map + behavior), this tracker, and the
    README when user-facing behavior changes, in the same commit.
16. *(v0.2)* **Signing secrets belong to the user.** They live only in the repository's GitHub
    Actions secrets, which the user sets. The agent never reads, prints, asks for, or handles
    their values. Workflows pass them to the one step that signs, and fall back to ad-hoc
    signing when they're absent (forks, or before the user has added them).
17. *(v0.2)* **CHANGELOG:** every user-facing change goes under `## [Unreleased]` in the same
    commit; LJ19 turns it into `[0.2.0]` with the date.
18. *(round 2)* **The page webview (`joan-page`) has no IPC at all** — no capability may name it
    (a Rust test enforces this from LJ22 on). Every trusted screen — the toolbar, the command
    bar, the welcome, the history view — lives in the bar webview (`joan-bar`) or the settings
    window. Tauri treats custom-scheme pages as local (Landmines), so this is what keeps
    extension code, which runs in `joan-page`, away from app commands.
19. *(round 2)* **Extension code never runs in `joan-bar` or `settings`,** and is never served
    from the app's own origin. It runs only as top-level `ljext:` documents in `joan-page`, with
    a CSP from its manifest, and reaches the app only through the validated `ljext` API
    (LJ29). The bar renders anything an extension supplies (titles, keywords) as text, never
    as HTML.
20. *(round 2)* **History never leaves the machine.** It lives in the app's config folder, is
    loaded only while the window is open (nothing extra at rest), and is never sent anywhere;
    favicons are loaded by the bar straight from the sites the user visited.
21. *(round 3)* **Little Joan never changes OS defaults or system settings silently.** It may
    register itself as a candidate (a browser for http / https links, LJ45), read whether it is
    the default, and open the OS's own settings when the user clicks a button. It never writes
    the default-browser choice (Windows `UserChoice`, macOS Launch Services' default handlers,
    `xdg-settings` / `xdg-mime default`), never writes a system shortcut or any other system
    setting, and reads system settings only where a step names the read. Uninstalling removes
    what installing registered.
22. *(round 3)* **Links from other apps are untrusted input.** They arrive only as `--link <url>`
    (exactly one argument after it, never read as a flag) or as macOS's `RunEvent::Opened`; only
    absolute `http` / `https` URLs open — never `file:`, `javascript:`, `data:`, extension or app
    URLs — through the same rules as typed addresses, and nothing in a link can trigger another
    flag, a setting or a command.
23. *(round 3)* **In-app popups are page trust.** A popup window (`joan-popup-<n>`, LJ47) is an
    IPC-free web page like `joan-page`: no capability names it (`capability_tests.rs` enforces
    it), it never loads extension or app URLs, it opens no further popups, and hibernating closes
    it — nothing survives at rest.
24. *(round 3, the owner's update 2026-10-18)* **Merging, checkpoints and reviews.** The agent
    merges a phase PR into `main` — a merge commit, never squash, never `--delete-branch` — once
    verify, package and smoke are green on every OS job (CLAUDE.md's standing exception, which
    covers the approved plan's v0.2 and v0.3 phases). That permission is no wider than before: a
    `review` step (a security-sensitive phase's end) marks the PR ready and STOPS — never merged
    by the agent — for an independent review and the owner's decision; a `checkpoint` step fills
    the PR with a precise checklist, may merge its green PR, and STOPS — the next phase starts
    only when the owner says go; tags and releases happen only on the owner's word, one at a
    time.
25. *(round 3)* **One keymap.** From LJ43 on, every key of Little Joan's own (page keys, the macOS
    menu's key equivalents, keys shown in the bar or Settings) comes from `keymap.rs` and the
    user's overrides in settings; no other file hard-codes such a key. Typing keys (a letter or
    digit without a modifier, or with Shift alone) can never be bound.
26. *(round 3)* **Saved credentials stay in the engine.** Passwords, passkeys and autofill data
    live only in the web engine's own store. Little Joan never reads, exports, logs or syncs them;
    it only switches the engine's features on or off and asks the engine to remove them.

### Stop conditions

The agent stops, reports which condition fired, and waits:

- Verify (local or CI) is still red after two honest fix attempts.
- Getting green would mean weakening verification (Ground Rule 5).
- The step needs something outside its prompt — a file it didn't name, a **dependency the
  prompt doesn't list**, a schema change, a design decision. Record it in Deferred follow-ups;
  continue only if the step can be completed without it.
- The diff is much larger than the step implies.
- The prompt contradicts the code (a named file or interface doesn't exist as described).
- Anything destructive or irreversible; any CI workflow change in a step whose prompt doesn't
  name `.github/workflows/ci.yml`.
- Anything that would write outside `LittleJoan/` or touch a GitHub repo other than
  `memoliertorerr/LittleJoan`.
- Free disk would drop below 3 GB.
- A checkpoint step, or the end of the phase.
- *(round 3)* A `review` step: the PR is marked ready and never merged by the agent.
- *(round 3)* Anything that would change an OS default (the default browser, URL or file
  associations beyond what LJ45 registers, login items beyond LJ11's) or write a system setting
  (Ground Rule 21).
- *(round 3)* A change that would give `joan-page` or a `joan-popup-*` webview a capability, or
  add an initialization script to either.
- *(round 3)* Starting the next phase after a checkpoint, or while the previous security-sensitive
  phase's PR that touches the same files is still open, without the owner's go.

### Rollback

| Situation | Command |
|---|---|
| Step wrong, not yet committed | `git restore . && git clean -fd -e .cache -e node_modules -e CLAUDE.md` (run `git status` first) |
| A committed step is bad | `git revert <sha>` (each step is one green commit, so this is clean) |
| A whole phase is bad, PR not merged | Close the PR; reset the branch locally and `--force-with-lease` — **the human runs this**, never the agent |
| Abandon the whole thing | Close the PRs; tag `lj-pre-build` marks where building started |
| Disk tight | `source scripts/env.sh && cargo clean --manifest-path src-tauri/Cargo.toml`; `rm -rf .cache/npm .cache/tmp` (re-created on demand) |

---

## 0. Before Starting

Greenfield bootstrap (done once, by the first session):

```bash
cd LittleJoan
git init -b main
git add .gitignore LICENSE README.md docs/BUILD_PLAN.md
git commit -m "lj-step-0: build plan, readme stub, license, gitignore"
gh repo create memoliertorerr/LittleJoan --public --source . --remote origin
git push -u origin main
git tag lj-pre-build && git push origin lj-pre-build
git switch -c lj/p1-foundation
```

Each later phase (v0.1, v0.2 round 1 — stacked):

```bash
git switch lj/p<n-1>-<name> && git pull --ff-only
git switch -c lj/p<n>-<name>
```

Since round 2 (rounds 2 and 3, v0.3) every phase starts from the latest `main`:

```bash
git status                       # must be clean
git switch main && git pull --ff-only
git switch -c lj/p<n>-<name>
```

---

## Design Decisions Already Made

Do not redesign these mid-plan; if a step seems to require changing one, that is a stop
condition.

- **Stack: Tauri 2 (Rust core, system webview), not Electron.** "Very, very lightweight" decides
  it:
  - *At rest:* Tauri's hibernating process is one small native process with a tray icon and a
    hotkey listener; with the window destroyed there is no webview process at all. Electron
    always runs its main process plus GPU/utility helpers (tens of MB at minimum, typically
    more) even with no window.
  - *Install size:* a Tauri installer is a few MB (it uses the OS webview); an Electron
    installer carries Chromium (~80–100 MB download, ~250 MB on disk).
  - *Local disk:* the Rust toolchain plus a debug build costs ~2–3 GB in `.cache/` — within the
    5 GB budget; Electron would cost less locally, but that is a one-time developer cost, not a
    user cost.
  - *Fullscreen video:* macOS WKWebView element fullscreen is enabled by wry (it calls
    `setElementFullscreenEnabled(true)`); WebKitGTK enters fullscreen by default; **WebView2
    needs our own `ContainsFullScreenElementChanged` handler** (wry 0.57 has none) — LJ5.
    Electron would handle all three uniformly; this is the main cost of the choice.
  - *Codecs:* WebView2 (Edge) plays H.264/AAC/VP9/AV1; macOS WKWebView plays H.264/HEVC/VP9
    (Safari's set); **WebKitGTK depends on the system's GStreamer plugins**, so the `.deb`
    depends on `gstreamer1.0-plugins-good`, `-plugins-bad`, and `gstreamer1.0-libav`, and the
    AppImage bundles the media framework. Electron ships H.264/AAC/VP9/AV1 everywhere. No DRM
    (Widevine) in either without extra work — out of scope.
  - *Global shortcut:* `tauri-plugin-global-shortcut` (global-hotkey crate) works on Windows,
    macOS, and **Linux X11 only** (it uses x11rb). On Wayland, `little-joan --toggle` bound in the
    desktop's own shortcut settings is the fallback (single-instance forwards it). Electron is
    also X11-first on Linux.
  - *Tray / menu bar:* built into Tauri 2 (`tray-icon` feature). On Linux it needs
    libayatana-appindicator; GNOME needs the AppIndicator extension to show it.
  - *Trade-offs accepted:* three engines means three rendering/compat surfaces (Safari-level
    compat on macOS, WebKitGTK on Linux); Windows 10 needs the WebView2 runtime (preinstalled on
    Windows 11; the installer bootstraps it); remote pages can detect Tauri's injected IPC
    object even though every call is denied (Ground Rule 10).
- **Closing discards the page.** The window and its webview are destroyed on close, so the
  engine's renderer processes exit and memory returns to the OS. The next summon starts at
  the empty field. Cookies and site storage persist in the webview's data store (so logins
  survive). Keeping the page is listed under Later.
- **Toggle semantics:** shortcut while hibernating → create the window and focus the field;
  shortcut while the window is open and focused → close it (hibernate); open but not focused →
  focus it. `--show [url]`, `--toggle`, `--hide` on the command line do the same through
  single-instance.
- **The start field is a local page in the same webview.** Enter → Rust normalizes the input
  (URL or search) and navigates the webview to it. Ctrl/Cmd+L returns to the start page with the
  current URL pre-filled (the page is left; Esc goes back to it).
- **Search:** Google by default; DuckDuckGo, Bing, Brave selectable. **Translate:** Google
  Translate's proxy, `https://translate.google.com/translate?sl=auto&tl=<lang>&hl=<lang>&u=<url>`
  (confirmed to 302 to `<host>.translate.goog`). Default target language `en`.
- **New-window requests** (`target=_blank`, `window.open`) open in the system default browser
  (http/https only) and are denied inside Little Joan — the Little Arc analogue of "open in the
  main browser".
- **Default shortcut `CommandOrControl+Shift+Space`** (Cmd+Shift+Space on macOS,
  Ctrl+Shift+Space elsewhere): free by default on all three desktops. Known app-level
  conflicts (VS Code parameter hints, Word's non-breaking space) are documented; it is
  configurable.
- **Window:** native decorations (title bar shows the page title, close button = the way to
  close), resizable, default 480×640 logical px, min 320×300, opens centered on the monitor
  under the cursor, remembers its last size. Not always-on-top.
- **macOS:** `ActivationPolicy::Accessory` (menu-bar icon, no Dock icon). A main menu with
  Edit (so Cmd+C/V/X/A work in WKWebView) plus page actions as key equivalents. Cmd+Q quits
  for real (standard); Cmd+W / Esc on the start page / the shortcut only hibernate.
- **Identifier** `io.github.memoliertorerr.littlejoan`, product name `Little Joan`, binary
  `little-joan`, version 0.1.0, MIT license.

### v0.2 (decided with the user, 2026-10-17)

- **Scope and order:** start at login + first-run welcome → Linux fullscreen → downloads + zoom
  → Wayland shortcut → macOS signing → release. All four feature groups were chosen.
- **Start at login:** `tauri-plugin-autostart`, used from Rust only (none of its JS permissions
  are granted). macOS uses its LaunchAgent launcher (no AppleScript, so no Automation prompt).
  **The OS login entry is the only state** — `settings.json` has no field for it, so removing
  the entry in the OS's own settings shows up in Little Joan's Settings. Off by default; turned
  on in Settings, from the welcome, or with `--start-at-login on|off`.
- **First launch:** when `settings.json` doesn't exist yet, the window opens **once** with a
  welcome on the start page (the shortcut, where the icon is, a start-at-login checkbox, the
  Wayland `--toggle` hint). Default settings are saved at once, so it never shows twice. Every
  later launch hibernates, as in v0.1.
- **Ubuntu 22.04 fullscreen:** build the AppImage on ubuntu-24.04 (it bundles WebKitGTK 2.52;
  its glibc floor rises to 2.39, so 22.04 and Debian 12 use the `.deb`), and add a stopgap for
  WebKitGTK older than 2.52: decline element fullscreen and fullscreen the window instead —
  only if a 22.04 CI run first shows that this avoids the abort. Draft a Launchpad report for
  the user to file.
- **Downloads:** to the OS Downloads folder under a name that never overwrites a file; the URL
  bar shows the status and "Show in folder". No notification plugin. Closing the window during a
  download cancels it — no webview stays alive for it (Ground Rule 9).
- **Zoom:** Ctrl/Cmd + / − / 0, caught natively like the other page keys, applied with
  `Webview::set_zoom`. Per window, reset on the next summon, not saved. Never Tauri's
  `zoom_hotkeys_enabled` (Landmines).
- **Wayland shortcut:** the XDG GlobalShortcuts portal through `ashpd` when the session is
  Wayland and the portal offers it; otherwise v0.1's behavior and the `--toggle` hint.
- **Signing:** macOS — the user has an Apple Developer account: Developer ID signing and
  notarization in CI from the user's secrets, ad-hoc when they're missing. Windows — unsigned
  for v0.2; the user applies to SignPath Foundation (free for open source), and a step is added
  if it's approved. Linux — nothing to sign; `SHA256SUMS.txt` as before.

### v0.2 round 2 (decided 2026-10-17; the user said "you decide, go ahead")

The user's words: look and feel very similar to Little Arc; when first opened, a mix of the
most frequent sites; logins survive; history; where Little Arc shows "Contact the Team", a way
into advanced settings; themes (1–3 muted colors that blend, like Arc's); an extension system,
documented as experimental so others can write extensions later; bundled extensions —
useful, productive, fun — including Snake. Decisions made from that, with research
(Arc's help center via search snippets, Firefox/Chrome ranking docs, Figma/VS Code/Raycast
extension models) and a read of the vendored Tauri/wry sources:

- **Order:** these phases run before phases 7–10. The user lets the agent merge each phase PR
  once CI is green; tags and releases still need the user's word (`v0.2.0-beta.1` was approved).
- **Copy the UX patterns, not the brand.** A command bar overlay with suggestion rows ("Title —
  domain", favicon, a highlighted row), a slim toolbar, theme gradients. Never Arc's name, logo,
  icons or artwork (Ground Rule 12).
- **Two webviews, two trust levels.** `joan-bar` holds every trusted screen and becomes an
  *overlay* covering the whole window while the command bar or history is open (the page
  webview is hidden meanwhile); otherwise it is the 40 px toolbar. `joan-page` shows web pages
  and extension panels and has no IPC at all (Ground Rule 18). The phase-6 welcome moves from
  the start page into the command bar's empty state.
- **On summon the command bar is open,** like Little Arc: empty state = the welcome (first run
  only), then up to 8 top sites by frecency; with no history yet, 4 generic starters (Wikipedia,
  YouTube, GitHub, Google Maps). Typing: a quick answer if one applies, the best history match
  (autofill only on a host prefix), "Search <engine> for …", more history matches, matching
  commands. The last row, where Little Arc has "Contact the Team", is **"Settings &
  extensions"**. Esc on an empty command bar with no page closes the window (as Esc on the
  start page did); with a page, it returns to the page.
- **Frecency** (testable with a fixed clock): per host (`www.` stripped), over the last 20
  visits within 90 days, score = Σ w · 2^(−age / 14 days), w = 2 for a visit the user typed or
  picked, 1 for a link, 0 for reloads and redirects. Typed matching: every token must match;
  points host prefix 10, host word boundary 8, title word prefix 6, path word boundary 4,
  substring 1, times the frecency. No remote search suggestions (Later).
- **History store:** JSON in the config folder (`history.json`), one record per URL (title,
  host, visit count, last 20 visit times and kinds), capped at 5,000 URLs (least frecent
  dropped), saved atomically like settings, loaded when the window opens and dropped when it
  closes. Never recorded: the app's own pages, `about:`, `ljext:`, and anything when the
  setting "Remember history" (default on) is off. No SQLite: one small file matches the
  project's footprint.
- **Logins:** persistent cookies already live in each engine's persistent store; CI proves it
  (LJ20). Session cookies are kept in memory across close → summon within one app run
  (WebView2 otherwise drops them on every close), never written to disk; quitting forgets them,
  as browsers do. On macOS, Quit waits for the page to unload before exiting, so fresh cookies
  reach disk.
- **Toolbar** (LJ24): favicon + domain (click = command bar with the full address selected), a
  back button when there is somewhere to go back to, and an "open in default browser" button —
  Little Arc's "Open in…" analogue. macOS: the traffic lights sit inside the toolbar
  (`TitleBarStyle::Overlay`, drag region on the empty toolbar). Windows and Linux keep the
  native title bar for now (custom chrome is Later). This replaces LJ10.1's "no buttons in the
  bar" rule.
- **Themes:** settings `theme = { mode: system|light|dark, colors: 1–3 hex, intensity 0–1,
  grain 0–1 }`, default a neutral preset. All color math in TypeScript (`ui/lib/theme.ts`,
  unit-tested): picks → OKLCH, lightness clamped (light 0.86–0.93, dark 0.22–0.32), chroma capped
  at 0.12 × intensity, written as sRGB CSS variables; gradients layered (a base linear gradient
  plus soft radial blobs for colors 2 and 3) with an SVG `feTurbulence` grain overlay. Applied
  to the toolbar, the command bar, history, settings, and handed to extensions. Presets: about
  10 muted palettes. Editor: one hue/chroma pad per color (Arc-style dot on a grid), up to
  three colors, intensity and grain sliders, mode buttons.
- **Extensions (experimental):** a folder with `extension.json` (id, name, version,
  description, author, icon, `apiVersion: 1`, permissions, contributes: `commands` that open a
  panel, `pageActions` that open a panel with the current page's URL and title) and web files.
  Bundled ones live in `extensions/<id>/` and are embedded at build time; with "Developer mode"
  on (Settings → Extensions, off by default) unpacked folders in `<config>/extensions/` load
  too. They run as top-level documents at `ljext:` URLs in `joan-page` (Ground Rule 19), with
  a CSP built from their permissions (no network unless declared hosts), and use a small API
  served by Rust on the same scheme (`storage` with a quota, `theme`, `pageInfo`, `openUrl`,
  `clipboard`, `close`), each call checked against the requesting webview, the page's current
  URL and the manifest. `little-joan --extension <id>` opens one (also handy for key bindings).
- **Quick answers** (calculator, unit and time-zone conversion, random / UUID / password,
  timestamps) are built in: our own TypeScript in the bar, listed under Extensions as
  "Built-in" and switchable off. Third-party quick-answer providers are Later (they would need
  code running on every keystroke).
- **Bundled extensions:** Snake, 2048, Notes, Focus timer, World clock, QR code (page action),
  Clean link (page action), Wayback Machine (page action), Copy as Markdown link (page action),
  JSON formatter, Text tools, Color tools. All local; Wayback only opens web.archive.org in the
  page.

### Round 3 (decided with the owner, 2026-10-18)

The owner's decisions for the rest of v0.2.0 and for v0.3, after the beta.2 checklist. Do not
relitigate them mid-plan; a step that seems to need a change is a stop condition.

- **Order:** phase 18 (fixes) → 19 (responsive resizing) → 20 (control) → round 1's phases 7–11
  (LJ13–LJ19; LJ19 is the v0.2.0 release) → v0.3's phases 21–26. Steps are numbered on from LJ36,
  phases from 18; LJ13–LJ19 keep their numbers.
- **Fixes (phase 18):** (a) YouTube's own fullscreen button on macOS — reproduce in CI first (a
  smoke page calling both `requestFullscreen()` and `video.webkitEnterFullscreen()` from a click,
  at narrow and wide sizes), then fix. **Confirmed by the owner: it was the phone site**
  (m.youtube.com under phone mode's iPhone agent; desktop YouTube works). So, decided: on WebKit
  (macOS and Linux) youtube.com and its subdomains use the desktop agent by default, through a
  small per-site table (`siteModes`) the owner can override, with the m. → www rewrite on reload;
  and on macOS layout and phone-mode switches are skipped while WebKit's fullscreen is on (wry's
  `set_bounds` would otherwise resize the webview inside WebKit's fullscreen window). Phase 18
  ships the table; phase 19 gives it its UI; (b) the history key is **Ctrl+H** on Windows and Linux,
  Cmd+Y stays on macOS; (c) macOS: the toolbar left blank when an extension opens from
  hibernation — the panel opens once the bar's script runs; (d) double-clicking the toolbar zooms
  the window (`core:window:allow-internal-toggle-maximize`, the bar only); (e) no default-theme
  flash on summon — the bar gets its theme before its first paint (the trusted bar only, never
  the page webview).
- **Responsive resizing (phase 19):** the owner: "Little Arc has no mobile toggle; I like a mobile
  version on narrow sizes, but outside of that websites should be more responsive; optimize or
  better the resizing." Decided from the research (2026-10-18):
  - **Like Little Arc, pages reflow live:** no scaling, no zoom-to-fit, no viewport emulation (the
    desktop engines ignore `<meta viewport>`; only Windows could emulate one, through CDP). Phone
    mode stays for narrow windows: the phone agent below 600 px, the desktop agent again from
    640 px (hysteresis), no tablet range.
  - **Smooth resizing per OS:** only changed layout calls (no `set_bounds` + `show()` on both
    webviews per event); macOS autoresizing masks per mode so AppKit resizes the webviews in its
    own pass (tao's `Resized` comes a frame late); layout and agent switches never during WebKit's
    fullscreen; both webviews get the theme's background (Windows' white gutter); Linux no
    `show_all` per event; never Tauri's `auto_resize` (it would grow the 40 px bar).
  - **Phone mode per site:** auto / phone / desktop per host (`siteModes`, from LJ36), a
    command-bar command ("Request desktop site" / "Request phone site") and a small toolbar pill,
    a Settings list; a reload only when the agent really changes; known mobile hosts rewritten
    when going back to desktop (m.youtube.com → www.youtube.com, *.m.wikipedia.org →
    *.wikipedia.org).
  - **Windows' phone agent with client hints:** CDP `Emulation.setUserAgentOverride` with
    metadata (Android, mobile) instead of a bare custom agent, which disables UA client hints;
    SetUserAgent stays the fallback; and Windows skips the reload when CDP's
    `Page.getLayoutMetrics` shows the page already fits. Measuring a page on macOS / Linux would
    need a script in the page (Ground Rule 10): not planned — Later, the owner's decision.
  - Phase 19 ends in a checkpoint: the feel is the owner's to judge.
- **Control (phase 20):** (a) an option to hide the tray / menu-bar icon (Settings → General); it
  comes back by itself whenever the global shortcut can't work; a second launch still opens the
  window; the command bar gets "Settings" and "Quit Little Joan". (b) Quick commands in the
  command bar for actions and settings, each showing its current state (LJ42 lists them). (c) A
  Keyboard tab: every Little Joan key configurable (focus address, history, translate, close,
  reload, back / forward, and later zoom and find), warnings for clashes with Little Joan's own
  keys and the global shortcut, a curated table of the OSes' reserved shortcuts, and "another app
  holds it" when the global shortcut can't be registered. Reading macOS's own enabled shortcuts
  (`com.apple.symbolichotkeys`, read-only) is decided in LJ44 by whether it is reliable; system
  settings are never written.
- **Round 1's phases after that:** zoom keys join the keymap (LJ16); LJ15's download status lives
  in the toolbar; LJ17's portal trigger shows in the Keyboard tab; LJ18 runs only if the owner's
  Apple secrets exist (`gh secret list`, names only) and is otherwise skipped and recorded; LJ19's
  version-bump PR may be merged by the agent, tagging and publishing wait for the owner's word, and
  it is a checkpoint with the owner's checklist.
- **v0.3 — links from other apps (phase 21, security-sensitive):** Little Joan can be the
  system's browser for http / https links: macOS `CFBundleURLTypes` (Tauri merges
  `src-tauri/Info.plist`) and `RunEvent::Opened`; Windows the registry's `Capabilities` /
  `URLAssociations` and `RegisteredApplications`, written per user by the NSIS installer's hooks,
  so it appears in Default apps; Linux the desktop file's `MimeType=x-scheme-handler/http;…`. A
  link opens in the Little Joan window. Settings gets a "Main browser" for handing pages on —
  Little Joan may itself be the default, so it never hands a link to itself — and a button that
  opens the OS's default-apps settings. Little Joan never changes defaults silently (Ground Rule
  21).
- **v0.3 — sign-in popups (phase 22, security-sensitive):** `window.open` / OAuth popups (sized
  ones, or known sign-in flows) open in a small child window with its own IPC-free page webview
  that shares the data store and the opener relationship, so the popup can post back and close;
  they close on hibernate; everything else still goes to the browser.
- **v0.3 — find in page (phase 23):** Ctrl/Cmd+F → a find field in the toolbar (the trusted bar)
  driving each engine's own find — WKWebView's find, WebView2's Find API (webview2-com 0.39.1 has
  it: checked while planning), WebKitGTK's FindController; next / previous, the match count where
  the engine reports it; the key lives in the keymap.
- **v0.3 — saved logins on Windows (phase 24, security-sensitive):** a Privacy setting for
  WebView2's own password saving (`IsPasswordAutosaveEnabled`, default off) and general autofill,
  plus removing saved passwords; macOS and Linux say it isn't available yet. **macOS passwords and
  passkeys are deferred by the owner** (Later, with the research). Windows passkeys through
  Windows Hello are a human check.
- **v0.3 — more bundled extensions (phase 25):** Reading list (a page action saves the page; a
  panel lists, opens, removes), Stopwatch & countdown, To-do, Minesweeper, Password generator,
  Regex tester, Hash & Base64 (SubtleCrypto), Emoji picker, Breathing timer — the conventions of
  LJ32–LJ34 (plain JS, pure logic tested, themed, no network).
- **Additions the owner allowed, chosen while planning:** **pinned sites** (LJ50 — pin the current
  site to the top of the command bar's empty state; small, and it rounds off the command bar the
  quick commands extend); **panels follow theme changes** and **`--extension` opens a page action**
  (LJ56 — two Deferred follow-ups that make the new extensions better at little cost); **downloads
  refused while the page shows an extension panel or in a popup** (LJ15 / LJ47 — extensions have
  no downloads permission, so they must not drop files). Bigger ideas went to Later.
- **v0.3 ends with a checkpoint** (LJ57): docs, screenshots, the owner's checklist; a v0.3
  pre-release only on the owner's word.
- **Checkpoints and merging (the owner's update):** checkpoints stop for the owner's own testing
  and review — the end of phase 19, LJ19 and LJ57. The agent's merge permission is no wider than
  before (Ground Rule 24).

---

## Steps

### LJ1 — Scaffold: toolchain, Tauri 2 + Vite + TS, icons, hibernating tray app, CI · strong · auto

**Verify:** `source scripts/env.sh && npm run verify` green on the Mac (tsc, eslint, vitest with
at least one test of the icon geometry helper, `vite build`, `cargo fmt --check`, clippy
`-D warnings`, `cargo test`); after push, the `verify` workflow is green on ubuntu-latest,
windows-latest, and macos-latest; `src-tauri/tauri.conf.json` has `"windows": []`.
**Commit:** `lj-step-1: scaffold tauri 2 app that starts hibernating in the tray, CI verify`

```text
Create the project skeleton. Nothing user-visible beyond a tray icon yet: the app must start
with NO window and NO webview (hibernation is the product — Ground Rule 9).

1. Environment (already present from bootstrap research; keep and commit): scripts/env.sh,
   scripts/setup-rust.sh, rust-toolchain.toml (channel 1.99.1, minimal profile, rustfmt +
   clippy). Run `source scripts/env.sh && bash scripts/setup-rust.sh` if .cache/rustup is
   missing.

2. Node side (package.json, no "type": "module"; config files are .mts / .mjs):
   - devDependencies, pinned exactly: typescript 6.0.3 (NOT 7.x — typescript-eslint supports
     TS < 6.1), vite ^8.3.4, vitest ^5.0.4, eslint ^10.12.0, @eslint/js ^10.0.1,
     typescript-eslint ^8.71.1, @tauri-apps/cli ^2.12.3, @types/node ^24.
     dependencies: @tauri-apps/api ^2.12.4 (bundled into the UI by Vite).
   - Scripts: "dev": "tauri dev", "build:ui": "vite build", "typecheck": "tsc --noEmit -p .",
     "lint": "eslint .", "test": "vitest run", "icons": "node scripts/make-icons.mts && tauri icon
     .cache/icons/app-icon.png", "verify": typecheck, lint, test, build:ui, then
     `cargo fmt --manifest-path src-tauri/Cargo.toml --check`,
     `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`,
     `cargo test --manifest-path src-tauri/Cargo.toml` — in that order, stopping at the first
     failure. (UI build first: tauri::generate_context! embeds dist/ at compile time.)
   - tsconfig.json: strict, noUncheckedIndexedAccess, module ESNext, moduleResolution bundler,
     target ES2022, lib DOM + ES2022, includes ui/, scripts/, tests/. Node 24 runs .mts scripts
     directly (type stripping), so scripts use only erasable TS syntax (no enums/namespaces).
   - vite.config.mts: root "ui", build.outDir "../dist", emptyOutDir, multi-page input
     (ui/index.html now; ui/settings.html arrives in LJ6), server port 1420 strictPort,
     clearScreen false.
   - vitest.config.mts: include ["ui/**/*.test.ts", "scripts/**/*.test.ts", "tests/**/*.test.ts"].
   - eslint.config.mjs: @eslint/js recommended + typescript-eslint recommendedTypeChecked,
     ignores dist, .cache, src-tauri, node_modules.
   - ui/index.html + ui/start.ts + ui/style.css: a placeholder start page (heading "Little
     Joan", an input with placeholder "Search or type a URL"). LJ3 wires it up. CSP is set in
     tauri.conf.json, not in the HTML.

3. Icons: scripts/make-icons.mts draws ORIGINAL artwork with a dependency-free PNG encoder
   (node:zlib deflate + CRC32): a 1024×1024 app icon (rounded square, a small window
   silhouette with a dot — no letters, nothing resembling Arc's logo) to
   .cache/icons/app-icon.png, and tray icons committed to src-tauri/icons/tray.png (32×32) and
   tray@2x.png (64×64), black on transparent so macOS can use them as template images. Put the
   pixel geometry in scripts/icon-geometry.mts (pure functions) with a vitest test
   (scripts/icon-geometry.test.ts) that checks e.g. corners are transparent and the center is
   opaque. Run `npm run icons` and commit the generated src-tauri/icons/*.

4. Rust side (src-tauri/):
   - Cargo.toml: package little-joan 0.1.0, edition 2024, license MIT, lib name
     little_joan_lib (crate-type rlib) + main.rs calling little_joan_lib::run(). Dependencies:
     tauri 2.12 (features tray-icon, image-png), serde 1 (derive), serde_json 1;
     build-dependencies tauri-build 2.7. [profile.dev] debug = false (disk). [profile.release]
     codegen-units = 1, lto = true, opt-level = "s", panic = "abort", strip = true.
   - build.rs: tauri_build::build().
   - tauri.conf.json: productName "Little Joan", mainBinaryName "little-joan", version 0.1.0,
     identifier io.github.memoliertorerr.littlejoan, build.frontendDist "../dist",
     build.devUrl http://localhost:1420, beforeDevCommand "npx vite", beforeBuildCommand
     "npm run build:ui", app.windows [] (NO windows at startup), app.security.csp
     "default-src 'self'; style-src 'self'; img-src 'self' data:", bundle.active true,
     bundle.icon [icons/32x32.png, icons/128x128.png, icons/128x128@2x.png, icons/icon.icns,
     icons/icon.ico].
   - src/lib.rs: run(): tauri::Builder::default().setup(...) that (a) on macOS sets
     ActivationPolicy::Accessory, (b) builds the tray (src/tray.rs): icon from
     include_bytes!("../icons/tray.png") with icon_as_template(true), tooltip "Little Joan",
     menu with one item "Quit Little Joan" → app.exit(0). Then .build(...) and .run() with a
     RunEvent handler: ExitRequested { code: None, api } → api.prevent_exit() (closing the
     last window must never quit; only Quit does).
   - capabilities/: none needed yet (no windows). If tauri-build demands a capabilities dir,
     create capabilities/.gitkeep — do not add permissions that aren't used.

5. CI: .github/workflows/ci.yml, workflow "ci", on pull_request (all bases), push to main,
   workflow_dispatch. Job "verify", matrix os [ubuntu-latest, windows-latest, macos-latest],
   fail-fast false, defaults shell bash. Steps: checkout; setup-node 24 with npm cache; on
   Linux apt-get install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev
   libxdo-dev libssl-dev build-essential; `source scripts/env.sh` then `npm ci`; cache
   .cache/cargo/registry, .cache/cargo/git, .cache/target keyed on OS + Cargo.lock
   (actions/cache); `source scripts/env.sh && npm run verify`. The runner's preinstalled
   rustup installs the pinned toolchain into .cache/rustup on first use (env.sh sets
   RUSTUP_HOME); cache .cache/rustup too.

6. docs/architecture.md: process model (one Tauri process; no window at rest), file map, how
   to build (`source scripts/env.sh` …), and why the GUI is never launched locally. README:
   keep the stub, add "Build from source" pointing at architecture.md.

Do NOT create any window, webview, global shortcut, or settings file in this step — LJ3 owns
the window lifecycle; adding it here would mix two reviews.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ1 to ✅ in
docs/BUILD_PLAN.md and commit everything as one commit: "lj-step-1: scaffold tauri 2 app that
starts hibernating in the tray, CI verify". Push, open the phase-1 draft PR into main, and
watch CI on all three OSes. Paste the verify output, `git log -1 --stat`, and the CI result. If
it is still red after two honest fix attempts, stop and report — never commit red, never
weaken a check to get green.
```

**Implementation notes (2026-10-16):**

- **Disk:** 15 GB free at the start of the step (the 9.9 GB seen at planning time had grown);
  after the Rust toolchain (470 MB), the crate registry (~800 MB, includes the Windows/Linux
  crates fetched during research), node_modules (100 MB), and the first clippy + test build
  (`.cache/target` 1.0 GB), 14 GB free.
- **Tray icon names differ from the prompt:** the macOS template images are
  `src-tauri/icons/tray-template.png` / `tray-template@2x.png`; Windows and Linux use the
  full-colour `icons/32x32.png` instead of a black glyph, because a black icon disappears on
  Windows 11's dark taskbar.
- **`tauri icon` also emits Android, iOS, and Windows Store icons.** `npm run icons` now writes
  them to `.cache/icons/generated` and `scripts/copy-icons.mts` copies only the desktop set.
- **No `capabilities/` directory yet:** tauri-build 2.7 compiles without one, so none was
  created (LJ3 adds the first capability with the first window).
- Resolved versions: vitest 5.0.5, tauri 2.12.4, wry 0.57.0, tao 0.37.1.

---

### LJ2 — Pure core: input, navigation policy, translate URL, settings, toggle, CLI args · strong · auto

**Verify:** `npm run verify` green, with Rust unit tests that prove at least: `example.com` →
`https://example.com/`; `localhost:3000` → `http://localhost:3000/`; `what is rust` → the
selected engine's search URL with the query percent-encoded; `javascript:alert(1)` and
`file:///etc/passwd` are never returned as navigable URLs (they become searches);
`is_allowed_navigation` refuses `file:`, `javascript:`, and `ms-settings:`; the translate URL
for `https://example.com/a?b=1` with `tr` equals
`https://translate.google.com/translate?sl=auto&tl=tr&hl=tr&u=https%3A%2F%2Fexample.com%2Fa%3Fb%3D1`;
translating a `*.translate.goog` or non-http URL returns `None`; corrupt or partial
settings JSON loads as defaults field by field; the toggle table matches Design Decisions;
`--show https://x.test --toggle` parses as documented. CI green on 3 OSes.
**Commit:** `lj-step-2: pure core — input normalization, navigation policy, translate, settings, toggle, cli`

```text
Write the logic every later step relies on, as plain Rust modules with NO tauri imports, each
with unit tests in the same file (#[cfg(test)] mod tests). This is the contract layer: a wrong
shape here ripples through every later step, so keep the APIs small and documented.

Add dependency: url 2 (only this one).

1. src-tauri/src/input.rs — `pub fn normalize(input: &str, engine: SearchEngine) ->
   Option<Url>`. Rules, in order: trim; empty → None; if it parses
   as an absolute URL with scheme http or https → that URL; if it looks like a host
   (contains a dot or is "localhost", optional :port and /path, no spaces) → https:// prefix,
   except localhost / 127.0.0.1 / [::1] / *.localhost which get http://; everything else
   (including any other scheme such as javascript:, file:, data:, about:) → the engine's
   search URL with the whole trimmed input as the query. `SearchEngine` enum: Google
   (https://www.google.com/search?q=), DuckDuckGo (https://duckduckgo.com/?q=), Bing
   (https://www.bing.com/search?q=), Brave (https://search.brave.com/search?q=); serde
   lowercase names "google" | "duckduckgo" | "bing" | "brave".

2. src-tauri/src/navigation.rs — `pub fn is_allowed_navigation(url: &Url) -> bool`: allow
   http, https, about (about:blank / about:srcdoc for frames), blob, data (frames and
   engine-internal use), and the app's own origins (scheme "tauri", host tauri.localhost on
   http/https); refuse everything else (file, javascript, custom schemes). And
   `pub fn is_external_openable(url: &Url) -> bool`: true only for http/https — the only URLs
   ever handed to the OS default browser.

3. src-tauri/src/translate.rs — `pub fn translate_url(page: &Url, lang: &str) -> Option<Url>`:
   None for non-http(s) pages, for hosts ending in ".translate.goog", for
   translate.google.com itself, and for an invalid lang; else
   https://translate.google.com/translate?sl=auto&tl=<lang>&hl=<lang>&u=<page, encoded>.
   `pub fn is_valid_lang(lang: &str) -> bool`: ^[a-z]{2,3}(-[A-Za-z]{2,4})?$ (no regex crate —
   hand-written check). `pub const LANGUAGES: &[(&str, &str)]` — ~30 common (code, English
   name) pairs used by the settings UI (en, tr, de, fr, es, it, pt, nl, pl, ru, uk, ar, he, fa,
   hi, ja, ko, zh-CN, zh-TW, id, vi, th, sv, da, fi, no, cs, el, ro, hu).

4. src-tauri/src/settings.rs — `Settings { version: u32, shortcut: String, search_engine:
   SearchEngine, translate_to: String, window_width: u32, window_height: u32 }`, serde
   camelCase, Default = (1, "CommandOrControl+Shift+Space", Google, "en", 480, 640).
   `pub fn parse(json: &str) -> Settings` is tolerant per field: a missing, wrong-typed, or
   invalid field falls back to its default without discarding the others (deserialize into
   serde_json::Value and read each field); window sizes are clamped to 320..=4096 width and
   300..=4096 height; an invalid translate_to → "en". Shortcut strings are NOT validated here
   (that needs the hotkey parser — LJ3/LJ6). `pub fn load(dir: &Path) -> Settings` (missing or
   unreadable file → defaults) and `pub fn save(dir: &Path, s: &Settings) -> io::Result<()>`
   (create dir, write settings.json.tmp, rename over settings.json — atomic). Tests use a
   temp dir under std::env::temp_dir() (env.sh points TMPDIR into .cache/).

5. src-tauri/src/toggle.rs — `enum WindowState { Hibernating, Open { focused: bool } }`,
   `enum Command { Toggle, Show(Option<Url>), Hide }`, `enum Action { Create(Option<Url>),
   Focus, Navigate(Url), FocusAndNavigate(Url), Destroy, Nothing }`, and
   `pub fn decide(state, command) -> Action` implementing exactly: Toggle+Hibernating →
   Create(None); Toggle+Open{focused:true} → Destroy; Toggle+Open{focused:false} → Focus;
   Show(u)+Hibernating → Create(u); Show(None)+Open → Focus; Show(Some(u))+Open →
   FocusAndNavigate(u); Hide+Open → Destroy; Hide+Hibernating → Nothing. Test every row.

6. src-tauri/src/cli.rs — `pub fn parse_args(args: &[String]) -> Option<Command>`: skips
   argv[0]; "--toggle" → Toggle; "--hide" → Hide; "--show" optionally followed by a value
   that doesn't start with "--" → Show(normalize-as-URL-only: http/https absolute URLs, else
   None — never a search); no recognized flag → None; the last recognized flag wins. Unknown
   flags are ignored (macOS passes -psn_… on some launches).

7. lib.rs: declare the modules as `pub mod` (the crate is a library, so its public items are
   not dead code even before LJ3 calls them — no `allow(dead_code)` needed). Update
   docs/architecture.md with each module's one-line purpose and the toggle table.

Do NOT touch the tray, windows, or plugins — LJ3 wires these modules into the app.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ2 to ✅ in
docs/BUILD_PLAN.md and commit everything as one commit: "lj-step-2: pure core — input
normalization, navigation policy, translate, settings, toggle, cli". Push and watch CI. Paste
the verify output (test count), `git log -1 --stat`, and the CI result. If it is still red
after two honest fix attempts, stop and report — never commit red, never weaken a check to get
green.
This ends phase 1: push the branch, fill the PR description from the template, mark it ready
once CI is green, then continue with phase 2 only if the user's instructions say the build
runs end to end; otherwise stop.
```

**Implementation notes (2026-10-16):**

- **Disk:** 14 GB free at the start; `.cache/target` 1.0 GB after the step.
- **Search URLs are form-encoded** (`q=what+is+rust`, `%40` for `@`), not `%20`-encoded — that is
  what `url::Url::parse_with_params` produces and what every engine accepts.
- **`Url::parse("localhost:3000")` succeeds** (as scheme `localhost`), so `normalize` accepts a
  parsed URL only when its scheme is http/https and it has a host; everything else goes through
  the host heuristic (`host_of`), which rejects anything whose `:` isn't followed by a port.
- **IP literals get `http://`** (not only localhost): routers and local dev servers rarely serve
  TLS. A dotted word whose last label is all digits (`v1.2`) is a search, not a host.
- `settings::Settings::with_window_size(f64, f64)` was added for LJ3's save-on-close (clamps,
  rounds, tolerates NaN).
- 29 Rust unit tests; clippy clean on the first build.

---

### LJ3 — Summon / hibernate: global shortcut, Joan window, start page, CLI, smoke log · strong · auto

**Verify:** `npm run verify` green, including Rust tests for `centered_in(work_area, size)`
(window lands inside the work area, clamped when larger) and for the smoke-log line format;
a vitest test for `ui/start-logic.ts` (Enter sends trimmed input; Esc with `?u=` goes back,
without it hibernates); `tauri.conf.json` still has `"windows": []`; capabilities grant the
`joan` window only `allow-open-input`, `allow-hibernate`, `allow-start-info` and no remote
URLs. CI green on 3 OSes.
**Commit:** `lj-step-3: global shortcut summons the joan window; closing hibernates`

```text
Wire the pure core into a working summon → browse → hibernate loop.

Add dependencies: tauri-plugin-global-shortcut 2.4, tauri-plugin-single-instance 2.5 (only
these two).

1. src-tauri/src/state.rs — `AppState` in tauri::State via app.manage(Mutex<...>): current
   Settings, data dir, and the last shortcut registered. Data dir: env LITTLEJOAN_DATA_DIR if
   set, else app.path().app_config_dir(). (Shipped builds never set the variable.)

2. src-tauri/src/smoke.rs — when env LITTLEJOAN_SMOKE_LOG names a file, append one line per
   event: `<unix millis> <event> <detail>`; events: started, shortcut-registered <accel>,
   shortcut-failed <accel>, window-created, page-loaded <url>, title <title>, fullscreen
   <true|false>, hibernated. Otherwise do nothing. Pure `fn format_line(...)` tested. Never log
   when the variable is unset (no logging of browsing in normal use).

3. src-tauri/src/joan.rs — the window lifecycle. `apply(app, Action)`:
   - Create(url): WebviewWindowBuilder::new(app, "joan", WebviewUrl::App("index.html")),
     title "Little Joan", inner_size from Settings, min_inner_size 320×300, resizable,
     decorations, visible false then position via centered_in(monitor work area under the
     cursor — app.cursor_position() + app.monitor_from_point(); fall back to primary monitor,
     then to .center()), then show() + set_focus(). Data directory: <data dir>/webview
     (honored on Windows/Linux; macOS uses its own store). If url is Some, navigate to it
     right after creation.
   - Focus → show + unminimize + set_focus. Navigate / FocusAndNavigate → webview.navigate.
   - Destroy → save the current inner size (logical) into Settings (settings::save), then
     window.destroy(). Also handle WindowEvent::CloseRequested for "joan" the same way (save
     size, let it close) and WindowEvent::Destroyed → smoke "hibernated".
   - WindowState for toggle::decide comes from app.get_webview_window("joan") and
     is_focused().
   - `pub fn centered_in(work: Rect, size: Size) -> Position` — pure, tested.

4. src-tauri/src/shortcut.rs — register Settings.shortcut with the global-shortcut plugin
   (Builder::new().with_handler: on ShortcutState::Pressed → toggle::decide(state, Toggle) →
   joan::apply). If the stored shortcut fails to parse or register, log to stderr + smoke
   "shortcut-failed", and register the default instead; if that fails too, keep running (tray
   still works).

5. Single instance + CLI: tauri_plugin_single_instance::init(|app, argv, _cwd| …) registered
   FIRST among plugins; its callback runs cli::parse_args(argv) → decide → apply. On first
   launch, run the same on std::env::args() in setup. A second launch with no recognized flag
   behaves like --show.

6. Commands (src-tauri/src/commands.rs), restricted with build.rs
   tauri_build::Attributes::new().app_manifest(AppManifest::new().commands(&["open_input",
   "hibernate", "start_info"])), and capabilities/joan.json: windows ["joan"],
   permissions ["allow-open-input", "allow-hibernate", "allow-start-info"] (no core:default
   beyond what the page needs; no remote).
   - open_input(input: String) → input::normalize with the configured engine → navigate the
     calling webview; error string if empty.
   - hibernate() → joan::apply(Destroy).
   - start_info() → { shortcut: String, prefill: Option<String> } (prefill is read from the
     page's own ?u= query in TS, so this only carries the shortcut label).

7. UI: ui/start-logic.ts (pure, tested in ui/start-logic.test.ts) + ui/start.ts (DOM glue):
   input autofocused; Enter → invoke("open_input", { input }); Esc → if location.search has u
   → history.back(), else invoke("hibernate"); prefill from ?u= and select it. Hint line:
   "Enter to open · Esc to close · <shortcut label>". ui/style.css: system font, light/dark via
   prefers-color-scheme, the field fills the width, nothing else.

8. Docs: architecture.md (lifecycle diagram in text, commands and capabilities, smoke log
   format, CLI flags); README usage section (shortcut, flags incl. Wayland --toggle).

Do NOT add navigation guards, title handling, new-window handling, or keyboard handling inside
pages — LJ4. Do NOT add fullscreen handling — LJ5. Do NOT add a settings window — LJ6.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ3 to ✅ in
docs/BUILD_PLAN.md and commit everything as one commit: "lj-step-3: global shortcut summons
the joan window; closing hibernates". Push (open the phase-2 draft PR into lj/p1-foundation if
none exists) and watch CI. Paste the verify output, `git log -1 --stat`, and the CI result. If
it is still red after two honest fix attempts, stop and report — never commit red, never
weaken a check to get green.
```

**Implementation notes (2026-10-16):**

- **Disk:** 14 GB free at the start; `.cache/target` 1.4 GB after the step.
- **Webview data directory:** set only when `LITTLEJOAN_DATA_DIR` is set (development). The
  prompt said "always `<data dir>/webview`", but the data dir is the *config* folder, which on
  Windows is the roaming profile — browsing caches don't belong there. Shipped builds keep each
  engine's default location (WebView2: `%LOCALAPPDATA%\<identifier>\EBWebView`).
- **`start_info` returns `{ shortcut, platform }`;** the `?u=` prefill is read by the page
  itself. `ui/lib/shortcut.ts` (display formatting) was created here for the hint line; LJ6
  extends it with the recorder mapping.
- **CSP gained `connect-src 'self' ipc: http://ipc.localhost`:** tauri-codegen 2.7 doesn't add
  the IPC origins to a custom CSP, and without them IPC falls back to postMessage after a CSP
  violation.
- `get_window` is behind Tauri's `unstable` feature, so lifecycle code works with
  `get_webview_window` everywhere.
- **Not verified locally (Ground Rule 7):** that the window actually appears, takes focus, and
  closes — the app is launched for the first time in CI (LJ9) and on real desktops (LJ10).
  Local evidence: the ACL compiled to a `joan` capability that is `local: true` with exactly
  the three permissions.
- 37 Rust tests, 19 TS tests.

---

### LJ4 — Page behavior: navigation guard, titles, new windows, page keys · strong · auto

**Verify:** `npm run verify` green, including Rust tests for `keys::action_for(key, mods,
platform)`: Ctrl+W → Close, Ctrl+L → OpenLocation on Windows and Linux; Ctrl+R / F5 → Reload
and Alt+Left / Alt+Right → Back / Forward on Linux only (WebView2 handles those natively);
plain W, Ctrl+Shift+W, and AltGr-style Ctrl+Alt+W → None; macOS → None for everything (the
menu owns keys there); `open_location_url(current)` builds the start-page URL with `u`
percent-encoded. CI green on 3 OSes (this is the first step whose Windows/Linux code only CI
compiles).
**Commit:** `lj-step-4: page behavior — navigation guard, titles, new windows, page keys`

```text
Make the loaded page behave like a minimal browser without giving it any IPC.

Add dependencies: tauri-plugin-opener 2.7; on Windows (target cfg) webview2-com 0.39 and
windows 0.62 with features ["Win32_UI_Input_KeyboardAndMouse"] (exact versions MUST match the
ones wry uses — check Cargo.lock: webview2-com 0.39.x, windows 0.62.x); on Linux gtk 0.18
(matches tauri's). No others.

1. In joan.rs's builder:
   - .on_navigation(|url| navigation::is_allowed_navigation(url)).
   - .on_document_title_changed: set the window title to the page title (empty → "Little
     Joan"); smoke "title <title>".
   - .on_page_load (Finished): smoke "page-loaded <url>".
   - .on_new_window(|url, _| { if navigation::is_external_openable(&url) { open it with the
     opener plugin's Rust API (app.opener().open_url) } ; NewWindowResponse::Deny }).

2. src-tauri/src/keys.rs (pure, tested): `enum PageAction { Close, OpenLocation, Reload, Back,
   Forward }` (Translate and OpenInBrowser arrive in LJ7), `enum Platform { Windows, Linux,
   MacOs }`, `fn action_for(key: Key, mods: Mods, platform) -> Option<PageAction>` per the
   Verify line. `fn open_location_url(current: &Url) -> String` → "index.html?u=<encoded>"
   relative to the app origin.

3. src-tauri/src/page_actions.rs: `perform(app, action)`: Close → joan::apply(Destroy);
   OpenLocation → navigate the joan webview to the app's index.html?u=<current url> (use
   WebviewUrl::App resolution — on Windows the app origin is http://tauri.localhost, on macOS
   and Linux tauri://localhost; get it from the webview's url() when it is the start page or
   build it with tauri's helpers, and test the pure part); Reload → webview.reload(); Back /
   Forward → webview.eval("history.back()") / ("history.forward()").

4. Native key glue, installed when the joan window is created:
   - src-tauri/src/platform/windows.rs (#[cfg(windows)]): window.with_webview(|wv|
     controller = wv.controller(); controller.add_AcceleratorKeyPressed(handler)): on
     KEY_DOWN / SYSTEM_KEY_DOWN read VirtualKey, modifiers via GetKeyState(VK_CONTROL /
     VK_SHIFT / VK_MENU) high bit; if keys::action_for returns Some → args.SetHandled(true)
     and dispatch perform() via app.run_on_main_thread.
   - src-tauri/src/platform/linux.rs (#[cfg(target_os = "linux")]): window.gtk_window() →
     connect_key_press_event (runs before WebKit gets the key): map gdk keyval + state to
     keys::Key/Mods; Some(action) → perform, Propagation::Stop; else Proceed.
   - src-tauri/src/menu.rs (#[cfg(target_os = "macos")]): app menu with About, Quit
     (Cmd+Q); File: Close Window (Cmd+W → joan Destroy; it is the predefined close item only if
     that triggers CloseRequested — otherwise a custom item), Open Location… (Cmd+L); Edit:
     the predefined Undo, Redo, Cut, Copy, Paste, Select All (WKWebView needs these for
     Cmd+C/V); View: Reload (Cmd+R), Back (Cmd+[), Forward (Cmd+]). Menu events → perform().
     Set with app.set_menu in setup.

5. Docs: architecture.md (navigation policy, new-window rule, key table per OS, why native
   hooks instead of injected scripts); README keyboard table.

Do NOT inject any script into remote pages and do NOT grant any remote capability — native
hooks only (Ground Rule 10). Do NOT handle fullscreen — LJ5.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ4 to ✅ in
docs/BUILD_PLAN.md and commit everything as one commit: "lj-step-4: page behavior — navigation
guard, titles, new windows, page keys". Push and watch CI; Windows and Linux compile errors
are fixed per Ground Rule 3. Paste the verify output, `git log -1 --stat`, and the CI result.
If it is still red after two honest fix attempts, stop and report — never commit red, never
weaken a check to get green.
```

**Implementation notes (2026-10-16):**

- **Disk:** 14 GB free at the start; `.cache/target` ~1.5 GB after the step.
- **The opener is used as a plain function** (`tauri_plugin_opener::open_url`), not registered
  as a plugin: that would also register its JS commands, and nothing in the UI needs them.
- **Linux hook goes through `with_webview`:** `WebviewWindow::gtk_window()` hands back a GTK
  object on a non-main thread (window creation runs off the main thread, see LJ3), and GTK must
  only be touched on its main thread. `with_webview` runs on that thread; the hook is connected
  to `webview.inner().toplevel()` (the GTK window), so only the `gtk` crate is needed, not
  `webkit2gtk`.
- **`tauri://` origins are opaque** in the `url` crate, so `Url::origin()` comparisons never
  match; `keys::is_app_url` compares scheme, host, and port instead (test included).
- Windows' module is `platform/win.rs` (not `windows.rs`) so `use windows::…` always means the
  crate.
- **Not verified locally:** the Windows and Linux hooks only compile in CI (Ground Rule 3); key
  behavior on real desktops is in LJ10's checklist.
- 46 Rust tests, 19 TS tests.

---

### LJ5 — Fullscreen video · strong · auto

**Verify:** `npm run verify` green, including Rust tests for `FullscreenTracker` (element
enters → window goes fullscreen; element exits → window leaves fullscreen only if the tracker
put it there; a window the user made fullscreen themselves stays fullscreen when an element
exits); `tests/smoke/smoke.html` exists and sets `document.title` to
`LJ-SMOKE ready h264=<…> vp9=<…> av1=<…>` on load and `LJ-SMOKE fullscreen=on|off` on
`fullscreenchange`; a vitest test covers its pure helper (`tests/smoke/smoke-logic.ts`). CI
green on 3 OSes.
**Commit:** `lj-step-5: html5 element fullscreen fills the screen on all three engines`

```text
Make HTML5 element fullscreen (YouTube's fullscreen button, video.requestFullscreen()) fill the
screen, and give CI a page to prove it.

No new dependencies.

1. src-tauri/src/fullscreen.rs (pure, tested): `FullscreenTracker { entered_by_element:
   bool }` with `on_element_change(contains: bool, window_is_fullscreen: bool) ->
   Option<bool>` returning the fullscreen state to set (None = leave it alone), per the
   Verify line.

2. Windows (platform/windows.rs): in the same with_webview call as the key handler,
   core = controller.CoreWebView2(); core.add_ContainsFullScreenElementChanged(handler): read
   ContainsFullScreenElement (windows::core::BOOL), ask the tracker, and
   window.set_fullscreen(..) for the joan window. wry 0.57 has no such handler — without it
   WebView2 only fills the webview's bounds, not the screen. Smoke "fullscreen <bool>".

3. macOS: wry already calls setElementFullscreenEnabled(true) (macOS 12.3+), and WebKit moves
   the element into its own fullscreen window. No code; add a comment in joan.rs naming this.
   Linux: WebKitGTK's default enter-fullscreen handler fullscreens the toplevel GtkWindow
   (enable-fullscreen defaults to TRUE). No code; same comment.

4. All OSes: on WindowEvent::Resized for "joan", when smoke logging is on, log "fullscreen
   <bool>" whenever window.is_fullscreen() changes (so CI can see the outcome on every
   engine, not only Windows).

5. tests/smoke/smoke.html + tests/smoke/smoke.ts → built? NO: keep it a single static HTML
   file with an inline <script> (it is served by a throwaway HTTP server in CI, never bundled
   into the app). Its pure logic (building the title string from canPlayType /
   MediaSource.isTypeSupported results) lives in tests/smoke/smoke-logic.ts with a vitest
   test; copy the function body into the HTML by hand and add a vitest test that asserts the
   HTML contains the same function text, so they cannot drift. The page: one big button
   covering most of the viewport ("Go fullscreen") that calls
   document.getElementById("stage").requestFullscreen(); a <video> element inside #stage (no
   source needed); title updates as in the Verify line.

6. Docs: architecture.md fullscreen section (per-engine behavior); README "Fullscreen video"
   note incl. the Linux GStreamer packages.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ5 to ✅ in
docs/BUILD_PLAN.md and commit everything as one commit: "lj-step-5: html5 element fullscreen
fills the screen on all three engines". Push and watch CI. Paste the verify output,
`git log -1 --stat`, and the CI result. If it is still red after two honest fix attempts, stop
and report — never commit red, never weaken a check to get green.
This ends phase 2: push the branch, fill the PR description from the template, mark it ready
once CI is green, then stop (or continue to phase 3 if the user's instructions say the build
runs end to end).
```

**Implementation notes (2026-10-16):**

- **Disk:** 13 GB free at the start; `.cache/target` ~1.5 GB.
- **The fullscreen change is queued off the WebView2 callback** (spawned on the async runtime,
  so the window message is processed after the callback returns): resizing the window resizes
  the WebView2 controller, which would re-enter WebView2 from inside its own event handler.
  The tracker is shared through `Arc<Mutex<…>>` for that reason.
- **Two smoke events instead of one:** `element-fullscreen` (Windows: the app changed the
  window for the page) and `fullscreen` (every engine: the window's fullscreen state changed,
  logged from `Resized`). On macOS WebKit uses its own fullscreen window, so our window never
  reports fullscreen there; the page title (`LJ-SMOKE fullscreen=on`) is the macOS signal.
- **The smoke page's logic is a plain-JS module** (`tests/smoke/smoke-logic.mjs`, JSDoc-typed;
  `tsconfig.json` now has `allowJs` + `checkJs`) that `smoke.html` imports and vitest tests
  directly — instead of a hand-copied function plus a drift test. Same guarantee, no copy.
- The smoke page also uses the WebKit-prefixed fullscreen API when the standard one is missing
  (older WKWebView).
- **Not verified locally:** WebView2 behavior (CI compiles it; LJ9 clicks the button on a
  Windows runner) and real-video fullscreen on all three OSes (LJ10).
- 49 Rust tests, 25 TS tests.

---

### LJ6 — Settings window: shortcut recorder, search engine, translate language · standard · auto

**Verify:** `npm run verify` green, including vitest tests for `ui/lib/shortcut.ts`:
`{ctrlKey, shiftKey, code:"Space"}` on Windows → `"CommandOrControl+Shift+Space"`; the same
with metaKey on macOS → `"CommandOrControl+Shift+Space"`; ctrlKey on macOS → `"Control+…"`;
a lone modifier or a key without any modifier → `null` (a global shortcut needs a modifier,
except F13–F24); display text `⌘⇧Space` on macOS and `Ctrl+Shift+Space` elsewhere; and Rust
tests that `save_settings` rejects an unparseable shortcut and, when registration of the new
shortcut fails, re-registers the old one and returns an error (registration is behind a small
trait so the test uses a fake). CI green on 3 OSes.
**Commit:** `lj-step-6: settings window with live shortcut re-registration`

```text
Add the only other window: Settings, opened from the tray ("Settings…") and on macOS from the
app menu (Cmd+,). Same lifecycle rule as Joan: created on demand, destroyed on close.

No new dependencies.

1. UI: ui/settings.html + ui/settings.ts + ui/lib/shortcut.ts (pure, tested). Fields:
   Shortcut (a recorder: click → "Press keys…" → the first valid combo is captured; Esc
   cancels), Search engine (select: Google, DuckDuckGo, Bing, Brave), Translate to (select from
   the LANGUAGES list served by Rust). Save and Cancel buttons; errors shown inline in
   sentence case. While the recorder is listening, call suspend_shortcut(true) so pressing the
   current shortcut doesn't summon/dismiss Joan; always resume on blur/cancel/save. Add
   ui/settings.html to vite.config.mts inputs.

2. Rust: commands get_settings, save_settings(settings), suspend_shortcut(on: bool),
   languages() — in commands.rs; add them to build.rs's AppManifest and
   capabilities/settings.json (windows ["settings"], only these four allow- permissions).
   save_settings: validate (shortcut parses with
   tauri_plugin_global_shortcut::Shortcut::from_str; engine/lang via the LJ2 validators),
   unregister old → register new → on failure re-register old and return
   "That shortcut is taken by another app or the system." → on success persist with
   settings::save and update AppState. Put the swap logic in shortcut.rs behind a
   `trait Registrar { fn register(&self, s: &str) -> Result<(), String>; fn unregister(...) }`
   so it is unit-tested with a fake.

3. src-tauri/src/settings_window.rs: open (or focus if open) a 420×360 non-resizable window
   "settings" with WebviewUrl::App("settings.html"), centered like Joan.

4. Tray: add "Settings…" above Quit; macOS app menu: Settings… (Cmd+,).

5. Docs: README settings section; architecture.md (settings file location per OS, the
   suspend/resume rule).

Do NOT add translate actions — LJ7. Do NOT add a start-at-login option — it is in Later.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ6 to ✅ in
docs/BUILD_PLAN.md and commit everything as one commit: "lj-step-6: settings window with live
shortcut re-registration". Push (open the phase-3 draft PR into lj/p2-summon if none exists)
and watch CI. Paste the verify output, `git log -1 --stat`, and the CI result. If it is still
red after two honest fix attempts, stop and report — never commit red, never weaken a check to
get green.
```

**Implementation notes (2026-10-16):**

- **Disk:** 13 GB free at the start; `.cache/target` ~1.6 GB.
- **`languages()` folded into `get_settings`:** the window gets settings, engine list, language
  list, and platform in one call (three commands instead of four).
- **Save closes the window** through `core:window:allow-close` (granted only to `settings`);
  Cancel does the same.
- **Suspension is tracked in state** (`shortcut_suspended`) and undone on Save, on recorder
  stop, and on the window's `Destroyed` event — closing the window mid-recording can't leave the
  app without a shortcut.
- **Shift alone doesn't count as a modifier** in the recorder (it would steal capital letters);
  F13–F24 may be used bare.
- A Python edit of `build.rs` silently didn't apply (rustfmt had rewrapped the line it matched),
  which tauri-build caught at once ("Permission allow-get-settings not found"). Later scripted
  edits assert that the text they replace exists.
- **Not verified locally:** recording and swapping a real shortcut (LJ10).
- 54 Rust tests, 33 TS tests.

---

### LJ7 — Translate page + open in default browser · standard · auto

**Verify:** `npm run verify` green, including Rust tests that `keys::action_for` maps
Ctrl+Shift+T → Translate on Windows and Linux (and nothing on macOS, where the menu owns it),
and that `page_actions::translate_target(current, lang)` returns None for the start page and
for an already-translated page, and the Google proxy URL otherwise. CI green on 3 OSes.
**Commit:** `lj-step-7: translate the current page and open it in the default browser`

```text
Add the two page actions that reach outside the page.

No new dependencies.

1. keys.rs: PageAction::Translate (Ctrl+Shift+T on Windows/Linux) and
   PageAction::OpenInBrowser (no key — tray and macOS menu only). Tests per the Verify line.

2. page_actions.rs: Translate → translate::translate_url(current webview URL,
   settings.translate_to) → navigate the joan webview; None → do nothing (no error UI).
   OpenInBrowser → if navigation::is_external_openable(current) → opener open_url, then
   hibernate (the page moved to the main browser, like Little Arc's "open in Arc").

3. Tray menu (top to bottom): "Open Little Joan" (Show), "Translate Page", "Open Page in
   Browser", separator, "Settings…", "Quit Little Joan". The two page items are enabled only
   while a page (not the start page) is open — update enabled state on page load and on
   hibernate. macOS menu: File → Translate Page (Cmd+Shift+T), Open Page in Browser
   (Cmd+Shift+O).

4. Docs: README (translate: what it does, that the page goes through Google, how to pick the
   language), architecture.md key table.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ7 to ✅ in
docs/BUILD_PLAN.md and commit everything as one commit: "lj-step-7: translate the current page
and open it in the default browser". Push and watch CI. Paste the verify output,
`git log -1 --stat`, and the CI result. If it is still red after two honest fix attempts, stop
and report — never commit red, never weaken a check to get green.
This ends phase 3: push the branch, fill the PR description from the template, mark it ready
once CI is green, then stop (or continue to phase 4 if the user's instructions say the build
runs end to end).
```

**Implementation notes (2026-10-16):**

- **Disk:** 13 GB free at the start; `.cache/target` ~1.6 GB.
- **`translate_target` takes the app origin as a parameter** so the start-page check is
  testable on every OS: on Windows the start page is `http://tauri.localhost/…`, an http URL
  that `translate_url` alone would happily translate.
- **Tray items' enabled state** is kept in managed state (`PageItems<R>`), set on every
  finished page load (web page → enabled; start page → disabled) and cleared on hibernate.
  The macOS menu items stay enabled (the menu bar is hidden) and do nothing without a page.
- "Open Little Joan" was added at the top of the tray menu (it is the only way to summon the
  window where the global shortcut can't work, e.g. Wayland with a tray).
- **Not verified locally:** the Google redirect inside each webview and the tray menu on real
  desktops (LJ10).
- 58 Rust tests, 33 TS tests.

---

### LJ8 — Packaging in CI · strong · auto

**Verify:** `npm run verify` green locally (no local release build — Ground Rule 8); the CI
`package` job is green on all three OSes and uploads: `little-joan-windows` (NSIS `.exe`),
`little-joan-macos` (universal `.dmg`), `little-joan-linux` (`.deb` + `.AppImage`); the job
summary lists each artifact's size; `dpkg-deb -I` output in the log shows the GStreamer
dependencies.
**Commit:** `lj-step-8: build installers for windows, macos, and linux in CI`

```text
Build what users download, in CI only.

No new dependencies. This step may edit .github/workflows/ci.yml and src-tauri/tauri.conf.json.

1. tauri.conf.json bundle config:
   - Windows: targets nsis; nsis.installMode "currentUser" (no admin); webviewInstallMode
     downloadBootstrapper (Windows 10 without WebView2 gets it).
   - macOS: targets dmg + app; macOS.signingIdentity "-" (ad-hoc, so the arm64 binary runs);
     minimumSystemVersion "12.3" (element fullscreen API).
   - Linux: targets deb + appimage; deb.depends ["gstreamer1.0-plugins-good",
     "gstreamer1.0-plugins-bad", "gstreamer1.0-libav"]; appimage.bundleMediaFramework true.
   - category "Utility", shortDescription, longDescription, copyright "© 2026 Mehmet Ali
     Ertorer", license MIT.
   Pass --bundles per OS on the command line rather than listing every target globally.

2. ci.yml: job "package" needs verify, matrix:
   - windows-latest: `npx tauri build --bundles nsis`
   - macos-latest: `rustup target add x86_64-apple-darwin aarch64-apple-darwin` then
     `npx tauri build --target universal-apple-darwin --bundles app,dmg`
   - ubuntu-22.04 (older glibc → the AppImage/deb run on more distros): apt deps as in verify
     + `npx tauri build --bundles deb,appimage`; print `dpkg-deb -I` for the .deb.
   Always `source scripts/env.sh` first. Upload each OS's bundle files with
   actions/upload-artifact (retention 14 days) under the names in the Verify line, and write
   their sizes to $GITHUB_STEP_SUMMARY.

3. Record the artifact sizes and any bundler friction in the implementation notes and in
   Landmines. Docs: README "Install" (per OS, including Gatekeeper's "Open anyway" /
   `xattr -dr com.apple.quarantine` for the unsigned build, SmartScreen "More info → Run
   anyway", the Linux packages), architecture.md "Packaging".

Do NOT sign, notarize, or publish a GitHub Release — Later.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ8 to ✅ in
docs/BUILD_PLAN.md and commit everything as one commit: "lj-step-8: build installers for
windows, macos, and linux in CI". Push (open the phase-4 draft PR into lj/p3-settings if none
exists) and watch CI until package is green. Paste the verify output, `git log -1 --stat`, the
CI result, and the artifact sizes. If it is still red after two honest fix attempts, stop and
report — never commit red, never weaken a check to get green.
```

**Implementation notes (2026-10-16):**

- **Disk:** 13 GB free at the start. No local release build (Ground Rule 8): the bundle config
  was validated locally only by tauri-build parsing `tauri.conf.json` during clippy.
- **The CI target dir is cached,** so the build step deletes earlier `release/bundle` folders
  first; otherwise a stale installer from an older run could be uploaded.
- The macOS job also prints `lipo -archs` and `codesign -dv` for the `.app`, so the PR shows the
  universal binary and the ad-hoc signature.
- Bundle metadata added: publisher, copyright, MIT license, homepage, category Utility, short
  and long descriptions.
- **First CI run (run 38922147279, recorded with LJ9's commit): every package job green on the
  first try.** Sizes: Windows NSIS `Little Joan_0.1.0_x64-setup.exe` **1.21 MiB**; macOS
  `Little Joan_0.1.0_universal.dmg` **3.25 MiB** (the `.app` is 7.1 MiB, `x86_64 arm64`,
  `Signature=adhoc`); Linux `Little Joan_0.1.0_amd64.deb` **2.0 MiB** and
  `Little Joan_0.1.0_amd64.AppImage` **142 MiB** (bundled WebKitGTK + GStreamer). Package job
  times: Windows 308 s, Linux 309 s, macOS 455 s (universal = two compiles).
- **The bundler adds the runtime libraries to the `.deb` by itself:** `Depends:` lists our three
  GStreamer packages plus `libayatana-appindicator3-1, libwebkit2gtk-4.1-0, libgtk-3-0`.
- Installer file names use the product name with its space (`Little Joan_0.1.0_…`); the README
  quotes them.

---

### LJ9 — Install-and-run smoke in CI · strong · auto

**Verify:** `npm run verify` green (including vitest tests for `scripts/ci/smoke-log.mts`, the
log parser/assertions); CI `smoke` job green on all three OSes, where each job installs the
artifact the way a user would, and the smoke log shows, in order: `started`,
`shortcut-registered`, `window-created`, `title LJ-SMOKE ready …`, (Windows and Linux at
least) `fullscreen true` after the scripted click, `hibernated` after `--hide`; and after
hibernation no webview helper processes remain (WebView2 `msedgewebview2.exe` children on
Windows, `WebKitWebProcess` on Linux; on macOS the WebContent count returns to its pre-show
value). Screenshots `start.png`, `page.png`, `fullscreen.png` are uploaded per OS and the agent
has looked at them (downloaded to test-results/ci/, then deleted after inspection).
**Commit:** `lj-step-9: smoke-test the installed app on all three OSes in CI`

```text
Prove the downloadable artifacts install and run — launching the unpacked build is NOT
enough (a previous project's real installer was broken while the unpacked build passed).

No new npm or cargo dependencies. This step may edit .github/workflows/ci.yml. Apt packages
for the Linux job (xvfb, xdotool, imagemagick, dbus-x11, a light window manager such as
openbox for fullscreen) are fine — they live on the disposable runner.

1. scripts/ci/smoke.mts (Node 24, run directly): given the path of the installed binary and
   an output dir: start a throwaway HTTP server for tests/smoke/ on 127.0.0.1 (node:http),
   launch the app with LITTLEJOAN_SMOKE_LOG set, wait for "started"; record the app's memory
   (Windows: Get-Process WorkingSet64/PrivateMemorySize64; macOS/Linux: ps -o rss); run the
   binary again with `--show` (no URL) → wait for window-created → screenshot start.png; run
   `--show http://127.0.0.1:<port>/smoke.html` → wait for the LJ-SMOKE ready title →
   screenshot page.png; click the window center (Windows: PowerShell SetCursorPos +
   mouse_event; Linux: xdotool; macOS: try a tiny swift CGEvent click and record if the runner
   forbids it) → wait for "fullscreen true" (timeout → recorded, see Verify for which OSes
   must pass) → screenshot fullscreen.png; run `--hide` → wait for "hibernated" → wait 5 s →
   count webview helper processes and record memory again. Write a summary JSON. All
   timeouts are explicit (≤ 30 s each) and every failure prints the smoke log.
   scripts/ci/smoke-log.mts holds the pure log parsing/assertions (vitest-tested).

2. ci.yml job "smoke" needs package, matrix per OS, downloads that OS's artifact and installs
   it like a user:
   - Windows: run the NSIS installer silently (`/S`), launch from
     "$LOCALAPPDATA/Little Joan/little-joan.exe" (check the real install path in the log).
     Screenshots via PowerShell System.Drawing CopyFromScreen.
   - macOS: hdiutil attach the .dmg, copy "Little Joan.app" to a temp Applications folder,
     detach, launch Contents/MacOS/little-joan. Screenshots via `screencapture -x`.
   - Linux: `sudo apt-get install -y ./little-joan_*.deb` (pulls the GStreamer deps), launch
     /usr/bin/little-joan under `dbus-run-session -- xvfb-run -s "-screen 0 1280x800x24"` with
     openbox running (single-instance needs a session bus; fullscreen needs a WM). Screenshots
     via `import -window root`. Also run the AppImage with --appimage-extract-and-run to the
     "started" line.
   Upload test-results as artifact "smoke-<os>" (screenshots, smoke log, summary JSON).

3. Download each smoke artifact into test-results/ci/<os>/ (gitignored), look at every
   screenshot, describe what each shows in the implementation notes, record the memory
   numbers, then delete the downloaded files (disk rule). Anything the runner cannot do
   (e.g. macOS synthetic clicks) is recorded honestly and moved to the human checklist.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ9 to ✅ in
docs/BUILD_PLAN.md and commit everything as one commit: "lj-step-9: smoke-test the installed
app on all three OSes in CI". Push and watch CI until smoke is green. Paste the verify output,
`git log -1 --stat`, the CI result, and what each screenshot shows. If it is still red after
two honest fix attempts, stop and report — never commit red, never weaken a check to get green.
```

**Implementation notes (2026-10-16):**

- **Disk:** 13 GB free at the start. Nothing is built or run locally for this step beyond
  `npm run verify` (the harness's pure parts are unit-tested: 40 TS tests).
- **The smoke page reports through `document.title`,** which the app logs — so the harness
  needs no IPC, no devtools protocol, and no browser driver.
- **Second launches are the remote control:** `--show`, `--show <url>`, `--hide` are forwarded
  by single-instance; the harness also asserts exactly one `started` line, which proves the
  forwarding worked instead of a second app starting.
- **Helper processes are counted by name** (`msedgewebview2.exe`, `com.apple.WebKit.WebContent`,
  `WebKitWebProcess`); the check is "no more after hibernating than before showing".
- **macOS synthetic clicks may be refused** on the runner (Accessibility permission); the
  harness therefore requires fullscreen only on Windows and Linux and records what macOS did.

**Implementation notes, CI results (2026-10-16):**

- **Windows passed everything on the first run** (run 38923984196): installed by the NSIS
  installer to `%LOCALAPPDATA%\Little Joan\little-joan.exe`; hibernating 12.8 MB working set
  (2.2 MB private); 6 `msedgewebview2.exe` with the smoke page, **0 within 2 s of hibernating**,
  0 after each of three summon cycles; the click made the window fullscreen; H.264, VP9, and
  AV1 all report MSE support. Screenshots: the start page (field focused, hint
  "Enter to open · Esc to close · Ctrl+Shift+Space to summon"), the smoke page in the 480×640
  window, and the green fullscreen stage covering the whole screen including the taskbar.
- **macOS worked but leaked time:** the synthetic click was allowed on the runner and the
  element went fullscreen on its own Space (screenshot: green stage, full screen); but 3
  `WebContent` processes (plus GPU and Networking) were still alive 60–90 s after hibernating.
  They did not grow across cycles (`[3,3,2]`, `[3,0,0]`). **Fix (a25263a):** every close now
  goes through the hibernate path, and on macOS the window is hidden, navigated to
  `about:blank`, and destroyed 400 ms later. After: 1 helper at 2–10 s, **0 at 30 s**, `[0,0,0]`
  per cycle; own memory (physical footprint) 11 MB at rest, 23 MB after a page (WebKit's
  framework state stays loaded).
- **Linux, Ubuntu 22.04: the app aborts (SIGABRT inside `libwebkit2gtk-4.1.so.0` 2.50.4,
  from a RunLoop dispatch) the moment a page requests element fullscreen.** Ruled out with
  smoke-only reruns (`workflow_dispatch`, see architecture.md): the DMABUF renderer and
  compositing mode switches, the window manager (crashes with openbox and with none), and —
  decisively — **Little Joan itself: a bare PyGObject WebKitGTK window aborts the same way.**
  On **Ubuntu 24.04 (WebKitGTK 2.52.6) neither crashes** and both reach fullscreen (screenshot:
  green stage filling the 1280×800 screen). Symbols weren't obtainable (debuginfod downloads
  time out under gdb; Ubuntu's ddebs have no 2.50.4 symbols).
- **Decision taken (the user may revert it):** the Linux smoke installs the `.deb` (still built
  on 22.04) on **Ubuntu 24.04**, where it tests our code rather than a known engine abort; the
  22.04 finding is recorded under Landmines and in the human checklist. The checks themselves
  are unchanged.
- **Linux window-fullscreen detection was wrong in the harness/app:** WebKitGTK fullscreens the
  GTK window itself, which tao's `is_fullscreen()` doesn't track, so the smoke log never said
  `fullscreen true` even when the screenshot showed it. The app now reads the GDK window state
  on Linux (`platform::is_fullscreen`).
- **CI friction:** an Azure apt mirror once took 22 minutes to deliver WebKitGTK (smoke timeout
  raised to 45 min); a scripted `--hide` after the crash started new instances (that's how the
  "exactly one started" check caught it).
- Commits: c1682df (harness), d61a271, db058c8, 2b123b4, a25263a, ce384a6, 697fe74, e8b3fa1,
  2a1a7db, 977390b (`lj-step-9-fix: …`). **d61a271 also carried LJ10's README rewrite and
  part of its plan edits** (they were in the working tree when the diagnostics were committed).

---

### LJ10 — Docs + human checkpoint · standard · checkpoint

**Verify:** `npm run verify` green; README covers what it is, install per OS, usage,
shortcuts, settings, translate, privacy notes (Google Translate sees translated pages; search
goes to the chosen engine), build from source, "inspired by Little Arc" and the unofficial
note; docs/architecture.md matches the code's file map (every file under src-tauri/src and
ui/ is listed); the phase-4 PR description contains the full human checklist below; CI green.
**Human check:** the checklist in this step's prompt, copied into the PR.
**Commit:** `lj-step-10: readme, architecture, and the human checklist`

```text
Finish the docs and hand the parts only a person can check to the user.

No new dependencies.

1. README.md: complete it (sections listed in the Verify line). Keep it short and plain.
2. docs/architecture.md: reconcile with the code — file map, lifecycle, commands and
   capabilities, per-OS native glue, smoke log, packaging, CI jobs.
3. This plan: Overall status, Landmines from real friction, Deferred follow-ups.
4. The phase-4 PR description gets this checklist under "Needs human check" (fill in real
   artifact names and the CI run URL):
   Windows 10/11 (NSIS installer):
   - [ ] Install; SmartScreen → More info → Run anyway; a tray icon appears, no window.
   - [ ] Task Manager: one little-joan.exe, no msedgewebview2.exe while hibernating; note MB.
   - [ ] Ctrl+Shift+Space opens the window centered on the monitor under the cursor with the
         field focused; type "example.com" + Enter → the page loads and fills the window.
   - [ ] Resize the window; close it; summon again → it opens at the new size, field empty.
   - [ ] Ctrl+W, the close button, Esc on the empty field, and the shortcut while focused all
         hibernate; msedgewebview2.exe processes disappear within ~10 s.
   - [ ] YouTube: play a video, click its fullscreen button → fills the monitor; Esc → back.
   - [ ] Ctrl+L → start page with the URL selected; Esc → back to the page.
   - [ ] Tray → Translate Page on a non-English page → Google's translated view.
   - [ ] Settings: record a new shortcut, save; the old one stops working, the new one works;
         try a taken one (e.g. one another app owns) → error, old one still works.
   macOS 12.3+ (universal DMG):
   - [ ] Open the DMG, drag to Applications, first launch: right-click → Open (or
         `xattr -dr com.apple.quarantine "/Applications/Little Joan.app"`); menu-bar icon, no
         Dock icon.
   - [ ] Cmd+Shift+Space summons on the current Space, field focused; Cmd+C/Cmd+V work in the
         field and on pages.
   - [ ] YouTube fullscreen → own Space; Esc returns to the small window at its size.
   - [ ] Cmd+W / shortcut → hibernate; Activity Monitor: WebContent process for the page goes
         away; note little-joan's memory.
   - [ ] Cmd+L, Cmd+R, Cmd+[, Cmd+], Cmd+Shift+T, Cmd+, behave as documented; Cmd+Q quits.
   Linux (X11 session, then Wayland session; .deb on Ubuntu/Debian, AppImage elsewhere):
   - [ ] X11: Ctrl+Shift+Space summons; tray icon visible (GNOME needs the AppIndicator
         extension).
   - [ ] Wayland: the built-in shortcut does not fire (expected); bind `little-joan --toggle`
         in Settings → Keyboard → Custom Shortcuts → it summons/dismisses.
   - [ ] YouTube plays (GStreamer packages installed by the .deb) and fullscreen works.
   - [ ] Ctrl+W, Ctrl+L, Ctrl+R, Alt+Left/Right, Ctrl+Shift+T as documented.
   All:
   - [ ] target=_blank links open in the default browser, not in Little Joan.
   - [ ] Nothing regressed: phases 1–3 checklists still hold.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ10 to ✅ in
docs/BUILD_PLAN.md and commit everything as one commit: "lj-step-10: readme, architecture, and
the human checklist". Paste the verify output and `git log -1 --stat`. If it is still red after
two honest fix attempts, stop and report — never commit red, never weaken a check to get green.
This is a checkpoint and ends phase 4: push the branch, update the PR with the checklist, mark
it ready once CI is green, tell the user exactly what to check and where, then stop.
```

**Implementation notes (2026-10-16):**

- **Disk:** 14 GB free; the repo folder is 3.3 GB, of which 3.2 GB is `.cache/` (toolchain,
  registry, debug target, npm cache) and can be deleted.
- The README rewrite landed early, in d61a271 (see LJ9's notes); this commit finishes it (status
  line, the Linux fullscreen known issue) together with architecture.md (close routing, the
  macOS release step, CI diagnostics, measured memory) and this plan (Landmines, Deferred
  follow-ups, statuses).
- **Human checklist** is in the phase-4 PR (#4), extended with the Linux 22.04 / AppImage
  fullscreen check and the macOS "audio stops when you hide" check.
- **Not verified by the agent:** everything on the checklist — real desktops, real input
  devices, YouTube, Wayland, Gatekeeper/SmartScreen, multi-monitor placement.

---

### LJ10.1 — Always-visible URL bar · strong · auto

> **Added mid-plan 2026-10-17** — not part of the original scope. The user tried v0.1 on their
> Mac and asked for an "always on url bar on top so i can change site when needed". No human
> checkpoint: the user asked for none; CI (verify, package, smoke on all three OSes) is the gate.

**Verify:** `npm run verify` green, including Rust tests for `layout::layout` (bar on top at
`BAR_HEIGHT`, page below; hidden bar → page fills the window; never negative; bar + page tile
the window exactly) and `bar::bar_text` (start page and `about:` show nothing; web pages show
their full address), and vitest tests for `ui/bar-logic.ts` (Enter opens the trimmed text, not
while composing; Esc reverts an edit, then returns to the page, or closes on the start page;
an address update never clobbers an edit in progress). Capabilities: the bar webview gets only
`allow-open-input`, `allow-page-url`, `allow-focus-page`, `allow-hibernate`; the page webview's
local start page only `allow-start-info`; no remote URLs anywhere. CI green on ubuntu, windows,
macos for verify, package, and smoke — the smoke log must show `bar-loaded` and
`bar-url http://127.0.0.1…` for the smoke page.
**Commit:** `lj-step-10.1: always-visible URL bar above the page (second, trusted webview)`

```text
Add an address bar that is always visible above the page, in the existing minimal style.

Design (decided, do not relitigate):
- The page window becomes a plain tauri::Window holding TWO webviews (Tauri 2 multi-webview):
  "joan-bar" — a trusted local page (ui/bar.html) pinned to the top, BAR_HEIGHT = 40 logical px
  — and "joan-page" — the web page below it (no IPC for remote content, as before). No UI is
  injected into remote pages.
- Tauri 2.12 keeps the multi-webview API (window::WindowBuilder, Window::add_child,
  Manager::get_window / get_webview) behind its `unstable` feature: enable it in Cargo.toml and
  say why in a comment. This is the one new "dependency" change allowed.
- Layout: pure src-tauri/src/layout.rs (tested). Windows/macOS place child webviews by bounds,
  so re-apply the layout on Resized / ScaleFactorChanged. Linux packs child webviews into the
  window's GtkBox (bounds don't apply): pack the bar with expand=false at a fixed height
  request; the page takes the rest.
- The bar always shows the page's address ("" + placeholder on the start page). Push updates
  from Rust with bar.eval("window.littleJoan.setUrl(<json>)") on every page load (Started and
  Finished — redirects), every title change, after app-started navigations, and a 1 s check
  while the window exists (history.pushState / #anchor changes raise no event). The bar asks
  once on load (page_url command) so it never misses the first address.
- Click the bar or Ctrl/Cmd+L (now "focus the bar", replacing the old start-page round trip):
  edit; Enter → open_input (same normalization as before) in the PAGE webview, then focus the
  page; Esc reverts an edit, then (nothing to revert) returns focus to the page, or on the start
  page closes the window (keeps v0.1's "Esc on the empty field closes"). Leaving the field
  without Enter reverts it.
- The page webview's local start page becomes a hint only ("Type an address or a search in the
  bar above", shortcut hint); the bar has the focus on summon.
- Element fullscreen: hide the bar and let the page fill the window; bring it back when it
  ends. Windows: in the ContainsFullScreenElementChanged handler. Linux: WebKitGTK's
  enter-fullscreen / leave-fullscreen signals (return FALSE so WebKitGTK proceeds). macOS:
  WebKit shows element fullscreen in its own window, so nothing to do.
- Keys: install the WebView2 AcceleratorKeyPressed hook on both webviews (Windows); the GTK
  window-level hook already covers both (Linux); the app menu covers both (macOS).
- Smoke: the app logs bar-loaded and bar-url <text>; scripts/ci/smoke-log.mts requires both
  (the smoke page's address must reach the bar).

Files: src-tauri/src/{bar.rs, layout.rs, placement.rs (moved out of joan.rs), joan.rs,
page_actions.rs, commands.rs, keys.rs, platform/*, lib.rs}, build.rs, capabilities/{bar.json,
joan.json}, ui/{bar.html, bar.ts, bar-logic.ts (+test), index.html, start.ts, start-logic.ts,
style.css}, vite.config.mts, scripts/ci/smoke-log.mts (+test), docs, README.

Do NOT add buttons (back/forward/reload) to the bar — the existing design is keys + tray.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ10.1 to ✅ in
docs/BUILD_PLAN.md and commit everything as one commit: "lj-step-10.1: always-visible URL bar
above the page (second, trusted webview)". Push (open the phase-5 PR into lj/p4-ship) and watch
CI until verify, package, and smoke are green on all three OSes; Windows/Linux-only failures
are fixed per Ground Rule 3. If it is still red after two honest fix attempts, stop and report —
never commit red, never weaken a check to get green.
```

**Implementation notes (2026-10-17):**

- **Disk:** 15 GB free at the start; `.cache/target` ~1.7 GB after enabling `unstable`.
- **`unstable` is required** for `window::WindowBuilder`, `Window::add_child`, and
  `Manager::get_window` / `get_webview` in Tauri 2.12; the Cargo.toml comment says so.
- **Linux ignores child-webview bounds.** tauri-runtime-wry builds Linux child webviews with
  `build_gtk(window.default_vbox())`, i.e. `pack_start(expand, fill)` into a GtkBox — both
  webviews would split the window evenly. `platform/linux.rs` re-packs the bar with
  `expand = false` and a 40 px height request; `set_bounds` calls are harmless no-ops there.
- **Capabilities are per webview label now** (`"webviews": [...]`), not per window: a
  window-scoped capability would have applied to both the bar and the page webview.
- **Ctrl/Cmd+L changed meaning:** it used to navigate the page to the start page with the URL
  pre-filled; now it focuses the bar. `keys::start_page_url` and `ui/start-logic.ts`'s key
  handling went away with it; the start page is a hint only.
- **The 1 s address check** is a plain thread per window generation (no tokio dependency); it
  stops within a second of hibernating or when a newer window replaces it.
- **WebView2 needs both webviews in one environment,** so they share the data directory.
- 63 Rust tests, 42 TS tests.

**CI results (2026-10-17, run 38989508042):** verify ×3 and package ×3 green; smoke green on
**Windows and Linux** with the bar (screenshots: the 40 px bar showing
`http://127.0.0.1:…/smoke.html` above the page; during element fullscreen the page alone fills
the screen — no bar); the smoke log shows `bar-loaded`, `bar-url start`, then the smoke page's
address, and `element-fullscreen true`. **macOS failed one check:** 3 `WebContent` processes
were alive 60 s after hibernating — but the listing shows they started ~50 s *after* the window
was gone, with their own new `Networking` processes, i.e. another app's WebKit on the runner
(ours had all exited by 2 s). The check counted every WebKit helper on the machine. **Fix
(lj-step-10.1-fix):** the smoke test now follows the process ids that appeared while our page
was shown and counts which of those survive (`newPids` / `survivors`, unit-tested); the same
per summon cycle. The bar on macOS itself worked (`bar-loaded`, `bar-url` with the smoke
page's address).

**Second macOS finding (smoke-only rerun 38990685304, with attribution):** after summon cycle 2
(start page only), 2 of *our* helpers were still alive 10 s after hibernating (cycle 3: 0).
v0.1 only unloaded the page before destroying; the URL bar's webview wasn't. **Fix
(lj-step-10.1-fix):** on macOS both webviews are navigated to `about:blank` before the window
is destroyed (the bar's navigation filter allows `about:blank` for that).

**Third macOS finding (run 38990971632, green, but the screenshot was wrong):** the bar sat
behind the title bar — only its bottom ~10 px showed — and the page started 40 px below the
window's top edge. Tauri's default macOS title bar style (`Visible`) turns on
`NSFullSizeContentViewWindowMask`, so the content view (and y = 0 for child webviews) starts
under the title bar; a single-webview window never showed it because WKWebView insets itself.
**Fix (lj-step-10.1-fix):** the page window uses `TitleBarStyle::Transparent` on macOS, which
keeps the content view below the title bar. Smoke checks can't see pixel placement, so this
was caught by looking at the screenshot.

---

### LJ10.2 — Project files for GitHub · standard · auto

> **Added mid-plan 2026-10-17** — the user asked to make it "a whole project on GitHub".

**Verify:** `npm run verify` green; `CHANGELOG.md` (0.1.0 with what it contains),
`CONTRIBUTING.md` (short: build, verify, conventions), and `SECURITY.md` (report privately
through GitHub's private vulnerability reporting) exist and are linked from the README; LICENSE
is MIT, © 2026 Mehmet Ali Ertorer; `gh repo view --repo memoliertorerr/LittleJoan --json
description,homepageUrl,repositoryTopics` shows the description, the README as homepage, and
topics. No releases, no tags.
**Commit:** `lj-step-10.2: changelog, contributing guide, and security policy`

```text
Make the repository read as a whole project. Add CHANGELOG.md (Keep a Changelog style; 0.1.0,
unreleased date, listing what v0.1 contains including the URL bar and the known Linux 22.04
fullscreen issue), CONTRIBUTING.md (how to build with scripts/env.sh, `npm run verify`, the
CI jobs, the no-IPC-for-remote-pages rule, commit style), SECURITY.md (supported version,
report privately via GitHub → Security → "Report a vulnerability" — private vulnerability
reporting; no public issues for security problems). Link them from the README. Check LICENSE
(MIT, © 2026 Mehmet Ali Ertorer).

Then set the repository metadata with `gh repo edit --repo memoliertorerr/LittleJoan`:
description, homepage = the README URL, topics (browser, tauri, rust, typescript,
popup-browser, webview, desktop-app, windows, macos, linux, system-tray). That is the only
GitHub settings change allowed. Do NOT create releases or tags.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ10.2 to ✅ and commit:
"lj-step-10.2: changelog, contributing guide, and security policy". Push and watch CI. This
ends phase 5: fill the PR description from the template, then stop and report.
```

**Implementation notes (2026-10-17):**

- LICENSE was already correct (MIT, © 2026 Mehmet Ali Ertorer); unchanged.
- **Repository metadata set** (the only GitHub settings change made): description,
  homepage `https://github.com/memoliertorerr/LittleJoan#readme`, topics browser, tauri, rust,
  typescript, popup-browser, webview, desktop-app, windows, macos, linux, system-tray,
  hibernation. `gh repo edit` takes the repository as a positional argument (it has no `--repo`
  flag), so the command was `gh repo edit memoliertorerr/LittleJoan …`.
- ~~**GitHub's private vulnerability reporting is OFF on the repository**
  (`gh api repos/memoliertorerr/LittleJoan/private-vulnerability-reporting` → `enabled: false`).
  SECURITY.md points there, so the owner needs to switch it on (Settings → Code security →
  Private vulnerability reporting → Enable); enabling it was outside the allowed settings
  change.~~ The owner has switched it on since: the same call returns `enabled: true`
  (checked 2026-10-17).
- No releases or tags were created.

---

### LJ10.3 — Page size check and phone mode · strong · auto

> **Added mid-plan 2026-10-17** — from the user's bug report on the v0.1 build ("resizing doesn't
> affect the page … when Little Joan is long and narrow I expect a website to behave as if I
> opened it on an iPhone"). Branch `lj/fix-viewport` from `main`, PR into `main`. No human
> checkpoint; CI is the gate.

**Verify:** `npm run verify` green, including Rust tests that the layout gives the same logical
page size at scale factors 1, 1.5 and 2 (390×844 → page 390×804; 1280×800; 480×640), that the
phone threshold is 600 px and off when the setting is off, which phone user agent each OS gets
(Chrome major version copied on Windows), that a switch happens only on change, and that the
debounce fires once, 400 ms after the last resize; vitest tests for the smoke page's viewport
report and the new smoke assertions. CI green on Windows, macOS, Linux and Linux at scale 2:
after `--size 390x600` the page reports `viewport=390x560 ua=phone`, after `--size 720x600`
`viewport=720x560 ua=desktop`.
**Commit:** `lj-step-10.3: pages fit the window at any scale; phone mode for narrow windows`

```text
1. Confirm or rule out a physical-vs-logical size mix-up in the URL-bar layout (Retina = scale
   2, CI = scale 1). Make the conversion one pure, tested function (layout::for_physical_size)
   used by bar::apply_layout; test scale 1, 1.5, 2 with narrow and wide windows.
2. Phone mode: below 600 logical px of page width, the page webview identifies as a phone
   (iPhone Safari on macOS/Linux, Chrome on Android on Windows); otherwise the engine's own
   agent. Set at runtime per OS (WKWebView customUserAgent, WebView2 Settings2.SetUserAgent,
   WebKitGTK settings "user-agent"); new windows get it before any web page; a threshold
   crossing applies after the resize settles (400 ms) and reloads the current web page; never
   during fullscreen. Setting "Mobile sites in a narrow window" (mobileSites, default on).
3. `--size WxH` (resize the open window) so CI can drive it; the smoke page reports
   innerWidth/innerHeight/agent/devicePixelRatio in its title; smoke-log requires the narrow
   and wide reports; a Linux smoke job at GDK_SCALE=2.
4. Docs: README (Using it), CHANGELOG 0.1.0 (Added), architecture.md, this entry.
```

**Implementation notes (2026-10-17):**

- **Root cause of the report: not a sizing bug.** The user's screenshot is the pre-URL-bar
  build (no bar): one webview that Tauri itself sizes to the window. Google chooses its mobile
  or desktop results page by **user agent**, and its desktop page has a minimum width, so a
  narrow window got a cropped desktop page whatever its size. Phone mode fixes that.
- **HiDPI check of the URL-bar build: no mismatch found.** `bar::apply_layout` already
  converted the physical inner size with the window's scale factor and passed **logical**
  bounds; tauri-runtime-wry passes them through unchanged, and wry converts them with the
  display's own scale (`backingScaleFactor` on macOS, the window's DPI on Windows). The
  conversion is now one tested function, and CI checks the page's real `innerWidth` at scale 1
  (all OSes) and 2 (Linux). The Windows and macOS runners can't run at another scale.
- **Threshold 600 px:** phone layouts target ~360–430 CSS px; 600 is where Material's "compact"
  width ends and where Android stops treating a screen as a phone. The default 480 px window is
  therefore a phone; half a laptop screen is not.
- **Which phone:** the browser that shares the engine — iPhone Safari for WKWebView and
  WebKitGTK, Chrome on Android for WebView2 (with WebView2's Chrome major version), so mobile
  pages built for that engine come back.
- **Runtime agent APIs instead of recreating the page webview:** no history loss and no new
  webview; Tauri 2.12 has no runtime user-agent API, so each OS's API is called through
  `with_webview` (macOS adds `objc2` / `objc2-foundation`, already in the build through wry).
- **No switching during fullscreen:** a fullscreen video makes the window wide; switching to
  the desktop agent would reload the page and end the video.
- **CI caught a Linux panic (lj-step-10.3-fix):** WebKitWebView's `settings` GObject property is
  write-only ("property 'settings' of type 'WebKitWebView' is not readable"), so reading it
  through glib aborted the app on its first user-agent change. Linux now calls
  `webkit_web_view_get_settings` through the `webkit2gtk` crate (already in the build via wry).
  macOS and Windows passed the new viewport and user-agent checks on the first run.

---

## v0.2 steps

> Planned 2026-10-17 with the user (decisions: "Design Decisions Already Made → v0.2"). Every
> step inherits the Ground Rules, the stop conditions, and the v0.2 must-not-disturb list in the
> scope blockquote. Each step updates `CHANGELOG.md` under `[Unreleased]` (Ground Rule 17).

### LJ11 — Start at login · strong · auto

> Phase 6. Branch `lj/p6-first-run` from `main`; the phase PR goes into `main`.

**Verify:** `npm run verify` green, including Rust tests for `--start-at-login on|off` (`cli.rs`:
`on` / `off` case-insensitive, anything else ignored, and it never maps to a window command)
and for the macOS translocation guard (`autostart.rs`: a path containing `/AppTranslocation/` is
refused, `/Applications/Little Joan.app/Contents/MacOS/little-joan` is allowed), and vitest
tests showing the settings window's change detection includes the new switch. Capabilities:
`allow-set-start-at-login` is granted to the `settings` window only (LJ12 adds the local start
page); no `autostart:*` permission anywhere. CI green on Windows, macOS and Linux for verify,
package and smoke; the smoke log shows `start-at-login true` after `--start-at-login on` and
`start-at-login false` after `off`, and the smoke reads the OS entry itself — Windows: a value
under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` pointing at the installed
`little-joan.exe`, then none; macOS: a plist in `~/Library/LaunchAgents` whose
`ProgramArguments` point into the installed `.app`, then none; Linux: a `.desktop` file in
`~/.config/autostart` whose `Exec=` is the installed binary, then none.
**Commit:** `lj-step-11: start at login (settings switch and --start-at-login), checked per OS in smoke`

```text
Add "Start at login", so the global shortcut works after a reboot without launching Little Joan
by hand.

Design (decided with the user; do not relitigate):
- Use tauri-plugin-autostart (current 2.x; check with `cargo tree` that its dependencies unify
  with Cargo.lock's instead of adding second copies). macOS: MacosLauncher::LaunchAgent (a plist
  in ~/Library/LaunchAgents). Never AppleScript, which triggers an Automation permission prompt.
  Pass no extra launch arguments: a plain launch already starts hibernating.
- The OS login entry is the ONLY state. settings.json gets no field for it. get_settings
  reports `autolaunch().is_enabled()`, and the new command `set_start_at_login(enabled) ->
  Result<bool, String>` calls enable()/disable() and returns is_enabled() read back. If the user
  removes the entry in the OS's own settings, Little Joan's Settings shows that the next time it
  opens.
- Register the plugin for its Rust API only. Do NOT grant any autostart:allow-* permission; our
  own command calls it. Add set_start_at_login to build.rs's app manifest and grant
  allow-set-start-at-login in capabilities/settings.json. (LJ12 grants it to the local start
  page; don't do that here.)
- Off by default. Nothing turns it on implicitly.
- macOS guard: refuse to enable when the executable path contains "/AppTranslocation/" (an app
  run straight from the DMG or from Downloads): the LaunchAgent would record a temporary path
  that disappears. Return "Move Little Joan to Applications first, then turn this on." Keep the
  check pure in a new src-tauri/src/autostart.rs, with tests; native glue stays thin.
- Linux AppImage: the plugin already writes $APPIMAGE into the entry (confirmed in its source),
  so nothing to do here, but don't break the smoke's AppImage launch check.
- Settings window (ui/settings.html, ui/settings.ts, ui/settings-logic.ts): a "Start Little
  Joan at login" checkbox under the shortcut. Apply it on Save, only if it changed. On an
  error, keep the window open and show the message the way the shortcut error is shown.
- CLI: `--start-at-login on|off` (cli.rs) changes the setting without touching the window,
  forwarded through single-instance like the other flags, and logs `start-at-login
  <true|false>` (the value read back) to the smoke log (smoke.rs). Give it its own parse result
  or Command variant and handle it before toggle::decide; the toggle table is about the window
  and stays as it is.

Smoke (scripts/ci/smoke.mts, scripts/ci/smoke-log.mts and its test): after the existing summon
cycles, run `--start-at-login on`, then check the OS entry (Windows: `reg query
HKCU\Software\Microsoft\Windows\CurrentVersion\Run`; macOS: the plist and its ProgramArguments;
Linux: ~/.config/autostart/*.desktop and its Exec=). It must point at the INSTALLED binary: the
NSIS install folder, the .app copied out of the DMG, /usr/bin/little-joan. Then run `off` and
check it's gone. Write what you found into summary.json. Read the entry's file name and label
from the plugin's source (the auto-launch crate) rather than guessing, and record them under
Landmines.

Files: src-tauri/Cargo.toml (+ Cargo.lock), src-tauri/build.rs, src-tauri/capabilities/settings.json,
src-tauri/src/{autostart.rs (new), cli.rs, commands.rs, lib.rs, smoke.rs}, ui/settings.html,
ui/settings.ts, ui/settings-logic.ts (+ test), scripts/ci/smoke.mts, scripts/ci/smoke-log.mts
(+ test), README (Using it: start at login; macOS: move to Applications first), CHANGELOG
([Unreleased] → Added), docs/architecture.md (file map, commands table, CLI, smoke-log events),
this plan (tracker, notes; Deferred follow-ups: mark "Start at login" as handled by LJ11).

Do NOT build the welcome (LJ12), and don't touch tray.rs.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ11 to ✅, add
implementation notes (with the df -h numbers), and commit everything as one commit:
"lj-step-11: start at login (settings switch and --start-at-login), checked per OS in smoke".
Push, open the phase-6 PR into main as a draft, and watch CI until verify, package and smoke are
green on all three OSes; fix Windows/Linux-only failures per Ground Rule 3. If it is still red
after two honest fix attempts, stop and report. Never commit red; never weaken a check.
```

**Implementation notes (2026-10-17):**

- **tauri-plugin-autostart 2.5.1** (the newest 2.x; 3.0 is alpha), through `init(MacosLauncher::
  LaunchAgent, None)` — its `Builder::macos_launcher` exists only on macOS, so `init` is the
  call that compiles on all three. `cargo tree`: it unifies with Cargo.lock (tauri, serde,
  thiserror 2 and, through auto-launch, thiserror 1 and Windows' winapi were already there),
  except **two small second copies on macOS and Linux: `dirs` 4.0.0 and `dirs-sys` 0.3.7**
  (auto-launch 0.5 pins `dirs ^4`; the lock already had `dirs` 7 / `dirs-sys` 0.5). They depend
  only on libc. `winreg` 0.10 (Windows) is new but has no other copy. `getrandom` 0.2 /
  `redox_users` appear in the lock for Redox only and aren't compiled here.
- **Entry names, read from auto-launch 0.5.0's source** (recorded under Landmines): named after
  `package_info().name` = the product name "Little Joan" — the Run value `Little Joan`, the
  plist `~/Library/LaunchAgents/Little Joan.plist` (Label "Little Joan"), the file
  `~/.config/autostart/Little Joan.desktop` (it always uses `$HOME/.config`, not
  `$XDG_CONFIG_HOME`). The smoke doesn't rely on the names: it reads every Run value / plist /
  autostart file and looks for one that runs the installed binary.
- **Only a change is applied, then read back:** auto-launch's Windows `disable` deletes the Run
  value and fails if there is none, so `autostart::set` compares with `is_enabled()` first. On
  Windows `is_enabled()` also honours Task Manager's "disabled" flag (`StartupApproved\Run`), and
  `enable` resets it — so a login item the user disabled in Task Manager shows as off and turning
  it on works.
- **The CLI result is a struct, not a new `Command`:** `cli::parse_args` returns `Args { command,
  start_at_login }`; `forwarded_command()` keeps v0.1's "a second launch without flags shows the
  window" and returns nothing for a launch that only says `--start-at-login`. The toggle table
  is unchanged. Both the CLI and the settings window go through `autostart::set`, which writes
  the smoke-log line (`start-at-login <read back>` or `start-at-login-failed <reason>`)
  through the existing `smoke::log`, so `smoke.rs` needed no change.
- **Settings window:** the switch sits under the shortcut. Save calls `save_settings` only if a
  setting changed and `set_start_at_login` only if the switch changed (settings first, so a
  taken shortcut stops Save before the login item changes); an error keeps the window open in
  the same error line. If the entry can't be read, `startAtLogin` is `null` and the switch is
  disabled with a note.
- **macOS guard** checks the canonical path of the executable (the same path the plugin
  records). It only looks for `/AppTranslocation/`, as the prompt says; an app run from a
  mounted DMG that isn't quarantined (`/Volumes/…`) isn't caught — not added, since apps on
  external disks live under `/Volumes` too.
- **No CI workflow change:** the smoke check lives in `scripts/ci/smoke.mts`, which reads the
  entries itself (`reg query`, the plists, the `.desktop` files). On macOS the smoke copies the
  `.app` to `$RUNNER_TEMP/Applications` (not quarantined, so not translocated).
- Disk: 15 GiB free at the start, 14 GiB after the dependency and verify; `.cache/target` 1.1 GB.
- **CI (run 39039477975): green on the first push** — verify, package and smoke on Windows,
  macOS, Linux and Linux at scale 2. The smoke found `HKCU\…\Run\Little Joan` =
  `C:\Users\runneradmin\AppData\Local\Little Joan\little-joan.exe ` (unquoted, trailing space,
  as the source said), `~/Library/LaunchAgents/Little Joan.plist` →
  `$RUNNER_TEMP/Applications/Little Joan.app/Contents/MacOS/little-joan`, and
  `~/.config/autostart/Little Joan.desktop` → `/usr/bin/little-joan`; all gone after `off`.
  Memory at rest unchanged (own: 2.2 MB Windows, 11 MB macOS, 19.5 MB Linux).

### LJ12 — A one-time welcome on the first launch · standard · auto

**Verify:** `npm run verify` green, including Rust tests that a first run is detected only when
`settings.json` is missing (a present but invalid file is not a first run), that the default
settings are saved as soon as the welcome is shown, and that the tray-hint text is right per OS
(a pure function: macOS menu bar; Windows system tray, possibly in the ^ overflow; Linux tray plus
the GNOME AppIndicator note), and vitest tests for the start page's welcome logic
(`ui/start-logic.ts`). Capabilities: `capabilities/joan.json` gives the page webview's local
start page `allow-start-info` and `allow-set-start-at-login`, nothing else, and no remote URLs.
CI green on all three. The main smoke flow runs on a pre-written `settings.json` and is
unchanged (it still hibernates at start, with the same memory checks). A new first-run check
launches with an empty `LITTLEJOAN_DATA_DIR` and requires `first-run`, `window-created`,
`bar-loaded` and a `welcome.png` screenshot; after `--hide` and a quit, a relaunch on the same
folder creates no window within 5 s, and `settings.json` exists.
**Commit:** `lj-step-12: a one-time welcome on the first launch`

```text
A new user sees only a tray icon after installing (on GNOME without the AppIndicator extension,
nothing at all) and doesn't know the shortcut. Show a welcome once.

Design (decided with the user; do not relitigate):
- First run means settings.json doesn't exist when state::init runs; keep that as
  `first_run: bool` in AppState. A file that exists but is unreadable or invalid is NOT a first
  run.
- On a first run, setup registers the shortcut as now, then logs `first-run`, saves the default
  settings at once (settings::save), so the welcome appears only once even if the user quits
  straight away, and dispatches Command::Show(None). A first launch WITH flags (`--show <url>`,
  `--toggle`, `--start-at-login …`) acts on the flags instead and still saves settings, with no
  welcome. Every later launch hibernates, as in v0.1.
- The welcome is the existing start page (ui/index.html, ui/start.ts, ui/start-logic.ts) in a
  "welcome" variant, not a new window. StartInfo gains `firstRun: bool`, `trayHint: string` (a
  pure Rust function: where the icon is on this OS), `startAtLogin: bool | null` (null where it
  can't be read) and `waylandNoShortcut: bool` (a Linux Wayland session, where the X11 shortcut
  can't fire; LJ17 turns it off when the portal registers the shortcut). The URL bar keeps the
  focus as always: typing there and pressing Enter just opens the page.
- Content, kept short: "Little Joan is ready." · what the shortcut does ("⌘⇧Space summons and
  dismisses this window"), or on Wayland "Bind `little-joan --toggle` to a key in your keyboard
  settings" · where the icon is (trayHint; on Linux add "On GNOME the icon needs the
  AppIndicator extension. Opening Little Joan from your app list also summons it.") · a "Start
  Little Joan at login" checkbox calling set_start_at_login (from LJ11), which
  capabilities/joan.json grants to the page webview's LOCAL start page only · "Esc or the
  shortcut closes this window."
- Only the first summon is a welcome. Clear firstRun once the start page has read it, so later
  summons show the normal hint.

Smoke (scripts/ci/smoke.mts, scripts/ci/smoke-log.mts and its test): the existing flow must stay
comparable, because memory at rest is measured before any window opens. Launch it on a fresh
LITTLEJOAN_DATA_DIR that already holds a default settings.json. Then add the first-run check at
the end: quit, launch on a new empty LITTLEJOAN_DATA_DIR, require `first-run`, then
`window-created` and `bar-loaded`, screenshot welcome.png, `--hide`, quit, relaunch on the same
folder, and require that no window opens within 5 s. Every build reads LITTLEJOAN_DATA_DIR
(state.rs), so this tests the installed app.

Files: src-tauri/src/{state.rs, lib.rs, commands.rs, smoke.rs, and the tray-hint function with
tests}, src-tauri/capabilities/joan.json, ui/{index.html, start.ts, start-logic.ts (+ test),
style.css}, scripts/ci/{smoke.mts, smoke-log.mts (+ test)}, README (first launch), CHANGELOG
([Unreleased] → Added), docs/architecture.md (lifecycle: first launch; capabilities), this plan
(Deferred follow-ups: strike "First-run discoverability", resolved by LJ12).

Do NOT add a separate welcome window, a notification or a tray balloon. Change nothing about
hibernation beyond the one first-run summon.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ12 to ✅, add
implementation notes (with df -h), and commit everything as one commit: "lj-step-12: a one-time
welcome on the first launch". Push and watch CI until verify, package and smoke are green on all
three OSes (Ground Rule 3; stop after two honest fix attempts). Download each OS's welcome.png
into test-results/ci/, look at it, then delete it. This ends phase 6. Fill the PR description
from the template. Needs human check, on your Mac:
(1) quit Little Joan, move ~/Library/Application Support/io.github.memoliertorerr.littlejoan/settings.json
aside, launch it: the welcome shows; quit and launch again: it doesn't;
(2) tick "Start at login", log out and back in: the menu-bar icon is there and the shortcut
works;
(3) untick it in Settings: ~/Library/LaunchAgents has no Little Joan plist;
(4) the must-not-disturb list.
Then mark the PR ready for review and stop.
```

**Implementation notes (2026-10-17):**

- **First run = `settings.json` missing** (`state::is_first_run`: `try_exists() == Ok(false)`), so
  an unreadable or invalid file, or a folder that can't be looked into, is not a first run.
  `state::finish_first_run` saves the defaults and decides the welcome (no flags only); setup
  logs `first-run` first, after the shortcut is registered, as the prompt orders.
- **`firstRun` is cleared by `start_info` (`mem::take`)** and, when the first launch had flags,
  by setup, so a later summon never turns into a welcome. `start_info` logs `welcome` when it
  hands the first run over — an extra smoke-log event (not in the prompt) so CI checks that the
  start page really got it, not only that a window opened.
- **The tray hint and the Wayland test are pure** in a new `welcome.rs` (`tray_hint(os)`,
  `wayland_no_shortcut(os, XDG_SESSION_TYPE, WAYLAND_DISPLAY)`): a Wayland session counts even
  when the app runs on XWayland, since the X11 grab still only sees keys while an X11 window
  has the focus. LJ17 is where the portal turns it off.
- **The welcome** sits above the usual lead on the start page (the normal hint line is hidden
  meanwhile). The checkbox calls `set_start_at_login` at once and shows what the OS reads back;
  an error (the macOS translocation guard) appears under it and the box flips back. It is left
  out when the login entry can't be read. Its text follows the prompt; "no shortcut" (when even
  the default couldn't be registered) points at Settings.
- **Smoke:** every launch now gets its own `LITTLEJOAN_DATA_DIR` under `$RUNNER_TEMP` (outside
  the uploaded folder: WebView2 keeps its profile there); the main run's and the AppImage
  launch check's already hold the default `settings.json`, so both hibernate as before. Between
  the runs the app is quit and the smoke waits until its process is gone (under gdb the app
  itself is killed), so the next launch is the single instance. Logs:
  `smoke-first-run.log`, `smoke-relaunch.log`; `summary.json` gains `firstRun`.
- **Not done here (Deferred follow-ups):** after a click on the checkbox the keyboard is in the
  page webview, where Esc does nothing (Ctrl/Cmd+W and the shortcut still close); the start page
  has no `hibernate` permission and the prompt grants it nothing else.
- Disk: 14 GiB free before and after verify; `.cache/target` 1.1 GB.

### LJ13 — Build the AppImage on Ubuntu 24.04 · strong · auto

> Phase 7 (runs after round 3's phase 20). Branch `lj/p7-linux-fullscreen` from the latest
> `main`; the PR goes into `main`, and the agent merges it once every job is green (Ground Rule
> 24).
>
> *Round 3 adjustments (2026-10-18):* by now `smoke.mts` runs much more than in v0.1 — round 2's
> command bar, history, settings, extension, bundled and theme runs, and round 3's fullscreen,
> tray, quick-command and keymap runs. The `linux-appimage` entry runs all of it, like `linux`.

**Verify:** `npm run verify` green. CI: `package (linux)` on ubuntu-22.04 builds only the
`.deb`, so its glibc floor is unchanged; a new `package (linux-appimage)` on ubuntu-24.04 builds
the `.AppImage`, and its job summary shows the size and the bundled WebKitGTK version (2.52.x).
The AppImage gets the full smoke on ubuntu-24.04 (matrix entry `linux-appimage`, with the same
checks as `linux`, including element fullscreen and the phone-mode viewports), replacing the
launch-only check. Every other job stays green, and `installers_from_run` still works.
**Commit:** `lj-step-13: build the AppImage on ubuntu-24.04 (WebKitGTK 2.52) and smoke-test it fully`

```text
The AppImage bundles the WebKitGTK of the machine that built it. It is built on ubuntu-22.04,
whose WebKitGTK 2.50.4 aborts on element fullscreen (Landmines), so the AppImage crashes on
fullscreen on EVERY distro it runs on (seen in CI; not yet tried on a desktop). Ubuntu hasn't
updated 22.04's WebKitGTK since 2026-01-13, while 24.04 got 2.52.6 on 2026-08-31.

Decided with the user: build the AppImage on ubuntu-24.04. Its glibc floor rises to 2.39
(Ubuntu 24.04+, Debian 13, Fedora 40+). Ubuntu 22.04 and Debian 12 use the .deb, which LJ14
makes survive fullscreen. Keep building the .deb on ubuntu-22.04 so it still installs there.

1. .github/workflows/ci.yml, package job: split the linux entry into `linux` (ubuntu-22.04,
   bundles deb) and `linux-appimage` (ubuntu-24.04, bundles appimage), each with its own
   artifact (little-joan-linux, little-joan-linux-appimage) and its own cache key. Put the
   bundled WebKitGTK version in the job summary: find libwebkit2gtk-4.1.so.* in the AppDir or
   in the extracted AppImage.
2. Smoke: replace the "AppImage starts" launch-only step with a matrix entry `linux-appimage`
   (ubuntu-24.04, artifact little-joan-linux-appimage) that runs the same smoke.mts flow
   against the AppImage (APPIMAGE_EXTRACT_AND_RUN=1, since the runners have no FUSE; keep gdb).
   The installer download step must fetch the right artifact for each entry, and
   `installers_from_run` must keep working.
3. Note the AppImage's size in the PR (it was 149 MB in v0.1.0). A big change in either
   direction is worth a line in the notes, not a fix.
4. Update the local CLAUDE.md "Release process" (it is gitignored; edit it in place): the
   AppImage now comes from the little-joan-linux-appimage artifact.
5. Docs: README (which Linux download suits which distro: the .deb for Ubuntu 22.04+ and
   Debian 12+, the AppImage for glibc 2.39+), CHANGELOG ([Unreleased] → Changed; Fixed: the
   AppImage's fullscreen crash), docs/architecture.md (the Packaging table, CI), this plan
   (update the WebKitGTK entry in Landmines).

Do NOT add the 22.04 stopgap (LJ14) or change app code.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ13 to ✅, add notes, and
commit: "lj-step-13: build the AppImage on ubuntu-24.04 (WebKitGTK 2.52) and smoke-test it
fully". Push, open the phase-7 PR into main as a draft, and watch CI until every job is green.
Download linux-appimage's fullscreen.png into test-results/ci/, look at it, then delete it. If a
job is still red after two honest fix attempts, stop and report.
```

**Implementation notes (2026-10-19):**

- **Package** (`.github/workflows/ci.yml`): `linux` (ubuntu-22.04) builds only the `.deb` and
  `linux-appimage` (ubuntu-24.04) only the `.AppImage`, each with its own artifact
  (`little-joan-linux`, `little-joan-linux-appimage`) and cache key (`package-<name>-…`, as before).
  The `.deb` job adds its binary's glibc floor to the summary (the newest `GLIBC_` symbol version in
  `usr/bin/little-joan`); the AppImage job extracts the AppImage it built (`--appimage-extract`),
  prints how it starts (`AppRun` and its hooks), asks the bundled `libwebkit2gtk-4.1.so.*` for its
  own version (`webkit_get_major/minor/micro_version` through Python's ctypes, so it is the
  library that ships, not just the build machine's package — that package is named too) and greps
  every file for the newest `GLIBC_` version (the floor). **One addition to the prompt:** a bundled
  WebKitGTK older than 2.52 fails the job — the point of the step, kept from regressing silently.
- **Smoke:** a matrix entry `linux-appimage` (ubuntu-24.04 always — not `linux_runner`, whose 22.04
  can't run a glibc-2.39 build) runs the whole `smoke.mts` against the AppImage, copied to the
  release's file name (`LittleJoan-<version>-linux-x86_64.AppImage`, no space) and made executable;
  only the X11 test tools are installed, no WebKitGTK. The launch-only "AppImage starts" step is
  gone; the bare-WebKitGTK diagnostics run only where the `.deb` runs (they test the system's
  engine). `installers_from_run` still fetches `little-joan-<artifact>` per entry (a run from before
  this step has no `little-joan-linux-appimage`, so its AppImage entry would fail there).
- **Two things the prompt didn't foresee, both about how an AppImage runs without FUSE:**
  `APPIMAGE_EXTRACT_AND_RUN=1` makes the runtime extract into `$TMPDIR/appimage_extracted_<md5>`,
  run `AppRun` as its child, wait — and then **delete that folder**. Every second instance (the
  smoke's remote control: `--show`, `--size`, `--hide` …) shares the folder, so the first one to
  exit would delete the running app's files. `NO_CLEANUP=1` (the runtime's own switch) keeps it.
  And **gdb can't follow the runtime into the app**: with plugin hooks (GTK, GStreamer) linuxdeploy
  writes `AppRun` as a shell script that sources them and then `exec`s the app, and gdb would
  follow its `$(…)` subshells instead (the package job prints `AppRun`, so CI shows what it is). So for the AppImage `smoke.mts` attaches gdb (`gdb -p <pid> -ex continue`,
  then the same backtraces) once the app runs, and the job sets `kernel.yama.ptrace_scope=0` on
  the runner so gdb may attach to a process that isn't its child. A crash in the first moment,
  before gdb is attached, shows only as the runtime's exit status.
- **Which process is the app** (`scripts/ci/smoke-linux.mts` + test, new): the newest process in the
  launch's tree whose executable (`/proc/<pid>/exe`) is a file called `little-joan` — gdb's child for
  the `.deb`, the runtime's child for the AppImage (its process name follows the `AppRun.wrapped`
  link it was run through, so the old `pgrep -x little-joan` would miss it). Second instances are
  started apart, never in that tree. `/proc/<pid>/stat` is read after the last `)` (a process name
  can hold spaces and parentheses).
- No app code changed. `npm run verify`: 593 TS tests, 342 Rust tests, clippy clean. Disk: 13 GiB
  free at the start of the phase and after verify; `.cache/target` 1.6 GB.
- **CI (run 39240281091):** verify and package green everywhere; smoke green on Windows, macOS,
  `linux` and `linux-scale-2`. **The package jobs:** the AppImage is **156.79 MiB** (164,407,800
  bytes; v0.1.0's was 142 MiB — about 10 % more, 24.04's newer WebKitGTK, GTK and GStreamer: a
  note, not a fix), bundles **WebKitGTK 2.52.6** (`usr/lib/libwebkit2gtk-4.1.so.0`, from
  `libwebkit2gtk-4.1-0 2.52.6-0ubuntu0.24.04.1`) and needs **glibc 2.39**; its `AppRun` is
  linuxdeploy's bash script (sourcing `linuxdeploy-plugin-gstreamer.sh` and
  `linuxdeploy-plugin-gtk.sh`, then `exec "$this_dir"/AppRun.wrapped "$@"`), as expected. The
  `.deb` is 3.0 MiB and its binary needs glibc 2.34. **`linux-appimage` smoke: the app never
  started** — `little-joan: error while loading shared libraries: libEGL.so.1` (exit 127): an
  AppImage leaves EGL, GL, GBM, ALSA, X11 and fonts to the system (linuxdeploy's excludelist),
  every desktop has them, the runner didn't have EGL; and the harness then waited out every step
  until the job's 45 minutes ran out.
- **`lj-step-13-fix`:** the AppImage job installs what a desktop has and the AppImage leaves to it
  (`libegl1 libgl1 libgles2 libgbm1 libegl-mesa0 libgl1-mesa-dri libasound2t64`; still no
  WebKitGTK), then lists any library that is neither in the AppImage nor on the runner (`ldd` over
  every ELF file in it; a warning, not a failure: an optional GStreamer plugin may need something
  no one loads); `smoke.mts` fails at once when the app ends before it logged `started`, instead of
  timing out step by step. The `.deb`'s comment and the Packaging table now say glibc 2.34 (the
  binary's real need) instead of 2.35.
- **CI for the fix (run 39247235206): all twelve jobs green after one re-run** — macOS's smoke first
  failed one rule this phase doesn't touch: a WebContent process still alive 10 s after the third
  summon cycle's hibernate (`afterEachCycle` `[0,0,1]`; every earlier sample 0, the 60 s sample of
  the first cycle 0); `gh run rerun --failed` once: green (Deferred follow-ups). **`linux-appimage`
  passed every rule of the full smoke:** codecs H.264 / VP9 / AV1 through MSE (the bundled
  GStreamer), no WebKit helper left after hibernating (0 at 2–60 s, `[0,0,0]` per cycle), the
  start-at-login entry pointing at the AppImage itself (`$APPIMAGE`), the fullscreen run's four
  paths entered, held ~2.2 s and left by the page's own Exit, and gdb attached to the app in each
  of the 20 launches. The library check found only the bundled fluidsynth MIDI plugin's
  `libjack.so.0` / `libpipewire-0.3.so.0` missing (a plugin no page here loads). Memory at rest: the
  AppImage's own (private dirty) 45.6 MB at start against the `.deb`'s 19.5 MB in the same run —
  its bundled libraries' pages aren't shared with the system's —, both ~44–45 MB after a page
  (Deferred follow-ups). **Screenshots** (`linux-appimage`): `fullscreen.png` — the green stage
  fills the whole 1280×800 screen, the page reporting 1280×800 (no crash: the bundled 2.52.6);
  `fs/element-390.png` — the element fills the screen with its Exit in the middle; `command-bar.png`
  — the command bar in openbox's frame, the smoke page as the top site, as on `linux`.

### LJ14 — WebKitGTK older than 2.52: fullscreen the window instead of aborting · strong · auto

**Verify:** first, the experiment: a CI run on ubuntu-22.04 of the bare PyGObject check, with
an `enter-fullscreen` handler that returns `True`, shows the process still running after the
click. If it aborts anyway, that is a stop condition: record it, change no app code, and finish
only point 4. If the experiment passes: `npm run verify` green, including Rust tests for
`fullscreen::needs_window_fallback(major, minor)` (2.44 → true, 2.50 → true, 2.52 → false,
2.54 → false). CI green everywhere, plus a permanent smoke entry `linux-22.04` (ubuntu-22.04,
the `.deb`) whose log shows `fullscreen-fallback`, then `fullscreen true`, then
`fullscreen false` after the page loads again, with no abort. The 24.04 entries never log
`fullscreen-fallback`. `docs/upstream/webkitgtk-2.50-fullscreen-abort.md` exists.
**Commit:** `lj-step-14: WebKitGTK older than 2.52 fullscreens the window instead of aborting; 22.04 smoke`

```text
Ubuntu 22.04's WebKitGTK 2.50.4 aborts the process when a page enters element fullscreen
(Landmines; reproduced with a bare WebKitGTK window). The .deb targets 22.04, so 22.04 users
crash on any video's fullscreen button. Decided with the user: a stopgap for WebKitGTK older
than 2.52. When a page asks for element fullscreen, decline it and fullscreen the WINDOW
instead (bar hidden, page fills the screen). The page doesn't know it's fullscreen, so a video
stays its in-page size, but nothing crashes.

1. Prove it helps before writing app code. Add a variant to ci.yml's "Bare WebKitGTK fullscreen
   check" whose enter-fullscreen handler returns True (handled, so WebKit cancels the request),
   and run it on ubuntu-22.04:
   `gh workflow run ci.yml --repo memoliertorerr/LittleJoan --ref <branch> -f installers_from_run=<a green run id> -f linux_runner=ubuntu-22.04 -f linux_plain_webkit_check=true`.
   If the log says "still running after the click" for the handled variant, go on. If it
   aborts anyway (the crash happens before the signal), STOP: record that in the
   implementation notes and Landmines, leave the app alone, keep the known issue in the README
   and CHANGELOG, still do point 4, and report.
2. App (src-tauri/src/platform/linux.rs, fullscreen.rs, bar.rs): read WebKitGTK's runtime
   version (webkit_get_major_version / webkit_get_minor_version). Add a pure
   `fullscreen::needs_window_fallback(major, minor) -> bool` that is true below 2.52, with
   tests. When it applies, the enter-fullscreen handler returns TRUE, logs
   `fullscreen-fallback`, fullscreens the window and hides the bar, through the same path as
   bar::set_element_fullscreen. Esc, a new page load (Started), or the window leaving
   fullscreen by other means ends it and brings the bar back. Esc is handled in
   platform/linux.rs's GTK key hook before the keymap (Esc is never a keymap key, Ground Rule
   25), and only while the fallback is active. FullscreenTracker's rule holds: undo
   only a fullscreen we caused. WebKitGTK 2.52+ keeps today's behavior untouched.
3. CI smoke: add a permanent matrix entry `linux-22.04` (ubuntu-22.04, artifact
   little-joan-linux, the .deb). Its smoke-log rules, unit-tested: `fullscreen-fallback`, then
   `fullscreen true`, then `fullscreen false` after the page loads again, and no abort.
   Everything else matches the `linux` entry — the whole smoke, round 2's and round 3's runs
   included (the extension runtime's security checks must hold on the oldest engine the .deb
   reaches). LJ36's fullscreen run expects the fallback there: each path logs
   `fullscreen-fallback` and window fullscreen, the page reports `entered=0` (its request was
   declined), and the fallback ends when the page loads again. If any other check fails only on
   22.04 for an engine reason, stop and report — don't exempt the entry from it. The 24.04
   entries must NOT log fullscreen-fallback. If the run's annotations announce the ubuntu-22.04
   image's retirement, record it under Landmines.
4. Write docs/upstream/webkitgtk-2.50-fullscreen-abort.md: a bug report for Ubuntu's
   webkit2gtk package on Launchpad. Jammy has stayed at 2.50.4-0ubuntu0.22.04.1 since
   2026-01-13, while noble and resolute got 2.52.6 on 2026-08-31. Include the bare PyGObject
   reproduction, the CI evidence (run ids), and the ask: 2.52.x for jammy. The user files it;
   do not post anything outside this repository.
5. Docs: README (on 22.04 a video's fullscreen button fills the screen with the page, not with
   the video), CHANGELOG ([Unreleased] → Fixed), docs/architecture.md (the Fullscreen table
   gains a WebKitGTK < 2.52 row), this plan (Landmines; in Deferred follow-ups, strike the Linux
   fullscreen line once LJ13 and LJ14 are both done).

Do NOT change the Windows or macOS fullscreen code.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ14 to ✅, add notes, and
commit: "lj-step-14: WebKitGTK older than 2.52 fullscreens the window instead of aborting; 22.04
smoke". Push and watch CI until every job is green (two honest attempts, then stop). Look at
linux-22.04's fullscreen.png. This ends phase 7. Fill the PR description. Needs human check:
- If you have an Ubuntu 22.04 desktop: in the .deb build, play a YouTube video and press its
  fullscreen button. Nothing crashes, the window fills the screen, and Esc brings it back.
- On 24.04: the AppImage's fullscreen fills the screen with the video.
- The must-not-disturb list.
These checks also go into LJ19's checklist. Once every job is green, merge the PR into main with a
merge commit (Ground Rule 24), then stop and report.
```

**Implementation notes (2026-10-19) — stopped at the experiment:**

- **Disk:** 13 GiB free at the start of the step and after verify; `.cache/target` 1.6 GB.
- **The experiment** (its own commit, `lj-step-14-experiment`: `gh workflow run` needs the variant
  on the branch, and the prompt puts it before any app code): the "Bare WebKitGTK fullscreen check"
  now runs the bare PyGObject window three ways — WebKitGTK's own handling (`default`), an
  `enter-fullscreen` handler that returns TRUE (`handled`), and that handler fullscreening the
  window too (`handled-window`, the stopgap's exact shape) — printing WebKitGTK's version, the
  page's titles (it also reports `fullscreenerror` and a refused promise) and the signal. It moved
  before the app's smoke: an app that aborts on 22.04 could otherwise run the job into its 45
  minutes before the check. **Run 39247249388** (`installers_from_run=39240281091`,
  `linux_runner=ubuntu-22.04`, `linux_plain_webkit_check=true`), on both 22.04 jobs (`linux`,
  `linux-scale-2`), WebKitGTK 2.50.4: in every variant `enter-fullscreen: <variant>` is printed —
  the signal comes, the handler runs and returns — and then `BARE WEBKITGTK (<variant>): exited
  with status 134` (SIGABRT). No `fullscreenerror`, no refused request and no window-state change
  came first. The app's own smoke in those jobs aborted at its first fullscreen click with v0.1's
  backtrace (the same addresses: frames in `libwebkit2gtk-4.1.so.0` under a task JavaScriptCore's
  RunLoop dispatched). Its Windows, macOS and AppImage smoke against the LJ13 installers passed
  meanwhile; the run ends as cancelled (cancelled once both 22.04 checks had run, the rest being
  the 22.04 app smoke, known to abort).
- **The step's stop condition fired:** the handled request aborts anyway — not before the signal,
  as the prompt guessed, but after it, whatever the handler returns. So, as the prompt says: **no app
  code** (no `needs_window_fallback`, no fallback, no `linux-22.04` smoke entry — it would only show
  the abort), Landmines updated, the known issue kept in the README (Fullscreen video: no workaround
  is possible in the app; avoid fullscreen buttons on 22.04) and the CHANGELOG (an `[Unreleased]`
  Known issues entry: 22.04's `.deb` still crashes; the AppImage, fixed by LJ13, needs 24.04+).
- **Point 4:** `docs/upstream/webkitgtk-2.50-fullscreen-abort.md` — the Launchpad report draft for
  Ubuntu's webkit2gtk (jammy): the bare reproduction in its three variants, the backtrace, what was
  ruled out (the embedding app, the renderer, the window manager, how the request is handled), the
  CI runs (38934582112, 38935403919, 39247249388) and the ask (2.52.x for jammy, as noble got).
  Nothing was posted: the owner files it (from a 22.04 machine, `ubuntu-bug libwebkit2gtk-4.1-0`).
- docs/architecture.md: the Fullscreen table's WebKitGTK < 2.52 row (what aborts, that no stopgap
  is in the app), the bare check's variants, `docs/upstream/` in the file map. Deferred follow-ups:
  the Linux fullscreen line stays open with what is left for 22.04.
- **Deviation:** the commit message. The plan's "lj-step-14: WebKitGTK older than 2.52 fullscreens
  the window instead of aborting; 22.04 smoke" would describe work the stop condition ruled out, so
  this commit says what it holds.
- `npm run verify`: 593 TS tests, 342 Rust tests, clippy clean (no code changed since
  `lj-step-13-fix`).

### LJ14.1 — WebKitGTK older than 2.52: element fullscreen off instead of the abort · strong · auto

> **Added mid-plan 2026-10-19** — the owner's call after LJ14 stopped (through the session's
> coordinator): "a crash is worse than no fullscreen", so try the remaining option before merging
> phase 7. Same branch (`lj/p7-linux-fullscreen`), same PR (#25), merged when green.

**Verify:** first, the experiment: a CI run on ubuntu-22.04 (two runners) of the bare PyGObject
check with WebKitSettings' `enable-fullscreen` FALSE shows the process still running after the
click; if it aborts too, no app code — record it under LJ14 and Landmines. If it survives: `npm run
verify` green, including Rust tests for `fullscreen::needs_fullscreen_off(major, minor)` (2.44 /
2.50 → true, 2.52 / 2.54 → false) and vitest tests for the new rules; CI green everywhere plus a
permanent smoke entry `linux-22.04` (ubuntu-22.04, the `.deb`) whose log shows `fullscreen-off
<version>`, no abort, the page still loading and reporting, and after the click no `fullscreen
true`; the 24.04 and AppImage entries never log `fullscreen-off`.
**Commit:** `lj-step-14.1-experiment: …` (the variant, as for LJ14), then `lj-step-14.1: WebKitGTK older than 2.52 turns element fullscreen off instead of crashing; Ubuntu 22.04 smoke`

```text
1. Experiment first: the bare check gains a variant with enable-fullscreen = FALSE; run it on
   ubuntu-22.04 on two runners (installers_from_run, linux_runner=ubuntu-22.04,
   linux_plain_webkit_check=true). A fullscreenerror or a rejected promise in the page is fine.
2. If it survives on both: platform/linux.rs sets enable-fullscreen FALSE on the page webview when
   the runtime WebKitGTK is older than 2.52 (pure fullscreen::needs_fullscreen_off, tested), and
   logs `fullscreen-off <version>`. A permanent smoke entry `linux-22.04` (ubuntu-22.04, the .deb):
   `fullscreen-off`, no abort, the page still loads and reports, no `fullscreen true` after the
   click; the 24.04 / AppImage entries never log `fullscreen-off`.
3. Docs: README and CHANGELOG ("on Ubuntu 22.04's WebKitGTK 2.50 a video can't go fullscreen;
   Little Joan turns fullscreen off there instead of crashing"), the architecture's Fullscreen
   table, Landmines, Deferred follow-ups (annotate the Linux fullscreen line).
4. If it still aborts: no app code; record it in LJ14's notes and Landmines; keep the known issue.
```

**Implementation notes (2026-10-19):**

- **Disk:** 11 GiB free at the start of the step (other things on the Mac used 2 GiB meanwhile),
  11 GiB after verify; `.cache/target` 1.6 GB.
- **The experiment** (`lj-step-14.1-experiment`): a fourth variant of the bare check,
  `fullscreen-off` (`view.get_settings().set_enable_fullscreen(False)` before the load; the handler
  now returns TRUE only for the two `handled` variants). Run 39253920848 (`installers_from_run=
  39251243907`, `linux_runner=ubuntu-22.04`, `linux_plain_webkit_check=true`); its first attempt's
  22.04 jobs were force-cancelled once the check had run, which threw their logs away (GitHub keeps
  no log of a force-cancelled job), so they were re-run (`gh run rerun --failed`, attempt 2) and
  left to end. **On both 22.04 runners (WebKitGTK 2.50.4):** `default`, `handled` and
  `handled-window` exit with status 134 again, and `fullscreen-off` prints `enable-fullscreen:
  False`, `title: plain ready`, then `BARE WEBKITGTK (fullscreen-off): still running after the
  click`. The page never changed its title: with the setting off WebKit doesn't expose the
  Fullscreen API at all, so the page's `requestFullscreen()` isn't there to call (no
  `fullscreenerror`, nothing to reject).
- **The app** (`platform/linux.rs`, `fullscreen.rs`): `needs_fullscreen_off(major, minor)` — older
  than 2.52 — (pure, tested at 1.99, 2.44, 2.50, 2.51, 2.52, 2.54, 3.0); `install_page` reads the
  runtime version (`webkit_get_major/minor/micro_version` through `webkit2gtk::ffi`, the library
  actually loaded: the system's for the `.deb`, the bundled one in the AppImage) and, below 2.52,
  sets WebKitSettings' `enable-fullscreen` FALSE on the page webview and logs `fullscreen-off
  <major.minor.micro>` — every page window, before any web page loads. The `enter-fullscreen` /
  `leave-fullscreen` hooks stay (they never fire there). Windows and macOS untouched; WebKitGTK
  2.52+ untouched.
- **Smoke:** `linux-22.04` (ubuntu-22.04, the `.deb`, `LJ_SMOKE_FULLSCREEN=off`) runs the whole
  smoke. The main run's click waits 4 s and expects nothing (window fullscreen and page fullscreen
  not required; the page's next load and the viewports after it show the app alive); the fullscreen
  run skips its Exit clicks (there is no fullscreen to leave) and `fullscreenFailures(…, true)` wants
  every element path to report `entered=0`; `smoke-fullscreen-off.mts` (+ test, new):
  `fullscreen-off` with a version older than 2.52 in the main and both fullscreen launches, no
  `fullscreen true` / `element-fullscreen true` there, no `LJ-SMOKE fullscreen=on`; on every other
  job no `fullscreen-off` at all. `summary.json` gets `fullscreenOff: { expected, versions }`.
- Docs: README (install and Fullscreen video), CHANGELOG (Fixed; Known issues: no fullscreen video
  on 22.04), architecture.md (the Fullscreen table's WebKitGTK < 2.52 row, the smoke log's
  `fullscreen-off`, the CI's `linux-22.04` job, the bare check's four variants, the file map), the
  Launchpad draft (switching fullscreen off is the only way around it found; the new run), this plan
  (Landmines; Deferred follow-ups: option (e) done).
- `npm run verify`: 599 TS tests, 343 Rust tests, clippy clean (the Linux code compiles only in
  CI's verify).
- **CI (run 39258604008): all thirteen jobs green on the first try**, the new `smoke (linux-22.04)`
  among them. On 22.04's WebKitGTK 2.50.4 every page window logged `fullscreen-off 2.50.4`, nothing
  logged `fullscreen true` or `element-fullscreen true`, and every fullscreen-run path reported
  `entered=0` — the element paths with `no-fullscreen-api` (the page has no Fullscreen API with the
  setting off), the phone player's iOS route with `enter-error:invalidstateerror` —, while the rest of
  the smoke passed there as on 24.04: codecs H.264 / VP9 / AV1 through MSE, no helper left after
  hibernating, logins, history, Settings, the extension runtime's security checks, the bundled
  extensions, themes, resize, site modes, quick commands, the keymap, the tray. Own memory at rest
  21.8 MB. No other job logged `fullscreen-off`. **Screenshots** (`linux-22.04`): `fullscreen.png` —
  the window as it was, 480×640 with the toolbar (the Phone pill) above the smoke page's "Go
  fullscreen", the click having changed nothing; `fs/element-720.png` — the 720 px window with
  "Element fullscreen" and "Phone player" in place, the page still `ready`. The run's annotations
  don't mention the ubuntu-22.04 image's retirement.

### LJ15 — Downloads · strong · auto

> Phase 8. Branch `lj/p8-downloads-zoom` from the latest `main`; the PR goes into `main`.
>
> *Round 3 adjustments (2026-10-18):* the bar is LJ24's toolbar by now, so the status area lives
> in `ui/toolbar/`; downloads are refused while the page shows an extension panel (extensions have
> no downloads permission); the new command goes into a child module; the smoke rules into their
> own file.

**Verify:** `npm run verify` green, including Rust tests for a pure `downloads.rs`. The file
name comes from the engine's suggested name, else the URL's last path segment, else
`download`. It is cleaned: no path separators, no `..`, no control characters, no Windows
reserved names (`CON`, `NUL`, `COM1`…) or trailing dots and spaces, capped at 200 bytes. It is
made unique in the folder: `report.pdf` → `report (1).pdf`, `archive.tar.gz` →
`archive (1).tar.gz`. Only `http`, `https`, `blob` and `data` downloads are accepted, and none
while the page shows an extension panel (`download-refused panel`). Vitest tests for the
toolbar's status logic in `ui/toolbar/logic.ts`: downloading → saved → cleared after 8 s;
failed; the command bar opening never loses a status. Capabilities: the bar gains only
`allow-reveal-download`. CI green on all three, with a new smoke check: `--show` a smoke-server
URL that answers `application/octet-stream` with `Content-Disposition: attachment;
filename="lj-smoke.bin"`. The log then shows `download-started lj-smoke.bin` and
`download-finished lj-smoke.bin ok`, the file is in that OS's Downloads folder with the exact
bytes, and a second identical download lands as `lj-smoke (1).bin`.
**Commit:** `lj-step-15: downloads go to the Downloads folder, with status in the URL bar`

```text
WebView2 saves downloads with its own UI; WKWebView and WebKitGTK drop them (Deferred
follow-ups). Make downloads work the same way everywhere.

Design (decided with the user; do not relitigate):
- The page webview (bar.rs, `page` builder) gets on_download. On Requested: accept only
  http/https/blob/data URLs (anything else returns false). Set the destination to the OS
  Downloads folder (app.path().download_dir(); if it reports none, as on Linux without XDG
  user dirs, use ~/Downloads and create it) under a cleaned, unique name. Base the name on the
  engine's suggested destination, else the URL. Log `download-started <name>`, remember the
  destination keyed by URL (DownloadEvent::Finished carries no path on macOS; see Landmines),
  and tell the bar. On Finished: log `download-finished <name> ok|failed` and tell the bar.
- While the page shows an extension panel (extensions::session's committed panel), refuse every
  download and log `download-refused panel`: no extension has a downloads permission, so a panel
  must not drop files in Downloads. (LJ47's popups refuse downloads too.)
- The toolbar (LJ24: ui/toolbar/logic.ts + toolbar.ts, ui/bar.html, ui/style.css) gets a small
  status area left of the open-in-browser button — where LJ40's phone / desktop pill sits; the
  status takes the pill's place while it shows, and both fit at 320 px — pushed from Rust like the
  address:
  bar.eval("window.littleJoan.setStatus(<json>)"). Only the bar holds trusted UI (LJ22).
  It shows "Downloading lj-smoke.bin…", then "lj-smoke.bin saved" with a "Show in folder"
  button, or "Download failed". It clears after 8 s; while the command bar is open it waits and
  shows when the toolbar comes back. The button calls a new command `reveal_download()` (in a
  child module, src-tauri/src/commands/downloads.rs), which reveals the last saved file with
  tauri_plugin_opener's reveal_item_in_dir. It is granted to the bar only and added to build.rs's
  app manifest. LJ16 reuses this status area for the zoom level, and LJ46 for "No other browser",
  so keep it generic (text + optional action).
- Closing the window during a download cancels it. Nothing keeps a webview alive for it
  (Ground Rule 9). No notification plugin.
- Windows: WebView2's own download flyout must not appear on top. Check it in the smoke
  screenshot; if wry doesn't suppress it, set `Handled` on the DownloadStarting event's
  arguments (ICoreWebView2DownloadStartingEventArgs) in platform/win.rs, keeping wry's
  destination.

Smoke (a new scripts/ci/smoke-downloads.mts + test for the run and its rules, wired into
smoke.mts; the smoke server lives in smoke.mts): add a route that answers with a fixed 1 KB body, Content-Type application/octet-stream and
Content-Disposition attachment; filename="lj-smoke.bin". Use `--show` on that URL twice. Check
the log events, the bytes of Downloads/lj-smoke.bin and of Downloads/lj-smoke (1).bin, and take
a screenshot of the bar (download.png). Delete both files afterwards (the runner is
disposable, but stay tidy).

Files: src-tauri/src/{downloads.rs (new, pure), bar.rs, commands/downloads.rs (new), lib.rs,
smoke.rs, platform/win.rs if needed}, src-tauri/build.rs, src-tauri/capabilities/bar.json,
ui/{bar.html, bar.ts, toolbar/logic.ts, toolbar/toolbar.ts (+ tests), style.css},
scripts/ci/{smoke.mts, smoke-downloads.mts (+ test)},
README, CHANGELOG ([Unreleased] → Added), docs/architecture.md (the page, the URL bar,
commands, smoke log), this plan (Deferred follow-ups: "Downloads" handled by LJ15).

Do NOT add a downloads list, a progress bar, pause/resume, or a "where to save" dialog (Later).

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ15 to ✅, add notes, and
commit: "lj-step-15: downloads go to the Downloads folder, with status in the URL bar". Push,
open the phase-8 PR into main as a draft, and watch CI until
every job is green (Ground Rule 3; two honest attempts, then stop). Look at each OS's
download.png.
```

**Implementation notes (2026-10-19):**

- **Disk:** 12 GiB free at the start of the phase and after verify; `.cache/target` 1.6 GB.
- **`downloads.rs`** (pure; 13 tests in `downloads/tests.rs`): `accepts` (http / https; `data:`;
  `blob:` whose page is http(s); never an extension URL in any form — `ljext:`, `http(s)://ljext.*`,
  a blob of either —, the app's own origin, Windows' `http://tauri.localhost` included, `file:` or
  another scheme), `file_name` (the engine's suggestion, else the URL's last path segment
  percent-decoded, else `download`; WebKitGTK's payload "name" for a `data:` URL counts as none),
  `clean` (the prompt's rules, and a little more, the restrictive choices: the characters Windows
  refuses (`<>:"|?*`) become `_` on every OS, format characters go with the control ones — a bidi
  override could show `…fdp.exe` as `…exe.pdf` (the manifest's `is_hidden_char`) —, leading dots go
  too, so no hidden files and no `..`; device names get a `_` in front, `extensions::paths::DEVICES`),
  the 200-byte cap keeping the extension at a character boundary, `split_extension` (the last dot
  part when it is 1–10 letters or digits, with a `.tar` before it: `archive (1).tar.gz` but
  `v1.2.3-setup (1).exe`, and `Chapter 1. Intro (1)` has none), `unique` (up to ` (9999)`, then
  refused), and `Downloads` — the window's downloads under way keyed by URL (the earliest of a URL
  first; reported twice → once), the last one saved, `decide` (panel → scheme → name; a name under
  way counts as taken, case aside, so two quick downloads can't pick one name before either lands)
  and `closed`.
- **Runtime** (`bar/download.rs`, a child of bar.rs — which moved its tests to `bar/tests.rs` to stay
  under 400 lines): `on_download` for WebView2 and WKWebView (Requested: the suggestion is the file
  name of the engine's own destination; Finished: by URL), `requested` / `finished` for every engine,
  the Downloads folder (`download_dir()`, else `~/Downloads`, created), the status pushed to the bar
  (`setStatus({ kind: "download", state, name })`) and kept per window to tell the bar again when its
  script runs (`tell_state`), and `window_closed` (from `joan::destroy`, before the page is
  released). Smoke log: `download-folder`, `download-started`, `download-finished <name> ok|failed`,
  `download-refused <reason>`, `download-closed`. "While the page shows an extension panel" is the
  committed panel **or** the one the guard admitted and that hasn't committed yet (the more
  restrictive reading; `--show <download>` over a panel leaves `committed` set, since a download
  never commits). A refused download shows "Download blocked" (panel) or "Download failed" (the
  rest) — the prompt named only the other three texts.
- **WebKitGTK, a deviation the code forced (`platform/linux.rs`, beyond the prompt's files):** read
  in wry 0.57 (`webkitgtk/web_context.rs`, `webkitgtk/mod.rs`, `lib.rs`) — every webview's
  attributes default to an allow-all `download_started_handler`, and on Linux wry registers it on
  the **web context** (`connect_download_started`, then `decide-destination` on each download) once
  per webview built; Tauri keeps one context per data folder for the app's lifetime on Linux
  (tauri-runtime-wry's `WebviewWrapper::drop`), shared by both webviews of every window (and
  Settings in installed builds). WebKitGTK's `decide-destination` stops at the first handler that
  returns TRUE (`g_signal_accumulator_true_handled`, checked in WebKit's `WebKitDownload.cpp`), and
  wry's always do — so the bar's default handler, registered first, would have decided every page
  download, `on_download`'s `Requested` would never have run, and each new window would have added
  one more `finished` report, with a `failed` flag that is never reset. So on Linux the page doesn't
  use Tauri's `on_download`: `install_page` hooks `download-started` once per context, takes only
  downloads whose `web_view` is the open window's page, disconnects the `decide-destination`
  handlers connected so far (`g_signal_handlers_disconnect_matched` by signal id; smoke log
  `download-deciders-removed <n>`) and connects its own, which calls the same `requested` and sets a
  `file://` URI (`g_filename_to_uri`: the 4.1 API's legacy form, which 22.04's 2.50 needs), with
  per-download `failed` / `finished` handlers and a list of the page's downloads under way, cancelled
  on the main thread when the window closes (`platform::cancel_downloads`). Landmines has it.
- **Windows' download flyout:** nothing to add in `platform/win.rs`: wry's `DownloadStarting` handler
  calls `SetHandled(true)` whenever `on_download` accepts (and `SetCancel(true)` when it refuses), and
  a handled download shows no WebView2 UI — the screenshot decides.
- **The toolbar** (`ui/toolbar/logic.ts` + `toolbar.ts`, `bar.html`, `style.css`): a generic status
  area — `statusMessage` (checked), `statusView` (text, tooltip, an optional action: only
  `reveal_download`), `lingerFor` (until replaced while downloading, else 8 s), and a small state
  (`withStatus`, `withToolbarVisible`, `shownStatus`, `nextStatusChange`) that counts only the time
  the toolbar is visible, one status per kind, the latest shown; 9 new tests in `logic.test.ts`
  (downloading → saved → cleared after 8 s; failed; the command bar opening never loses one; a
  status that arrives while the command bar is open waits; names shortened in the middle; markup
  stays text). While a status shows, the pill hides. `#toolbar-status` sits between the pill and
  open-in-browser, `max-width: 60%`, the name ellipsized, the button kept, so site and status share
  320 px.
- **`reveal_download()`** (`commands/downloads.rs`; build.rs's manifest; `capabilities/bar.json` gains
  exactly `allow-reveal-download`; `capability_tests.rs`' `only_the_bar_may_reveal_downloads`): the
  last file saved in this window, `tauri_plugin_opener::reveal_item_in_dir`.
- **Smoke** (`scripts/ci/smoke-downloads.mts` + test, 7 tests; the smoke server's routes in
  `smoke.mts`): its own launch after the keymap run. The address (`/dl-attachment`) names something
  else than the header, so the name must come from `Content-Disposition`. Two more things,
  informational in `summary.json`'s `downloads`: a PDF sent as an attachment (`/dl-pdf`) — whether
  each engine downloads it or shows it (wry's WKWebView response policy looks only at
  `canShowMIMEType`) —, and **closing the window during a download** (`/dl-slow`, 40 × 1 KB, one
  every 500 ms): whether the server saw the engine stop, what is left in Downloads, the log after
  `--hide` — the evidence for what WKWebView and WebView2 do (below, with CI).
- Docs: README (Downloads; Privacy), CHANGELOG (Added), docs/architecture.md (file map, the page's
  Downloads, the toolbar's status area, commands, capabilities, smoke log, CI), this plan (Deferred
  follow-ups, Landmines).
- `npm run verify`: 615 TS tests, 357 Rust tests, clippy clean (the Linux path compiles only in CI).
- **CI (run 39270397918): verify ×3, package ×4 and smoke on Windows, macOS, `linux`,
  `linux-scale-2` and `linux-appimage` green; `smoke (linux-22.04)` red — no download at all.**
  Everywhere else both attachments landed in `~/Downloads` (Windows `C:\Users\runneradmin\Downloads`)
  under the header's name with their exact bytes, the second as `lj-smoke (1).bin`, and hello's panel
  refused the third (`download-refused panel`, no file). On Linux 24.04 the hook removed wry's two
  deciders (`download-deciders-removed 2`, the bar's and the page's). **Screenshots:**
  `download.png` — the toolbar's status "lj-smoke.bin saved" with the accent **Show in folder**
  button in the pill's place, between the site and open-in-browser, on Windows (**no WebView2
  download flyout**: wry's `SetHandled` is enough), macOS (traffic lights, then the status) and Linux
  (scale 1 and 2, crisp); `download-progress.png` — "Downloading lj-slow.bin…". **Informational:**
  the PDF attachment was downloaded on Windows and Linux, but **macOS showed it in the page** (wry's
  WKWebView response policy asks only `canShowMIMEType`); **closing the window during the slow
  download** stopped it on Windows (the server sent 5 of 40 chunks, then the connection closed; no
  file left) and on Linux 24.04 (cancelled; no file), but **not on macOS**: 22 chunks at the 8 s mark
  and still going, `lj-slow.bin` growing in Downloads — the WKDownload lives in WebKit's network
  process, not in the closed web view. And on macOS the toolbar went on naming the download's
  address after `--show <download>` (`bar-url …/dl-attachment` with the smoke page still shown):
  WKWebView raises no event for the page that stays (WebView2 and WebKitGTK raise a `Finished` for it,
  which also records one more visit — the Deferred follow-up about cancelled navigations, Linux too).
  **linux-22.04 (WebKitGTK 2.50.4):** the hook ran (`download-deciders-removed 0` — no handler of
  wry's was there) but the `decide-destination` handler it connected never ran, nothing was logged,
  the server sent the slow download in full, and nothing landed in `~/Downloads` (no XDG download
  dir on the runner: WebKitGTK's own default is the home folder).
- **`lj-step-15-fix` (one commit for the three):** (1) **Linux decides in an emission hook** on
  `WebKitDownload::decide-destination` (`platform/linux/downloads.rs`, moved out of `linux.rs`):
  `g_signal_add_emission_hook`, added once, runs for every download in the process before any
  handler — whatever the context, whenever the signal comes —, so the order of WebKitGTK 2.50's
  signals no longer matters; for a page download it disconnects the download's other deciders (a
  handler disconnected during an emission is skipped) and sets the destination (or cancels), and the
  class handler keeps it (`download-decide other|no-url` logs a download that isn't the page's). The
  context's `download-started` hook is gone. (2) **macOS cancels its WKDownloads when the window
  closes** (`platform/mac/downloads.rs`): wry keeps the WKDownload to itself, but gives every one its
  delegate with `-[WKDownload setDelegate:]` (a public WebKit class and selector), so that setter's
  implementation is wrapped once (objc2's `Method::set_implementation`) to note the download and
  then call the original; on close the noted downloads are cancelled (`-cancel:` with no resume
  block, through `performSelector:withObject:`; on a finished one it does nothing) and the partial
  files of those under way are removed 0.5 s later (`download-noted`, `download-cancelled <n>`).
  `platform::cancel_downloads` now takes those paths. (3) **The address of a navigation that became
  a download goes back** (`PageAddress::download`, tested: once, only for the app's own navigation,
  to no address when nothing was loaded before), so the toolbar names the page that stays. The
  smoke also lists files of the run found in the home folder or the working folder
  (`downloads.elsewhere`, informational, tidied). `npm run verify`: 615 TS tests, 358 Rust tests,
  clippy clean.
- **CI for the fix (run 39275497527): macOS and Windows' downloads right, every Linux job red.**
  macOS: the WKDownloads were noted (`download-noted` before each), **closing the window cancelled
  the slow one** — the server saw the connection close after 4 of 40 chunks, `download-cancelled 4`,
  nothing left in Downloads — and after each `--show <download>` the toolbar went back to the smoke
  page (`bar-url …/smoke.html`). Linux, 24.04 and 22.04 alike: **the hook was never added** — the
  app printed "no hook on WebKitGTK's downloads": `g_signal_lookup` finds a signal only once its
  class has been initialised, and before the first download WebKitDownload's hadn't (GLib's docs say
  so; the old lookup ran during a download) — so wry's handlers decided, putting the files in the
  working folder (`downloads.elsewhere` listed them: wry falls back to the current folder without an
  XDG download dir), which also tells what 22.04 did in the first run. macOS's smoke also failed
  one rule of the bundled run this step doesn't touch: the QR page action's panel loaded 5 s after
  `extension-opened qr` and never made its API calls — the same macOS symptom as LJ29's race flake
  (Deferred follow-ups).
- **`lj-step-15-fix2`:** `g_type_class_ref(WebKitDownload)` before the lookup (kept for the process's
  lifetime), smoke log `download-hook decide-destination` once added; `download-decide
  page|no-view|other` for each download; and a download without a web view counts as the page's
  while a page window is open (nothing else of Little Joan's downloads: the bar and Settings stay on
  the app's pages) — in case WebKitGTK 2.50 decides on a download object it hasn't tied to the view.
  GLib's `signal_emit_unlocked_R` (2.72, 22.04's) checked: emission hooks run after a RUN_FIRST class
  closure and before the handler list is even looked up, and a disconnected handler is skipped.
- **CI for fix2 (run 39280858307, delayed by GitHub's Actions incident that afternoon): all thirteen
  jobs green.** Every smoke job — Windows, macOS, `linux`, `linux-scale-2`, `linux-appimage` and
  `linux-22.04` — saved both attachments in `~/Downloads` with their bytes, refused the panel's, and
  **closing the window ended the slow download everywhere**: the server saw the connection go after
  4–5 of 40 chunks and nothing was left in Downloads (macOS `download-cancelled 4`). Linux logged
  `download-hook decide-destination` once, then `download-decide page` for each download; on 24.04
  `download-deciders-removed 2` (wry's), on **22.04 `0` — on WebKitGTK 2.50.4 no handler of wry's is on
  the download when `decide-destination` comes**, which is why the first design (a handler connected in
  `download-started`) saw nothing there. The PDF attachment: saved on Windows and Linux, shown in the
  page on macOS. Nothing turned up outside Downloads. macOS's bundled run passed this time.
- **`lj-step-15-fix3`** (after LJ16, before the merge): closing the window during a download is a smoke
  rule now (`closeFailures`: the slow download started, the server saw the engine go before its last
  chunk, nothing of it left in Downloads; 1 more test), since every engine does it.

### LJ16 — Page zoom · standard · auto

> *Round 3 adjustments (2026-10-18):* the zoom keys join LJ43's keymap — so they appear in the
> Keyboard tab and can be changed — instead of new hard-coded arms in `keys.rs`.

**Verify:** `npm run verify` green, including Rust tests for a pure `zoom.rs`: levels 50, 67,
75, 80, 90, 100, 110, 125, 150, 175, 200, 250, 300 %; zooming in or out steps to the next level
from any value, stops at the ends, and reset goes to 100 %. `keymap.rs` tests: the actions
`zoom-in`, `zoom-out`, `zoom-reset` with defaults — Windows / Linux Ctrl+= (aliases Ctrl+Shift+=
and Ctrl+numpad +), Ctrl+- (alias Ctrl+numpad −), Ctrl+0 (alias Ctrl+numpad 0); macOS Cmd+=,
Cmd+-, Cmd+0 as the View menu's Zoom In, Zoom Out, Actual Size (built from the keymap); an
override replaces the first binding and drops the aliases; the exact-match rules still hold
(Ctrl+Alt never matches). `keymap_info` lists them, so the Keyboard tab shows them. `zoom_hotkeys_enabled` appears nowhere in
`src-tauri/src`. CI green; the Linux smoke sends ctrl+equal with xdotool, and the Windows smoke
sends it with SendKeys `^=`. The page then reports a smaller `innerWidth` (about 655 instead of
720 at 110 %), and after Ctrl+0 it reports 720 again. On macOS the check is required only if the
synthetic key reached the page (the same rule as the fullscreen click).
**Commit:** `lj-step-16: page zoom with Ctrl/Cmd + / − / 0`

```text
Add page zoom. Keys only, plus a short status in the URL bar.

Design (decided with the user; do not relitigate):
- The keys are keymap actions (keymap.rs, LJ43: zoom-in, zoom-out, zoom-reset with the
  Verify's defaults and aliases) performed as PageAction::ZoomIn / ZoomOut / ZoomReset
  (page_actions.rs), so they go through the same native paths as every other page key —
  WebView2's AcceleratorKeyPressed (platform/win.rs), the GTK key hook (platform/linux.rs), the
  View menu built from the keymap (menu.rs) — and the Keyboard tab lists them with no extra
  work. They call Webview::set_zoom on the PAGE webview with the level from zoom.rs.
- Phase 19 doesn't scale pages (LJ39: like Little Arc, pages reflow), so this is the only page
  zoom. Read LJ39's and LJ40's implementation notes for anything that changed the page's bounds;
  if the user's zoom can't compose with it, stop and report.
- NEVER enable Tauri's zoom_hotkeys_enabled on joan-page. On macOS and Linux it injects a
  script into the page that needs core:webview:allow-set-webview-zoom, which is IPC for remote
  pages (Ground Rule 10; Landmines). On Windows it is WebView2's IsZoomControlEnabled, which
  stays off so our levels are the only ones.
- Zoom is per window: it survives navigations within the window, starts at 100 % on every
  summon, and is never saved. Keep the current level next to the window's other per-window
  state in bar.rs or joan.rs.
- Each change shows "Zoom 110 %" for 1.5 s in the bar's status area (from LJ15). At 100 % it
  shows "Zoom 100 %" once, then nothing.
- Phone mode is unaffected: its 600 / 640 px thresholds (LJ39) are the window's logical width,
  not the page's CSS width.

Smoke: after the wide (720×600) viewport report, send Ctrl+= (Linux: `xdotool key ctrl+equal`;
Windows: SendKeys "^="; macOS: a CGEvent key press, best-effort), and require the next viewport
report to be narrower than 720. Then send Ctrl+0 and require 720 again. Add the rules to
smoke-log.mts with tests.

Files: src-tauri/src/{zoom.rs (new, pure), keymap.rs (+ keymap/tests.rs), keys.rs (the numpad
and punctuation keys, if LJ43 didn't add them), page_actions.rs, menu.rs, bar.rs},
ui/toolbar/logic.ts (+ test) if the status needs it, scripts/ci/{smoke.mts, smoke-downloads.mts or
a smoke-zoom.mts (+ test)}, README (page keys table), CHANGELOG
([Unreleased] → Added), docs/architecture.md (the page keys table), this plan (Later: zoom
moved here; find in page stays Later).

Do NOT add Ctrl+mouse-wheel or pinch zoom, per-site zoom, or a zoom control in the bar (Later).

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ16 to ✅, add notes, and
commit: "lj-step-16: page zoom with Ctrl/Cmd + / − / 0". Push and watch CI until every job is
green (two honest attempts, then stop). This ends phase 8. Fill the PR description. Needs human
check:
- On your Mac: download a PDF from a real site. It appears in ~/Downloads under its own name,
  and "Show in folder" opens Finder with it selected.
- Download it again: you get "name (1).pdf".
- ⌘+ / ⌘− / ⌘0 zoom and reset, with the level shown briefly in the bar; Settings → Keyboard
  lists the three, and a changed zoom key works.
- The must-not-disturb list.
These checks also go into LJ19's checklist. Once every job is green, merge the PR into main with a
merge commit (Ground Rule 24), then stop and report.
```

**Implementation notes (2026-10-19):**

- **Disk:** 12 GiB free at the start of the step, 3.6 GiB after the last verify — other things on
  the Mac took about 8 GiB meanwhile (`.cache/` stayed at 3.3 GB, `.cache/target` 1.6 GB); emptying
  `.cache/npm` and `.cache/tmp` (re-created on demand) left 4.0 GiB, above the 3 GB floor.
- **`zoom.rs`** (pure, 5 tests): `LEVELS` 50 … 300 %, `step(current, In | Out | Reset)` — the next
  level up or down from any value, stopping at the ends; reset 100 % — and `factor`. One more test
  reads every `.rs` file under `src-tauri/src` and fails if Tauri's zoom-hotkeys option is named
  anywhere (the needle is built with `concat!`, so the test file doesn't hold it either).
- **The keymap** (`keymap.rs`; 4 tests in a new `keymap/zoom_tests.rs`, since `keymap/tests.rs` was
  already near 500 lines): `zoom-in`, `zoom-out`, `zoom-reset` (titles Zoom In, Zoom Out, Actual
  Size; in Settings' order after Forward), `PageAction::ZoomIn / ZoomOut / ZoomReset`. Defaults:
  Windows / Linux Ctrl+= with the aliases Ctrl+Shift+= (Ctrl++) and Ctrl+keypad +, Ctrl+- with
  Ctrl+keypad −, Ctrl+0 with Ctrl+keypad 0; macOS ⌘=, ⌘-, ⌘0, one each (a menu item has one key
  equivalent; muda turns `Equal` / `Minus` / `Digit0` into `=` / `-` / `0`). Tests: the defaults,
  exact matching (Ctrl+Alt, a plain =, Ctrl+Shift+- and Ctrl+Shift+0 match nothing; GDK's `plus`,
  `KP_Add`, `KP_0` and Windows' `VK_OEM_PLUS`, `VK_ADD`, `VK_NUMPAD0` are the keys), an override
  replacing the first binding and dropping the aliases (`""` = no key), validation (a clash with
  zoom-in's alias, the summon shortcut, a plain = on macOS) and `keymap_info`'s list — so Settings →
  Keyboard shows the three with no change of its own. `keys.rs` already had every key (LJ43).
- **Runtime** (`bar/zoom.rs`): the window's level next to its other per-window state, reset to 100 %
  in `add_webviews`, `Webview::set_zoom(level / 100)` on the page, smoke log `zoom <percent>`, and the
  status `{ kind: "zoom", percent }`. Not re-applied on navigation: WebView2's controller
  `ZoomFactor` "becomes the new default zoom … across navigations", and WKWebView's `pageZoom` and
  WebKitGTK's zoom level are properties of the view. The macOS View menu gains Zoom In, Zoom Out,
  Actual Size from the keymap (`menu-keys zoom-in=⌘=` …). Phone mode is the window's logical width,
  untouched (LJ39 / LJ40's notes: only the window's size and the agent change the page's bounds).
- **The status area, shared** (`bar/status.rs`, out of `bar/download.rs`): one `Status` enum
  (`download`, `zoom`), the last status told again when the bar's script runs. `ui/toolbar/logic.ts`:
  "Zoom 110 %" for `ZOOM_LINGER_MS` = 1.5 s (each change starts it again; "Zoom 100 %" once, then
  nothing); 4 more tests (checked input, the 1.5 s, a new change restarting it, a download under way
  coming back after it).
- **Smoke** (`scripts/ci/smoke-zoom.mts` + test, 4 tests — a file of its own rather than
  `smoke-log.mts`, which is past 400 lines): in the main run after `wide.png`, Ctrl+= (xdotool
  `ctrl+equal`, SendKeys `^=`, a ⌘= key event on macOS) → `zoom 110`, `zoom.png` (the status), the
  page's next report narrower than 720 px; Ctrl+0 → `zoom 100` and 720×560 again. Required on
  Windows and Linux; on macOS once a `zoom` line shows the key arrived.
- Docs: README (the keys table, a line on zoom), CHANGELOG (Added), docs/architecture.md (page keys,
  the keymap, Page zoom, the status area, file map, smoke log, CI), this plan (Later).
- `npm run verify`: 623 TS tests, 367 Rust tests, clippy clean.
- **CI (run 39283498532, with `lj-step-15-fix2` under it): all thirteen jobs green on the first try.**
  The zoom check on every job: Ctrl+= (macOS ⌘=, whose key event reached the View menu) logged `zoom
  110`, the page reported **655×509 on Windows and 654×509 on macOS and the four Linux jobs** (dpr
  1.1 — the engines' zoom, not the window), and Ctrl+0 / ⌘0 logged `zoom 100` and brought 720×560
  back. macOS's menu was built with `menu-keys zoom-in=⌘=`, `zoom-out=⌘-`, `zoom-reset=⌘0`. The
  downloads run (fix2's code) passed on all six smoke jobs again. **Screenshots:** `zoom.png` — the
  toolbar's status "Zoom 110 %" in the pill's place on Windows (the page at 655 px), macOS (654 px,
  traffic lights in the toolbar) and Linux (scale 1 and 2); `settings-keyboard.png` — the Keyboard tab
  as before, Zoom In / Zoom Out / Actual Size below the fold after Forward (`keymap_info`'s list, tested).
  Disk: 6.3 GiB free with the artifacts downloaded; deleted after looking.

### LJ17 — Wayland-native global shortcut · strong · auto

> Phase 9. Branch `lj/p9-wayland-shortcut` from the latest `main`; the PR goes into `main`.
>
> *Round 3 adjustments (2026-10-18):* the global shortcut's recorder lives in Settings →
> Keyboard (LJ44) now, and LJ41's icon rule asks whether the shortcut can work.

**Verify:** `npm run verify` green, including Rust tests for the pure parts:
- accelerator → XDG shortcut trigger (`CommandOrControl+Shift+Space` → `CTRL+SHIFT+space`; Alt →
  `ALT`, Super → `LOGO`; keys the spec can't express → `None`);
- the backend decision (not Linux, or an X11 session → global-hotkey as now; a Wayland session
  with the GlobalShortcuts portal → portal; a Wayland session without it → global-hotkey on
  XWayland as now, plus the `--toggle` hint);
- the smoke-log events (`shortcut-registered <accel> portal`, `shortcut-portal-unavailable`).

`cargo tree -i zbus` shows one `zbus` 5, shared by `ashpd` and the existing dependencies. CI green
on all three OSes. The X11 smoke is unchanged (it still logs `shortcut-registered` from
global-hotkey). The portal path can't run in CI (no portal backend under Xvfb), so it is the
user's check.
**Commit:** `lj-step-17: Wayland-native global shortcut through the XDG GlobalShortcuts portal`

```text
global-hotkey's Linux backend is X11 only, so on a Wayland session the shortcut fires only while
an XWayland window has focus (Landmines). The XDG GlobalShortcuts portal is the Wayland way: KDE
Plasma has it, GNOME since 48, so Ubuntu 26.04 has it and 24.04 (GNOME 46) doesn't.

Design (decided with the user; do not relitigate):
- Add `ashpd` (current version, default features off, the tokio feature to match
  tauri::async_runtime; it must use the zbus 5 already in Cargo.lock) as a Linux-only
  dependency.
- In shortcut.rs (or a new src-tauri/src/platform/portal.rs if shortcut.rs would pass ~400
  lines): on Linux, when XDG_SESSION_TYPE is "wayland" (or WAYLAND_DISPLAY is set), try the
  portal first. Create a session, bind one shortcut with id "toggle", description "Summon or
  dismiss Little Joan", and the preferred trigger converted from the settings accelerator.
  Listen for Activated → joan::dispatch(Command::Toggle). Keep the session alive for the life
  of the process. Log `shortcut-registered <trigger> portal`. If the portal isn't there or
  refuses: log `shortcut-portal-unavailable` and do exactly what v0.1 does.
- The desktop owns the final key. It may show its own dialog the first time and may change the
  trigger. Take the trigger it reports and show that in the start-page hint and in Settings.
- Host apps and app ids: GNOME's portal identifies a non-sandboxed app by its desktop file id.
  Check what Tauri's .deb installs (dpkg-deb -c on the CI artifact). If it is "Little Joan.desktop"
  (with a space), the portal may not match it. Then use ashpd's host-app registration with
  io.github.memoliertorerr.littlejoan, and install the desktop file under that id if that's
  needed. Record what you find in Landmines.
- Settings → Keyboard (LJ44's "Summon Little Joan" row, ui/settings/keyboard.ts and
  keyboard-logic.ts) and General's shortcut line: on the portal path, show the trigger the portal
  reports instead of the recorder, plus "Change it in your desktop's keyboard settings". On other
  paths nothing changes.
- StartInfo.waylandNoShortcut (LJ12) becomes false once the portal registers the shortcut, and
  LJ41's shortcut_works counts a portal-registered shortcut as working: call
  tray::apply_visibility after the portal answers, so a user who hid the icon keeps it hidden on
  GNOME 48+ / KDE, and sees it come back where the portal is missing.

Files: src-tauri/Cargo.toml (+ Cargo.lock), src-tauri/src/{shortcut.rs and/or
platform/portal.rs, lib.rs, commands.rs or its child modules, tray.rs, welcome.rs, smoke.rs},
ui/settings/{keyboard.ts, keyboard-logic.ts (+ test)}, ui/settings.ts, possibly
src-tauri/tauri.conf.json and a desktop file template (LJ45 later adds the link handler to the
same template), README (Wayland), CHANGELOG
([Unreleased] → Added), docs/architecture.md (global shortcut), this plan (Later: the Wayland
row moved here).

Do NOT remove the `--toggle` fallback or the X11 path.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ17 to ✅, add notes, and
commit: "lj-step-17: Wayland-native global shortcut through the XDG GlobalShortcuts portal".
Push, open the phase-9 PR into main as a draft, and watch CI until every job is green (two honest
attempts, then stop). This ends phase 9. Needs human check (they also go into LJ19's checklist):
- On a Wayland session with GNOME 48+ or KDE Plasma: install the .deb and launch. The desktop
  asks once to allow the shortcut (or shows it in its shortcut settings). The shortcut then
  summons and dismisses the window, with no XWayland window focused.
- On Ubuntu 24.04 Wayland: the welcome shows the `--toggle` hint, as before.
- On X11, nothing changed.
- With the icon hidden (LJ41) on GNOME 48+: it stays hidden; on 24.04 Wayland it comes back.
- The must-not-disturb list.
Once every job is green, merge the PR into main with a merge commit (Ground Rule 24), then stop and
report.
```

**Implementation notes (2026-10-19):**

- **Disk:** 6.1 GiB free at the start of the phase (other activity on the owner's Mac), 5.5 GiB
  after verify; `.cache/target` 2.1 GB (ashpd and zbus built once for macOS by the type-check
  below), then `cargo clean`.
- **`ashpd` 0.13.13** (the current version; default features off, `tokio` and — new in 0.13 —
  `global_shortcuts`), Linux only. Cargo.lock gains only `ashpd`; its dependencies were all there
  (enumflags2, futures-util, getrandom 0.4, serde, tokio, zbus), tokio picks up two more edges to
  crates already locked (signal-hook-registry, tracing). **`cargo tree -i zbus --target all`: one
  `zbus v5.13.2`, used by ashpd, tauri-plugin-opener and tauri-plugin-single-instance.** ashpd's
  `tokio` feature turns on zbus's, which then holds for those plugins too — read in zbus 5.13.2:
  with `tokio` its tasks go to `tokio::spawn` and its blocking API (single-instance's, opener's)
  runs inside a multi-threaded tokio runtime of zbus's own (`utils::block_on`), so the
  single-instance callback now runs on that runtime's threads. Nothing on that path calls
  `block_on` (Little Joan's callback only dispatches to tauri's runtime); the opener already moves
  its blocking calls to a thread of their own for this reason. Landmines has it. ashpd 0.13 says
  `rust-version = "1.87"` (Cargo's MSRV-aware resolver would have picked 0.12.3 for the crate's
  1.85; tauri 2.12.4 already needs 1.90, so nothing changes for the pinned 1.99.1).
- **The pure part** (`src-tauri/src/shortcut/backend.rs`, 7 tests): `Backend` — `Hotkey` (not
  Linux, or an X11 session), `Portal` (a Wayland session whose portal took the shortcut),
  `HotkeyOnXwayland` (a Wayland session without it: global-hotkey on XWayland as before, plus the
  `--toggle` hint) — from `backend(wayland_session, &PortalState)`; `PortalState` (`None`,
  `Asking`, `Registered { trigger }`); `works` (the icon rule: global-hotkey with a shortcut outside
  Wayland, the portal once the desktop gave it a key); `trigger(accelerator)` — the XDG shortcuts
  spec's syntax, read as Linux reads it (`CommandOrControl+Shift+Space` → `CTRL+SHIFT+space`, Alt →
  `ALT`, Super → `LOGO`), the key from the same parser registration uses and its xkb name
  (`keysym`: letters, digits, F1–F35, the keypad, punctuation, navigation, media keys); `None` for
  what the spec can't name (Caps Lock and Num Lock, which are its modifiers; Fn, the Intl keys) or
  an accelerator that doesn't parse — the portal is then asked without a preference; the smoke-log
  lines `shortcut-registered <trigger> portal` and `shortcut-portal-unavailable <why> [<error>]`
  (tested through `smoke::format_line`). The welcome rule (`welcome_waits`) too. `shortcut.rs`
  moved its tests to `shortcut/tests.rs` to stay under 400 lines.
- **The portal** (`src-tauri/src/platform/portal.rs`, Linux only): in a Wayland session
  `shortcut::register_at_launch` sets `PortalState::Asking` and spawns it on tauri's runtime. It
  opens its own session-bus connection, registers it as `io.github.memoliertorerr.littlejoan` with
  the portal's host registry (`ashpd::register_host_app_with_connection`, before any other portal
  call, as the registry wants; on failure it logs `shortcut-portal-host unregistered <error>` and
  takes a fresh connection, so a refused registration can't stick to the one it uses), makes the
  GlobalShortcuts proxy and a session (all within 15 s: D-Bus activation of a portal that hangs
  must not leave the app without a shortcut), subscribes to `Activated` and `ShortcutsChanged`,
  then binds `toggle` / "Summon or dismiss Little Joan" with the preferred trigger — without a time
  limit, since GNOME and KDE may ask the user. Registered: `shortcut-registered <trigger> portal`,
  `Activated` for `toggle` → `joan::dispatch(Command::Toggle)`, `ShortcutsChanged` →
  `shortcut-portal-changed <trigger>` and the icon rule again; the session lives as long as the
  process. Anything else — `no-bus`, `no-portal` (`PortalNotFound`, or D-Bus's ServiceUnknown /
  UnknownInterface / UnknownMethod / UnknownObject), `refused`, `cancelled`, `timeout`,
  `no-shortcut`, `ended` (a signal stream ended later), `failed` (including a panic: the work runs in
  a task of its own, since ashpd asserts on some answers) — logs `shortcut-portal-unavailable` and
  goes through `shortcut::portal_answered(None)`: global-hotkey registers exactly as v0.1 did
  (`shortcut-registered <accel>`), the icon rule runs. No timer crate: the 15 s limit is a sleeping
  thread of tauri's blocking pool, raced with `poll_fn`; streams are read through
  `zbus::export::futures_core::Stream` (no new dependency for `StreamExt`).
- **The desktop file id — what the `.deb` installs** (`Little Joan_0.2.0-beta.3_amd64.deb` of
  `main`'s run 39290492192, listed with `ar` + `tar` on the Mac): `/usr/share/applications/Little
  Joan.desktop` (Tauri names it after the product), `usr/bin/little-joan`, three icons. Read in
  xdg-desktop-portal's source (main): a host app's id comes from its systemd unit
  (`app-gnome-Little\x20Joan-<n>.scope` → "Little Joan" when the desktop started it; nothing from a
  terminal), GlobalShortcuts' CreateSession refuses an app without one ("An app id is required"),
  and the host registry's `Register` creates the app info with `REQUIRE_GAPPINFO` — the app id
  needs a desktop file of that name (its docs: it "must be able to match the basename of a .desktop
  file"). So the `.deb` now also installs `src-tauri/linux/io.github.memoliertorerr.littlejoan.desktop`
  (`bundle.linux.deb.files`; the same entry with `NoDisplay=true`, so the app list keeps one Little
  Joan), and the portal registers that id. Tauri's own file and its name are unchanged (renaming
  the product would orphan the login entries, Landmines). The AppImage installs no desktop file, so
  there registration fails and GNOME may refuse; KDE and an AppImage started from a desktop entry
  may still work. Landmines has it.
- **The rest of the app:** `AppState` gains `portal` and `welcome_after_portal`; `tray::shortcut_works`
  now takes the session and the portal state (a portal shortcut with a key counts as working, so a
  hidden icon stays hidden on GNOME 48+ / KDE and comes back where the portal is missing), as do
  the quick commands' `iconForced`; `StartInfo.waylandNoShortcut` and `get_settings`'s are
  `Backend::toggle_hint` (false once the portal holds the shortcut); `start_info` and `keymap_info`
  report `portalShortcut: { trigger } | null` (`commands/keys.rs`); `save_settings` swaps nothing
  while the portal holds (or is asked for) the shortcut; `welcome.rs`'s detection is named for what
  it detects (`wayland_session`). **A first launch's welcome in a Wayland session waits for the
  portal's answer** (`shortcut::welcome_now` / `portal_answered`, race-free under the state lock),
  so on GNOME 48 it opens after the desktop's dialog and names its key — an addition the prompt
  didn't ask for; without it the welcome would have shown the `--toggle` hint while the dialog was
  up. X11, Windows and macOS: nothing waits.
- **UI:** the welcome (`ui/command/welcome.ts`, where LJ22 moved the start page's hint): "Your
  desktop's shortcut summons and dismisses this window: <trigger>." (the desktop's words in a
  `<kbd>`), or, with no key yet, to choose one in the desktop's keyboard settings. Settings →
  Keyboard (`keyboard.ts`, `keyboard-logic.ts` + 3 tests): with `portalShortcut` the summon row
  shows the desktop's key as text instead of the recorder, no Reset, the note "Change it in your
  desktop's keyboard settings."; its problems aren't judged (the desktop's key can never block
  Save — a saved shortcut that doesn't parse would otherwise have, with no recorder to fix it), and
  Reset all leaves the preferred trigger alone; General's line shows the key with the same note
  instead of **Change…**; the icon note says "press your shortcut". Other paths: nothing changes.
- **Files beyond the prompt's list** (each needed by what the prompt asks): `state.rs` (the two
  fields), `platform/mod.rs` (the module and `ask_portal`), `commands/quick.rs` (the icon rule's
  new input), `ui/command/welcome.ts` + test (the start page's hint lives there since LJ22),
  `ui/settings.html` (a span for the desktop's key, General's note), `src-tauri/linux/…desktop`.
  `smoke.rs` needed no change (the events are `backend.rs`' constants). No CI workflow or smoke
  harness change: the portal path can't run in CI (no portal backend under Xvfb).
- **Checked on the Mac without launching anything:** `platform/portal.rs` compiles only on Linux, so
  with `ashpd` made a dependency for every target and the module enabled on macOS for a moment
  (then restored from copies), `cargo check` and `cargo clippy --all-targets` passed on it (one
  unused import fixed) — ashpd and zbus build on macOS too.
- `npm run verify`: 628 TS tests, 376 Rust tests, clippy clean (the Linux code compiles only in
  CI's verify).
- **CI (run 39297748893): all thirteen jobs green on the first try** — verify on Ubuntu compiled
  and clippy-checked `platform/portal.rs`, and every smoke job's single-instance forwarding (now on
  zbus's own tokio runtime) worked as before. **The `.deb`** (listed from the artifact):
  `usr/share/applications/Little Joan.desktop` as before and the new
  `io.github.memoliertorerr.littlejoan.desktop` (`NoDisplay=true`); `usr/bin/little-joan` 7.72 MB
  (7.40 MB on `main`: ashpd and zbus on tokio). **The portal path's fallback, in the tray run's
  Wayland launch** (`XDG_SESSION_TYPE=wayland` under Xvfb), on all four Linux jobs:
  `tray-visible true`, then `shortcut-portal-host unregistered …`, `shortcut-portal-unavailable
  no-portal …`, `shortcut-registered CommandOrControl+Shift+Space` (global-hotkey, as v0.1) — on
  `linux`, `linux-scale-2` and `linux-22.04` the runner's xdg-desktop-portal answered but offers
  neither `org.freedesktop.host.portal.Registry` nor `org.freedesktop.portal.GlobalShortcuts`
  (ashpd's PortalNotFound); on `linux-appimage` (no `.deb` dependencies installed) there is no
  portal at all (`org.freedesktop.portal.Desktop was not provided by any .service files`). No
  other launch on any job logged a portal line; X11, Windows and macOS logged
  `shortcut-registered CommandOrControl+Shift+Space` as before. **Screenshots** (no new ones; the
  ones this step could touch): `settings-keyboard.png` — the summon recorder (Ctrl+Shift+Space,
  ⌘⇧Space) and the keys as before; `settings-general.png` / `settings-general-tray.png` — the
  shortcut line with **Change…** and the icon note naming the shortcut; `welcome.png` — "Ctrl+Shift+Space
  summons and dismisses this window." (Linux scale 2). Disk: 9.2 GiB free with the artifacts
  downloaded; deleted after looking.
- **CI on the docs commit (run 39301164697): all thirteen jobs green after one re-run** — the first
  attempt's `smoke (windows)` failed LJ16's zoom check: Ctrl+= arrived (`zoom 110`, the page at
  655×509) but the SendKeys Ctrl+0 that followed never did (no `zoom 100` within 15 s), in a check
  this phase doesn't touch, with the same code green in run 39297748893; `gh run rerun --failed`
  once: green (Deferred follow-ups).

### LJ18 — macOS Developer ID signing and notarization · strong · auto

> Phase 10. Branch `lj/p10-macos-signing` from the latest `main`; the PR goes into `main`.
> **Runs only if the owner has done the human pre-step below.** Check with
> `gh secret list --repo memoliertorerr/LittleJoan` (it lists names only, never values; never ask
> for, read or print a value). *Round 3 (the owner's decision, 2026-10-18):* if any of the six
> names is missing, **skip this phase** — don't create the branch; flip LJ18's row to
> `⏭ skipped — no Apple signing secrets on <date>` and say so in the overall status, both in LJ19's
> release PR (nothing is ever committed to `main` directly); LJ19's checklist then keeps the
> documented Gatekeeper step for the ad-hoc-signed DMG. Then go on with LJ19.

**Human pre-step (the user, once, about 20 minutes; the agent never sees these values):**
1. developer.apple.com → Certificates, IDs & Profiles → Certificates → + → **Developer ID
   Application** (only the Account Holder can create one). Make the certificate signing
   request in Keychain Access → Certificate Assistant → Request a Certificate From a Certificate
   Authority (save it to disk). Download the `.cer` and double-click it.
2. In Keychain Access → My Certificates, export "Developer ID Application: …" (with its private
   key) as a `.p12` with a strong password.
3. App Store Connect → Users and Access → Integrations → App Store Connect API → Team Keys → +,
   with Developer access. Download the `.p8` (it downloads only once). Note the Key ID and the
   Issuer ID.
4. Add the repository secrets (GitHub → Settings → Secrets and variables → Actions, or
   `gh secret set <NAME> --repo memoliertorerr/LittleJoan`):
   - `APPLE_CERTIFICATE` = `base64 -i DeveloperID.p12`
   - `APPLE_CERTIFICATE_PASSWORD`
   - `APPLE_SIGNING_IDENTITY` = exactly what `security find-identity -v -p codesigning` prints,
     e.g. `Developer ID Application: <Name> (<TEAMID>)`
   - `APPLE_API_ISSUER`
   - `APPLE_API_KEY` = the Key ID
   - `APPLE_API_KEY_P8` = the `.p8` file's contents
5. Keep the `.p12`, its password and the `.p8` in a password manager, and delete the loose
   copies.

**Verify:** `npm run verify` green. CI, on branches of this repository: `package (macos)` signs
with the Developer ID. `codesign -dv --verbose=2` shows `Authority=Developer ID Application: …`,
the hardened runtime flag and a secure timestamp. With notarization on (pushes to `main`, and
`workflow_dispatch` with `notarize=true`): the DMG users download is notarized and stapled
(`xcrun stapler validate` passes on the DMG and the app). On the smoke (macos) job,
`spctl -a -vv -t exec` on the installed app says `accepted` with `source=Notarized Developer ID`,
and `spctl -a -vv -t open --context context:primary-signature` on the DMG is accepted. Without
the secrets (fork PRs), the job falls back to ad-hoc signing and stays green. No secret value
appears in any log. Every existing smoke check still passes.
**Commit:** `lj-step-18: sign the macOS app with Developer ID and notarize the DMG in CI`

```text
macOS users currently need `xattr -dr com.apple.quarantine` or System Settings → Open Anyway,
because the build is ad-hoc signed. The user has an Apple Developer account and has added the
secrets listed in this step's pre-step. Sign and notarize in CI. You never see, print, or
handle the secret values (Ground Rule 16).

1. Read how the Tauri CLI in node_modules/@tauri-apps/cli (and tauri-bundler's source in the
   cargo registry, if it's there) treats APPLE_CERTIFICATE, APPLE_CERTIFICATE_PASSWORD,
   APPLE_SIGNING_IDENTITY, APPLE_API_ISSUER, APPLE_API_KEY and APPLE_API_KEY_PATH, and how the
   environment interacts with tauri.conf.json's `"signingIdentity": "-"`. Write what you find
   into the step's notes before changing anything.
2. .github/workflows/ci.yml, package (macos) only:
   - When APPLE_CERTIFICATE is non-empty, pass the signing variables to the "Build installers"
     step only, through `env:` from secrets. Never `echo` them, and never write them to
     $GITHUB_ENV. Write APPLE_API_KEY_P8 to a file under $RUNNER_TEMP with mode 600, set
     APPLE_API_KEY_PATH to it, and delete the file in an `if: always()` step.
   - Notarize only on push to main and on workflow_dispatch with a new boolean input `notarize`.
     PR builds are Developer ID-signed but not notarized, to save time and Apple submissions.
   - Without the secrets, keep today's ad-hoc build.
   - Make the DMG users download pass Gatekeeper: if Tauri notarizes only the .app, sign the DMG
     (codesign --timestamp), notarize it (xcrun notarytool submit --wait) and staple it. Don't
     do anything twice that Tauri already does.
   - "Inspect the macOS app": print codesign -dv --verbose=2 and, when notarized,
     `xcrun stapler validate` for the app and the DMG.
3. Smoke (macos): when the build was notarized, run the spctl checks from Verify on the installed
   app and on the DMG, and fail if they aren't accepted. Otherwise log that it was skipped.
   Everything else is unchanged; the hardened runtime must not break WKWebView, downloads
   (LJ15) or start at login (LJ11).
4. tauri.conf.json: keep "-" as the fallback if the environment overrides it; otherwise make
   the fallback explicit in the workflow. Add entitlements only if a check proves one is needed,
   and record why.
5. Docs: README (macOS install: no xattr step for notarized builds; keep it for builds from
   source), CHANGELOG ([Unreleased] → Changed), CONTRIBUTING (forks build ad-hoc), SECURITY
   (official macOS builds are notarized), docs/architecture.md (Packaging), this plan (Later:
   macOS signing moved here; Windows stays pending SignPath).

Do NOT touch Windows or Linux signing, and don't add the updater.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ18 to ✅, add notes, and
commit: "lj-step-18: sign the macOS app with Developer ID and notarize the DMG in CI". Push, open
the phase-10 PR as a draft (base: lj/p9-wayland-shortcut, or main), let CI run, then run
`gh workflow run ci.yml --repo memoliertorerr/LittleJoan --ref lj/p10-macos-signing -f notarize=true`
and watch both runs until green (two honest attempts, then stop). This ends phase 10. Needs
human check (it also goes into LJ19's checklist): download the DMG artifact of the notarized run
with a browser (so it gets quarantined), open it, drag Little Joan to Applications and launch it.
There should be no Gatekeeper warning. The hardened runtime must not break round 3's features
either: the smoke's fullscreen, tray, quick-command and keymap runs stay green. Once every job is green,
merge the PR into main with a merge commit (Ground Rule 24), then stop and report.
```

**⏭ Skipped (2026-10-19):** `gh secret list --repo memoliertorerr/LittleJoan` listed no secrets at all,
so none of the six Apple names exists. No `lj/p10-macos-signing` branch was made and nothing changed;
the macOS build stays ad-hoc signed (`signingIdentity: "-"`), the README keeps its Gatekeeper
step, and LJ19's checklist keeps it too. The owner decided to record the skip with the next phase's
PR (phase 10's branch `lj/p10-arm64`, which carries LJ18.1) instead of LJ19's. This step runs
unchanged once the owner has done its pre-step.

### LJ18.1 — Linux ARM64 builds · strong · auto

> **Added mid-plan 2026-10-19** — the owner's decision: they have an Apple silicon Mac and will test
> Ubuntu 22.04 (ARM64) in Parallels at LJ19's checkpoint, so Little Joan needs arm64 Linux builds.
> Phase 10's branch `lj/p10-arm64` from the latest `main` (LJ18 is skipped, its first commit records
> that); the PR goes into `main`, and the agent merges it once every job is green (Ground Rule 24).

**Verify:** `npm run verify` green. CI: two new package jobs on GitHub's arm64 runners —
`package (linux-arm64)` on ubuntu-22.04-arm builds only the `.deb` (its architecture `arm64`, the
runner's, and its binary's glibc floor in the summary) and `package (linux-appimage-arm64)` on
ubuntu-24.04-arm only the AppImage (its app an aarch64 ELF, the bundled WebKitGTK ≥ 2.52 or the job
fails, the glibc floor and the size in the summary); artifacts `little-joan-linux-arm64` and
`little-joan-linux-appimage-arm64`, each with its own cache key. Three new smoke jobs run the whole
`smoke.mts`, like their x86_64 twins: `linux-arm64` (the arm64 `.deb` on ubuntu-24.04-arm, WebKitGTK
2.52, fullscreen on), `linux-22.04-arm64` (the arm64 `.deb` on ubuntu-22.04-arm, WebKitGTK 2.50.4,
`LJ_SMOKE_FULLSCREEN=off`: `fullscreen-off <version>` and no fullscreen, as LJ14.1) and
`linux-appimage-arm64` (the aarch64 AppImage on ubuntu-24.04-arm, under the release's name
`LittleJoan-<v>-linux-aarch64.AppImage`, extracted and run without FUSE). No scale-2 arm64 job. The
x86_64 jobs stay exactly as they were (their checks only gain the architecture checks).
**Commit:** `lj-step-18.1: Linux ARM64 builds — arm64 .deb (22.04) and aarch64 AppImage (24.04), smoke-tested on ARM runners`

```text
The owner has an Apple silicon Mac and will test Ubuntu 22.04 (ARM64) in Parallels at LJ19's
checkpoint. Build and smoke-test arm64 Linux packages natively on GitHub's arm64 runners (free for
public repositories); never cross-build or build them locally. CI and docs only: no app code, no
new dependencies.

1. .github/workflows/ci.yml, package job: two entries — `linux-arm64` (ubuntu-22.04-arm, bundles
   deb: the same glibc floor as `linux`, installs on Ubuntu 22.04+ / Debian 12+) and
   `linux-appimage-arm64` (ubuntu-24.04-arm, bundles appimage: bundles WebKitGTK 2.52 like
   `linux-appimage`). Artifacts little-joan-linux-arm64 and little-joan-linux-appimage-arm64; the
   cache key per entry name (so per architecture). The .deb and AppImage inspections run for every
   build of their kind (key them on `bundles`, not the name): the bundled WebKitGTK ≥ 2.52 check, the
   glibc floor and the sizes in the summary, plus a check that each package is built for the
   runner's architecture. If a runner image lacks rustup, install it into .cache/ with
   scripts/setup-rust.sh.
2. Smoke: three entries, each exactly like its x86_64 twin — `linux-arm64` (the arm64 .deb on
   ubuntu-24.04-arm), `linux-22.04-arm64` (the arm64 .deb on ubuntu-22.04-arm, fullscreen "off":
   LJ14.1's rules, WebKitGTK 2.50.4 there) and `linux-appimage-arm64` (the aarch64 AppImage on
   ubuntu-24.04-arm, APPIMAGE_EXTRACT_AND_RUN=1 NO_CLEANUP=1). A field per entry says which kind of
   installer it takes (deb / appimage), so the install steps don't list artifact names; the
   AppImage's copy and library check use `uname -m` (its release name, `<machine>-linux-gnu`). No
   scale-2 arm64 job.
3. If the AppImage tooling can't build or run on aarch64 (linuxdeploy, FUSE, plugins): two honest
   attempts, then ship the .deb only and record why under Deferred follow-ups / Landmines.
4. Release names: LittleJoan-<ver>-linux-arm64.deb and LittleJoan-<ver>-linux-aarch64.AppImage —
   the local CLAUDE.md "Release process" (gitignored; edit in place, never commit), LJ19's prompt.
5. Docs: README (Install: Linux ARM64), CHANGELOG ([Unreleased] → Added), docs/architecture.md
   (Packaging table, CI jobs), CONTRIBUTING if it lists the jobs, this plan (tracker, this section,
   notes with CI evidence, Landmines for anything ARM-specific, Later: Linux ARM64).

Run `source scripts/env.sh && npm run verify`. If green, commit "lj-step-18.1: Linux ARM64 builds —
arm64 .deb (22.04) and aarch64 AppImage (24.04), smoke-tested on ARM runners", push, open the PR
"Linux ARM64 builds; LJ18 skipped (no signing secrets)" as a draft, and watch CI. Look at the arm64
smoke screenshots (test-results/ci/, then delete them). Once every job is green: fill the PR
(Needs human check: install the arm64 .deb in Ubuntu 22.04 ARM in Parallels on the owner's Mac —
summon, a page, phone mode, a video's fullscreen button does nothing and nothing crashes), mark it
ready, merge it with a merge commit, check main's CI, then stop. Don't start LJ19.
```

**Implementation notes (2026-10-19):**

- **Disk:** 8.2 GiB free at the start of the step and after verify; `.cache/target` 1.1 GB. Nothing
  for ARM was built on the Mac (CI only).
- **Package** (`.github/workflows/ci.yml`): `linux-arm64` (ubuntu-22.04-arm, `--bundles deb`) and
  `linux-appimage-arm64` (ubuntu-24.04-arm, `--bundles appimage`), natively — Tauri doesn't
  cross-build an AppImage. Their artifacts are `little-joan-<name>` and their cache keys
  `package-<name>-…`, as for every entry, so per architecture. The inspections now key on the
  entry's `bundles`, not its name, so the x86_64 and arm64 builds of a kind run the same checks; both
  gained one: the `.deb`'s `Architecture` must be the runner's (`dpkg --print-architecture`), and the
  AppImage's `little-joan` an ELF for the runner's machine (`file`: `x86-64` / `ARM aarch64`). The
  summary lines name the architecture. "Rust toolchain" installs rustup into `.cache/` with
  `scripts/setup-rust.sh` (which already knew `Linux-aarch64`) only when an image has none — the
  arm64 runners are partner images; on every other runner nothing changes.
- **Smoke:** each Linux entry says which installer it takes (`installer: deb` / `appimage`), and the
  install steps and the bare-WebKitGTK diagnostics key on that instead of artifact names (the x86_64
  entries take what they took before). The AppImage's release name and its library check use
  `uname -m` (`LittleJoan-<v>-linux-<machine>.AppImage`, `usr/lib/<machine>-linux-gnu`) — on x86_64
  exactly the old values. `linux-arm64`, `linux-22.04-arm64` (`fullscreen: "off"`) and
  `linux-appimage-arm64` copy their twins' settings; no harness change was needed (`smoke.mts` and
  `smoke-linux.mts` have nothing architecture-specific: `/proc`, xdotool, ImageMagick, gdb).
- CONTRIBUTING doesn't list the jobs (it says CI builds and smoke-tests every installer), so it is
  unchanged.
- `npm run verify`: 628 TS tests, 376 Rust tests, clippy clean (no code changed).
- **CI (run 39321011322): all eighteen jobs green on the first try** — verify ×3, package ×6, smoke
  ×9; 20 minutes in all, the arm64 jobs no slower than their twins (cold caches: the arm64 `.deb`
  4½ minutes, the AppImage 8½ — 3½ of them linuxdeploy —, each arm64 smoke about 7). The arm64
  runners already have rustup (the toolchain synced as `1.99.1-aarch64-unknown-linux-gnu`), so the
  fallback didn't run. **`package (linux-arm64)`:** `Little Joan_0.2.0-beta.3_arm64.deb`, **3.03
  MiB** (3,181,616 bytes; the amd64 one 3.15 MiB), `Architecture: arm64`, `usr/bin/little-joan` an
  `ELF 64-bit LSB pie executable, ARM aarch64`, glibc floor **2.34** (as amd64). **`package
  (linux-appimage-arm64)`:** `Little Joan_0.2.0-beta.3_aarch64.AppImage`, **148.62 MiB** (155,834,888
  bytes; x86_64's 156.92 MiB), its app an aarch64 ELF, bundling **WebKitGTK 2.52.6**
  (`usr/lib/libwebkit2gtk-4.1.so.0`, from `libwebkit2gtk-4.1-0 2.52.6-0ubuntu0.24.04.1`), glibc floor
  **2.39**; Tauri fetched `AppRun-aarch64` (tauri-apps/binary-releases `apprun-old`),
  `linuxdeploy-aarch64.AppImage` (`linuxdeploy-07333c6`) and `linuxdeploy-plugin-appimage-aarch64`
  (`continuous`), and `AppRun` is the same linuxdeploy bash script with the GStreamer and GTK hooks
  as on x86_64. Nothing about the AppImage tooling needed a workaround on aarch64.
- **The arm64 smoke jobs passed every rule of the full smoke**, as their twins: codecs H.264 / VP9 /
  AV1 through MSE on all three; no WebKit helper left after hibernating (0 at 2–60 s, `[0,0,0]` per
  cycle; 2 helpers with a page); the login entry pointing at `/usr/bin/little-joan`, or for the
  AppImage at `…/LittleJoan-0.2.0-beta.3-linux-aarch64.AppImage` (the release's name), and gone after
  `off`. `linux-arm64` and `linux-appimage-arm64` (WebKitGTK 2.52.6): window and page fullscreen
  after the click, every fullscreen-run path entered, held ~2.15 s and left by the page's own Exit;
  no `fullscreen-off`. `linux-22.04-arm64` (WebKitGTK **2.50.4**, as amd64's 22.04): `fullscreen-off
  2.50.4` in every launch that opened a page (29 lines), no fullscreen, every path `entered=0`
  (`no-fullscreen-api`, the phone player's `enter-error:invalidstateerror`), no abort. The AppImage's
  library check found only the bundled fluidsynth MIDI plugin's `libjack.so.0` /
  `libpipewire-0.3.so.0` missing — exactly as on x86_64 — and gdb attached to the app in all 21
  launches. Own memory at rest (private dirty): the arm64 `.deb` 20.1 MB on 24.04 and 22.0 MB on
  22.04, the AppImage 52.5 MB (the bundled libraries, as on x86_64). **Screenshots** (downloaded,
  looked at, deleted): `fullscreen.png` — on `linux-arm64` and `linux-appimage-arm64` the green stage
  fills the whole 1280×800 screen, the page reporting 1280×800; on `linux-22.04-arm64` the 480×600
  window as it was, toolbar with the Phone pill above "Go fullscreen"; `fs/element-390.png` (AppImage)
  — the element fills the screen with its Exit; `command-bar.png` (`linux-arm64`) — the command bar
  with the smoke page as the top site; `narrow.png` (22.04 arm64) — 390×560 with the phone agent.

### LJ19 — Release v0.2.0 · standard · checkpoint

> Phase 11. Branch `lj/release-0.2.0` from the latest `main`, once phases 18, 19, 20 and 7–10 are
> merged (or LJ18 recorded as skipped). *Round 3 adjustments (2026-10-18):* the phases are no
> longer stacked, so this is the only open PR; the agent may merge it once it is green (the owner's
> decision); tagging and publishing still wait for the owner's word. It is a **checkpoint**
> (Ground Rule 24).

**Verify:** `npm run verify` green. Version `0.2.0` (from `0.2.0-beta.2`) in `package.json` (+
lock), `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` (+ `Cargo.lock`). `CHANGELOG.md` has
`[0.2.0] — <date>` built from `[Unreleased]`, with a fresh empty `[Unreleased]` above it and the
link references at the bottom. The README points at the v0.2.0 assets. CI green on the release
PR. **Checkpoint:** the owner ticks the checklist. Only after the owner says to release: `main`'s
CI green for the merge commit, the assets renamed (`LittleJoan-0.2.0-windows-x64-setup.exe`,
`-macos-universal.dmg`, `-linux-amd64.deb`, `-linux-x86_64.AppImage`, and since LJ18.1
`-linux-arm64.deb`, `-linux-aarch64.AppImage`), `SHA256SUMS.txt`,
`gh release create v0.2.0 --target <that main commit> --latest`, and then the release, its tag
and one downloaded asset's checksum checked.
**Human check:** the checklist below, copied into the PR.
**Commit:** `lj-release-0.2.0: version 0.2.0, changelog and readme`

```text
Prepare v0.2.0 exactly as CLAUDE.md's "Release process" says (it was done that way for v0.1.0
and the betas): bump the three versions, turn [Unreleased] into [0.2.0] with today's date, and
point the README at the release. Do it all through this PR. If LJ18 was skipped, this PR also
flips its row to ⏭ skipped and records it in the overall status (LJ18's header).

Fill the PR description from the template. Needs human check (the owner's checklist):
- [ ] Mac (your Retina machine): install the DMG — notarized: no warning; ad-hoc (LJ18 skipped):
      the README's Gatekeeper step; the welcome on a fresh profile; start at login across a
      logout; memory at rest in Activity Monitor (no WebContent processes for Little Joan).
- [ ] YouTube (phase 18): its fullscreen button, F and the player's exit button, in a narrow and
      a wide window.
- [ ] Resizing (phase 19): LJ40's checklist items still hold on this build.
- [ ] Control (phase 20): hide the icon, quit, launch again → the window opens; Settings →
      Keyboard: a rebound key works, a Spotlight clash warns; quick commands (theme, mobile sites,
      copy link, clear history asks twice).
- [ ] Downloads and zoom (phase 8): a real PDF lands in ~/Downloads under its own name, "Show in
      folder" selects it, again → "name (1).pdf"; ⌘+ / ⌘− / ⌘0 with the level in the toolbar.
- [ ] Round 2's features: the command bar, the toolbar (drag; double-click zooms), logins across
      close / quit / reboot, history ⌘Y, themes (no flash), extensions (also from hibernation).
- [ ] Windows, if you have one: the installer (SmartScreen still warns, since it's unsigned), the
      welcome, start at login across a reboot, Ctrl+H, a download, zoom, YouTube fullscreen,
      resizing without white strips, hiding the icon.
- [ ] Linux, if you have one: the .deb on 22.04 (the fullscreen fallback), the AppImage on 24.04
      (real fullscreen), the Wayland shortcut on GNOME 48+ or KDE (or the `--toggle` hint on
      24.04), hiding the icon with the AppIndicator extension.
- [ ] Linux ARM64 (LJ18.1), Ubuntu 22.04 ARM in Parallels on your Mac: `sudo apt install
      ./LittleJoan-0.2.0-linux-arm64.deb`; summon, a page, phone mode in a narrow window; a video's
      fullscreen button does nothing and nothing crashes (fullscreen is off on 22.04's WebKitGTK).
- [ ] The must-not-disturb list.

Once every CI job is green, merge the PR into main with a merge commit (the owner allowed it for
this PR), then STOP and wait (Ground Rule 24). Tagging and releasing need the owner's word. When
they give it: let main's CI build the installers for the merge commit, download them
(`gh run download <run> --repo memoliertorerr/LittleJoan`; the AppImage is in
little-joan-linux-appimage, the arm64 builds in little-joan-linux-arm64 and
little-joan-linux-appimage-arm64), rename them, write SHA256SUMS.txt, and run `gh release create v0.2.0
--repo memoliertorerr/LittleJoan --target <that main commit> --latest` with notes taken from the
CHANGELOG. Then check the release, its tag, and one downloaded asset's checksum. Flip LJ19 to ✅
and record the release in the plan's overall status in the next PR (the first v0.3 phase's first
commit — main is never committed to directly).

Run `source scripts/env.sh && npm run verify` before the commit; commit everything as one commit:
"lj-release-0.2.0: version 0.2.0, changelog and readme". Push, open the PR into main, and watch CI
until every job is green (two honest fix attempts, then stop). This is a checkpoint: fill the PR
with the checklist above, merge it when green, tell the owner exactly what to check and where,
then stop. Don't start phase 21 until the owner says go.
```

---

## v0.2 round 2 steps — the Little Arc feel

> Planned 2026-10-17 (decisions: "Design Decisions Already Made → v0.2 round 2"; Ground Rules
> 18–20). These run BEFORE phases 7–10. Every phase branches from the latest `main`, and its PR
> goes into `main`. The agent merges the phase PR itself (a merge commit, never squash, never
> `--delete-branch`) once verify, package and smoke are green on all three OSes. That's the
> user's standing exception in CLAUDE.md. Steps whose look matters name CI screenshots: download
> them into `test-results/ci/`, look at them on all three OSes, describe what you saw in the
> notes, then delete them.

### LJ20 — Logins survive closing and quitting · strong · auto

> Phase 12. Branch `lj/p12-logins-history` from `main`.

**Verify:** `npm run verify` green, including Rust tests for a pure `cookies.rs`: a cookie with
no expiry is a session cookie; entries are de-duplicated by name + domain + path; at most 500
are kept; nothing is ever written to disk. `tests/smoke/smoke-logic.mjs` tests cover the new
persistence report, and `scripts/ci/smoke-log.mts` tests cover the new rules. CI green on all
three OSes, with this smoke sequence on the installed app:
- `?persist=set`, then `--hide` and `--show ?persist=check` → `LJ-PERSIST p=1 s=1 l=1`;
- set a new token, `--quit` within 1 s, relaunch on the same `LITTLEJOAN_DATA_DIR`, check →
  `p=1 s=0 l=1` (session cookies end with the app run, by design).
**Commit:** `lj-step-20: logins survive closing and quitting (cookie and storage checks in smoke)`

```text
The user wants logins (e.g. Instagram) to survive later sessions. They haven't seen a logout
yet; make it a tested guarantee.

What the vendored sources say (Landmines, round 2):
- macOS uses WKWebsiteDataStore.defaultDataStore (persistent).
- Windows and Linux: Tauri forces data_directory to app_local_data_dir when the app sets none,
  so state.rs's comment "None keeps the engine's default location" is wrong; correct it.
- Windows: every wry webview gets its own WebView2 environment, which ends when the window is
  destroyed, so SESSION cookies vanish on every close.
- macOS: tray Quit → app.exit(0) → process::exit without unloading the webviews, so cookies set
  just before quitting may not reach disk.

1. Checks first.
   - tests/smoke/smoke.html (+ smoke-logic.mjs and its test): `?persist=set&t=<token>` sets a
     persistent cookie `lj_p=<token>; Max-Age=31536000; Path=/; SameSite=Lax`, a session cookie
     `lj_s=<token>; Path=/`, and localStorage `lj_l=<token>`, then reports `LJ-PERSIST set`.
   - `?persist=check&t=<token>` reports `LJ-PERSIST p=<0|1> s=<0|1> l=<0|1>` in the title.
   - Add `--quit` to cli.rs: forwarded through single-instance, it takes the same path as the
     tray's Quit. Smoke uses it to quit gracefully.
   - smoke.mts and smoke-log.mts get the sequence from Verify. The smoke server keeps the same
     port across the relaunch, because localStorage is per origin (cookies ignore the port).
   - Push and READ which checks fail on which OS before fixing anything; write that down in the
     notes.
2. Session cookies across close → summon: in joan::destroy, before the page is released, read
   the page webview's cookies (tauri 2.12's Webview::cookies) and keep the session ones in
   AppState (memory only). In joan::create, before the first navigation, put them back with the
   webview's cookie setter. Check the exact API in the vendored tauri-2.12.4 source; if there
   is no way to set a cookie, STOP and record it. Keep the selection rules in a pure cookies.rs
   with tests. Never write session cookies to disk; quitting forgets them.
3. macOS quit: if the quit-right-after-set check fails, make Quit (tray.rs and --quit) go
   through the normal destroy path when a window is open, and wait for the destroy (at most
   1.5 s) before app.exit(0). If the check passes without it, don't add it, and say so.
4. If a check still fails after that on an OS, it's a stop condition: report what the engine
   did.
5. Docs: README (privacy: what Little Joan keeps and where — cookies and site data in the
   engine's store, session cookies only in memory), docs/architecture.md (a data-locations table
   per OS, from the Landmines), CHANGELOG ([Unreleased] → Fixed / Added `--quit`), this plan.

Do NOT build history (LJ21) or touch the bar UI.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ20 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-20: logins survive closing and
quitting (cookie and storage checks in smoke)". Push, open the phase-12 PR into main as a draft,
and watch CI until verify, package and smoke are green on all three OSes (Ground Rule 3; two
honest fix attempts, then stop).
```

**Implementation notes (2026-10-17):**

- **Checks first, as the prompt orders.** The step commit (`lj-step-20`) carries only the checks,
  `--quit` and the doc corrections, so CI shows what each engine does before anything is fixed;
  the fixes follow as `lj-step-20-fix` commits, and the tracker row turns ✅ with the fix.
- **What the checks found before any fix** (run 39063980861, installed builds):

  | | close → summon (want p=1 s=1 l=1) | quit → relaunch (want p=1 s=0 l=1) |
  |---|---|---|
  | Windows (WebView2) | **p=1 s=0 l=1** — the session cookie is lost | p=1 s=0 l=1 ✔ (quit 49 ms after the set) |
  | macOS (WKWebView) | p=1 s=1 l=1 ✔ | **no report**: the relaunched app opened its window, but neither webview ever loaded, and 1.1 s later it aborted — wry 0.57's `url()` unwraps WKWebView's unset URL (`wkwebview/mod.rs:1370`), hit by the address watcher |
  | Linux (WebKitGTK, scale 1 and 2) | p=1 s=1 l=1 ✔ | p=1 s=0 l=1 ✔ |

  Every smoke job also failed "--start-at-login opened a window": that rule looked at the whole
  rest of the log, and the new persistence steps open windows after it. It now stops at a count
  of events taken 1.5 s after `off` (`eventsUntil`, tested both ways).
- **Fix 1, session cookies (Windows only):** `cookies.rs` (pure): a session cookie has neither
  an expiry date nor a max-age; one per name + domain (any case) + path, the last read wins; at
  most 500; a cookie without a name or domain is dropped (WKWebView would refuse it), a missing
  path becomes `/`, and `SameSite=None` without `Secure` is set without SameSite (Chromium,
  so WebView2, refuses that pair). `joan::destroy` reads `Webview::cookies()` (off the main thread, as
  WebView2 needs) before the page is released and keeps the session ones in `AppState`;
  `joan::create` sets them with `Webview::set_cookie` right after the webviews exist, then reads
  the cookies once more as a barrier (WebView2's cookie manager answers in order), before any
  web page loads. `ENGINE_KEEPS_SESSION_COOKIES` (`!cfg!(windows)`) skips both on macOS and
  Linux: CI showed their engines keep them, and setting them again there would add copies. The
  module has no file or serialization code (a test reads its source) and its `Debug` prints only
  a count. Smoke log: `session-cookies-kept <n>`, `session-cookies-restored <n>`.
- **Fix 2, macOS quit (the prompt's step 3):** `joan::quit` on macOS closes an open window
  through `handle(Hide)` → `destroy` (the page is unloaded) on its own thread and waits at most
  1.5 s before `app.exit(0)`. Cmd+Q now goes through it too (`menu.rs` has its own Quit item,
  id `menu-quit`, instead of the standard one, which ended the process the same way). Windows
  and Linux passed without it, so they exit at once (the prompt: don't add it where it passes).
- **Fix 3, the abort:** `bar::page_url` reads the page's address only after the page webview
  reported a page load in this window (`PAGE_STARTED`, reset per window); `current_text`,
  `publish_url`, Translate and Open in Browser use it. Before that the bar shows the start page.
- **After the fixes** (run 39065170176, every job green): close → summon `p=1 s=1 l=1` and quit
  → relaunch `p=1 s=0 l=1` on Windows (`session-cookies-kept 1`, `-restored 1`), macOS (the
  window closed 441 ms after `quit`; the relaunch loaded at once) and Linux at scale 1 and 2;
  `--quit` came 51–80 ms after the set; no webview helpers left after any cycle.
- **Fix 4 (`lj-step-20-fix2`, first commit of phase 13, from `main`'s CI after the phase-12
  merge):** run 39066776951's `smoke (macos)` aborted ~340 ms after `window-created` of a
  `--show <persist url>` summon while hibernating (create with a URL), before any page load:
  `thread 'tokio-rt-worker' panicked at core/src/panicking.rs:225:5: panic in a function that
  cannot unwind … thread caused non-unwinding panic. aborting.` The same code passed on both PR
  runs. **The panicking call was not `Webview::url()`** (that runs on the main thread, through
  tauri-runtime-wry's `webview_getter!`, and would panic there with wry's own `unwrap` message).
  It was tauri 2.12.4's `tauri://` asset handler (`protocol/tauri.rs`: `async_runtime::spawn` →
  `responder.respond`), which answers WKURLSchemeTasks **from a tokio worker**: the start page
  (`index.html`) was still loading when `create` navigated the page to the URL, WebKit stopped
  its task, and wry 0.57's responder (`wkwebview/class/url_scheme_handler.rs`) raced it — it
  checks the task, then calls `didReceiveResponse` / `didReceiveData` / `didFinish` inside
  `objc2::exception::catch` for exactly that "task already stopped" exception. objc2's docs:
  that catch **cannot work when Rust is compiled with `panic = "abort"`**, our release profile,
  so the Objective-C exception hit a non-unwinding Rust frame and the process aborted. **Fix:**
  the release profile uses `panic = "unwind"` (Cargo.toml says why), so wry's catch works for
  every such race (the page's start page, the bar's own files when a window closes quickly).
  Asked for at the same time, and done: the smoke run gives the app `RUST_BACKTRACE=1` (release
  builds are stripped, so frames come without names, but the panicking thread and order show);
  `bar::page_url` no longer asks the engine — `page_address.rs` (pure, 5 tests) keeps the
  address from the page's `Started` / `Finished` events and the app's own navigations, and the
  once-a-second in-page check is the only `Webview::url()` caller, only while the last event
  was `Finished`, dropping an answer that raced with a newer navigation (a sequence token).
  `PAGE_STARTED` is gone. The size cost of unwinding is in phase 13's notes (CI's package job).
  Disk: 13 GiB free; `.cache/target` 1.2 GB.
- **Not fixed (Deferred follow-ups, Landmines):** Tauri's cookie type drops the leading dot of a
  domain cookie, so a session cookie set for `.example.com` comes back for `example.com` only.
- Disk: checks commit 13 GiB free at the start, 12 GiB after verify; fix commit 13 GiB free;
  `.cache/target` 1.2 GB.
- **The smoke page** (`tests/smoke/smoke.html`, logic in `smoke-logic.mjs`): `?persist=set&t=…`
  writes `lj_p` (`Max-Age=31536000; Path=/; SameSite=Lax`), `lj_s` (`Path=/`, no expiry) and
  localStorage `lj_l`, then titles itself `LJ-PERSIST set`; `?persist=check&t=…` titles itself
  `LJ-PERSIST p=<0|1> s=<0|1> l=<0|1>` (1 = still the token). In persist mode it skips the codec
  and viewport reports so the title stays put. Tokens are cookie-safe (`[\w-]{1,64}`).
- **`--quit`** (`cli.rs`): a third result next to the window command and `--start-at-login`;
  it wins over window flags, so a quitting launch never opens the window. Single-instance hands
  it to `joan::quit`, which the tray's Quit now uses too (logs `quit`, then `app.exit(0)`, as
  before). A `--quit` launch with nothing running just ends (a first run still saves its
  settings, without the welcome).
- **The smoke sequence** (`smoke.mts`, after start at login): set → `--hide` → `--show` check;
  new token → `--quit` as soon as the set is logged (the log is polled every 50 ms) → wait for
  the process to end by itself (15 s, else it is killed and that fails) → 2 s → relaunch on the
  same data folder (log `smoke-after-quit.log`) → check → `--quit`. The node server stays up
  across the relaunch, so the origin (and its localStorage) is the same. `persistFailures`
  (`smoke-log.mts`, unit-tested) wants `p=1 s=1 l=1` after close → summon, `p=1 s=0 l=1` after
  quit → relaunch, the app's `quit` within 1000 ms of the page's `LJ-PERSIST set` (both from
  the app's own log clock, so the check really is "quit right after setting"), one `started`
  and no `first-run` in the relaunch. `summary.json` gains `persist`.
- **`state.rs`'s comment was wrong**: with no `data_directory`, Tauri forces
  `app_local_data_dir` on Windows and Linux (`manager/webview.rs`); macOS ignores it. The
  comment and docs/architecture.md ("Where data lives") now say so.

### LJ21 — History: recorder, store, frecency · strong · auto

**Verify:** `npm run verify` green, including Rust tests for a pure `history.rs`:
- **Recording:** fragments are dropped and queries kept; only `http`/`https` URLs are recorded
  (never the app's pages, `about:` or `ljext:`); a title change updates the latest record; visit
  kinds are `typed` (`open_input` or a picked suggestion), `link`, and `reload` (the reload page
  action).
- **Frecency:** exact values for the decided formula with a fixed clock.
- **`top_sites(n, exclude)`:** per host, `www.` stripped, ties go to the latest visit; it opens
  the host's most-visited URL with at most one path segment, else the origin.
- **`search(query, limit)`:** the decided points, every token must match.
- **Store:** capped at 5,000 URLs (least frecent dropped); a bad or missing file is empty
  history, never a crash; saves are atomic.

Settings gain `rememberHistory` (default true, tolerant parse). CI smoke: after the smoke page
loads, `history.json` in `LITTLEJOAN_DATA_DIR` holds its URL, title and a visit. A run with
`rememberHistory: false` records nothing.
**Commit:** `lj-step-21: local history with frecency, top sites and matching`

```text
History feeds the command bar (LJ23) and the history view (LJ25). This step builds the data
layer only.

Design (decided; do not relitigate): see "Design Decisions Already Made → v0.2 round 2"
(Frecency, History store) and Ground Rule 20.
- Record natively, with no IPC from the page: in bar.rs's page builder, on_page_load(Finished)
  records a visit; on_document_title_changed updates the title. The visit kind comes from
  state: open_input (and, from LJ23, a picked suggestion) sets "typed" for the next load, the
  reload page action sets "reload", and everything else is "link".
- src-tauri/src/history.rs: pure types and functions (record, frecency, top_sites, search, cap,
  parse/serialize) with tests that use an injected clock. A thin store glue (history_store.rs,
  or part of state.rs) loads history.json from the data folder when the window opens, keeps it
  in AppState while the window exists, saves it atomically on destroy and every 30 s while
  dirty, and drops it on hibernate (Option → None), so nothing extra is held at rest.
- Commands (bar capability only, read-only for now): `history_top(n)` and
  `history_search(query, limit)`, returning rows of { url, title, host, score }.
- settings.rs: `rememberHistory` (default true). When it is false, nothing is recorded; the
  existing history stays until LJ25's clear.

Files: src-tauri/src/{history.rs (new), bar.rs, commands.rs, state.rs, settings.rs,
page_actions.rs (reload kind), lib.rs, build.rs}, src-tauri/capabilities/bar.json,
scripts/ci/{smoke.mts, smoke-log.mts (+ test)}, docs/architecture.md (history: where, format,
cap, what is never recorded), CHANGELOG ([Unreleased] → Added: history is kept locally), this plan.

Do NOT build any UI (LJ23, LJ25).

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ21 to ✅, add notes, and
commit: "lj-step-21: local history with frecency, top sites and matching". Push and watch CI until
every job is green (two honest attempts, then stop). This ends phase 12. Fill the PR description
from the template ("Needs human check": nothing visible yet; the regression list). Once CI is
green, merge the PR into main with a merge commit (CLAUDE.md standing exception), then stop and
report.
```

**Implementation notes (2026-10-17):**

- **`history.rs`** (pure; its tests are in `history/tests.rs` to keep the file near 400 lines):
  `recordable` (http/https only, the fragment dropped, the query kept; never the app's origin
  via `keys::is_app_url`, never Windows' `tauri.localhost` / `ipc.localhost` /
  `ljext.localhost`, never a URL over 4,096 bytes), `host_key` (lower case, `www.` stripped, a
  non-default port kept so local servers on different ports are different sites), `record`,
  `set_title`, `frecency`, `top_sites`, `search`, the cap, `to_json` / `parse`. Titles are cut
  at 300 characters. The two size limits are additions (not in the prompt) so one page can't
  bloat the file.
- **Interpretations, stated so LJ23 can revisit them:** frecency is per host for top sites (as
  decided) and the same formula over the URL's own visits for matching and the cap — with
  host frecency, every page of a site would tie and the latest video would beat the site's
  home page. Matching keeps zero-score matches (visits older than 90 days) at the end instead
  of hiding them. A top site's "URL with at most one path segment" also needs no query, so a
  search-results or video page is never a site's tile; sites with no visit in 90 days aren't
  top sites. `exclude` takes hosts; `history_top` passes none for now.
- **Recording** (`history_store.rs`, `bar.rs`): `on_page_load(Finished)` records a visit,
  `on_document_title_changed` updates that URL's record, and a title that arrived before the
  load is kept for it (`last_title`, same page but for the fragment). The kind: `open_input`
  sets `typed` for the next load, the Reload page action `reload`; the next finished load takes
  it, whatever its URL, and everything else is `link`. WebView2's built-in F5 / Ctrl+R never
  reach the page action, so on Windows those reloads count as `link`.
- **Store:** loaded in `joan::create` after the webviews exist, saved and dropped in
  `joan::destroy` before the page is released, every 30 s while dirty (a thread that checks
  once a second and ends within a second of the window closing), and in `joan::quit` (Windows
  and Linux exit without `destroy`, so unsaved visits would be lost). Saves share one lock, so
  the autosave and the close can't write the temp file at once.
- **Commands** `history_top(n)` and `history_search(query, limit)` (both capped at 50 rows),
  only in `capabilities/bar.json`; nothing calls them yet (LJ23).
- **Settings:** `rememberHistory` (default true, a wrong type falls back). `save_settings`
  keeps fields the window doesn't send, so it survives a Save; the Settings switch is LJ25/26.
- **Smoke:** after the persistence steps the main run's `history.json` is read (it must hold
  the smoke page with an `LJ-SMOKE …` title, its host and a visit, and no app page), then a run
  with `"rememberHistory": false` loads the smoke page, hibernates and quits, and must leave no
  records (`historyFailures`, unit-tested). Smoke log: `history-visit`, `history-saved`.
- Disk: 13 GiB free at the start and after verify; `.cache/target` 1.2 GB.

### LJ22 — Trusted screens move into the bar; the page webview has no IPC; overlay mode · strong · auto

> Phase 13. Branch `lj/p13-command-bar` from `main`.

**Verify:** `npm run verify` green, including:
- a Rust test that reads every file in `src-tauri/capabilities/` and fails if any capability
  names `joan-page`, or uses a window-level scope covering the page window;
- `layout.rs` tests for the new mode enum (`Toolbar`, `Overlay`, `Fullscreen`) at scale 1, 1.5
  and 2 (overlay: the bar is the whole window and the page is hidden);
- vitest tests for the bar's view state (toolbar ↔ overlay, plus the Esc rules: no page →
  hibernate; page → close the overlay and focus the page).

`capabilities/joan.json` is gone. `start_info` and `set_start_at_login` are granted to the bar.
`ui/index.html` makes no IPC call. CI green on all three:
- the smoke log shows `overlay true` right after `window-created`, and `overlay false` once the
  smoke page is opened;
- the first-run check still sees `welcome`, now from the bar;
- screenshots: `start.png` (the overlay fills the window) and `page.png` (40 px bar above the
  page), including the Linux scale-2 job.
**Commit:** `lj-step-22: trusted screens live in the bar webview; the page webview has no IPC; overlay mode`

```text
Tauri treats custom-scheme pages as LOCAL, so anything local in joan-page gets joan-page's
local capabilities (Landmines). Extensions (LJ29) will run in joan-page, so joan-page must have
NO capabilities. Every trusted screen moves into joan-bar, which becomes an overlay covering the
window while the command bar (and, from LJ25, history) is open. Ground Rule 18.

1. layout.rs: a mode enum. Toolbar = today's layout. Overlay = the bar takes the whole window and
   the page is hidden. Fullscreen = today's element-fullscreen case (the bar hidden). Element
   fullscreen always wins: starting it closes the overlay. Tests at scale 1, 1.5 and 2.
2. bar.rs: `set_overlay(app, on)`. Windows/macOS: the bar's bounds become the whole window and
   page.hide(); turning it off restores the toolbar layout and page.show(). Linux (GtkBox,
   platform/linux.rs): change the bar's height request to the window's height and hide the
   page widget, then undo both. Log `overlay true|false` to the smoke log. Open the overlay on
   every summon, on Ctrl/Cmd+L (page keys and the macOS menu, with the address selected) and
   from focus_bar. Close it after open_input navigates. Add a command `set_overlay(on)` for
   the bar (allow-set-overlay) so the bar's own Esc can close it.
3. Move the phase-6 welcome out of the start page: its markup, logic and tests go from
   ui/start.ts / start-logic.ts into the bar (a new ui/command/ module; keep files small). It
   shows in the overlay below the field on the first run. capabilities/bar.json gains
   allow-start-info and allow-set-start-at-login; delete capabilities/joan.json; ui/index.html
   becomes a plain page with no script that calls IPC (a neutral background).
4. The overlay's content for now: the field at the top (as today), the welcome on the first run,
   and an empty area that LJ23 fills. Esc follows the rules in Verify.
5. Add the capability test from Verify (it parses the JSON files; use serde_json, which is
   already a dependency).
6. Smoke and docs: smoke.mts / smoke-log.mts rules for `overlay`; README (summon opens the
   command bar), docs/architecture.md (the two trust levels; capabilities table), this plan.

Do NOT build suggestions (LJ23) or restyle the toolbar (LJ24).

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ22 to ✅, add notes, and
commit: "lj-step-22: trusted screens live in the bar webview; the page webview has no IPC; overlay
mode". Push, open the phase-13 PR into main as a draft, and watch CI until every job is green
(two honest attempts, then stop). Look at start.png and page.png on every OS.
```

**Implementation notes (2026-10-17):**

- **Phase 13 opened with `lj-step-20-fix2`** (notes under LJ20): `main`'s CI after the phase-12
  merge had aborted on macOS; the fix is the branch's first commit, before this step.
- **Modes** (`layout::Mode`: `Toolbar`, `Overlay`, `Fullscreen`; `Layout` has an optional page
  as well as an optional bar). The view state moved out of `bar.rs` into `bar/view.rs`
  (`bar::` keeps the same functions), so both stay under ~400 lines.
- **A new window is built in Overlay mode,** before it is shown, so the toolbar never flashes
  first. `overlay true` is therefore logged right after `window-created` as the state the window
  was built in (`bar::log_overlay`), not by a change; every later change logs itself.
- **Showing a page closes the overlay** in `bar::navigate`, so Enter (`open_input`), `--show
  <url>` (the smoke's path), `Create(url)` and Translate all close it. Opening it: every summon,
  Ctrl/Cmd+L (`focus_bar`, page keys and the macOS menu) and — not in the prompt, but the
  editing path the toolbar needs until LJ24 replaces the field — a click on the toolbar's
  address (`set_overlay(true)` from the bar). Element fullscreen closes it and keeps it from
  opening.
- **Linux, a deviation with the same result:** instead of a height request equal to the
  window's height (a GTK size request is also the window's minimum, so the window couldn't
  shrink while the command bar is open, and it would need updating on every resize), the bar is
  re-packed with `expand = fill = true` and keeps its 40 px request as a minimum. The page
  widget is hidden with `no_show_all` set, because tao shows a window with `show_all()` (tao
  0.37.1 `linux/window.rs`), which would bring a hidden page back on the first show.
- **The bar learns its view** from `window.littleJoan.setOverlay` on every change, and again
  when its own page finishes loading (a change during its load can't be missed); its script
  starts in the overlay view. Rules in `ui/command/view-state.ts` (tested): Esc reverts an edit,
  then with no page closes the window, with a page closes the overlay (`set_overlay(false)`,
  which focuses the page); in the toolbar Esc still just returns to the page. A document-level
  Esc listener gives the same rules after a click elsewhere in the bar — that resolves the
  LJ12 follow-up "Esc after a click in the welcome".
- **Files:** the welcome's logic moved with its tests (`git mv ui/start-logic.ts
  ui/command/welcome.ts`), its DOM glue into `ui/command/panel.ts`; `ui/start.ts` and
  `capabilities/joan.json` are gone; `ui/index.html` is a plain page (a `blank` body class, the
  shared stylesheet, no script); `bar-logic.ts` keeps only the field helpers (`barKey` became
  `view-state.ts`'s `onKey`). The bar capability gained `allow-start-info`,
  `allow-set-start-at-login` and the new `allow-set-overlay`.
- **`capability_tests.rs`** (5 tests): reads every file tauri-build would read
  (`capabilities/**/*`; only JSON may be there, since TOML/JSON5 couldn't be checked) and inline
  capabilities in `tauri.conf.json`, accepting the single / list / `{ "capabilities": [...] }`
  forms; fails if a `webviews` pattern matches `joan-page` or a `windows` pattern matches `joan`
  (glob `*` / `?`; character classes are refused outright), if any capability has `remote`, or if
  any source calls `add_capability`; and checks the bar and settings keep theirs. Checked by
  hand that it fails with the old `joan.json` and with a `"windows": ["jo*"]` file.
- **Smoke:** `overlayFailures` (unit-tested): after every `window-created` the first `overlay`
  event, before the next window, is `true`; with the smoke page, an `overlay false` comes after
  the first window and before `LJ-SMOKE ready`. The first-run check applies the first rule too.
  `smoke.mts` waits for both before `start.png` / `page.png`. A start-at-login test had a fixed
  event count (`eventsUntil: 16`) that the longer fixture broke; it now counts the log.
- `npm run verify`: 93 TS tests, 135 Rust tests, clippy clean. Disk: 13 GiB free at the start
  and after verify; `.cache/target` 1.2 GB.
- **CI (run 39069197262): verify ×3, package ×3, smoke macOS and Windows green; both Linux smoke
  jobs red.** Screenshots: macOS and Windows `start.png` — the command bar fills the window (the
  field at the top with its focus ring, the hint under it, no page visible); `page.png` — the 40
  px bar with the smoke page's address above the page; macOS `welcome.png` — the welcome under
  the field. Linux at scale 2, `start.png`: the overlay fills the window as well. **But on Linux
  the page came back 1 px tall after the overlay closed** (`viewport=480x1`; `page.png` shows the
  bar's white page over the whole window) and at scale 2 half the window (`480x300`, `390x280`
  instead of 600 / 560), so the fullscreen click missed and the viewport checks failed. Cause: a
  WebKitGTK view's natural height is its page's contents size, never smaller than the view; the
  overlay made the bar as tall as the window, and once it was packed with `expand = false` again
  the GtkBox still gave it that natural height first (naturals before `expand`; at scale 2 both
  views' naturals were large, so they split the window). **Fix (`lj-step-22-fix`):**
  `platform/linux.rs` moves the bar into a frame — a GtkScrolledWindow with external scrolling
  (it doesn't pass its child's natural size on) around a GtkViewport with the minimum scroll
  policy (it sizes the child to itself) — and packs the frame instead; the frame follows the
  bar's show / hide (element fullscreen). The prompt's "height request = the window's height"
  would have hit the same natural-size loop (and pinned the window's minimum size).
- **After the fix (run 39070607023, with LJ23 on top): every job green** — verify ×3, package ×3,
  smoke on Windows, macOS, Linux and Linux at scale 2. The Linux viewports are right again
  (`480x600` at both scales, `390x560` / `720x560` after `--size`), and `page.png` at scale 1
  and 2 shows the 40 px bar above the smoke page.

### LJ23 — The command bar · strong · auto

**Verify:** `npm run verify` green, including vitest tests for a pure `ui/command/rows.ts`:
- **Rows** from (query, history rows, quick-answer hook, extension commands, engine), in the
  decided order;
- **Autofill** only on a host prefix, and only at or above the average score;
- **At most** 8 suggestion rows, plus "Settings & extensions", which is always last;
- **Keys:** ↑/↓ move without wrapping, Enter opens, Tab accepts the autofill, and
  Shift+Delete removes a history row;
- **Starters:** the 4 generic ones when history is empty;
- **Text only:** a title containing `<img src=x onerror=…>` renders as text.

Rust: `suggest(query)` returns the rows; `history_remove(url)` exists. The CSP adds `https:` to
`img-src` only. CI green on all three:
- after the smoke page is visited and the window re-summoned, the smoke log shows
  `suggestions <n> first=127.0.0.1`;
- screenshot `command-bar.png` (the empty state, with the smoke host as a top site);
- screenshot `command-typing.png` (after typing `127`; xdotool on Linux, SendKeys on Windows,
  best effort on macOS).
**Commit:** `lj-step-23: the command bar — top sites, history matches, search, keyboard`

```text
Build the command bar in the overlay (decisions: "v0.2 round 2", the command-bar and Frecency
bullets). The look follows the user's screenshot of Little Arc's command bar:
- a rounded panel with a search icon, the placeholder "Search or enter URL…" and a close (×)
  button on the right;
- one row per suggestion: a favicon, then "Title — domain" with the domain muted, and the
  selected row filled with the accent color;
- the last row "Settings & extensions" with a gear icon (where Little Arc shows "Contact the
  Team").
Keyboard first; the mouse works too.

1. Rust: `suggest(query)` (bar capability). It returns { kind: topSite|history|search|starter,
   title, url, host } rows from history.rs: top sites when the query is empty (excluding the
   current page), matches otherwise, and the 4 starters when history is empty. Also
   `history_remove(url)` for Shift+Delete, and `open_settings(tab?)` (opens the settings window,
   settings_window.rs; tab is used from LJ26). A picked history row records the next visit as
   "typed" (LJ21).
2. ui/command/rows.ts (pure, tested): build the row list as in Verify. A quick-answer hook
   (empty until LJ31) and a commands hook (empty until LJ30) exist from now on.
3. ui/command/view.ts: render the rows with textContent only, never innerHTML for anything that
   came from history or an extension. Favicons are `<img src="https://<host>/favicon.ico">`
   with a fallback: the host's first letter in a tinted circle. In tauri.conf.json, add `https:`
   to img-src and nothing else.
4. Typing debounces suggest() by 60 ms. Enter on the field with no selection keeps today's
   behavior (open_input: address or search).
5. Smoke: re-summon after the smoke page and log `suggestions <count> first=<host>` (from the
   bar). Take both screenshots. Add the smoke-log rules with tests.
6. Docs: README (the command bar), docs/architecture.md (commands, CSP), CHANGELOG.

Do NOT add quick answers (LJ31), extension commands (LJ30) or the history view (LJ25).

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ23 to ✅, add notes, and
commit: "lj-step-23: the command bar — top sites, history matches, search, keyboard". Push and
watch CI until every job is green (two honest attempts, then stop). Look at both screenshots on
every OS.
```

**Implementation notes (2026-10-17):**

- **Rust:** `suggest.rs` (pure, 7 tests) builds what `suggest(query)` returns: nothing typed →
  up to 8 top sites leaving out the open page's site (`exclude`, only for an http(s) page), or
  the 4 starters (Wikipedia, YouTube, GitHub, Google Maps) when there are no top sites — so also
  when the only site is the open one; typing → up to 8 history matches and one `search` row
  (title = the engine's name, url = `SearchEngine::search_url`, host = its site, for the icon).
  Rows carry the score too (autofill compares it). `History::remove` (+ test) and
  `history_store::remove` back `history_remove(url)`: the record goes from memory and the file
  is rewritten with the next save (close, the 30 s autosave, quit) — LJ25 makes removal write at
  once. `open_settings(tab?)` opens Settings (the tab waits for LJ26). All three are in the
  bar's capability only.
- **History loads before the webviews** in `joan::create` now (it closes again if building the
  webviews fails): the command bar asks for suggestions as soon as it loads, and a request that
  beat `history_store::open` would have shown the starters instead of the user's sites.
- **`ui/command/rows.ts`** (pure, 23 tests) as the Verify lists; choices it makes: a search row
  is built even without Rust's (then Enter goes through `open_input`); a title equal to its site,
  or empty, shows the site once; the autofill completes the best match's *address* as typed
  (`typedForm`: no scheme, no `www.`, no lone `/`) — the condition stays a host prefix — so Enter
  opens exactly what the field shows; ↑ from the first row goes back to the field (no wrap);
  `queryFor` treats the open page's untouched address (Ctrl/Cmd+L) as "nothing typed". Hooks for
  quick answers (LJ31) and commands (LJ30) exist and are empty.
- **`ui/command/view.ts`** draws with `textContent` only, against a small `ElementLike` slice of
  the DOM, so `view.test.ts` draws into a stand-in document whose `innerHTML` setter throws (a
  title `<img src=x onerror=…>` comes out as the title's text, and the only image is the site's
  favicon). jsdom would have been a new dependency. The favicon (`referrerpolicy=no-referrer`)
  shows once loaded, over the site's first letter in a tinted circle (stable hue per site), and
  is removed if it fails.
- **`ui/command/command.ts`** (glue): 60 ms debounce, only the latest answer is drawn, the
  inline autofill only right after an insertion at the end of the field, hover selects and click
  opens (mousedown is cancelled so the keyboard stays in the field). In the command bar the field
  no longer reverts on blur (a click on a row or the welcome mustn't lose the text); the toolbar
  still reverts. The × button follows Esc's rules (`onCloseButton`).
- **The hint line is gone** (`hintText` and its test went with it): the rows fill the panel, as
  in Little Arc; the welcome still says how to close.
- **CSP:** `img-src 'self' data: https:` — nothing else changed. The command bar's colors are new
  tokens in `style.css` (backdrop, panel, hairline, shadow, selection), light and dark; LJ27's
  theme engine replaces them.
- **Smoke:** `smoke-command.mts` (new, 6 tests; `smoke-log.mts` was already past 400 lines): Rust
  logs `suggestions <rows> first=<site> typed=<characters>` (never the text). The main run must
  show the starters first before any page, then — after the smoke page, in a new window —
  `first=127.0.0.1…` with nothing typed, and after typing `127` the smoke page first (required on
  Linux and Windows, best effort on macOS). The extra summon comes after the helper cycles: wait
  for the top site, `command-bar.png`, type `127` (xdotool; `WScript.Shell.AppActivate` +
  `SendKeys`; a Swift script posting key events after activating the app), `command-typing.png`,
  `--hide`. `summary.json` gains `commandBar`.
- `npm run verify`: 126 TS tests, 143 Rust tests, clippy clean. Disk: 13 GiB free;
  `.cache/target` 1.2 GB.
- **CI (run 39070607023, together with `lj-step-22-fix`): every job green.** The smoke log has
  `suggestions 4 first=wikipedia.org typed=0` for the first window on all four jobs, then
  `first=127.0.0.1:<port>` in every later window and `typed=3` after typing — on macOS too, so
  its best-effort key events got through. Screenshots, all four alike in layout:
  `start.png` — the command bar over the whole window: a rounded white panel on a light-grey
  backdrop, a search icon, "Search or enter URL…" with the caret, × on the right, then
  Wikipedia, YouTube, GitHub and Google Maps with their real favicons (loaded from the sites) as
  "Title — domain" with the domain in grey, a hairline, and "Settings & extensions" with the
  gear; `command-bar.png` — the smoke page as the only top site ("LJ-SMOKE viewport=720x560 …
  — 127.0.0.1:<port>") with its stand-in icon (a "1" in a tinted circle: the favicon can't load
  over https from the http-only smoke server) and Settings under it; `command-typing.png` —
  "127" typed and ".0.0.1:<port>/smoke.html" completed and selected in the field, the smoke page
  row filled with the accent blue (white text), "Search Google for “127”" with Google's icon,
  then Settings. At scale 2 everything is crisp and twice the pixels. Text stays text: the long
  titles are plain, no markup. **One finding, Linux scale 1:** the completed field had *no*
  selected row: xdotool's second and third keys came 61 ms apart, the answer for "12" completed
  the field after "7" was typed, and the answer for "127" then found the field no longer equal to
  the typed text, dropped its autofill (no selection, Tab dead) and left the old completion on
  screen. **Fix (`lj-step-23-fix`):** `rows.ts`'s `showsTyped` (pure, tested) accepts the typed
  text followed by our own still-selected completion as "untouched", so the newer answer
  completes the field again and selects its row; when an answer has no autofill, a completion an
  earlier one left is taken away.

### LJ24 — The Little Arc-style toolbar · strong · auto

**Verify:** `npm run verify` green, including:
- vitest tests for the toolbar's pure logic (the domain shown: `www.` stripped, IPs as is, long
  hosts shortened in the middle; the back button enabled only after the page navigated within
  this window);
- Rust tests for the navigation counter that drives it.

CI green on all three; `page.png` shows the favicon, `127.0.0.1`, a back button and an
"open in default browser" button. On macOS, `TitleBarStyle::Overlay` puts the traffic lights
inside the toolbar with no overlap, and the screenshot shows it. The bar's capability gains
`core:window:allow-start-dragging` (the empty toolbar area drags the window).
**Commit:** `lj-step-24: Little Arc-style toolbar; macOS traffic lights inside it`

```text
Restyle the 40 px bar (Toolbar mode) after Little Arc: a slim, theme-tinted strip.
- Left: a back button (enabled once the page has somewhere to go back to, tracked natively by
  counting this window's page loads and backs).
- Middle: the favicon and the domain; a click opens the command bar with the full address
  selected.
- Right: "open in default browser" (the existing page action, then hibernate, as today).

This replaces LJ10.1's "no buttons in the bar" rule (decided by the user's Little Arc request).

- macOS: the page window uses TitleBarStyle::Overlay with a hidden title, so the traffic lights
  sit inside the toolbar. Leave about 78 px of padding on the left, and check the overlay
  command bar too. The bar webview starts at y = 0 under the transparent title bar. The LJ10.1
  landmine about content under the title bar is the point here; re-check it in the
  screenshot. Empty toolbar areas get `data-tauri-drag-region`, and the bar's capability gains
  core:window:allow-start-dragging.
- Windows and Linux keep the native title bar (custom chrome is Later).
- Keep keyboard paths: Ctrl/Cmd+L, Alt+Left / Cmd+[, and the tray items all still work.

Files: ui/bar.html, ui/bar.ts, a small ui/toolbar/ module (+ tests), ui/style.css,
src-tauri/src/{joan.rs, bar.rs, commands.rs if a command is needed}, src-tauri/capabilities/bar.json,
README, docs/architecture.md, CHANGELOG, this plan.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ24 to ✅, add notes, and
commit: "lj-step-24: Little Arc-style toolbar; macOS traffic lights inside it". Push and watch CI
until every job is green (two honest attempts, then stop). Look at page.png and command-bar.png
on every OS. This ends phase 13. Fill the PR description:
- Needs human check (at LJ35): dragging the window by the toolbar on macOS.
- The regression list.
Once CI is green, merge into main (merge commit), then stop and report.
```

**Implementation notes (2026-10-17):**

- **The navigation counter** (`back_stack.rs`, pure, 6 tests): Tauri 2.12 can't ask a webview
  whether it can go back, and the page has no IPC, so the bar's back button follows a trail of
  this window's finished web-page loads (fragment dropped): the page before the last one loading
  again is a step back (whoever caused it — the button, Alt+Left, Cmd+[, the page itself), the
  last one again is a reload, anything else is pushed (at most 100). The app's pages and
  `about:` never count, so the first web page never offers "back" to the start page. Rust tells
  the bar `setCanGoBack(<bool>)` on a change and when the bar loads.
- **Commands** `go_back` and `open_in_browser` (the existing page actions Back and
  OpenInBrowser) and `core:window:allow-start-dragging` join the bar's capability.
  `set_overlay(true)` now goes through `focus_bar`, so a click on the domain opens the command
  bar with the full address selected, exactly like Ctrl/Cmd+L.
- **The toolbar** (`ui/toolbar/logic.ts`, 9 tests; `toolbar.ts`): back, a drag gap, the site
  (the command bar's lettered stand-in under the favicon, then the domain), a drag gap, open in
  browser (an original "box with an arrow" glyph). The domain drops `www.`, shows IPs without the
  port (`127.0.0.1`), and shortens names past 32 characters in the middle. The icon is only
  rebuilt for a new site, so the favicon isn't fetched for every page. The field is no longer
  visible in the toolbar view, so LJ22's "click the address" path (`onFieldPress`, and its test)
  went away; the domain button replaced it.
- **macOS title bar:** `TitleBarStyle::Overlay` + hidden title. Tauri's `window::WindowBuilder`
  (the multi-webview one) has no `traffic_light_position`, only `WebviewWindowBuilder` does —
  but `WindowBuilder::from_config` reads `WindowConfig.traffic_light_position` (tauri-runtime-wry
  `lib.rs`), so the macOS builder starts from a `WindowConfig` (label, style, hidden title,
  lights at x 14, y 20: tao's inset, i.e. the close button's left edge and the height of the
  title-bar area under the buttons, chosen to centre them in the 40 px toolbar — to be checked
  in the screenshot). The toolbar leaves 78 px for them; the command bar puts a 40 px draggable
  strip above its panel on macOS. The bar finds out it's on macOS from its user agent (`isMac`).
- **Keyboard focus follows the page out of the command bar** (`bar::navigate`): CI's LJ22
  `page.png` on macOS showed the field still focused after `--show <url>`; with no field in the
  toolbar any more the page now gets the keyboard whenever opening a page closes the overlay.
- Not granted (Deferred follow-ups): double-clicking the toolbar to zoom the window on macOS
  (Tauri's drag script calls `internal_toggle_maximize`, which needs its own permission).
- `npm run verify`: 134 TS tests, 149 Rust tests, clippy clean. Disk: 13 GiB free;
  `.cache/target` 1.2 GB.
- **CI (run 39071547922): every job green — but `page.png` was wrong on every OS:** the 40 px
  strip above the page was blank (no back, domain or open-in-browser). The toolbar's `<nav>` had
  the class `toolbar`, which is also the body's view class, so the base rule hiding the strip
  (`.toolbar { display: none }`) hid the whole body in the toolbar view. Smoke checks can't see
  pixels; the screenshot did. The rest was right: macOS `start.png` / `command-bar.png` /
  `command-typing.png` show the traffic lights in a 40 px backdrop strip above the command bar's
  panel, vertically centred (x 14, y 20 works), no overlap with the search icon or the field, the
  title bar transparent over the bar's backdrop; in `page.png` the lights sit in the toolbar's
  strip and the page starts right below it (`viewport=480x600`). Windows and Linux are unchanged
  in the command bar. **Fix (`lj-step-24-fix`):** the strip is `.toolstrip`; a new
  `ui/bar-markup.test.ts` fails if a view class (`overlay`, `toolbar`) is put on any element but
  `<body>` or styled without `.bar-body` (checked: it fails on the old stylesheet).
- **After the fixes (run 39072675033, at the plan-status commit): every job green** — verify ×3,
  package ×3, smoke on Windows, macOS, Linux and Linux at scale 2. `page.png`: Windows and Linux
  — under the native title bar, the toolbar's light strip with a greyed back chevron (one smoke
  page per window, so nowhere to go back), the site's "1" stand-in and `127.0.0.1` centred, the
  open-in-browser glyph on the right; Linux at scale 2 the same, crisp; macOS — the traffic
  lights inside the toolbar's left end, the back chevron just after them with no overlap, the
  site centred, the glyph on the right, the page right below (`480x600`). The Linux scale-1
  `command-typing.png` now has the completed field *and* the smoke page's row selected
  (`lj-step-23-fix`). Login persistence (`p=1 s=1 l=1` / `p=1 s=0 l=1`), the viewports and
  "no webview helpers after hibernating" (0 / 0 / 0 per cycle) held on all four.
- Final `npm run verify`: 138 TS tests, 149 Rust tests, clippy clean. Disk at the end: 13 GiB
  free; `.cache/target` 1.2 GB.

### LJ25 — History view · standard · auto

> Phase 14. Branch `lj/p14-history-settings` from `main`.

**Verify:** `npm run verify` green, including:
- vitest tests for `ui/history/logic.ts`: grouping by local day (Today, Yesterday, then dates),
  a search filter, and the "Clear" range choices;
- Rust tests: `history_remove(url)` and `history_clear(range: hour|day|week|all)` remove
  entries from memory AND rewrite `history.json`.

Ctrl/Cmd+Y (page keys and the macOS menu) and a "History" command row open the overlay in its
history view. CI green; screenshot `history.png` (opened with Ctrl+Y: xdotool on Linux, SendKeys
on Windows, best effort on macOS) shows the smoke page under Today.
**Commit:** `lj-step-25: history view (Ctrl/Cmd+Y): by day, search, delete, clear`

```text
A history view in the overlay (trusted, in joan-bar).
- A list grouped by day: each row shows the favicon, title, domain and time; a click opens the
  page.
- A search field.
- Each row has a delete (×) button.
- "Clear history…" offers last hour / today / this week / everything.
- Esc returns to the command bar.

Rust: history_clear(range) (bar capability, and the settings window's for LJ26's Privacy tab).
Keys: Ctrl+Y on Windows/Linux and Cmd+Y on macOS (keys.rs, the macOS menu in menu.rs,
platform/win.rs and linux.rs through keys.rs). The command bar gets a "History" row when the
query matches "history".

Files: ui/history/ (new, + tests), ui/bar.ts, src-tauri/src/{history.rs, commands.rs, keys.rs,
menu.rs, page_actions.rs}, build.rs, capabilities/{bar,settings}.json, scripts/ci/smoke.mts (+ log
rules), README (page keys table), docs/architecture.md, CHANGELOG, this plan.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ25 to ✅, add notes, and
commit: "lj-step-25: history view (Ctrl/Cmd+Y): by day, search, delete, clear". Push, open the
phase-14 PR into main as a draft, and watch CI until every job is green (two honest attempts,
then stop). Look at history.png.
```

**Implementation notes (2026-10-18):**

- **`history_top` / `history_search` are removed** (the Deferred follow-up from LJ23): the
  history view needs every page with its latest visit time and filters as you type in
  TypeScript (the Verify puts the search filter in `ui/history/logic.ts`), which neither command
  gave. In their place `history_list()` returns every record, latest visit first, as `{ url,
  title, host, lastVisit, visitCount }` (bar only; empty with no window), and logs
  `history-list <n>`. Their permission files went with them; `History::search` / `top_sites`
  stay (suggest.rs uses them).
- **`history_clear(range)`** (`hour` \| `day` \| `week` \| `all`, the bar and — as this prompt
  says — already the settings window's capability for LJ26): the pure part is
  `history/manage.rs` (a child module, so `history.rs` stays near its size; 6 tests): the visits
  at or after the range's start go (a visit stamped in the future too), a page left without
  visits goes, one with older visits keeps them with its count lowered. **The ranges are rolling
  windows** (the last hour, 24 hours, 7 days): Rust has no local calendar without a new
  dependency, so the view's choices say "Last hour", "Last 24 hours", "Last 7 days",
  "Everything" rather than the prompt's "today / this week", which would have promised a local
  day the clear doesn't use.
- **Removing and clearing rewrite `history.json` at once** (`history_store::edit_in`, 3 tests
  with a temp folder: with a window open, memory and file both change; with none, the file is
  read, changed and written without loading anything into `AppState`; "everything" replaces
  even an unreadable file, a smaller range leaves it alone). `history_remove` now returns
  `Result<bool, String>`. One `FILE` lock (it replaced `SAVING`) now covers every
  read-modify-write of the file and every change of the window's history — open, close, the
  autosave's snapshot + write, remove, clear — taken before the app state's lock, so an
  autosave's older snapshot can't land after a clear.
- **The view** (`ui/history/`: `logic.ts` 19 tests, `view.ts` 3 tests with a stand-in document
  that refuses `innerHTML`, `history.ts` glue, `history.css`): **one row per page at its latest
  visit** (the store keeps one record per URL, and × removes the record), grouped by the local
  day of that visit — Today, Yesterday, then "Monday, October 12" (with the year when it isn't
  this one; `Intl`, so the system's language). Rows reuse the command bar's look (site icon,
  "Title — site", plus the time and a ×). Choices this step made: Enter with no row chosen opens
  the first one; Shift+Delete removes (Delete alone edits the search field); Esc first closes
  the clear choices; 200 rows are drawn, 200 more when the list is scrolled to its end; the
  clear choices start on the smallest range and need a second click (Clear).
- **How it opens:** `PageAction::History` — Ctrl+Y in `keys.rs` (Windows and Linux, through the
  existing WebView2 / GTK hooks, so also while the bar has the keyboard), Cmd+Y as a View → History
  item in `menu.rs` — runs `bar::show_history`: open the overlay (which tells the bar first),
  focus the bar, `window.littleJoan.showHistory()`; not during element fullscreen. The command
  bar's built-in **History** row (`rows.ts` `builtInCommands`, a clock icon) shows the view
  without Rust. It appears when the typed text starts "history" (two letters or more) or
  contains the word; **built-in commands always find room among the 8 rows** (history matches
  make way), extension commands still take what is left.
- **The bar** keeps the panel (command / history) itself: the body gets a third view class,
  `history` (`bar-markup.test.ts` now checks it too, in every stylesheet `bar.html` links);
  opening or closing the overlay starts again at the command bar.
- **Not in the prompt's file list but needed:** `history_store.rs` (the store moved there in
  LJ21), `bar.rs` (`show_history`), `lib.rs`, `ui/command/{rows,view,command}.ts` (the History
  row), `ui/bar.html`, `ui/bar-markup.test.ts`, `scripts/ci/smoke-command.mts` (+ test).
- **Smoke:** after `command-typing.png`, Ctrl+Y (xdotool; SendKeys `^y`; a Swift key event with
  the Command flag on macOS, best effort like typing) must log `history-view` and then a
  `history-list` of at least one page (`historyViewFailures`, 3 tests); then `history.png`.
- `npm run verify`: 168 TS tests, 159 Rust tests, clippy clean. Disk: 13 GiB free at the start
  and after verify; `.cache/target` 1.2 GB.
- **CI (run 39076586164): every job green** — verify ×3, package ×3, smoke on Windows, macOS,
  Linux and Linux at scale 2. Ctrl+Y reached the app on Windows and Linux, and Cmd+Y on macOS
  too (`history-view`, then `history-list 1` on all four). `history.png`: the overlay's rounded
  white panel with a back chevron, "Search history…" (the caret in it on Windows and macOS) and
  "Clear history…" on the right, a hairline, the muted "TODAY" heading, and the smoke page's row —
  its lettered stand-in icon, "LJ-SMOKE viewport=720x560 ua=desktop dpr=1 — 127.0.0.1:<port>"
  with the site muted, "10:57 PM" and a muted × at the right end; nothing cut off, the same
  look as the command bar. macOS: the traffic lights in the 40 px strip above the panel. Linux
  at scale 2: the same, crisp (`dpr=2`). The rows kept the command bar's 12 px spacing, which
  is airy for a list; LJ26 makes the history list's rows tight.

### LJ26 — Settings window with tabs · standard · auto

**Verify:** `npm run verify` green, including vitest tests for `ui/settings-logic.ts` (tab
state; change detection across tabs; the Privacy tab's clear action). The settings window has
these tabs:
- **General:** shortcut, search engine, translate language, mobile sites, start at login.
- **Appearance:** a placeholder until LJ27.
- **Extensions:** a placeholder until LJ30.
- **Privacy:** remember history, clear history, and what is stored where.
- **About:** version, license, repository link, and the "inspired by Little Arc" line as plain
  text.

`little-joan --settings [tab]` opens it on a tab, and "Settings & extensions" in the command bar
opens it on Extensions. CI green; screenshots `settings-general.png` and `settings-privacy.png`
via `--settings`.
**Commit:** `lj-step-26: settings window with tabs; "Settings & extensions" opens it`

```text
Turn the settings window into the "advanced settings" the user asked for, where Little Arc has
"Contact the Team".
- Tabs: a sidebar on the left, the content on the right, in a window about 600×480.
- Save/Cancel stay at the bottom and cover all tabs.
- The window's capability gains history_clear (from LJ25).
- `--settings [general|appearance|extensions|privacy|about]` in cli.rs is forwarded through
  single-instance and opens or focuses the window on that tab (settings_window.rs).
- open_settings(tab) from LJ23 uses the same path.

Files: ui/settings.{html,ts}, ui/settings-logic.ts (+ test), ui/style.css,
src-tauri/src/{settings_window.rs, cli.rs, commands.rs, lib.rs}, capabilities/settings.json,
scripts/ci/smoke.mts (screenshots), README, docs/architecture.md, CHANGELOG, this plan.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ26 to ✅, add notes, and
commit: "lj-step-26: settings window with tabs; \"Settings & extensions\" opens it". Push and watch
CI until every job is green (two honest attempts, then stop). Look at both screenshots. This
ends phase 14: fill the PR description, merge into main once green (merge commit), then stop and
report.
```

**Implementation notes (2026-10-18):**

- **The window** (`settings_window.rs`): 600×480, not resizable, a sidebar of tabs on the left
  and the tab on the right, Save / Cancel along the bottom across both. `SettingsTab` (General,
  Appearance, Extensions, Privacy, About; by name in any case, 2 tests) is the only tab type in
  Rust. A new window gets its tab from an initialization script (`window.__littleJoanSettingsTab
  = "<tab>"`, which runs before the page's script; names are ours, lower-case letters only — a
  test checks); an open one is focused and switched with `window.littleJoanSettings.showTab`.
  Every opener goes through `settings_window::open(app, tab)`: `--settings [tab]`
  (single-instance and a first launch), `open_settings(tab)` (the command bar's row asks for
  `extensions`; the tab now travels in the row's action, `SETTINGS_ROW.action.tab`, so it is
  tested), the tray and Cmd+, (no tab: General when new, an open one stays where it is). Smoke
  log: `settings-window <tab | focus>`, and `settings-loaded` from `get_settings`.
- **`--settings [tab]`** (`cli.rs`, 1 new test): General without a tab or for a name it doesn't
  know (the window still opens); it never opens the page window (`forwarded_command` → none),
  `--quit` wins over it, a first launch with it shows no welcome (`state::finish_first_run`).
- **Hardening, not in the prompt:** the settings window only navigates within the app's origin,
  and links open in the default browser (web URLs only) — About's repository link would
  otherwise have loaded GitHub inside the trusted window.
- **Privacy:** **Remember history** joins `SettingsInput` (`save_settings` sets it; Save covers
  it with the rest); **Clear history** is a range list (the history view's choices), "Clear…", a
  question ("Remove the history of the last 7 days?") with Cancel / Remove, then `history_clear`
  at once — it isn't a setting, so Cancel doesn't undo it — and what went (the history view's
  wording). Its states are `clearStep` in `settings-logic.ts` (pick / ask / cancel / confirm /
  done / failed; nothing changes while Rust works). **What is stored where** comes from
  `get_settings`'s new `storage` (`folder`: the data folder of settings.json and history.json;
  `siteData`: the webview data folder — `LITTLEJOAN_DATA_DIR/webview` or `app_local_data_dir` —
  or on macOS a description of WebKit's own store), plus "session cookies in memory only" and
  "history never leaves this computer". About gets `version` (the package's).
- **Appearance / Extensions** say what is coming (LJ27 / LJ30); no controls yet.
- **`ui/settings-logic.ts`** (11 tests, 4 before): tabs (`TABS`, `tabFrom`, `tabForKey` for
  ↑/↓/Home/End without wrapping), change detection across tabs (`rememberHistory` counts; edits
  on General and Privacy make one Save), the clear action. The old tests changed only where the
  input gained `rememberHistory`.
- **Styles:** a new `ui/settings.css` (the settings window only; style.css keeps the shared
  tokens): the sidebar uses the toolbar's tint and the command bar's selection color, so the
  window matches the command bar in light and dark. The old settings rules left `style.css`,
  which brings it back toward 400 lines. One of them, a settings `.row` rule, had also given the
  command bar's rows their 12 px spacing; that margin is now on the command bar's own `.row`
  rule, so its look is unchanged (the history view's denser rows set their own).
- **Smoke:** the history-off run (hibernating, before it quits) runs `--settings` → General must
  open and load (`settings-general.png`), then `--settings privacy` must switch the open window
  (`settings-privacy.png`, Remember history off there), and no page window may open
  (`settingsFailures`, 3 tests).
- Files beyond the prompt's list: `ui/settings.css`, `ui/command/{rows,command}.ts` (+ test),
  `src-tauri/src/{state,menu,tray}.rs`, `scripts/ci/smoke-command.mts` (+ test).
- `npm run verify`: 179 TS tests, 162 Rust tests, clippy clean. Disk: 13 GiB free at the start
  and after verify; `.cache/target` 1.2 GB.
- **CI (run 39077448620): every job green** — verify ×3, package ×3, smoke on Windows, macOS,
  Linux and Linux at scale 2; every job logged `settings-window general`, `settings-loaded`,
  `settings-window privacy` and opened no page window. `settings-general.png`: the window
  centred, the tinted sidebar with General selected (accent fill, white text and icon) over
  Appearance, Extensions, Privacy, About with their glyphs; on the right "General", the shortcut
  (`Ctrl+Shift+Space`, `⌘⇧Space` on macOS), the two lists, Mobile sites checked, Start at login
  unchecked, each note muted under its row; Cancel and a dimmed Save along the bottom across
  both columns. `settings-privacy.png`: Privacy selected; Remember history unchecked (this run
  has it off), its note, "Clear history" with "Last hour" and "Clear…", "What is stored where"
  with the run's data folder, its `webview` folder (Windows, Linux) or "WebKit's own store …"
  (macOS), session cookies in memory, and the closing note. Linux draws its lists GTK-style
  (taller, bordered); at scale 2 everything is crisp. History rows are now tight (`history.png`),
  and the command bar's rows look as before (`command-typing.png`). **One finding, macOS:** its
  wider text wrapped the Privacy tab about a line past the window, so a scrollbar showed. **Fix
  (`lj-step-26-fix`):** a little less padding above the tab and its heading, and a shorter
  Remember-history note. Run 39078418578 (every job green): macOS `settings-privacy.png` now
  ends with the closing note above Save / Cancel and no scrollbar; Windows and Linux unchanged.

### LJ27 — Theme engine · strong · auto

> Phase 15. Branch `lj/p15-themes` from `main`.

**Verify:** `npm run verify` green, including:
- vitest tests for `ui/lib/theme.ts`:
  - hex ↔ OKLab/OKLCH conversions match known values;
  - the clamps (light L 0.86–0.93, dark L 0.22–0.32, C ≤ 0.12 × intensity) hold;
  - the gradient CSS for 1, 2 and 3 colors is deterministic;
  - text reaches a contrast ratio of at least 4.5:1 against every gradient stop for every
    preset in both modes;
  - the grain overlay is an SVG data URI with `feTurbulence` at the grain's opacity;
- Rust `settings.rs` tests for a tolerant `theme` field: invalid colors are dropped, 1–3 are
  kept, numbers are clamped, mode is `system|light|dark`, and the default is the neutral preset.

The theme is applied to the toolbar, the command bar, the history view and the settings window,
and changes reach an open window without reopening it. CI green; with a two-color, grain 0.4
theme written into `settings.json`, the screenshots `command-bar.png` and `page.png` show the
gradient, plus a `mode: dark` run's `command-bar-dark.png`.
**Commit:** `lj-step-27: theme engine — 1–3 muted colors, intensity, grain, light/dark/system`

```text
Arc-like themes (decisions: "v0.2 round 2", Themes bullet). All color math in TypeScript, so all
three engines get plain sRGB values.
1. ui/lib/theme.ts (pure, tested):
   - conversions between hex and OKLab/OKLCH;
   - `mute(color, mode, intensity)` with the decided clamps;
   - `tokens(theme, systemDark)` → CSS variables: --lj-bg-0..2, --lj-fg, --lj-fg-muted,
     --lj-accent, --lj-field, --lj-row-hover, --lj-row-selected;
   - `gradientCss(tokens)`: a base linear gradient in sRGB stops, computed in OKLab for the
     midpoints, plus radial blobs for colors 2 and 3;
   - `grainUri(grain)`: an SVG feTurbulence tile;
   - about 10 PRESETS of muted palettes.
   Text color: pick near-black or near-white so the contrast test holds.
2. settings.rs: `theme` (tolerant, with tests). get_settings and save_settings carry it.
3. Applying it:
   - the bar reads the theme on load (start_info gains it) and exposes
     window.littleJoan.setTheme;
   - after a save, Rust pushes the new theme to the bar with eval and to the settings window
     with an event;
   - ui/style.css switches to the --lj-* variables; system mode follows prefers-color-scheme;
   - ui/index.html stays neutral, since it has no IPC.
4. Smoke: write the theme into settings.json for the screenshot runs (a separate launch, so the
   main flow's memory numbers don't move), and take the screenshots from Verify.
5. Docs: README (themes), docs/architecture.md, CHANGELOG, this plan.

Do NOT build the editor (LJ28).

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ27 to ✅, add notes, and
commit: "lj-step-27: theme engine — 1–3 muted colors, intensity, grain, light/dark/system". Push,
open the phase-15 PR into main as a draft, and watch CI until every job is green (two honest
attempts, then stop). Look at the screenshots in light and dark.
```

**Implementation notes (2026-10-18):**

- **Files:** the color math is its own module, `ui/lib/color.ts` (hex ↔ sRGB ↔ OKLab ↔ OKLCH with
  Ottosson's matrices, fitting into sRGB by lowering chroma at the same lightness and hue,
  mixing in OKLab, WCAG contrast; 9 tests), so `ui/lib/theme.ts` (mute, backgrounds, palette,
  `gradientCss`, `grainUri`, `tokens`, `themeFrom`) stays under 400 lines; the conversion tests
  the Verify lists for theme.ts are in `color.test.ts`. Presets are `ui/lib/presets.ts` (11, the
  first, **Mist** `#8c9bb5` at intensity 0.45 and grain 0.15, is the default). Rust:
  `settings/theme.rs` (a child module of settings.rs; 5 tests), `Settings` lost `Eq` (the theme
  has floats).
- **Interpretations, stated:** a pick's lightness is **mapped** into the band (light 0.86–0.93,
  dark 0.22–0.32: L′ = low + (high − low) · L) rather than clamped, so it always holds the clamps
  and lighter picks stay lighter — clamping would have pushed nearly every pick onto the band's
  edge. Chroma is min(C, 0.12) · intensity (≤ 0.12 × intensity, and the slider scales every pick).
  One pick gets two neighbours (lighter/24° warmer, darker/30° cooler); two picks get their
  OKLab midpoint as the third background.
- **Tokens beyond the prompt's list** (the surfaces needed them): `--lj-on-selected` (white on
  the selected row), `--lj-panel` (the frosted card: light 66 % / dark 52 % opaque, so the
  gradient shows through), `--lj-surface` (that card over `--lj-bg-0`, opaque), `--lj-tile`
  (favicons sit on a light tile in both modes: the preview showed GitHub's black icon vanishing on
  a dark one), `--lj-hairline`, `--lj-shadow`, `--lj-gradient`, `--lj-grain`. The old `--bg`,
  `--fg`, `--muted`, … are gone; every color in style.css, history.css and settings.css is an
  `--lj-*` variable. The built-in rows' symbols (gear, search, clock) became CSS masks in the
  muted text color (they were grey SVGs that disappeared on dark panels).
- **Contrast:** text, muted text and the accent are stepped (OKLCH L in 0.01 steps from
  near-black / near-white) until they reach 4.5:1 against every gradient stop, the grain's
  darkest and lightest speck over each, and the panel over all of those; the selected row is dark
  enough for white text. Tested for every preset in light, dark and system-dark, and 150 random
  themes (seeded).
- **Grain:** the first version (grey noise with its own alpha) was invisible; measured on a canvas
  in Chromium it moved pixels by a standard deviation of under 1 level. The tile now takes the
  red channel stretched to full contrast with full alpha (`feColorMatrix`), at 0.14 × grain:
  grain 0.4 → about 4 levels of noise, the default 0.15 → 1.6.
- **Applying:** `apply-theme.ts` sets the variables on `<html>` with `style.setProperty` (CSSOM is
  allowed by `style-src 'self'`; an injected `<style>` would not be) plus `color-scheme`; system
  mode re-applies on the media query's change. The bar applies `start_info`'s new `theme` as it
  loads (`panel.ts` no longer asks start_info itself; `bar.ts` does and hands it the answer),
  the settings window `get_settings`' `settings.theme`. Until then `ui/lib/theme-defaults.css`
  (written by `npm run theme:css`, `@import`ed by style.css, checked by a test, as is the Rust
  default against presets.ts) shows the default theme for the system's scheme.
- **Deviation:** after a save Rust pushes the theme to the settings window with `eval`
  (`window.littleJoanSettings.setTheme`), like `showTab`, not with an app event: listening to
  events would have given the settings window `core:event:allow-listen` / `allow-unlisten`, its
  first core permissions beyond `allow-close`, for no gain.
- **`save_settings`** takes an optional `theme` (left out, the saved one stays; given, it goes
  through the same tolerant parse); LJ27's settings window doesn't send one yet.
- **Looks, tuned locally before CI:** the real `bar.html` / `settings.html` in iframes with a
  mocked `__TAURI_INTERNALS__`, served by Vite and viewed in Chromium (a throwaway harness in the
  gitignored `test-results/`, never committed). Changes from it: overlay padding 10 → 12 px, the
  panel's opacity, the favicon tile, the grain.
- **Smoke:** `smoke-theme.mts` (new, 4 tests) holds the three theme runs and `themeFailures`; each
  run is its own launch with the theme in settings.json (the main run is untouched): two colors
  `#9d8fd0` + `#e0a2a0`, intensity 0.6, grain 0.4 in light (`theme/page.png`,
  `theme/command-bar.png`, `theme/history.png`, `theme/settings.png`) and dark
  (`theme/page-dark.png`, `theme/command-bar-dark.png`, `theme/settings-dark.png`), and the
  default theme in dark (`theme/command-bar-default-dark.png`). The screenshots live in
  `<out>/theme/` so the main run's `command-bar.png` / `page.png` (default theme, light) keep
  their names. Rust logs `theme <mode> colors=<n>` from `start_info`.
- Files beyond the prompt: `ui/lib/{color,presets,theme-css,apply-theme}.ts`,
  `ui/lib/theme-defaults.css`, `scripts/theme-css.mts` (+ `npm run theme:css`), the
  `ui/command/{panel,welcome}.ts` and `ui/settings{.ts,-logic.ts}` glue, `ui/{bar,settings}.html`
  (the `themed` body class), `history.css`, `settings.css`, `bar.rs` / `settings_window.rs`
  (`set_theme`), `scripts/ci/smoke-theme.mts` (+ test).
- `npm run verify`: 207 TS tests, 168 Rust tests, clippy clean. Disk: 13 GiB free at the start
  and after verify; `.cache/target` 1.3 GB.
- **CI (run 39082533736): verify red on Windows only** — the check that `theme-defaults.css` is
  current compared bytes, and Windows' checkout had turned its line endings into CRLF.
  **Fix (`lj-step-27-fix`):** the test compares with line endings normalized (they mean nothing
  in CSS).
- **CI (run 39082804310, with that fix): every job green** — verify ×3, package ×3, smoke on
  Windows, macOS, Linux and Linux at scale 2; every theme run logged its own `theme <mode>
  colors=<n>`. Screenshots: **the default theme** (main run, light) — `command-bar.png` /
  `page.png` as before but on a faint cool blue-grey gradient: the frosted card is near-white, the
  toolbar strip a pale slate; `theme/command-bar-default-dark.png` — a deep slate gradient, the
  card a darker frosted slate, text near-white. **The two-color theme, light**
  (`theme/command-bar.png`, `page.png`, `history.png`, `settings.png`): lavender at the top left
  blending into peach at the top right and lavender-pink below, a 12 px frame of it around the
  card (on macOS also the 40 px strip with the traffic lights), the card faintly pink-white; the
  toolbar strip lavender → peach with the domain and buttons in near-black; the history view and
  Settings' sidebar the same gradient, the selected tab a muted violet with white text. Enlarged
  crops: fine grain (visible up close, not at a glance), no banding, text crisp. **Dark**
  (`command-bar-dark.png`, `page-dark.png`, `settings-dark.png`): deep plum to aubergine with
  grain, the card a darker frosted plum, near-white text and muted grey domains; on Linux the GTK
  lists turn dark with `color-scheme`; on macOS the settings window's title bar turns dark too.
  All four OSes alike; Windows and Linux keep their native (light) title bars. **One finding:**
  macOS `theme/page.png` and Linux scale-2 `theme/page-dark.png` showed the 40 px bar still in
  the command-bar view (an empty strip on macOS, the card's squeezed outline at scale 2) above the
  page — the run opened the page with `--show <url>` while hibernating, and the bar's log shows
  it asking for suggestions after `overlay false`. Cause: the re-tell of the view when the bar's
  page finishes loading can come before the bar's script runs (the bar's `start_info` came 90 ms
  after `bar-loaded` on macOS, and never within 0.9 s in the history-off run), so
  `window.littleJoan` didn't exist yet and the change was lost. A pre-existing race that the
  main run never showed (it opens the page into an already-loaded bar). **Fix
  (`lj-step-27-fix2`):** `start_info` — which the bar's script calls once it runs — also tells
  the bar its view and back state (`bar::tell_state`, through the same ordered eval channel, so a
  later change still wins).
- **CI after fix 2 (run 39083883621):** the fix worked — macOS `theme/page.png` now shows back,
  the site and open-in-browser on the lavender → peach strip beside the traffic lights, and Linux
  scale-2 `theme/page-dark.png` the near-white domain on the plum strip — but `smoke (macos)` was
  red on one check: 2 WebContent helpers still alive 10 s after the third quick summon / hide
  cycle (`afterEachCycle [0, 0, 2]`; the 60 s samples after the page were 0). The runner was slow
  in that cycle (the bar's first `suggest` came 1.3 s after it loaded) and nothing in fix 2
  touches releasing webviews; the next full run (LJ28's) had `[0, 0, 0]` on every OS. Recorded as
  a one-off; it was the first time this check failed in any run.

### LJ28 — Appearance editor · standard · auto

**Verify:** `npm run verify` green, including vitest tests for `ui/settings/appearance-logic.ts`:
- pad coordinates ↔ hue/chroma (round trip within the muted range);
- adding and removing colors (1–3);
- presets (at least 10, all passing LJ27's contrast test);
- preview vs. saved state (Cancel restores the saved theme).

The Appearance tab has mode buttons (System / Light / Dark), preset swatches, up to three color
pads (a dot on a hue/chroma grid, like Arc's), intensity and grain sliders, and a live preview of
the toolbar and the command bar. The open window previews while you edit, and Save keeps it. CI
green; screenshot `settings-appearance.png` via `--settings appearance`.
**Commit:** `lj-step-28: appearance editor with color pads, presets and live preview`

```text
The editor for LJ27's themes, in the settings window's Appearance tab.
- A color pad is a 2-D grid: x is hue (0–360) and y is chroma (0 to the muted maximum). Drag
  the dot with the mouse, or move it with the arrow keys for accessibility.
- "+" adds a color (up to 3); clicking a color's dot selects it, and a button removes it.
- Presets apply in one click.
- Intensity and grain are range inputs.
- While editing, the settings window sends a preview to the page window (an app event; not
  saved). Cancel restores the saved theme; Save persists it.

Files: ui/settings/ (appearance view and logic, + tests), ui/settings.{html,ts}, ui/style.css,
src-tauri/src/{commands.rs (preview_theme), lib.rs}, build.rs, capabilities/settings.json,
scripts/ci/smoke.mts (screenshot), README, CHANGELOG, this plan.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ28 to ✅, add notes, and
commit: "lj-step-28: appearance editor with color pads, presets and live preview". Push and watch
CI until every job is green (two honest attempts, then stop). Look at the screenshot. This ends
phase 15: fill the PR description, merge into main once green, then stop and report.
```

**Implementation notes (2026-10-18):**

- **`ui/settings/appearance-logic.ts`** (pure, 12 tests): pads (`colorAt` / `pointFor`: x = hue
  0–360°, y = chroma from 0.12 at the top to grey at the bottom), `nudge` (arrow keys: 5° / 5 %,
  Shift 30° / 20 %; hue wraps, chroma stops at the edges), `describePoint` (the pad's
  `aria-valuetext`), `padField` (the pad's background: 13 hue stops at full pad chroma under a fade
  to grey), `nextColor` (**+**: the last color's chroma, 45° further round), the editor state
  (`saved`, `draft`, `selected`) and its `edit` steps (mode, preset, select, move, add, remove,
  intensity, grain, cancel, saved), `sameTheme`, `isDirty`, `presetOf`, `swatch`, the sliders'
  0–100 mapping.
- **Pads at OKLCH L 0.75:** the one lightness where every hue reaches the pad's top chroma (0.12,
  the themes' cap) inside sRGB (computed: the weakest hue, 200°, reaches 0.1275 there; at 0.78
  blue falls to 0.111), so the pad is a true hue × chroma grid and the round trip holds. A pick is
  stored at that lightness; the theme maps it into its band. The round-trip test allows the hue
  error that rounding to `#rrggbb` really costs (about 0.07 / chroma degrees, so more near grey).
- **One pad per color** (the decision's "one hue/chroma pad per color"), three side by side
  (120 × 64 px); the selected pad has the accent ring and, with more than one color, a × in its
  corner; an empty slot holds the dashed **+**. Focusing a pad selects its color.
- **Presets** are round swatches of each preset's own colors (the first version drew them as the
  muted backgrounds and they were too pale to tell apart in the local preview); the one the theme
  matches has the accent ring. Mode buttons sit beside the heading. The live preview is a small
  toolbar strip and command bar card over the theme's gradient; the whole settings window wears
  the draft too, so everything re-colors as you edit. All of it fits the 600 × 480 window without
  a scrollbar (checked in the local preview; CI's screenshot below).
- **Previewing:** a new command `preview_theme(theme?)` (the settings window only;
  `allow-preview-theme`) — the prompt's "app event" — checks the theme like a saved one, keeps it
  in `AppState::theme_preview` (so a page window summoned while editing shows it too: `start_info`
  hands out `shown_theme()`), and pushes it to the bar (`bar::set_theme`). The tab sends at most
  one preview a frame. Closing Settings without saving (Cancel or the close button:
  `settings_window`'s `Destroyed`) clears the preview and puts the saved theme back in the bar;
  `save_settings` clears it too (the preview is the theme now). Smoke log: `theme-preview <mode>
  colors=<n>` / `ended`.
- **Save / Cancel:** `toInput` takes the draft theme (an optional 6th argument, so the existing
  calls stay as they are) and `isChanged` compares it (`sameTheme`), so an edited theme enables
  Save; Save sends it and Rust pushes the saved theme to the bar and the window.
- **Styles** went into a new `ui/settings/appearance.css` (linked by settings.html, like the bar's
  `history.css`) rather than `ui/style.css`, which the bar loads too. Buttons there are
  `button.<class>` because settings.css styles `.settings-body button:not(.tab)` — a first try lost
  the `[hidden]` rule of the × that way, and the local preview showed it on every pad.
- **Checked locally** in the preview harness (Chromium, mocked IPC; not committed): a preset, a
  mode, an arrow key and **+** each changed the window's variables and `color-scheme`, previews
  were coalesced (two `preview_theme` calls for four edits), Save became enabled. Dragging and the
  page window following live need a real window: human check (LJ35).
- `theme.ts` gained `minimumContrast(theme, systemDark)` for the presets test.
- **Smoke:** the history-off run adds `--settings appearance` after Privacy →
  `settings-appearance.png` (default theme); `settingsFailures` wants Appearance after Privacy.
  The theme runs open Settings on Appearance (`theme/settings.png`, `theme/settings-dark.png`).
- Files beyond the prompt: `ui/settings/appearance.css`, `ui/settings-logic.ts` (+ test),
  `src-tauri/src/{state,settings_window}.rs`, `ui/lib/theme.ts`, `scripts/ci/smoke-command.mts`
  and `smoke-theme.mts` (+ tests), docs/architecture.md.
- `npm run verify`: 220 TS tests, 169 Rust tests, clippy clean. Disk: 13 GiB free at the start
  and after verify; `.cache/target` 1.3 GB.
- **CI (run 39084768446): every job green** — verify ×3, package ×3, smoke on Windows, macOS,
  Linux and Linux at scale 2; Settings opened on General, Privacy, Appearance in every job, every
  theme run logged its own theme, no webview helpers left after any cycle. `settings-appearance.png`
  (default theme, all OSes alike): "Appearance" with System pressed beside it, the preview (a pale
  slate toolbar strip with "example.com" over a card with the selected slate row), the 11 swatches
  with Mist ringed, one pad with its dot in the blue-grey area and the dashed +, Intensity and
  Grain; no scrollbar, the footer clear. `theme/settings.png` (two colors, light): the same on the
  lavender/peach gradient, Light pressed, two pads (× on the selected one, dots at violet and
  peach), the selected tab and row a muted violet. `theme/settings-dark.png`: plum, Dark pressed;
  on Linux the sliders and lists draw dark. On Windows the preview card's second row is cut at the
  preview's bottom edge mid-line (its larger font pushes it down); intended to read as "the list
  goes on", left as is.

### LJ29 — Extension runtime · strong · auto

> Phase 16. Branch `lj/p16-extensions` from `main`.

**Verify:** `npm run verify` green, including Rust tests for:
- **Manifest validation:**
  - id `^[a-z0-9][a-z0-9-]{1,31}$`, `apiVersion` 1;
  - permissions only from `storage`, `clipboard`, `pageInfo` and `network:<https host>`
    (optionally with a leading `*.`);
  - `contributes.commands` / `pageActions` whose panel paths are inside the folder.
- **Path resolution:** no `..`, no absolute paths, no percent-encoded traversal, no symlink
  leaving the folder.
- **The CSP builder:** `default-src 'self'; script-src 'self'; style-src 'self'
  'unsafe-inline'; img-src 'self' data: blob:; media-src 'self' data: blob:; font-src 'self'
  data:; connect-src 'self' <declared hosts>; frame-src 'none'; object-src 'none'; base-uri
  'none'; form-action 'none'`.
- **API request checks:**
  - the webview is `joan-page`;
  - the Origin is the extension's ljext origin;
  - the page's current URL is inside that extension;
  - the request is a POST with the `X-LJ-Api: 1` header;
  - the method is on the allowlist and the permission is declared;
  - storage is capped at 1 MB per extension, with keys of 128 characters at most.
- **The navigation guard:**
  - `ljext:` URLs are allowed in `joan-page` only when the app opened that extension;
  - Windows' `http(s)://ljext.localhost` form counts as ljext, never as plain http;
  - `ljext:` is never allowed in the bar.

CI green on all three:
- `--extension hello` shows `LJ-EXT hello storage=ok theme=ok ipc=denied net=blocked` in the
  title;
- the smoke page's own `fetch` to the ljext API reports `ljext-api=blocked`.
**Commit:** `lj-step-29: extension runtime — manifest, ljext protocol, per-extension CSP, checked API`

```text
The extension platform (decisions: "v0.2 round 2", Extensions bullet; Ground Rules 18–19;
Landmines on custom schemes and Windows iframes). Experimental, but the security model is not.

1. src-tauri/src/extensions/ (keep each file under ~400 lines):
   - manifest.rs: types, parse and validate (pure, tested);
   - registry.rs: bundled extensions plus, when settings `developerMode` is true (default
     false), unpacked folders in `<config dir>/extensions/<id>/`; a bad extension is skipped
     with a logged reason, never a crash;
   - protocol.rs: register_asynchronous_uri_scheme_protocol("ljext"). It serves files with the
     CSP from the manifest, the right MIME type, and `X-Content-Type-Options: nosniff`, and
     answers `/__lj/sdk/v1.js` (the JS SDK, an embedded string) and `/__lj/api/v1/<method>`
     (api.rs);
   - api.rs: the request checks from Verify, then these methods: storage.get/set/remove/keys
     (`<config dir>/extension-data/<id>.json`, saved atomically), theme (the current tokens
     from LJ27), pageInfo (the URL and title the panel was opened for; pageInfo permission),
     openUrl({url, where: page|browser}) (http/https only), clipboard.writeText (clipboard
     permission), and close (back to the previous page, or the command bar).
2. Bundled extensions: a top-level `extensions/<id>/` folder, embedded at build time by build.rs
   (it generates `$OUT_DIR/bundled_extensions.rs` with include_bytes!; no new crate). Add
   `extensions/hello/`, the template and smoke target: it calls storage and theme, tries
   `window.__TAURI_INTERNALS__?.invoke("start_info")` (must fail or be absent), tries a fetch
   to https://example.com (the CSP must block it), and reports the line from Verify in its
   title.
3. URLs and origins: try one origin per extension (`ljext://<id>/…`) on macOS/Linux, and find
   what WebView2 maps it to. If Windows can't give each extension its own origin, use
   `ljext://localhost/<id>/…` everywhere and document that localStorage is shared, so authors
   must use littleJoan.storage. Record what you find under Landmines.
4. navigation.rs and bar.rs: the guard from Verify. Opening an extension = closing the overlay
   and navigating joan-page to its panel URL, with an "expected ljext URL" flag the guard
   consumes. The toolbar shows the extension's name and icon instead of a domain.
5. cli.rs: `--extension <id>` (forwarded; it opens the window if needed). Smoke: run it and the
   smoke page's blocked-API fetch; add the smoke-log rules with tests.
6. Docs: docs/architecture.md (an extensions section with the trust diagram), CHANGELOG, this
   plan. (docs/extensions.md comes in LJ30.)

Do NOT show extensions in the command bar or Settings yet (LJ30), and do not add third-party
quick-answer providers (Later).

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ29 to ✅, add notes, and
commit: "lj-step-29: extension runtime — manifest, ljext protocol, per-extension CSP, checked API".
Push, open the phase-16 PR into main as a draft, and watch CI until every job is green (two
honest attempts, then stop).
```

**Implementation notes (2026-10-18):**

- **Origins: one per extension on every OS** (the prompt's step 3). macOS / Linux:
  `ljext://<id>.localhost/<path>` (origin `ljext://<id>.localhost`). Windows: WebView2 can't load
  custom schemes, so wry serves every `http://ljext.*` request from the handler and turns
  `http://ljext.` back into `ljext://` first (wry 0.57 `custom_protocol_workaround.rs`, filter
  `http://ljext.*`); `ljext://<id>/` would have become `http://ljext.<id>/`, a public-looking
  host — for ids like `app` or `dev` Chromium upgrades HSTS-preloaded TLDs to https, which misses
  the handler and reaches the internet. So the host carries `.localhost` everywhere and Windows
  uses `http://ljext.<id>.localhost` (never resolved, never upgraded, a secure context). The app
  navigates to the Windows form itself (wry's `load_url` doesn't rewrite custom schemes). No
  shared origin, so localStorage is per extension too; the docs still point authors at
  `littleJoan.storage`. Recorded under Landmines.
- **Modules** (`src-tauri/src/extensions/`): `manifest.rs` (strict: unknown fields refused;
  texts limited, no control characters; permissions `storage`, `clipboard`, `pageInfo`,
  `network:<host>` — lower-case DNS names, optional `*.`, two labels at least, no IPs, ports or
  schemes; panels `.html` and icons `.svg` / `.png`, relative, plain), `paths.rs` (plain names
  only — letters, digits, `. _ -`, no dotfiles, no trailing dot, no Windows device names, at most
  8 deep, `__lj` reserved — so `..`, absolute paths, drive letters, backslashes and every
  percent-encoding are refused without decoding anything; on disk the canonical path must stay
  inside the folder's canonical path), `urls.rs`, `csp.rs`, `registry.rs` (bundled + developer
  folders; folder name = id; every named file must exist; an unpacked extension can't take a
  bundled id; at most 64; files ≤ 16 MB), `api.rs` (+ `api/tests.rs`), `protocol.rs`,
  `storage.rs`, `theme.rs`, `clipboard.rs`, `session.rs`. Tests for manifest and registry live in
  child `tests.rs` files (as history's do).
- **Checks, in order** (`api::route`; the scheme is registered for every webview): the webview
  is `joan-page`; the URL names an existing extension; files / SDK: `GET`, not for a frame
  (`Sec-Fetch-Dest` iframe / frame / embed / object / fencedframe), the page shows that
  extension, a plain path of a served type; API: `POST`, `X-LJ-Api: 1` exactly once, `Origin`
  exactly the extension's origin (once; `null`, other extensions, the other OS's form refused),
  `Sec-Fetch-Site: same-origin` when present, the page shows that extension **and the app opened
  it** (the session's id), the method on the allowlist, the permission declared. Every check has
  its own test with a request that passes all the others. **Additions beyond the prompt, the more
  restrictive choices:** serving files only while the page shows that extension (otherwise a web
  page could probe which developer extensions are installed with `<img>`), the frame and
  same-site checks, `frame-ancestors 'none'`, `X-Frame-Options: DENY`,
  `Cross-Origin-Resource-Policy: same-origin`, a `Permissions-Policy`, `no-store`,
  `Referrer-Policy: same-origin` (first `no-referrer` — see the CI notes below),
  and refusing `blob:` / `data:` documents while a panel is shown (they would keep its origin).
  URL parsing resolves `.` / `..` (also as `%2e`) before any path is checked, and never above the
  extension's root (tested).
- **The guard** (`session::NavGate`): opening an extension sets the "expected URL" (exact string,
  used once, fresh for 5 s) before `bar::navigate`; `allow_page_navigation` admits an extension
  URL in any form (`ljext:`, `http(s)://ljext.*`) only then. The Reload page action re-arms it for
  a panel. `navigation::is_allowed_navigation` refuses every extension-like URL (so on macOS and
  Linux a real site named `ljext.<something>` can't be opened either — accepted), and
  `is_external_openable`, history, the back trail and Translate ignore them. The bar's guard is
  `navigation::is_allowed_bar_navigation` and the settings window's `settings_window::may_navigate`,
  both tested against both forms. WebView2's `NavigationStarting` is main-frame only (frames are
  not guarded there), which is why serving files needs the page to show the extension.
- **The API's methods:** storage (`<config dir>/extension-data/<id>.json`, read per call, one
  lock, atomic write, 1 MB of JSON, keys 1–128 characters, a refused write changes nothing),
  `theme` (see below), `pageInfo` (null until LJ30's page actions), `openUrl` (absolute http(s),
  never extension-like; `page` replaces the panel, `browser` hands it to the opener), `close`
  (back to the web page the panel was opened from, else the start page with the command bar),
  `clipboard.writeText` — **native** (NSPasteboard via objc2's `msg_send!`, GTK's clipboard, the
  Win32 clipboard: two features added to the existing `windows` dependency,
  `Win32_System_DataExchange` and `Win32_System_Memory`; no new crate), because a panel's own
  `navigator.clipboard` needs a click and the bar can't write without one on WebKit.
- **Theme tokens:** all theme math is TypeScript, so the bar reports the `--lj-*` variables it
  painted with a new bar command, `set_theme_tokens` (bar capability only; checked: `--lj-*`
  names, ≤ 40 of them, short values without `;{}<>\`), kept in memory while the window exists;
  `theme` waits up to 3 s for the first report (a panel opened with the window can ask before the
  bar loaded). `apply-theme.ts` gained an `onPaint` hook that only the bar passes.
- **The toolbar** shows a panel's extension: `session::publish_label` sends
  `window.littleJoan.setExtension({ name, icon })` (the icon as a base64 `data:` URI made in Rust
  — the bar can't and mustn't load `ljext:`; a 20-line base64 encoder, tested), again with the
  bar's `start_info`; `ui/toolbar/logic.ts` checks it (name ≤ 40 characters, icon only
  `data:image/svg+xml|png;base64`), draws the name with `textContent`, and disables "open in
  default browser".
- **Bundled extensions** are embedded by `build.rs` (`$OUT_DIR/bundled_extensions.rs`,
  `include_bytes!`, folders with an `extension.json`; dotfiles, symlinks and `*.test.*` left
  out). `extensions/hello/` (manifest, `index.html`, `hello.js`, `hello.css`, `icon.svg`) counts
  its openings in storage, applies the theme, calls `__TAURI_INTERNALS__.invoke("start_info")`
  (must reject, 4 s timeout), fetches `https://example.com` with `no-cors` and reports
  `net=blocked` only when the fetch failed **and** a `securitypolicyviolation` for connect-src
  named that URL (a plain network failure would say `failed`). The SDK is `extensions/sdk/v1.js`
  (a classic script defining a frozen global `littleJoan`) with types in `v1.d.ts`;
  `extensions/` joined `tsconfig.json` (checkJs) and ESLint (with `no-undef` left to tsc for
  those files, as for TS).
- **`--extension <id>`** (`cli.rs`, `toggle.rs`: `Command::Extension` → `CreateExtension` /
  `FocusAndExtension`): opens the extension's first command; an unknown id opens nothing (logged
  `extension-missing`).
- **Smoke** (`smoke-extensions.mts`, 5 tests; `tests/smoke/ext-probe.mjs`, 3 tests): its own
  launch — `--extension hello` (title, `extension-opened`, the panel's `page-loaded`, storage and
  theme calls `ok`, `extension-hello.png`), then the smoke page with `?ext=probe&base=<hello's
  origin>`: the SDK-style API call and a simple text/plain POST (5 s timeout each), an `<img>`
  of hello's icon, `location.href` to the panel; it must report `LJ-EXTPROBE ljext-api=blocked
  ljext-file=blocked ljext-nav=refused`, with no `ext-api … ok`, no extension `page-loaded` and a
  `navigation-refused` after it, and hello's `extension-data/hello.json` holding its count and no
  `smoke-probe` key (`extension-probe.png`). New log events: `extension-opened`,
  `extension-missing`, `extension-skipped`, `ext-api`, `ljext-refused`, `navigation-refused`.
- **Files beyond the prompt:** `commands.rs`, `lib.rs`, `capabilities/bar.json`
  (`set_theme_tokens`), `settings.rs` (`developerMode`), `toggle.rs`, `joan.rs`, `page_actions.rs`,
  `history.rs`, `back_stack.rs`, `settings_window.rs`, `platform/*.rs` (clipboard), `bar/watch.rs`
  (the address watcher moved out of `bar.rs` to keep it under 400 lines), `ui/bar.ts`,
  `ui/toolbar/*`, `ui/lib/apply-theme.ts`, `tsconfig.json`, `eslint.config.mjs`, README (the
  flag, privacy), SECURITY.md.
- `npm run verify`: 230 TS tests, 230 Rust tests, clippy clean. Disk: 13 GiB free at the start
  and after verify; `.cache/target` 1.3 GB.
- **CI (run 39089886960): verify ×3 and package ×3 green — the Win32 / GTK / AppKit clipboard code
  compiled on its first try — and `smoke (windows)` green; `smoke (macos)`, `smoke (linux)` and
  `smoke (linux-scale-2)` red on one thing:** hello reported `storage=error-origin
  theme=error-origin ipc=denied net=blocked` — every API call of the panel's own was refused by
  the Origin check (`ljext-refused origin joan-page ljext://hello.localhost/__lj/api/v1/…`).
  Cause: the responses said `Referrer-Policy: no-referrer`, and under that policy a same-origin
  (non-CORS) POST carries `Origin: null` (the Fetch standard's "serializing a request origin";
  WebKit follows it, Chromium evidently didn't). Everything else held on all four jobs: `ipc=denied`,
  `net=blocked` (a real connect-src violation), and the probe — `ljext-api=blocked
  ljext-file=blocked ljext-nav=refused`, with the log showing **why**: the web page's SDK-style POST
  reached the handler on WebKit **without a CORS preflight** (custom schemes skip it there) and was
  refused for its Origin, its simple POST for the missing `X-LJ-Api`, its `<img>` for the page
  check, and `location.href` by the guard (`navigation-refused`). So the Origin check is what
  stops a web page's request on WebKit, not the preflight. **Fix 1 (`lj-step-29-fix`), on a wrong
  guess:** `Referrer-Policy: same-origin` (nothing goes to other origins, so no more leaks than
  before; kept), the SDK's own fetch sets `referrerPolicy: "same-origin"` too, and an Origin
  refusal logs what the engine sent. Run 39090864816 (verify, package, Windows smoke green again;
  the three WebKit smoke jobs red the same way) showed `origin=-`: **WebKit sends no Origin at all**
  with a same-origin POST to a custom scheme (it adds one only on the CORS path; the web page's
  cross-origin POST had `origin=http://127.0.0.1:…`), not `null`. **Fix 2 (`lj-step-29-fix2`):**
  `api::origin_ok` — an Origin that is sent must be exactly the extension's (once; `null` and an
  empty one refused); Windows must send one (WebView2 always does); on macOS and Linux a missing
  Origin is the extension's own request, unless a Referer from another origin comes with it.
  Every cross-origin request carries an Origin on WebKit (spec, and the probe), so a web page's
  call is still refused by it, and the page, webview and header checks are unchanged. Tests:
  `webkit_sends_no_origin_from_the_extension_itself` (accepted on macOS / Linux, refused on
  Windows, foreign Referer refused, empty Origin refused, other checks still apply); the refusal
  log now carries the Referer too.
- **CI after fix 2 (run 39091695597, with LJ30): LJ29's checks green on all four smoke jobs** —
  hello `storage=ok theme=ok ipc=denied net=blocked`, the probe refused (WebKit: the web page's POST
  by its `Origin: http://127.0.0.1:…`, its simple POST by the header, its `<img>` by the page check;
  Windows: the preflight by `http-method`, then the same), the guard's `navigation-refused` — but
  **the Windows log showed a race the rules didn't catch:** `--extension hello` while hibernating
  logged `extension-opened`, then the start page's late `page-loaded http://tauri.localhost/`, then
  `ljext-refused page … hello.localhost/index.html` — the panel's own document refused, its title
  the refusal's (`extension-hello.png` showed the JSON). The start page's `Finished` (WebView2's
  NavigationCompleted, delivered after the app had begun opening the panel) overwrote the page's
  address, which the page check read; on WebView2 `Started` comes only at ContentLoading, after the
  handler answered. The title the rule wanted came later from the command bar's second open.
  **Fix 3 (`lj-step-29-fix3`):** which extension the page shows is now decided by the navigation
  guard itself (`NavGate::showing`: an admitted panel URL sets it, any other navigation the guard
  allows clears it, refused ones change nothing) — every navigation passes the guard first and in
  order, while load events can come late. `api::Context` takes `showing` instead of the address
  (tests unchanged in substance: the helper derives what the guard would show from the page URL),
  `session::tests::the_guard_decides_which_extension_the_page_shows` is new, and the smoke rule now
  wants `--extension hello`'s own panel to report before anything opens it again, with none of
  hello's requests refused meanwhile (2 more test cases, one replaying CI's log). The blob / data
  rule uses the same state.
- **Security review (2026-10-18), an independent read-only review of PR #17 at `cdc3225`: no
  Critical or High findings; one must-fix and a list of cheaper ones, fixed on the branch:**
  - **M1 — the expected URL was predictable (`lj-step-29-fix4`).** On WebKit the guard is asked
    for frames too, so a web page looping `<iframe src="ljext://<id>.localhost/<panel>">` (or a
    malicious panel navigating and stopping) could consume the app's expectation in the moment
    between `expect` and the app's own navigation: the app's navigation was then refused while
    `navigating()` had already set the address, and the toolbar showed the extension's name and
    icon over the web page; Reload re-armed the gate from that stale address. **Fix:** the URL the
    app opens carries a fresh single-use 128-bit nonce, `?lj-open=<hex>` (`extensions/nonce.rs`:
    `/dev/urandom`, `BCryptGenRandom` — one feature added to the existing `windows` crate, no new
    crate — and a keyed-hash fallback that logs), and the guard wants that exact URL; the panel's
    own later navigations inside its extension stay refused (it is one page), so nothing ever
    re-arms the gate for another document. What the user sees now follows what a load event
    **committed** (`NavGate::commit`: the admitted panel's URL arriving in the page's own load
    event — since fix 6 only `Started`, the engines' commit callback, not `Finished`; since
    fix 10 exactly the admitted URL, nonce included), not the guard's admission; the app's extension navigations — opening,
    closing, `openUrl` to the page (also review item I5) — go through `bar::navigate_pending`, which
    leaves the page's address alone until those events arrive (`PageAddress::pending`). Reload on a
    committed panel opens the same thing again with a new nonce (`session::reopen`), never from
    the old address; on an uncommitted one the guard refuses it. Tests:
    `a_racing_document_cant_claim_the_expected_panel` (no-nonce, empty, wrong, extended and
    fragment forms all refused, the expectation survives, the app's URL is admitted, once),
    `the_toolbar_follows_what_a_load_event_committed`, `panel_urls_carry_the_nonce`,
    `a_pending_navigation_keeps_the_address_until_its_events_arrive`, nonce tests (1,000 distinct
    128-bit values, the OS source works, constant-time compare). Smoke: the race above
    (`raceFailures`, `labelFailures`; 3 test cases), with new log events `page-started`,
    `toolbar-label`, `new-window-refused`. Docs no longer say "web pages can't, in any form":
    architecture.md and extensions.md describe the nonce, the frames each engine shows the guard,
    and the commit rule.
  - **The cheaper findings (`lj-step-29-fix5`):**
    - **L4 (Windows) service / shared workers:** a worker would serve the extension's documents
      without the handler's policy or checks and outlive the panel. `api::embedding` refuses
      `Sec-Fetch-Dest: serviceworker` / `sharedworker` and any `Service-Worker` header, for every
      request (`Refusal::Worker`). Test: `service_and_shared_workers_get_nothing` (a dedicated
      `worker` stays allowed).
    - **I4 duplicated headers:** a `Sec-Fetch-Dest` sent twice is now a refusal (`X-LJ-Api` and
      `Origin` already were: `Incoming::header` treats a repeat as missing, and missing is refused).
      Tests in `files_are_get_only_…` (two values, and the same value twice).
    - **L1 `openUrl`:** to the default browser it needs a new manifest permission, `openBrowser`
      ("Open web addresses in your default browser" in Settings), and runs at most once every 2 s
      per extension (429 `busy`); both targets refuse this computer and the local network
      (`api::is_private_host`: 127/8, 10/8, 172.16/12, 192.168/16, 169.254/16, 0/8, `::1`, `::`,
      fc00::/7, fe80::/10, IPv4-mapped forms, `localhost`, `*.localhost`, `*.local`; the URL parser
      already turns `2130706433` and `0x7f000001` into 127.0.0.1). No bundled extension used it.
      Tests: `extensions_open_web_addresses_only` (21 private forms, 5 public),
      `open_browser_is_a_permission_of_its_own`, `a_rate_limit_counts_per_key_within_its_window`.
    - **L3 names:** manifest texts (name, description, author, titles, keywords) refuse Unicode
      format characters (Cf, incl. the bidi controls and U+200B–U+200F, U+FEFF, tags) as well as
      controls (`manifest::is_hidden_char`, a table of the Unicode 15 Cf ranges — std has no
      categories); a name that reads as a web address (a URL, `www.…`, a host name with an
      alphabetic TLD, an IP address) is refused (`looks_like_address`; "Node.js"-style names go
      too — restrictive by choice); the toolbar shows an **"Extension"** badge next to an
      extension's name (`toolbarView().badge`). Tests: `texts_refuse_invisible_and_bidi_characters`,
      `names_never_read_as_web_addresses`, toolbar `logic.test.ts`.
    - **L2 `network:` isn't a confidentiality boundary:** `X-DNS-Prefetch-Control: off` on every
      response (csp test), and docs/extensions.md now says plainly what the CSP doesn't cover
      (WebRTC, DNS prefetch, preconnect) and that extensions are local tools, not a sandbox for
      hostile code. Removing the WebRTC constructors is a Deferred follow-up (reason there).
    - **L5 resources:** `theme` no longer blocks a thread for up to 3 s — until the bar reports
      it answers at once with the default theme's light variables, parsed from the generated
      `ui/lib/theme-defaults.css` (test `the_default_theme_answers_at_once_until_the_bar_reports`);
      at most 8 API calls in flight per extension (`limits::Call`, released on drop; 429),
      `storage.set` at most 20 a second (429); a body whose `Content-Length` is over the limit is
      refused before parsing. What wry already did by then: every engine's handler gets the
      **whole body in memory** (WebView2 reads the request stream to the end, WebKitGTK reads
      `http_body`, WKWebView takes `HTTPBody` or drains `HTTPBodyStream`), so an oversized body is
      refused after it was buffered, never parsed. Tests: `limits::tests` (3).
    - **I2 `pageInfo`:** checked again inside the call — the opening's extension must be the
      caller — not only at routing.
    - **I3 ids:** `is_valid_id` refuses Windows device names (`con`, `nul`, `aux`, `prn`,
      `com1`–`com9`, `lpt1`–`lpt9`; `paths::DEVICES`), which would name a device as the storage
      file. Test: `ids_are_never_windows_device_names`.
    - **I1 a session token:** each opening gets a second 128-bit secret; the handler writes it into
      the SDK it serves to that panel (`__LJ_SESSION_TOKEN__`), and the API wants it as `X-LJ-Api`,
      compared in constant time (`nonce::same_secret`), instead of the fixed `1`. The WebKit
      missing-Origin acceptance no longer rests on the engine's behaviour alone: a request also
      needs a secret only the panel's own SDK has. Tests: `the_api_needs_its_header_exactly_once`
      (the old `1`, near misses, a prefix, padding, upper case, twice, no opening at all).
    - **What the handler receives, per OS:** the smoke log now records each `ljext` request's
      header names and the values the checks read (`ljext-headers … dest= site= mode= origin=
      service-worker=`; never the token or a Referer), to answer the reviewer's open question
      whether WebKit sends `Sec-Fetch-*` on custom-scheme requests. **CI run 39097343759: no
      engine does.** WebView2 passes `Accept`, `Referer` (on everything but the app's own
      navigation), `User-Agent`, `sec-ch-ua*`, `Upgrade-Insecure-Requests` on documents, `Origin`
      + `Content-Type` + `X-LJ-Api` on API POSTs — and `Sec-Fetch-Mode: cors` only on a CORS
      preflight; WebKit (macOS, Linux alike) passes `Accept`, `User-Agent`, a `Referer` on POST
      only, no `Origin` from the panel itself. So the `Sec-Fetch-Dest` / `-Site` checks never fire
      on our engines (kept for one that sends them); whether WebKit honours `frame-ancestors` /
      XFO for `ljext:` can't be watched from the log, since its guard refuses frame navigations
      before any request. Landmines has it.
  - **Fix 6 (`lj-step-29-fix6`) — what the header log showed on Windows.** In the race, three of
    the page's iframe requests for hello's `index.html` arrived **between the guard admitting the
    app's panel URL and the panel's commit** (~60 ms) and were **served** (status 200): the page
    check passed (the guard already said "hello"), and with no `Sec-Fetch-Dest` there was nothing
    to refuse a frame. `X-Frame-Options` / `frame-ancestors` and the API's Origin + token checks
    kept them from doing anything (the web page couldn't script them or call the API), but files
    — the SDK with the token too — could reach a web page in that moment. **Fix:** until a load
    event committed the panel (`NavGate::committed`), only its document is served — the request
    whose query is exactly `lj-open=<this opening's nonce>` (`session::nonce`, constant-time
    compare); every other file, the SDK and the API are `Uncommitted` and the handler holds them
    (10 ms polls, up to `api::COMMIT_WAIT` = 2 s) until the panel committed — by then the web page
    and its frames are gone, so a racer that waited is answered into nothing — or refuses them.
    And a Referer, when sent, must be a page of the extension's own origin (`api::referer_ok`;
    twice: refused), which turns the Windows racers away at once. Only the page's `Started` event
    commits now (the engines' commit callbacks), no longer `Finished`, which can arrive late and
    on WebKit carries the URL of a navigation that is still provisional (WKWebView's `URL`,
    WebKitGTK's `uri` change when a load starts). The panel's own subresources
    came after its commit event in every CI log, by 1–50 ms, so the wait is a safety net, not the
    usual path. The request log gained `status=` and `referer=own|other|-|twice`, and the race
    rule `servedToOthers` fails on any `status=200 referer=other`. Tests:
    `before_the_panel_commits_only_its_document_is_served` (all three platforms: the nonce
    document served; no / empty / wrong / upper-case / extended / extra-parameter / fragment
    nonces, other files, the SDK, the API all `Uncommitted`, then served once committed; no
    opening, no nonce; a web page shown → `page`), `a_referer_must_be_the_extensions_own` (12
    foreign forms incl. look-alike hosts and the other platform's form, on files, the SDK, the API
    and the nonce document; twice), smoke `servedToOthers` (vitest). Also seen: fix4's CI run had
    one Windows smoke failure in the race — hello's document loaded but none of its
    subresources (unstyled, no script, broken icon; no refusal logged — fix4 logged refusals
    only, so whether they reached the handler is unknown) while a racer's `window.open` landed in
    the same moment; fix5's run was green.
  - **Fix 7 (`lj-step-29-fix7`) — opening from a web page leaves it first.** Fix 6's CI run
    (39098950325) failed the Windows race again, with the full request log this time: the app's
    document request was answered 200, the racers were refused (`referer`), yet the page committed
    **WebView2's own "HTTP ERROR 404" page** for the panel's nonce URL — a response the handler
    never sent — and no subresource request followed. In both failing runs (fix4, fix6) the race
    page's `window.open` landed between `extension-opened` and the panel's commit (16 and 91 ms
    after); in the passing run (fix5) none did. So a web page's popup request while the panel
    loads breaks that load in WebView2 (the engine's own behaviour; the mechanism isn't visible
    from here — wry answers `NewWindowRequested` through a deferral). **Fix:** when the page
    shows a web page, `session::open` navigates it to the app's start page first (what it shows
    when nothing is open, whose load events CI has seen on every engine — not so for
    `about:blank` on WebView2) and opens the panel (expect + navigate) only when that page
    committed (`session::page_committed`, from a thread of its own, not inside the engine's
    event) or after `LEAVE_FOR` = 1 s — so normally no web page, nor its frames, subresources or
    popups, is alive while a panel loads. The detour is **best effort, for availability**: after
    1 s the panel opens anyway, and another navigation can supersede the detour. Security rests
    on the nonce and the commit rule (fixes 4, 6, 10), not on it. `gate::Leaving` numbers each wait
    so a stale timer can't open a later opening early; `close` and a new window drop a wait.
    Reload on a panel keeps the web page Close returns to (it lost it since fix 4). Opening from
    the start page, hibernation or another panel goes straight to the panel, as before. The start
    page is no web page anywhere (the bar shows the start state, history and back skip it). To
    keep files near 400 lines, the pure gate moved to `extensions/gate.rs` (`NavGate`,
    `page_may_navigate`, `Leaving`; re-exported by session.rs) with session's tests in
    `session/tests.rs`, and `openable` / `is_private_host` to `extensions/web_address.rs`
    (re-exported by api.rs). Test: `an_opening_waits_for_the_page_to_leave_its_web_page`; smoke:
    the race (now the app's opening must pass through the start page there, `raceFailures`).
    CI 39100400716: all 10 green — but no popup happened to land during the opening there (the
    race opened one on 10 % of its ticks).
  - **Fix 8 (`lj-step-29-fix8`) — a race that always overlaps.** The race page now opens a popup
    on every second tick (every 50 ms; `RACE.popupEvery`), so one lands while the app is leaving
    the page on Windows and macOS (the leave takes ~40–80 ms there; ~5 ms on Linux), and fix 7 is
    tested against the failure it fixes, not just run beside it. **CI 39101372328: all 10 green**;
    on Windows two popups and on macOS one landed between `extension-opened` and the start
    page's commit, and hello opened and reported each time; nothing was served to another page
    (`servedToOthers`). On Linux the race's popups never reach the app at all (WebKitGTK blocks a
    `window.open` without a user gesture; 0 `new-window-refused` in every run), so there the race
    is its iframes only.
  - **Fix 10 (`lj-step-29-fix10`) — the follow-up review of `aa14610` ("merge": M1 fixed, nothing
    usable leaks) asked for four last changes.**
    - **A new opening starts uncommitted.** When the guard admits a panel URL it now also clears
      `committed` (`gate::page_may_navigate`), so Reload of the same extension — whose old panel
      had committed — gets the "nonce document only" rule until its own commit too. And `commit`
      now wants **exactly the admitted URL** (`NavGate::admitted`, nonce included), so a late
      commit event of an earlier opening of the same extension can't commit the new one. Test:
      `a_new_opening_is_uncommitted_until_its_own_commit`.
    - **Service workers, end to end.** L4 rested on WebView2 passing `Service-Worker: script` to
      the handler, which nothing tested. Hello now ships a real `sw.js` and calls
      `navigator.serviceWorker.register("sw.js")` from its panel (`checkServiceWorker`), reporting
      `sw=refused` (the registration failed — the handler refused the script, or the engine
      couldn't fetch it), `sw=unsupported` (no `navigator.serviceWorker`, or a `SecurityError` /
      `NotSupportedError`), or `sw=registered` (unregistered at once). The smoke requires
      `refused` or `unsupported` on every OS (`isHelloReport`) and fails on any `200` for
      `/sw.js` in the request log (`serviceWorkerFailures`), whatever the page reports; hello's
      own expected `ljext-refused … /sw.js` doesn't count as a refused request of its panel. What
      each engine did is in the CI notes of the PR and the report of this round. The backstop —
      clearing `ljext` service-worker registrations on reload / switch-off — needs new crates, so
      it is a Deferred follow-up.
    - **Docs:** the start-page detour is described as best effort, for availability (the panel
      opens after 1 s anyway; a navigation can supersede it) — security rests on the nonce and
      the commit rule; this plan's M1 entry says commit is the `Started` event only (since fix 6)
      and the exact admitted URL (since fix 10); the PR description gives the real body limit,
      `MAX_BODY_BYTES` = the 1 MB storage quota + 64 KB.
    - **Deferred follow-ups** gained: the private-host filter is best effort (`openBrowser` is the
      real control), dot look-alikes in names (the badge covers them), no cap on concurrent file
      GETs plus wry's body buffering, and the service-worker backstop.

### LJ30 — Extensions in the command bar and Settings; docs · strong · auto

**Verify:** `npm run verify` green, including vitest tests:
- extension commands match by title and keywords and render as text with the extension's name;
- page actions show only while a web page is open (and say "for this page");
- disabled extensions don't show.

Settings → Extensions lists Built-in, Bundled and Developer extensions, with an
enable/disable switch for each (settings `disabledExtensions`) and details: version, author, and
permissions in plain words. It has a Developer mode switch with a warning, "Open extensions
folder" (revealed with the opener) and "Reload extensions". `docs/extensions.md` exists, marked
experimental, and the README links it. CI green; screenshot `settings-extensions.png`, plus
`command-hello.png` (typing "hello": xdotool / SendKeys).
**Commit:** `lj-step-30: extensions in the command bar and Settings; developer mode; extension docs`

```text
1. Commands: `extensions_list()` (bar and settings capabilities) returns the enabled
   extensions' commands and page actions. The command bar's commands hook from LJ23 matches
   them. Picking a page action opens its panel with the current page's URL and title (when it
   has pageInfo).
2. Settings → Extensions tab, as in Verify. settings.rs gains `disabledExtensions: string[]` and
   `developerMode: bool` (from LJ29), parsed tolerantly.
3. docs/extensions.md (experimental):
   - what an extension is and isn't;
   - a quick start (copy extensions/hello into the extensions folder, turn on Developer mode,
     run `little-joan --extension hello`);
   - the manifest reference;
   - the permissions;
   - the SDK/API reference with examples;
   - the CSP and what is blocked;
   - storage limits;
   - testing;
   - the security model (why there is no IPC, and what the checks are);
   - how bundled extensions get into the repository (a PR).
   The README links it under "Extensions (experimental)".
4. extensions/hello/README.md explains each file, as the template.

Files: ui/command/ (+ tests), ui/settings/ (extensions view and logic, + tests),
src-tauri/src/{extensions/registry.rs, commands.rs, settings.rs}, build.rs, capabilities/,
docs/extensions.md (new), extensions/hello/README.md, README, CHANGELOG, this plan.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ30 to ✅, add notes, and
commit: "lj-step-30: extensions in the command bar and Settings; developer mode; extension docs".
Push and watch CI until every job is green (two honest attempts, then stop). Look at both
screenshots.
```

**Implementation notes (2026-10-18):**

- **Commands:** `extensions_list()` (bar and settings) returns every loaded extension —
  `{ id, name, version, description, author, kind, enabled, icon, permissions, commands,
  pageActions }` — with `developerMode`, the folder and what was skipped (`extensions/list.rs`,
  1 test); the bar keeps the switched-on ones. Beyond the prompt, because the command bar must
  open what it lists and Settings needs its buttons: `open_extension(id, command, pageAction)`
  (bar only; a page action needs an open web page, and its panel gets the page's address and
  window title only when the manifest asks for `pageInfo`), `reload_extensions()` and
  `open_extensions_folder()` (Settings only; the folder is created, then revealed with the
  opener's `reveal_item_in_dir`). `session::open` takes a `Target` (first command, a command, a
  page action with its page).
- **Switched off** (`disabledExtensions`, settings.rs: valid ids and `builtin:<name>`, each once,
  at most 256; 1 test): the registry marks them, `Registry::get` (opening, serving, the API)
  never returns one, `find` / `all` (Settings) do (tests in registry and api). Saving settings
  forgets the registry so the next use reads the new switches; closing Settings forgets it too
  when no page window is open (nothing at rest). `save_settings` takes optional
  `disabledExtensions` and `developerMode`.
- **The command bar** (`ui/command/extensions.ts`, 6 tests): `extensions_list` once each time the
  command bar opens; a command matches when every typed word starts a word of its title, its
  keywords or its extension's name; rows show "Title — Extension" with the extension's icon (a
  checked `data:image/…;base64` URI drawn over a four-tile symbol, `row-icon-extension`); page
  actions only while the bar's address is http(s), as "Extension, for this page". The drawn-text
  test renders a title with markup through `view.ts` into a stand-in document that refuses
  `innerHTML`.
- **Settings → Extensions** (`ui/settings/extensions-logic.ts`, 4 tests; `extensions.ts`,
  `extensions.css`): "Experimental" badge and a link to the guide; groups Built-in (hidden until
  LJ31 fills it), Bundled, Developer — icon, name, version · author, description, permissions in
  plain words, a switch; Developer mode with its warning, the folder, the two buttons, a note when
  the group is empty, and each skipped folder with its reason. Switches and Developer mode are a
  draft that Save covers (`settings-logic.ts`: `toInput` gained an optional 7th argument and
  `isChanged` compares the ids in any order; 1 test).
- **Docs:** `docs/extensions.md` (experimental: what an extension is and isn't, quick start with
  the folder per OS — the copy needs a new id, since unpacked extensions can't take a bundled
  one's —, the manifest reference, permissions, the SDK with examples, panels and origins, files,
  the CSP, storage, testing, the security model, getting a bundled extension in); README
  "Extensions (experimental)"; `extensions/hello/README.md` (each file, as the template — it is
  embedded and served as text like any `.md`).
- **Smoke:** the history-off run turns Developer mode on with a copy of hello as `dev-hello` and a
  manifest with `apiVersion: 2` in its extensions folder; `--settings extensions` must list two
  and skip one (`extensions-list 2 skipped=1`, `extension-skipped broken …`) →
  `settings-extensions.png`. The extension run, after `--extension hello`: hide, summon, type
  `hello` → `command-hello.png`, then ↓ ↓ Enter must open hello a second time (Windows, Linux;
  best effort on macOS) — a new `pressKeys` helper (xdotool, SendKeys, CGEvent key codes). Rules in
  `smoke-extensions.mts` (`settingsExtensionsFailures`, `commandHelloFailures`; 3 more tests).
- **LJ29's CI, observed on the fixed run, recorded for later:** on Windows a navigation the guard
  cancelled still sent its request to the handler (`navigation-refused http://ljext.hello…` then
  `ljext-refused page … index.html`) — the page check refused it — and the cancelled navigation
  raised one more `Finished` for the page that stayed (a second `page-loaded` and history visit).
  Landmines has both.
- `npm run verify`: 244 TS tests, 234 Rust tests, clippy clean. Disk: 13 GiB free at the start and
  after verify; `.cache/target` 1.3 GB.
- **CI (run 39091695597, LJ30 with `lj-step-29-fix2`): verify ×3 and package ×3 green; all four
  smoke jobs red on one LJ30 rule**: "Settings → Extensions didn't list hello and dev-hello". The
  app was right — `extensions-list 2 skipped=1` and `extension-skipped broken \`apiVersion\` must be 1`
  were logged — but when the settings window first loaded (on General), since the tab asks once,
  not when `--settings extensions` switched to it. **Fix (`lj-step-30-fix`):** the rule looks from
  the settings window's first event on (a test reproduces CI's order; the command bar's own list
  before Settings opened doesn't count). Everything else held: hello `storage=ok theme=ok
  ipc=denied net=blocked` and the probe refused on all four; picking "Say hello" in the command
  bar opened hello again on Windows and Linux (macOS's synthetic typing didn't arrive this run —
  best effort, as before). Screenshots: `settings-extensions.png` — "Extensions" with the
  Experimental badge, the link, Bundled: Hello (icon, "Version 1.0.0 · by Little Joan", its
  description, "· Keep its own data on this computer (up to 1 MB)", switch on), Developer: the
  switch on, the warning in italics, the run's folder, both buttons, then "Hello (unpacked)" under
  the fold — alike on all four, crisp at scale 2; `command-hello.png` (Windows, Linux) — "hello"
  typed, "Search Google for “hello”", then "Say hello — Hello" with hello's icon, then Settings;
  on macOS the empty command bar (typing didn't arrive); `extension-hello.png` — the toolbar with
  hello's icon and "Hello" (back and open-in-browser greyed), the themed panel listing ok / ok /
  denied / blocked — except on Windows, see LJ29's notes (`lj-step-29-fix3`).

### LJ31 — Built-in quick answers · standard · auto

**Verify:** `npm run verify` green, including vitest tests for `ui/answers/`:
- **Calculator:** a real parser, never `eval` / `Function`. It handles + − × ÷ ^ %, parentheses,
  decimals, `pi`, `e`, `sqrt`, `sin`, `cos`, `log`, division by zero, and precision.
- **Units:** length, mass, temperature, volume, speed and data sizes (`10 km in mi`,
  `72f to c`).
- **Time zones:** `time in Tokyo`, `3pm london to new york`. They use Intl with an injected
  clock and a map of about 150 cities to IANA zones.
- **Random:** `uuid`, `password 20`, `roll 2d6`, `flip a coin`, `random 1-100`.
- **Timestamps:** `1700000000`, `now unix`.

The answer row comes first and Enter copies the answer. Each built-in can be switched off in
Settings → Extensions → Built-in. CI green; screenshot `answer.png` (typing `= 12*7`).
**Commit:** `lj-step-31: built-in quick answers — calculator, units, time zones, random, timestamps`

```text
Quick answers are our own code in the bar (decisions: they stay built-in; third-party
providers are Later). Fill LJ23's quick-answer hook with ui/answers/{calc,units,time,random,
timestamp}.ts. Each module exports a pure `answer(query, now) -> {title, subtitle, copy} | null`
plus tests.
- Copying uses navigator.clipboard.writeText from the trusted bar on the Enter keypress, falling
  back to a hidden textarea and execCommand; show "Copied" briefly.
- They are listed as Built-in in Settings → Extensions, switched off through
  `disabledExtensions` with ids `builtin:calc` and so on.

Files: ui/answers/ (new, + tests), ui/command/rows.ts, ui/settings/ (built-in list),
scripts/ci/smoke.mts (screenshot), README (examples), docs/extensions.md (built-ins), CHANGELOG,
this plan.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ31 to ✅, add notes, and
commit: "lj-step-31: built-in quick answers — calculator, units, time zones, random, timestamps".
Push and watch CI until every job is green (two honest attempts, then stop). Look at answer.png.
This ends phase 16:
- Fill the PR description. The security model is the review focus: list the checks and where
  each one is tested.
- Merge into main once green (merge commit), then stop and report.
```

**Implementation notes (2026-10-18):**

- **`ui/answers/`** — each module a pure `answer(query, …) → { title, subtitle, copy } | null`
  with its own tests (27 in all):
  - `calc.ts`: a tokenizer and a precedence-climbing parser (never `eval` / `Function`): + − × ÷
    (also `x`, `−`), `^` (right-associative, tighter than a unary minus: `-2^2 = -4`), `%` as a
    percent after an operand (`50% * 30 = 15`) and modulo between two (`10 % 3 = 1`), parentheses,
    implicit multiplication (`2(3+4)`, `2pi`), decimals and exponents, `pi`, `e`, `tau`, `sqrt`,
    `cbrt`, `abs`, `sin`/`cos`/`tan` (and inverses), `log` (base 10), `ln`, `exp`,
    `round`/`floor`/`ceil`. Division by zero, unknown names, unbalanced parentheses and
    non-finite results have no answer. Results are written with 12 significant digits
    (`0.1 + 0.2 = 0.3`), exponents past 1e15. `= …` always asks it; bare text only with a digit
    and an operator or a function — not a lone number (timestamps), not `2024-10-18`, not an IP.
  - `units.ts`: length, mass, temperature, volume (US cups / pints / quarts / gallons), speed,
    data sizes (decimal kB… and binary KiB…; `b` is bytes, bits aren't converted), by symbol or
    name, `in` / `to` / `as` / `into`, 10 significant digits.
  - `time.ts` + `cities.ts` (203 names: ~170 cities, a few countries' capitals and zone
    abbreviations → IANA; every zone checked against Intl in a test): `time in <city>` /
    `<city> time` / `what time is it in …`, and `<3pm | 15:30 | noon | midnight> <city> to|in
    <city>` (a bare `3` is too vague). Offsets come from `Intl.DateTimeFormat#formatToParts`
    with a passed-in clock and locale; tested across the 2026 October week when Europe has left
    summer time and the US hasn't. `Europe/Kiev`, not `Kyiv`, for older ICU data.
  - `random.ts`: `uuid` (v4), `password [8–128]` (default 16; letters, digits, symbols, no
    look-alikes, at least one of each kind, shuffled), `[roll] NdM` (≤ 20 dice), `flip a coin`,
    `random [a-b]` (default 1–100). The randomness is passed in; in the bar it is
    `crypto.getRandomValues` with rejection sampling (`cryptoInt`), never `Math.random`.
  - `timestamp.ts`: 10 digits = seconds, 13 = milliseconds → UTC time and ISO 8601 (copied);
    `now unix` / `unix now` / `timestamp` / `epoch`.
  - `index.ts`: `BUILT_INS` (ids `builtin:calc`, `builtin:units`, `builtin:time`,
    `builtin:random`, `builtin:timestamp`), asked in that order; the first answer becomes the row.
- **The command bar:** the quick-answer hook from LJ23 is filled (`command.ts`), the answer row
  comes first **and is selected** (`rows.ts` `initialSelection`, tested), so Enter copies it;
  a click copies too. `copy.ts` uses `navigator.clipboard.writeText` in the trusted bar on that
  keypress, falling back to a hidden textarea and `execCommand("copy")`, then gives the field the
  keyboard back; the row's subtitle says "Copied" (or "Couldn't copy") for 1.2 s, as text. An
  equals-sign symbol (`row-icon-answer`).
- **Switching off:** Settings → Extensions → Built-in lists the five with their examples and a
  switch each (the `disabledExtensions` draft from LJ30); `extensions_list` now also returns
  `disabledBuiltIns` (the `builtin:` entries), which the command bar skips.
- **Smoke:** in the extension run, after the command-bar hello, a third window: type `= 12*7` →
  `answer.png`, Enter, and the system clipboard is read back (`Get-Clipboard`, `pbpaste`, `xclip`
  / `xsel` if the runner has one) into `summary.json` — informational, not a pass/fail rule.
- Files beyond the prompt: `ui/command/copy.ts`, `ui/command/command.ts`, `ui/command/extensions.ts`
  (the view's type), `ui/settings.ts`, `ui/style.css`, `src-tauri/src/extensions/list.rs`,
  `src-tauri/src/commands.rs`, docs/architecture.md.
- `npm run verify`: 272 TS tests, 235 Rust tests, clippy clean. Disk: 13 GiB free at the start
  and after verify; `.cache/target` 1.3 GB.
- **CI (run 39092961418: LJ31 with `lj-step-30-fix` and `lj-step-29-fix3`): every job green** —
  verify ×3, package ×3, smoke on Windows, macOS, Linux and Linux at scale 2. `--extension hello`
  from hibernation now reports on its own on every OS with none of its requests refused (Windows
  too: only WebView2's own `favicon.ico` request is answered 404); the probe is refused as before;
  "Say hello" picked in the command bar opened hello again on Windows and Linux; Enter on the
  quick answer put **`84` on the Windows clipboard** (read back with `Get-Clipboard`; the Linux
  runners have no clipboard tool; macOS's typing didn't arrive). Screenshots: `answer.png`
  (Windows, Linux, scale 2) — "= 12*7" typed, the first row selected (accent fill, white text):
  an equals sign, "84 — = 12*7", then "Search Google for “= 12*7”" and Settings; on macOS the
  command bar without the typing — and, a finding, with "YouTube — m.youtube.com" as a top site:
  the earlier ↓ ↓ Enter had arrived although the typing hadn't, and opened a starter. **Fix
  (`lj-step-31-fix`):** the smoke presses keys only after its typing showed up in the log.
  `settings-extensions.png` (all four) now starts with **Built-in**: Calculator, Unit conversion,
  Time zones, Random, Unix timestamps, each "Built in" with its examples and a switch — but with an
  empty white tile where an icon goes; the same fix gives icon-less rows a symbol ("=" for the
  built-ins, the four tiles for an extension without an icon). `extension-hello.png` on Windows:
  the themed Hello panel (the race is gone). `command-hello.png`, `extension-probe.png`: as in
  LJ30's notes.

### LJ32 — Bundled games: Snake and 2048 · standard · auto

> Phase 17. Branch `lj/p17-bundled-extensions` from `main`.

**Verify:** `npm run verify` green. Extensions are plain JavaScript, with no build step, so
authors can copy them. vitest includes `extensions/**/*.test.ts`, which import each game's pure
`logic.js`:
- **Snake:** movement on a grid, growth, wall and self collision, no 180° reversal, speed-up,
  pause on Space, arrows and WASD.
- **2048:** one merge per tile per move, spawning, game over, best score.

High scores go through `littleJoan.storage`, and colors come from `littleJoan.theme`. CI green;
screenshots `snake.png` and `2048.png` via `--extension snake` / `--extension 2048`.
**Commit:** `lj-step-32: bundled games — Snake and 2048`

```text
Two small games as bundled extensions in extensions/snake/ and extensions/2048/. Each has
extension.json (a command "Play Snake" / "Play 2048", permission storage), index.html, logic.js
(pure, exported), ui.js (canvas or DOM, themed), an icon.svg made from original simple
geometry, and a test file. Keyboard only; responsive to the panel size, including phone width;
no network.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ32 to ✅, add notes, and
commit: "lj-step-32: bundled games — Snake and 2048". Push, open the phase-17 PR into main as a
draft, and watch CI until every job is green (two honest attempts, then stop). Look at both
screenshots.
```

**Implementation notes (2026-10-18):**

- **One layout for every bundled extension after hello** (documented in architecture.md and
  extensions.md): `index.html` loads the SDK (a classic script) and then `<script type="module"
  src="ui.js">`; `ui.js` is the DOM glue and imports `./logic.js`, the pure rules (an ES module,
  JSDoc-typed, `// @ts-check`), which `logic.test.ts` imports too; `style.css` paints with the
  theme's `--lj-*` variables (system-color fallbacks until `littleJoan.theme.apply()` answers — at
  first the body was hidden until then, which `lj-step-32-fix` undid); `icon.svg` is original
  geometry. Module scripts
  over `ljext:` work on every engine the way the app's own Vite pages do over `tauri:` (same-origin
  CORS-mode fetches; WebKitGTK registers both schemes alike, as secure). vitest now includes
  `extensions/**/*.test.ts`. Esc closes a panel (`littleJoan.close()`) — in Snake it first pauses a
  running game; Ctrl/Alt/Cmd presses are left to Little Joan and the system.
- **Snake** (`extensions/snake/`, permission `storage`): an 18 × 18 grid on a canvas sized to the
  largest whole-cell square that fits (sharp at any device pixel ratio, down to phone width). Rules
  (`logic.js`, 13 tests): movement one cell per step, growth on food, walls and the body end it (the
  cell the tail leaves is free unless it grows), no 180° reversal — also not through two quick
  presses, since a turn is compared with the last *waiting* one (up to two wait, so a U-turn in two
  steps works) —, food on a random free cell (none left = a win), 140 ms per step minus 4 per food,
  never under 60 ms; Space pauses / resumes / starts, a turn resumes, Enter starts again; arrows and
  WASD by `KeyboardEvent.code` (the same place on AZERTY). The snake is the accent, fading toward
  `--lj-surface` at the tail; the food is the accent turned 180° and a little more saturated; eyes
  look where it goes next. The loop runs on `requestAnimationFrame` only while a game runs; it
  pauses when the panel loses focus. Best score in storage.
- **2048** (`extensions/2048/`, permission `storage`): DOM tiles keyed by id so they slide (CSS
  transforms, 110 ms; the merged-away tile slides under its partner and goes, the doubled one pops,
  the new one appears; none of it with reduced motion). Rules (`logic.js`, 11 tests): lines packed
  toward the move's edge, one merge per tile per move (`[2,2,2,2] → [4,4]`, `[4,4,8] → [8,8]`), the
  tile nearer the edge survives a merge, a new 2 (a 4 one time in ten) only after a real move, game
  over with a full board and no equal neighbours, won at 2048 (Enter keeps playing), N for a new
  game. Tile colors come from the theme (`palette`): light mode from `--lj-surface` to
  `--lj-row-selected`, dark mode from a lifted surface to `--lj-accent` (so tiles always stand out
  from the empty cells), with whichever of `--lj-fg`, `--lj-on-selected` and `--lj-surface` reads
  best — tested at ≥ 3:1 (large bold text) for every tile up to 2^17 in both default palettes. The
  game in progress and the best score are saved 400 ms after the last move (storage takes 20 writes
  a second at most), and before Esc closes the panel; a saved game is checked from scratch when read
  (`restore`).
- **`extensions/bundled.test.ts`** checks every bundled extension (folder = id, icon and panels
  exist, the SDK first, no inline script or event handler, no network API or `network:` host, no
  `innerHTML` / `eval`, a pure `logic.js` with tests); the Rust test
  `the_shipped_extensions_are_all_valid` now also wants every bundled extension's icon and panels
  served and no network host, and Snake and 2048 present.
- **Smoke** (`scripts/ci/smoke-bundled.mts`, 5 tests): a launch of its own, `--extension snake` then
  `--extension 2048`; each panel must come up — `extension-opened`, its URL with the nonce in
  `page-started`, its script's own `ext-api <id> theme ok`, `toolbar-label <id>`, none of its
  requests refused (the engine's favicon aside), no failed call — then `snake.png` / `2048.png`.
  2048 gets ← ↑ → ↓ first; whether they reached the panel (its own save) is recorded in
  `summary.json`, informational. `pressKeys` learned the arrows (SendKeys, xdotool, macOS key codes).
  Settings → Extensions' rule now counts the bundled extensions (`extensions-list <bundled + 1>
  skipped=1`).
- Files beyond the prompt: `vitest.config.mts`, `extensions/bundled.test.ts`,
  `scripts/ci/smoke-bundled.mts` (+ test), `scripts/ci/smoke.mts`, `scripts/ci/smoke-extensions.mts`
  (+ test), `src-tauri/src/extensions/registry/tests.rs`, README (a table of the bundled
  extensions), CHANGELOG, architecture.md, extensions.md. Checked locally in a plain browser with a
  stand-in SDK (a dev aid, not committed): both games at 480 × 600 and 360 × 560, light and dark.
- `npm run verify`: 324 TS tests, 256 Rust tests, clippy clean. Disk: 15 GiB free at the start and
  after verify; `.cache/target` 1.3 GB.
- **CI (run 39118669089): verify ×3, package ×3 and smoke on Windows, macOS and Linux green;
  `smoke (linux-scale-2)` red on an older rule** — "the command bar never asked for suggestions
  before the smoke page opened": that runner's first window took longer than the smoke's fixed
  1.5 s to ask (later windows asked within ~220 ms). Both games came up on all four jobs, and the
  arrow keys reached 2048 on Windows and both Linux jobs (its own save; not on macOS, where
  synthetic keys are best effort). Screenshots: Windows and Linux — the themed panels with the
  toolbar's icon, name and "Extension" badge; Snake's board with the snake, the food and the start
  card; 2048 after four moves (score 12 / 8). But **Snake, the first panel of the run (opened from
  hibernation), was a blank white page on macOS and Linux at scale 2** (its script had run: theme
  and storage calls were answered), and on macOS the toolbar showed an empty field — hello's
  screenshot in the extension run had that empty toolbar on macOS too. Two causes: (1) the panels
  hid `<body>` (opacity 0) until the theme answered, and WebKit doesn't commit a first frame for a
  page with nothing visible, so a page that becomes visible a moment later may stay unpainted;
  (2) on macOS, when the panel commits before the bar's own page has loaded (the window built for
  `--extension` from hibernation), the toolbar can stay unpainted — the app logged `toolbar-label`
  for it twice, so it is the bar's view, not the label (Deferred follow-ups). **Fix
  (`lj-step-32-fix`):** no more hidden body (the fallback colors show for at most a frame); the
  smoke waits for the first window's suggestions instead of a fixed pause, and the bundled run
  summons the window before opening panels (hello's run keeps covering hibernation), with 1.5 s
  before each screenshot. Its run (39120155218) had verify ×3 and package on macOS and Linux green,
  but `package (windows)` hung in `actions/cache` restoring for more than 15 minutes (a GitHub
  cache stall; the job has no timeout), so the LJ33 push superseded it — LJ33's run checks the fix.

### LJ33 — Bundled productivity: Notes, Focus timer, World clock · standard · auto

**Verify:** `npm run verify` green, including vitest tests for each `logic.js`:
- **Notes:** several notes, autosave with a debounce, search, copy a note.
- **Focus timer:** 25 / 5 / 15 cycles, pause and resume. The end time is stored, so reopening
  shows the time left; there are no background alarms while Little Joan hibernates, and the
  panel says so.
- **World clock:** add and remove cities, correct across DST with an injected clock.

CI green; screenshots `notes.png`, `focus.png`, `clock.png` via `--extension`.
**Commit:** `lj-step-33: bundled productivity — Notes, Focus timer, World clock`

```text
Three bundled extensions in extensions/notes/, extensions/focus/ and extensions/clock/, with
the same layout as LJ32. All local, using storage only (Notes: clipboard too). The focus timer
plays a short WebAudio chime while the panel is open.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ33 to ✅, add notes, and
commit: "lj-step-33: bundled productivity — Notes, Focus timer, World clock". Push and watch CI
until every job is green (two honest attempts, then stop). Look at the screenshots.
```

**Implementation notes (2026-10-18):**

- Same layout as LJ32 (`index.html`, `ui.js`, pure `logic.js` + `logic.test.ts`, `style.css`,
  original `icon.svg`); each panel is keyboard-first, fits phone width and takes the theme.
- **Notes** (`extensions/notes/`, permissions `storage`, `clipboard`; 10 tests): several notes
  (`{ id, text, created, updated }`, newest edit first, at most 500, 200,000 characters each), a
  title from the first line (Markdown `#` dropped) and a preview from the rest, search where every
  word must appear (any order, case and accent: `fold`), autosave with a debounce
  (`autosave(save, timers, 500 ms)`: one save per burst with the latest text, `flush()` on leaving
  the editor, Esc, blur and `pagehide`, a failed save stays waiting — tested with fake timers),
  copy through `littleJoan.clipboard.writeText`, delete with a 6-second Undo (also Ctrl/Cmd+Z), an
  empty note isn't kept, a "Saved" / quota message, and how much of the 1 MB the notes use past
  75 %. One pane at a time in a narrow window (list ↔ editor), side by side from 640 px. The list
  takes the keys (N new, `/` search, ↑ ↓, Enter, Delete); what storage holds is checked from
  scratch (`parseNotes`).
- **Focus timer** (`extensions/focus/`, permission `storage`; 9 tests): focus 25, short break 5,
  long break 15 after the fourth round; Space starts / pauses / resumes, R resets the phase, S skips
  (a skipped focus session doesn't count). Storage keeps the **end time** of a running phase, so a
  reopened panel shows the time left; a phase that ran out while the panel was closed is reported
  ("Focus ended at 10:25, while this panel was closed.") and the next phase waits — only one phase
  ends per tick, nothing runs on by itself (`restore`, `tick`). The panel says that Little Joan has
  no background alarms and the chime plays only while it is open. The chime is Web Audio (a rising
  triad after focus, a falling pair after a break; an `AudioContext` made on the first Start, since
  engines allow audio only after a key press or click). A ring (SVG stroke) empties as the phase
  runs; the window title counts down while it runs (`17:29 · Focus`).
- **World clock** (`extensions/clock/`, permission `storage`; 10 tests): `cities.js`, 177 cities
  (name, country, IANA zone — Little Joan's own list, the zones the quick answers use; every zone
  checked against Intl in a test); your time (zone, UTC offset, date) and the cities you add —
  each with its time in your locale's style, the difference ("+9 h", "−4 h", "+5 h 30 min"), the
  day there (Today / Tomorrow / Yesterday) and a sun or moon. Intl does the daylight-saving work
  (`formatToParts` with a passed-in instant): tests stand on both sides of 2026's European (25 Oct)
  and US (1 Nov) changes, Sydney's start (4 Oct), half- and quarter-hour zones, the hour that
  repeats in London, and a month's end. Add in the combobox (type, ↑ ↓, Enter), Delete removes a
  city, Alt+↑ ↓ moves it; the first list (nothing saved yet) is London, New York, Tokyo, Sydney
  minus your own zone.
- **Smoke** (`smoke-bundled.mts`, 7 tests now): the bundled run's data folder starts with sample
  content in `extension-data/` (three notes, a focus session 7½ minutes in, four cities —
  `seedFile` writes Little Joan's own storage format), and opens Notes, Focus timer and World clock
  after the games (`notes.png`, `focus.png`, `clock.png`); the focus timer must show the session
  running from its stored end time — its title `1x:xx · Focus` (`title` rule).
- Checked locally in a plain browser with a stand-in SDK (not committed): all three at 480 × 600
  and Notes at 760 px (two panes), Notes' keys and search, the timer running and its "ended while
  closed" message after a reload, adding a city with the keyboard.
- Files beyond the prompt: `extensions/bundled.test.ts`, `scripts/ci/smoke-bundled.mts` (+ test),
  `scripts/ci/smoke.mts`, `src-tauri/src/extensions/registry/tests.rs`, README, CHANGELOG,
  architecture.md.
- `npm run verify`: 370 TS tests, 256 Rust tests, clippy clean. Disk: 15 GiB free at the start and
  after verify; `.cache/target` 1.3 GB.
- **CI (run 39121642833, with `lj-step-32-fix`): all 10 jobs green** — verify ×3, package ×3, smoke
  on Windows, macOS, Linux and Linux at scale 2; the first window's suggestions now arrive before
  the smoke page on every job. All five panels came up on all four smoke jobs (the focus timer's
  title counted down from its stored end time: `17:2x · Focus`), and the arrow keys reached 2048 on
  Windows and both Linux jobs (not on macOS — see Deferred follow-ups). Screenshots, alike on every
  OS (crisp at scale 2): `snake.png` — the toolbar with Snake's icon, name and "Extension" badge
  (on macOS beside the traffic lights now that the panel opens in a summoned window), the board,
  the snake and the food, the start card above it; `2048.png` — after four moves on Windows and
  Linux, the first two tiles on macOS; `notes.png` — the three sample notes (Groceries selected,
  "4 min ago", "Yesterday", "Oct 9") with search and New note; `focus.png` — FOCUS, a ring a
  third empty, 17:13, "Ends at 11:41 AM", round 2 of 4 with one dot done, Pause, the note about
  background alarms; `clock.png` — your time (UTC on the runners, shown as "UTC · UTC", which
  LJ34 tidies to "UTC"), London +1 h, New York −4 h, Tokyo +9 h, Sydney +11 h (all "Today"), suns
  and a moon.

### LJ34 — Bundled page actions and tools · standard · auto

**Verify:** `npm run verify` green, including vitest tests:
- **QR code:** a QR encoder (byte mode, versions 1–10, error correction M), compared with
  reference matrices.
- **Clean link:** rules that strip `utm_*`, `fbclid`, `gclid`, `igshid`, `mc_eid`, `si`, `ref_src`
  and similar, while keeping functional parameters.
- **Markdown link:** escaping.
- **JSON:** pretty-print, minify, and validation with the error position.
- **Text tools:** counts, case, base64 and URL encode/decode, sorting lines.
- **Color tools:** hex / rgb / hsl / oklch conversions and contrast.

The page actions (QR code, Clean link, Wayback Machine, Copy as Markdown link) appear in the
command bar only while a web page is open, and receive it through pageInfo. Wayback opens
`https://web.archive.org/web/*/<url>` in the page. CI green; screenshots `qr.png` (on the smoke
page) and `json.png`.
**Commit:** `lj-step-34: bundled page actions and tools`

```text
Bundled extensions: extensions/qr/, extensions/clean-link/, extensions/wayback/ and
extensions/markdown-link/ (page actions with pageInfo; clean-link and markdown-link also use
clipboard), plus extensions/json/, extensions/text-tools/ and extensions/color/ (commands). Same
layout as LJ32. All local; nothing fetches the network (Wayback only opens a page). Write the
clean-link rules from scratch (don't copy ClearURLs' LGPL rules), with tests.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ34 to ✅, add notes, and
commit: "lj-step-34: bundled page actions and tools". Push and watch CI until every job is green
(two honest attempts, then stop). Look at the screenshots.
```

**Implementation notes (2026-10-18):**

- Seven bundled extensions in LJ32's layout. Page actions (`pageInfo`): **QR code** (`qr`),
  **Clean link** (`clean-link`, + `clipboard`), **Wayback Machine** (`wayback`), **Copy as
  Markdown link** (`markdown-link`, + `clipboard`); commands: **JSON** (`json`), **Text tools**
  (`text-tools`), **Color tools** (`color`) — the three with `storage` (drafts, last colors) and
  `clipboard`. None declares a `network:` host, and none asks for `openBrowser` (a Rust test checks
  both, and that the four page actions are page actions with `pageInfo`); `extensions/bundled.test.ts`
  finds no network API in any of them.
- **QR code** (7 tests): an encoder written from the standard — byte mode (UTF-8), versions 1–10,
  level M: the data codewords (mode, count, terminator, 0xEC / 0x11 padding), Reed–Solomon over
  GF(256) (generator per block size), the block split and interleaving, finders, separators,
  timing, alignment patterns, the dark module, format bits (BCH, XOR 0x5412) and version bits
  (v7+), the zigzag placement, the eight masks and the standard's penalty (runs, 2 × 2 blocks,
  finder-like patterns, dark balance). **Reference matrices:** 13 texts covering versions 1–10
  (UTF-8 too) were encoded locally with macOS Core Image's `CIQRCodeGenerator` at level M — an
  independent encoder — and stored as `reference.test-data.json` (not embedded: build.rs leaves
  `*.test.*` out); 11 match ours module for module with the same mask, and the 2 where Core Image
  picked another mask (penalty heuristics differ between encoders) match exactly with its mask.
  Every code ours made for them also decoded with Core Image's `CIDetector` (a local, one-off
  check with a small Swift tool kept out of the repository). Also tested: the standard's format
  and version bit tables, alignment positions, capacities, and two Reed–Solomon worked examples.
  The panel draws the code as one SVG path (black on white whatever the theme, so cameras read it),
  shows version and size, and lets you edit what it encodes; over 213 bytes it says so.
- **Clean link** (6 tests): rules written for Little Joan (not ClearURLs'): tracking names removed
  everywhere (`utm_*`, `pk_*`, `mtm_*`, `fbclid`, `gclid`, `dclid`, `gbraid`, `wbraid`, `msclkid`,
  `yclid`, `twclid`, `ttclid`, `mc_cid`, `mc_eid`, `_hsenc`, `mkt_tok`, `igshid`, `ref_src`, …) and
  names that are tracking only on their own sites (YouTube `si` / `feature` / `pp`, Spotify `si`,
  X `s` / `t`, Amazon `ref`, `pd_rd_*`, `pf_rd_*`, `tag`, `crid`, Google's `ved` / `ei` / `oq` /
  `sxsrf` …, Bing's `form` / `cvid`, LinkedIn's `trk`, Reddit's `share_id`, TikTok's share
  fields, eBay's `_trk*`, AliExpress's `spm` / `aff_*`, Etsy, Medium's `source`, NYT's `smid`);
  Amazon's `/ref=…` path segment; redirect wrappers unwrapped (Google `/url`, `l.facebook.com`,
  `l.instagram.com`, `out.reddit.com`, YouTube `/redirect`, Tumblr, Slack, Messenger, VK) — only
  into http(s), never for look-alike hosts. What stays keeps its exact spelling and order (raw
  pairs, not re-encoded); unknown parameters stay. The panel shows the clean link, what was
  removed (struck-through chips) and the original; Enter copies and returns to the page.
- **Wayback Machine** (4 tests): `https://web.archive.org/web/*/<url>` (every snapshot, the default)
  or `…/web/<url>` (newest), without user info or fragment; ↑ ↓ and Enter; opened with
  `littleJoan.openUrl` in the window — nothing is fetched before.
- **Copy as Markdown link** (7 tests): `[text](url)` with backslashes, brackets, `*`, `_`,
  backticks and angle brackets escaped in the text and spaces, parentheses and angle brackets
  percent-encoded in the address; also HTML (escaped),
  plain "Title — address" and Org mode; an editable title with an offer to drop a trailing site
  name ("… - YouTube"); ↑ ↓ format, Enter copies and returns to the page.
- **JSON** (9 tests): its own parser (RFC 8259, BOM allowed, depth ≤ 256) with messages and
  line / column that are the same on every engine (trailing commas, single quotes, unclosed
  strings, bad escapes, leading zeros, extra text…), a caret excerpt, and printers (2 / 4 spaces,
  tabs, minified; sort keys) that keep every number and string exactly as written (no rounding of
  `12345678901234567890`); highlighting with text nodes only; Ctrl/Cmd+Enter formats in place.
- **Text tools** (6 tests): counts (code points, words in any script, lines, UTF-8 bytes,
  sentences, reading time), nine cases (Title Case keeps small words lower inside), Base64 (UTF-8,
  URL-safe and unpadded input accepted, invalid input and non-UTF-8 bytes refused) and URL
  encoding, six line operations (natural sort with Intl.Collator); tools act on the selection or
  everything, with an Undo (also Ctrl/Cmd+Z right after a tool).
- **Color tools** (7 tests): hex (3/4/6/8), `rgb()` / `rgba()`, `hsl()` / `hsla()` (deg, turn,
  rad, grad), `oklch()` (percent forms; out-of-gamut colors lose chroma until they fit) and 23
  names; hex / RGB / HSL / OKLCH with a Copy each (OKLab with Björn Ottosson's matrices, checked
  against his reference values); WCAG contrast (see-through colors composited), the AA / AA large /
  AAA levels, ratios rounded down so 4.499 never reads as 4.5; the theme's colors as chips.
- **Smoke** (`smoke-bundled.mts`, 8 tests): JSON, Text tools and Color tools open after the
  productivity panels with sample drafts (`json.png`, `text-tools.png`, `color.png`); then the smoke
  page and, from the command bar, Ctrl/Cmd+L, `qr`, ↓ ↓ Enter → the QR code page action must come
  up and have `pageInfo` answered (`pageActionFailures`; required on Windows and Linux, best effort
  on macOS) → `qr.png`. `pressShortcut` learned L. **Typing on macOS** now posts real key codes
  (US layout; `scripts/ci/smoke-keys.mts`, 2 tests) instead of key events that carry only a
  Unicode string — those never reached the app on CI's macOS runners, while ↓ ↓ Enter, sent as
  key codes, did (LJ31's notes); the macOS rules stay best effort.
- Found while building, not fixed (Deferred follow-ups): Back after a panel opened a page (Wayback,
  Clean link's "Open it here") meets the panel's own entry in the engine's history. Also: the World
  clock's "your time" no longer says "UTC · UTC" when the zone is UTC itself (seen in LJ33's CI).
- Checked locally in a plain browser with a stand-in SDK (not committed): every panel at 480 × 600,
  the QR code against Core Image's decoder (above).
- Files beyond the prompt: `extensions/bundled.test.ts`, `scripts/ci/smoke-bundled.mts` (+ test),
  `scripts/ci/smoke.mts`, `src-tauri/src/extensions/registry/tests.rs`, README, CHANGELOG,
  architecture.md, extensions.md.
- `npm run verify`: 454 TS tests, 256 Rust tests, clippy clean. Disk: 14 GiB free at the start and
  after verify; `.cache/target` 1.3 GB.
- **CI (run 39123351961): verify ×3, package ×3 and smoke on macOS, Linux and Linux at scale 2
  green; `smoke (windows)` red on the new page-action rule** — "picking qr in the command bar never
  opened it": Ctrl+L opened the command bar over the smoke page (`overlay true`, its suggestions),
  but the typed "qr" never arrived. The smoke's typing activates the app's window first
  (`AppActivate`), and activating a window already in front evidently gave the keyboard back to
  the page webview the app had just moved it away from (typing into a command bar opened by Ctrl+L
  over a page hadn't been tried on Windows before; the other typing follows a fresh window). On
  both Linux jobs the QR code opened for the smoke page (`qr.png`: its title from `pageInfo`, the
  code, "Version 3 · 33 of 213 bytes"); on macOS the typing didn't arrive (best effort — though the
  main run's "127" did, with the new key-code typing). Every command panel came up on all four
  jobs, and the arrows reached 2048 everywhere, macOS included this time. Screenshots: `json.png`
  — the sample formatted with highlighted keys, "Valid JSON · 2 objects, 1 array, 9 values, 2
  levels deep", but the input showing the far end of its one long minified line; `text-tools.png`
  — the sample with "86 characters · 16 words · 2 lines · 86 bytes · 1 min read" and the three
  tool groups; `color.png` — #e0675a as HEX / RGB / HSL / OKLCH, 3.35:1 on white (AA large only),
  6.25:1 on black (AA), the sample text. **Fix (`lj-step-34-fix`):** the page-action flow types
  and presses keys without activating the window again (`typeText` / `pressKeys` take
  `activate`); JSON's input wraps long lines and starts at the top.

### LJ35 — Docs, screenshots, the user's checklist · standard · checkpoint

**Verify:** `npm run verify` green. The README's "Using it" covers the command bar, the toolbar,
history, themes and extensions, with screenshots in `docs/images/` taken from CI (the smoke page
only, so no personal data). CHANGELOG `[Unreleased]` lists everything from round 2.
`docs/architecture.md` and `docs/extensions.md` are current. CI green. **Checkpoint:** the
user ticks the checklist.
**Commit:** `lj-step-35: docs and screenshots for the command bar, themes and extensions`

```text
Bring the docs up to date with round 2, then hand the user a checklist.

Fill the PR description. Needs human check, on your Mac with the CI build of this PR (install
it as before):
- [ ] Summon: the command bar opens, like Little Arc's. After a day or two of use, the empty
      state shows the sites you open most, and typing finds them.
- [ ] The toolbar: domain, back, open in browser; the traffic lights sit inside it; you can drag
      the window by the toolbar.
- [ ] Logins: log in to Instagram; close and summon; quit and relaunch; reboot. You stay logged
      in.
- [ ] History: ⌘Y, search, delete one, clear the last hour.
- [ ] Themes: try three presets, then make your own with three colors; light, dark and system.
- [ ] Extensions: Snake, 2048, Notes, Focus timer, World clock; on a page, QR code, Clean link,
      Wayback; ⌘L then "= 12*7", "10 km in mi", "time in Tokyo".
- [ ] The welcome on a fresh profile still works; start at login still works.
- [ ] The must-not-disturb list.

Merge the PR once CI is green (docs only), then STOP for the checklist. A `v0.2.0-beta.2`
pre-release happens only on the user's word, the same way as beta.1: bump the versions through
a PR, merge, build on main, rename the installers, SHA256SUMS.txt, then
`gh release create v0.2.0-beta.2 --prerelease --latest=false --target <main commit>`.
```

**Implementation notes (2026-10-18):**

- **README:** the status line says what `main` adds since beta.1; "Using it" gained an Extensions
  bullet (how to open one, the twelve bundled, the badge, Esc); the command-line bullet says
  `--extension` opens a first command and page actions open from the command bar; the Extensions
  section lists every bundled extension and page action and says none reaches the network.
  **Screenshots** (`docs/images/`, 424 KB): six cropped from CI's macOS smoke run 39124661621 —
  the smoke test's page and its sample data only, nothing personal — `command-bar.png`,
  `snake.png` (under the intro), `appearance.png` and `theme-dark.png` (Settings → Appearance),
  `notes.png` and `focus.png` (Extensions). Cropped with ImageMagick (the window's bounding box,
  rounded transparent corners) and quantized to 256 colors with dithering (the grain makes
  full-color PNGs ~390 KB each).
- **The other docs:** CHANGELOG `[Unreleased]` already lists every round-2 change (command bar,
  toolbar, logins, history, themes, Appearance, Settings tabs, the extension runtime, the command
  bar's extensions, quick answers, and LJ32–LJ34's bundled extensions); architecture.md has the
  bundled extensions, the smoke's bundled run and `docs/images/`; extensions.md points at the
  bundled ones as worked examples; CONTRIBUTING.md says how a bundled extension is laid out and
  checked. SECURITY.md needed nothing new (no new permission or API).
- `npm run verify`: 454 TS tests, 256 Rust tests, clippy clean. Disk: 14 GiB free at the start and
  after verify; `.cache/target` 1.3 GB.

**The owner's checklist (2026-10-18), on their Mac with `main` at 5f3fa08:** "All works
amazingly". Logins survive, themes are "really good", QR code → Back returned to the page. One bug:
**YouTube's own fullscreen button** enters fullscreen and drops back to the normal size at once;
pressing F (YouTube's key) fullscreens fine, and then the player's exit button does nothing (Esc
exits). Recorded under Deferred follow-ups; nothing else failed. The owner then said to release
the current state: `v0.2.0-beta.2` (pre-release, like beta.1), released from the commit `main`'s
CI built. **Confirmed by the owner (2026-10-18, while round 3 was planned): it was the phone
version of YouTube — phone mode's iPhone agent, m.youtube.com — that failed; desktop YouTube
works.** LJ36 fixes it.

### Requested for the next version (2026-10-18) — NOT started; waits for the owner's yes

The owner's list after the checklist (their words, condensed). The owner said explicitly not to
start this work until they say yes; a proposal goes to them first, and it becomes plan steps only
then.

- Fix YouTube's fullscreen button on macOS (above).
- An option to hide the tray / menu-bar icon (some people prefer no icon); the shortcut, the
  command bar and a second launch still reach everything.
- The shortcut recorder warns when a shortcut clashes with a system (or another app's) shortcut.
- Change Little Joan's own keys too (Cmd/Ctrl+L and the rest), not just the global shortcut.
- More command-bar commands for quick actions and settings (translate, theme, search engine, …).
- Saved logins: credential storage, Apple Passwords / iCloud Keychain, and passkeys already on
  the device, where each engine allows it (needs research per OS first).
- A few more bundled extensions.
- Windows (and Linux): the history key becomes **Ctrl+H** instead of Ctrl+Y (decided: "Ctrl+H is
  better").
- Decided: installer size growth from `panic = "unwind"` is fine ("installers can grow").
- Still planned from round 1: phases 7–10 (Linux fullscreen, downloads + zoom, the Wayland
  shortcut, macOS signing), then LJ19 (v0.2.0).

→ *2026-10-18: the owner said yes, with decisions ("Design Decisions Already Made → round 3"):
planned as round 3 — LJ36–LJ44 and round 1's LJ13–LJ19 finish v0.2.0, LJ45–LJ57 are v0.3.*

---

## v0.2 round 3 steps — finishing v0.2.0

> Planned 2026-10-18 with the owner (decisions: "Design Decisions Already Made → round 3"; Ground
> Rules 21–26). **Execution order: 18 → 19 → 20 → 7 → 8 → 9 → 10 → 11 (v0.2.0) → 21 → 22 → 23 →
> 24 → 25 → 26 (v0.3).** Every phase branches from the latest `main`, and its PR goes into
> `main` (no stacking). Gates (Ground Rule 24): at the end of an `auto` phase the agent merges the
> green PR (a merge commit, never squash, never `--delete-branch`); a `checkpoint` step fills the
> PR with the owner's checklist, may merge its green PR, and STOPS — the next phase waits for the
> owner's go; a `review` step ends a security-sensitive phase — the PR is marked ready and the
> agent stops without merging. Steps whose look matters name CI screenshots: download them into
> `test-results/ci/`, look at them on every OS, describe them in the notes, then delete them.
> Each step updates CHANGELOG `[Unreleased]` (Ground Rule 17), the README and
> `docs/architecture.md` (Ground Rule 15). `src-tauri/src/commands.rs` (481 lines),
> `scripts/ci/smoke-log.mts` (468) and `scripts/ci/smoke.mts` (1,227) are already past the
> ~400-line guide: new commands go into child modules (`src-tauri/src/commands/<topic>.rs`), new
> smoke runs and their rules into their own `scripts/ci/smoke-<topic>.mts` (+ test), wired into
> `smoke.mts`'s sequence the way `smoke-theme.mts` is.

### LJ36 — YouTube's fullscreen on macOS: reproduce in CI, guard WebKit's fullscreen, YouTube on the desktop agent · strong · auto

> Phase 18. Branch `lj/p18-fixes` from the latest `main`; the PR goes into `main`.

**Verify:** `npm run verify` green, including:
- vitest tests for the smoke page's new logic (`tests/smoke/fullscreen-logic.mjs`, + test: the
  button positions; the report title for each path and outcome; `playerPath(userAgent)` — the
  phone-like player takes the iOS path under an iPhone agent, element fullscreen otherwise) and
  for the new rules (`scripts/ci/smoke-fullscreen.mts`, + test: a log where fullscreen ends within
  1 s of entering fails; one where it holds until the page's own exit passes; a macOS log with a
  `page-bounds` line while `wk-fullscreen` isn't `not` fails);
- Rust tests in `phone.rs`: `site_mode(host, platform, overrides)` — the built-in WebKit list
  gives `youtube.com` and its subdomains (`www.`, `m.`) the desktop agent on macOS and Linux and
  nothing on Windows; a user entry wins, for the host or a parent domain; `"auto"` overrides a
  built-in; `wants_mobile` honours the mode (desktop / phone at any width, auto by width);
  `desktop_url` rewrites `m.youtube.com/watch?v=x` → `www.youtube.com/watch?v=x` and
  `en.m.wikipedia.org/wiki/X` → `en.wikipedia.org/wiki/X` and nothing else;
- `settings.rs`: `siteModes` (`{ "<host>": "auto" | "phone" | "desktop" }`; keys are lower-case
  host names without `www.` and without a port — a mode is per host, not per port —, IP addresses
  as they are; bad entries dropped, at most 500); the lookup walks a name up to its last two
  labels and never walks an IP address;
- the macOS guard's pure part (`fullscreen::page_layout_allowed(webkit_state, in_own_window)`).

CI green on all three OSes with a **fullscreen run** (its own launch): the smoke page in `?fs=both`
mode at 390×600 (phone agent) and at 720×600 (desktop agent) — (a) a click on "Element
fullscreen" (`requestFullscreen()` on a container, as desktop YouTube does), (b) 2 s later a click
on the page's "Exit" (`document.exitFullscreen()`), (c) a click on "Phone player" (behaves like
m.youtube.com's player: under an iPhone agent `video.webkitEnterFullscreen()`, then it waits for
`webkitbeginfullscreen` / `webkitendfullscreen` and tries `screen.orientation.lock`), (d) its own
exit — each reported as `LJ-FS <element|phone> width=<w> ua=<phone|desktop> entered=<0|1>
held_ms=<n> exited_by_page=<0|1> events=<list>`; then a second launch with `"siteModes":
{ "127.0.0.1": "desktop" }` at 390×600 (added with the fix), where the page reports `ua=desktop`
and its player takes element fullscreen. **The first commit makes the run informational** (results in `summary.json`
and the notes), so CI shows the bug before anything changes. **The fix commit
(`lj-step-36-fix`) turns the rules on:** every element-fullscreen path holds until the page exits
it and leaves when asked (required on Windows and Linux; on macOS whenever the click reached the
page — the existing macOS click rule); the `siteModes` launch reports `ua=desktop` at 390 on every
OS; on macOS no `page-bounds` / `page-visible` line appears while `wk-fullscreen` isn't `not`. The
phone path under the iPhone agent stays informational (it is how WebKit on a Mac treats a page
written for iOS; the desktop default routes YouTube around it). Screenshots `fs/element-390.png`,
`fs/element-720.png`, `fs/phone-390.png`, `fs/desktop-site-390.png` (1 s after each entry).
**Commit:** `lj-step-36: reproduce the macOS fullscreen bug in CI (element and phone-player fullscreen, narrow and wide)`, then `lj-step-36-fix: guard WebKit's fullscreen on macOS; YouTube gets the desktop agent on WebKit`

```text
The owner's checklist (LJ35's notes): on macOS, YouTube's own fullscreen button entered
fullscreen and dropped back at once; then the player's exit button did nothing (Esc exited).
**Confirmed by the owner (2026-10-18): it was the PHONE version of YouTube — phone mode's
iPhone agent, m.youtube.com — that failed; desktop YouTube works.** The research for phase 19
explains it: the default 480 px window is below phone mode's 600 px, so YouTube serves its mobile
site, whose player expects iOS events (webkitbeginfullscreen / webkitendfullscreen,
screen.orientation.lock) that never fire on macOS. It also found a second hazard of our own: wry's
set_bounds sets a WKWebView's frame inside its CURRENT superview (wry 0.57 wkwebview/mod.rs
1031–1047), and during WebKit's element fullscreen that is WebKit's fullscreen window — so any
apply_layout then shrinks the video. macOS has no element-fullscreen hook in Little Joan
(fullscreen.rs, platform/mod.rs), so bar::element_fullscreen() is never true there:
joan::on_window_event's Resized → bar::apply_layout → set_bounds() + show() on the page, and
phone_mode::apply (it checks only window.is_fullscreen(), false for WebKit's separate window), run
as if nothing were fullscreen.

Decided (the owner's confirmation; do not relitigate): reproduce in CI first; guard WebKit's
fullscreen on macOS; on WebKit (macOS and Linux) youtube.com and its subdomains use the desktop
agent by default, through a small per-site table the owner can override, with the m. → www
rewrite. Phase 19 (LJ40) builds the table's UI — the command-bar rows, the toolbar pill, the
Settings list, Windows' agent — on top of what this step ships; keep the shapes below so it needs
no migration.

1. Reproduction (the first commit):
   - tests/smoke/smoke.html gains `?fs=both`: large buttons at known positions
     (tests/smoke/fullscreen-logic.mjs exports them as fractions of the viewport, so smoke.mts can
     click each): "Element fullscreen" (requestFullscreen on a container div, prefixed fallback),
     "Exit" (shown in fullscreen; document.exitFullscreen, prefixed fallback), "Phone player" (a
     <video> playing a canvas.captureStream() — no media file in the repository — whose click
     handler follows playerPath(navigator.userAgent): under an iPhone agent
     video.webkitEnterFullscreen() and a 1 s wait for webkitbeginfullscreen, then
     screen.orientation.lock("landscape") in a try, as m.youtube.com's player does; otherwise
     element fullscreen of its container), and the player's own exit. It listens to
     fullscreenchange, webkitfullscreenchange, webkitbeginfullscreen, webkitendfullscreen and
     reports as the Verify says. Pure logic in fullscreen-logic.mjs (JSDoc-typed, tested, like
     smoke-logic.mjs). If the engine has no captureStream, or the video never reaches
     HAVE_METADATA, the phone path reports `unavailable`.
   - scripts/ci/smoke-fullscreen.mts (new, + test): the run and fullscreenFailures(events). Its
     own launch, so the main run's memory numbers don't move: --show the page, --size 390x600,
     click each button in turn with a variant of smoke.mts's clickCentre that takes a point
     (factor it out; clickCentre keeps its behaviour), screenshot, then --size 720x600 and again.
     (The second launch, with siteModes, comes with the fix commit, which implements the setting.)
     Exits only through the page's buttons, never a key: the bug is about clicks.
   - macOS instrumentation (smoke mode only, smoke::enabled()): platform/mac.rs polls WKWebView's
     fullscreenState (macOS 13+, objc2 msg_send after respondsToSelector:; objc2-web-kit 0.3.2's
     WKWebView.rs documents it) every 100 ms while a page window exists and logs `wk-fullscreen
     <not|entering|in|exiting>` on each change; bar/view.rs's place() logs `page-bounds <x,y,w,h>`
     and `page-visible <bool>` for the page webview; joan::on_window_event logs Resized / Focused
     / Moved while wk-fullscreen isn't `not`. They stay after the fix — they are how CI proves it.
   Commit, push, open the phase-18 PR into main as a draft, let CI run, download the macOS logs and
   the screenshots, and write into the notes exactly what happened per path, width and agent on
   every OS.

2. The fix (`lj-step-36-fix`):
   a. The guard (macOS). A pure fullscreen::page_layout_allowed(webkit_state, in_own_window) and
      platform::page_in_webkit_fullscreen(page) in platform/mac.rs: fullscreenState isn't
      NotInFullscreen (macOS 13+), or — on macOS 12, which lacks the property — the page's
      WKWebView no longer sits in the Little Joan window ([webview window] isn't ours). Read it
      where the decision is made (on the main thread, through with_webview): bar/view.rs's
      apply_layout and place() don't set bounds on, show or hide the page while it is true (the
      bar may still be laid out); phone_mode::apply doesn't switch; set_overlay(true) / focus_bar
      don't open the command bar. When the guard skipped a layout, check again every 100 ms only
      until WebKit's fullscreen has ended, then apply the layout once (nothing runs at rest or
      while nothing was skipped). Windows and Linux keep their hooks.
   b. The per-site table, minimal. settings.rs: siteModes (Verify's shape). phone.rs:
      site_mode(host, platform, overrides) with a built-in list for the WebKit engines (macOS,
      Linux): youtube.com → desktop (subdomains included); a user entry for the host or a parent
      domain (walking up to the last two labels; an IP address is never walked; the port is
      ignored) wins, "auto" included. wants_mobile(enabled,
      width, mode): desktop and phone hold at any width; auto follows the width when mobileSites
      is on and is desktop when it is off. phone_mode::apply takes the page's host
      (bar::page_url) into account; the app's own navigations (bar::navigate: typed addresses,
      picked rows, links from other apps later) set the agent for the target's host first; for a
      navigation the page starts itself, the page's Started event applies it, and when the agent
      really changes the page is reloaded once (never twice in a row for the same URL).
   c. The rewrite. phone::desktop_url (pure, Verify's rules) — when the agent goes to desktop for
      a page on a known mobile host, navigate to the rewritten URL instead of reloading; when the
      app navigates to such a URL for a host whose mode is desktop, it navigates to the rewritten
      one.
   - No UI in this step (LJ40 adds it); the README says that YouTube uses the desktop site on
     macOS and Linux, and that `siteModes` in settings.json overrides it.
   - If the element path still drops out of fullscreen on macOS with the guard in place, stop and
     report with the log — that would be a different cause.
   - Leave Windows' and Linux's fullscreen code alone; if the run fails there, fix it only if it
     is a bug in our code, otherwise record it and stop.

3. Docs: docs/architecture.md (the Fullscreen video table's macOS row: the guard; Page size and
   phone mode: siteModes and the built-in list; the smoke-log events), README (phone mode:
   YouTube on the desktop site on macOS and Linux), CHANGELOG ([Unreleased] → Fixed), this plan
   (Deferred follow-ups: strike the YouTube line once CI is green, naming LJ36 — the owner's eyes
   on real YouTube are the phase's human check; Landmines: what CI showed).

Do NOT add a fullscreen setting, switch macOS to window fullscreen, or add YouTube checks anywhere
but the built-in site list and the rewrite rules.

Run `source scripts/env.sh && npm run verify`. If it is green, add implementation notes (with the
df -h numbers) and commit everything as one commit: "lj-step-36: reproduce the macOS fullscreen bug
in CI (element and phone-player fullscreen, narrow and wide)". Push, open the phase-18 PR into main
as a draft, and watch CI (`gh run watch <id> --repo memoliertorerr/LittleJoan`). Then the fix as
above: local verify green, flip LJ36 to ✅ with notes, commit "lj-step-36-fix: guard WebKit's
fullscreen on macOS; YouTube gets the desktop agent on WebKit", push, and watch CI until verify,
package and smoke are green on all three OSes; fix Windows/Linux-only failures per Ground Rule 3.
If it is still red after two honest fix attempts, stop and report. Never commit red; never weaken
a check.
```

**Implementation notes (2026-10-18) — the reproduction (`lj-step-36`):**

- **The smoke page's `?fs=both`** (`tests/smoke/smoke.html`, logic in `tests/smoke/fullscreen-logic.mjs`
  + test): "Element fullscreen" (a container `requestFullscreen()`s itself, prefixed fallback) on
  the top third, "Phone player" (a `<video>` playing a `canvas.captureStream(30)`) on the bottom
  third, a gap around the middle; in fullscreen a big Exit in the middle of the screen (25–75 % ×
  30–70 %). `playerPath(ua)`: an iPhone agent takes the iOS route (`webkitEnterFullscreen()`, a 1 s
  wait for `webkitbeginfullscreen`, then `screen.orientation.lock("landscape")` in a try; a click on
  the fullscreen video is the player's exit), anything else element fullscreen of the player. One
  `FsAttempt` per click watches `fullscreenchange`, `webkitfullscreenchange`,
  `webkitbeginfullscreen`, `webkitendfullscreen` and a 100 ms poll (`fullscreenElement` /
  `webkitDisplayingFullscreen`), gives up 4 s after a click that never entered or 6 s after
  entering (then leaves fullscreen itself, so the next attempt starts windowed), and reports
  `LJ-FS <element|phone> width= ua= entered= held_ms= exited_by_page= events=<list>`; a player
  without `captureStream` or metadata reports `unavailable=<why>`. Ready: `LJ-FS ready width= ua=`.
- **Where to click:** the windows aren't centred after `--size` (the top-left stays), so the app
  logs `page-screen <x>,<y>,<w>,<h> scale=<s>` (smoke mode, every layout: the window's inner
  position + the page's bounds, physical pixels) and the run clicks each button's centre there
  (points on macOS); exits are clicked at the screen's centre, which the fullscreen Exit covers.
  `clickCentre` became `clickAt(point | "centre")` with the old behaviour for `"centre"`.
- **The run** (`scripts/ci/smoke-fullscreen.mts` + test; `smoke.mts` passes a small driver):
  its own launch after the theme runs; `--show smoke.html?fs=both`, then per size (`--size
  390x600`, phone agent; `--size 720x600`, desktop agent) it waits for the page's ready title
  after its last `page-started` (phone mode's reload) and steady for 1.5 s, clicks "Element
  fullscreen", screenshots 1 s later (`fs/element-<w>.png`), clicks Exit 2 s after the click
  (skipped when the attempt already reported), waits for the report; the same for "Phone player"
  (`fs/phone-390.png`). `fullscreenFailures` (the rules) and `fullscreenSummary` exist; in this
  commit the results only go to `summary.json` (`fullscreenRun`: the reports, WebKit's states,
  the window's events during it, what the rules would say).
- **macOS instrumentation** (smoke mode only): `platform/mac.rs` polls `fullscreenState`
  (`respondsToSelector:` first) every 100 ms while a page window exists and logs `wk-fullscreen
  <not|entering|in|exiting>` on each change (`fullscreen::WebKitFullscreen`, an unknown value
  counts as `in`); `bar/view.rs`'s `place()` logs `page-bounds` / `page-visible` for the page on
  every OS; `joan::on_window_event` logs `wk-window resized|focused|moved` while the last state
  logged isn't `not`.
- `npm run verify`: 482 TS tests, 257 Rust tests, clippy clean. Disk: 12 GiB free at the start of
  the phase, 11 GiB after verify; `.cache/target` 1.4 GB.

**What CI showed (run 39147876852, every job green — the run was informational):**

| | Element, 390 px (phone agent) | Phone player, 390 px | Element, 720 px (desktop agent) | Phone player, 720 px |
|---|---|---|---|---|
| Windows (Android agent: every path is element fullscreen) | held 3.3 s, left by its Exit | held 3.4 s, left by its Exit | held 3.3 s, left by its Exit | held 2.9 s, left by its Exit |
| Linux, scale 1 and 2 | held 2.2 s / 2.3 s, left by its Exit | iOS route: entered (`fullscreenchange`, no `webkitbeginfullscreen`, no `screen.orientation.lock`), left by its `webkitExitFullscreen` | held, left by its Exit (scale 2: the exit seen by the poll) | held, left by its Exit |
| **macOS** | **entered, then the Exit did nothing (6 s timeout)** | iOS route: entered, held 3.4 s, left by its own exit | **no report: the page reloaded 300 ms into fullscreen** | (on the phone agent by then) iOS route, held 3.4 s, left by its own exit |

- **macOS, the cause (the log):** while WebKit entered element fullscreen (`wk-fullscreen entering`),
  tao delivered `Moved`, `Resized` and `Moved` for the Little Joan window — `resized 480x640`, the
  size the window was created at, not its 390×600 —, and `apply_layout` set the page's bounds
  (`page-bounds 0,40,480,600`). wry's `set_bounds` sets the WKWebView's frame in its current
  superview, WebKit's fullscreen window: `fs/element-390.png` shows that black fullscreen window
  with the page shrunk to a 480×600 frame in its top-left corner — the owner's "drops back to
  the normal size at once" —, the centred Exit out of reach of the click at the screen's centre
  ("the exit button does nothing"). At 720 px the same spurious 480 px width also crossed phone
  mode's 600 px: `user-agent phone` and a reload ended the fullscreen (`fs/element-720.png` caught
  the aborted transition, the window behind it) and left the page on the phone agent at 720 px.
  The phone player's iOS route on macOS, by contrast, held until its own exit in both runs, with
  no window events: it is the element route, and our layout during it, that broke. The phone
  site makes it likelier only through phone mode (the 600 px switch).
- **Screenshots:** Windows and Linux (scale 1 and 2) — the element fills the whole screen
  (yellow, the Exit in the middle; the fullscreen color rule lost to the id's, fixed in the fix
  commit), the phone player's element route fills it in blue, Linux's iOS route shows the video
  with WebKitGTK's media controls; macOS — `element-390.png` as above, `phone-390.png` the video
  fullscreen with WebKit's native controls ("Live Broadcast"), `element-720.png` the aborted
  transition.

**Implementation notes — the fix (`lj-step-36-fix`):**

- **The guard (macOS):** `fullscreen::page_layout_allowed(state, in_own_window)` (pure, tested):
  only while `fullscreenState` is "not in fullscreen" (`None` on macOS 12) **and** the page's
  WKWebView sits in the Little Joan window — both, so CI's macOS 15 runs test the window check
  too. `platform::page_in_webkit_fullscreen(page)` reads them on the main thread
  (`mac::webkit_fullscreen`: `with_webview`, which runs at once on the main thread and is
  answered within 500 ms elsewhere; no answer counts as "not"). `bar/view.rs`'s `apply_layout`
  still lays out the bar but sets no bounds on, shows or hides the page meanwhile; `set_overlay(true)`,
  `focus_bar` and `show_history` don't open the command bar; `phone_mode` doesn't switch. A
  skipped layout or switch starts one checker (`after_webkit_fullscreen`, every 100 ms, smoke log
  `layout-deferred` / `layout-resumed`) that applies the layout and phone mode once when the
  fullscreen has ended, and stops with the window. Windows and Linux keep their hooks (the
  function is `false` there).
- **`siteModes`** (`settings/site_modes.rs`): keys through `site_key` (the `url` crate's host:
  lower case, IDNs in ASCII, `www.` dropped; a scheme, port, path, user or space drops the entry;
  IPv6 as `[::1]`), values `auto` / `phone` / `desktop`, the first of two spellings of one site
  wins, at most 500 (in key order, serde_json's map being sorted). `candidates` walks a name up to
  its last two labels and returns an IP address (or a single label) alone. Settings saves keep it
  (`save_settings` only changes the fields Settings sends).
- **`phone.rs`:** `site_mode(host, platform, overrides)` — a user entry for the host or its
  nearest parent wins (`auto` too), else `WEBKIT_DESKTOP_SITES = ["youtube.com"]` on macOS and
  Linux (subdomains included), else auto; `wants_mobile(enabled, width, mode)`; `desktop_url`
  (`m.youtube.com` → `www.youtube.com`, `<lang>.m.wikipedia.org` → `<lang>.wikipedia.org`, http(s)
  only, nothing else).
- **`phone_mode.rs`:** the agent follows the page's host (web pages only — the app's pages and
  extension panels are auto). `prepare` (called by `bar::navigate`, so typed addresses, picked
  rows, `--show <url>` and Translate) sets the target's agent first and turns a known phone address
  of a site on `desktop` into its desktop one; `page_started` (the page's `Started` event, handled
  off the engine's callback) switches for a navigation the page made itself and loads the page
  once more when the agent really changed — never twice in a row for the same URL; the reload
  counts as a reload visit. Going to the desktop agent on a known phone host loads its desktop
  address (`bar::load`, which leaves the view alone) instead of reloading.
- **The smoke** turns the rules on (`fullscreenFailures`) and adds the second launch with
  `"siteModes": { "127.0.0.1": "desktop" }` at 390×600 (`fs/desktop-site-390.png`).
- `npm run verify`: 482 TS tests, 271 Rust tests, clippy clean. Disk: 13 GiB free at the start of
  the fix and after verify; `.cache/target` 1.4 GB.

**CI for the fix (run 39149305504):** verify, package and smoke green on Windows and both Linux
jobs; **macOS smoke red on one rule, a race in the instrumentation**: every element path now held
until the page's own Exit (390 px: 3.6 s; 720 px: 3.7 s; the siteModes launch reported
`ua=desktop` at 390 and its player took element fullscreen, 3.7 s), the layout during WebKit's
entry was skipped (`layout-deferred`) and applied once it ended (`layout-resumed`) — but the
resumed layout's `page-bounds` landed 4 ms before the 100 ms poll logged `wk-fullscreen not`,
although the guard itself had read "not" (and the view back in its window) for that decision.
**`lj-step-36-fix2`:** the guard's own read logs the state too (`note_state`, on the main thread,
before the caller acts), so the log shows what each decision saw; the rule is unchanged.
Also seen: **WebKit's element fullscreen puts the Little Joan window back to the size it was
created at** — after a `--size 390x600` the window came back from fullscreen 480×640 (`resized
480x640` during the entry, and the resumed layout at that size; the page then reported 480 px, and
at 720 px phone mode rightly switched to the phone agent for the 480 px window). Whether a window
resized by dragging does the same on a real Mac is the owner's check; recorded under Deferred
follow-ups (not fixed here: it isn't the fullscreen bug, and restoring the size would be new
behavior).
**CI for fix2 (run 39150759131):** all ten jobs green. Every element-fullscreen path held until its
own Exit on every OS (macOS 3.3 s at 390 and 720 px; Windows 3.3–5.5 s; Linux 2.1–2.6 s at scale 1
and 2), the iOS route held too where it ran (macOS, Linux), the siteModes launch reported the
desktop agent at 390 px with its player in element fullscreen everywhere, and no layout touched
the page during WebKit's fullscreen. Screenshots: macOS `element-390.png` and `desktop-site-390.png`
now show the element filling the whole screen (green, the Exit in the middle), as Windows and
Linux do.

### LJ37 — Ctrl+H opens history on Windows and Linux; double-clicking the toolbar zooms the window · standard · auto

**Verify:** `npm run verify` green, including `keys.rs` tests: Ctrl+H → `PageAction::History` on
Windows and Linux; Ctrl+Y → nothing (the page gets it back: redo in WebView2's text fields);
Ctrl+Shift+H, Ctrl+Alt+H (AltGr) and a plain H → nothing; `Key::from_windows_vk(0x48)` and
`Key::from_gdk_name("h")` → `Key::Letter('h')`; macOS still gets nothing from `keys.rs` (Cmd+Y
stays the View menu's item in `menu.rs`). `capabilities/bar.json` gains exactly
`core:window:allow-internal-toggle-maximize`, and a new `capability_tests.rs` test fails if any
other capability grants it. CI green on all three: the smoke's history key becomes Ctrl+H on
Windows (SendKeys `^h`) and Linux (xdotool `ctrl+h`), Cmd+Y on macOS as before, with
`history-view` and `history-list` following; then, with the view closed again, Ctrl+Y on Windows
and Linux must open nothing within 2 s (`historyViewFailures` gains the case, with a test).
**Commit:** `lj-step-37: Ctrl+H opens history on Windows and Linux; double-clicking the toolbar zooms the window`

```text
Two small decided changes (LJ35's "Requested for the next version"; Deferred follow-ups).

1. The history key. The owner decided: Ctrl+H on Windows and Linux (Chrome's history key), so
   Ctrl+Y is redo again in WebView2's text fields; Cmd+Y stays on macOS.
   - src-tauri/src/keys.rs: the 'y' arm of action_for becomes 'h'; the module table and the tests
     follow (the Verify's cases).
   - menu.rs (Cmd+Y) and bar.rs (show_history) don't change.
   - Every place that says Ctrl+Y: ui/bar.ts, ui/command/rows.ts (HISTORY_ROW's comment),
     src-tauri/src/bar.rs, README (Using it; the page keys table), docs/architecture.md (the page
     keys table), scripts/ci/smoke.mts and scripts/ci/smoke-command.mts (+ test): "Ctrl+H (Cmd+Y
     on macOS)".
   - Smoke (scripts/ci/smoke.mts, where Ctrl+Y opens the history view today): Ctrl+H on Windows
     and Linux; then the Ctrl+Y check from Verify.
   LJ43 later moves every key into one keymap with Ctrl+H as the default; don't build that here.

2. Double-click the toolbar. Tauri's drag script turns a double click on a drag region into
   internal_toggle_maximize (tauri 2.12.4 window/scripts/drag.js; the command maximizes or
   restores a resizable window, window/plugin.rs), which needs
   core:window:allow-internal-toggle-maximize. Grant it in src-tauri/capabilities/bar.json only
   (update that capability's description and docs/architecture.md's capabilities table), and add
   the capability test from Verify. It applies on Windows and Linux too, where the toolbar's
   empty parts are drag regions under the native title bar: a double click there maximizes like
   the title bar does — keep it, and say so in the notes. It doesn't follow macOS's "Double-click
   a window's title bar to: Minimize" preference (Tauri's command always zooms); record that under
   Deferred follow-ups instead of reading the system preference. CI can't usefully double-click
   on the runners' fixed screens: this is a human check.

Do NOT change any other key, the toolbar's markup or Tauri's drag script.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ37 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-37: Ctrl+H opens history on
Windows and Linux; double-clicking the toolbar zooms the window". Push and watch CI until verify,
package and smoke are green on all three OSes (Ground Rule 3). If it is still red after two honest
fix attempts, stop and report. Never commit red; never weaken a check.
```

**Implementation notes (2026-10-18):**

- **The history key:** `keys.rs`'s `'y'` arm became `'h'` (Ctrl+H → History on Windows and
  Linux); Ctrl+Y now reaches the page (redo in WebView2's text fields). Tests: Ctrl+H on both;
  Ctrl+Shift+H, Ctrl+Alt+H (AltGr), Alt+H and a plain H nothing; `Key::from_windows_vk(0x48)` and
  `Key::from_gdk_name("h")` → `Letter('h')`; Ctrl+Y nothing anywhere; macOS nothing from
  `keys.rs` (Cmd+Y stays the View menu's item). `menu.rs` and `bar::show_history` unchanged; the
  comments in `ui/bar.ts`, `ui/command/rows.ts`, `bar.rs`, the smoke scripts, the README and
  architecture.md say "Ctrl+H (Cmd+Y on macOS)".
- **Smoke:** the history key is Ctrl+H on Windows (SendKeys `^h`) and Linux (xdotool `ctrl+h`),
  Cmd+Y on macOS (`HISTORY_KEY`; the theme runs too); then, Windows and Linux, Esc (back to the
  command bar) and Ctrl+Y: the log of the next 2 s must hold no `history-view`
  (`historyViewFailures`' `afterCtrlY`, with a test). `pressKeys` learned Escape.
- **Double-click:** `capabilities/bar.json` gains exactly `core:window:allow-internal-toggle-maximize`
  (its description says so); `capability_tests.rs`' `only_the_bar_may_toggle_maximize` fails if any
  other capability grants it — or `core:window:default` / `core:default`, which hold it — and if the
  bar loses it. Tauri's drag script was left alone, and so was the toolbar's markup. On Windows and
  Linux the toolbar's empty parts are drag regions under the native title bar too, so a double
  click there maximizes like the title bar does — kept, as the prompt says. It always zooms;
  macOS's "Double-click a window's title bar to: Minimize" preference isn't read (Deferred
  follow-ups). CI can't usefully double-click on its fixed screens: a human check.
- `npm run verify`: 483 TS tests, 273 Rust tests, clippy clean. Disk: 13 GiB free at the start and
  after verify.
- **CI (run 39151866041):** all ten jobs green. Windows and both Linux jobs: `pressed Ctrl+H` →
  `history-view` and the list; Esc, then `pressed Ctrl+Y` → nothing at all in the next 2 s. macOS:
  Cmd+Y opened the view as before (the theme run's too). The screenshots look as before
  (`history.png` on every OS: the history view over the command bar, the smoke page under Today).

### LJ38 — The bar's first paint: its theme before it paints; panels opened from hibernation wait for the bar · strong · auto

**Verify:** `npm run verify` green, including:
- Rust tests: `bar::theme_script(&Theme)` (pure) returns `window.__littleJoanTheme = <json>;` with
  the serde JSON of a checked theme (only `#rrggbb` colors, a mode name and two numbers; a test
  feeds the extremes and asserts no `<`, newline or second statement); only the bar's builder adds
  an initialization script (a test reads `bar.rs` and finds exactly one `initialization_script(`
  call, in the bar's builder — the page webview never gets one; cookies.rs has a source test of
  the same kind);
- `extensions/session` tests: an opening while the bar hasn't called `start_info` in this window
  waits for it (at most 2 s, then opens anyway and logs `bar-ready timeout`); with the bar ready it
  opens at once;
- vitest tests for `ui/bar-boot.ts` (pure part): the theme from `window.__littleJoanTheme` is
  applied before the first `await`; `start_info`'s theme is applied again only if it differs (a
  preview may have changed it); a missing or malformed value falls back to today's path.

CI green on all three: every theme run logs `theme-early <mode> colors=<n>` before that window's
`theme <mode> colors=<n>` from `start_info` (`themeFailures` gains the rule, 2 test cases); the
extension run's `--extension hello` from hibernation logs `bar-ready` before hello's
`page-started`, and macOS's `extension-hello.png` shows the toolbar with hello's icon, name and the
Extension badge, not an empty field — look at it.
**Commit:** `lj-step-38: the bar gets its theme before it paints; panels opened from hibernation wait for the bar`

```text
Two first-paint problems of the trusted bar (Deferred follow-ups), fixed in the bar only: the
page webview never gets an initialization script (Ground Rules 10 and 18), and the settings
window stays as it is.

1. The default-theme flash. The bar applies the theme when start_info answers (ui/bar.ts →
   applyTheme, ui/lib/apply-theme.ts), a moment after its page first painted with
   ui/lib/theme-defaults.css (Mist), so a summon with another theme can flash Mist.
   - bar.rs, add_webviews, the bar's WebviewBuilder only: .initialization_script(theme_script(
     &theme)) with AppState::shown_theme() (the preview while Settings edits one, else the saved
     theme — what start_info hands out). theme_script is pure and tested (Verify).
     Initialization scripts run at document start, before the page's own scripts; the engine
     injects them, so the bar's CSP doesn't block them.
   - ui/bar-boot.ts (new; its pure part tested; bar.ts calls it first, at module top level):
     read window.__littleJoanTheme, check it the way apply-theme.ts checks a theme, and call
     applyTheme synchronously — before any await, so it lands before the first frame of a page as
     small as bar.html. The bar can't write the smoke log, so its first set_theme_tokens report
     carries a new optional `early: true`, and Rust logs `theme-early <mode> colors=<n>` for it
     (commands.rs set_theme_tokens; the report's checks stay as strict). start_info's theme is
     applied only when it differs.
   - All color math stays TypeScript (v0.2 round 2 decisions); Rust only hands over the settings.
   If the owner still sees a flash on their Mac (the human check), the next move is handing the
   bar the painted --lj-* variables themselves (its last report, kept in AppState across
   windows): record that as a Deferred follow-up; don't build it now.

2. macOS: a panel opened from hibernation can leave the toolbar unpainted. With --extension <id>
   while hibernating, the window is built with the command bar, and the panel can commit before
   the bar's script has run; the toolbar then stayed an empty field (LJ32's CI) although
   toolbar-label was sent twice. Decided (the second option in Deferred follow-ups): open the
   panel only once the bar's script runs.
   - Per-window state `bar_ready` (bar.rs, next to the other per-window state; reset in
     add_webviews), set by start_info — the bar's first call — which logs `bar-ready`.
   - extensions/session.rs's opening path for a new window (joan::open_extension, CreateExtension)
     waits for it on its own thread — never on the main thread, never inside an engine callback —
     at most 2 s, then opens anyway and logs `bar-ready timeout`. Openings in a window whose bar
     already runs (the command bar's rows, FocusAndExtension) don't wait.
   - The nonce, the commit rule and the detour through the start page (LJ29's fixes 4–10) are
     unchanged: the wait comes before `expect`.
   - Smoke (scripts/ci/smoke-extensions.mts + test): the rule from Verify.

Files: src-tauri/src/{bar.rs, commands.rs (start_info, set_theme_tokens), extensions/session.rs
(+ session/tests.rs), joan.rs if the opening path needs it}, ui/{bar.ts, bar-boot.ts (new, +
test)}, ui/lib/apply-theme.ts only if its check must be exported, scripts/ci/{smoke-theme.mts,
smoke-extensions.mts} (+ tests), docs/architecture.md (Themes → "Applying it"; how a panel opens),
CHANGELOG ([Unreleased] → Fixed), this plan (Deferred follow-ups: strike the macOS
unpainted-toolbar line once CI's screenshot shows the toolbar, naming LJ38; annotate the
theme-flash line — the owner's eyes close it).

Do NOT add an initialization script to the page webview or to Settings, hide the bar's body until
the theme arrives (LJ32's landmine: WebKit may then never paint it), or delay opening a panel in a
window whose bar already runs.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ38 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-38: the bar gets its theme
before it paints; panels opened from hibernation wait for the bar". Push and watch CI until every
job is green (Ground Rule 3; two honest fix attempts, then stop). Look at extension-hello.png and
the theme runs' screenshots on every OS. This ends phase 18. Fill the PR description from the
template. Needs human check, on your Mac (Windows / Linux where you have them):
- YouTube in a narrow (< 600 px) window: the desktop site now (macOS); its fullscreen button, F,
  and the player's exit button work; in a wide window too.
- Windows / Linux: Ctrl+H opens history; Ctrl+Y redoes in a text field.
- Double-click the toolbar's empty part: the window zooms; again: it comes back.
- With a non-default theme, summon ten times: no Mist flash.
- Quit, then `little-joan --extension snake`: the toolbar shows Snake.
- The must-not-disturb list.
Once verify, package and smoke are green on every OS job, merge the PR into main with a merge
commit (`gh pr merge <n> --merge --repo memoliertorerr/LittleJoan`; never squash, never
--delete-branch), then stop and report.
```

**Implementation notes (2026-10-18):**

- **The theme before the first paint:** `bar/boot.rs` (new; `bar.rs` was at its ~400-line limit,
  so `set_theme` moved there too): `theme_script(&Theme)` checks the theme again
  (`settings::theme::parse`) and returns `window.__littleJoanTheme = <json>;` — tests: a two-color
  theme's exact script; hostile colors (`</script>…`, a newline and a second statement), an
  infinite intensity, a negative grain, the extremes: no `<`, no line break, one `;`, bad colors
  dropped and numbers clamped. `add_webviews` adds it to the bar's `WebviewBuilder` only, with
  `AppState::shown_theme()` (the preview while Settings edits one, else the saved theme), and
  remembers it for the log. A test reads `bar.rs` and finds exactly one `initialization_script`,
  between the bar's builder and the page's, and no other source file but `settings_window.rs`
  (its tab) has one — the page webview never gets a script. `ui/bar-boot.ts` (+ test):
  `earlyTheme` (an object with a mode string and a colors list, then `themeFrom`'s checks; else
  null — today's path), `bootTheme` (applies it before it returns — the test sees the call
  synchronously), `needsTheme` (start_info's theme only when it differs). `bar.ts` calls it first
  at top level; its first `set_theme_tokens` report carries `early: true`, and `start_info` is
  asked only after that report settled, so the log's order is the bar's (`set_theme_tokens` is an
  async command and could otherwise land after `start_info`). Rust logs `theme-early <mode>
  colors=<n>` for a report that passed the checks (the checks unchanged).
- **Panels from hibernation wait for the bar:** `bar::Ready` (a flag and a condition variable,
  per window: reset in `add_webviews`), set by `start_info` through `bar::mark_ready`, which logs
  `bar-ready` once per window. `joan`'s `CreateExtension` builds the window, then
  `session::open_when_bar_ready` opens the panel from a thread of its own after `after_bar`
  (waits at most `BAR_WAIT`, then opens anyway and logs `bar-ready timeout`; 2 s as the prompt
  said, raised to 5 s by `lj-step-38-fix`, below); a failed
  thread spawn opens at once, as before. `FocusAndExtension` and the command bar's rows open at
  once. The nonce, the commit rule and the detour are untouched: the wait comes before `open`.
  Tests: an opening waits until the bar is ready (set 60 ms later), goes on after the limit
  without it, and goes at once when it is ready.
- **Smoke:** `themeFailures` gains `earlyThemeFailures` (every `theme` after a `theme-early` of the
  run's theme since the previous one; tests for both); the extension run's `barReadyFailures`
  (`bar-ready` before hello's panel's `page-started`, no `bar-ready timeout`; tests).
- `npm run verify`: 493 TS tests, 280 Rust tests, clippy clean. Disk: 12 GiB free at the start and
  after verify; `.cache/target` 1.4 GB.
- **CI (run 39153143119):** all ten jobs green. Every theme run, on every OS, logged `theme-early`
  with the run's theme before each window's `theme` (two windows per run). The extension run's
  `--extension hello` from hibernation logged `bar-ready` before hello's panel committed on every
  OS (macOS: `theme-early` → `bar-ready` + `theme` → `extension-opened hello` → `page-started
  ljext://hello…` 100 ms later; no `bar-ready timeout` anywhere). **`extension-hello.png`:** the
  toolbar shows hello's icon, "Hello" and the Extension badge on macOS (the traffic lights beside
  it — before LJ38 it was an empty field there), Windows, Linux and Linux at scale 2. The theme
  runs' screenshots look as before (macOS's dark command bar, Windows' light two-color toolbar
  above the smoke page).
- **`lj-step-38-fix` — the wait is 5 s, not 2 s (a deviation from the prompt).** The docs commit's
  CI (run 39154369426) went red on macOS only: `bar-ready timeout` in the extension run. That cold
  launch needed 2.08 s from `window-created` to the bar's `start_info` (the page's start page took
  1 s to commit, the bar's page 1.1 s to load, and its sync `start_info` 0.8 s more on the busy
  main thread); LJ38's own run needed 1.61 s, and LJ36's fullscreen run, before LJ38, already
  showed 1.3 s from the bar's load to `start_info` on macOS CI. The panel still committed after `bar-ready` there (both
  were queued behind the same busy main thread), but that is luck. A real Mac runs the bar well
  within a second; the limit only bounds a bar that never runs, so 5 s keeps the wait's purpose on
  slow machines at no cost to fast ones. The smoke rule stays as strict (no timeout allowed).

### LJ39 — Live resize like Little Arc · strong · auto

> Phase 19. Branch `lj/p19-resizing` from the latest `main`; the PR goes into `main`. Designed
> from the round-3 resize research (2026-10-18; "Design Decisions Already Made → Round 3").

**Verify:** `npm run verify` green, including Rust tests:
- `layout.rs`: `changes(previous: Option<&Layout>, next: &Layout) -> Changes` (pure) — only what
  changed: the bar's and the page's bounds, and their visibility; nothing when nothing changed;
  everything for a new window; at scale 1, 1.5 and 2, in every mode;
- `layout.rs` / a pure part for macOS: `mask_for(mode)` — the autoresizing masks per mode: the
  page width- and height-sizable while shown; the bar width-sizable with a flexible bottom margin
  in Toolbar mode, width- and height-sizable in Overlay mode; nothing changes in Fullscreen;
- `phone.rs`: hysteresis — the phone agent below 600 px, the desktop agent again from 640 px,
  in between the previous one stays (a new window decides by 600); a sweep 380 → 820 → 380
  switches exactly twice; LJ36's per-site modes still win;
- the webviews' background color: a pure `background_for(theme_tokens?, dark)` — the bar's last
  painted `--lj-bg-0` (checked `#rrggbb`) else white / near-black from the system's scheme.

CI green on all three with a **resize run** (`scripts/ci/smoke-resize.mts`, + test, its own
launch): a burst of 12 `--size` steps from 380 to 820 px wide, 30 ms apart, then back; the final
viewport is right (`innerWidth` = the window's width, the height minus the 40 px toolbar); exactly
one `user-agent` switch and one reload per crossing; a new smoke-log count `bounds-set <n>` per
Resized event is at most 2 (one per webview, only when they changed); `--size 620x600` coming from
phone stays `ua=phone`, `--size 660x600` gives `ua=desktop`. Screenshots `resize-380.png`,
`resize-820.png` (dark theme run: no white strip at the right or bottom edge on Windows).
**Commit:** `lj-step-39: live resize like Little Arc — only changed layout, macOS autoresizing, themed webview backgrounds, phone-mode hysteresis`

```text
The owner: "Little Arc has no mobile toggle; I like a mobile version on narrow sizes, but outside
of that websites should be more responsive; optimize or better the resizing." The research: Little
Arc is a plain desktop viewport that reflows live — no scaling, no mobile mode. So Little Joan
keeps reflowing pages (no zoom-to-fit, no viewport emulation: desktop engines ignore <meta
viewport>), keeps phone mode for narrow windows, and makes resizing itself smooth. This step is the
smoothness and the threshold; LJ40 is phone mode per site.

What the research found in our code and the vendored sources (REG = the cargo registry under
.cache/cargo/registry/src/index.crates.io-*/):
- Every Resized (src-tauri/src/joan.rs on_window_event) calls bar::apply_layout, which calls
  set_bounds AND show() on both webviews (src-tauri/src/bar/view.rs apply_layout / place). On
  Windows that is ShowWindow + SetIsVisible twice per event; on Linux show() is show_all().
- macOS is about a frame behind: wry gives child WKWebViews the autoresizing mask
  ViewMinYMargin (REG/wry-0.57.0/src/wkwebview/mod.rs 502–504), and tao delivers Resized from a
  BeforeWaiting observer after Core Animation has committed (REG/tao-0.37.1/src/platform_impl/
  macos/window_delegate.rs 94–99, observer.rs 239–241, app_state.rs 384–396), so the bounds we
  set land a frame late and the edges show gaps while dragging.
- Windows: Resized arrives synchronously in WM_SIZE, but the renderer is out of process, so a
  fast drag shows a white gutter — the webviews' default background. WebviewBuilder's
  background_color (tauri 2.12.4 webview/mod.rs ~1205) is unused today.
- Tauri's auto_resize scales child webviews proportionally, so the 40 px bar would grow: never
  use it.

Design (decided; do not relitigate):
1. Only what changed. bar/view.rs keeps the last applied Layout per window (reset with the
   window); apply_layout computes layout::changes and calls set_bounds only for changed bounds
   and show / hide only for changed visibility. In smoke mode log `bounds-set <n>` per call.
   Linux: no show_all per event — the GtkBox packing (platform/linux.rs) is GTK's layout; only
   mode changes touch it, as today.
2. macOS autoresizing. platform/mac.rs sets each WKWebView's autoresizingMask from
   layout::mask_for(mode) (objc2 msg_send setAutoresizingMask:, as mac.rs already talks to the
   WKWebView) after the webviews are built and on every mode change, so AppKit resizes them in its
   own layout pass during a live resize; apply_layout still runs on Resized and confirms the exact
   bounds (with the diffing above, usually nothing). Never touch the page's mask or bounds while
   it is in WebKit's fullscreen (LJ36's guard: platform::page_in_webkit_fullscreen).
3. Windows (and every OS): both webviews get a background color — WebviewBuilder::background_color
   at build time from background_for(the last tokens the bar painted, kept in AppState across
   windows: a few bytes, memory only), and again when the bar reports a new theme
   (set_theme_tokens) if tauri's Webview has a runtime setter for it (check
   REG/tauri-2.12.4/src/webview/mod.rs; if it doesn't, the next window gets it — say so). Also
   the page webview: its background shows only before a page paints.
4. Phone-mode hysteresis. phone.rs: phone below 600, desktop again from 640 (PHONE_BELOW = 600,
   DESKTOP_FROM = 640), the previous agent kept in between; no tablet range. phone_mode.rs keeps
   its 400 ms still period and reloads only when the agent really changes (it does; keep it), and
   never during fullscreen (LJ36's guard on macOS, element_fullscreen elsewhere).

Smoke: scripts/ci/smoke-resize.mts + test (the Verify's sequence; the burst sends `--size` through
single-instance 30 ms apart; count `user-agent` switches and `page-loaded` reloads between
crossings). The run uses a dark theme so a white gutter would show in the screenshots.

Docs: docs/architecture.md (Page size and phone mode: the hysteresis, the diffing, macOS masks,
the background), CHANGELOG ([Unreleased] → Changed), README (phone mode: below 600 px, back to
desktop from 640 px).

Do NOT scale or zoom pages to fit, emulate a viewport, use Tauri's auto_resize, or add a tablet
mode.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ39 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-39: live resize like Little Arc —
only changed layout, macOS autoresizing, themed webview backgrounds, phone-mode hysteresis". Push,
open the phase-19 PR into main as a draft, and watch CI until every job is green (Ground Rule 3; two
honest fix attempts, then stop). Look at both screenshots on every OS.
```

**Implementation notes (2026-10-18):**

- **Only what changed** (`layout/diff.rs`, pure, tested at scale 1, 1.5 and 2 in every mode):
  `changes(previous, next)` gives per webview the bounds to set (`None` when within 0.01 logical
  px) and whether to show or hide it; a view shown again gets its bounds too; a new window gets
  everything. `bar/view.rs` keeps the last applied layout per window (`Applied`: the layout, the
  scale, the mode), forgets it on a new window, when WebKit's fullscreen had the page (macOS) and
  at another scale factor (the engines turn logical bounds into pixels at the scale of the
  moment), and logs `bounds-set <n>` for every layout. The cache is read and written around the
  engine calls, never held across them (the macOS fullscreen check can wait for the main thread).
  Linux: no `show_all` per event any more (wry's `set_visible(true)` is `show_all()`); the GtkBox
  packing still changes only with the mode.
- **macOS masks** (`layout::mask_for`, `layout::flipped` for a flipped superview — tao's content
  view isn't flipped; `platform::set_autoresizing` → `mac::set_autoresizing`, objc2
  `setAutoresizingMask:`): Toolbar — the bar `WidthSizable | MinYMargin`, the page `WidthSizable |
  HeightSizable`; Overlay — both `WidthSizable | HeightSizable`; Fullscreen — nothing. Set when a
  mode is first laid out (and again after WebKit's fullscreen), never on the page while WebKit has
  it. Smoke log `autoresizing <webview> <mask> flipped= follows=` (whether the superview
  autoresizes its subviews). **A deviation from "usually nothing":** `Resized` still sets both
  frames on macOS — the same ones AppKit's masks already gave them — instead of modelling AppKit's
  autoresizing to skip them: layouts also run off the main thread (posted, so they can land late),
  and a model that believed AppKit had done it could leave a view stale with nothing to correct
  it. The masks are what makes the edges follow in the same frame; the confirming `set_bounds` is
  idempotent.
- **Backgrounds** (`bar/background.rs`, pure part tested): `background_for(tokens, dark)` — the
  `--lj-bg-0` the bar painted last (a checked `#rrggbb`) when it was painted in that scheme, else
  `#ffffff` / `#1c1c1e` (the start page's `.blank`, a test reads `ui/style.css`); `dark` is the
  theme's mode, or the system's (`Window::theme`) for `system`. `AppState::painted_background`
  keeps the scheme and that one variable across windows (memory only), set by `set_theme_tokens`
  (which now takes the app handle) and forgotten when Settings saves or previews another theme.
  The bar is built with `WebviewBuilder::background_color` and gets `Webview::set_background_color`
  whenever it paints another theme (tauri 2.12.4 has the runtime setter; smoke log
  `bar-background <scheme>`). **A deviation: the page webview keeps the engine's white.** The
  prompt's premise — the page's background "shows only before a page paints" — holds on no engine:
  a page without a CSS background of its own is drawn over the webview's color (WebView2's
  `DefaultBackgroundColor` docs say so; WebKitGTK's `webkit_web_view_set_background_color`; wry's
  WKWebView turns `drawsBackground` off, so such a page would show the window's grey), so a dark
  theme would have made plain pages — `text/plain`, Hacker News' margins, many simple sites —
  dark on dark. The page webview is hidden behind the command bar until a page loads, and every
  page paints its own background, so a theme color there would buy only a dark gutter beside a
  white page. Recorded under Deferred follow-ups.
- **Hysteresis** (`phone.rs`: `PHONE_BELOW` 600, `DESKTOP_FROM` 640; `wants_mobile(enabled, width,
  mode, current)`; phone_mode.rs passes the agent the page has): tests for a new window (600
  decides), the 600–640 band holding either agent, a 380 → 820 → 380 sweep at strides 1, 7 and 40
  switching exactly twice, hovering around 600 never flipping, and the site modes still winning at
  every width.
- **Smoke** (`scripts/ci/smoke-resize.mts` + test, wired after the fullscreen run on its driver):
  its own launch, a dark two-color theme, the window created 820×600 (the Windows runner's screen
  is 1024 px wide and a `--size` keeps the top-left, so a window centred at 480 px would have put
  820 px off the screen's edge); the steps and rules as the architecture's CI section says. A burst
  sends its 12 `--size` 30 ms apart without waiting, then the last width once more (waited): the
  second instances can land out of order.
- `npm run verify`: 506 TS tests, 296 Rust tests, clippy clean. Disk: 12 GiB free at the start of
  the phase, 11 GiB after verify; `.cache/target` 1.4 GB.
- **`lj-step-39-fix`:** Windows' verify (run 39160049052) failed one new test only — the check that
  the fallback colors are the start page's read `ui/style.css` with `\n` line ends, and Windows
  checks the file out with CRLF; the test now folds whitespace first.
- **CI for the fix (run 39160486225): all ten jobs green.** The resize run, every OS: `narrow`
  (820 → 380) one switch to phone and one reload; each 12-step burst one switch and one reload
  (380 → 820 desktop, 820 → 380 phone), 11 layouts per burst (the first width was the window's
  already), every `bounds-set` 2 on Windows and both Linux jobs, on macOS 9 × 2 and 2 × 0 going up
  (two `Resized` read a size the last layout had already applied) and 11 × 2 going down; no
  `page-visible` in any step; 620 coming from phone stayed phone, 660 switched to desktop with one
  reload, 620 coming from desktop stayed desktop; every final viewport W×560 with the right agent.
  macOS logged `autoresizing joan-bar 18 / joan-page 18` (Overlay) then `joan-bar 10 / joan-page 18`
  (Toolbar), `flipped=false follows=true`, and `bar-background dark`. The fullscreen run's rules
  still pass everywhere. **Screenshots:** `resize-820.png` / `resize-380.png` — Windows: the dark
  toolbar and the navy smoke page reach the window's right and bottom edges, no white strip;
  macOS: the same, the traffic lights inside the dark toolbar; Linux (scale 1 and 2): the same
  (the thin light line under the window is openbox's frame handle, not ours).

### LJ40 — Phone mode per site: a command, a toolbar pill, Windows' agent with client hints; the owner's check · strong · checkpoint

> Phase 19, last step. A **checkpoint**: the resizing feel needs the owner's eyes (Ground Rule 24).

**Verify:** `npm run verify` green, including:
- Rust tests: `set_site_mode(mode)` (bar capability; the host is the open web page's, taken by Rust
  — the bar sends only `auto | phone | desktop`) writes LJ36's `siteModes` for that host (`auto`
  removes a user entry, or stores `"auto"` when a built-in would otherwise apply), refuses
  without a web page, and applies at once: the agent changes and the page reloads only if the
  agent really changed — through `desktop_url` when going to desktop on a known mobile host;
  `site_mode_info()` returns `{ host, mode, builtIn, agent }` for the toolbar and the command bar;
- Windows: a pure builder for CDP's `Emulation.setUserAgentOverride` parameters — the phone agent
  with `userAgentMetadata` `{ platform: "Android", mobile: true, brands / fullVersionList from
  WebView2's own agent }`, the desktop agent with WebView2's own metadata — and the decision
  `skip_reload(css_content_width, viewport_width)` (the page already fits: no reload);
- vitest: the toolbar pill (`ui/toolbar/logic.ts`): shown while the page uses the phone agent or
  a site has its own mode ("Phone" / "Desktop"), a click toggles phone ↔ desktop for the site;
  the command bar's built-in rows "Request desktop site" / "Request phone site" / "Automatic for
  this site" with the current one marked; Settings → General's "Phone or desktop per site" list
  (each entry, built-in ones marked, Remove / Reset).

Capabilities: the bar gains only `allow-set-site-mode` and `allow-site-mode-info`. CI green on all
three: the smoke server records each request's `User-Agent`, `Sec-CH-UA-Mobile` and
`Sec-CH-UA-Platform` (into the run's folder) and the smoke page's title adds
`uamobile=<navigator.userAgentData?.mobile>`; on Windows the phone agent sends `Sec-CH-UA-Mobile:
?1` and platform `"Android"`, and `uamobile=true` — if CDP is refused with DevTools disabled or
doesn't persist across navigations, the notes say so and the SetUserAgent fallback stays (then the
rule is informational on Windows); at 390 px the command bar's "Request desktop site" (typed,
Windows / Linux; best effort on macOS) stores `"127.0.0.1": "desktop"` and the page reloads once
with `ua=desktop`; "Automatic for this site" brings `ua=phone` back. Screenshots `phone-pill.png`,
`settings-site-modes.png`.
**Human check:** the checklist in this step's prompt, copied into the PR.
**Commit:** `lj-step-40: phone mode per site — command, toolbar pill, Settings list; Windows' agent with client hints`

```text
Phone mode per site (the research's recommendation, matching what browsers that do it well do:
automatic by size, a quick toggle, memory per site, a reload only when the agent changes). LJ36
shipped the table (settings siteModes, phone::site_mode with the built-in WebKit entry for
youtube.com, desktop_url); this step gives it its UI and makes Windows' phone agent coherent.

Design (decided; do not relitigate):
- Commands (a child module src-tauri/src/commands/site.rs; build.rs's manifest; bar capability):
  site_mode_info() and set_site_mode(mode) as Verify says. Rust takes the host from the open web
  page (bar::page_url, keyed as LJ36's siteModes are: no www., no port), never from the bar;
  extension panels and the start page have no site mode.
- The toolbar pill (ui/toolbar/: logic.ts pure + tests, toolbar.ts, styles): a small "Phone" or
  "Desktop" label between the site and the right-hand buttons, shown only while the phone agent is
  in use or the site has its own mode (a desktop page in a wide window shows nothing — most of the
  time there is no pill); a click toggles phone ↔ desktop for this site. It must fit beside the
  site at 320 px (shorten the domain first; LJ15 later puts its status area there too).
- The command bar: built-in rows (ui/command/rows.ts builtInCommands, like History) "Request
  desktop site", "Request phone site", "Automatic for this site", offered while a web page is open
  and the typed text matches ("desktop", "phone", "mobile", "site"); the current mode is marked.
  LJ42 later folds them into the quick commands with their state.
- Settings → General: "Phone or desktop per site" — the user's entries and the built-in ones
  ("youtube.com — Desktop, built in on macOS and Linux"), each with Remove (or Reset for a
  built-in that was overridden); Save covers it (save_settings takes siteModes).
- Windows' phone agent with client hints. Today platform/win.rs sets the agent with
  ICoreWebView2Settings2::SetUserAgent, which "may clear" UA client hints; Chromium disables UA-CH
  for a custom agent without metadata, so sites get mixed signals. Instead call
  ICoreWebView2::CallDevToolsProtocolMethod("Emulation.setUserAgentOverride", <json>) with the
  metadata from Verify (webview2-com-sys 0.39.1 has no client-hints API of its own). Unverified,
  so CI must show it: whether it works while release builds call SetAreDevToolsEnabled(false) (wry
  0.57 webview2/mod.rs ~640) and whether it holds across navigations (if not, apply it again on
  each NavigationStarting). If it fails, keep SetUserAgent as the fallback, log
  `ua-override fallback <reason>`, and say so in the notes — don't enable DevTools to make it work.
- Windows only: before reloading for an agent change, ask CDP's Page.getLayoutMetrics; when the
  page's cssContentSize width already fits the viewport, skip the reload (`ua-reload skipped
  fits`). macOS and Linux would need a script in the page to measure it, which Ground Rule 10
  forbids: they reload as today (Later records the option for the owner).
- Sessions: cookies are shared across agents; per-site memory reduces agent flips, which matter
  for sessions bound to the agent (Landmines, round 2). Nothing more to do.

Smoke: the smoke server (scripts/ci/smoke.mts) writes each request's agent headers to a file in
the run's folder; the rules in scripts/ci/smoke-resize.mts (+ test) or a smoke-sitemode.mts (+
test); the typing with the existing helpers.

Docs: README (phone mode: per site, the pill, the command), docs/architecture.md (Page size and
phone mode; commands; capabilities), CHANGELOG ([Unreleased] → Added).

Do NOT measure pages with a script on macOS or Linux, enable DevTools, add a tablet mode, or show
the pill on the start page or a panel.

The owner's checklist (the PR's "Needs human check"):
- [ ] Retina Mac: drag the window's edges and corner slowly and fast, narrow and wide — the toolbar
      and the page follow the edge with no gap and no lag; in the dark theme no light strips.
- [ ] Windows laptop (if you have one): the same — no white strips while dragging.
- [ ] YouTube: desktop site by default (macOS); its fullscreen button, F and the exit button work;
      force "Request phone site" with the pill or the command: the phone site, and fullscreen as
      far as WebKit allows (record what happens) — then "Automatic for this site".
- [ ] Google, YouTube and Wikipedia: drag across 600 px both ways — one reload per crossing, no
      flip-flop between 600 and 640, the right version each time (Wikipedia's m. site becomes the
      desktop one).
- [ ] Signed-in sites stay signed in across the switches (Instagram, Google).
- [ ] The pill: shown in phone mode and for sites with their own mode, gone otherwise; a click
      toggles; Settings → General lists the sites, Remove works.
- [ ] The must-not-disturb list.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ40 to ✅ with the note
"waiting for the owner's check", add implementation notes (with df -h), and commit everything as
one commit: "lj-step-40: phone mode per site — command, toolbar pill, Settings list; Windows' agent
with client hints". Push and watch CI until every job is green (Ground Rule 3; two honest fix
attempts, then stop). Look at the screenshots on every OS. This ends phase 19 and is a checkpoint
(Ground Rule 24): fill the PR description from the template with the checklist above as "Needs
human check", merge the PR into main with a merge commit once every CI job is green, tell the owner
exactly what to check and where, then STOP — don't start phase 20 until the owner says go.
```

**Implementation notes (2026-10-18):**

- **Commands** (`src-tauri/src/commands/site.rs`; build.rs's manifest; `capabilities/bar.json`
  gains exactly `allow-site-mode-info` and `allow-set-site-mode`, and
  `capability_tests.rs`' `only_the_bar_may_read_and_set_site_modes` fails if any other capability
  grants them): `site_mode_info()` → `{ host, mode, builtIn, agent }`, `set_site_mode(mode)`. The
  site is always the open web page's (`bar::page_url`, web pages only — the start page and panels
  have none); the bar sends only the mode. The runtime is `phone_mode::{info, publish,
  set_site_mode}`; the decisions are pure in `phone/sites.rs` (moved out of `phone.rs`, which
  passed 400 lines): `with_site_mode` (phone / desktop kept; auto removes the entry, or keeps
  `"auto"` when a built-in or a parent's entry would still decide; refused with 500 sites, or
  without a site), `site_info`, `reaction` (stay, reload, or load the desktop address going to
  desktop on a known phone host). Rust pushes the info to the bar (`setSiteMode`) on every address
  change (`bar::publish_url`) and agent switch, and again when the bar's script runs.
- **A deviation, decided while building it: a site's phone address is the site.** The prompt
  says `set_site_mode` writes the entry "for that host". Kept literally, "Request desktop site" on
  `m.facebook.com` would hold for `m.facebook.com` alone: the site then sends the desktop agent to
  `www.`, where the width gives the phone agent again, and the page bounces between the two for
  ever (`RELOADED` stops only a reload of the same URL twice in a row). So `phone::mode_site` keys
  a mode under the desktop name (`desktop_url`'s known hosts; an `m.` / `mobile.` label in front of
  a name of two labels or more dropped), and a lookup asks the page's own host first, then that
  site — LJ36's entries and tests behave as before.
- **The pill** (`ui/toolbar/`: `siteModeInfo`, `pillFor` + tests): "Phone" while the phone agent
  is in use, "Desktop" while the site has its own mode (a built-in one counts — YouTube on macOS
  shows "Desktop", and the click is how to try its phone site); nothing for a desktop page that
  follows the width, the start page or a panel. Between the site and "open in default browser";
  the site's `max-width` and ellipsis let the domain shorten first at 320 px.
- **The command bar** (`ui/command/rows.ts` `siteModeRows`, part of `builtInCommands` so they
  always find room, like History): every typed word (two letters or more) must start one of a
  row's words — "Request desktop site" (request, desktop, site), "Request phone site" (… phone,
  mobile …), "Automatic for this site" (automatic, auto, site, desktop, phone, mobile) —, the
  site's mode now marked "current"; picking one calls `set_site_mode`, then closes the command bar.
- **Settings → General** (`ui/settings/sites-logic.ts` + test, `sites.ts`): the user's entries and
  this OS's built-ins (`get_settings`' `builtInSiteModes`: YouTube on macOS and Linux), "youtube.com
  — Desktop, built in on macOS and Linux" with Remove (which keeps `"auto"` for it), an overridden
  built-in with Reset, the user's entries with Remove. Save sends `siteModes` only when the list was
  edited (so a pill click while Settings is open survives its Save); `save_settings` checks it
  entry by entry.
- **Windows' agent** (`platform/win.rs`, `phone/agent.rs` — built on Windows and in tests): the
  phone agent through `ICoreWebView2::CallDevToolsProtocolMethod("Emulation.setUserAgentOverride")`
  with `userAgentMetadata` (Android, mobile, brands "Chromium" / "Google Chrome" — the agent string
  says Chrome — and a GREASE brand, full versions from `ICoreWebView2Environment::BrowserVersionString`)
  and `platform: "Linux armv8l"` for `navigator.platform`; desktop clears the override
  (`{ "userAgent": "" }`, as Playwright does — WebView2's own agent and metadata exactly, which
  reconstructing them couldn't promise). The settings' agent is never touched while the protocol
  works; a refusal falls back to `SetUserAgent` for good (`ua-override fallback <reason>`). The
  call waits off the main thread (≤ 2 s; the main thread is noted in setup and never waits) so the
  reload that follows goes out with the new agent. Before a resize's reload for the phone agent,
  `Page.getLayoutMetrics`: a page whose content fits the viewport is not reloaded (`ua-reload
  skipped fits`) — only going narrow; going wide always reloads, or a phone page would stay in a
  wide window. The smoke page therefore has a 600 px minimum width under a desktop agent (like most
  desktop pages; `?fits=1` drops it), or Windows would rightly skip its reloads and LJ39's
  one-reload-per-crossing rule would not see them.
- **Smoke** (`scripts/ci/smoke-sitemode.mts` + test; the smoke server's `agent-headers.jsonl`; the
  smoke page's `uamobile=`): as the architecture's CI section says.
- `npm run verify`: 535 TS tests, 311 Rust tests, clippy clean. Disk: 11 GiB free at the start of
  the step and after verify; `.cache/target` 1.4 GB.
- **CI (run 39162372528): all ten jobs green.** **The DevTools protocol works in release builds**
  (wry calls `SetAreDevToolsEnabled(false)`; `CallDevToolsProtocolMethod` still answers) **and the
  override holds across navigations** (it is set on the start page when the window is built, and
  the smoke page's first request — a navigation later — carried it). Windows: the phone agent's
  request `Mozilla/5.0 (Linux; Android 10; K) … Chrome/154.0.0.0 Mobile Safari/537.36`,
  `Sec-CH-UA-Mobile: ?1`, `Sec-CH-UA-Platform: "Android"`, `Sec-CH-UA: "Chromium";v="154",
  "Google Chrome";v="154", "Not_A Brand";v="99"`, the page `uamobile=true`; after "Request desktop
  site" (`{ "userAgent": "" }` cleared it) WebView2's own `… Edg/154.0.0.0`, `?0`, `"Windows"`,
  `"Microsoft Edge";v="154", "Not A(Brand";v="99", "Microsoft Edge WebView2";v="154",
  "Chromium";v="154"`, `uamobile=false`; "Automatic for this site" back to `?1` / `"Android"`. No
  `ua-override fallback` anywhere. `?fits=1` going to 390 px: `layout-metrics content=390
  viewport=390`, `ua-reload skipped fits`, no reload; the resize run's narrowing steps
  (`content=600 viewport=380`, the smoke page's desktop minimum width) reloaded as LJ39 requires.
  Windows, Linux and Linux at scale 2: "Request desktop site" logged `site-mode 127.0.0.1
  desktop`, settings.json held `"127.0.0.1": "desktop"`, one reload, `390x560 ua=desktop`;
  "Automatic for this site" `site-mode 127.0.0.1 auto`, the entry gone, one reload, `ua=phone`.
  Linux: no client hints at all (WebKit), `uamobile=undefined`; `?fits=1` reloaded. The resize and
  fullscreen runs' rules pass everywhere. **Screenshots:** `phone-pill.png` — the toolbar's
  "Phone" pill between the site and "open in default browser" on Windows, macOS (traffic lights
  inside the toolbar) and Linux; `settings-site-modes.png` — Windows: General lists "127.0.0.1 —
  Desktop" with Remove (the pill behind it now says "Desktop"); Linux: that and "youtube.com —
  Desktop, built in on macOS and Linux" with Remove (the tab now scrolls a little at the bottom).
- **macOS's run went wrong in the harness, not the app: `lj-step-40-fix`.** Cmd+L opened the
  command bar, then the typed `request desktop` quit the app (`quit` in the log right after
  `overlay true`): the synthetic key events, made from no event source, took the session's
  modifiers, and the synthetic Cmd+L had left Command down, so "q" was Cmd+Q — and with the app
  gone, the rest of the run's keys went to Finder (`settings-site-modes.png` caught Spotlight and a
  Finder window). The bundled run's `qr` page action has quit the app the same way on macOS (its
  log: `quit` after the typing), unnoticed because that step is best effort there. The fix: every
  synthetic key and mouse event on macOS carries explicit flags, and the shortcut script posts a
  Command key-up after Cmd+<letter>. The rules stayed informational on macOS (typing is best
  effort there), so the run was green.
- **CI for the fix (run 39164040097): all ten jobs green, and macOS types now.** macOS's site-mode
  run went through like the others — `request desktop` reached the command bar (`typed=15`), ↓ ↓
  Enter logged `site-mode 127.0.0.1 desktop`, one reload to `390x560 ua=desktop`, settings.json
  held the entry; `automatic` removed it and brought `ua=phone` back with one reload — so its rules
  applied and passed; `settings-site-modes.png` shows "127.0.0.1 — Desktop" and "youtube.com —
  Desktop, built in on macOS and Linux", each with Remove, the toolbar behind it saying "Desktop".
  Side effect: the bundled run's QR page action now opens on macOS too (`qr.png` shows the code for
  the smoke page), and the app no longer quits mid-run there.

**The owner's check (2026-10-18), on their Mac with `main` at 3200ea2:** YouTube in the narrow
window gets the desktop site; its fullscreen button, F, the exit button and Esc all work, and
leaving fullscreen puts Little Joan back exactly as it was (same size, same place). Dragging the
window's edges is smooth with no gaps, in light and dark. Crossing 600 px switches once each way;
the pill and "Request desktop site" work; double-clicking the toolbar zooms the window. Two notes
for later (Deferred follow-ups): with a non-default theme, half of the toolbar shows the default
colours for a split second while the window opens; and "Request desktop site" changes nothing on
sites that serve one responsive page to every agent. On the owner's word the state was released
as `v0.2.0-beta.3` (pre-release).

### LJ41 — Hide the tray / menu-bar icon (safely); Settings and Quit in the command bar · standard · auto

> Phase 20. Branch `lj/p20-control` from the latest `main`; the PR goes into `main`.

**Verify:** `npm run verify` green, including:
- Rust tests for a pure `tray::icon_visible(show_icon, shortcut_works) -> bool` (shown unless the
  setting is off **and** the shortcut works) and `shortcut_works(active_shortcut,
  wayland_no_shortcut)` (no registered shortcut → false; a Wayland session, where the X11 grab
  can't fire → false; suspended while Settings records one → still working);
- `settings.rs`: `showTrayIcon` (default true; a wrong type falls back);
- `cli.rs`: a second launch without flags still shows the window (`forwarded_command()` →
  `Show(None)`; add the test if none pins it);
- vitest: `rows.ts` `builtInCommands` offers **Settings** for "settings" / "preferences" /
  "options" and **Quit Little Joan** for "quit" / "exit" (two letters or more, like History), both
  always finding room among the 8 rows; `settings-logic.ts` change detection includes the switch.

Capabilities: the bar gains only `allow-quit-app`. CI green on all three with a **tray run**
(`scripts/ci/smoke-tray.mts`, + test, its own launch): with `"showTrayIcon": false` the log has
`tray-visible false` after `shortcut-registered`; a second launch without flags logs
`window-created`; typing "quit" in the command bar and Enter logs `quit` and the process ends by
itself (Windows, Linux; best effort on macOS); on Linux a launch with `XDG_SESSION_TYPE=wayland`
in its environment (it still runs on Xvfb's X11 but counts as a Wayland session, where the X11
shortcut can't fire) logs `tray-visible true` despite the setting. Screenshot
`settings-general-tray.png`.
**Commit:** `lj-step-41: hide the tray / menu-bar icon (it comes back when the shortcut can't work); Settings and Quit in the command bar`

```text
Some people prefer no icon (the owner's request). Hiding it must never strand anyone: with no
icon and no working shortcut, Little Joan would be unreachable.

Design (decided; do not relitigate):
- settings.rs: `showTrayIcon` (default true). Settings → General: "Show the icon in the menu bar"
  (macOS) / "Show the icon in the system tray" (Windows, Linux), with the note: "Without it, press
  <shortcut> or open Little Joan again from your apps. The icon comes back by itself whenever the
  shortcut can't work." (ui/settings.html, ui/settings.ts, ui/settings-logic.ts; Save covers it;
  save_settings takes it.)
- The rule (pure, in tray.rs or a small child module): visible = showTrayIcon ||
  !shortcut_works. shortcut_works is false when no shortcut is registered (AppState's
  active_shortcut is None: the saved one and the default both failed) and on a Linux Wayland
  session (welcome::wayland_no_shortcut_here(); LJ17 makes a portal-registered shortcut count as
  working). Suspension while Settings records a shortcut counts as working, so the icon doesn't
  blink.
- tray::apply_visibility(app), logging `tray-visible <bool>` once at startup and on each change:
  after shortcut::register_from_settings in lib.rs's setup, and after every save_settings (a
  swap that failed and restored the old shortcut still works; one that left no shortcut brings
  the icon back). Use the tray's own API (app.tray_by_id("main") → set_visible); on Linux that is
  the AppIndicator's status — read tray-icon 0.25's Linux set_visible in the cargo registry and
  note what it does.
- A second launch without flags already shows the window (single-instance → forwarded_command →
  Show(None)): keep it, and keep a test on it.
- Everything stays reachable without the icon: the command bar's built-in rows
  (ui/command/rows.ts builtInCommands, like HISTORY_ROW) gain **Settings** (Settings on General,
  through the existing open_settings) and **Quit Little Joan** (a new bar command `quit_app` →
  joan::quit; build.rs's app manifest, capabilities/bar.json `allow-quit-app`; in a new
  src-tauri/src/commands/app.rs child module). Symbols as CSS masks like the others: the gear,
  and a power glyph for Quit. "Settings & extensions" stays the last row.
- Hiding the icon creates nothing; hibernation is unchanged.

Smoke: the tray run from Verify (scripts/ci/smoke-tray.mts + test, its own LITTLEJOAN_DATA_DIR so
the main run's memory numbers stay put) and the General tab's screenshot with the switch off.

Docs: README (Using it: hiding the icon, and how to reach Little Joan without it),
docs/architecture.md (the tray, settings fields, the bar's commands and capability table),
CHANGELOG ([Unreleased] → Added).

Do NOT remove the tray menu's items, hide the icon by default, or add a Dock icon on macOS.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ41 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-41: hide the tray / menu-bar icon
(it comes back when the shortcut can't work); Settings and Quit in the command bar". Push, open the
phase-20 PR into main as a draft, and watch CI until every job is green (Ground Rule 3; two honest
fix attempts, then stop). Look at the screenshot on every OS.
```

**Implementation notes (2026-10-19):**

- **The rule** (`tray.rs`, pure and tested): `icon_visible(show_icon, shortcut_works)` = shown unless
  the setting is off and the shortcut works; `shortcut_works(active_shortcut, wayland_no_shortcut)`
  — no registered shortcut (`AppState::active_shortcut` `None`: the saved one and the default both
  failed) or a Linux Wayland session (`welcome::wayland_no_shortcut_here`) → false; a shortcut
  paused while Settings records one stays the active one, so it still counts (a test builds that
  state). `apply_visibility` reads the setting and the active shortcut, calls the tray's own
  `set_visible` only when the answer changed (an `AtomicU8`: nothing yet / shown / hidden — the
  icon is built visible), and logs `tray-visible <bool>` the first time and on each change. It
  runs after `shortcut::register_from_settings` in setup and after every `save_settings`.
- **What `set_visible` does** (tray-icon 0.25.1, read in the registry): macOS removes the
  `NSStatusItem` and builds it again from its kept attributes (icon, menu, tooltip) when shown;
  Windows sends the tray's hidden window `WM_USER_SHOW_TRAYICON`, which hides or shows the
  notification icon; **Linux sets the AppIndicator's status to `Passive` (hidden) or `Active`**.
  Tauri runs it on the main thread (`run_item_main_thread!`: inline on the main thread, else it
  waits for it), so it is never called with the state lock held. At startup the icon is created
  visible and hidden in the same main-thread turn of setup, before the event loop runs.
- **A swap that left no shortcut** (a small change to `shortcut::swap`, which the prompt's "one
  that left no shortcut brings the icon back" needs): its error is now `SwapError { message,
  old_kept }` — `old_kept` false when there was none, or the old one couldn't be registered again
  after the new one was refused (another app took it in that moment; a test). `save_settings`
  then forgets the active shortcut and applies the rule before returning the message, so the icon
  comes back; a failed swap that restored the old one changes nothing.
- **Settings:** `showTrayIcon` (default true; a wrong type falls back; tests). General: "Show the
  icon in the menu bar" (macOS) / "… in the system tray" (`trayLabelFor`), right under the shortcut,
  with the note from the prompt naming the shortcut as the recorder shows it (`trayText`, both
  tested); `toInput` sends `showTrayIcon` and `isChanged` counts it (tests); `save_settings` takes
  it as an optional field (left out, the saved one stays), like Developer mode.
- **The command bar:** `builtInCommands` gains **Settings** (`settings`, `preferences`, `options`:
  Settings on General through `open_settings`, the gear) and **Quit Little Joan** (`quit`, `exit`:
  the new `quit_app`, a power symbol as a CSS mask), matched like History (one shared `names`: the
  typed text starts a word, two letters or more, or a typed word is one); both always find room
  among the 8 rows (a test with 12 history matches and a quick answer). "Settings & extensions"
  stays the last row. `quit_app` (`src-tauri/src/commands/app.rs`) is `joan::quit`; build.rs's
  manifest and `capabilities/bar.json` gain exactly `allow-quit-app`, and
  `capability_tests.rs`' `only_the_bar_may_quit` fails if any other capability grants it.
- `cli.rs`' `a_plain_second_launch_shows_the_window` already pins `forwarded_command()` →
  `Show(None)` for a launch without flags: kept as it is.
- **Smoke** (`scripts/ci/smoke-tray.mts` + test, after the site-mode run): its own launches through
  `fullscreenLaunch` (which now takes extra environment variables). In the command bar "quit" is
  the second row (the search row comes first), so the run presses ↓ ↓ Enter as the site-mode run
  does; `command-quit.png` shows the typed command bar. The Settings screenshot is a relaunch of
  its own (`tray-settings`), because Settings open beside the page window could take the
  keyboard from the command bar's typing.
- `npm run verify`: 545 TS tests, 317 Rust tests, clippy clean. Disk: 10 GiB free at the start of
  the phase, 9.2 GiB after verify; `.cache/target` 1.6 GB.
- **CI (run 39210723268): all ten jobs green.** The tray run on every OS: `tray-visible false` right
  after `shortcut-registered` and never `true`; a plain second launch logged `window-created`;
  "quit" reached the command bar (macOS too), ↓ ↓ Enter logged `quit` and the process ended by
  itself; on both Linux jobs the `XDG_SESSION_TYPE=wayland` launch logged `tray-visible true`.
  **Screenshots:** `settings-general-tray.png` — General with "Show the icon in the menu bar"
  (macOS) / "… in the system tray" (Windows, Linux) unchecked under the shortcut, and the note
  naming ⌘⇧Space / Ctrl+Shift+Space (on macOS the menu bar shows no Little Joan icon);
  `command-quit.png` — "Search Google for "quit"", then "Quit Little Joan" with the power glyph,
  then "Settings & extensions", on every OS (Linux at scale 2 crisp).

### LJ42 — Quick commands in the command bar, each showing its state · standard · auto

**Verify:** `npm run verify` green, including:
- vitest tests for a pure `ui/command/quick.ts`: matching (every typed word starts a word of the
  title or a keyword; two letters or more); the page commands (Translate page, Open in browser,
  Copy link, Copy as Markdown link, Reload) only while a web page is open; each row's state as text
  ("Mobile sites — on", "Theme: Dark ✓", "Search with DuckDuckGo ✓", "Start at login — off",
  "Icon — shown (the shortcut can't work)"); "Clear history" offers its four ranges and needs a
  second Enter on the same row (Esc or moving away cancels); quick commands share the 8 rows like
  the built-ins (up to 3 always find room; history matches make way);
- vitest for `settings-logic.ts` `rebase(draft, oldSaved, newSaved)`: a field the open Settings
  window didn't edit follows a change made from the command bar; an edited one keeps the edit;
- Rust tests for `quick_command`'s input (a serde enum, `deny_unknown_fields`, fixed kinds — anything
  else is refused) and for `settings::apply_change` (shared with `save_settings`: a bad engine id,
  an unknown theme mode or a bad color changes nothing);
- the Markdown text comes from `extensions/markdown-link/logic.js`'s `link("markdown", …)`, imported
  by the bar (one implementation).

Capabilities: the bar gains only `allow-quick-state` and `allow-quick-command`. CI green on all
three with a **quick-commands run** (`scripts/ci/smoke-quick.mts`, + test): type `mobile` →
Enter → `quick-command setMobileSites false` and settings.json `"mobileSites": false`; type `dark`
→ Enter → `theme dark colors=<n>` and the saved theme's mode `dark`; over the smoke page, type
`copy link` → Enter → the clipboard holds its URL where the runner can read it (informational, as
for quick answers). Windows and Linux required, macOS best effort. Screenshots `quick-theme.png`
(typing "theme") and `quick-page.png` (typing "copy" over the smoke page).
**Commit:** `lj-step-42: quick commands in the command bar, each showing its current state`

```text
The owner: more command-bar commands for actions and settings, each showing its current state.
The list (decided): Translate page, Open in browser, Copy link, Copy as Markdown link, Reload;
the theme presets and Theme: System / Light / Dark; Search engine (one row per engine); Mobile
sites on / off; Remember history on / off; Clear history (last hour, 24 hours, 7 days,
everything); Start at login on / off; Settings → <each tab>; Reload extensions; Show / hide the
icon (LJ41); Quit (LJ41's row); and LJ40's "Request desktop site" / "Request phone site" /
"Automatic for this site", which move from the built-in rows into the quick commands with their
state ("Desktop site — this site ✓").

Design (decided; do not relitigate):
- Rust, a new child module src-tauri/src/commands/quick.rs:
  - quick_state() → { page: { url, title } | null (a web page only; title = the window's title),
    theme, searchEngine, engines: [{ id, name }], mobileSites, rememberHistory, startAtLogin: bool
    | null, showTrayIcon, iconForced (shown although the setting is off, LJ41) }.
  - quick_command(command): a serde enum, #[serde(tag = "kind", deny_unknown_fields)]:
    translate, openInBrowser, reload (the existing page actions); setThemeMode { mode }; setTheme
    { theme } (a preset's whole theme, built by the bar from ui/lib/presets.ts and checked exactly
    like save_settings' theme); setSearchEngine { id }; setMobileSites { on };
    setRememberHistory { on }; clearHistory { range } (history_store's clear, as history_clear);
    setStartAtLogin { on } (autostart::set); openSettings { tab } (settings_window::open);
    reloadExtensions; setTrayIcon { on }. It logs `quick-command <kind> [<value>]`, never a URL
    or a title.
  - Settings changes share one path with save_settings: factor settings::apply_change(app,
    change) out of commands.rs's save_settings (validate, update AppState, atomic save, push a new
    theme to the bar and the settings window, re-run LJ41's tray rule, phone_mode::apply when
    mobileSites changed); save_settings calls it too, so the two can't drift.
  - Settings only, never from the command bar: the global shortcut and keys, developer mode,
    switching extensions, custom theme colors. The bar is trusted (Ground Rule 18), but these
    change what code runs or what keys do, and keep the Settings window's Save.
- If the settings window is open when a quick command changes a setting, Rust tells it
  (window.littleJoanSettings.savedChanged(<settings json>), by eval like showTab), and
  settings.ts rebases its draft (rebase, Verify), so a later Save doesn't undo the change.
- The bar (ui/command/quick.ts pure + tests; the glue in ui/command/command.ts): quick_state when
  the command bar opens (alongside extensions_list); rows show their state as text after the
  title; Enter runs the command, and the rows refresh from a new quick_state (settings keep the
  command bar open; page commands close it as they do today). Copy link and Copy as Markdown link
  copy in the bar with ui/command/copy.ts on the Enter keypress; the Markdown text is
  link("markdown", title, url) imported from extensions/markdown-link/logic.js (plain ES, already
  type-checked through tsconfig's checkJs) — if Vite or tsc can't take the import, stop and
  report; don't copy the escaping. Rows draw with textContent only (view.ts).
- Clearing history asks a second Enter on the same row (its subtitle says what will go); nothing
  else asks twice.

Smoke: scripts/ci/smoke-quick.mts + test, its own launch on a fresh data folder, with the smoke
page for the page commands; type with the existing helpers (typeText, pressKeys) and read
settings.json after each settings command.

Docs: README (the command bar: quick commands), docs/architecture.md (commands table,
capabilities), CHANGELOG ([Unreleased] → Added).

Do NOT add commands that change keys, the shortcut or extensions, or let a page's title or URL
choose a command.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ42 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-42: quick commands in the command
bar, each showing its current state". Push and watch CI until every job is green (Ground Rule 3;
two honest fix attempts, then stop). Look at both screenshots on every OS.
```

**Implementation notes (2026-10-19):**

- **One path for settings changes — split in two (a deviation in shape, not in substance):** the
  prompt's `settings::apply_change(app, change)` would put Tauri types into `settings.rs`, which is
  pure (Ground Rule 14). So `settings::apply_change(&Settings, &Change) -> Result<Settings, String>`
  (`settings/change.rs`, pure) checks every part of a change — a search engine id, a language, a
  whole theme (new `theme::parse_strict`: a known mode, 1–3 colors that all parse, numbers in 0–1),
  a theme mode, the extensions' and sites' lists (entry by entry, as before), the bools — and
  refuses the whole change for one bad part (tests: a bad engine id, an unknown mode, a bad color,
  a bad language, an empty shortcut change nothing). `settings_live::apply(app, &change)` puts it to
  work under the state lock (checked again against that moment's settings), ends a preview when the
  theme changed, pushes a new theme to the bar (smoke log `theme <mode> colors=<n>` — `theme` now
  also means "handed a new one"), re-runs LJ41's tray rule, forgets the registry when extensions
  changed, runs `phone_mode::apply` when `mobileSites` or `siteModes` changed, and saves atomically.
  `save_settings` checks the whole change first (so a refused part never swaps the shortcut), swaps
  the shortcut, then calls the same function; its `searchEngine` is now an id string checked there,
  and its theme is checked strictly (the Settings window only ever sends a valid one).
- **`quick_state` / `quick_command`** (`src-tauri/src/commands/quick.rs`; build.rs's manifest;
  `capabilities/bar.json` gains exactly `allow-quick-state` and `allow-quick-command`;
  `capability_tests.rs`' `only_the_bar_may_run_quick_commands`, through a shared
  `only_the_bar_grants` that `only_the_bar_may_quit` uses too). The command is a serde enum
  `#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]`; **the field-less kinds are
  empty structs (`Reload {}`), not unit variants** — a test showed serde's internally tagged unit
  variants accept extra fields despite `deny_unknown_fields`. Tests refuse other kinds (`setShortcut`,
  `setDeveloperMode`, `setExtension`, `setKeys`), extra fields, wrong types and missing ones. Page
  commands are refused without a web page (`commands::web_page`, now shared with `open_extension`).
  It logs `quick-command <kind> [<value>]` once accepted — values filtered to short words, never a URL
  or a title (a test). After a settings command an open Settings window gets
  `window.littleJoanSettings.savedChanged(<settings>, <startAtLogin | null>)`.
- **The bar** (`ui/command/quick.ts`, pure + 13 tests; glue in `command.ts`): the list from the prompt,
  each row's state after the title as text — "Mobile sites — on", "Theme: Dark ✓" (a new `mark` on a
  row, drawn as text by `view.ts`, in the accent color), "Search with DuckDuckGo ✓", "Start at login —
  off", "Icon — shown (the shortcut can't work)"; a preset row sends the preset's whole theme with the
  user's mode, and is ✓ when the theme matches it. LJ40's three rows moved in as "Desktop site",
  "Phone site", "Automatic", each "— this site", the current ✓ (the prompt's "Desktop site — this site
  ✓"; "request" and "automatic" still find them, so LJ40's smoke types the same). Settings → <tab>
  covers Appearance, Extensions, Privacy and About: **General is LJ41's "Settings" row**, so it isn't
  offered twice. Clear history offers its four ranges; the first Enter turns the row into "press
  Enter again to remove the history of the last hour" (`quickEnter`, tested), a second Enter on that
  row runs it and says what went; Esc (`command.cancel()`, before the bar's own Esc), moving away or
  typing cancels. Up to three quick rows always find room among the 8 (`QUICK_ROOM` in `buildRows`;
  history matches make way); quick rows are never preselected, so Enter on typed text still opens it
  — the smoke presses ↓ ↓ Enter as for LJ40's rows (the prompt's "type `mobile` → Enter" taken as that).
  Copy link / Copy as Markdown link copy in the bar on the keypress (`copy.ts`), the Markdown from
  `extensions/markdown-link/logic.js`'s `link("markdown", …)` imported by the bar (Vite and tsc took
  it as is); page commands then close the command bar, settings ones keep it open and the rows show
  the new state from a fresh `quick_state` (asked when the command bar opens, with `extensions_list`,
  and again after each command; rows are built again if it lands after the typing).
- **Settings follows** (`ui/settings-logic.ts` `rebase(draft, oldSaved, newSaved)`, tested): a field
  the window didn't edit follows the command bar's change, an edited one keeps the edit (the theme
  compared deeply, lists in any order); `settings.ts`' `savedChanged` applies it to the controls, the
  Appearance editor, the Extensions tab and the site list (each only when not edited) and the login
  switch.
- `npm run verify`: 561 TS tests, 327 Rust tests, clippy clean. Disk: 9.2 GiB free at the start of
  the step, 8.9 GiB after verify; `.cache/target` 1.6 GB.
- **CI (run 39212735555): all ten jobs green.** The quick-commands run, every OS (macOS typed too):
  "mobile" logged `quick-command setMobileSites false` and saved `"mobileSites": false`; "dark"
  logged `quick-command setThemeMode dark`, handed the bar `theme dark colors=1` and saved the mode;
  Copy link put the smoke page's address on the clipboard on Windows and macOS (Linux's runner has
  no clipboard tool: informational). LJ40's site-mode run, typing the same words, passed with the
  renamed rows. **Screenshots:** `quick-theme.png` — after "dark", the dark command bar with Theme:
  System / Light / Dark ✓ / Mist ✓ / Sand / Sage / Lagoon (the ✓ in the accent colour), on every OS;
  `quick-page.png` — over the smoke page, "Copy link" and "Copy as Markdown link" with the copy
  symbol, then the bundled Clean link and Copy as Markdown link page actions (Deferred follow-ups:
  the quick row and the page action of the same name both show).

### LJ43 — One keymap: every key of Little Joan's own, configurable (engine side) · strong · auto

**Verify:** `npm run verify` green, including Rust tests for a pure `src-tauri/src/keymap.rs` (tests
in `keymap/tests.rs`):
- the actions' default bindings per OS reproduce today's keys exactly (a table test — Windows /
  Linux: Ctrl+W, Ctrl+L, Ctrl+Shift+T, Ctrl+H, and on Linux also Ctrl+R, F5, Alt+← / Alt+→;
  macOS: Cmd+W, Cmd+L, Cmd+Shift+T, Cmd+Shift+O, Cmd+R, Cmd+[, Cmd+], Cmd+Y);
- a binding is an accelerator string with a `KeyboardEvent.code` key (`CommandOrControl+KeyL`,
  `Alt+ArrowLeft`, `F5`), `CommandOrControl` resolved per OS; matching stays exact (Ctrl+Shift+L is
  not Ctrl+L; Ctrl+Alt never matches on Windows and Linux);
- settings' `keys` overrides (`{ "<action-id>": "<accelerator>" | "" }`): unknown ids and
  unparsable accelerators are dropped, `""` means no key, and an override replaces the action's
  first binding and drops its fixed aliases (Linux reload's F5 stays only while reload has its
  default);
- validation: two actions on one binding → refused; an action on the global shortcut → refused; a
  typing key (no modifier and not an F-key, or Shift alone with a letter or digit) → refused; on
  macOS a binding needs Cmd or Ctrl;
- every default, and every binding the recorder can produce, parses as a menu accelerator (the
  parser menu.rs's items use);
- `Key` covers what the recorder records — letters, digits, `Equal`, `Minus`, `BracketLeft` /
  `BracketRight`, `Comma`, `Period`, `Slash`, `Semicolon`, `Quote`, `Backquote`, `Backslash`,
  F1–F12, arrows, Home / End / PageUp / PageDown, numpad keys — from Windows virtual keys and GDK
  key names.

The settings round trip keeps `keys`. CI green on all three with a **keymap run**
(`scripts/ci/smoke-keymap.mts`, + test): settings.json `"keys": { "history":
"CommandOrControl+Shift+KeyK" }` → Ctrl+Shift+K (SendKeys `^+k`, xdotool `ctrl+shift+k`) opens the
history view and Ctrl+H doesn't; on macOS `menu-keys history=⌘⇧K` in the log shows the menu was
built from the keymap (the key press itself best effort).
**Commit:** `lj-step-43: one keymap for every Little Joan key, configurable in settings (engine side)`

```text
The owner: change Little Joan's own keys too, not just the global shortcut. This step builds the
engine side — one keymap every native key path reads; LJ44 builds the Settings tab. Ground Rule 25
applies from here on.

Today (read it): keys.rs's action_for hard-codes the Windows / Linux keys; platform/win.rs
(AcceleratorKeyPressed on both webviews) and platform/linux.rs (the GTK window's
key-press-event) call it; menu.rs hard-codes macOS's key equivalents in the hidden menu bar.

Design (decided; do not relitigate):
- src-tauri/src/keymap.rs (pure, no Tauri types; tests in keymap/tests.rs): KeyAction — close,
  open-location, reload, back, forward, translate, history, open-in-browser (ids are these
  strings; LJ16 adds zoom-in / zoom-out / zoom-reset, LJ49 find). Per OS an ordered list of
  default bindings: the first is what Settings shows and lets you change, the rest are fixed
  aliases; an action may have none (open-in-browser on Windows and Linux).
  Keymap::new(platform, overrides); action_for(press) -> Option<KeyAction>; binding(action) for
  the menu and for display; validate(overrides, global_shortcut) -> Result<(), Vec<Problem>>.
- Bindings use the accelerator syntax the global shortcut already uses (shortcut.rs,
  ui/lib/shortcut.ts: "CommandOrControl+Shift+Space"), keys named by KeyboardEvent.code, so the
  recorder and the display text are shared. Store CommandOrControl, never the resolved modifier
  (Landmines).
- Not in the keymap (fixed): Esc / Enter / arrows / Tab in the bar, the Edit menu, Cmd+Q and
  Cmd+, on macOS. On Windows, WebView2's own browser keys (Ctrl+R, F5, Alt+← / Alt+→) keep working
  beside a custom binding — WebView2 handles them itself, and turning them all off
  (AreBrowserAcceleratorKeysEnabled) would take other keys too, so don't; the docs say so.
- settings.rs: `keys` (a map of overrides, tolerant per entry). save_settings takes it (optional,
  like disabledExtensions) and refuses a set that fails validate, with the first problem's
  message. AppState holds the built Keymap; a save rebuilds it.
- keys.rs keeps Key, Mods, Platform and the conversions from Windows virtual keys and GDK key
  names (extended as Verify lists); action_for moves into keymap.rs (keys.rs re-exports what
  callers need, or the callers change — keep the diff readable). PageAction stays what
  page_actions.rs performs; KeyAction → PageAction is mapped in one place.
- macOS: menu.rs takes the page items' accelerators from the keymap (an action without a key gets
  none), and a save sets the menu again (app.set_menu(menu::create(..)), on the main thread). In
  smoke mode it logs `menu-keys <action>=<display>` for each item once the menu is built.
- keymap_info() (bar and settings capabilities; a child module commands/keys.rs): { platform,
  actions: [{ id, title, binding, defaults }], globalShortcut } — titles as the menu has them.
  LJ44 uses it.

Smoke: scripts/ci/smoke-keymap.mts + test, its own launch: the Verify's sequence.

Docs: docs/architecture.md (the page keys table becomes the default keys, plus the keymap),
CHANGELOG ([Unreleased] → Added: the keys can be changed in settings.json; LJ44 adds the tab).

Do NOT build the Keyboard tab (LJ44), change any default key, or make the bar's own keys
configurable.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ43 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-43: one keymap for every Little
Joan key, configurable in settings (engine side)". Push and watch CI until every job is green
(Ground Rule 3; two honest fix attempts, then stop).
```

**Implementation notes (2026-10-19):**

- **`keymap.rs`** (pure; 11 tests in `keymap/tests.rs`): `KeyAction` (ids `open-location`,
  `history`, `translate`, `reload`, `back`, `forward`, `open-in-browser`, `close`; titles as the
  menu has them; `page_action()` — the one place an action meets a `PageAction`), `Binding` (an
  accelerator's modifiers unresolved — `CommandOrControl`, `Control`, `Super`, `Alt`, `Shift` —
  and a `Key`; `parse` takes the global shortcut's names in any case and refuses a repeated or
  unknown modifier; `accelerator()` writes `CommandOrControl…` back, never the resolved modifier;
  `mods(platform)` resolves; `display` as Settings shows it), `defaults(platform, action)` (the
  table test: today's keys exactly), `Keymap::new(platform, overrides)`, `action_for(key, mods)`
  (exact), `binding`/`bindings`, `bindable`, `validate(platform, overrides, global_shortcut)` with
  `Problem::message`, and `info()` for `keymap_info`.
- **`keys.rs`** keeps `Key`, `Mods`, `Platform`, `PageAction` and the origin; `action_for` moved to
  the keymap, its tests with it. `Key` now covers what the recorder records — letters, digits,
  F1–F24, arrows, Home / End / PageUp / PageDown, `Equal`, `Minus`, the brackets, `Comma`,
  `Period`, `Slash`, `Semicolon`, `Quote`, `Backquote`, `Backslash`, the keypad's digits and
  operators — from `KeyboardEvent.code` names (`from_code` / `code`, round-trip tested), Windows
  virtual keys (`VK_OEM_*` as on a US keyboard) and GDK key names (with Shift held GDK names the
  shifted character — `braceleft`, `exclam` — counted as its key on a US layout). Keys match by what
  they type, as before: on AZERTY Ctrl+W is still the key that types W (Windows' virtual keys,
  GDK's names and macOS's key equivalents all work that way).
- **Typing keys, a little wider than the prompt's words:** "Shift alone with a letter or digit" is
  refused for every key that types (punctuation and the keypad too: Shift+= types "+"); Shift with
  an arrow, Home/End/Page or an F-key is allowed (they type nothing).
- **Settings:** `keys` (`settings/keys.rs`: unknown ids and unparsable accelerators dropped alone,
  kept in the keymap's spelling; the round-trip test keeps them). A typing key from a hand-edited
  file is never bound (`Keymap::new` ignores an override that isn't `bindable`; the default stays).
  `save_settings` takes `keys` (optional, like `disabledExtensions`), through LJ42's shared change:
  `settings::apply_change` refuses a set that fails `keymap::validate` with the first problem's
  message — and also **a new global shortcut that is one of the keys** (validated whenever the
  shortcut or the keys change). `AppState::keymap` is built at startup and again by
  `settings_live::apply`.
- **The hooks:** Windows' `AcceleratorKeyPressed` (both webviews) and Linux's `key-press-event`
  ask `AppState::keymap` (a lock per press, held only for the lookup). **macOS:** `menu.rs` builds
  the page items from the keymap (an action without a key gets none) and logs `menu-keys
  <action>=<key | none>` in smoke mode; `settings_live` sets the menu again on the main thread
  when the keys changed (`menu::refresh`). **Close Window is now Little Joan's own item** (the
  standard `close_window` item's Cmd+W was fixed): its key comes from the keymap, and with Settings
  in front it closes Settings, as the standard item did. `setup` now builds the state before the
  menu (the menu needs the keymap).
- **"Parses as a menu accelerator":** Tauri's menu items parse with muda and silently drop an
  accelerator that fails, so the test matters — but muda isn't a direct dependency (adding one is a
  stop condition). The test parses every default and every binding the recorder can produce (each
  key with five modifier sets) with the global shortcut's parser (global-hotkey 0.8.0), whose
  modifier and key names are the same as muda 0.20.0's for everything `accelerator()` writes
  (checked in both crates' sources: muda additionally accepts `Meta` and falls back to
  `keyboard_types::Code`'s names), and checks each round-trips through `Binding::parse`.
- **`keymap_info`** (`src-tauri/src/commands/keys.rs`): `{ platform, actions: [{ id, title,
  binding, defaults }], globalShortcut }`; `allow-keymap-info` in `capabilities/bar.json` and
  `capabilities/settings.json`, and `capability_tests.rs`' `only_the_bar_and_settings_may_read_the_keymap`.
- **Ground Rule 25 check:** a search of `ui/` and `src-tauri/src/` for key strings finds only the
  global shortcut's default, macOS's fixed Settings… (Cmd+,) and Quit (Cmd+Q), the recorder's
  modifier names and tests.
- **Smoke** (`scripts/ci/smoke-keymap.mts` + test, after the quick-commands run): `pressShortcut`
  learned Shift (SendKeys `^+k`, xdotool `ctrl+shift+k`, the Swift script's Command + Shift flags).
- `npm run verify`: 565 TS tests, 338 Rust tests, clippy clean. Disk: 8.9 GiB free at the start of
  the step, 8.8 GiB after verify; `.cache/target` 1.6 GB.
- **CI (run 39214438496): all ten jobs green.** The keymap run, every OS: Ctrl+Shift+K (macOS ⌘⇧K,
  through the menu) opened the history view and the old key (Ctrl+H; ⌘Y) then opened nothing;
  macOS logged `menu-keys history=⌘⇧K` and `open-location=⌘L`. The main run's default history key
  and LJ40's site-mode and LJ42's quick runs passed as before.

### LJ44 — Settings → Keyboard: record every key; clashes and the system's reserved shortcuts · standard · auto

**Verify:** `npm run verify` green, including vitest tests for:
- `ui/settings/keyboard-logic.ts`: recording (the next key press with a modifier, or an F-key; Esc
  cancels; Backspace means no key); the same rules as Rust's validation (duplicate → error, the
  global shortcut → error, typing key → refused, macOS without Cmd / Ctrl → refused, Ctrl+Alt on
  Windows / Linux → warning); reserved-shortcut warnings; Reset per row and Reset all; change
  detection with the other tabs;
- `ui/settings/reserved-shortcuts.ts` (data): every entry parses, has a name and an OS, no
  duplicates per OS; it includes at least macOS ⌘Space (Spotlight), ⌃Space and ⌃⌥Space (input
  sources), ⌘⇧3 / ⌘⇧4 / ⌘⇧5 (screenshots), ⌘Tab, ⌘\`, ⌃⌘Q, ⌃⌘Space (emoji), ⌘⌥Esc, ⌘H, ⌘M; Windows
  Alt+Tab, Alt+F4, Alt+Space, Ctrl+Esc, Ctrl+Shift+Esc, Win+<any key> (one wildcard entry),
  Ctrl+Win+← / →, PrintScreen; Linux (GNOME, KDE, Ubuntu) Super and Super+<any key>, Alt+Tab,
  Alt+F2, Alt+F4, Alt+Space (KDE's KRunner), Ctrl+Alt+T, Ctrl+Alt+arrows, Super+Space; and on every
  OS the editing keys (Ctrl / Cmd + C, V, X, A, Z, Shift+Z) as errors ("used for editing in
  pages");
- Rust: `SettingsTab::Keyboard` (by name, any case), `--settings keyboard`; get_settings /
  keymap_info report `shortcutProblem` (`{ wanted, active, reason: taken | invalid }`) when the
  saved global shortcut couldn't be registered at launch; `shortcut::TAKEN` reads "Another app or
  the system already uses that shortcut. Try another one."

CI green on all three: `--settings keyboard` opens the tab (`settings-window keyboard`), screenshot
`settings-keyboard.png`; a short launch whose settings.json holds an unparsable shortcut shows the
problem line (`settings-keyboard-problem.png`; Little Joan fell back to the default).
**Commit:** `lj-step-44: Settings → Keyboard: every key recordable, clashes and the system's reserved shortcuts`

```text
The Settings side of LJ43's keymap, plus what the owner asked for the global shortcut: warn about
clashes with Little Joan's own keys and with the system's, and say plainly when another app holds
the shortcut.

Design (decided; do not relitigate):
- A Keyboard tab (settings_window.rs SettingsTab::Keyboard, between General and Appearance;
  ui/settings.html, ui/settings.ts, new ui/settings/keyboard.ts glue + keyboard-logic.ts pure +
  keyboard.css; `--settings keyboard` in cli.rs). Rows: "Summon Little Joan" — the global
  shortcut; its recorder moves here from General, with its suspend / resume behaviour unchanged,
  and General keeps a line with the current shortcut and a "Change…" button that switches to this
  tab — then every keymap action (keymap_info) with its key as a recorder button, "Reset" when it
  isn't the default, and "No key"; "Reset all" at the bottom. Save covers the tab (keys to
  save_settings, which validates again).
- Problems: errors block Save and say why ("⌘L is already Open Location", "That's the shortcut
  that summons Little Joan", "Plain keys type text — add ⌘ or ⌃"); warnings don't ("macOS uses
  ⌘Space for Spotlight", "Ctrl+Alt is AltGr on many keyboards"). The global shortcut's recorder
  gets the same rules (an app key equal to it → error, a reserved one → warning).
- The reserved table is curated data (ui/settings/reserved-shortcuts.ts: { os, accelerator or a
  modifier wildcard, name }): the entries in Verify plus what you can confirm in the OSes' own
  documentation. No network at runtime.
- macOS system shortcuts, read-only — decide here, by evidence: macOS keeps the user's system
  shortcuts in the com.apple.symbolichotkeys preferences domain (AppleSymbolicHotKeys: id → {
  enabled, value: { parameters: [character, virtual key code, modifier flags] } }). Read it with
  NSUserDefaults persistentDomainForName: through objc2 msg_send in platform/mac.rs (no new
  crate), map virtual key codes (US layout) and modifier flags to accelerators, and test the
  parser on a CI runner's export (a one-off step of a workflow_dispatch run: `defaults export
  com.apple.symbolichotkeys -`, saved as test data — a runner holds nothing personal). Ship it only
  if it is reliable: the domain lists only entries that were written, so defaults such as
  Spotlight's may be absent — the curated table stays the base, a read entry marked disabled drops
  that curated warning, and an enabled one adds a warning. If the export is empty or the mapping
  isn't trustworthy, don't ship the read, and record why under Later. Never write a system setting
  (Ground Rule 21).
- The global shortcut's failure: shortcut::register_from_settings keeps what failed
  (AppState::shortcut_problem: the wanted accelerator, the active one or none, taken | invalid);
  get_settings and keymap_info report it; Keyboard and General show "⌘⇧K couldn't be registered:
  another app or the system already uses it. Little Joan is using ⌘⇧Space instead." (or "…Little
  Joan has no shortcut now."). shortcut::TAKEN changes as Verify says.
- On a Linux Wayland session the Summon row explains `little-joan --toggle` as the welcome does
  (LJ17 later shows the portal's trigger there instead).

Smoke: the Verify's two screenshots (the history-off run's Settings sequence opens Keyboard after
Appearance; a short launch of its own for the problem line).

Docs: README (Settings → Keyboard; the page keys table says the keys can be changed),
docs/architecture.md (the settings tabs table, the CLI), CHANGELOG ([Unreleased] → Added).

Do NOT change a default key, write any OS setting, or block Save on a warning.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ44 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-44: Settings → Keyboard: every
key recordable, clashes and the system's reserved shortcuts". Push and watch CI until every job is
green (Ground Rule 3; two honest fix attempts, then stop). Look at both screenshots on every OS.
This ends phase 20. Fill the PR description from the template. Needs human check:
- Turn the icon off, quit, launch Little Joan again: the window opens; the shortcut works.
- Settings → Keyboard: set the global shortcut to ⌘Space — the Spotlight warning shows; set History
  to ⌘⇧K, Save: ⌘⇧K opens history and ⌘Y doesn't; Reset all.
- Quick commands: Theme: Dark / Light, Mobile sites, Copy link, Copy as Markdown link, Clear
  history (asks twice), Quit.
- The must-not-disturb list.
Once verify, package and smoke are green on every OS job, merge the PR into main with a merge
commit, then stop and report.
```

**Implementation notes (2026-10-19):**

- **Rust:** `SettingsTab::Keyboard` between General and Appearance (`--settings keyboard`, any
  case; tests). `shortcut::settle(registrar, wanted)` (pure, tested with the fake registrar):
  registers the saved shortcut, else the default, and returns what went wrong — `Problem { wanted,
  active, reason: taken | invalid }` (`invalid` when it doesn't parse) —, kept as
  `AppState::shortcut_problem` until a Save registers a shortcut; `get_settings` and `keymap_info`
  report it as `shortcutProblem`, and `get_settings` also `waylandNoShortcut`. `shortcut::TAKEN`
  reads "Another app or the system already uses that shortcut. Try another one." (a test).
- **The tab** (`ui/settings/keyboard.ts` glue, `keyboard-logic.ts` pure + 13 tests,
  `reserved-shortcuts.ts` data + 2 tests, `keyboard.css`): "Summon Little Joan" — the global
  shortcut's recorder moved here from General with its suspend / resume unchanged (General keeps a
  line with the shortcut and **Change…**, which switches here) —, then every keymap action from
  `keymap_info` as a recorder button with **No key** and **Reset**, its problems under it as text,
  **Reset all** at the bottom (the keys and the summon shortcut). Recording takes the next key with
  a modifier (Shift counts, then the rules judge it) or an F-key; Esc cancels; Backspace means no
  key; **a letter is the one the key types in the user's layout** (`KeyboardEvent.key`, so the
  recorded `KeyA` is what Windows' virtual keys, GDK's names and macOS's key equivalents match —
  `code` alone would record AZERTY's A as `KeyQ`), other keys by `code`. The recorder buttons are
  updated in place while listening (rebuilding the rows would have dropped the focus and its blur
  rule). The rules mirror `keymap::validate` — duplicates (aliases included), the summon shortcut,
  typing keys, macOS without ⌘ or ⌃ — as errors that disable Save; the editing keys (Ctrl/Cmd + C,
  V, X, A, Z, Shift+Z) are errors on every OS ("used for editing in pages"); the OSes' reserved
  shortcuts and Ctrl+Alt on Windows / Linux are warnings. The summon recorder gets the same: one of
  Little Joan's keys → error, a reserved one → warning. Save sends `keys` only when edited
  (`isChanged` and `rebase` know them).
- **The reserved table** is curated from the OSes' documentation (Apple's Mac keyboard shortcuts,
  Microsoft's Windows keyboard shortcuts, GNOME's / KDE's / Ubuntu's defaults): the Verify's
  entries plus ⌘⌥Space, ⌘⇧Q, ⌘⌥H, ⌘⌥D, ⌃⌘F, ⌃↑ / ⌃↓ / ⌃← / ⌃→ on macOS, Ctrl+Alt+Delete on
  Windows and Linux, Print Screen on Linux. `Super+*` matches Super with any key.
- `ui/lib/shortcut.ts`' `keyName` now shows punctuation and arrows by their labels (`[`, `←`), so
  "⌘[" reads as it does in macOS's menus.
- **Smoke:** the history-off run's Settings sequence opens Keyboard after Appearance
  (`settings-keyboard.png`; `settingsFailures` wants `settings-window keyboard` after Appearance,
  tests); a short launch whose settings.json says `"shortcut": "Hyper+Nope"` must log
  `shortcut-failed`, register the default, and open Keyboard (`settings-keyboard-problem.png`;
  `keyboardProblemFailures`, tests). On macOS the runner's `com.apple.symbolichotkeys` is exported
  into the artifacts (read-only, informational) — the evidence for the read below.
- **Reading macOS's own shortcuts — decided by the runner's export** (this commit's CI); see the
  follow-up note below.
- `npm run verify`: 581 TS tests, 339 Rust tests, clippy clean. Disk: 8.8 GiB free at the start of
  the step and after verify; `.cache/target` 1.6 GB.
- **CI (run 39216324726): all ten jobs green after one re-run.** The first attempt's linux-scale-2
  smoke failed one rule of LJ36's fullscreen run — both "Phone player" clicks (at 2080,1453) never
  reached the page (no report at 390 or 720 px) — a synthetic click that didn't arrive, in a run
  this phase doesn't touch (LJ41–LJ43's CI passed it on that job); `gh run rerun --failed` once:
  green. The Settings sequence logged `settings-window keyboard` after Appearance everywhere; the
  problem launch logged `shortcut-failed Hyper+Nope: invalid`, registered the default and opened
  Keyboard on every OS. **Screenshots:** `settings-keyboard.png` — the Keyboard tab between General
  and Appearance: "Summon Little Joan" (⌘⇧Space / Ctrl+Shift+Space), then Little Joan's keys with
  their recorders, "No key" and no Reset (all defaults): macOS ⌘L, ⌘Y, ⌘⇧T, ⌘R, ⌘[; Windows Ctrl+L,
  Ctrl+H, Ctrl+Shift+T and "No key" for Reload / Back / Forward; Linux (scale 1 and 2) Ctrl+R, Alt+←,
  Alt+→ too; `settings-keyboard-problem.png` — the problem line ("“Hyper+Nope” couldn't be
  registered: it isn't a shortcut Little Joan can use. Little Joan is using ⌘⇧Space instead.") on
  every OS, but the summon row read "HyperNope" on macOS (the display joined text that isn't a
  shortcut), and nothing stopped saving that shortcut again — fixed below; General shows "Shortcut
  Ctrl+Shift+Space [Change…]".

**`lj-step-44-fix` — macOS's own shortcuts read (decided), and what the screenshots showed:**

- **The decision, by the runner's export** (`symbolichotkeys.json` from run 39216324726, kept as
  test data in `src-tauri/src/system_shortcuts/macos-runner.json` — a runner holds nothing
  personal): 19 entries; 12 say only `enabled: false` (no key), Mission Control's space moves
  (79–82) only `enabled: true`, the input sources (60, 61) are switched off **with their keys**
  (`[32, 49, 262144]` → ⌃Space, `[32, 49, 786432]` → ⌃⌥Space: exactly the curated table's), and 164
  is a `modifier` entry (a lone modifier, no key). So the domain is not empty and the mapping is
  trustworthy where an entry has a key; entries without one say nothing — which the plan's design
  already handles: **shipped**, with the curated table as the base. `system_shortcuts::parse`
  (pure; tests on the export and on decoded flags — ⇧⌘3, ⌃← with the function-key flag, unknown
  codes) and `platform/mac.rs`' `symbolic_hotkeys()` (NSUserDefaults `persistentDomainForName:`,
  the property list walked into JSON through `msg_send`, in an autorelease pool; no new crate);
  `keymap_info` adds `systemShortcuts` (empty on Windows and Linux). Curated macOS entries carry
  their domain ids (Spotlight 64, Finder search 65, input sources 60 / 61, screenshots 28 / 30 /
  184, ⌘` 27, ⌥⌘D 52, ⌃↑ 32, ⌃↓ 33, ⌃← 79, ⌃→ 81); in `reservedProblems` an entry switched off — or
  moved to another key — drops its curated warning, and one that is on with a key warns there
  ("macOS uses ⌥Space for Spotlight", else "one of its keyboard shortcuts"). It reads only; nothing
  is ever written (Ground Rule 21). The read wasn't run on the development Mac (it would read the
  owner's own preferences): CI's macOS smoke compares what `keymap_info` read (`system-shortcuts
  <n> keys=<k>`) with the runner's export (`systemShortcutFailures`, tests).
- **Fixes from the screenshots:** a summon shortcut that isn't one shows as written ("Hyper+Nope",
  `show`) and is an error ("That isn't a shortcut Little Joan can use: record another one, or
  Reset.") so Save can't store it again (`isShortcutKey`, exported from `ui/lib/shortcut.ts`);
  Windows gets a note that WebView2's own Ctrl+R, F5 and Alt+← / Alt+→ work beside these keys (the
  README said so, the tab didn't); Rust's `Binding::display` shows punctuation and arrows as
  Settings does (⌘[, Alt+←), so the menu log and Rust's refusals read like the tab.
- `npm run verify`: 585 TS tests, 342 Rust tests, clippy clean. Disk: 14 GiB free.

**`lj-step-44-fix2` — LJ36's fullscreen run clicks where the page is.** The docs commits' runs
(39222245523, 39224458382) failed linux-scale-2 on the rule the first LJ44 run had hit — "phone at
720: the page never reported" —, so it wasn't a one-off: 3 of 9 runs. The log showed why: after
leaving element fullscreen two layouts follow, and when the one computed from the fullscreen size
lands last, `page-screen` says `800,260,2560,1520` while the page reports 720 px; the run clicked
the player's centre from that line (2080,1453), below the real page (y 260–1380). The run now
takes the latest `page-screen` whose width is the page's own (`pageScreenFor`, falling back to the
latest; a test with the logged lines). The rule is unchanged — the click must reach the page and
the page must report. The app's race is recorded under Deferred follow-ups.
**CI for fix2 (run 39226938206): all ten jobs green after one re-run** — linux-scale-2 passed its
fullscreen run; macOS's smoke first failed LJ29's race step (the panel's load came 5 s after
`extension-opened hello`, no file was served to it and hello never reported; every racer was
still refused), a step this phase doesn't touch that passed in the six earlier runs; `gh run
rerun --failed` once: green (Deferred follow-ups).
- **CI for the fix (run 39219919487): all ten jobs green.** macOS's Keyboard tab read
  `system-shortcuts 19 keys=2` — exactly the runner's export (`systemShortcutFailures` passed);
  `settings-keyboard-problem.png` now shows "Hyper+Nope" as written with "That isn't a shortcut
  Little Joan can use: record another one, or Reset." under it (Save off). Windows' note about
  WebView2's own keys sits under the list, below the fold of the screenshot.

---

## v0.3 steps

> Planned 2026-10-18 with the owner (decisions: "Design Decisions Already Made → round 3"). They
> run after LJ19 (v0.2.0), in the order 21 → 22 → 23 → 24 → 25 → 26. Phases 21, 22 and 24 are
> **security-sensitive**: each ends in a `review` step — the PR is marked ready, green, and NOT
> merged; an independent read-only review follows (as for phase 16), its findings are fixed in
> `lj-step-N-fix<k>` commits, and the owner decides the merge. The next phase starts from `main`
> only when the owner says go; phase 22 changes the same new-window code as phase 21, so it waits
> until phase 21 is merged. The `auto` phases 23 and 25 are merged by the agent once green, as in
> v0.2 (CLAUDE.md's standing exception covers v0.3; Ground Rule 24); LJ57 is a checkpoint.

### LJ45 — Links from other apps: register as a browser and receive links · strong · auto

> Phase 21 (v0.3, security-sensitive). Branch `lj/p21-links` from the latest `main`; the PR goes
> into `main` and is not merged by the agent.

**Verify:** `npm run verify` green, including Rust tests:
- `cli.rs`: `--link <url>` takes exactly the next argument as data, even when it starts with `--`
  (`--link --quit` opens nothing and doesn't quit); only absolute `http`/`https` URLs open —
  `file:`, `javascript:`, `data:`, `about:`, `ljext:`, `http://ljext.<id>.localhost`, the app's own
  origin and a URL over 4,096 bytes are refused (`link-refused <scheme>`, never the URL); a launch
  with `--link` never toggles, hides, quits, or opens Settings or an extension, whatever else is on
  its command line; `--link` with nothing after it is a plain launch;
- a pure `links::accept(&Url) -> Option<Url>` that both `--link` and macOS's `RunEvent::Opened` go
  through; from a batch of opened URLs only the first acceptable one opens (`link-ignored <n>` for
  the rest);
- the toggle table: a link while hibernating → `Create(url)`; with the window open →
  `FocusAndNavigate(url)` (the command bar closes; the page before stays in the back trail).

Packaging: macOS `src-tauri/Info.plist` (merged by Tauri) declares `CFBundleURLTypes` for `http` and
`https` (role Viewer) and no document types; Windows `src-tauri/windows/hooks.nsh`
(`bundle.windows.nsis.installerHooks`) writes the per-user registration below and removes it on
uninstall; Linux `src-tauri/linux/little-joan.desktop.hbs` (the deb's and the AppImage's
`desktopTemplate`) adds `MimeType=x-scheme-handler/http;x-scheme-handler/https;`,
`Categories=Network;WebBrowser;` and `--link %u` to `Exec`. CI green on all three with a **links
run** (`scripts/ci/smoke-links.mts`, + test): the registration is read back (Windows `reg query`;
macOS `plutil` on the installed app's Info.plist and Launch Services' handler list for `https`
including `io.github.memoliertorerr.littlejoan`; Linux the installed `.desktop` file, which
`desktop-file-validate` accepts); the OS's default is untouched (Windows: no `UserChoice` names
`LittleJoanURL`; macOS: Launch Services' default `https` handler isn't Little Joan; Linux: `xdg-mime
query default x-scheme-handler/https` isn't ours); a link launched the way the OS would — Windows:
the ProgId's registered command with `%1` = the smoke page's URL; macOS: `open -a <installed app>
<url>`; Linux: the desktop file's `Exec` with `%u` — logs `link-opened` and the page reports
`LJ-SMOKE ready`; `--link "--quit"`, `--link "file:///etc/passwd"` and `--link "javascript:alert(1)"`
leave the app running with `link-refused`; at the end the Windows uninstaller (`/S`) leaves none of
the keys behind.
**Commit:** `lj-step-45: links from other apps — Little Joan registers as a browser candidate and opens links`

```text
v0.3's first feature (the owner's decision): Little Joan can be the system's browser for http and
https links, and a link from another app opens in the Little Joan window. It is security-sensitive:
a link is untrusted input arriving from outside (Ground Rule 22), and registering touches the OS's
app registry (Ground Rule 21). This step registers and receives; LJ46 adds the main browser, the
loop guard and the default-apps button.

Design (decided; do not relitigate):
- Little Joan only becomes a candidate. It never makes itself the default: no Windows UserChoice
  write (the OS protects it anyway), no LSSetDefaultHandlerForURLScheme, no `xdg-settings set` or
  `xdg-mime default` — not from the app, not from an installer, not from a package script. The
  user picks it in the OS's own settings (LJ46's button opens them).
- Receiving:
  - Windows and Linux: the OS starts `little-joan --link <url>`; single-instance forwards it to
    the running app as for every flag. cli.rs: `--link` takes exactly the next argument, whatever
    it looks like (Chrome's --single-argument idea): that is what keeps a crafted link from
    becoming a flag. cli.rs is at 392 lines: move its tests into cli/tests.rs first (a separate
    commit isn't needed; say so in the notes).
  - macOS: links arrive as Apple Events → tauri's RunEvent::Opened { urls } in lib.rs's app.run
    closure (the run loop already handles ExitRequested there). It exists only on macOS in tauri
    2.12.4 (app.rs, cfg(target_os = "macos")).
  - Both go through a pure links::accept (new src-tauri/src/links.rs): input::normalize is NOT
    used (a link is never a search); Url::parse, then the navigation rules for typed addresses
    (navigation::is_allowed_navigation and is_external_openable: web only, never extension-like,
    never the app origin), at most 4,096 bytes. Then joan::dispatch(Command::Show(Some(url))) —
    the toggle table already turns that into Create(url) or FocusAndNavigate(url). The visit
    records as `link` (history.rs). Log `link-opened` (no URL), `link-refused <scheme>`,
    `link-ignored <count>`.
- Registration (all per-user, matching the NSIS currentUser install):
  - macOS: src-tauri/Info.plist with CFBundleURLTypes = [{ CFBundleURLName "Web site URL",
    CFBundleURLSchemes [http, https], CFBundleTypeRole Viewer }]. No CFBundleDocumentTypes:
    Little Joan never opens files (Ground Rule 11). If the smoke shows Launch Services doesn't
    list Little Joan for https with URL types alone, STOP and report — claiming HTML documents
    would be a design change.
  - Windows: src-tauri/windows/hooks.nsh, referenced from tauri.conf.json bundle.windows.nsis.
    installerHooks (tauri-utils 2.10.2 config.rs documents the NSIS_HOOK_POSTINSTALL /
    PREUNINSTALL / POSTUNINSTALL macros). POSTINSTALL writes, under HKCU:
    Software\Clients\StartMenuInternet\LittleJoan (default "Little Joan"; Capabilities with
    ApplicationName, ApplicationDescription, ApplicationIcon "<exe>,0"; Capabilities\URLAssociations
    http = https = "LittleJoanURL"; DefaultIcon; shell\open\command = "\"<exe>\""),
    Software\RegisteredApplications "Little Joan" = "Software\Clients\StartMenuInternet\
    LittleJoan\Capabilities", and Software\Classes\LittleJoanURL (default "Little Joan URL";
    DefaultIcon; shell\open\command = "\"<exe>\" --link \"%1\""), then
    SHChangeNotify(SHCNE_ASSOCCHANGED). Never Software\Classes\http or \https themselves, never
    UserChoice. The uninstall hooks delete exactly these keys and values, then notify again.
  - Linux: a desktop file template (tauri-utils' deb and appimage desktop_template; its variables
    are categories, comment, exec, icon and name — extend the one LJ17 added, if it did) with Exec={{exec}} --link %u,
    MimeType=x-scheme-handler/http;x-scheme-handler/https; and Categories=Network;WebBrowser;
    (GNOME's Default Apps lists web browsers by that category). Launched from the app menu
    without a URL, `--link` gets nothing and it is a plain launch, as today.
- Focus: a link while Little Joan is open brings its window to the front (it was asked for).
  Local programs could already do that with --show <url>; links don't widen that.

Threat model for the PR's review focus (each item with the test or smoke rule that covers it):
argument injection through the link (single-argument --link; the registry's and desktop file's
quoting), what schemes and hosts may open (links::accept), no new IPC (no command added; no
capability changes — capability_tests.rs unchanged and green), the OS's defaults untouched (the
smoke's read-backs), uninstall leaving nothing behind (Windows), and a link flood (each link is one
navigation of the one window, as --show already is).

Smoke: scripts/ci/smoke-links.mts + test, its own launch, the sequence from Verify. On Windows read
the registered command from HKCU\Software\Classes\LittleJoanURL\shell\open\command and run it with
%1 replaced, as Explorer would; on macOS use a short Swift snippet for Launch Services'
LSCopyAllHandlersForURLScheme / LSCopyDefaultHandlerForURLScheme (read-only).

Files: src-tauri/src/{links.rs (new, + tests), cli.rs (+ cli/tests.rs), lib.rs (RunEvent::Opened,
single-instance), toggle.rs only if a test is missing}, src-tauri/Info.plist (new),
src-tauri/windows/hooks.nsh (new), src-tauri/linux/little-joan.desktop.hbs (new, or LJ17's),
src-tauri/tauri.conf.json, scripts/ci/smoke-links.mts (+ test), scripts/ci/smoke.mts (wiring),
.github/workflows/ci.yml only for the uninstall check at the end of the Windows smoke job,
README (Little Joan as your browser for links — written in LJ46 once the button exists; here only
the CLI flag), docs/architecture.md (CLI, packaging, a "Links from other apps" section), CHANGELOG
([Unreleased] → Added), SECURITY.md (links are untrusted input).

Do NOT make Little Joan the default anywhere, claim file types, or open anything but http(s) from
a link.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ45 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-45: links from other apps —
Little Joan registers as a browser candidate and opens links". Push, open the phase-21 PR into
main as a draft, and watch CI until every job is green (Ground Rule 3; two honest fix attempts,
then stop).
```

### LJ46 — The main browser: handing pages on without loops; the default-apps button · strong · review

**Verify:** `npm run verify` green, including Rust tests for a pure `browsers.rs`:
- `handoff_target(main_browser, os_default, installed, self_id)`: a chosen main browser that is
  installed wins; else the OS default unless it is Little Joan; else (Little Joan is the default,
  no main browser) the first other installed browser in the OS's own order; none → no target
  (`handoff-none`, and the toolbar's status says "No other browser to open this in"); a target that
  is Little Joan itself is always refused (`handoff-self-refused`);
- the per-OS readers' pure parts: Windows `StartMenuInternet` entries and the `UserChoice` ProgId,
  Linux `.desktop` files' `MimeType` / `Exec` / `NoDisplay` and `xdg-mime`'s answer, macOS bundle
  ids;
- a source test: `tauri_plugin_opener::open_url` appears only in `browsers.rs`'s glue (every
  handoff goes through one place).

Settings gain `mainBrowser` (an id from the OS's list, or null = "System default"; tolerant). CI
green on all three; the links run adds: `browsers-listed <n> default=<self|other|none>`; on Linux
and Windows a fake browser installed by the smoke (a `.desktop` file / a `StartMenuInternet` entry
whose command writes its URL to a file) chosen as `mainBrowser` receives the smoke page's
`target=_blank` link (clicked at a known point) and the app logs `handoff browser=<id>`; on Linux,
with the smoke making Little Joan the runner's default (`xdg-mime default` run by the smoke on the
disposable runner, never by the app) and no main browser chosen, the same click goes to the fake
browser (the first other one) and no `link-opened` follows (no loop). Screenshot
`settings-general-browser.png`.
**Commit:** `lj-step-46: the main browser for "Open in browser" — no loops when Little Joan is the default; the default-apps button`

```text
Once Little Joan can be the default browser (LJ45), "Open in browser", target=_blank links,
Settings' links and extensions' openUrl(where: "browser") would hand pages to the OS default —
possibly Little Joan itself, which would bring them straight back. This step routes every handoff
through one place that knows the user's main browser.

Design (decided; do not relitigate):
- browsers.rs (pure decisions + a thin per-OS reader in platform/{win,mac,linux}.rs, read-only):
  - installed browsers: macOS NSWorkspace URLsForApplicationsToOpenURL: for
    https://example.com (macOS 12+; our minimum is 12.3) → bundle ids and names; Windows HKCU and
    HKLM Software\Clients\StartMenuInternet\* (name, shell\open\command); Linux the .desktop files
    in the XDG data dirs whose MimeType has x-scheme-handler/https (not NoDisplay).
  - the OS default: macOS URLForApplicationToOpenURL:; Windows HKCU\Software\Microsoft\Windows\
    Shell\Associations\UrlAssociations\https\UserChoice ProgId ("LittleJoanURL" = Little Joan);
    Linux `xdg-mime query default x-scheme-handler/https`.
  - opening with one browser: macOS `open -b <bundle id> <url>`; Windows the browser's registered
    command with the URL as one argument (never through a shell); Linux `gio launch <desktop file>
    <url>` (GLib ≥ 2.68; Ubuntu 22.04 has 2.72). The OS default (when it isn't Little Joan): the
    opener as today.
  - hand_off(app, url): handoff_target (Verify), then open; logs `handoff <os-default |
    browser=<id> | none>`.
- Every handoff calls hand_off: page_actions::open_in_browser, bar.rs's page on_new_window,
  settings_window.rs's links, extensions/protocol.rs's openUrl to the browser. Nothing else calls
  tauri_plugin_opener::open_url (the source test).
- Settings → General gains a "Browser" section: "Little Joan is your default browser" / "isn't";
  "Open default apps settings" — macOS x-apple.systempreferences:com.apple.Desktop-Settings.extension
  (Desktop & Dock holds "Default web browser" on macOS 13+; macOS 12:
  x-apple.systempreferences:com.apple.preference.general), Windows
  ms-settings:defaultapps?registeredAppUser=Little%20Joan (Windows 11 opens Little Joan's page;
  Windows 10 opens Default apps), Linux `gnome-control-center default-apps`, then `systemsettings
  kcm_componentchooser`, else a note telling the user where to set it (with the xdg-settings
  command for them to run themselves); and "Main browser": System default + the installed browsers
  (never Little Joan). When Little Joan is the default and no main browser is chosen, the section
  says which browser pages go to ("Opening pages in Safari, because Little Joan is your default
  browser"). Commands (settings capability; a child module commands/browsers.rs):
  browsers_info(), open_default_apps_settings().
- Opening the OS's settings is the only thing the button does (Ground Rule 21).

Threat model additions for the review: no loop can form (handoff_target's self refusal, tested;
the Linux smoke), the browser command lines are built without a shell, the browser list comes
from the OS (never from a page), and Settings' strings are drawn as text.

Smoke: the links run's additions from Verify (scripts/ci/smoke-links.mts + test). The fake
browsers are created by the smoke on the disposable runners and removed afterwards; on macOS the
handoff is covered by unit tests and the human check.

Docs: README ("Little Joan as your browser for links": how to choose it per OS, what the main
browser does), docs/architecture.md (handoff), CHANGELOG ([Unreleased] → Added), SECURITY.md.

Do NOT change the OS default from the app or from the smoke on macOS / Windows, or open a page in
Little Joan when a handoff finds no target.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ46 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-46: the main browser for \"Open
in browser\" — no loops when Little Joan is the default; the default-apps button". Push and watch
CI until every job is green (Ground Rule 3; two honest fix attempts, then stop). Look at the
screenshot on every OS. This ends phase 21, which is security-sensitive (gate `review`, Ground Rule
24): fill the PR description from the template — the review focus is LJ45's and LJ46's threat
models, each item with its test or smoke rule. Needs human check: on your Mac pick Little Joan in
System Settings → Desktop & Dock → Default web browser, click a link in Mail or Notes: it opens in
Little Joan; "Open in browser" goes to your main browser and nothing bounces back; set Safari back.
On Windows (if you have one): Little Joan appears in Settings → Default apps, and the same checks.
On Linux: GNOME Settings → Default Apps lists it. Mark the PR ready once every job is green and
STOP. Never merge it: an independent security review comes next, and the owner decides.
```

### LJ47 — Sign-in popups: which ones open inside, and the popup window · strong · auto

> Phase 22 (v0.3, security-sensitive). Branch `lj/p22-popups` from the latest `main`, after phase
> 21's PR is merged (both change the page's new-window handler); if it is still open, stop and ask.

**Verify:** `npm run verify` green, including Rust tests for a pure `popups.rs`:
- `decide(opener, target, features, open_popups) -> InApp | Browser | Refuse`: the opener must be
  an http(s) web page in `joan-page` (an extension panel or an app page → today's behaviour);
  InApp when the request asks for a size (`window.open(url, name, "width=…,height=…")`, as sign-in
  libraries do) or the target is a known sign-in flow (accounts.google.com, appleid.apple.com,
  login.microsoftonline.com, login.live.com, github.com/login/oauth/…, www.facebook.com/…/dialog/
  oauth, discord.com/oauth2/…, x.com / twitter.com /i/oauth2/…, slack.com/oauth/…); `about:blank`
  with a size → InApp (libraries open a blank popup, then set its location), without one → Refuse;
  at most 2 popups at once (more → Browser); everything else → Browser (through LJ46's handoff),
  as today;
- the popup's navigation rules: http(s) and `about:blank` only — never extension-like URLs, never
  the app origin;
- capability_tests.rs: no capability's `webviews` or `windows` pattern matches `joan-popup-1` (or
  any popup label).

CI green on all three with a **popups run** (`scripts/ci/smoke-popups.mts`, + test; the smoke
server serves `tests/smoke/signin.html` on a second port, a different origin, like a real
provider): a click on the smoke page's "Sign in" (`?popup=oauth`) opens it with
`window.open(url, "lj-signin", "width=480,height=600")`; the popup posts
`window.opener.postMessage("lj-signed-in", "*")` and calls `window.close()`; the opener reports
`LJ-POPUP received=1 closed=1`; `?popup=blank` (a sized popup to `about:blank`, then
`popup.location = <signin url>`) reports the same; a plain `target=_blank` link logs `handoff …` and
opens no popup; with a popup open, `--hide` logs `popup-closed hibernate`, and no webview helpers
outlive the window (the existing helper rule, counting the popup's). Screenshot `popup.png` (the
popup over the window).
**Commit:** `lj-step-47: sign-in popups open in a small window of their own, with no IPC`

```text
"Sign in with Google / Apple / Microsoft / GitHub" often opens a popup and waits for it to post
back. Today every popup goes to the default browser, so those flows break (Later's "Popups inside
the app"). Decided with the owner: sign-in popups open in a small child window of Little Joan's
own, with an IPC-free page webview that shares the data store and the opener relationship (so
the popup can post back and close); they close on hibernate; everything else keeps going to the
browser. Security-sensitive: a popup is a web page and gets exactly what joan-page gets — nothing
(Ground Rule 23).

Design (decided; do not relitigate):
- popups.rs (pure, tests): decide (Verify) and the popup navigation rule.
- bar.rs's page on_new_window: InApp → build the popup and return
  NewWindowResponse::Create { window } (tauri 2.12.4 webview/mod.rs); Browser → LJ46's hand_off;
  Refuse → today's `new-window-refused`. Building: WebviewWindowBuilder::new(app,
  "joan-popup-<n>", WebviewUrl::External(<target>)) with .window_features(features) — it gives the
  popup the opener's webview configuration (macOS), environment (Windows) and related view (Linux),
  which is what makes window.opener and the shared cookie store work (webview_window.rs
  window_features). Follow tauri's own docs / example for building inside on_new_window (the
  handler must return the window); if that deadlocks on Windows (joan::dispatch exists because
  building windows from the main thread can), stop and report what the engine did.
- The window: inner size from the features, clamped to 320–1200 × 300–900; centred over the
  Little Joan window on its monitor; owned by it (.parent(&joan window)) so it stays above it;
  resizable; not always-on-top. Its title is the popup's current host (set by Rust on each
  navigation, never the page's own title), so you always see where you are signing in.
  on_navigation: popups::may_navigate. on_new_window inside a popup → hand_off (no nested
  popups). on_download: refused (`download-refused popup`). Permission requests: denied.
- Lifecycle: joan::destroy closes every popup first, before the page is released; a page that
  calls window.close() closes its popup — check per engine in CI (WebView2 WindowCloseRequested,
  WKWebView's webViewDidClose:, WebKitGTK's close signal) and wire whichever wry doesn't already;
  a popup closing hands the keyboard back to the page. Logs: `popup-opened <host>`,
  `popup-closed <by-page | hibernate | user>`.
- The data store: popups share the page's (the opener's environment / configuration), so a login
  made in the popup is the page's login. No separate profile.

Smoke: scripts/ci/smoke-popups.mts + test, tests/smoke/signin.html (new), `?popup=oauth|blank`
in smoke.html with logic in tests/smoke/popup-logic.mjs (+ test); the smoke server listens on a
second port for the sign-in origin.

Files: src-tauri/src/{popups.rs (new, + tests), bar.rs, joan.rs, capability_tests.rs, smoke.rs if
a helper is needed}, tests/smoke/{smoke.html, signin.html, popup-logic.mjs (+ test)},
scripts/ci/smoke-popups.mts (+ test), scripts/ci/smoke.mts (wiring), docs/architecture.md (a
Popups section; the trust table gains joan-popup-*), CHANGELOG ([Unreleased] → Added).

Do NOT give popups the bar webview, a toolbar, IPC, extensions or downloads, and don't keep them
across hibernation.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ47 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-47: sign-in popups open in a
small window of their own, with no IPC". Push, open the phase-22 PR into main as a draft, and watch
CI until every job is green (Ground Rule 3; two honest fix attempts, then stop). Look at popup.png
on every OS.
```

### LJ48 — Popups: closing with their page, keys, docs and the review · strong · review

**Verify:** `npm run verify` green, including Rust tests: popups close when the page webview
commits a page of another origin than their opener's (a reload or a same-origin navigation keeps
them); Ctrl/Cmd+W in a popup closes the popup only (the page keys' Close, routed by the focused
window); no other page key acts in a popup. CI green on all three; the popups run adds: the
opener navigates to another origin with a popup open → `popup-closed opener-left`; Ctrl+W in the
popup (Windows, Linux; best effort on macOS) → `popup-closed user` and the page window stays.
`docs/architecture.md`, `SECURITY.md` and the README describe popups.
**Commit:** `lj-step-48: popups close with their page; Ctrl/Cmd+W closes a popup; popup docs`

```text
The rest of the popup lifecycle, then the security review.

1. A popup belongs to the page that opened it: when the page webview commits a page of another
   origin than the popup's opener (bar.rs's page load Started), close its popups
   (`popup-closed opener-left`); reloads and same-origin navigations keep them (sign-in flows
   reload the opener).
2. Keys: Ctrl+W (Windows: AcceleratorKeyPressed on the popup's controller — platform/win.rs's
   install for a third kind of webview, keys only; Linux: the popup's GTK window key hook) and
   Cmd+W (macOS: the menu's Close Window closes the key window) close the popup. No other page key
   acts in a popup (no bar to focus, no history to show). Through the keymap (LJ43): only the
   close action applies to popups.
3. Docs: docs/architecture.md (Popups: which requests become popups, the window, its lifecycle,
   the trust table), README (Sign-in popups), SECURITY.md (popups are page trust; their title is
   the host), CHANGELOG.

Threat model for the review (each item with its test or smoke rule): popups have no IPC
(capability_tests.rs), can't load extension or app URLs (popups::may_navigate), can't open further
popups (hand_off), can't download, get no permissions, show their real host, share only the page's
own store, and never outlive the window (hibernate, opener-left). Also review what decide() lets
in: a page can always ask for a size, so any web page can open one in-app popup at a time (two at
most) — the same reach as a page's own window.open in any browser.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ48 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-48: popups close with their
page; Ctrl/Cmd+W closes a popup; popup docs". Push and watch CI until every job is green (Ground
Rule 3; two honest fix attempts, then stop). This ends phase 22, which is security-sensitive (gate
`review`): fill the PR description from the template — the review focus is the threat model above.
Needs human check: sign in with Google on a site that offers it (e.g. a "Sign in with Google"
button), with Apple and with GitHub where you have accounts: the popup shows the provider's host,
you sign in, it closes, and the site is signed in; close and summon — still signed in; a popup
left open goes away when you dismiss Little Joan. Mark the PR ready once every job is green and
STOP. Never merge it: an independent security review comes next, and the owner decides.
```

### LJ49 — Find in page · strong · auto

> Phase 23 (v0.3). Branch `lj/p23-find-pins` from the latest `main`.

**Verify:** `npm run verify` green, including:
- Rust tests for a pure `find.rs`: the find state per window (a query of at most 1,000
  characters, case-insensitive, next / previous wrap, a new query restarts, Stop clears, a new
  page ends the find); results `{ matches?, active?, found }` merged per engine (Windows and Linux
  count; macOS knows only found / not found); the smoke-log line;
- keymap: a `find` action, default Ctrl+F / Cmd+F (Windows: caught in AcceleratorKeyPressed so
  WebView2's own find bar never opens);
- vitest for `ui/toolbar/find-logic.ts`: the strip's state — open, typing debounced 120 ms →
  `find_start`, Enter next, Shift+Enter previous, Esc closes and gives the page the keyboard; the
  label "3 of 12" / "No matches" / "Found" (macOS) / "Find isn't available in this version of
  Windows' web engine".

Capabilities: the bar gains only `allow-find`. CI green on all three with a **find run**
(`scripts/ci/smoke-find.mts`, + test): the smoke page `?find=1` shows `lj-needle` three times;
Ctrl/Cmd+F opens the strip (`find-open`), typing `lj-needle` logs `find-result matches=3 active=1`
(Windows, Linux) or `find-result found=true` (macOS, best effort for typing), Enter → `active=2`,
Esc → `find-closed`. Screenshot `find.png`.
**Commit:** `lj-step-49: find in page — Ctrl/Cmd+F, the engines' own find, a find strip in the toolbar`

```text
Find in page (Later → v0.3, the owner's decision): Ctrl/Cmd+F opens a find field in the toolbar
(the trusted bar), driving each engine's native find. The page itself gets no script and no IPC
(Ground Rules 10 and 18).

Design (decided; do not relitigate):
- Native find, in platform/{win,mac,linux}.rs behind one find module (find.rs pure +
  thin glue):
  - Windows: webview2-com 0.39.1 has it (checked while planning: webview2-com-sys bindings.rs —
    ICoreWebView2_28::Find → ICoreWebView2Find with Start(options, handler), FindNext,
    FindPrevious, Stop, MatchCount, ActiveMatchIndex, add_MatchCountChanged,
    add_ActiveMatchIndexChanged; options from ICoreWebView2Environment15::CreateFindOptions with
    SetFindTerm, SetIsCaseSensitive(false), SetShouldHighlightAllMatches(true),
    SetSuppressDefaultFindDialog(true)). Cast the core webview to ICoreWebView2_28; if the
    installed runtime lacks it, report unavailable. Record CI's WebView2 runtime version.
  - macOS: WKWebView findString:withConfiguration:completionHandler: (macOS 11+) with a
    WKFindConfiguration (caseSensitive NO, wraps YES, backwards for previous); its WKFindResult
    says only matchFound — no count. The completion handler is a block: add block2 = "0.6" (0.6.2
    is already in Cargo.lock through wry — nothing new to fetch) as a macOS dependency and call it
    with objc2 msg_send as platform/mac.rs does; use objc2-web-kit 0.3 (also in the lock) only if
    msg_send gets unwieldy, and say which in the notes. Stop has nothing to undo but the selection
    the find left; leave it.
  - Linux: webkit2gtk's FindController (find_controller(); search(text, CASE_INSENSITIVE |
    WRAP_AROUND, max), search_next, search_previous, search_finish, count_matches; signals
    found-text, failed-to-find-text, counted-matches). WebKitGTK doesn't report the active match:
    track it in find.rs (1 after a search, ± with next / previous, wrapping) and say so.
- Rust ↔ bar: one bar command `find(action)` (`{ kind: start, query } | next | previous | stop`,
  a serde enum; commands/find.rs; allow-find) and results pushed with
  bar.eval("window.littleJoan.setFindResult(<json>)"). A page load (Started) ends the find and
  closes the strip.
- The keymap (LJ43) gains `find` (Ctrl+F / Cmd+F; macOS a View menu item "Find…" built from the
  keymap). Ctrl/Cmd+F with a web page or a panel opens the strip; on the start page it does
  nothing; it closes the command bar first if it is open.
- The strip (ui/toolbar/find-logic.ts pure + tests, ui/toolbar/find.ts glue, toolbar markup and
  styles): in the toolbar's middle, in place of the site while open — the field, the count, ↑ ↓
  buttons and ×; back and open-in-browser stay. Text drawn as text.

Smoke: scripts/ci/smoke-find.mts + test, `?find=1` in smoke.html (three visible `lj-needle`s),
keys with the existing helpers.

Docs: README (Find in page; the page keys table), docs/architecture.md (find per engine, commands,
keymap), CHANGELOG ([Unreleased] → Added); this plan's Later row "Find in page" → LJ49.

Do NOT inject a find script into the page, add regex or whole-word options, or find in the bar's
own screens.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ49 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-49: find in page — Ctrl/Cmd+F,
the engines' own find, a find strip in the toolbar". Push, open the phase-23 PR into main as a
draft, and watch CI until every job is green (Ground Rule 3; two honest fix attempts, then stop).
Look at find.png on every OS.
```

### LJ50 — Pinned sites at the top of the command bar · standard · auto

**Verify:** `npm run verify` green, including Rust tests: settings `pinnedSites` (at most 8 `{ url,
title }`, http(s) only, one per URL, tolerant parse); `suggest.rs` puts pinned sites first in the
empty state (kind `pinned`), then top sites without the pinned hosts, still at most 8 rows plus
Settings; and vitest tests: `rows.ts` draws pinned rows with a pin glyph; Shift+Delete on a pinned
row unpins it (instead of removing history); the quick commands (LJ42) offer "Pin this site" /
"Unpin this site" with the page open, showing its state. CI green on all three: a launch with the
smoke page pinned in settings.json shows `suggestions … first=127.0.0.1…` in its very first window
(no history yet); typing `pin` over another page and Enter adds it (settings.json). Screenshot
`pinned.png`.
**Commit:** `lj-step-50: pinned sites at the top of the command bar`

```text
An owner-approved addition: pin the current site so it always leads the command bar's empty state
(Little Arc keeps favourites at hand; frecency alone moves sites around).

Design (decided):
- settings.rs: `pinnedSites` (Verify's rules). The quick command setPinned { url, title, on }
  (LJ42's enum; the bar sends the page from quick_state, Rust checks it is the open web page).
- suggest.rs: pinned first (kind `pinned`, in the order pinned), then top sites minus pinned hosts;
  the starters only when there are neither. history.rs is unchanged.
- The bar: rows.ts / view.ts draw a pin glyph (a CSS mask); Shift+Delete on a pinned row unpins
  (setPinned … on: false). No reordering (Later).

Smoke: in the quick run (scripts/ci/smoke-quick.mts + test) or a short launch of its own.

Docs: README (pinning), docs/architecture.md (suggest, settings), CHANGELOG ([Unreleased] → Added).

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ50 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-50: pinned sites at the top of
the command bar". Push and watch CI until every job is green (Ground Rule 3; two honest fix
attempts, then stop). Look at pinned.png. This ends phase 23. Fill the PR description from the
template. Needs human check: Ctrl/Cmd+F on a long article and on a page in another language;
Enter / Shift+Enter; Esc; pin two sites and summon: they lead the list; unpin one with
Shift+Delete; the must-not-disturb list. Once every job is green, merge the PR into main with a
merge commit (Ground Rule 24), then stop and report.
```

### LJ51 — Saved logins on Windows: the engine's password saving and autofill as Privacy settings · strong · auto

> Phase 24 (v0.3, security-sensitive). Branch `lj/p24-saved-logins` from the latest `main`.

**Verify:** `npm run verify` green, including Rust tests: settings `savePasswords` (default false)
and `autofillForms` (default true — WebView2's own default, what Windows users have had since
v0.1), tolerant; the settings window uses the page window's webview data folder (`AppState::
webview_data_dir`; a test on the builder's input). Capabilities: the settings window gains only
`allow-clear-saved-logins`. CI green on all three with a **logins run** (`scripts/ci/
smoke-logins.mts`, + test; Windows rules): a default launch logs `logins password-autosave=false
autofill=true` (read back from the page webview's settings with the getters); with
`"savePasswords": true` → `password-autosave=true`; the settings window's profile path equals the
page's (`webview-profile page=<p> settings=<p>`, the same folder); Settings → Privacy → Remove saved
passwords and form data, while hibernating, logs `saved-logins-cleared ok`. macOS and Linux:
`settings-privacy-logins.png` shows the "isn't available" note. Screenshots
`settings-privacy-logins.png` on every OS.
**Commit:** `lj-step-51: Windows — the web engine's password saving and autofill as Privacy settings, and removing them`

```text
The owner's request "saved logins", per engine, as far as each allows. Windows' WebView2 has its
own password saving and autofill; Little Joan has never set them (WebView2's defaults: password
saving off, general autofill on). macOS passwords and passkeys are DEFERRED by the owner (Later);
Linux's WebKitGTK has no password manager. Security-sensitive: credentials (Ground Rule 26).

Design (decided; do not relitigate):
- settings.rs: savePasswords (default false) and autofillForms (default true, the engine's own
  default, so nothing changes for anyone until they choose; the owner may flip that default).
- platform/win.rs: on each page webview (and each popup, LJ47) right after it is built and after
  a Save: ICoreWebView2Settings4 SetIsPasswordAutosaveEnabled / SetIsGeneralAutofillEnabled
  (webview2-com 0.39.1 has both; checked while planning). Log the values read back.
- Removing: Settings → Privacy (Windows) gets "Offer to save passwords (kept by Windows' web
  engine, on this computer)", "Fill in forms (addresses and the like)", and "Remove saved
  passwords and form data…" — a question, then clear_saved_logins (settings capability;
  commands/logins.rs): ICoreWebView2Profile2::ClearBrowsingData(PASSWORD_AUTOSAVE |
  GENERAL_AUTOFILL) on the settings window's own WebView2 profile, so it works while
  hibernating (no page window, no extra webview — Ground Rule 9). That needs the settings window
  to share the page's user data folder: settings_window.rs sets no data_directory today; in
  installed builds both already end up in app_local_data_dir (Tauri forces it), but with
  LITTLEJOAN_DATA_DIR (dev, CI) the page uses <dir>/webview and Settings didn't (Landmines). Give
  Settings AppState::webview_data_dir too. CI checks both profiles' paths are the same
  (ICoreWebView2_13 Profile → ProfilePath); if they aren't, stop.
- macOS and Linux Privacy: "Saving passwords isn't available in Little Joan on this system yet."
  plus one line why (macOS: "Apple lets only browsers with a special entitlement use Passwords
  and passkeys"; Linux: "WebKitGTK has no password manager"). No switch.
- Ground Rule 26: Little Joan never reads, exports, logs or syncs a credential; the smoke never
  types a password.

Smoke: scripts/ci/smoke-logins.mts + test (Windows rules; the screenshots everywhere).

Docs: README (Privacy: saved passwords on Windows), docs/architecture.md (Where data lives: saved
passwords and form data; the settings window's data folder), CHANGELOG ([Unreleased] → Added),
SECURITY.md (credentials stay in the engine).

Do NOT build a password manager, read the engine's stores, or turn password saving on by default.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ51 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-51: Windows — the web engine's
password saving and autofill as Privacy settings, and removing them". Push, open the phase-24 PR
into main as a draft, and watch CI until every job is green (Ground Rule 3; two honest fix
attempts, then stop). Look at the screenshots on every OS.
```

### LJ52 — Passkeys and saved logins: what each engine offers; docs; the review · standard · review

**Verify:** `npm run verify` green, including vitest tests for `tests/smoke/webauthn-logic.mjs`
(the report). CI green on all three: the smoke page `?webauthn=1` reports `LJ-WEBAUTHN api=<0|1>
platform=<true|false|error> conditional=<true|false|error>` (`PublicKeyCredential`,
`isUserVerifyingPlatformAuthenticatorAvailable()`, `isConditionalMediationAvailable()`) on every
OS into `summary.json` (informational: it is evidence, not a rule). README, SECURITY.md and
docs/architecture.md describe what is saved where and what isn't available; Later holds the macOS
research.
**Commit:** `lj-step-52: what each engine offers for passkeys and saved logins; docs and the macOS research`

```text
Close the saved-logins phase with evidence and docs, then the security review.

1. tests/smoke/smoke.html `?webauthn=1` + tests/smoke/webauthn-logic.mjs (+ test): the report in
   Verify; scripts/ci/smoke-logins.mts adds it to summary.json for every OS. Write what each engine
   reported into the notes.
2. Docs: README (Privacy: saved passwords on Windows, passkeys where the engine offers them —
   Windows Hello in WebView2 is the owner's check; not available on macOS and Linux yet, and why),
   SECURITY.md, docs/architecture.md (Where data lives).
3. This plan's Later: the macOS row (the owner deferred it) with the research — WKWebView passkeys
   for an app that isn't Safari need Apple's restricted browser entitlement
   (com.apple.developer.web-browser.public-key-credential, granted by Apple on request to apps
   that are web browsers), which in turn needs Developer ID signing with a provisioning profile
   (LJ18's signing is a precondition); WKWebView's password AutoFill isn't dependable for embedding
   apps. Record what CI's `?webauthn=1` showed on macOS next to it. Linux: no engine support;
   would need libsecret and our own UI.

Threat model for the review (each with its test): password saving off by default; the setting
reaches every page webview and popup (the read-back log); removal really clears the page's profile
(same profile path, tested in CI); Little Joan never touches credential data (no code reads the
engine's stores — say how you checked); the Privacy texts are honest about where data lives.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ52 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-52: what each engine offers for
passkeys and saved logins; docs and the macOS research". Push and watch CI until every job is green
(Ground Rule 3; two honest fix attempts, then stop). This ends phase 24, which is security-sensitive
(gate `review`): fill the PR description from the template — the review focus is LJ51's design and
the threat model above. Needs human check (Windows): turn on "Offer to save passwords", sign in to
a site and accept WebView2's offer; close, summon: the login form fills in; remove saved passwords:
it no longer does; on a site with passkeys (e.g. github.com) sign in with Windows Hello. On macOS:
Privacy says it isn't available. Mark the PR ready once every job is green and STOP. Never merge it:
an independent security review comes next, and the owner decides.
```

### LJ53 — More bundled extensions: Reading list and To-do · standard · auto

> Phase 25 (v0.3). Branch `lj/p25-more-extensions` from the latest `main`.

**Verify:** `npm run verify` green, including vitest tests for each `logic.js`:
- **Reading list** (`extensions/reading-list/`, permissions `storage`, `pageInfo`): saving (URL
  without its fragment; the same page again moves to the top instead of doubling; at most 1,000),
  read / unread, remove with Undo, search (every word, any case and accent), the order (newest
  first, read ones folded under "Read");
- **To-do** (`extensions/todo/`, permission `storage`): add, edit, toggle, delete with Undo,
  reorder (Alt+↑ / ↓), "Clear done", at most 1,000 items of 500 characters, what storage holds is
  checked from scratch on read.

`extensions/bundled.test.ts` and the Rust test `the_shipped_extensions_are_all_valid` cover both
(no network, the SDK first, text drawn as text). CI green on all three: the bundled run opens both
(`reading-list.png` with seeded items, `todo.png`); from the smoke page, the command bar's "Save to
reading list" (Ctrl/Cmd+L, typing `reading`, the page action's row, Enter) saves it — the panel's
own storage call `ok` and the item in `extension-data/reading-list.json` (Windows, Linux; best
effort on macOS).
**Commit:** `lj-step-53: bundled extensions — Reading list and To-do`

```text
v0.3's bundled extensions (the owner's list), in LJ32's layout: extension.json, index.html (the
SDK, then <script type="module" src="ui.js">), ui.js (DOM glue), logic.js (pure ES module,
JSDoc-typed, // @ts-check) + logic.test.ts, style.css (the theme's --lj-* variables with fallbacks
from the first frame — LJ32's landmine), an original icon.svg. Keyboard first, fits phone width,
themed, no network (no network: host, no fetch), text drawn as text.

- Reading list: a page action "Save to reading list" (pageInfo) whose panel saves the page at once
  and shows the list with "Saved"; a command "Reading list" shows the list. Enter opens an item in
  the page (littleJoan.openUrl(url, { where: "page" })) and marks it read; R toggles read; Delete
  removes (Undo for 6 s, also Ctrl/Cmd+Z); / searches. Site icons are the command bar's lettered
  stand-ins (a panel can't load https images: its CSP allows 'self' data: blob: only).
- To-do: one list; N or the field adds, Enter edits, Space toggles, Delete removes (Undo), Alt+↑ /
  ↓ moves, "Clear done". Autosave like Notes (a debounce, a flush on leaving).

Smoke: scripts/ci/smoke-bundled.mts (+ test): seed both (Little Joan's own storage format,
seedFile), open both, and the page action from the smoke page as for QR code (pageActionFailures).

Docs: README (the bundled extensions table), docs/architecture.md (file map), CHANGELOG
([Unreleased] → Added).

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ53 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-53: bundled extensions — Reading
list and To-do". Push, open the phase-25 PR into main as a draft, and watch CI until every job is
green (Ground Rule 3; two honest fix attempts, then stop). Look at the screenshots.
```

### LJ54 — More bundled extensions: Stopwatch & countdown, Breathing timer, Minesweeper · standard · auto

**Verify:** `npm run verify` green, including vitest tests for each `logic.js`:
- **Stopwatch & countdown** (`extensions/timer/`, `storage`): the stopwatch (start / stop / laps /
  reset with an injected clock), the countdown from a stored end time (reopening shows the time
  left; one that ended while closed is reported, nothing runs in the background — the panel says
  so, as Focus timer does), presets 1 / 5 / 10 / 25 min and a custom time;
- **Breathing timer** (`extensions/breathe/`, `storage`): the patterns box 4-4-4-4, 4-7-8 and
  5.5-5.5, the phase sequence and timing with a fake clock, sessions of 1 / 3 / 5 minutes;
- **Minesweeper** (`extensions/minesweeper/`, `storage`): beginner 9×9 / 10, intermediate 16×16 /
  40, expert 30×16 / 99; the first reveal never hits a mine (nor its neighbours); flood fill;
  chording; flags; win / lose; best times; a seeded generator in tests.

CI green on all three: the bundled run opens all three (`timer.png` with a countdown running from
seeded storage — its title counts down, `breathe.png`, `minesweeper.png`).
**Commit:** `lj-step-54: bundled extensions — Stopwatch & countdown, Breathing timer, Minesweeper`

```text
Three more in LJ32's layout (see LJ53's prompt for the shared rules).

- Stopwatch & countdown: Space starts / stops, L adds a lap, R resets; the countdown keeps its end
  time in storage like Focus timer (extensions/focus/ is the model) and chimes with Web Audio only
  while open; the window title shows the time while running.
- Breathing timer: an expanding / contracting circle (CSS transforms; with reduced motion, a text
  count instead), the phase name ("Breathe in", "Hold", "Breathe out"), the chosen pattern
  remembered.
- Minesweeper: keyboard first (arrows move, Space or Enter reveals, F flags, D chords), the mouse
  too (right click flags); cells sized to fit the panel — expert is offered only when 30 columns
  fit at a usable size; best times per level.

Smoke: scripts/ci/smoke-bundled.mts (+ test): the three panels, with the countdown seeded.

Docs: README (the table), docs/architecture.md (file map), CHANGELOG.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ54 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-54: bundled extensions —
Stopwatch & countdown, Breathing timer, Minesweeper". Push and watch CI until every job is green
(Ground Rule 3; two honest fix attempts, then stop). Look at the screenshots.
```

### LJ55 — More bundled extensions: Password generator, Regex tester, Hash & Base64, Emoji picker · standard · auto

**Verify:** `npm run verify` green, including vitest tests for each `logic.js`:
- **Password generator** (`extensions/password/`, `clipboard`, `storage` for the options): length
  8–128, the chosen sets each at least once, look-alikes left out on request, rejection sampling
  over `crypto.getRandomValues` (no modulo bias; the randomness passed in for tests), the strength
  shown as bits (log2(set size) × length); no word lists;
- **Regex tester** (`extensions/regex/`, `storage`): flags g i m s u v y d, matches and groups
  (named too), the replace preview with `$1` / `$<name>`; matching runs in a dedicated worker
  (`worker.js`, same origin) that is terminated after 500 ms, so a catastrophic pattern says
  "Took too long" instead of freezing the panel;
- **Hash & Base64** (`extensions/hash/`, `clipboard`): SHA-1 / 256 / 384 / 512 with
  `crypto.subtle.digest` over UTF-8 text or a dropped file (read locally, at most 64 MB), hex and
  Base64 output; Base64 / Base64url both ways (invalid input refused); MD5 isn't in SubtleCrypto
  and isn't offered;
- **Emoji picker** (`extensions/emoji/`, `clipboard`, `storage` for recents): about 400 emoji in 8
  categories with English names and keywords written for Little Joan (no external data file); every
  entry is one RGI emoji (`/^\p{RGI_Emoji}$/v`), names are unique; search; grid keys; recents (24).

CI green on all three: the bundled run opens all four (`password.png`, `regex.png`, `hash.png`,
`emoji.png`); the hash panel's own self-test reports `subtle=ok` in its title on every OS (SubtleCrypto
needs a secure context: `ljext:` on macOS / Linux, `http://ljext.<id>.localhost` on Windows) — if
it isn't available on an engine, stop and report.
**Commit:** `lj-step-55: bundled extensions — Password generator, Regex tester, Hash & Base64, Emoji picker`

```text
Four tools in LJ32's layout (see LJ53's prompt for the shared rules).

- Password generator: like the quick answer `password 20` (ui/answers/random.ts), but with
  options; the panel can't import ui/, so logic.js has its own cryptoInt with tests. Enter copies.
- Regex tester: the pattern and flags at the top, the test text below with the matches marked
  (text nodes only, never innerHTML), a groups table, a replace field with its preview. The worker
  is a dedicated Worker from the extension's own origin — the runtime refuses service and shared
  workers (LJ29, L4) but not dedicated ones; check that it starts under the extension CSP on every
  engine.
- Hash & Base64: a text field or a dropped file; the four SHA digests; Base64 both ways. Text tools
  already has Base64 for text; this one adds files and digests.
- Emoji picker: categories as tabs, a search field, a grid (arrows, Enter copies and says
  "Copied"), recents first. No skin tones (Later).

Smoke: scripts/ci/smoke-bundled.mts (+ test): the four panels; the hash panel's subtle report.

Docs: README (the table), docs/architecture.md (file map), CHANGELOG.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ55 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-55: bundled extensions — Password
generator, Regex tester, Hash & Base64, Emoji picker". Push and watch CI until every job is green
(Ground Rule 3; two honest fix attempts, then stop). Look at the screenshots.
```

### LJ56 — Panels follow theme changes; `--extension` opens a page action · standard · auto

**Verify:** `npm run verify` green, including:
- Rust tests: after the bar reports new theme tokens while the page shows a **committed** panel
  (`NavGate::committed`), Rust tells that panel (`page.eval` of a dispatch of the
  `littlejoan:themechange` event, which carries no data), and never a web page or an uncommitted
  panel; `--extension <id>` for an extension without commands opens its first page action for the
  open web page (with `pageInfo` as the command bar builds it), and with no web page open logs
  `extension-missing <id> no-page`;
- vitest: the SDK's `littleJoan.theme.onChange(callback)` (extensions/sdk/v1.js + v1.d.ts) calls
  back with a fresh `theme` on the event; every bundled extension re-applies its colors on change.

CI green on all three: in the bundled run, with Snake open, Ctrl/Cmd+L, typing `dark` and Enter
(LJ42's quick command) logs `panel-theme-changed snake` and then Snake's own `ext-api snake theme
ok` (Windows, Linux; best effort on macOS); `--extension qr` over the smoke page opens QR code for
it, with `pageInfo` answered.
**Commit:** `lj-step-56: panels follow theme changes; --extension opens a page action`

```text
Two small platform touch-ups from Deferred follow-ups that make the bundled extensions better.
docs/extensions.md stays apiVersion 1: both are additions.

1. A theme change reaches an open panel. Today littleJoan.theme.get() is a snapshot (LJ29). When
   the bar reports new tokens (commands.rs set_theme_tokens) and the page shows a committed panel,
   Rust evals `window.dispatchEvent(new Event("littlejoan:themechange"))` into the page webview —
   checked against NavGate::committed right before the eval, and only then (a web page that
   dispatches the event on itself gains nothing: it carries no data). The SDK adds
   theme.onChange(callback), which asks `theme` again and calls back; bundled extensions use it.
   Log `panel-theme-changed <id>`.
2. `--extension <id>` and page actions (Deferred follow-ups): with no command, open the first page
   action for the open web page — the same path as the command bar's open_extension (session::open
   with Target page action and its page); no web page → `extension-missing <id> no-page`.

Smoke: the bundled run gets the two checks from Verify (scripts/ci/smoke-bundled.mts + test),
after the panels' screenshots so they keep the default theme.

Docs: docs/extensions.md (theme.onChange; --extension with page actions), README (the CLI),
docs/architecture.md, CHANGELOG; this plan (Deferred follow-ups: strike both lines, naming LJ56).

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ56 to ✅, add implementation
notes (with df -h), and commit everything as one commit: "lj-step-56: panels follow theme changes;
--extension opens a page action". Push and watch CI until every job is green (Ground Rule 3; two
honest fix attempts, then stop). This ends phase 25. Fill the PR description from the template.
Needs human check: each of the nine new extensions with the keyboard only, in light and dark;
change the theme with Snake open — Snake recolors; bind `little-joan --extension reading-list` to a
key in your OS and save a page with it. Once every job is green, merge the PR into main with a
merge commit (Ground Rule 24), then stop and report.
```

### LJ57 — v0.3: docs, screenshots, the owner's checklist · standard · checkpoint

> Phase 26 (v0.3). Branch `lj/p26-v0.3-docs` from the latest `main`. Phases 21, 22 and 24 are in
> only if the owner merged them after their reviews; anything not merged is listed in the
> checklist as not included.

**Verify:** `npm run verify` green. README's "Using it" covers Little Joan as the browser for
links, the main browser, sign-in popups, find in page, pinned sites, saved logins on Windows and
the new extensions, with screenshots in `docs/images/` from CI's macOS run (the smoke page and
sample data only). CHANGELOG `[Unreleased]` lists everything since v0.2.0. `docs/architecture.md`,
`docs/extensions.md`, `SECURITY.md` and `CONTRIBUTING.md` are current. CI green. **Checkpoint:** the
owner ticks the checklist.
**Human check:** the checklist below, copied into the PR.
**Commit:** `lj-step-57: docs and screenshots for v0.3`

```text
Bring the docs up to date with v0.3, then hand the owner a checklist.

Fill the PR description from the template. Needs human check, on your Mac with the CI build of
this PR (and Windows / Linux where you have them):
- [ ] Links: choose Little Joan as the default browser; a link from Mail / Notes opens in it; "Open
      in browser" goes to your main browser; nothing bounces back; switch back.
- [ ] Sign-in popups: Google, Apple, GitHub sign-ins complete in the popup, which shows the
      provider's host and closes; dismissing Little Joan closes an open popup.
- [ ] Find: Ctrl/Cmd+F, Enter, Shift+Enter, Esc on a long page.
- [ ] Pinned sites lead the command bar; Shift+Delete unpins.
- [ ] Windows: saving a password and filling it in; removing saved passwords; a passkey with
      Windows Hello. macOS: Privacy says it isn't available.
- [ ] The nine new extensions, keyboard only, light and dark; Reading list from a page.
- [ ] Everything from v0.2.0's checklist still holds: summon / hibernate, the command bar,
      history, themes, logins surviving close and quit, phone mode and resizing, the Keyboard tab,
      quick commands, downloads, zoom, fullscreen.
- [ ] Memory at rest: no webview helpers while hibernating, also after a popup was open.

Once every CI job is green, merge the PR into main with a merge commit (docs only), then STOP for
the checklist (Ground Rule 24). Do not start any further work until the owner says go. A
`v0.3.0-beta.1` pre-release happens only on the owner's word, the way beta.2 was made: bump the
three versions through a PR, merge it, let main's CI build the installers, rename them
(LittleJoan-0.3.0-beta.1-windows-x64-setup.exe, -macos-universal.dmg, -linux-amd64.deb,
-linux-x86_64.AppImage), write SHA256SUMS.txt, then
`gh release create v0.3.0-beta.1 --repo memoliertorerr/LittleJoan --prerelease --latest=false
--target <that main commit>` with notes from the CHANGELOG, and check the release, its tag and one
asset's checksum.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ57 to ✅ with the note
"waiting for the owner's checklist", add implementation notes (with df -h), and commit everything
as one commit: "lj-step-57: docs and screenshots for v0.3". Push, open the phase-26 PR into main,
and watch CI until every job is green (two honest fix attempts, then stop). This is a checkpoint:
fill the PR with the checklist above, merge it when green, tell the owner exactly what to check
and where, then stop.
```

---

## Deferred follow-ups (cross-step, live tracker)

> Things a step noticed but were out of its own scope. Kept here so they don't get buried in
> per-step notes. Strike through when genuinely handled; don't silently delete lines.

- ~~**First-run discoverability**~~ (surfaced by LJ3). Hibernating from the very first launch
  means a new user sees only a tray icon — and on GNOME without the AppIndicator extension,
  nothing at all. Option: when no `settings.json` exists yet, show the window once with the
  shortcut hint. → *The user chose the one-time welcome on 2026-10-17: resolved by LJ12.*
- ~~**Esc after a click in the welcome**~~ (surfaced by LJ12). Clicking the welcome's checkbox moves
  the keyboard into the page webview, where Esc does nothing until the URL bar has the focus
  again; Ctrl/Cmd+W, the close button and the shortcut still close. Options: give the start page
  `allow-hibernate` for Esc, or a command that hands the focus back to the bar. → *Resolved by
  LJ22: the welcome lives in the bar now, and Esc anywhere in the bar follows the field's rules.*
- ~~**Start at login**~~ (surfaced by LJ8's README). Without it the shortcut is dead after a
  reboot until the user launches the app; already designed under Later — the most useful next
  step. → *Handled by LJ11.*
- **The Windows Run value is unquoted** (surfaced by LJ11). auto-launch 0.5 writes
  `C:\Users\<you>\AppData\Local\Little Joan\little-joan.exe ` without quotes, and the plugin
  offers no way to change the path it records. Windows resolves such a command line by trying
  `…\Local\Little.exe` first, then the real path, so it starts — unless a `Little.exe` exists in
  `%LOCALAPPDATA%` (only the user can write there). Options: upstream a quoting fix to
  auto-launch / the plugin, or build the `AutoLaunch` ourselves. Needs a real Windows logon to
  confirm (CI can't log on).
- **macOS menu page items are always enabled** (surfaced by LJ7). The menu bar is hidden
  (Accessory app), so only the key equivalents matter, and they do nothing without a page. The
  tray items do track page state.
- ~~**Downloads use each engine's default**~~ (surfaced by LJ4). WebView2 saves to Downloads with its
  own UI; WKWebView and WebKitGTK ignore downloads unless the app handles them. Designed under
  Later. → *Planned as LJ15. Handled by LJ15: every engine saves to the Downloads folder under a
  cleaned, unique name, with the status in the toolbar.*
- **Linux fullscreen on Ubuntu 22.04 / the AppImage** (surfaced by LJ9). WebKitGTK 2.50.4
  aborts on element fullscreen (Landmines). Options for the user: (a) build the AppImage on
  ubuntu-24.04 so it bundles WebKitGTK 2.52 (raises its glibc floor to 2.39); (b) wait for
  Ubuntu to ship a fixed 2.50.x/2.52 to 22.04; (c) report upstream with the bare-WebKitGTK
  reproduction; (d) as a stopgap, handle `enter-fullscreen` and fullscreen only the window
  (the page wouldn't know it is fullscreen — YouTube's player would stay small). → *The user
  chose (a) + (d) on 2026-10-17, with (c) as a Launchpad report they file: planned as LJ13 and
  LJ14. Waiting (b) looks unpromising: jammy's webkit2gtk hasn't moved since 2026-01-13.* →
  *2026-10-19: (a) done by LJ13 — the AppImage bundles 2.52.6 and passes the whole smoke,
  fullscreen included. (d) is impossible: LJ14's experiment shows 2.50.4 aborting even when the
  request is declined (run 39247249388), so LJ14 stopped without app code. (c) is drafted
  (`docs/upstream/webkitgtk-2.50-fullscreen-abort.md`) for the owner to file. Left for 22.04's
  `.deb`, the owner's call: (b) a jammy update, which the report asks for; (e) untested — switch
  WebKitGTK's `enable-fullscreen` setting off below 2.52: the page's request would then fail
  without reaching the code that aborts (probably), so no crash, but no fullscreen either and no
  signal for a window fallback; one more variant of the bare check
  (`view.get_settings().set_enable_fullscreen(False)`) would show whether it avoids the abort.* →
  *The owner chose (e) on 2026-10-19 ("a crash is worse than no fullscreen"): the bare window
  survives with it on two 22.04 runners, and LJ14.1 switches fullscreen off below WebKitGTK 2.52 —
  22.04's `.deb` no longer crashes, and plays video in the window only. Still open: (b) a jammy
  update (then the switch turns itself off: it follows the runtime version), and (c) filing the
  drafted report — the owner's.*
- **macOS footprint after the first page stays ~12 MB above the hibernating baseline**
  (11 → 23 MB) — WebKit's framework state in our process. Acceptable for v0.1; noted so a
  future memory pass knows where to look. Not in v0.2: worth a time-boxed look (vmmap /
  footprint on a CI runner before and after the first page), not a phase.
- **Windows code signing** (planning v0.2, 2026-10-17). Unsigned for v0.2, so SmartScreen warns.
  The user applies to SignPath Foundation (free for OSI-licensed projects; the signer shown is
  "SignPath Foundation"; it needs a public code-signing policy in the README and the project's
  roles). If approved, add a step: sign the NSIS installer in the package job, plus a smoke
  check (`Get-AuthenticodeSignature` → Valid). Azure Artifact Signing ($9.99/month) only takes
  individuals in the US or Canada.
- ~~**`ubuntu-latest` moves to Ubuntu 26 from 2026-10-19**~~ (GitHub's notice on run
  39021598084). Handled by PR #8: the Linux `verify` job runs on `ubuntu-24.04`. A later
  ubuntu-26.04 verify job is optional.
- **Domain-wide session cookies come back host-only on Windows** (surfaced by LJ20). Tauri's
  cookie type (the `cookie` crate) strips a domain cookie's leading dot when it is read, so a
  session cookie set for `.example.com` is set again for `example.com` alone, and
  `www.example.com` no longer gets it after a close → summon. Persistent cookies are untouched
  (the engine keeps them). Options: read and set Windows' cookies through webview2-com
  (`ICoreWebView2CookieManager`) to keep the dot, or upstream a raw-domain accessor to
  `cookie` / wry.
- ~~**Release builds are bigger since `panic = "unwind"`** (surfaced by `lj-step-20-fix2`): CI's
  installer artifacts grew from 3.56 to 4.48 MB (macOS), 1.31 to 1.78 MB (Windows), and the Linux
  bundle by about 1 MB. Only macOS needs unwinding (wry's Objective-C exception catch); a
  target-specific `-C panic=unwind` for macOS with `abort` elsewhere would win most of it back,
  at the cost of a second place that decides the panic strategy. The owner's call; the README's
  sizes describe the v0.2.0-beta.1 downloads until the next release.~~ → *The owner decided
  2026-10-18: keep `unwind` everywhere ("installers can grow").*
- ~~**Double-click the toolbar to zoom the window on macOS**~~ (surfaced by LJ24). Tauri's drag
  script turns a double click on a drag region into `internal_toggle_maximize`, which needs
  `core:window:allow-internal-toggle-maximize`; the bar has only `allow-start-dragging` (what
  LJ24 named), so a double click does nothing. → *Planned as LJ37 (round 3). Handled by LJ37: the
  bar has the permission (and only the bar, a test says).*
- **A double click on the toolbar always zooms** (surfaced by LJ37). Tauri's
  `internal_toggle_maximize` maximizes or restores; macOS's "Double-click a window's title bar
  to: Minimize" (System Settings → Desktop & Dock; `AppleActionOnDoubleClick`) isn't followed.
  Option: read that preference (read-only, Ground Rule 21) and minimize instead — it would need
  our own double-click handling in the bar instead of Tauri's drag script.
- ~~**`history_top` / `history_search` have no caller**~~ since LJ23 (the command bar uses
  `suggest`). They stay in the bar's capability for the history view (LJ25) to use or remove. →
  *Removed by LJ25: the history view uses `history_list` and filters in TypeScript.*
- ~~**Ctrl+Y on a page is no longer redo on Windows**~~ (surfaced by LJ25). The plan's history key
  is Ctrl+Y on Windows and Linux, caught natively before the page sees it; in WebView2 (as in
  Chromium) Ctrl+Y was also "redo" in text fields, so pages lose it there (Ctrl+Shift+Z still
  redoes). WebKitGTK has no Ctrl+Y binding. Option: Ctrl+H (Chrome's history key) on Windows and
  Linux instead — the owner's call. → *Decided 2026-10-18: Ctrl+H on Windows and Linux, in the
  next version (see "Requested for the next version" under LJ35). Planned as LJ37. Handled by LJ37:
  Ctrl+H opens history, Ctrl+Y is the page's again (CI presses it and sees nothing open).*
- **A custom theme may show the default theme for a frame** (surfaced by LJ27). The bar applies
  the theme when `start_info` answers, a moment after its page (with `theme-defaults.css`) first
  paints, so a summon can flash the default Mist colors before the user's. Not visible in CI's
  screenshots. Option: also hand the bar the theme in an initialization script (as the settings
  window gets its tab), applied before the first paint. → *Planned as LJ38 (round 3). LJ38 gives the bar its theme in an initialization script, applied
  before its first frame; CI's theme runs log `theme-early` before every `theme`. Whether a flash
  is gone on a real Mac is the owner's check (phase 18's PR) — if one remains, the next move is
  handing the bar the painted `--lj-*` variables themselves (its last report, kept in `AppState`
  across windows).*
- **A panel can leave for any web address** (surfaced by LJ29). `network:` limits what a panel
  can fetch (CSP `connect-src`), not where it can go: `location.href`, a link, `openUrl` or
  `window.open` (→ the default browser) can carry data in a URL to any site — visible to the
  user, since the page changes. Options: a `navigation` permission (or a confirmation) for
  leaving the panel; CSP's `navigate-to` was never shipped by any engine.
- **The page webview can load any of the app's own pages** (surfaced by LJ29):
  `is_allowed_navigation` admits the whole app origin (`tauri://localhost/…`, Windows'
  `http://tauri.localhost/…`), so a page or panel can navigate it to `bar.html`. Harmless today
  (`joan-page` has no capability at all), but the page webview only needs `index.html`. Option:
  admit only the start page there.
- **A theme change doesn't reach an open panel** (surfaced by LJ29): `littleJoan.theme.get()` is
  a snapshot; a panel learns of a new theme (Save, a preview, the system switching) only when it
  asks again. Option: the bar's report also evals a `littleJoan` event into the open panel (Rust,
  not the panel, decides when). → *Planned as LJ56 (v0.3).*
- **A cancelled navigation counts as a second visit on Windows** (surfaced by LJ29's probe):
  WebView2 raises one more `Finished` load event for the page that stays when `NavigationStarting`
  cancels a navigation, so history records that page twice. *(LJ15)* A navigation that becomes a
  download does the same on Windows and Linux (WebKitGTK too raises a `Finished` for the page that
  stays). Option: ignore a `Finished` whose URL
  equals the last finished one within a short time (history_store), or track cancellations.
- **WebRTC is open to extension panels** (security review, L2). The CSP doesn't cover
  `RTCPeerConnection`, so a panel can reach a STUN/TURN server it doesn't declare (and learn local
  addresses where the engine exposes them). Not fixed in phase 16: removing the constructors needs
  a script injected before every panel's own (a user script per engine, or a wrapper the handler
  adds to each served HTML file — which the panel's own code could still undo before it runs),
  and WebKitGTK's `enable-webrtc` setting would also turn it off for web pages. Options: wry/Tauri
  per-webview WebRTC switches if they appear; an injected freeze of `RTCPeerConnection` for
  `ljext:` documents; a `webrtc` permission. Documented as a limit (docs/extensions.md).
- **`openUrl`'s private-host filter is best effort** (follow-up review). It looks at the URL
  only: a DNS name that resolves to a private address, single-label hosts (`http://router/`),
  `.lan` / `.internal` / `.home.arpa`, carrier-grade NAT 100.64/10 and benchmarking 198.18/15
  all pass. For the page it's the same reach any web page has; for the default browser the
  **`openBrowser` permission is the real control** (the user sees it in Settings before turning
  an extension on), the filter and the 2 s limit only narrow it. Options: add those suffixes and
  ranges; resolve the host first (and accept the race with DNS); ask before a browser open.
- **Dot look-alikes pass the name check** (follow-up review): `looks_like_address` splits on the
  ASCII dot only, so `paypal․com` (U+2024 ONE DOT LEADER), `paypal．com` (U+FF0E) or `paypal。com`
  (U+3002) is a valid name. The toolbar's "Extension" badge next to every extension name is what
  covers it. Options: normalize those to `.` before the check (or refuse them in names).
- **File requests have no concurrency cap, and wry buffers every body** (follow-up review): the
  8-calls-in-flight limit covers the API only; a panel can fire any number of file GETs (each a
  blocking-pool task that reads a file of at most 16 MB). And wry reads a whole request body into
  memory before the handler runs, on every engine, so `MAX_BODY_BYTES` refuses an oversized body
  only after it was buffered. Options: a per-extension cap on concurrent file reads; a streaming
  body in wry (upstream).
- **Service-worker registrations of `ljext` origins aren't cleared** (follow-up review, L4
  backstop). The handler refuses a service worker's script (and the smoke checks it on every OS),
  so none should exist; clearing them anyway when extensions are reloaded or switched off needs
  per-origin website-data removal, which the app's dependencies don't expose: WebView2's
  `Storage.clearDataForOrigin` (DevTools protocol) needs `webview2-com` as a direct dependency,
  WKWebView's `WKWebsiteDataStore.removeData` needs `objc2-web-kit`, WebKitGTK's
  `WebsiteDataManager` needs `webkit2gtk` — new crates — and Tauri's `clear_all_browsing_data`
  would wipe the user's cookies and sites too. Options: those calls behind a small platform
  module once a step allows the crates.
- **`--extension <id>` can't open a page action** (surfaced by LJ34): it opens an extension's
  first *command*, and the four bundled page actions have none, so `little-joan --extension qr`
  only logs `extension-missing` (a key binding for "QR code of this page" isn't possible yet).
  Options: fall back to the first page action for the open web page (as the command bar's
  `open_extension` builds its `pageInfo`), or a `--page-action <id>` flag. → *Planned as LJ56
  (v0.3): the first option.*
- **Back after a panel opens a page meets the panel's own history entry** (surfaced by LJ34, from
  reading the code — not seen in CI). A panel's `openUrl(…, { where: "page" })` (Wayback Machine,
  Clean link's "Open it here") and a link followed from a panel navigate the page webview from
  the panel, so the engine's history holds the panel (and, when it was opened from a web page, the
  start page of the detour) before the new page. The toolbar's Back (`history.back()`) then goes to
  the panel's URL, which the guard refuses (its nonce was used), so Back seems to do nothing; the
  command bar still opens anything. Options: Back goes `history.go(-n)` past extension and start-page
  entries (`back_stack` would count them), or the app navigates to the previous web page itself.
- **macOS CI: synthetic arrow keys don't always reach a panel** (surfaced by LJ32–LJ34's CI): 2048
  got its four arrows on Windows and both Linux jobs in every run, on macOS in two runs out of four
  (not in LJ32's and LJ33's, yes in LJ34's and its fix's), while key codes sent to the command bar
  do arrive there. Possibly a CI-only matter (the page webview's first responder on an unattended
  runner), possibly real; the owner's checklist asks whether the games take the keys without a
  click. Option, if real: make the page webview first responder when a panel commits
  (`focus_page`).
- ~~**macOS: a panel opened from hibernation can leave the toolbar unpainted**~~ (surfaced by LJ32's
  CI). With `--extension <id>` while hibernating, the window is built with the command bar and the
  panel's commit can arrive before the bar's own page has loaded; on macOS the toolbar then showed
  an empty field instead of the extension's name (hello in run 39118669089; the label was sent
  twice, once with `start_info`), while the same flow on `main` (bar loaded first) drew it. Opening
  from the command bar (the window already open) isn't affected. Options: have the bar ask for its
  label and overlay state in one call when it loads (`start_info` already forces the label); or
  open the panel only after the bar reported `bar-loaded`. → *Planned as LJ38 (round 3): the
  second option, keyed on the bar's `start_info`. Handled by LJ38: from hibernation the opening
  waits for the bar's `start_info` (CI: `bar-ready` before hello's panel commits on every OS, and
  macOS's `extension-hello.png` shows the toolbar naming hello).*
- ~~**YouTube's fullscreen button on macOS**~~ (the owner's checklist, 2026-10-18). The player's own
  fullscreen button enters fullscreen and drops back to the normal size at once; YouTube's F key
  fullscreens fine, and then the player's exit button does nothing (Esc exits). CI's smoke page
  (a click on a big `requestFullscreen` button) passes on macOS, so something YouTube does
  differs. Leads to check first: whether the window was under 600 px (phone mode gives an iPhone
  agent, and YouTube then uses the iOS-style `video.webkitEnterFullscreen()` path, which WKWebView
  on macOS handles differently), and whether a bounds or layout update reaches the page webview
  during WebKit's fullscreen transition. Reproduce with a smoke page that calls both APIs from a
  click before changing anything. → *Confirmed by the owner (2026-10-18): the phone version
  (m.youtube.com under the iPhone agent) failed; desktop YouTube works. The round-3 research adds
  that m.youtube.com's player waits for iOS events that never fire on macOS, and that wry's
  `set_bounds` would resize the webview inside WebKit's fullscreen window. Planned as LJ36:
  reproduce in CI, guard WebKit's fullscreen, and youtube.com on the desktop agent on WebKit.
  Handled by LJ36: CI reproduced it (our layout and phone mode acting inside WebKit's fullscreen
  window) and, with the guard, every element path holds until its own exit on all four smoke jobs
  (run 39150759131). The owner's eyes on real YouTube are phase 18's human check.*
- **macOS: a video's fullscreen may put the window back to the size it opened at** (surfaced by
  LJ36's CI). WebKit's element-fullscreen entry moves and resizes the Little Joan window (tao's
  `Moved` / `Resized`), and in CI the window came back 480×640 — its size at creation — after a
  `--size 390x600`. The guard keeps the page alone during the fullscreen and lays it out for the
  restored size afterwards, but the user's size is lost. Unknown whether a window resized by
  dragging (not `set_size`) does the same on a real Mac. Options, if it does: remember the window's
  frame when WebKit's fullscreen starts and set it again when it ends (macOS only); or find what
  AppKit restores (tao's `set_inner_size` uses `setContentSize:` asynchronously). → *The owner
  (2026-10-18): after leaving fullscreen the window was back exactly as before — same size and
  place — for a window sized by dragging; CI's case (a scripted `--size`) stays a note.*
- **The page webview's background stays white** (surfaced by LJ39). The round-3 decision gave both
  webviews the theme's background; LJ39 gave it only to the bar, because every engine draws a page
  without a CSS background over the webview's color (a dark theme would make such pages dark on
  dark). So on Windows a fast drag can still show a white strip at the edge of a *dark page* for a
  frame. Options: the theme's color only while no web page is shown (set it back to white on the
  first web page's `Started`) — which buys almost nothing, since the page webview is hidden behind
  the command bar until a page loads; or the page's own background color, which only Windows could
  read without a script (CDP `CSS`/`DOM`), not planned.
- **An agent switch on a page's own redirect could still bounce between two hosts** (surfaced by
  LJ40). `phone_mode::page_started` reloads when a navigation the page made lands on a host with
  another mode, and `RELOADED` stops only the same URL twice in a row. LJ40 keys a mode under the
  site (`m.` / `mobile.` and known phone hosts count as the site), which covers the `m.` ↔ `www.`
  case; two unrelated hosts that redirect to each other by agent, with different modes, would
  still alternate. Option: a reload budget (at most two agent reloads within a few seconds).
- **Windows' phone metadata is partly fixed** (surfaced by LJ40): `platformVersion` "10.0.0" (the
  reduced agent's "Android 10"), an empty `model`, the brands "Chromium" and "Google Chrome" (the
  agent string says Chrome) with WebView2's version, and a fixed GREASE brand. Sites that read
  high-entropy hints see a generic Android phone. Option: follow Chrome's own GREASE algorithm per
  version, if a site ever cares.
- **The toolbar can still show the default theme for a split second while the window opens**
  (the owner, 2026-10-18, after LJ38). With a non-default theme, about half of the toolbar paints
  in the default colours during the window's opening, then turns. LJ38 hands the bar its theme
  before its first frame, so this is likely the native window or the bar webview's background
  showing before the page paints. Leads: give the bar webview and the window the theme's
  background colour at creation (WebviewBuilder::background_color / the window's background), and
  show the window only after the bar reports its first frame.
- **"Request desktop site" changes nothing on some sites** (the owner, 2026-10-18). Sites that send
  one responsive page to every agent look the same either way, so the pill and the command seem to
  do nothing. Options: say so in the pill's tooltip, hide the pill for hosts where both agents gave
  the same page (Windows can compare `Page.getLayoutMetrics`; WebKit can't without page script),
  or offer page zoom (phase 8) as the way to get a "desktop" layout in a narrow window.
- **"Copy as Markdown link" shows twice when typing "copy"** (surfaced by LJ42's CI screenshot): the
  quick command (copies at once) and the bundled page action of the same name (opens its panel with
  the formats). Options: drop the quick row while the extension is switched on, or rename one
  ("Copy Markdown link" for the quick row).
- **After leaving element fullscreen, a layout from the fullscreen size can land last** (surfaced in
  phase 20 on linux-scale-2, 3 of 9 runs): `element-fullscreen false` and the window's `Resized`
  each lay out, and the one computed from the fullscreen size (2560×1520 physical) sometimes lands
  after the restored one — the smoke log's `page-screen` then said `800,260,2560,1520` for a page
  that reported 720 px. On Linux the GtkBox, not the bounds, places the page, so nothing showed;
  but LJ36's fullscreen run clicked from that line and missed the page (no `LJ-FS phone` report at
  720 px). `lj-step-44-fix2` made the harness click by the latest `page-screen` of the page's own
  width (`pageScreenFor`). The app's race stays: on Windows and macOS, where bounds do place the
  webviews, a stale layout would size the page wrong until the next `Resized` — not seen in CI
  there. Option: lay out once more on the main thread after a fullscreen change settles (read the
  size then), or drop a layout whose size isn't the window's current one.
- **LJ29's race step flaked once on macOS** (run 39226938206, phase 20; green on a re-run): with the
  racing page alive, the panel's load came 5 s after `extension-opened hello`, the start-page
  detour didn't show, no `ljext` request was answered for the panel, and hello never reported — the
  security rules held (every racer refused). If it recurs, look at the main thread's load during
  the race (the detour's 1 s limit) first.
- **The AppImage's own memory at rest is about twice the `.deb`'s** (surfaced by LJ13's CI): 45.6 MB
  private dirty at start against 19.5 MB, on the same runner and run — the bundled libraries'
  relocated pages are the app's own instead of shared with the system's copies; after a page both
  are ~44–45 MB. Resident memory is about the same (101 vs 99 MB). Not fixed: it is what bundling
  costs. A look with `smaps` per mapping would say which libraries weigh most.
- **The AppImage's login entry isn't quoted** (surfaced by LJ13, from the code): auto-launch writes
  `Exec=<$APPIMAGE> ` unquoted (LJ11's Landmine), so an AppImage kept under a path with a space —
  CI's artifact is named `Little Joan_<v>_amd64.AppImage` — would start at login as a broken
  command on desktops that follow the desktop-entry spec. Release assets are renamed without a
  space (`LittleJoan-<v>-linux-x86_64.AppImage`, which CI's AppImage smoke uses); the `.deb`'s
  `/usr/bin/little-joan` has none. The smoke's check accepts an unquoted path. Option: quote the
  path ourselves (build the `AutoLaunch` with a quoted path, as for the Windows Run value).
- **macOS CI: a WebContent process can outlive a hibernate by more than 10 s** (phase 7, run
  39247235206; green on a re-run): after the third summon cycle one helper was still there when the
  cycle check looked (10 s), while the main sample — 2, 10, 30 and 60 s after the first hibernate —
  saw none, as in every earlier run. LJ9 measured "1 helper at 2–10 s, 0 at 30 s" on macOS. If it
  recurs, give the cycle check macOS's 30 s rather than loosen the rule.
- **macOS shows an attachment it can display instead of downloading it** (surfaced by LJ15's CI): a
  PDF sent with `Content-Disposition: attachment` opens in the page on macOS (wry's WKWebView
  response policy asks only `canShowMIMEType`), while Windows and Linux save it. A link with the
  `download` attribute does download. Options: a response-policy hook of our own (wry has none to
  give; it would mean wrapping wry's navigation delegate as LJ15 wraps `setDelegate:`), or upstream a
  `Content-Disposition` check to wry.
- **A window summoned through the portal may open without the keyboard focus on GNOME** (surfaced by
  LJ17, from reading the spec; not seen — CI has no portal). GlobalShortcuts v2's `Activated`
  carries an `activation_token` in its options, which a Wayland app must hand to the compositor to
  take the focus past its focus-stealing prevention; Little Joan only dispatches `Toggle`. If the
  owner sees the window open unfocused (then a second press focuses instead of dismissing): hand
  the token to GDK (`gdk_wayland_display_set_startup_notification_id` before the window is
  presented, on the main thread) — Linux FFI in `platform/linux.rs`.
- **The portal isn't asked again after it restarts** (LJ17). The host registry's docs ask apps to
  watch the portal's `NameOwnerChanged` and register again; Little Joan, when its session's signals
  end, logs `shortcut-portal-unavailable ended` and falls back to global-hotkey (the icon comes
  back) until the next launch. Option: wait for the portal's name to come back and run the whole
  setup again.
- **The AppImage has no desktop file under the app id** (LJ17), so the portal's host registry
  refuses it and GNOME may refuse the shortcut (then `--toggle`). Options: install one on first run
  into `~/.local/share/applications/` (an AppImage writes nothing outside its own files today), or
  leave it to the user's AppImage integration tool.
- **Settings could open the desktop's own shortcut dialog** (LJ17): GlobalShortcuts v2 has
  `ConfigureShortcuts`, which a "Change…" button on the portal path could call instead of the note
  "Change it in your desktop's keyboard settings."
- **CI doesn't require the portal path's fallback line** (LJ17): the tray run's Wayland launch
  (`XDG_SESSION_TYPE=wayland` under Xvfb) asks the portal and must fall back, but `smoke-tray.mts`
  (not in LJ17's files) only checks `tray-visible true` there. Option: also require
  `shortcut-portal-unavailable` and then `shortcut-registered <accel>` in that launch (run
  39297748893 logged exactly that on all four Linux jobs).
- **Windows CI: a synthetic Ctrl+0 can miss the app** (phase 9, run 39301164697; green on a re-run):
  in the main run's zoom check SendKeys' Ctrl+= reached the page and Ctrl+0 3 s later didn't — the
  first miss of that check in the five Windows runs that had it. If it recurs, look at where the
  keyboard focus is after the zoom status shows (the toolbar's status area changes), or send the
  key twice, rather than loosen the rule.
- *(Steps add to this list as they surface out-of-scope friction.)*

---

## Landmines

Populated from real friction, not speculation:

- **TypeScript 7 is the native (Go) compiler and typescript-eslint doesn't support it.**
  `npm view typescript-eslint peerDependencies` → `typescript >=4.8.4 <6.1.0`. Pin
  TypeScript 6.0.3 until typescript-eslint supports 7.
- **wry 0.57 handles element fullscreen on macOS only.** Its WKWebView setup calls
  `setElementFullscreenEnabled(true)`; its WebView2 setup has no
  `ContainsFullScreenElementChanged` handler, so on Windows a fullscreen video only fills the
  webview's bounds until LJ5's handler resizes the window.
- **`CommandOrControl` is resolved at parse time per OS** (global-hotkey 0.8: Super on macOS,
  Control elsewhere), so the stored string is portable — keep storing `CommandOrControl`,
  never the resolved modifier.
- **Linux global shortcuts are X11-only.** global-hotkey 0.8's Linux backend is x11rb; under
  Wayland it grabs keys on XWayland, which only sees them when an X11 window has focus. The
  `--toggle` CLI flag (forwarded by single-instance) is the Wayland path. *(LJ17)* Where the XDG
  GlobalShortcuts portal exists (KDE Plasma, GNOME 48+) Little Joan asks it first; `--toggle`
  stays the path without it.
- **tauri-plugin-single-instance differs per OS:** macOS uses a Unix socket at
  `/tmp/<identifier>_si.sock`; Linux uses the D-Bus session bus and, if there is none, logs a
  warning and "launches normally" — a second `--show` would then start a second app instead of
  forwarding. Headless Linux CI must run inside `dbus-run-session`.
- **rustup's auto-install is deprecated.** `rustup show active-toolchain` still installs a
  missing toolchain but warns that this will stop; `scripts/setup-rust.sh` uses
  `rustup toolchain install` (reads rust-toolchain.toml).
- **Scripted text edits fail silently after rustfmt rewraps a line.** A Python `replace` on
  `build.rs` matched nothing in LJ6 (rustfmt had reflowed it), and only tauri-build's
  "Permission allow-get-settings not found" revealed it. Scripted edits now assert that their
  anchor exists.
- **tauri-codegen 2.7 does not add the IPC origins to a custom CSP** (checked in its source):
  without `connect-src ipc: http://ipc.localhost`, IPC falls back to postMessage after a CSP
  violation. `tauri.conf.json` lists them explicitly.
- **`Manager::get_window` is behind Tauri's `unstable` feature;** lifecycle code uses
  `get_webview_window` everywhere (LJ3).
- **`tauri://` URLs have an opaque origin in the `url` crate,** so `a.origin() == b.origin()` is
  always false for the macOS/Linux app origin; compare scheme/host/port (`keys::is_app_url`).
- **Installer file names contain a space** (`Little Joan_0.1.0_x64-setup.exe`,
  `Little Joan_0.1.0_amd64.deb`): quote them in scripts and docs.
- **WebKitGTK 2.50.4 (Ubuntu 22.04's package in Oct 2026) aborts the process when a page
  enters element fullscreen** — in CI under Xvfb, with or without a window manager, with the
  DMABUF renderer or compositing disabled, and in a bare PyGObject WebKitGTK window with no
  Tauri at all. WebKitGTK 2.52.6 (Ubuntu 24.04) works. The Linux smoke therefore runs the
  `.deb` on 24.04. *(LJ13)* The AppImage bundles the WebKitGTK of the machine that builds it, so
  it is built on ubuntu-24.04 since LJ13 (2.52; the package job fails on anything older), and its
  glibc floor is 2.39; the `.deb`, still built on 22.04, uses whatever the system has. Whether
  real 22.04 desktops crash too is a human check. *(LJ14)* **Declining the request doesn't help:**
  2.50.4 emits `enter-fullscreen` and then aborts whatever the handler returns — TRUE (declined),
  with or without fullscreening the window itself — in a bare PyGObject window on two 22.04
  runners (run 39247249388). No stopgap through that signal exists.
- **On Linux, tao doesn't see WebKitGTK's fullscreen.** WebKitGTK fullscreens the GTK window
  itself; `Window::is_fullscreen()` (tao's own state) stays false. Read the GDK window state
  (`platform::is_fullscreen`).
- **macOS keeps a closed WKWebView's page processes alive for minutes** unless the page is
  unloaded first (hide → `about:blank` → destroy, LJ9 fix). Measured in CI: 3 `WebContent`
  processes ≥ 90 s before the fix, 0 within 30 s after.
- **Debugging a WebKitGTK crash in CI:** gdb with debuginfod stalls app start past the smoke
  timeouts, and Ubuntu's ddebs had no symbols for the updated WebKitGTK. The cheapest
  discriminator was reproducing with a bare WebKitGTK window (`linux_plain_webkit_check`).
- **GitHub's Azure apt mirror can take 20+ minutes** for WebKitGTK; the smoke job's timeout
  is 45 minutes.
- **Google Translate's proxy picks the UI language from the requester's IP** (`_x_tr_hl=it`
  came back from a plain request), so the translate URL passes `hl=<lang>` too.
- *(v0.2 planning, 2026-10-17)* **Tauri's `zoom_hotkeys_enabled` is IPC for the page on macOS
  and Linux.** Its docs (tauri 2.12.4, `webview/mod.rs`) say it "injects a polyfill" that
  "requires `core:webview:allow-set-webview-zoom` permission", so the page would need IPC. Zoom
  goes through native keys and `Webview::set_zoom` instead (LJ16).
- *(v0.2 planning)* **`DownloadEvent::Finished` has no path on macOS** ("always empty, due to
  API limitations", tauri 2.12.4). Remember the destination set in `Requested`, keyed by URL
  (LJ15).
- *(v0.2 planning)* **tauri-plugin-autostart records the canonical path of the running
  executable on macOS** (`current_exe()?.canonicalize()` in its source). An app run straight
  from a DMG or Downloads is translocated to a random temporary path, and a LaunchAgent made
  then would point at nothing after a reboot. LJ11 refuses to enable it from
  `/AppTranslocation/`. On Linux the plugin uses `$APPIMAGE` for the AppImage.
- *(v0.2 planning)* **`ubuntu-latest` moves to Ubuntu 26 from 2026-10-19** (GitHub's notice on
  run 39021598084). The Linux `verify` job is pinned to `ubuntu-24.04` (PR #8); packaging (22.04)
  and smoke (24.04) were already pinned.
- *(LJ11)* **tauri-plugin-autostart's entries are named after the product name** (auto-launch
  0.5.0's source): Windows — the value `Little Joan` under `HKCU\Software\Microsoft\Windows\
  CurrentVersion\Run`, plus a 12-byte "enabled" flag under `…\Explorer\StartupApproved\Run`
  that `disable` leaves behind; macOS — `~/Library/LaunchAgents/Little Joan.plist` with `Label`
  "Little Joan", `ProgramArguments` = [the executable's canonical path], `RunAtLoad`; Linux —
  `~/.config/autostart/Little Joan.desktop` (always `$HOME/.config`) with `Exec=<binary> ` (a
  trailing space; `$APPIMAGE` for the AppImage). Renaming the product would orphan existing
  entries. Windows' value is **unquoted** (Deferred follow-ups), and Windows' `disable` fails
  when the value is missing.
- *(LJ11)* **`tauri_plugin_autostart::Builder::macos_launcher` exists only on macOS**
  (`#[cfg(target_os = "macos")]`); `init(MacosLauncher::LaunchAgent, None)` compiles everywhere.
- *(v0.2 planning)* **Ubuntu 22.04's WebKitGTK has stood still:** jammy has
  2.50.4-0ubuntu0.22.04.1 (2026-01-13), while noble and resolute got 2.52.6 on 2026-08-31
  (Launchpad, checked 2026-10-17). Don't wait on a jammy update.
- *(beta.1, 2026-10-17)* **A Windows profile can lack `HKCU\…\CurrentVersion\Run`.**
  auto-launch 0.5 opens it with a plain `?`, so on the beta release PR's runner every start-at-login
  call failed with "The system cannot find the file specified" (PR #10's runner happened to have
  the key). Fixed in lj-step-11-fix (`autostart.rs` creates the key first, through `winreg` 0.10,
  which auto-launch already uses). The Windows smoke now deletes the key before testing.
- *(round 2 planning, vendored tauri 2.12.4 / wry 0.57)* **Custom-scheme pages count as LOCAL**
  for capabilities (`webview/mod.rs` `is_local_url`: any registered custom scheme), so an
  `ljext:` page gets every local capability of its webview label, and a `remote.urls` entry can
  never match it. That is why `joan-page` must have no capabilities at all (Ground Rule 18).
- *(round 2 planning)* **Windows injects Tauri's IPC scripts into every frame.** wry adds them with
  `AddScriptToExecuteOnDocumentCreated`, ignoring "main frame only"; on macOS and Linux they stay in
  the main frame. Never put untrusted code in an iframe of a trusted page and rely on that.
  Opaque-origin frames are rejected by the IPC (`Origin: null` doesn't parse), but don't lean on it.
- *(round 2 planning)* **Where webview data lives:** macOS uses WebKit's `defaultDataStore`, keyed
  by bundle id; `data_directory` does nothing there. Windows and Linux: when the app sets no
  `data_directory`, Tauri forces it to `app_local_data_dir` (`%LOCALAPPDATA%\<identifier>`,
  `~/.local/share/<identifier>`). Linux keeps cookies in a text file there. Dev builds with
  `LITTLEJOAN_DATA_DIR` use `<dir>/webview` instead, so switching between a dev build and the
  installed one looks like a logout.
- *(round 2 planning)* **WebView2 loses session cookies on every window close** (each wry webview
  has its own environment, which ends with the window). **macOS Quit** goes `app.exit(0)` →
  `process::exit` without unloading the webviews, so cookies written just before quitting may not
  reach disk. LJ20 tests both.
- *(round 2 planning)* **Phone mode changes the user agent** between the phone and desktop ones.
  Cookies aren't keyed by user agent, but some sessions are bound to it (Cloudflare clearance,
  some banks and SSO, Google risk checks), so a site may ask to sign in again after the window
  crosses 600 px. Not fixed; Later has "keep one user agent per signed-in site".
- *(round 2 planning)* **WKWebView's tracking prevention caps cookies written by JavaScript at
  7 days** for sites without recent interaction. This is Apple's behavior; a site that keeps a
  login only in a JS cookie can log out after a week idle.
- *(LJ20)* **wry 0.57 panics reading an unset WKWebView address** (`url_from_webview`:
  `webview.URL().unwrap()`, on the main thread, where it can't unwind). The address is unset
  until the first navigation commits, and goes back to unset when that first navigation never
  does — CI saw it in a relaunch right after `process::exit`. Read the page's address through
  `bar::page_url`, which serves it from the load events (`page_address.rs`); only the in-page
  check asks the engine, after a finished load.
- *(LJ22)* **A WebKitGTK view's natural height is its contents size,** never smaller than the
  view, and GtkBox gives children their natural size before `expand`. A webview that was made
  tall once (the command bar's overlay) stays tall when packed with `expand = false` again, and
  its sibling gets 1 px. Pack a frame that doesn't propagate natural size (GtkScrolledWindow,
  external scrolling, around a minimum-policy GtkViewport) instead of the view.
- *(LJ20 fix2)* **`panic = "abort"` breaks wry's Objective-C exception catch on macOS.** Tauri
  answers `tauri://` requests from a tokio worker, and when the page navigates away while one of
  them is in flight (the start page replaced at once by `--show <url>`; the bar unloaded by a
  quick close), WebKit raises "task already stopped" from `didReceiveResponse` & co. wry wraps
  those calls in `objc2::exception::catch`, which can't catch under `panic = "abort"`, so the app
  aborted with "panic in a function that cannot unwind" on `tokio-rt-worker`. Keep the release
  profile at `panic = "unwind"`.
- *(LJ20)* **Tauri's `cookie` crate strips a domain cookie's leading dot** (`Cookie::domain`
  and `domain_raw` both do), and wry passes `domain()` to the engines, so a cookie read with
  `Webview::cookies` and set again becomes host-only. Deferred follow-ups has the options.
- *(LJ20)* **`Webview::cookies_for_url` can abort the app**: tauri-runtime-wry's dispatcher does
  `rx.recv().unwrap()` for it (not for `cookies()`), so a webview gone before the message is
  handled panics. Use `cookies()`.
- *(LJ20, macOS)* **After `process::exit` with a page open, the next launch's WebKit may load
  nothing** (both webviews stayed blank in CI). Quit closes the window through `destroy` first.
- *(round 2 planning)* **`navigation.rs` refuses `ljext:`** on macOS and Linux, frames included,
  while Windows' `http://ljext.localhost` form passes as plain `http`. LJ29 must handle both. →
  *LJ29: every `http(s)://ljext.*` URL counts as an extension URL on every OS
  (`extensions::urls::is_ext_like`).*
- *(LJ29)* **WebView2's custom-scheme workaround is a plain prefix match.** wry 0.57 intercepts
  every `http://ljext.*` request (`AddWebResourceRequestedFilter("http://ljext.*")`, all request
  kinds, frames included) and rewrites `http://ljext.` to `ljext://` (every occurrence in the
  URI) before the handler sees it, so a web page's `http://ljext.evil.com/` request reaches our
  handler as `ljext://evil.com/`. `Webview::navigate` / wry's `load_url` does **not** apply the
  workaround: navigate to the `http://ljext.…` form yourself on Windows. A host outside
  `.localhost` could be upgraded to https by Chromium (HSTS-preloaded TLDs such as `.app`,
  `.dev`), miss the filter and go to the network, hence `http://ljext.<id>.localhost`.
- *(LJ29)* **WebView2's `NavigationStarting` is main-frame only** — wry doesn't hook
  `FrameNavigationStarting`, so on Windows `on_navigation` never sees a frame's navigation (on
  macOS and Linux it sees frames too). Don't rely on the navigation guard for frames: the `ljext`
  handler itself refuses files unless the page shows that extension, plus `frame-ancestors`.
- *(LJ29)* **Custom schemes are registered for every webview** (`register_*_uri_scheme_protocol`
  in tauri 2.12 adds the handler to each webview the app builds), so the handler must check
  `UriSchemeContext::webview_label()` — the bar and Settings can request `ljext:` URLs too.
- *(LJ29)* **WebKit sends no `Origin` with a same-origin POST to a custom scheme** (macOS,
  Linux): only its CORS path adds the header, so a panel's own fetch to `/__lj/api/v1/…` arrived
  without one (`origin=-` in CI's log; not `null`, which was the first guess). Every cross-origin
  request does carry one. WebView2 sends it always. `api::origin_ok` encodes that per platform.
  And **WebKit sends cross-origin requests to a custom scheme without a CORS preflight**, custom
  header and all: a web page's POST to `ljext:` reaches the handler, and only the handler's checks
  (Origin, page) refuse it.
- *(LJ29, Windows)* **Load events can arrive after the app started the next navigation**:
  WebView2's `Finished` (NavigationCompleted) of the start page landed after `--extension hello`
  had begun navigating to the panel, and `Started` only comes at ContentLoading, after the request
  was answered. Anything that must know "what document is this request for" can't read the page's
  address from load events; the extension page check follows the navigation guard instead.
- *(LJ29, Windows)* **A navigation cancelled in `NavigationStarting` may still request its URL**:
  CI's probe saw the guard refuse `http://ljext.hello.localhost/index.html` and then that very GET
  reach the `ljext` handler (refused there by the page check). The cancelled navigation also raised
  one more `Finished` load event for the page that stayed (a second history visit). Never rely on
  the navigation guard alone for what a URL serves.
- *(LJ29, security review)* **No engine passes `Sec-Fetch-*` to a custom-scheme handler** (CI's
  `ljext-headers` log, all four smoke jobs): WebView2 gives the handler `Accept`, `Referer`,
  `User-Agent`, `sec-ch-ua*`, `Origin` on POST and `Sec-Fetch-Mode` only on a CORS preflight;
  WebKit gives `Accept`, `User-Agent` and a `Referer` on POST only. Checks keyed on
  `Sec-Fetch-Dest` / `-Site` never fire on our engines; a frame's request on Windows looks like
  any other GET except for its Referer. Don't build a defence on them.
- *(LJ29, Windows)* **A web page still runs between the guard admitting a navigation and its
  commit**, and its frames' requests reach the `ljext` handler in that moment (~60 ms in CI). The
  guard's "showing" is therefore not enough to serve files: only the nonce document is served
  before the commit (fix 6).
- *(LJ29, Windows)* **A web page's `window.open` while the page webview loads another document
  can break that load**: twice in CI a racing page's popup request landed between the app's
  navigation to a panel and its commit, and the panel came up once without any subresource
  (unstyled, no script, broken icon) and once as WebView2's own "HTTP ERROR 404" page for the
  panel's URL — though the handler had answered the document 200. Don't load something important
  while a hostile page is still alive: the app leaves the web page for its start page first
  (fix 7). If the race step flakes on Windows, look here first.
- *(LJ32)* **WebKit may not paint a page that starts invisible.** A panel that hid `<body>` with
  `opacity: 0` until its theme arrived stayed a blank white page in CI (macOS, Linux at scale 2)
  although its script ran and showed it a moment later — WebKit waits for a visually non-empty
  first layout before committing frames. Show content from the first frame (fallback colors).
- *(LJ29)* **Dot segments are gone before the handler runs:** engines and the `url` crate resolve
  `..` and `.` — also written `%2e` — while parsing (WHATWG URL), never above the root. A test
  that puts `/../x` in a URL tests the parser, not the path check; test `paths::clean` directly.
- *(round 3 planning, read in the code)* **macOS has no element-fullscreen hook.** Only
  `platform/win.rs` and `platform/linux.rs` call `bar::set_element_fullscreen`, so on macOS
  `bar::element_fullscreen()` is never true: `apply_layout` (on every `Resized`) still sets the
  page webview's bounds and shows it, and `phone_mode::apply` checks only `window.is_fullscreen()`,
  which is false for WebKit's separate fullscreen window. LJ36 starts there.
- *(round 3 planning, vendored tauri 2.12.4)* **`NewWindowResponse::Create` takes a whole
  `WebviewWindow`,** and what makes `window.opener` and the shared cookie store work is
  `WebviewWindowBuilder::window_features(features)` — it copies the opener's webview configuration
  (macOS), environment (Windows) or related view (Linux) (`webview_window.rs`). LJ47 relies on it.
- *(round 3 planning, vendored tauri 2.12.4)* **`RunEvent::Opened` exists only on macOS** (and
  iOS / Android): on Windows and Linux a link from another app arrives as a command-line argument,
  through single-instance. LJ45.
- *(round 3 planning, read in the code)* **The settings window doesn't use the page's webview data
  folder.** `settings_window.rs` sets no `data_directory`, so Tauri forces `app_local_data_dir`;
  the page uses `AppState::webview_data_dir` — the same folder in installed builds, but
  `<LITTLEJOAN_DATA_DIR>/webview` in dev builds and CI. Anything that acts on the page's profile
  through the settings window (LJ51) must give it the same folder first.
- *(round 3 planning, vendored tauri-utils 2.10.2)* **Tauri's desktop-file template offers only
  `categories`, `comment`, `exec`, `icon` and `name`;** `MimeType=` and `%u` go into the template's
  text (LJ45).
- *(round 3 resize research, vendored wry 0.57)* **wry's `set_bounds` sets a WKWebView's frame in
  its current superview** (`wkwebview/mod.rs` 1031–1047). During WebKit's element fullscreen that
  is WebKit's fullscreen window, so a layout then shrinks the video. LJ36 skips layout while
  WebKit's fullscreen is on.
- *(round 3 resize research, vendored wry 0.57 / tao 0.37.1)* **macOS resizes are a frame late.**
  wry gives child WKWebViews the autoresizing mask `ViewMinYMargin` (`wkwebview/mod.rs` 502–504),
  and tao delivers `Resized` from a BeforeWaiting observer after Core Animation committed
  (`window_delegate.rs` 94–99, `observer.rs` 239–241, `app_state.rs` 384–396), so bounds set in
  `Resized` show up a frame later. LJ39 sets the masks per mode instead.
- *(round 3 resize research, vendored tauri 2.12.4)* **Tauri's `auto_resize` scales child webviews
  proportionally** — the 40 px bar would grow with the window. Never use it (LJ39).
- *(round 3 resize research)* **A bare custom user agent turns WebView2's client hints off.**
  `put_UserAgent` "may clear" UA client hints, and Chromium disables UA-CH for a custom agent
  without metadata, so a site sees an Android agent with no (or desktop) hints. webview2-com-sys
  0.39.1 has no client-hints API; CDP's `Emulation.setUserAgentOverride` with
  `userAgentMetadata` is the way (LJ40) — whether it works with DevTools disabled is for CI to show.
- *(round 3 resize research)* **The desktop engines ignore `<meta name="viewport">`.** A narrow
  window gets a narrow layout only from the page's own CSS; only Windows could emulate a mobile
  viewport (CDP `Emulation.setDeviceMetricsOverride`), and macOS has only private SPI. Phase 19
  doesn't emulate viewports.
- *(LJ36, CI runs 39147876852, 39149305504)* **macOS: WebKit's element fullscreen moves and resizes
  the Little Joan window** — tao reports `Moved` / `Resized` during the entry, to the size the
  window was created at (480×640 after a `--size 390x600`), and the window keeps that size after
  the fullscreen (Deferred follow-ups). Anything that lays out the page or
  switches phone mode on `Resized` then acts inside WebKit's fullscreen window: CI saw the page
  shrunk to 480×600 in a black fullscreen window, and at 720 px phone mode's reload ending the
  fullscreen. Lay out the page on macOS only through `bar::apply_layout`, which asks
  `platform::page_in_webkit_fullscreen` first.
- *(LJ14.1)* **On WebKitGTK 2.50.4 only `enable-fullscreen` FALSE avoids the abort:** with it the
  bare window survives the click on two 22.04 runners (run 39253920848), because the page doesn't
  even get the Fullscreen API (`requestFullscreen` is missing: no `fullscreenerror`, no rejected
  promise). Little Joan sets it on the page webview below 2.52 (`fullscreen-off` in the smoke log).
  **A force-cancelled job keeps no log** (`gh api …/force-cancel`): its log blob is never written,
  so the diagnostics it ran are lost — cancel normally, or let it end.
- *(LJ15, read in wry 0.57)* **On Linux wry's download handler belongs to the web context, and the
  first one wins.** Every webview's attributes default to an allow-all `download_started_handler`,
  and on WebKitGTK wry registers it on the shared `WebKitWebContext` once per webview built (Tauri
  keeps one context per data folder for the app's lifetime on Linux, shared by the bar, the page and
  — in installed builds — Settings). WebKitGTK's `decide-destination` stops at the first handler
  that returns TRUE, and wry's always do: the first webview ever built decides every download's
  destination, Tauri's `on_download` on a later webview never sees `Requested`, each new webview adds
  one more `finished` report, and wry's per-registration `failed` flag is never reset. LJ15 handles
  the page's downloads in `platform/linux.rs` instead (one hook per context, wry's deciders
  disconnected). On Windows and macOS `on_download` is per webview and works. Also: WebKitGTK 4.1's
  `webkit_download_set_destination` takes a `file://` URI on 2.50 (newer versions take a path too).
  *(LJ15's CI)* **On WebKitGTK 2.50.4 a `decide-destination` handler connected in the context's
  `download-started` never runs** (and wry's weren't there either), while 2.52 runs it: decide in an
  emission hook on `decide-destination` instead, which runs before every handler of every download.
- *(LJ15's CI)* **A WKDownload outlives its WKWebView.** It lives in WebKit's network process: after
  the window closed (web view unloaded and destroyed) a download went on writing its file. wry keeps
  the WKDownload objects to itself; LJ15 notes them through a wrapped `-[WKDownload setDelegate:]` to
  cancel them on close.
- *(LJ15's CI)* **wry's WKWebView shows any attachment it can display** (a PDF, an image, text):
  its response policy downloads only what `canShowMIMEType` refuses, `Content-Disposition:
  attachment` aside. WebView2 and WebKitGTK download it.
- *(LJ13)* **An AppImage without FUSE (`APPIMAGE_EXTRACT_AND_RUN=1`) deletes its extracted folder
  when it exits** — `$TMPDIR/appimage_extracted_<md5>`, shared by every launch of the same image.
  A second instance (single-instance's remote control) exiting would delete the running first
  instance's files; `NO_CLEANUP=1` keeps them. And its `AppRun` is linuxdeploy's bash script
  (sourcing the GStreamer and GTK hooks, then `exec …/AppRun.wrapped`): gdb run on the AppImage
  follows the runtime, not the app, so CI attaches gdb to the app's pid instead (ptrace_scope 0 on
  the runner), and the app's process name is `AppRun.wrapped`, not `little-joan` — find it by
  `/proc/<pid>/exe` (`smoke-linux.mts`).
- *(LJ13)* **An AppImage leaves EGL, GL, GBM, ALSA, X11 and fonts to the system** (linuxdeploy's
  excludelist). GitHub's `ubuntu-24.04` image has no `libEGL.so.1`, so the AppImage didn't start
  there (`error while loading shared libraries`, exit 127) until the job installed `libegl1` & co.
  Every desktop has them; a minimal container wouldn't.
- *(LJ17, read in the `.deb` and xdg-desktop-portal's source)* **The portal can't place "Little
  Joan.desktop".** Tauri's `.deb` names its desktop file after the product (`/usr/share/applications/
  Little Joan.desktop`). xdg-desktop-portal gives a host app an id only from its systemd unit
  (`app-gnome-Little\x20Joan-<n>.scope` → "Little Joan" when the desktop started it; none from a
  terminal), and GlobalShortcuts' CreateSession refuses an app without one ("An app id is
  required"). The host registry (`org.freedesktop.host.portal.Registry.Register`, x-d-p 1.19.1+)
  sets one, but only when a desktop file of that name exists (`REQUIRE_GAPPINFO`), only before any
  other portal call on the connection, and only once. So the `.deb` also installs
  `io.github.memoliertorerr.littlejoan.desktop` (`NoDisplay=true`), and the portal code registers on
  its own connection, taking a fresh one if the registry refuses. The AppImage installs no desktop
  file. Don't rename the product to fix the name: the login entries are named after it (LJ11).
- *(LJ17, read in zbus 5.13.2)* **ashpd's `tokio` feature switches zbus to tokio for every user of
  it** (Cargo features unify: tauri-plugin-single-instance and -opener share the one zbus). zbus's
  blocking API then runs on a tokio runtime of its own (`utils::block_on`), which panics ("Cannot
  start a runtime from within a runtime") when called inside another tokio runtime, and
  single-instance's callback runs on that runtime's threads: never call zbus's blocking API or
  `tauri::async_runtime::block_on` from the callback, or from an async command (the opener already
  puts its blocking zbus calls on a thread of their own).
- *(LJ17, read in xdg-desktop-portal-gnome)* **GNOME's preferred trigger is turned into a GTK
  accelerator** (`CTRL+SHIFT+space` → `<ctrl><shift>space`, modifier names upper case, the key an
  xkb name), and its `trigger_description` is a translated sentence ("Press <ctrl><shift>space"):
  show it as the desktop wrote it, never parse it.
- *(LJ18.1)* **Tauri names its Linux packages per architecture, but not alike:** `_amd64.deb` /
  `_amd64.AppImage` on x86_64, `_arm64.deb` / `_aarch64.AppImage` on ARM. The release names follow
  Debian for the `.deb` (`-linux-amd64.deb`, `-linux-arm64.deb`) and the machine for the AppImage
  (`-linux-x86_64.AppImage`, `-linux-aarch64.AppImage`) — rename from the artifact, never by pattern
  on Tauri's name. The AppImage smoke builds its name from `uname -m`.
- *(LJ18.1)* **The AppImage tooling is downloaded at build time, per architecture**, from moving
  releases: `AppRun-<arch>` (tauri-apps/binary-releases `apprun-old`), `linuxdeploy-<arch>.AppImage`
  (`linuxdeploy-07333c6`) and `linuxdeploy-plugin-appimage-<arch>.AppImage` (linuxdeploy's
  `continuous`). If an AppImage build breaks one day with no change here — on one architecture only,
  say — look at those first (the package log prints each URL).
- *(round 3 research, to watch)* **macOS 26 / 27: an `.accessory` app's element fullscreen may
  leave the Dock hidden** (Apple Developer Forums thread 846037). Little Joan is an Accessory app
  and uses WebKit's element fullscreen; not seen here yet. If the owner sees the Dock missing after
  a video's fullscreen, look there first.

---

## Later — designed, NOT built (v0.1 → v0.3)

Everything below is intentionally out of scope, recorded so nobody half-builds it. Rows that v0.2
or round 3 picked up point at their step and stay here for the record.

| Item | Design | What it needs |
|---|---|---|
| **Start at login** → *v0.2, LJ11* | Settings checkbox → tauri-plugin-autostart (LaunchAgent / registry Run key / XDG autostart) | New dependency, settings field, per-OS human check |
| **Keep the page after close** | Settings "Keep page for N minutes": hide instead of destroy, destroy on a timer | Breaks the "no webview at rest" invariant for N minutes — needs a measured memory budget |
| **Downloads** → *v0.2, LJ15 (status in the URL bar, no notifications)* | `on_download` → save to the OS Downloads folder, tray notification | Handler + notification permission per OS |
| **Find in page** (zoom → *v0.2, LJ16: done*) → *v0.3, LJ49* | Native keys (Ctrl/Cmd+F) → engine APIs where they exist (WebView2 Find, WKWebView find, WebKitGTK find controller); the box lives in the URL bar | Per-engine glue |
| **Popups inside the app (OAuth)** → *v0.3, LJ47–LJ48* | `NewWindowResponse::Create` with a second small window for `window.open` with features | Lifecycle for child windows; hibernation must close them too. Some "Sign in with …" flows break today because the popup opens in the default browser |
| **Code signing / notarization / SmartScreen reputation** → *macOS: v0.2, LJ18* | Apple Developer ID + notarytool in CI; Windows Authenticode certificate | Windows: SignPath Foundation if the user's application is approved (Deferred follow-ups) |
| **Auto-update and GitHub Releases** | tauri-plugin-updater with a signed `latest.json` on Releases | Signing keys, release workflow |
| **Wayland-native global shortcut** → *v0.2, LJ17: done* | XDG GlobalShortcuts portal (via ashpd) when `WAYLAND_DISPLAY` is set | New dependency, portal support varies by desktop |
| **rpm package, MSI, Windows ARM64, ~~Linux ARM64~~** (Linux ARM64 → *LJ18.1: done* — an `arm64` `.deb` and an `aarch64` AppImage, built and smoke-tested on GitHub's arm64 runners) | Extra `--bundles` / targets in the package job | CI time + smoke coverage per artifact |
| **Hide Tauri's IPC object from remote pages** | Remote pages can see `__TAURI_INTERNALS__` (every call is denied); a future Tauri option or a separate content webview without IPC injection | Upstream support or a multi-webview layout |
| **Incognito mode** | `incognito(true)` on the builder for a "private" summon variant (second shortcut) | Settings + second shortcut |
| *(v0.2)* **Zoom extras** | Ctrl+mouse wheel and pinch zoom; zoom remembered per site | Wheel/gesture hooks per engine; a per-origin store |
| *(v0.2)* **Downloads list** | A small list of recent downloads (in Settings or the tray), pause/resume, "ask where to save" | UI + state across summons |
| *(v0.2)* **Build provenance for release assets** | `actions/attest-build-provenance` in the release run; users check with `gh attestation verify` | A release-workflow step; free, no secrets |
| *(round 2)* **Search suggestions from the engine while typing** | Rust fetches the engine's suggest endpoint (a webview fetch hits CORS), a setting, off by default | An HTTP client dependency, privacy wording |
| *(round 2)* **Third-party quick answers** | An extension-provided provider running on every keystroke in a no-IPC context | A sandbox that can be called per keystroke safely; timeouts |
| *(round 2)* **Extension gallery / signing** | A list of reviewed community extensions in the repository; signed bundles | Review process, a trust model |
| *(round 2)* **Custom window chrome on Windows and Linux** | Frameless window with our own close / minimize, resize borders | Per-OS hit-testing; accessibility |
| *(round 2)* **One user agent per signed-in site** | Remember the agent a site was signed in with and keep it across phone-mode switches | Per-host user agent at navigation time |
| *(round 3, deferred by the owner)* **macOS saved passwords and passkeys** | Passkeys (WebAuthn) in an app's WKWebView that isn't Safari need Apple's restricted web-browser entitlement (`com.apple.developer.web-browser.public-key-credential`), which Apple grants on request to apps that are web browsers, and which needs Developer ID signing with a provisioning profile (LJ18 first); WKWebView's password AutoFill for embedding apps isn't dependable. LJ52 records what CI's `?webauthn=1` page showed | Apple's entitlement (a request to Apple), signing with a profile, then the engine's own UI — never a password store of Little Joan's own (Ground Rule 26) |
| *(round 3)* **Saved logins on Linux** | WebKitGTK has no password manager; one would be Little Joan's own (libsecret + UI) | A new dependency and a credential store to secure — big; against Ground Rule 26's spirit |
| *(round 3, the owner's decision)* **Measuring a page's width on macOS and Linux** | Skip a phone-mode reload when the page already fits the window, as Windows does with CDP's `Page.getLayoutMetrics` (LJ40) | A script in the page (an isolated world, still app code in a web page) — against Ground Rule 10 unless the owner decides otherwise |
| *(round 3)* **Viewport emulation** | A mobile viewport for narrow windows (Windows: CDP `Emulation.setDeviceMetricsOverride { mobile: true }`) | Windows only — macOS has only private SPI; Little Arc doesn't do it, and phase 19 reflows instead |
| *(round 3)* **Routing links by site** | Rules that send some sites from Little Joan straight to the main browser (or the reverse) | A rules editor; LJ46's handoff is the hook |
| *(round 3)* **Keyboard extras** | Chorded keys, keys for extension commands, per-site keys | The keymap (LJ43) is the base |
| *(round 3)* **Pinned sites extras** | Reorder by drag, folders, pinned extension commands | UI in the command bar's empty state |
| *(round 3)* **Find extras** | Whole word, regex, find in the bar's own screens | Engine support differs (macOS has no count) |
| *(round 3)* **Emoji skin tones and a full emoji set** | The full Unicode set with CLDR names | A data file and its licence notice (Unicode License v3); the bundled picker ships a curated set written for Little Joan |

**When Later work starts:** v0.1's order (start at login, then downloads, then signing) became
v0.2; popups inside the app and find in page became v0.3. What's left, roughly by value:
auto-update (best after Windows signing), macOS passwords and passkeys (after the signing and
Apple's entitlement), then the rest.
