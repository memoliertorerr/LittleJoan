# Little Joan — Build Plan (v0.1, v0.2)

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
> **Never use Arc's name, logo, or artwork as branding.** "Inspired by Little Arc" in the README
> is the only mention. The icon is original.
>
> **Autopilot:** one step = one commit, made by the agent only when verify is green. One
> phase = one pull request, reviewed and merged by a human. The agent stops at checkpoints, at
> the end of each phase, and at any stop condition.

---

## Progress Tracker

> Each session flips its own row to ✅ in the same commit as the step's change. Read this table
> at the start of a session to know where to resume. `auto` steps run back to back;
> `checkpoint` steps end with a stop for a human check.

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
| LJ13 | *(v0.2)* AppImage built on Ubuntu 24.04 (WebKitGTK 2.52); the AppImage gets the full smoke | strong | auto | ⬜ todo |
| LJ14 | *(v0.2)* WebKitGTK older than 2.52: fullscreen the window instead of aborting; an Ubuntu 22.04 smoke job | strong | auto | ⬜ todo |
| LJ15 | *(v0.2)* Downloads: save to the Downloads folder; status and "Show in folder" in the URL bar | strong | auto | ⬜ todo |
| LJ16 | *(v0.2)* Page zoom: Ctrl/Cmd + / − / 0 through native keys and `Webview::set_zoom` | standard | auto | ⬜ todo |
| LJ17 | *(v0.2)* Wayland-native global shortcut through the XDG GlobalShortcuts portal | strong | auto | ⬜ todo |
| LJ18 | *(v0.2)* macOS Developer ID signing and notarization in CI (starts once the user's secrets exist) | strong | auto | ⬜ todo |
| LJ19 | *(v0.2)* Release v0.2.0: versions, CHANGELOG, README; the user's checklist | standard | checkpoint | ⬜ todo |
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
| LJ32 | *(v0.2, round 2)* Bundled games: Snake and 2048 | standard | auto | ⬜ todo |
| LJ33 | *(v0.2, round 2)* Bundled productivity: Notes, Focus timer, World clock | standard | auto | ⬜ todo |
| LJ34 | *(v0.2, round 2)* Bundled page actions and tools: QR code, Clean link, Wayback, Markdown link, JSON + text + color tools | standard | auto | ⬜ todo |
| LJ35 | *(v0.2, round 2)* Docs, screenshots, the user's checklist; v0.2.0-beta.2 on the user's word | standard | checkpoint | ⬜ todo |

**Overall status: v0.1.0 (2026-10-17). LJ1–LJ10 passed the user's own check on macOS. Added
mid-plan: LJ10.1–LJ10.2 (the always-visible URL bar and the project files) and LJ10.3 (phone mode
for narrow windows, and layout tests at scale factors 1, 1.5 and 2). PRs #1–#7 were merged into
`main` at the user's request. The GitHub Release `v0.1.0` carries the Windows, macOS and Linux
installers CI built from the tagged commit. Still open: Linux fullscreen on Ubuntu 22.04 and in
the AppImage (WebKitGTK 2.50.4 aborts on element fullscreen; see Deferred follow-ups).**

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
"Security review"); the merge decision is the owner's. Next after it: phase 17 (LJ32 → LJ35).**

**Model note:** phases 1, 2, and 4 need the strongest model available — phase 1 sets the shape
every later step inherits, phase 2 is native per-OS code the local Mac cannot compile for
Windows or Linux (CI is the only compiler for those paths), and phase 4 is CI packaging, where
mistakes only show up after a 15-minute round trip. Phase 3 is ordinary UI + glue on top of
settled contracts and can run on a standard model. In v0.2 the strong steps are the ones that
touch three OSes' native APIs or CI packaging (LJ11, LJ13–LJ15, LJ17, LJ18); LJ12, LJ16 and
LJ19 are UI, key tables and release chores on settled code.

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
   (b) you never force-push, rewrite pushed history, merge a PR, or delete a branch;
   (c) you stop at checkpoints, at the end of the phase, and at every stop condition in
       Ground Rules — list them;
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
5. If the step's gate is `checkpoint`, update the PR, tell me exactly what to check, and stop.
   If it is `auto`, continue to the next step.

