import { describe, expect, it } from "vitest";
import { exeName, findApp, parseStatPpid, type Proc } from "./smoke-linux.mts";

describe("/proc/<pid>/stat (LJ13)", () => {
  it("reads the parent's pid after the command name", () => {
    expect(parseStatPpid("4242 (little-joan) S 4100 4242 4100 0 -1 4194560 1203 0 0 0")).toBe(4100);
  });

  it("copes with spaces and parentheses in the command name", () => {
    expect(parseStatPpid("77 (Little Joan_0.2) S 12 77 12 0 -1")).toBe(12);
    expect(parseStatPpid("78 (a) b (c)) R 9 78 9 0")).toBe(9);
  });

  it("gives nothing for a line it can't read", () => {
    expect(parseStatPpid("")).toBeUndefined();
    expect(parseStatPpid("12 no-parentheses S 1")).toBeUndefined();
    expect(parseStatPpid("12 (x) S")).toBeUndefined();
  });
});

describe("which process is the app (LJ13)", () => {
  // The AppImage: smoke.mts spawned its runtime (100); the runtime's child ran AppRun (bash), which
  // exec'd the app through its AppRun.wrapped link: /proc shows the real file.
  const appImage: Proc[] = [
    { pid: 1, ppid: 0, exe: "/usr/lib/systemd/systemd" },
    { pid: 100, ppid: 50, exe: "/tmp/runner/appimage/LittleJoan-0.2.0-linux-x86_64.AppImage" },
    { pid: 101, ppid: 100, exe: "/tmp/appimage_extracted_0123abcd/usr/bin/little-joan" },
    { pid: 140, ppid: 101, exe: "/tmp/appimage_extracted_0123abcd/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitWebProcess" },
    // A second instance (the smoke's remote control), started apart from the first.
    { pid: 150, ppid: 50, exe: "/tmp/runner/appimage/LittleJoan-0.2.0-linux-x86_64.AppImage" },
    { pid: 151, ppid: 150, exe: "/tmp/appimage_extracted_0123abcd/usr/bin/little-joan" },
  ];

  it("finds the AppImage's app under its runtime, not a second instance", () => {
    expect(findApp(appImage, 100)).toBe(101);
    expect(findApp(appImage, 150)).toBe(151);
  });

  it("finds gdb's child, and a plain launch itself", () => {
    const gdb: Proc[] = [
      { pid: 200, ppid: 50, exe: "/usr/bin/gdb" },
      { pid: 201, ppid: 200, exe: "/usr/bin/little-joan" },
      { pid: 210, ppid: 201, exe: "/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitNetworkProcess" },
    ];
    expect(findApp(gdb, 200)).toBe(201);
    expect(findApp(gdb, 201)).toBe(201);
  });

  it("gives nothing while the app hasn't started yet (AppRun is still bash)", () => {
    const early: Proc[] = [
      { pid: 100, ppid: 50, exe: "/tmp/runner/LittleJoan.AppImage" },
      { pid: 101, ppid: 100, exe: "/usr/bin/bash" },
    ];
    expect(findApp(early, 100)).toBeUndefined();
    expect(findApp([], 100)).toBeUndefined();
  });

  it("matches the file's name exactly, a deleted file too", () => {
    expect(exeName("/usr/bin/little-joan")).toBe("little-joan");
    expect(exeName("/tmp/x/usr/bin/little-joan (deleted)")).toBe("little-joan");
    const lookalikes: Proc[] = [
      { pid: 300, ppid: 50, exe: "/usr/bin/little-joan-helper" },
      { pid: 301, ppid: 300, exe: "/opt/not-little-joan" },
    ];
    expect(findApp(lookalikes, 300)).toBeUndefined();
  });
});
