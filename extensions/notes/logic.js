// @ts-check
// Notes, pure: the notes list (add, edit, delete, search), how a note is titled and previewed,
// what storage holds (checked from scratch when read), and the autosave's debounce — with the
// clock and timers passed in, so logic.test.ts can drive them. ui.js is the panel.

/**
 * @typedef {object} Note
 * @property {string} id
 * @property {string} text
 * @property {number} created unix ms
 * @property {number} updated unix ms
 */

/** A note's text is cut here (storage holds 1 MB for all of them). */
export const MAX_TEXT = 200_000;
/** At most this many notes are kept. */
export const MAX_NOTES = 500;
/** Autosave waits this long after the last keystroke. */
export const SAVE_DELAY_MS = 500;
/** Little Joan's storage quota per extension, as JSON. */
export const QUOTA_BYTES = 1024 * 1024;

/**
 * Notes newest first (by their last edit).
 * @param {Note[]} notes
 * @returns {Note[]}
 */
export function sorted(notes) {
  return notes.slice().sort((a, b) => b.updated - a.updated || b.created - a.created);
}

/**
 * A new, empty note at the top.
 * @param {Note[]} notes
 * @param {string} id
 * @param {number} now
 * @returns {{ notes: Note[], note: Note }}
 */
export function addNote(notes, id, now) {
  const note = { id, text: "", created: now, updated: now };
  return { notes: [note, ...notes].slice(0, MAX_NOTES), note };
}

/**
 * The note `id` with new text (and a new edit time); unchanged text changes nothing.
 * @param {Note[]} notes
 * @param {string} id
 * @param {string} text
 * @param {number} now
 * @returns {Note[]}
 */
export function editNote(notes, id, text, now) {
  const cut = text.slice(0, MAX_TEXT);
  return notes.map((note) => (note.id === id && note.text !== cut ? { ...note, text: cut, updated: now } : note));
}

/**
 * The notes without `id`; `removed` is the note and where it was, for Undo.
 * @param {Note[]} notes
 * @param {string} id
 * @returns {{ notes: Note[], removed: { note: Note, index: number } | null }}
 */
export function deleteNote(notes, id) {
  const index = notes.findIndex((note) => note.id === id);
  const note = notes[index];
  if (!note) {
    return { notes, removed: null };
  }
  return { notes: notes.filter((other) => other.id !== id), removed: { note, index } };
}

/**
 * Undo a delete: the note back where it was.
 * @param {Note[]} notes
 * @param {{ note: Note, index: number }} removed
 * @returns {Note[]}
 */
export function restoreNote(notes, removed) {
  if (notes.some((note) => note.id === removed.note.id)) {
    return notes;
  }
  const copy = notes.slice();
  copy.splice(Math.min(removed.index, copy.length), 0, removed.note);
  return copy;
}

/** @param {string} text */
const lines = (text) =>
  text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);

/**
 * A note's title: its first line that isn't empty (Markdown heading marks dropped), cut to 80
 * characters; "New note" while it is empty.
 * @param {string} text
 */