Stop immediately at any stop condition in Ground Rules and tell me which one.
After the last step of the range: push the branch, fill the PR description from the template,
mark it ready for review once CI is green, then stop.
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
| Installers build | CI `package` job (LJ8) | NSIS `.exe`, universal `.dmg`, `.deb`, `.AppImage` uploaded as artifacts |
| Installed app launches, summons, loads a page, element fullscreen toggles the window, closing frees the webview | CI `smoke` job (LJ9), agent reads the logs and looks at the screenshots | Smoke log shows the expected events; screenshots show the start page and a loaded page |
| **Nothing regressed** | you, in each PR | Earlier phases' behaviors in the PR's checklist still hold |
| Global shortcut on real desktops (Windows, macOS, Linux X11; Wayland fallback) | **you** — LJ10 checkpoint | The shortcut summons and dismisses the window; focus lands in the field |
| Resizing feel, window placement on multi-monitor, theme | **you** — LJ10 | The window opens on the monitor with the cursor, resizes smoothly, remembers its size |
| Fullscreen video on real hardware (YouTube) | **you** — LJ10 | Video fills the screen, Esc returns to the window, audio/video play |
| Memory at rest in Task Manager / Activity Monitor / `ps` | **you** — LJ10 (CI gives a first number) | No webview helper processes while hibernating |
| Gatekeeper (unsigned macOS), SmartScreen (unsigned Windows), tray on GNOME | **you** — LJ10 | The documented workaround opens the app |
| *(v0.2)* Login entry written and removed per OS | CI `smoke` (LJ11) | The registry value / LaunchAgent plist / autostart `.desktop` appears after `--start-at-login on`, points at the installed binary, and is gone after `off` |
| *(v0.2)* Start at login across a real logout or reboot; the first-run welcome | **you** — phase 6 PR | The icon is there after logging in and the shortcut works; a fresh profile shows the welcome once |
| *(v0.2)* Fullscreen on a real Ubuntu 22.04 desktop (the `.deb`), and the AppImage on 24.04 | **you** — phase 7 PR, if you have the machines | 22.04: no crash, the window fills the screen; 24.04 AppImage: the video fills the screen |
| *(v0.2)* A real download (a PDF from a site), "Show in folder", zoom keys | **you** — phase 8 PR | The file is in Downloads under its own name; the folder opens with it selected; zoom steps and resets |
| *(v0.2)* Wayland-native shortcut (GNOME 48+ or KDE Plasma, Wayland session) | **you** — LJ17 (CI has no portal backend) | The desktop asks once to allow the shortcut; it then summons and dismisses the window |
| *(v0.2)* Gatekeeper on a DMG downloaded with a browser (notarized build) | **you** — LJ18 / LJ19 | It opens without any warning or `xattr` step |
| *(round 2)* Cookies and storage survive close, quit and relaunch | CI `smoke` (LJ20) | The smoke page reports its persistent cookie, session cookie and localStorage value back after each step, on all three OSes |
| *(round 2)* No IPC for the page webview, ever | `cargo test` (LJ22) + smoke (LJ29) | A Rust test reads every capability file and finds no `joan-page`; the test extension reports `ipc=denied` |
| *(round 2)* How the command bar, toolbar, history view, themes and extensions look | the agent from CI screenshots on all three OSes (each step names its screenshots), then **you** — LJ35 | Screenshots match the step's description in light and dark; at LJ35 it feels like Little Arc on your Mac |
| *(round 2)* Suggestions feel right with real history | **you** — LJ35 | After a few days of use, the empty command bar shows the sites you actually open most, and typing finds them fast |

The agent self-verifies everything a compiler, a test suite, and CI (including headless
screenshots of the installed app) can show. Everything that needs a real desktop, real input
devices, real GPUs, or a real user session is the human's, collected in LJ10's checkpoint.

---

## Ground Rules

1. **Branches:** one per phase, `lj/p<n>-<name>`, each cut from the previous phase's branch
   (phase 1 from `main`). PRs are stacked: phase 1 → `main`, phase n → phase n-1's branch.
   After the bootstrap commit, **nothing is ever committed to `main`** by the agent.
