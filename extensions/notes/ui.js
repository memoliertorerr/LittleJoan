// @ts-check
// Notes' panel: a list with search and an editor (side by side in a wide window, one at a time
// in a narrow one). Notes live in littleJoan.storage, saved a moment after you stop typing; Copy
// uses littleJoan.clipboard. The rules are logic.js.
import { addNote, autosave, counts, deleteNote, editNote, listKey, parseNotes, previewOf, restoreNote, search, serialize, titleOf, usage, when } from "./logic.js";

/** @typedef {import("./logic.js").Note} Note */

/** @param {string} id */
function element(id) {
  const found = document.getElementById(id);
  if (!found) {
    throw new Error(`#${id} is missing`);
  }
  return found;
}

const app = element("app");
const searchField = /** @type {HTMLInputElement} */ (element("search"));
const list = element("list");
const empty = element("empty");
const editor = /** @type {HTMLTextAreaElement} */ (element("editor"));
const status = element("status");
const countsText = element("counts");
const usageText = element("usage");
const toast = element("toast");
const toastText = element("toast-text");
const wide = matchMedia("(min-width: 640px)");

/** @type {Note[]} */
let notes = [];
/** @type {string | null} the note the list has selected */
let selected = null;
/** @type {string | null} the note in the editor */
let editing = null;
/** @type {{ note: Note, index: number } | null} */
let lastDeleted = null;
/** @type {ReturnType<typeof setTimeout> | undefined} */
let toastTimer;
/** @type {ReturnType<typeof setTimeout> | undefined} */
let statusTimer;

const saver = autosave(
  async (/** @type {Note[]} */ value) => {
    try {
      await littleJoan.storage.set("notes", serialize(value));
      if (!saver.pending()) {
        say("Saved");
      }
    } catch (error) {
      saveFailed(error);
      throw error;
    }
  },
  { set: (run, ms) => setTimeout(run, ms), clear: (handle) => clearTimeout(/** @type {number} */ (handle)) },
);

/** @param {string} text */
function say(text) {
  clearTimeout(statusTimer);
  status.textContent = text;
}

/** @param {unknown} error */
function saveFailed(error) {
  const code = error instanceof Error && "code" in error ? String(error.code) : "";
  say(code === "quota" ? "Couldn't save: Notes is full (1 MB). Delete or shorten a note." : "Couldn't save. Your text is kept while the panel is open.");
}

/** Saves what changed, now (leaving the editor, closing). */
async function saveNow() {
  try {
    await saver.flush();
  } catch {
    // The saver said why (saveFailed); the text stays waiting for the next try.
  }
}

/** @param {Note[]} next */
function commit(next) {
  notes = next;
  say("Saving…");
  saver.schedule(notes);
  showUsage();
}

function showUsage() {
  const used = usage(notes);
  usageText.textContent = used > 0.75 ? `${Math.round(used * 100)}% of the 1 MB space used` : "";
}

/** The notes the list shows now. */
function visible() {
  return search(notes, searchField.value);
}

function renderList() {
  const shown = visible();
  if (!shown.some((note) => note.id === selected)) {
    selected = shown[0]?.id ?? null;
  }
  const now = Date.now();
  list.replaceChildren(
    ...shown.map((note) => {
      const item = document.createElement("li");
      item.id = `note-${note.id}`;
      item.setAttribute("role", "option");
      item.setAttribute("aria-selected", String(note.id === selected));
      item.classList.toggle("open", note.id === editing);
      const title = document.createElement("strong");
      title.textContent = titleOf(note.text);
      const time = document.createElement("time");
      time.textContent = when(note.updated, now);
      const preview = document.createElement("span");
      preview.className = "preview";
      preview.textContent = previewOf(note.text) || "No more text";
      item.append(title, time, preview);
      item.addEventListener("click", () => open(note.id));
      return item;
    }),
  );
  if (selected) {
    list.setAttribute("aria-activedescendant", `note-${selected}`);
    document.getElementById(`note-${selected}`)?.scrollIntoView({ block: "nearest" });
  } else {
    list.removeAttribute("aria-activedescendant");
  }
  empty.hidden = shown.length > 0;
  empty.textContent = notes.length === 0 ? "No notes yet. Press N or New note to write one." : `No note has “${searchField.value.trim()}”. Enter makes one.`;
}

/** @param {"list" | "editor"} view */
function show(view) {
  app.dataset.view = view;
  // In a wide window the editor stays in sight with no note open: it waits for one.
  editor.disabled = editing === null;
  editor.placeholder = editing === null ? "Choose a note, or press N for a new one." : "Write something… The first line is the title.";
  if (editing === null) {
    editor.value = "";
    countsText.textContent = "";
    say("");
  }
  for (const id of ["copy", "delete"]) {
    /** @type {HTMLButtonElement} */ (element(id)).disabled = editing === null;
  }
}

/** @param {string} id */
function open(id) {
  const note = notes.find((n) => n.id === id);
  if (!note) {
    return;
  }
  editing = id;
  selected = id;
  editor.value = note.text;
  showCounts();
  say(note.text ? `Edited ${when(note.updated, Date.now())}` : "");
  show("editor");
  renderList();
  editor.focus();
  editor.setSelectionRange(editor.value.length, editor.value.length);
}

