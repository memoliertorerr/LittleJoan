import { describe, expect, it } from "vitest";
import {
  MAX_NOTES,
  MAX_TEXT,
  addNote,
  autosave,
  counts,
  deleteNote,
  editNote,
  fold,
  listKey,
  parseNotes,
  previewOf,
  restoreNote,
  search,
  serialize,
  sorted,
  titleOf,
  usage,
  when,
  type Note,
} from "./logic.js";

const note = (id: string, text: string, updated: number): Note => ({ id, text, created: updated, updated });

/** Timers that run only when the test says so. */
function fakeTimers() {
  let now = 0;
  const pending = new Map<number, { at: number; run: () => void }>();
  let next = 1;
  return {
    timers: {
      set: (run: () => void, ms: number) => {
        pending.set(next, { at: now + ms, run });
        return next++;
      },
      clear: (handle: unknown) => {
        pending.delete(handle as number);
      },
    },
    advance(ms: number) {
      now += ms;
      for (const [handle, timer] of [...pending]) {
        if (timer.at <= now) {
          pending.delete(handle);
          timer.run();
        }
      }
    },
    count: () => pending.size,
  };
}

describe("notes", () => {
  it("keeps several notes, newest first", () => {
    let notes: Note[] = [];
    const first = addNote(notes, "a", 1000);
    notes = first.notes;
    expect(first.note).toEqual({ id: "a", text: "", created: 1000, updated: 1000 });
    notes = addNote(notes, "b", 2000).notes;
    expect(notes.map((n) => n.id)).toEqual(["b", "a"]);
    notes = editNote(notes, "a", "Shopping\nmilk", 3000);
    expect(sorted(notes).map((n) => n.id)).toEqual(["a", "b"]);
    expect(notes.find((n) => n.id === "a")?.updated).toBe(3000);
    // The same text again changes nothing, not even the time.
    expect(editNote(notes, "a", "Shopping\nmilk", 4000)).toEqual(notes);
    expect(editNote(notes, "a", "x".repeat(MAX_TEXT + 10), 5000).find((n) => n.id === "a")?.text).toHaveLength(MAX_TEXT);
    const many = Array.from({ length: MAX_NOTES }, (_, i) => note(`n${i}`, "", i));
    expect(addNote(many, "new", 1e9).notes).toHaveLength(MAX_NOTES);
  });

  it("deletes a note and puts it back where it was", () => {
    const notes = [note("a", "A", 3), note("b", "B", 2), note("c", "C", 1)];
    const { notes: left, removed } = deleteNote(notes, "b");
    expect(left.map((n) => n.id)).toEqual(["a", "c"]);
    expect(removed).toEqual({ note: notes[1], index: 1 });
    expect(restoreNote(left, removed!).map((n) => n.id)).toEqual(["a", "b", "c"]);
    expect(restoreNote(notes, removed!)).toBe(notes);
    expect(deleteNote(notes, "zzz")).toEqual({ notes, removed: null });
  });

  it("titles a note by its first line and previews the rest", () => {
    expect(titleOf("")).toBe("New note");
    expect(titleOf("\n\n  Groceries  \nmilk\neggs")).toBe("Groceries");
    expect(titleOf("## Plan for Monday\n- call")).toBe("Plan for Monday");
    expect(titleOf("x".repeat(100))).toHaveLength(80);
    expect(previewOf("Groceries\nmilk\n\neggs")).toBe("milk · eggs");
    expect(previewOf("Only a title")).toBe("");
    expect(previewOf(`t\n${"y".repeat(200)}`)).toHaveLength(120);
  });

  it("searches every word, in any order, case and accent", () => {
    const notes = [note("a", "Café list\nmilk, beans", 1), note("b", "Meeting notes\nBudget café", 2), note("c", "Ideas", 3)];
    expect(search(notes, "").map((n) => n.id)).toEqual(["c", "b", "a"]);
    expect(search(notes, "cafe").map((n) => n.id)).toEqual(["b", "a"]);
    expect(search(notes, "BEANS cafe").map((n) => n.id)).toEqual(["a"]);
    expect(search(notes, "café budget").map((n) => n.id)).toEqual(["b"]);
    expect(search(notes, "nothing")).toEqual([]);
    expect(fold("Ångström")).toBe("angstrom");
  });

  it("autosaves once per burst of typing, with the latest text", async () => {
    const clock = fakeTimers();
    const saved: string[] = [];
    const save = autosave(async (value: string) => {
      saved.push(value);
      return Promise.resolve();
    }, clock.timers, 500);
    save.schedule("h");
    clock.advance(200);
    save.schedule("he");
    clock.advance(200);
    save.schedule("hey");
    expect(saved).toEqual([]);
    expect(save.pending()).toBe(true);
    clock.advance(499);
    expect(saved).toEqual([]);
    clock.advance(1);
    await Promise.resolve();
    expect(saved).toEqual(["hey"]);
    expect(save.pending()).toBe(false);
    expect(clock.count()).toBe(0);
    // flush saves what waits at once (closing the panel), and nothing when nothing waits.
    save.schedule("hey you");
    await save.flush();
    expect(saved).toEqual(["hey", "hey you"]);
    expect(clock.count()).toBe(0);
    await save.flush();
    expect(saved).toHaveLength(2);
  });

  it("keeps a failed save waiting for the next try", async () => {
    const clock = fakeTimers();
    let fail = true;
    const saved: string[] = [];
    const save = autosave(async (value: string) => {
      if (fail) {
        throw new Error("quota");
      }
      saved.push(value);
      return Promise.resolve();
    }, clock.timers, 500);
    save.schedule("draft");
    await expect(save.flush()).rejects.toThrow("quota");
    expect(save.pending()).toBe(true);
    fail = false;
    await save.flush();
    expect(saved).toEqual(["draft"]);
  });

  it("reads what storage holds from scratch", () => {
    expect(parseNotes(undefined)).toEqual([]);
    expect(parseNotes({ notes: "x" })).toEqual([]);
    const notes = parseNotes({
      notes: [
        { id: "a", text: "A", created: 1, updated: 2 },
        { id: "a", text: "duplicate", created: 1, updated: 2 },
        { id: "../b", text: "bad id", created: 1, updated: 2 },
        { id: "c", text: 3 },
        null,
        { id: "d", text: "no times" },
      ],
    });
    expect(notes).toEqual([
      { id: "a", text: "A", created: 1, updated: 2 },
      { id: "d", text: "no times", created: 0, updated: 0 },
    ]);
    expect(parseNotes(serialize(notes))).toEqual(notes);
    expect(usage([])).toBeLessThan(0.001);
    expect(usage([note("big", "x".repeat(524_288), 1)])).toBeGreaterThan(0.5);
  });

  it("counts words and characters", () => {
    expect(counts("")).toEqual({ words: 0, characters: 0 });
    expect(counts("  two words \n")).toEqual({ words: 2, characters: 13 });
    expect(counts("👋 hi")).toEqual({ words: 2, characters: 4 });
  });

  it("says when a note was edited", () => {
    const now = Date.UTC(2026, 9, 18, 15, 0);
    const options = { locale: "en-GB", timeZone: "UTC" };
    expect(when(now - 20_000, now, options)).toBe("Just now");
    expect(when(now - 5 * 60_000, now, options)).toBe("5 min ago");
    expect(when(Date.UTC(2026, 9, 18, 9, 5), now, options)).toBe("9:05");
    expect(when(Date.UTC(2026, 9, 18, 9, 5), now, { locale: "en-US", timeZone: "UTC" }).replace(/\s/g, " ")).toBe("9:05 AM");
    expect(when(Date.UTC(2026, 9, 17, 23, 0), now, options)).toBe("Yesterday");
    expect(when(Date.UTC(2026, 8, 2, 12, 0), now, options)).toBe("2 Sept");
    expect(when(Date.UTC(2025, 8, 2, 12, 0), now, options)).toBe("2 Sept 2025");
  });

  it("knows the list's keys", () => {
    expect(listKey({ key: "n", code: "KeyN" })).toBe("new");
    expect(listKey({ key: "/" })).toBe("search");
    expect(listKey({ key: "Enter" })).toBe("open");
    expect(listKey({ key: "ArrowDown" })).toBe("down");
    expect(listKey({ key: "Delete" })).toBe("delete");
    expect(listKey({ key: "Backspace" })).toBe("delete");
    expect(listKey({ key: "Escape" })).toBe("back");
    expect(listKey({ key: "z", code: "KeyZ", metaKey: true })).toBe("undo");
    expect(listKey({ key: "z", code: "KeyZ", ctrlKey: true })).toBe("undo");
    expect(listKey({ key: "Z", code: "KeyZ", ctrlKey: true, shiftKey: true })).toBeNull();
    expect(listKey({ key: "n", code: "KeyN", ctrlKey: true })).toBeNull();
    expect(listKey({ key: "x", code: "KeyX" })).toBeNull();
  });
});