2. **One step = one commit**, message `lj-step-N: <message>`, made **only when
   `npm run verify` is green**. Never commit red.
3. **CI-only failures:** the Mac cannot compile the Windows/Linux code paths, so CI is part of
   each step's verification. If CI goes red after a step was pushed, fix it in a follow-up
   commit `lj-step-N-fix: <what>` (local verify green first) and note it in the step's
   implementation notes. Never amend or rebase a pushed commit.
4. **Push and PR:** the agent pushes after every step commit and opens the phase PR as a draft
   after the first one. It never force-pushes, never rewrites pushed history, never merges any
   PR, and never deletes a branch. `gh` commands always pass `--repo memoliertorerr/LittleJoan`.
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

Each later phase:

```bash
git switch lj/p<n-1>-<name> && git pull --ff-only
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

> Phase 7. Branch `lj/p7-linux-fullscreen` from `lj/p6-first-run` (or from `main` if phase 6 is
> merged).

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
fully". Push, open the phase-7 PR as a draft (base: lj/p6-first-run, or main if phase 6 is
merged), and watch CI until every job is green. Download linux-appimage's fullscreen.png into
test-results/ci/, look at it, then delete it. If a job is still red after two honest fix
attempts, stop and report.
```

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
   fullscreen by other means ends it and brings the bar back. Esc goes through keys.rs and the
   GTK key hook, and only while the fallback is active. FullscreenTracker's rule holds: undo
   only a fullscreen we caused. WebKitGTK 2.52+ keeps today's behavior untouched.
3. CI smoke: add a permanent matrix entry `linux-22.04` (ubuntu-22.04, artifact
   little-joan-linux, the .deb). Its smoke-log rules, unit-tested: `fullscreen-fallback`, then
   `fullscreen true`, then `fullscreen false` after the page loads again, and no abort.
   Everything else matches the `linux` entry. The 24.04 entries must NOT log
   fullscreen-fallback.
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
Then mark the PR ready and stop.
```

### LJ15 — Downloads · strong · auto

> Phase 8. Branch `lj/p8-downloads-zoom` from `lj/p7-linux-fullscreen` (or from `main`).

**Verify:** `npm run verify` green, including Rust tests for a pure `downloads.rs`. The file
name comes from the engine's suggested name, else the URL's last path segment, else
`download`. It is cleaned: no path separators, no `..`, no control characters, no Windows
reserved names (`CON`, `NUL`, `COM1`…) or trailing dots and spaces, capped at 200 bytes. It is
made unique in the folder: `report.pdf` → `report (1).pdf`, `archive.tar.gz` →
`archive (1).tar.gz`. Only `http`, `https`, `blob` and `data` downloads are accepted. Vitest
tests for the bar's status logic in `ui/bar-logic.ts`: downloading → saved → cleared after 8 s;
failed; an address edit in progress is never disturbed. Capabilities: the bar gains only
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
- The URL bar (ui/bar.html, ui/bar.ts, ui/bar-logic.ts) gets a small status area at its right
  edge, pushed from Rust like the address: bar.eval("window.littleJoan.setStatus(<json>)").
  *(Round 2 runs first: by then the bar is LJ24's Little Arc-style toolbar. Put the status
  area there, left of the open-in-browser button, and keep LJ22's rule that only the bar holds
  trusted UI.)*
  It shows "Downloading lj-smoke.bin…", then "lj-smoke.bin saved" with a "Show in folder"
  button, or "Download failed". It clears after 8 s, or at once when the user starts editing
  the address. The button calls a new command `reveal_download()`, which reveals the last
  saved file with tauri_plugin_opener's reveal_item_in_dir. It is granted to the bar only and
  added to build.rs's app manifest. LJ16 reuses this status area for the zoom level, so keep it
  generic (text + optional action).
- Closing the window during a download cancels it. Nothing keeps a webview alive for it
  (Ground Rule 9). No notification plugin.
- Windows: WebView2's own download flyout must not appear on top. Check it in the smoke
  screenshot; if wry doesn't suppress it, set `Handled` on the DownloadStarting event's
  arguments (ICoreWebView2DownloadStartingEventArgs) in platform/win.rs, keeping wry's
  destination.

Smoke (scripts/ci/smoke.mts + smoke-log.mts and its test; the smoke server in smoke.mts): add a
route that answers with a fixed 1 KB body, Content-Type application/octet-stream and
Content-Disposition attachment; filename="lj-smoke.bin". Use `--show` on that URL twice. Check
the log events, the bytes of Downloads/lj-smoke.bin and of Downloads/lj-smoke (1).bin, and take
a screenshot of the bar (download.png). Delete both files afterwards (the runner is
disposable, but stay tidy).

Files: src-tauri/src/{downloads.rs (new, pure), bar.rs, commands.rs, lib.rs, smoke.rs,
platform/win.rs if needed}, src-tauri/build.rs, src-tauri/capabilities/bar.json, ui/{bar.html,
bar.ts, bar-logic.ts (+ test), style.css}, scripts/ci/{smoke.mts, smoke-log.mts (+ test)},
README, CHANGELOG ([Unreleased] → Added), docs/architecture.md (the page, the URL bar,
commands, smoke log), this plan (Deferred follow-ups: "Downloads" handled by LJ15).

Do NOT add a downloads list, a progress bar, pause/resume, or a "where to save" dialog (Later).

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ15 to ✅, add notes, and
commit: "lj-step-15: downloads go to the Downloads folder, with status in the URL bar". Push,
open the phase-8 PR as a draft (base: lj/p7-linux-fullscreen, or main), and watch CI until
every job is green (Ground Rule 3; two honest attempts, then stop). Look at each OS's
download.png.
```

