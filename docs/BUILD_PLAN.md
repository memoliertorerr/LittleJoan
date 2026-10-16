# Little Joan — Build Plan (v0.1)

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
| LJ4 | Page behavior: navigation guard, titles, new windows → default browser, page keys (Win / Linux / macOS menu) | strong | auto | ⬜ todo |
| LJ5 | Fullscreen video: WebView2 handler, engine checks, smoke test page | strong | auto | ⬜ todo |
| LJ6 | Settings window: shortcut recorder, search engine, translate language; live shortcut re-register | standard | auto | ⬜ todo |
| LJ7 | Translate page + open in default browser (tray, macOS menu, keys) | standard | auto | ⬜ todo |
| LJ8 | Packaging in CI: NSIS (Windows), universal DMG (macOS), deb + AppImage (Linux), artifacts uploaded | strong | auto | ⬜ todo |
| LJ9 | Install-and-run smoke in CI: install what users download, drive it, screenshots, memory check | strong | auto | ⬜ todo |
| LJ10 | Docs + human checkpoint: README, architecture, checklist in the last PR | standard | checkpoint | ⬜ todo |

**Overall status: phase 1 done (PR #1); LJ3 done. Next: LJ4, LJ5, then the phase-2 PR.**

**Model note:** phases 1, 2, and 4 need the strongest model available — phase 1 sets the shape
every later step inherits, phase 2 is native per-OS code the local Mac cannot compile for
Windows or Linux (CI is the only compiler for those paths), and phase 4 is CI packaging, where
mistakes only show up after a 15-minute round trip. Phase 3 is ordinary UI + glue on top of
settled contracts and can run on a standard model.

**Phases** — one pull request each, stacked (each phase branches from the previous one):
1. **LJ1 → LJ2** · strong — `lj/p1-foundation` → PR into `main`. Review: the toolchain/caching
   layout, the pure-core contracts (URL normalization rules, settings shape, toggle table).
2. **LJ3 → LJ5** · strong — `lj/p2-summon` → PR into `lj/p1-foundation`. Review: window
   lifecycle (nothing alive at rest), the no-IPC-for-remote-pages rule, per-OS native glue.
3. **LJ6 → LJ7** · standard — `lj/p3-settings` → PR into `lj/p2-summon`. Review: settings UI,
   shortcut re-registration with rollback, translate.
4. **LJ8 → LJ10** · strong — `lj/p4-ship` → PR into `lj/p3-settings`. Review: installers, the
   CI smoke screenshots, and the **human checklist**.

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
3. Push. If the phase has no PR yet, open it as a draft (base = previous phase's branch, or
   main for phase 1) so CI runs. Watch CI with
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
| Windows + Linux code paths (`#[cfg(windows)]`, `#[cfg(target_os = "linux")]`) | CI (`verify` job on ubuntu-latest, windows-latest, macos-latest) | All three jobs green on the pushed commit |
| Installers build | CI `package` job (LJ8) | NSIS `.exe`, universal `.dmg`, `.deb`, `.AppImage` uploaded as artifacts |
| Installed app launches, summons, loads a page, element fullscreen toggles the window, closing frees the webview | CI `smoke` job (LJ9), agent reads the logs and looks at the screenshots | Smoke log shows the expected events; screenshots show the start page and a loaded page |
| **Nothing regressed** | you, in each PR | Earlier phases' behaviors in the PR's checklist still hold |
| Global shortcut on real desktops (Windows, macOS, Linux X11; Wayland fallback) | **you** — LJ10 checkpoint | The shortcut summons and dismisses the window; focus lands in the field |
| Resizing feel, window placement on multi-monitor, theme | **you** — LJ10 | The window opens on the monitor with the cursor, resizes smoothly, remembers its size |
| Fullscreen video on real hardware (YouTube) | **you** — LJ10 | Video fills the screen, Esc returns to the window, audio/video play |
| Memory at rest in Task Manager / Activity Monitor / `ps` | **you** — LJ10 (CI gives a first number) | No webview helper processes while hibernating |
| Gatekeeper (unsigned macOS), SmartScreen (unsigned Windows), tray on GNOME | **you** — LJ10 | The documented workaround opens the app |

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

---

## Deferred follow-ups (cross-step, live tracker)

> Things a step noticed but were out of its own scope. Kept here so they don't get buried in
> per-step notes. Strike through when genuinely handled; don't silently delete lines.

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
- **Google Translate's proxy picks the UI language from the requester's IP** (`_x_tr_hl=it`
  came back from a plain request), so the translate URL passes `hl=<lang>` too.

---

## Later — designed, NOT built in v0.1

Everything below is intentionally out of scope, recorded so nobody half-builds it.

| Item | Design | What it needs |
|---|---|---|
| **Start at login** | Settings checkbox → tauri-plugin-autostart (LaunchAgent / registry Run key / XDG autostart) | New dependency, settings field, per-OS human check |
| **Keep the page after close** | Settings "Keep page for N minutes": hide instead of destroy, destroy on a timer | Breaks the "no webview at rest" invariant for N minutes — needs a measured memory budget |
| **Downloads** | `on_download` → save to the OS Downloads folder, tray notification | Handler + notification permission per OS |
| **Find in page, zoom** | Native keys (Ctrl/Cmd+F, +/-) → engine APIs where they exist (WebView2 has zoom; WKWebView `pageZoom`) | Per-engine glue; WebKitGTK find controller |
| **Popups inside the app (OAuth)** | `NewWindowResponse::Create` with a second small window for `window.open` with features | Lifecycle for child windows; hibernation must close them too |
| **Code signing / notarization / SmartScreen reputation** | Apple Developer ID + notarytool in CI; Windows Authenticode certificate | Paid certificates and secrets in CI — the user's call |
| **Auto-update and GitHub Releases** | tauri-plugin-updater with a signed `latest.json` on Releases | Signing keys, release workflow |
| **Wayland-native global shortcut** | XDG GlobalShortcuts portal (via ashpd) when `WAYLAND_DISPLAY` is set | New dependency, portal support varies by desktop |
| **rpm package, MSI, Windows ARM64, Linux ARM64** | Extra `--bundles` / targets in the package job | CI time + smoke coverage per artifact |
| **Hide Tauri's IPC object from remote pages** | Remote pages can see `__TAURI_INTERNALS__` (every call is denied); a future Tauri option or a separate content webview without IPC injection | Upstream support or a multi-webview layout |
| **Incognito mode** | `incognito(true)` on the builder for a "private" summon variant (second shortcut) | Settings + second shortcut |

**When Later work starts:** start at login first (it makes the shortcut useful after a
reboot), then downloads, then signing (it removes the scariest install friction).
