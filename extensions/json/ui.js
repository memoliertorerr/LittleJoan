// @ts-check
// The JSON tool's panel: paste JSON, see it formatted (two or four spaces, tabs, or minified;
// keys sorted if you like) with highlighting, or the error's line and column with a caret. Copy
// takes the result; Ctrl/Cmd+Enter puts it in place of the text. The draft is kept in
// littleJoan.storage. The parser and printers are logic.js.
import { INDENTS, excerpt, parse, sortKeys, stats, tokens } from "./logic.js";

/** @typedef {import("./logic.js").Style} Style */
/** @typedef {import("./logic.js").Token} Token */

/** Past this many tokens the result is shown without colors (it stays quick). */
const MAX_HIGHLIGHT = 30_000;
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

const input = /** @type {HTMLTextAreaElement} */ (element("input"));
const output = element("output");
const status = element("status");
const styleSelect = /** @type {HTMLSelectElement} */ (element("style"));
const sortButton = /** @type {HTMLButtonElement} */ (element("sort"));
const copyButton = /** @type {HTMLButtonElement} */ (element("copy"));
let result = "";
/** @type {ReturnType<typeof setTimeout> | undefined} */
let parseTimer;
/** @type {ReturnType<typeof setTimeout> | undefined} */
let saveTimer;

/** @param {Token[]} list */
function paint(list) {
  if (list.length > MAX_HIGHLIGHT) {
    output.textContent = list.map(([, text]) => text).join("");
    return;
  }
  output.replaceChildren(
    ...list.map(([kind, text]) => {
      if (kind === "space") {
        return document.createTextNode(text);
      }
      const span = document.createElement("span");
      span.className = `tk-${kind}`;
      span.textContent = text;
      return span;
    }),
  );
}

function update() {
  const text = input.value;
  copyButton.disabled = true;
  status.className = "status";
  if (!text.trim()) {
    result = "";
    output.replaceChildren();
    status.textContent = "Paste or type JSON above.";
    return;
  }
  const parsed = parse(text);
  if (!parsed.ok) {
    result = "";
    status.classList.add("bad");
    status.textContent = `Line ${parsed.error.line}, column ${parsed.error.column}: ${parsed.error.message}`;
    output.textContent = excerpt(text, parsed.error);
    output.classList.add("error");
    return;
  }
  output.classList.remove("error");
  const node = sortButton.getAttribute("aria-pressed") === "true" ? sortKeys(parsed.value) : parsed.value;
  const list = tokens(node, INDENTS[/** @type {Style} */ (styleSelect.value)] ?? "  ");
  result = list.map(([, part]) => part).join("");
  paint(list);
  const counts = stats(parsed.value);
  status.classList.add("good");
  status.textContent = `Valid JSON · ${counts.objects} ${counts.objects === 1 ? "object" : "objects"}, ${counts.arrays} ${counts.arrays === 1 ? "array" : "arrays"}, ${counts.values} ${counts.values === 1 ? "value" : "values"}, ${counts.depth} ${counts.depth === 1 ? "level" : "levels"} deep`;
  copyButton.disabled = false;
}

function keepDraft() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    const draft = input.value.length <= MAX_DRAFT ? input.value : "";
    littleJoan.storage.set("draft", draft).catch(() => {
      // Without storage the draft lasts while the panel is open.
    });
  }, 800);
}

input.addEventListener("input", () => {
  clearTimeout(parseTimer);
  parseTimer = setTimeout(update, 120);
  keepDraft();
});
styleSelect.addEventListener("change", () => {
  update();
  littleJoan.storage.set("style", styleSelect.value).catch(() => {
    // Not kept, then.
  });
});
sortButton.addEventListener("click", () => {
  sortButton.setAttribute("aria-pressed", String(sortButton.getAttribute("aria-pressed") !== "true"));
  update();
});
copyButton.addEventListener("click", () => {
  littleJoan.clipboard.writeText(result).then(
    () => {
      copyButton.textContent = "Copied";
      setTimeout(() => {
        copyButton.textContent = "Copy";
      }, 1200);
    },
    () => {
      status.textContent = "Couldn't copy.";
    },
  );
});
window.addEventListener("keydown", (event) => {
  if (event.key === "Enter" && (event.ctrlKey || event.metaKey) && !event.altKey) {
    event.preventDefault();
    update();
    if (result) {
      input.value = result;
      keepDraft();
      input.focus();
    }
  } else if (event.key === "Escape" && !event.ctrlKey && !event.altKey && !event.metaKey) {
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
    const [draft, style] = await Promise.all([littleJoan.storage.get("draft"), littleJoan.storage.get("style")]);
    if (typeof draft === "string") {
      input.value = draft;
    }
    if (typeof style === "string" && style in INDENTS) {
      styleSelect.value = style;
    }
  } catch {
    // Start empty.
  }
  update();
  input.focus();
}

void start();