### LJ16 — Page zoom · standard · auto

**Verify:** `npm run verify` green, including Rust tests for a pure `zoom.rs`: levels 50, 67,
75, 80, 90, 100, 110, 125, 150, 175, 200, 250, 300 %; zooming in or out steps to the next level
from any value, stops at the ends, and reset goes to 100 %. `keys.rs` tests: Ctrl+= / Ctrl+Shift+=
/ Ctrl++ (keypad) zoom in, Ctrl+- (and keypad -) zoom out, Ctrl+0 resets (Windows and Linux).
The exact-match rules still hold (Ctrl+Alt never matches). macOS: View menu items Zoom In ⌘+,
Zoom Out ⌘−, Actual Size ⌘0 in `menu.rs`. `zoom_hotkeys_enabled` appears nowhere in
`src-tauri/src`. CI green; the Linux smoke sends ctrl+equal with xdotool, and the Windows smoke
sends it with SendKeys `^=`. The page then reports a smaller `innerWidth` (about 655 instead of
720 at 110 %), and after Ctrl+0 it reports 720 again. On macOS the check is required only if the
synthetic key reached the page (the same rule as the fullscreen click).
**Commit:** `lj-step-16: page zoom with Ctrl/Cmd + / − / 0`

```text
Add page zoom. Keys only, plus a short status in the URL bar.

Design (decided with the user; do not relitigate):
- The keys are caught natively like the other page keys (keys.rs → page_actions.rs:
  PageAction::ZoomIn / ZoomOut / ZoomReset): WebView2 AcceleratorKeyPressed (platform/win.rs),
  the GTK key hook (platform/linux.rs), and a View menu in menu.rs on macOS. They call
  Webview::set_zoom on the PAGE webview with the level from zoom.rs.
- NEVER enable Tauri's zoom_hotkeys_enabled on joan-page. On macOS and Linux it injects a
  script into the page that needs core:webview:allow-set-webview-zoom, which is IPC for remote
  pages (Ground Rule 10; Landmines). On Windows it is WebView2's IsZoomControlEnabled, which
  stays off so our levels are the only ones.
- Zoom is per window: it survives navigations within the window, starts at 100 % on every
  summon, and is never saved. Keep the current level next to the window's other per-window
  state in bar.rs or joan.rs.
- Each change shows "Zoom 110 %" for 1.5 s in the bar's status area (from LJ15). At 100 % it
  shows "Zoom 100 %" once, then nothing.
- Phone mode is unaffected: its 600 px threshold is the window's logical width, not the page's
  CSS width.

Smoke: after the wide (720×600) viewport report, send Ctrl+= (Linux: `xdotool key ctrl+equal`;
Windows: SendKeys "^="; macOS: a CGEvent key press, best-effort), and require the next viewport
report to be narrower than 720. Then send Ctrl+0 and require 720 again. Add the rules to
smoke-log.mts with tests.

Files: src-tauri/src/{zoom.rs (new, pure), keys.rs, page_actions.rs, menu.rs, platform/win.rs,
platform/linux.rs, bar.rs}, ui/bar-logic.ts (+ test) if the status needs it,
scripts/ci/{smoke.mts, smoke-log.mts (+ test)}, README (page keys table), CHANGELOG
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
- ⌘+ / ⌘− / ⌘0 zoom and reset, with the level shown briefly in the bar.
- The must-not-disturb list.
Then mark the PR ready and stop.
```

