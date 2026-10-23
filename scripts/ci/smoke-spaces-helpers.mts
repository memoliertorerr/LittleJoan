// The over-full-screen run's helpers (LJ19.2, smoke-spaces.mts): another app's full-screen window
// per OS, written next to the smoke's output and started in the background. Each appends one JSON
// line to a status file every 500 ms — whether its window is full screen and active, and which
// process has the foreground — so the run can tell what happened from outside the app. Nothing to
// install: Swift on the macOS runners, PowerShell with WinForms on Windows, Python 3 with ctypes and
// libX11 (xdotool's) on Linux.

/**
 * macOS: `swift fs-helper.swift fullscreen <status> <seconds>` — a regular app whose window enters
 * native full screen (its own Space), as Safari or a video player would. A `fullscreen` line once
 * didEnterFullScreen came; its status: `fullscreen` (the style mask), `onActiveSpace` (its Space is still the one
 * on screen), `active` / `key`, `frontPid` (NSWorkspace's frontmost app) and `top` (the topmost
 * window at its centre is its own). It reads commands from `<status>.cmd`: `activate` (back to the
 * front, as a click would) and `stop` (leave full screen, then quit).
 *
 * `swift fs-helper.swift probe <status> <bits>` — informational: an accessory app (no Dock icon,
 * like Little Joan) orders a window with the collection behaviour `bits` front and activates itself
 * the way tao's set_focus does, then reports where the window went and quits: what this runner
 * does without Little Joan's setting (0), with it, and with more.
 */
export const MAC_HELPER = `import AppKit

let args = CommandLine.arguments
let mode = args.count > 1 ? args[1] : "fullscreen"
let statusPath = args.count > 2 ? args[2] : NSTemporaryDirectory() + "lj-fs-helper.jsonl"
let commandPath = statusPath + ".cmd"
let app = NSApplication.shared

func report(_ fields: [String: Any]) {
  var line = fields
  line["t"] = Int64(Date().timeIntervalSince1970 * 1000)
  line["pid"] = Int(ProcessInfo.processInfo.processIdentifier)
  guard var bytes = try? JSONSerialization.data(withJSONObject: line) else { return }
  bytes.append(0x0a)
  if let handle = FileHandle(forWritingAtPath: statusPath) {
    handle.seekToEndOfFile()
    handle.write(bytes)
    handle.closeFile()
  } else {
    FileManager.default.createFile(atPath: statusPath, contents: bytes)
  }
}

func onTop(_ window: NSWindow) -> Bool {
  let frame = window.frame
  let hit = NSWindow.windowNumber(at: NSPoint(x: frame.midX, y: frame.midY), belowWindowWithWindowNumber: 0)
  return hit == window.windowNumber
}

if mode == "probe" {
  let bits = UInt(args.count > 3 ? args[3] : "0") ?? 0
  app.setActivationPolicy(.accessory)
  let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 480, height: 360), styleMask: [.titled, .closable, .resizable], backing: .buffered, defer: false)
  window.title = "LJ probe \\(bits)"
  window.collectionBehavior = NSWindow.CollectionBehavior(rawValue: bits)
  window.backgroundColor = NSColor.orange
  window.center()
  DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
    window.makeKeyAndOrderFront(nil)
    app.activate(ignoringOtherApps: true)
  }
  DispatchQueue.main.asyncAfter(deadline: .now() + 2.5) {
    report(["state": "probe", "bits": Int(bits), "behavior": Int(window.collectionBehavior.rawValue), "level": window.level.rawValue, "onActiveSpace": window.isOnActiveSpace, "key": window.isKeyWindow, "active": app.isActive, "top": onTop(window)])
    app.terminate(nil)
  }
} else {
  let seconds = Double(args.count > 3 ? args[3] : "300") ?? 300
  app.setActivationPolicy(.regular)
  let screen = NSScreen.main?.frame ?? NSRect(x: 0, y: 0, width: 1024, height: 768)
  let window = NSWindow(contentRect: NSRect(x: screen.midX - 320, y: screen.midY - 210, width: 640, height: 420), styleMask: [.titled, .closable, .miniaturizable, .resizable], backing: .buffered, defer: false)
  window.title = "LJ fullscreen helper"
  window.collectionBehavior = [.fullScreenPrimary]
  window.backgroundColor = NSColor(calibratedRed: 0.13, green: 0.55, blue: 0.13, alpha: 1)
  window.isReleasedWhenClosed = false
  _ = NotificationCenter.default.addObserver(forName: NSWindow.didEnterFullScreenNotification, object: window, queue: .main) { _ in
    report(["state": "fullscreen"])
  }
  window.makeKeyAndOrderFront(nil)
  app.activate(ignoringOtherApps: true)
  DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) { window.toggleFullScreen(nil) }
  _ = Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { _ in
    if let command = try? String(contentsOfFile: commandPath, encoding: .utf8) {
      try? FileManager.default.removeItem(atPath: commandPath)
      let name = command.trimmingCharacters(in: .whitespacesAndNewlines)
      if name == "activate" {
        window.makeKeyAndOrderFront(nil)
        app.activate(ignoringOtherApps: true)
      } else if name == "stop" {
        if window.styleMask.contains(.fullScreen) { window.toggleFullScreen(nil) }
        DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) { app.terminate(nil) }
      }
      report(["state": "command", "command": name])
    }
    let front = NSWorkspace.shared.frontmostApplication?.processIdentifier ?? -1
    report(["state": "status", "fullscreen": window.styleMask.contains(.fullScreen), "onActiveSpace": window.isOnActiveSpace, "active": app.isActive, "key": window.isKeyWindow, "frontPid": Int(front), "top": onTop(window)])
  }
  DispatchQueue.main.asyncAfter(deadline: .now() + seconds) { app.terminate(nil) }
}
app.run()
`;