export function titleOf(text) {
  const first = (lines(text)[0] ?? "").replace(/^#{1,6}\s+/, "");
  if (!first) {
    return "New note";
  }
  return first.length > 80 ? `${first.slice(0, 79)}…` : first;
}

/**
 * What the list shows under the title: the next lines run together, cut to 120 characters.
 * @param {string} text
 */
export function previewOf(text) {
  const rest = lines(text).slice(1).join(" · ");
  return rest.length > 120 ? `${rest.slice(0, 119)}…` : rest;
}

/**
 * Text for matching: lower case, without accents (so "cafe" finds "Café").
 * @param {string} text
 */
export function fold(text) {
  return text.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
}

/**
 * The notes (newest first) whose text holds every word of `query`, in any order and case.
 * @param {Note[]} notes
 * @param {string} query
 * @returns {Note[]}
 */
export function search(notes, query) {
  const words = fold(query).split(/\s+/).filter(Boolean);
  const all = sorted(notes);
  if (words.length === 0) {
    return all;
  }
  return all.filter((note) => {
    const text = fold(note.text);
    return words.every((word) => text.includes(word));
  });
}

/**
 * Counts for the editor's footer.
 * @param {string} text
 */
export function counts(text) {
  const words = text.trim() ? text.trim().split(/\s+/).length : 0;
  return { words, characters: [...text].length };
}

/**
 * What storage holds, checked from scratch: notes with a string id, text and times; anything
 * else is dropped (one bad note doesn't lose the others).
 * @param {unknown} value
 * @returns {Note[]}
 */
export function parseNotes(value) {
  const list = typeof value === "object" && value !== null && Array.isArray(/** @type {{ notes?: unknown }} */ (value).notes) ? /** @type {unknown[]} */ (/** @type {{ notes: unknown[] }} */ (value).notes) : [];
  /** @type {Note[]} */
  const notes = [];
  const seen = new Set();
  for (const item of list) {
    if (typeof item !== "object" || item === null) {
      continue;
    }
    const { id, text, created, updated } = /** @type {Record<string, unknown>} */ (item);
    if (typeof id !== "string" || !/^[\w-]{1,64}$/.test(id) || seen.has(id) || typeof text !== "string") {
      continue;
    }
    const made = typeof created === "number" && Number.isFinite(created) ? created : 0;
    const edited = typeof updated === "number" && Number.isFinite(updated) ? updated : made;
    seen.add(id);
    notes.push({ id, text: text.slice(0, MAX_TEXT), created: made, updated: edited });
    if (notes.length >= MAX_NOTES) {
      break;
    }
  }
  return notes;
}

/**
 * What storage keeps.
 * @param {Note[]} notes
 */
export function serialize(notes) {
  return { version: 1, notes };
}

/**
 * How much of the quota the notes take (the stored JSON's length in UTF-8 bytes), 0–1.
 * @param {Note[]} notes
 */
export function usage(notes) {
  return new TextEncoder().encode(JSON.stringify({ notes: serialize(notes) })).length / QUOTA_BYTES;
}

/**
 * "Just now", "5 min ago", "14:05" (today), "Yesterday", or a date — for the list.
 * @param {number} then unix ms
 * @param {number} now unix ms
 * @param {{ locale?: string, timeZone?: string }} [options]
 */
export function when(then, now, options = {}) {
  const ago = now - then;
  if (ago < 60_000) {
    return "Just now";
  }
  if (ago < 60 * 60_000) {
    return `${Math.floor(ago / 60_000)} min ago`;
  }
  const day = (/** @type {number} */ t) =>
    new Intl.DateTimeFormat("en-CA", { timeZone: options.timeZone, year: "numeric", month: "2-digit", day: "2-digit" }).format(t);
  if (day(then) === day(now)) {
    return new Intl.DateTimeFormat(options.locale, { timeZone: options.timeZone, hour: "numeric", minute: "2-digit" }).format(then);
  }
  if (day(then) === day(now - 86_400_000)) {
    return "Yesterday";
  }
  const sameYear = day(then).slice(0, 4) === day(now).slice(0, 4);
  return new Intl.DateTimeFormat(options.locale, { timeZone: options.timeZone, month: "short", day: "numeric", year: sameYear ? undefined : "numeric" }).format(then);
}

/**
 * @typedef {object} Timers
 * @property {(run: () => void, ms: number) => unknown} set
 * @property {(handle: unknown) => void} clear
 */

/**
 * Autosave with a debounce: `schedule(value)` saves `value` once nothing new was scheduled for
 * `delay` ms; `flush()` saves what is waiting now. `save` is called once per burst, with the
 * latest value; a save that fails keeps the value waiting for the next try.
 * @template T
 * @param {(value: T) => Promise<void>} save
 * @param {Timers} timers
 * @param {number} [delay]
 */
export function autosave(save, timers, delay = SAVE_DELAY_MS) {
  /** @type {{ value: T } | null} */
  let waiting = null;
  /** @type {unknown} */
  let handle = null;
  const flush = async () => {
    if (handle !== null) {
      timers.clear(handle);
      handle = null;
    }
    const next = waiting;
    if (!next) {
      return;
    }
    waiting = null;
    try {
      await save(next.value);
    } catch (error) {
      waiting ??= next;
      throw error;
    }
  };
  return {
    /** @param {T} value */
    schedule(value) {
      waiting = { value };
      if (handle !== null) {
        timers.clear(handle);
      }
      handle = timers.set(() => {
        handle = null;
        flush().catch(() => {
          // Kept waiting; the next keystroke or flush tries again.
        });
      }, delay);
    },
    flush,
    pending: () => waiting !== null,
  };
}

/** @typedef {"new" | "search" | "open" | "up" | "down" | "delete" | "back" | "undo"} ListAction */

/**
 * What a key press means on the list (not while typing in a field): N a new note, / search,
 * Enter open, ↑ ↓ move, Delete remove, Esc leave, Ctrl/Cmd+Z undo a delete.
 * @param {{ key: string, code?: string, ctrlKey?: boolean, altKey?: boolean, metaKey?: boolean, shiftKey?: boolean }} event
 * @returns {ListAction | null}
 */
export function listKey(event) {
  if ((event.ctrlKey || event.metaKey) && !event.altKey && event.code === "KeyZ" && !event.shiftKey) {
    return "undo";
  }
  if (event.ctrlKey || event.altKey || event.metaKey) {
    return null;
  }
  /** @type {Record<string, ListAction>} */
  const keys = { "/": "search", Enter: "open", ArrowUp: "up", ArrowDown: "down", Delete: "delete", Backspace: "delete", Escape: "back" };
  return keys[event.key] ?? (event.code === "KeyN" ? "new" : null);
}