### LJ17 — Wayland-native global shortcut · strong · auto

> Phase 9. Branch `lj/p9-wayland-shortcut` from `lj/p8-downloads-zoom` (or from `main`).

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
- Settings window: on the portal path, replace the recorder with the trigger the portal reports,
  plus "Change it in your desktop's keyboard settings". On other paths nothing changes.
- StartInfo.waylandNoShortcut (LJ12) becomes false once the portal registers the shortcut.

Files: src-tauri/Cargo.toml (+ Cargo.lock), src-tauri/src/{shortcut.rs and/or
platform/portal.rs, lib.rs, commands.rs, smoke.rs}, ui/settings.{html,ts}, ui/settings-logic.ts
(+ test), possibly src-tauri/tauri.conf.json (desktop file), README (Wayland), CHANGELOG
([Unreleased] → Added), docs/architecture.md (global shortcut), this plan (Later: the Wayland
row moved here).

Do NOT remove the `--toggle` fallback or the X11 path.

Run `source scripts/env.sh && npm run verify`. If it is green, flip LJ17 to ✅, add notes, and
commit: "lj-step-17: Wayland-native global shortcut through the XDG GlobalShortcuts portal".
Push, open the phase-9 PR as a draft (base: lj/p8-downloads-zoom, or main), and watch CI until
every job is green (two honest attempts, then stop). This ends phase 9. Needs human check:
- On a Wayland session with GNOME 48+ or KDE Plasma: install the .deb and launch. The desktop
  asks once to allow the shortcut (or shows it in its shortcut settings). The shortcut then
  summons and dismisses the window, with no XWayland window focused.
- On Ubuntu 24.04 Wayland: the welcome shows the `--toggle` hint, as before.
- On X11, nothing changed.
- The must-not-disturb list.
Mark the PR ready and stop.
```

### LJ18 — macOS Developer ID signing and notarization · strong · auto

> Phase 10. Branch `lj/p10-macos-signing` from `lj/p9-wayland-shortcut` (or from `main`).
> **Starts only after the user has done the human pre-step below.** Check with
> `gh secret list --repo memoliertorerr/LittleJoan` (it lists names only, never values). If any
> name is missing, stop and ask.

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
human check: download the DMG artifact of the notarized run with a browser (so it gets
quarantined), open it, drag Little Joan to Applications and launch it. There should be no
Gatekeeper warning. Mark the PR ready and stop.
```

### LJ19 — Release v0.2.0 · standard · checkpoint

> Phase 11. Branch `lj/release-0.2.0` from `lj/p10-macos-signing` (or from `main` once the
> phases are merged).

**Verify:** `npm run verify` green. Version `0.2.0` in `package.json` (+ lock), `tauri.conf.json`
and `Cargo.toml` (+ `Cargo.lock`). `CHANGELOG.md` has `[0.2.0] — <date>` built from
`[Unreleased]`, with a fresh empty `[Unreleased]` above it and the link references at the
bottom. The README points at the v0.2.0 assets. CI green on the release PR. **Checkpoint:** the
user ticks the checklist. Only after the user says to merge and release: merges in order (merge
commits), `main`'s CI green, the assets renamed (`LittleJoan-0.2.0-windows-x64-setup.exe`,
`-macos-universal.dmg`, `-linux-amd64.deb`, `-linux-x86_64.AppImage`), `SHA256SUMS.txt`,
`gh release create v0.2.0 --target <that main commit> --latest`, and then the release, its tag
and one downloaded asset's checksum checked.
**Commit:** `lj-release-0.2.0: version 0.2.0, changelog and readme`

