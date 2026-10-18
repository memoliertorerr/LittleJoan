// @ts-check
// The Wayback Machine page action: the page, and two ways into the Internet Archive's copies of
// it — every snapshot (a calendar) or the newest one. Choosing opens web.archive.org in this
// window (littleJoan.openUrl); nothing is fetched before that. The addresses are logic.js.
import { archiveUrls, move } from "./logic.js";

/** @typedef {import("./logic.js").Choice} Choice */

/** @param {string} id */
function element(id) {
  const found = document.getElementById(id);
  if (!found) {
    throw new Error(`#${id} is missing`);
  }
  return found;
}

const status = element("status");
/** @type {Record<Choice, HTMLButtonElement>} */
const buttons = {
  calendar: /** @type {HTMLButtonElement} */ (element("calendar")),
  newest: /** @type {HTMLButtonElement} */ (element("newest")),
};
/** @type {{ calendar: string, newest: string } | null} */
let urls = null;
/** @type {Choice} */
let chosen = "calendar";

/** @param {Choice} choice */
function choose(choice) {
  chosen = choice;
  buttons[choice].focus();
}

/** @param {Choice} choice */
function go(choice) {
  if (!urls) {
    return;
  }
  status.textContent = "Opening the Wayback Machine…";
  littleJoan.openUrl(urls[choice]).catch(() => {
    status.textContent = "Couldn't open it.";
  });
}

for (const choice of /** @type {Choice[]} */ (["calendar", "newest"])) {
  buttons[choice].addEventListener("click", () => go(choice));
  buttons[choice].addEventListener("focus", () => {
    chosen = choice;
  });
}
window.addEventListener("keydown", (event) => {
  if (event.ctrlKey || event.altKey || event.metaKey) {
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    void littleJoan.close();
  } else if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    choose(move(chosen, event.key === "ArrowDown" ? 1 : -1));
  }
});

async function start() {
  try {
    await littleJoan.theme.apply();
  } catch {
    // The fallback colors in style.css stay.
  }
  try {
    const page = await littleJoan.pageInfo();
    urls = page ? archiveUrls(page.url) : null;
    if (page && urls) {
      element("page-title").textContent = page.title || page.url;
      element("page-url").textContent = page.url;
    }
  } catch {
    urls = null;
  }
  if (!urls) {
    element("page-title").textContent = "No page";
    element("page-url").textContent = "Open the Wayback Machine on a web page.";
    buttons.calendar.disabled = true;
    buttons.newest.disabled = true;
    return;
  }
  choose("calendar");
}

void start();
