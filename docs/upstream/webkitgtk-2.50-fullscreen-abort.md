# Draft: webkit2gtk 2.50.4 on Ubuntu 22.04 aborts when a page enters element fullscreen

> **For the owner — a draft, not filed.** This is a bug report for Ubuntu's `webkit2gtk` source
> package on Launchpad, written from Little Joan's CI evidence (LJ9, LJ14). Nothing has been posted
> anywhere. To file it: on an Ubuntu 22.04 machine, `ubuntu-bug libwebkit2gtk-4.1-0` (it attaches
> the system's details), or https://bugs.launchpad.net/ubuntu/+source/webkit2gtk/+filebug, and paste
> the text below the line. If you can, first run the reproduction on a real 22.04 desktop and say
> what it did there — every result below comes from GitHub Actions' `ubuntu-22.04` runners (Xvfb).
> Little Joan's own name is optional: the reproduction needs no Little Joan code.

---

**Title:** webkit2gtk 2.50.4 (jammy) aborts the process whenever a page enters element fullscreen —
please update jammy to 2.52.x

**Package:** webkit2gtk (Ubuntu) — `libwebkit2gtk-4.1-0` 2.50.4-0ubuntu0.22.04.1 (jammy-updates),
with `libjavascriptcoregtk-4.1-0` of the same version.

### Summary

On Ubuntu 22.04, any WebKitGTK 4.1 application aborts (SIGABRT, exit status 134) the moment a page
calls `element.requestFullscreen()` — a video player's fullscreen button, for example. A minimal
PyGObject window with one WebKitWebView is enough; no other code is involved. The same reproduction
on Ubuntu 24.04 with 2.52.6-0ubuntu0.24.04.1 enters fullscreen and keeps running.

The abort does not depend on how the application handles the request: it happens with WebKitGTK's
default handling of `WebKitWebView::enter-fullscreen`, and also when the application connects a
handler that returns `TRUE` (so WebKit should decline the request) — with or without the handler
fullscreening the window itself. The signal is emitted, the handler returns, and the process aborts.
So applications have no way to avoid it.

### Steps to reproduce

1. `sudo apt install python3-gi gir1.2-webkit2-4.1`
2. Save as `/tmp/plain.html`:

   ```html
   <!doctype html><title>plain ready</title>
   <div id="s" style="position:fixed;inset:0;background:#f7c948"
        onclick="this.requestFullscreen().catch((e) => { document.title = 'plain request refused: ' + e.name; })"></div>
   <script>
   document.addEventListener("fullscreenchange", () => { document.title = "plain fullscreen=" + (document.fullscreenElement ? "on" : "off"); });
   document.addEventListener("fullscreenerror", () => { document.title = "plain fullscreenerror"; });
   </script>
   ```

3. Save as `/tmp/plain.py`:

   ```python
   import sys
   import gi
   gi.require_version("Gtk", "3.0")
   gi.require_version("WebKit2", "4.1")
   from gi.repository import GLib, Gtk, WebKit2
   mode = sys.argv[1] if len(sys.argv) > 1 else "default"
   print(f"plain ({mode}): WebKitGTK {WebKit2.get_major_version()}.{WebKit2.get_minor_version()}.{WebKit2.get_micro_version()}", flush=True)
   window = Gtk.Window()
   window.set_default_size(480, 640)
   window.set_position(Gtk.WindowPosition.CENTER)
   view = WebKit2.WebView()
   view.connect("notify::title", lambda v, _: print("title:", v.get_title(), flush=True))
   def on_enter(_view):
       print(f"enter-fullscreen: {mode}", flush=True)
       if mode == "handled-window":
           window.fullscreen()
       return mode != "default"
   view.connect("enter-fullscreen", on_enter)
   window.add(view)
   view.load_uri("file:///tmp/plain.html")
   window.show_all()
   GLib.timeout_add_seconds(30, Gtk.main_quit)
   Gtk.main()
   print(f"plain ({mode}): main loop ended normally", flush=True)
   ```

4. Run `python3 /tmp/plain.py` (or with `handled`, or `handled-window`) and click anywhere in the
   yellow page.

**Expected:** the page goes fullscreen (`default`), or the request is declined and the page gets
`fullscreenerror` (`handled`, `handled-window`); the process keeps running.

**Actual:** in all three modes, right after the click:

```
plain (default): WebKitGTK 2.50.4
title: plain ready
enter-fullscreen: default
→ the process aborts: exit status 134 (SIGABRT)

plain (handled): WebKitGTK 2.50.4
title: plain ready
enter-fullscreen: handled
→ exit status 134

plain (handled-window): WebKitGTK 2.50.4
title: plain ready
enter-fullscreen: handled-window
→ exit status 134
```

### Backtrace

From a Rust/Tauri application (wry 0.57, WebKitGTK 4.1) running under gdb on the same system, at
the same click. The frames inside WebKitGTK have no symbols: jammy's `libwebkit2gtk-4.1-0-dbgsym`
is still 2.36.0-2ubuntu1, so it can't be installed next to 2.50.4, and debuginfod downloads timed
out. The addresses are identical in every run (no ASLR under gdb), which may help with a matching
build:

```
Thread 1 "little-joan" received signal SIGABRT, Aborted.
#0  __pthread_kill_implementation (no_tid=0, signo=6, threadid=140737141551552) at ./nptl/pthread_kill.c:44
#1  __pthread_kill_internal (signo=6, threadid=140737141551552) at ./nptl/pthread_kill.c:78
#2  __GI___pthread_kill (threadid=140737141551552, signo=signo@entry=6) at ./nptl/pthread_kill.c:89
#3  0x00007fffef442476 in __GI_raise (sig=sig@entry=6) at ../sysdeps/posix/raise.c:26
#4  0x00007fffef4287f3 in __GI_abort () at ./stdlib/abort.c:79
#5  0x00007ffff2b095e3 in  () at /lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so.0
#6  0x00007ffff30e513d in  () at /lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so.0
#7  0x00007ffff304ac4a in  () at /lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so.0
#8  0x00007ffff31691dc in  () at /lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so.0
#9  0x00007ffff2c5c5be in  () at /lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so.0
#10 0x00007ffff30443bb in  () at /lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so.0
#11 0x00007ffff3045115 in  () at /lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so.0
#12 0x00007ffff3045c6d in  () at /lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so.0
#13 0x00007ffff11bc09a in  () at /lib/x86_64-linux-gnu/libjavascriptcoregtk-4.1.so.0
#14 0x00007ffff126f5fd in  () at /lib/x86_64-linux-gnu/libjavascriptcoregtk-4.1.so.0
#15 0x00007ffff1270423 in  () at /lib/x86_64-linux-gnu/libjavascriptcoregtk-4.1.so.0
#16 0x00007ffff7c07c94 in g_main_context_dispatch () at /lib/x86_64-linux-gnu/libglib-2.0.so.0
#17 0x00007ffff7c5d5f8 in  () at /lib/x86_64-linux-gnu/libglib-2.0.so.0
#18 0x00007ffff7c05433 in g_main_context_iteration () at /lib/x86_64-linux-gnu/libglib-2.0.so.0
#19 0x00007ffff1c48e15 in gtk_main_iteration_do () at /lib/x86_64-linux-gnu/libgtk-3.so.0
```

The abort is raised on the UI process's main thread, from a task that JavaScriptCore's RunLoop
dispatched through the GLib main context.

### What we ruled out

All on GitHub Actions `ubuntu-22.04` runners (image 20261012.x), Xvfb 1280×800×24, a D-Bus session
bus (`dbus-run-session`):

- **The embedding application:** the bare PyGObject window above aborts exactly like the Tauri
  application.
- **The renderer:** with `WEBKIT_DISABLE_DMABUF_RENDERER=1`, and with compositing mode disabled.
- **The window manager:** with openbox, and with no window manager at all.
- **How the request is handled:** WebKitGTK's own handling, an `enter-fullscreen` handler returning
  `TRUE`, and the same handler fullscreening the window itself (above).

Not affected: Ubuntu 24.04 (noble) with `libwebkit2gtk-4.1-0` 2.52.6-0ubuntu0.24.04.1 — the same
script reports `plain fullscreen=on` and keeps running.

### CI evidence

- https://github.com/memoliertorerr/LittleJoan/actions/runs/39247249388 (2026-10-19): the three
  modes above on two `ubuntu-22.04` runners (jobs "smoke (linux)" and "smoke (linux-scale-2)",
  step "Bare WebKitGTK fullscreen check") — every mode exits with status 134 after
  `enter-fullscreen` was emitted; the same jobs' application run shows the backtrace above.
- https://github.com/memoliertorerr/LittleJoan/actions/runs/38934582112 (2026-10-16): the
  application aborts under gdb (the backtrace above) and the bare window exits with status 134,
  on `ubuntu-22.04`.
- https://github.com/memoliertorerr/LittleJoan/actions/runs/38935403919 (2026-10-16): the same bare
  window on `ubuntu-24.04` with 2.52.6 — `plain fullscreen=on`, still running.

(GitHub keeps these logs for a limited time, 90 days by default.)

### The ask

jammy has stayed at 2.50.4-0ubuntu0.22.04.1 since 2026-01-13, while noble and resolute received
2.52.6 on 2026-08-31. Please update jammy's webkit2gtk to 2.52.x as well (or backport the fix), so
that WebKitGTK applications on 22.04 — every video's fullscreen button in them — stop aborting.