/**
 * Windows: `powershell -File fs-helper.ps1 -Status <status> -Seconds <n>` — a borderless WinForms
 * window covering the primary screen, not topmost (a browser in F11, a borderless-full-screen
 * game). Its status: `active` (it is the foreground window), `frontPid` (the foreground window's
 * process), `fullscreen` (visible, with the screen's bounds).
 */
export const WINDOWS_HELPER = `param([string]$Status, [int]$Seconds = 300)
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type -Namespace LJ -Name Front -MemberDefinition @'
[DllImport("user32.dll")] public static extern System.IntPtr GetForegroundWindow();
[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(System.IntPtr hWnd, out uint processId);
'@
$form = New-Object System.Windows.Forms.Form
$form.Text = 'LJ fullscreen helper'
$form.FormBorderStyle = [System.Windows.Forms.FormBorderStyle]::None
$form.StartPosition = [System.Windows.Forms.FormStartPosition]::Manual
$form.Bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$form.TopMost = $false
$form.BackColor = [System.Drawing.Color]::FromArgb(34, 139, 34)
$started = Get-Date
function Report([hashtable]$fields) {
  $fields.t = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
  $fields.pid = $PID
  Add-Content -LiteralPath $Status -Value ($fields | ConvertTo-Json -Compress)
}
$timer = New-Object System.Windows.Forms.Timer
$timer.Interval = 500
$timer.Add_Tick({
  $fg = [LJ.Front]::GetForegroundWindow()
  [uint32]$fgPid = 0
  [void][LJ.Front]::GetWindowThreadProcessId($fg, [ref]$fgPid)
  $screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  Report @{ state = 'status'; active = ($fg -eq $form.Handle); frontPid = [int64]$fgPid; fullscreen = ($form.Visible -and $form.Bounds.Equals($screen)); topMost = $form.TopMost }
  if (((Get-Date) - $started).TotalSeconds -gt $Seconds) { $timer.Stop(); $form.Close() }
})
$form.Add_Shown({
  $form.Activate()
  Report @{ state = 'shown'; hwnd = $form.Handle.ToInt64() }
  $timer.Start()
})
[System.Windows.Forms.Application]::Run($form)
`;

/**
 * Linux: `python3 fs-helper.py <status> <seconds>` — an X11 window the size of the screen with
 * `_NET_WM_STATE_FULLSCREEN` set before it maps, which the window manager (openbox) honours. Its
 * status: `fullscreen` (the state on its window), `active` (the root's `_NET_ACTIVE_WINDOW` is its
 * window) and `activeWindow`.
 */