```text
Prepare v0.2.0 exactly as CLAUDE.md's "Release process" says (it was done that way for v0.1.0):
bump the three versions, turn [Unreleased] into [0.2.0] with today's date, and point the README
at the release. Do it all through this PR.

Fill the PR description from the template. Needs human check (the user's checklist):
- [ ] Mac (your Retina machine): install from the notarized DMG with no warning; the welcome on
      a fresh profile; start at login across a logout; a real download and "Show in folder";
      zoom keys; YouTube fullscreen; phone mode at a narrow width; memory at rest in Activity
      Monitor (no WebContent processes for Little Joan).
- [ ] Windows, if you have one: the installer (SmartScreen still warns, since it's unsigned),
      the welcome, start at login across a reboot, a download, zoom, YouTube fullscreen.
- [ ] Linux, if you have one: the .deb on 22.04 (fullscreen fallback), the AppImage on 24.04
      (real fullscreen), and the Wayland shortcut on GNOME 48+ or KDE.
- [ ] The must-not-disturb list.

Then STOP and wait. Merging, tagging and releasing need the user's word for each one. When they
give it: merge the open phase PRs in order with merge commits, let main's CI build the
installers, download them (`gh run download <run> --repo memoliertorerr/LittleJoan`; the AppImage
is in little-joan-linux-appimage), rename them, write SHA256SUMS.txt, and run `gh release create
v0.2.0 --repo memoliertorerr/LittleJoan --target <that main commit> --latest` with notes taken
from the CHANGELOG. Then check the release, its tag, and one downloaded asset's checksum. Flip
LJ19 to ✅ and record the release in the plan's overall status in a follow-up PR (main is never
committed to directly).
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
- **Downloads use each engine's default** (surfaced by LJ4). WebView2 saves to Downloads with its
  own UI; WKWebView and WebKitGTK ignore downloads unless the app handles them. Designed under
  Later. → *Planned as LJ15.*
- **Linux fullscreen on Ubuntu 22.04 / the AppImage** (surfaced by LJ9). WebKitGTK 2.50.4
  aborts on element fullscreen (Landmines). Options for the user: (a) build the AppImage on
  ubuntu-24.04 so it bundles WebKitGTK 2.52 (raises its glibc floor to 2.39); (b) wait for
  Ubuntu to ship a fixed 2.50.x/2.52 to 22.04; (c) report upstream with the bare-WebKitGTK
  reproduction; (d) as a stopgap, handle `enter-fullscreen` and fullscreen only the window
  (the page wouldn't know it is fullscreen — YouTube's player would stay small). → *The user
  chose (a) + (d) on 2026-10-17, with (c) as a Launchpad report they file: planned as LJ13 and
  LJ14. Waiting (b) looks unpromising: jammy's webkit2gtk hasn't moved since 2026-01-13.*
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
- **Release builds are bigger since `panic = "unwind"`** (surfaced by `lj-step-20-fix2`): CI's
  installer artifacts grew from 3.56 to 4.48 MB (macOS), 1.31 to 1.78 MB (Windows), and the Linux
  bundle by about 1 MB. Only macOS needs unwinding (wry's Objective-C exception catch); a
  target-specific `-C panic=unwind` for macOS with `abort` elsewhere would win most of it back,
  at the cost of a second place that decides the panic strategy. The owner's call; the README's
  sizes describe the v0.2.0-beta.1 downloads until the next release.
- **Double-click the toolbar to zoom the window on macOS** (surfaced by LJ24). Tauri's drag
  script turns a double click on a drag region into `internal_toggle_maximize`, which needs
  `core:window:allow-internal-toggle-maximize`; the bar has only `allow-start-dragging` (what
  LJ24 named), so a double click does nothing.
- ~~**`history_top` / `history_search` have no caller**~~ since LJ23 (the command bar uses
  `suggest`). They stay in the bar's capability for the history view (LJ25) to use or remove. →
  *Removed by LJ25: the history view uses `history_list` and filters in TypeScript.*
- **Ctrl+Y on a page is no longer redo on Windows** (surfaced by LJ25). The plan's history key
  is Ctrl+Y on Windows and Linux, caught natively before the page sees it; in WebView2 (as in
  Chromium) Ctrl+Y was also "redo" in text fields, so pages lose it there (Ctrl+Shift+Z still
  redoes). WebKitGTK has no Ctrl+Y binding. Option: Ctrl+H (Chrome's history key) on Windows and
  Linux instead — the owner's call.
- **A custom theme may show the default theme for a frame** (surfaced by LJ27). The bar applies
  the theme when `start_info` answers, a moment after its page (with `theme-defaults.css`) first
  paints, so a summon can flash the default Mist colors before the user's. Not visible in CI's
  screenshots. Option: also hand the bar the theme in an initialization script (as the settings
  window gets its tab), applied before the first paint.
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
  not the panel, decides when).
- **A cancelled navigation counts as a second visit on Windows** (surfaced by LJ29's probe):
  WebView2 raises one more `Finished` load event for the page that stays when `NavigationStarting`
  cancels a navigation, so history records that page twice. Option: ignore a `Finished` whose URL
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
  `--toggle` CLI flag (forwarded by single-instance) is the Wayland path.
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
  `.deb` on 24.04; the AppImage (built on 22.04) bundles 2.50.4. Whether real 22.04 desktops
  crash too is a human check.
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
- *(LJ29)* **Dot segments are gone before the handler runs:** engines and the `url` crate resolve
  `..` and `.` — also written `%2e` — while parsing (WHATWG URL), never above the root. A test
  that puts `/../x` in a URL tests the parser, not the path check; test `paths::clean` directly.

---

## Later — designed, NOT built in v0.1

Everything below is intentionally out of scope, recorded so nobody half-builds it. Rows that v0.2
picked up point at their step and stay here for the record.

| Item | Design | What it needs |
|---|---|---|
| **Start at login** → *v0.2, LJ11* | Settings checkbox → tauri-plugin-autostart (LaunchAgent / registry Run key / XDG autostart) | New dependency, settings field, per-OS human check |
| **Keep the page after close** | Settings "Keep page for N minutes": hide instead of destroy, destroy on a timer | Breaks the "no webview at rest" invariant for N minutes — needs a measured memory budget |
| **Downloads** → *v0.2, LJ15 (status in the URL bar, no notifications)* | `on_download` → save to the OS Downloads folder, tray notification | Handler + notification permission per OS |
| **Find in page** (zoom → *v0.2, LJ16*) | Native keys (Ctrl/Cmd+F) → engine APIs where they exist (WebView2 Find, WKWebView find, WebKitGTK find controller); the box lives in the URL bar | Per-engine glue |
| **Popups inside the app (OAuth)** | `NewWindowResponse::Create` with a second small window for `window.open` with features | Lifecycle for child windows; hibernation must close them too. Some "Sign in with …" flows break today because the popup opens in the default browser |
| **Code signing / notarization / SmartScreen reputation** → *macOS: v0.2, LJ18* | Apple Developer ID + notarytool in CI; Windows Authenticode certificate | Windows: SignPath Foundation if the user's application is approved (Deferred follow-ups) |
| **Auto-update and GitHub Releases** | tauri-plugin-updater with a signed `latest.json` on Releases | Signing keys, release workflow |
| **Wayland-native global shortcut** → *v0.2, LJ17* | XDG GlobalShortcuts portal (via ashpd) when `WAYLAND_DISPLAY` is set | New dependency, portal support varies by desktop |
| **rpm package, MSI, Windows ARM64, Linux ARM64** | Extra `--bundles` / targets in the package job | CI time + smoke coverage per artifact |
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

**When Later work starts:** v0.1's order (start at login, then downloads, then signing) became
v0.2. What's left, roughly by value: popups inside the app (OAuth), find in page, auto-update
(best after Windows signing), then the rest.
