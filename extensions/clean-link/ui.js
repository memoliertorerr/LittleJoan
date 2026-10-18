// @ts-check
// The Clean link page action: the page's address without its tracking parameters (and out of
// redirect wrappers), what was taken out, and Copy (Enter) or Open. The rules are logic.js.
import { clean } from "./logic.js";

/** @param {string} id */
function element(id) {
  const found = document.getElementById(id);
  if (!found) {
    throw new Error(`#${id} is missing`);
  }
  return found;
}

const result = element("result");
const before = element("before");
const removedList = element("removed");
const verdict = element("verdict");
const status = element("status");
const copyButton = /** @type {HTMLButtonElement} */ (element("copy"));
const openButton = /** @type {HTMLButtonElement} */ (element("open"));
let cleanUrl = "";
let busy = false;

/** @param {string} text */
function say(text) {
  status.textContent = text;
}

async function copy() {
  if (!cleanUrl || busy) {
    return;
  }
  busy = true;
  try {
    await littleJoan.clipboard.writeText(cleanUrl);
    say("Copied. Back to the page…");
    setTimeout(() => void littleJoan.close(), 700);
  } catch {
    say("Couldn't copy.");
    busy = false;
  }
}

copyButton.addEventListener("click", () => void copy());
openButton.addEventListener("click", () => {
  if (cleanUrl) {
    littleJoan.openUrl(cleanUrl).catch(() => say("Couldn't open it."));
  }
});
window.addEventListener("keydown", (event) => {
  if (event.ctrlKey || event.altKey || event.metaKey) {
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    void littleJoan.close();
  } else if (event.key === "Enter" && !(event.target instanceof HTMLButtonElement)) {
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
  const cleaned = page ? clean(page.url) : null;
  if (!page || !cleaned) {
    verdict.textContent = "Open Clean link on a web page.";
    copyButton.disabled = true;
    openButton.disabled = true;
    return;
  }
  cleanUrl = cleaned.url;
  result.textContent = cleaned.url;
  before.textContent = page.url;
  const changed = cleaned.url !== page.url;
  verdict.textContent = changed
    ? cleaned.unwrapped
      ? `Taken out of a ${cleaned.unwrapped} redirect${cleaned.removed.length ? `, ${cleaned.removed.length} tracking ${cleaned.removed.length === 1 ? "part" : "parts"} removed` : ""}.`
      : `${cleaned.removed.length} tracking ${cleaned.removed.length === 1 ? "part" : "parts"} removed.`
    : "This link is already clean.";
  element("app").classList.toggle("unchanged", !changed);
  removedList.replaceChildren(
    ...cleaned.removed.map((name) => {
      const chip = document.createElement("li");
      chip.textContent = name;
      return chip;
    }),
  );
  openButton.disabled = !changed;
  copyButton.focus();
}

void start();