export const LINUX_HELPER = `import ctypes, json, os, sys, time
from ctypes import POINTER, byref, c_char_p, c_int, c_long, c_uint, c_ulong, c_void_p

status_path = sys.argv[1]
seconds = float(sys.argv[2]) if len(sys.argv) > 2 else 300.0

def report(fields):
    fields["t"] = int(time.time() * 1000)
    fields["pid"] = os.getpid()
    with open(status_path, "a") as status:
        status.write(json.dumps(fields) + "\\n")

x = ctypes.CDLL("libX11.so.6")
x.XOpenDisplay.restype = c_void_p
x.XOpenDisplay.argtypes = [c_char_p]
x.XDefaultScreen.argtypes = [c_void_p]
x.XRootWindow.restype = c_ulong
x.XRootWindow.argtypes = [c_void_p, c_int]
x.XDisplayWidth.argtypes = [c_void_p, c_int]
x.XDisplayHeight.argtypes = [c_void_p, c_int]
x.XCreateSimpleWindow.restype = c_ulong
x.XCreateSimpleWindow.argtypes = [c_void_p, c_ulong, c_int, c_int, c_uint, c_uint, c_uint, c_ulong, c_ulong]
x.XInternAtom.restype = c_ulong
x.XInternAtom.argtypes = [c_void_p, c_char_p, c_int]
x.XChangeProperty.argtypes = [c_void_p, c_ulong, c_ulong, c_ulong, c_int, c_int, c_void_p, c_int]
x.XStoreName.argtypes = [c_void_p, c_ulong, c_char_p]
x.XSelectInput.argtypes = [c_void_p, c_ulong, c_long]
x.XMapRaised.argtypes = [c_void_p, c_ulong]
x.XFlush.argtypes = [c_void_p]
x.XPending.argtypes = [c_void_p]
x.XNextEvent.argtypes = [c_void_p, c_void_p]
x.XGetWindowProperty.argtypes = [c_void_p, c_ulong, c_ulong, c_long, c_long, c_int, c_ulong, POINTER(c_ulong), POINTER(c_int), POINTER(c_ulong), POINTER(c_ulong), POINTER(c_void_p)]
x.XFree.argtypes = [c_void_p]

display = x.XOpenDisplay(None)
if not display:
    print("fs-helper: no X display", flush=True)
    sys.exit(1)
screen = x.XDefaultScreen(display)
root = x.XRootWindow(display, screen)
width, height = x.XDisplayWidth(display, screen), x.XDisplayHeight(display, screen)
window = x.XCreateSimpleWindow(display, root, 0, 0, width, height, 0, 0, 0x228B22)
x.XStoreName(display, window, b"LJ fullscreen helper")
XA_ATOM, XA_CARDINAL, XA_WINDOW = 4, 6, 33
NET_WM_STATE = x.XInternAtom(display, b"_NET_WM_STATE", 0)
FULLSCREEN = x.XInternAtom(display, b"_NET_WM_STATE_FULLSCREEN", 0)
ACTIVE = x.XInternAtom(display, b"_NET_ACTIVE_WINDOW", 0)
state = (c_ulong * 1)(FULLSCREEN)
x.XChangeProperty(display, window, NET_WM_STATE, XA_ATOM, 32, 0, ctypes.cast(state, c_void_p), 1)
own_pid = (c_ulong * 1)(os.getpid())
x.XChangeProperty(display, window, x.XInternAtom(display, b"_NET_WM_PID", 0), XA_CARDINAL, 32, 0, ctypes.cast(own_pid, c_void_p), 1)
x.XSelectInput(display, window, (1 << 15) | (1 << 17))
x.XMapRaised(display, window)
x.XFlush(display)
report({"state": "mapped", "xid": window, "width": width, "height": height})

def values(of, prop, kind):
    actual, fmt, count, after, data = c_ulong(), c_int(), c_ulong(), c_ulong(), c_void_p()
    failed = x.XGetWindowProperty(display, of, prop, 0, 64, 0, kind, byref(actual), byref(fmt), byref(count), byref(after), byref(data))
    if failed != 0 or not data.value:
        return []
    items = ctypes.cast(data, POINTER(c_ulong))
    found = [items[i] for i in range(count.value)]
    x.XFree(data)
    return found

event = ctypes.create_string_buffer(192)
deadline = time.time() + seconds
next_report = 0.0
while time.time() < deadline:
    while x.XPending(display):
        x.XNextEvent(display, event)
    now = time.time()
    if now >= next_report:
        active = values(root, ACTIVE, XA_WINDOW)
        report({"state": "status", "fullscreen": FULLSCREEN in values(window, NET_WM_STATE, XA_ATOM), "active": bool(active) and active[0] == window, "activeWindow": active[0] if active else 0})
        next_report = now + 0.5
    time.sleep(0.05)
`;
