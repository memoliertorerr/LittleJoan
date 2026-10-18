// @ts-check
// The Copy as Markdown link page action: the page's title (editable) and address as a link in
// the format picked — Markdown by default — previewed, then copied with Enter (and back to the
// page). The formatting is logic.js.
import { FORMATS, link, move, trimSiteName } from "./logic.js";

/** @typedef {import("./logic.js").Format} Format */

/** @param {string} id */
function element(id) {
  const found = document.getElementById(id);
  if (!found) {
    throw new Error(`#${id} is missing`);
  }
  return found;
}

const titleField = /** @type {HTMLInputElement} */ (element("title"));
const preview = element("preview");
const formats = element("formats");
const status = element("status");
const trimButton = /** @type {HTMLButtonElement} */ (element("trim"));
const copyButton = /** @type {HTMLButtonElement} */ (element("copy"));
let pageUrl = "";
let pageTitle = "";
/** @type {Format} */
let format = "markdown";
let busy = false;

function render() {
  preview.textContent = pageUrl ? link(format, titleField.value, pageUrl) : "";
  for (const option of formats.querySelectorAll("button")) {
    const on = option.dataset.format === format;
    option.setAttribute("aria-checked", String(on));
    option.tabIndex = on ? 0 : -1;
  }
  trimButton.hidden = trimSiteName(titleField.value) === titleField.value.trim();
}

/** @param {Format} next */
function pick(next) {
  format = next;
  render();
}

async function copy() {
  if (!pageUrl || busy) {
    return;
  }
  busy = true;
  try {
    await littleJoan.clipboard.writeText(link(format, titleField.value, pageUrl));
    status.textContent = "Copied. Back to the page…";
    setTimeout(() => void littleJoan.close(), 700);
  } catch {
    status.textContent = "Couldn't copy.";
    busy = false;
  }
}

formats.replaceChildren(
  ...FORMATS.map(({ id, label }) => {
    const option = document.createElement("button");
    option.type = "button";
    option.className = "format";
    option.setAttribute("role", "radio");
    option.dataset.format = id;
    option.textContent = label;
    option.addEventListener("click", () => {
      pick(id);
      titleField.focus();
    });
    return option;
  }),
);
titleField.addEventListener("input", render);
trimButton.addEventListener("click", () => {
  titleField.value = trimSiteName(titleField.value);
  render();
  titleField.focus();
});
copyButton.addEventListener("click", () => void copy());
window.addEventListener("keydown", (event) => {
  if (event.ctrlKey || event.altKey || event.metaKey) {
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    void littleJoan.close();
  } else if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    pick(move(format, event.key === "ArrowDown" ? 1 : -1));
  } else if (event.key === "Enter" && !(event.target instanceof HTMLButtonElement && event.target !== copyButton)) {
    event.preventDefault();
    void copy();
  }
});

async function start() {
  try {
    await littleJoan.theme.apply();
  } catch {
    // The fallback colors in style.css stay.
  }
  const page = await littleJoan.pageInfo().catch(() => null);
  if (!page) {
    status.textContent = "Open Copy as Markdown link on a web page.";
    copyButton.disabled = true;
    render();
    return;
  }
  pageUrl = page.url;
  pageTitle = page.title;
  titleField.value = pageTitle;
  element("url").textContent = pageUrl;
  render();
  titleField.focus();
  titleField.select();
}

void start();
