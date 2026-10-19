// Linux process helpers for the smoke test (LJ13): which process is the app, read from /proc.
//
// The app isn't always the process smoke.mts spawned: under gdb it is gdb's child, and the AppImage's
// runtime extracts the image (APPIMAGE_EXTRACT_AND_RUN, CI has no FUSE) and starts AppRun as its
// child — a shell script that ends in `exec …/usr/bin/little-joan`, so the app keeps AppRun's pid
// while its name follows the file it ran (AppRun.wrapped, a link), not "little-joan". So the app is
// the newest process under the spawned one whose executable is a file called little-joan. gdb can't
// follow the AppImage's runtime into the app (it would follow AppRun's subshells instead), so
// smoke.mts attaches it to that process once it runs. The pure parts are unit-tested
// (smoke-linux.test.ts).
import { readdirSync, readFileSync, readlinkSync } from "node:fs";

/** One process: its id, its parent's, and the file it runs (`/proc/<pid>/exe`). */
export interface Proc {
  pid: number;
  ppid: number;
  exe: string;
}

/**
 * The parent's pid from a `/proc/<pid>/stat` line: the field after the state, which follows the
 * command name in parentheses — a name that may hold spaces and parentheses itself, so the fields
 * start after the LAST `)`.
 */
export function parseStatPpid(stat: string): number | undefined {
  const end = stat.lastIndexOf(")");
  if (end < 0) {
    return undefined;
  }
  const ppid = Number(stat.slice(end + 1).trim().split(/\s+/)[1]);
  return Number.isInteger(ppid) && ppid >= 0 ? ppid : undefined;
}

/** The file name a process runs, without the kernel's " (deleted)" for a file removed meanwhile. */
export function exeName(exe: string): string {
  return (exe.split("/").at(-1) ?? "").replace(/ \(deleted\)$/, "");
}

/**
 * The app among `procs`: the newest process (the highest pid) in `root`'s tree — `root` itself
 * included — whose executable is called `name`. Second instances (the smoke's remote control) are
 * started apart, so they are never in the tree.
 */
export function findApp(procs: readonly Proc[], root: number, name = "little-joan"): number | undefined {
  const children = new Map<number, number[]>();
  for (const proc of procs) {
    children.set(proc.ppid, [...(children.get(proc.ppid) ?? []), proc.pid]);
  }
  const tree = new Set<number>();
  const queue = [root];
  while (queue.length > 0) {
    const pid = queue.pop() as number;
    if (!tree.has(pid)) {
      tree.add(pid);
      queue.push(...(children.get(pid) ?? []));
    }
  }
  const matches = procs.filter((proc) => tree.has(proc.pid) && exeName(proc.exe) === name).map((proc) => proc.pid);
  return matches.length > 0 ? Math.max(...matches) : undefined;
}

/** Every process this user may look at (others' `exe` links can't be read, and are left out). */
export function listProcs(): Proc[] {
  const procs: Proc[] = [];
  for (const entry of readdirSync("/proc")) {
    if (!/^\d+$/.test(entry)) {
      continue;
    }
    try {
      const ppid = parseStatPpid(readFileSync(`/proc/${entry}/stat`, "utf8"));
      if (ppid !== undefined) {
        procs.push({ pid: Number(entry), ppid, exe: readlinkSync(`/proc/${entry}/exe`) });
      }
    } catch {
      // Gone meanwhile, or not ours.
    }
  }
  return procs;
}
