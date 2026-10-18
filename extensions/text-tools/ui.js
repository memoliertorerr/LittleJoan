// @ts-check
// Text tools' panel: a text box, live counts, and tools for case, encoding and lines that work
// on the selection (or all of the text). Undo takes back a tool; Copy copies the text (clipboard
// permission). The draft is kept in littleJoan.storage. The tools are logic.js.
import { TextToolError, apply, counts } from "./logic.js";

/** A draft longer than this isn't kept (storage holds 1 MB in all). */
const MAX_DRAFT = 256 * 1024;

/** @param {string} id */
function element(id) {
  const found = document.getElementById(id);
  if (!found) {
    throw new Error(`#${id} is missing`);
  }
  return found;
}

const box = /** @type {HTMLTextAreaElement} */ (element("text"));
const countsText = element("counts");
const status = element("status");
const undoButton = /** @type {HTMLButtonElement} */ (element("undo"));
/** @type {{ text: string, start: number, end: number }[]} */
const history = [];
/** The text right after the last tool ran (Ctrl/Cmd+Z then undoes the tool, not typing). */
let lastApplied = "";
/** @type {ReturnType<typeof setTimeout> | undefined} */
let saveTimer;

/** @param {number} n @param {string} one @param {string} many */
const plural = (n, one, many) => `${n.toLocaleString()} ${n === 1 ? one : many}`;

function showCounts() {
  const c = counts(box.value);
  countsText.textContent = [
    plural(c.characters, "character", "characters"),
    plural(c.words, "word", "words"),
    plural(c.lines, "line", "lines"),
    plural(c.bytes, "byte", "bytes"),
    c.readingMinutes ? `${c.readingMinutes} min read` : "",
  ]
    .filter(Boolean)
    .join(" · ");
}

function keepDraft() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    littleJoan.storage.set("draft", box.value.length <= MAX_DRAFT ? box.value : "").catch(() => {
      // Without storage the text lasts while the panel is open.
    });
  }, 800);
}

/** @param {string} text */
function say(text) {
  status.textContent = text;
}

/** @param {string} tool */
function run(tool) {
  try {
    const before = { text: box.value, start: box.selectionStart, end: box.selectionEnd };
    const after = apply(tool, before.text, before.start, before.end);
    if (after.text === before.text) {
      say("Nothing to change.");
      box.focus();
      return;
    }
    history.push(before);
    box.value = after.text;
    lastApplied = after.text;
    box.focus();
    box.setSelectionRange(after.start, after.end);
    undoButton.disabled = false;
    say(before.start === before.end ? "Done (all of the text)." : "Done (the selection).");
    showCounts();
    keepDraft();
  } catch (error) {
    say(error instanceof TextToolError ? error.message : "That didn't work.");
    box.focus();
  }
}

function undo() {
  const before = history.pop();
  if (!before) {
    return;
  }
  box.value = before.text;
  box.focus();
  box.setSelectionRange(before.start, before.end);
  lastApplied = history.length ? box.value : "";
  undoButton.disabled = history.length === 0;
  say("Undone.");
  showCounts();
  keepDraft();
}

for (const button of document.querySelectorAll("button[data-tool]")) {
  if (button instanceof HTMLButtonElement) {
    // Keep the text box's selection: a click on a tool mustn't take the focus first.
    button.addEventListener("mousedown", (event) => event.preventDefault());
    button.addEventListener("click", () => run(button.dataset.tool ?? ""));
  }
}
box.addEventListener("input", () => {
  showCounts();
  keepDraft();
  say("");
});
box.addEventListener("keydown", (event) => {
  if (event.key.toLowerCase() === "z" && (event.ctrlKey || event.metaKey) && !event.shiftKey && history.length && box.value === lastApplied) {
    event.preventDefault();
    undo();
  }
});
undoButton.addEventListener("click", undo);
element("copy").addEventListener("click", () => {
  littleJoan.clipboard.writeText(box.value).then(
    () => say("Copied."),
    () => say("Couldn't copy."),
  );
});
window.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && !event.ctrlKey && !event.altKey && !event.metaKey) {
    event.preventDefault();
    void littleJoan.close();
  }
});

async function start() {
  try {
    await littleJoan.theme.apply();
  } catch {
    // The fallback colors in style.css stay.
  }
  try {
    const draft = await littleJoan.storage.get("draft");
    if (typeof draft === "string") {
      box.value = draft;
    }
  } catch {
    // Start empty.
  }
  showCounts();
  box.focus();
}

void start();