function showCounts() {
  const { words, characters } = counts(editor.value);
  countsText.textContent = `${words} ${words === 1 ? "word" : "words"} · ${characters} ${characters === 1 ? "character" : "characters"}`;
}

/** @param {string} [text] */
function create(text = "") {
  const id = Array.from(crypto.getRandomValues(new Uint8Array(8)), (b) => b.toString(16).padStart(2, "0")).join("");
  const added = addNote(notes, id, Date.now());
  commit(text ? editNote(added.notes, id, text, Date.now()) : added.notes);
  searchField.value = "";
  open(id);
}

/** Back from the editor to the list; an empty note isn't kept. */
function leaveEditor() {
  const note = notes.find((n) => n.id === editing);
  if (note && !note.text.trim()) {
    notes = deleteNote(notes, note.id).notes;
    commit(notes);
  }
  editing = null;
  void saveNow();
  show("list");
  renderList();
  list.focus();
}

/** @param {string} id */
function remove(id) {
  const result = deleteNote(notes, id);
  if (!result.removed) {
    return;
  }
  lastDeleted = result.removed;
  commit(result.notes);
  if (editing === id) {
    editing = null;
    show("list");
  }
  renderList();
  list.focus();
  toastText.textContent = `Deleted “${titleOf(result.removed.note.text)}”`;
  toast.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toast.hidden = true;
    lastDeleted = null;
  }, 6000);
}

function undo() {
  if (!lastDeleted) {
    return;
  }
  commit(restoreNote(notes, lastDeleted));
  selected = lastDeleted.note.id;
  lastDeleted = null;
  toast.hidden = true;
  renderList();
}

/** @param {number} step */
function moveSelection(step) {
  const shown = visible();
  const at = shown.findIndex((note) => note.id === selected);
  const next = shown[Math.min(shown.length - 1, Math.max(0, at + step))];
  if (next) {
    selected = next.id;
    renderList();
  }
}

editor.addEventListener("input", () => {
  if (editing) {
    commit(editNote(notes, editing, editor.value, Date.now()));
    showCounts();
    if (wide.matches) {
      renderList();
    }
  }
});
editor.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    event.preventDefault();
    leaveEditor();
  }
});
editor.addEventListener("blur", () => {
  void saveNow();
});

searchField.addEventListener("input", () => {
  selected = null;
  renderList();
});
searchField.addEventListener("keydown", (event) => {
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    moveSelection(event.key === "ArrowDown" ? 1 : -1);
  } else if (event.key === "Enter") {
    event.preventDefault();
    if (selected) {
      open(selected);
    } else if (searchField.value.trim()) {
      create(searchField.value.trim());
    }
  } else if (event.key === "Escape") {
    event.preventDefault();
    if (searchField.value) {
      searchField.value = "";
      renderList();
    }
    list.focus();
  }
});

list.addEventListener("keydown", (event) => {
  const action = listKey(event);
  if (!action) {
    return;
  }
  event.preventDefault();
  switch (action) {
    case "new":
      create();
      break;
    case "search":
      searchField.focus();
      searchField.select();
      break;
    case "open":
      if (selected) {
        open(selected);
      }
      break;
    case "up":
    case "down":
      moveSelection(action === "down" ? 1 : -1);
      break;
    case "delete":
      if (selected) {
        remove(selected);
      }
      break;
    case "undo":
      undo();
      break;
    case "back":
      void saveNow().then(() => littleJoan.close());
      break;
  }
});

// N (new) and Ctrl/Cmd+Z (undo a delete) from anywhere but a text field.
document.addEventListener("keydown", (event) => {
  const target = /** @type {HTMLElement | null} */ (event.target instanceof HTMLElement ? event.target : null);
  if (target === list || target?.matches("input, textarea")) {
    return;
  }
  const action = listKey(event);
  if (action === "new" || action === "undo" || action === "search") {
    event.preventDefault();
    if (action === "new") {
      create();
    } else if (action === "undo") {
      undo();
    } else {
      searchField.focus();
    }
  }
});

element("new").addEventListener("click", () => create());
element("back").addEventListener("click", leaveEditor);
element("toast-undo").addEventListener("click", undo);
element("delete").addEventListener("click", () => {
  if (editing) {
    remove(editing);
  }
});
element("copy").addEventListener("click", () => {
  littleJoan.clipboard.writeText(editor.value).then(
    () => {
      say("Copied");
      statusTimer = setTimeout(() => say(""), 1500);
    },
    () => say("Couldn't copy"),
  );
});
window.addEventListener("pagehide", () => {
  void saveNow();
});
// "5 min ago" moves on.
setInterval(renderList, 60_000);

async function start() {
  try {
    await littleJoan.theme.apply();
  } catch {
    // The fallback colors in style.css stay.
  }
  try {
    notes = parseNotes(await littleJoan.storage.get("notes"));
  } catch {
    say("Couldn't read your notes.");
  }
  show("list");
  renderList();
  showUsage();
  // The list takes the keys: N, /, ↑ ↓, Enter.
  list.focus();
}

void start();
