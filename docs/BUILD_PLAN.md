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

## Deferred follow-ups (cross-step, live tracker)

> Things a step noticed but were out of its own scope. Kept here so they don't get buried in
> per-step notes. Strike through when genuinely handled; don't silently delete lines.

- ~~**First-run discoverability**~~ (surfaced by LJ3). Hibernating from the very first launch
  means a new user sees only a tray icon — and on GNOME without the AppIndicator extension,
  nothing at all. Option: when no `settings.json` exists yet, show the window once with the
  shortcut hint. → *The user chose the one-time welcome on 2026-10-17: resolved by LJ12.*
- **Esc after a click in the welcome** (surfaced by LJ12). Clicking the welcome's checkbox moves
  the keyboard into the page webview, where Esc does nothing until the URL bar has the focus
  again; Ctrl/Cmd+W, the close button and the shortcut still close. Options: give the start page
  `allow-hibernate` for Esc, or a command that hands the focus back to the bar.
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

**When Later work starts:** v0.1's order (start at login, then downloads, then signing) became
v0.2. What's left, roughly by value: popups inside the app (OAuth), find in page, auto-update
(best after Windows signing), then the rest.
