// @ts-check
// The QR code page action: the page's address as a QR code to scan with a phone (dark on white,
// whatever the theme, so every camera reads it). The text can be edited; the code follows.
// The encoder is logic.js.
import { byteCapacity, encode, MAX_VERSION, svgPath, tooLong } from "./logic.js";

const SVG = "http://www.w3.org/2000/svg";
/** Modules of light margin around the code (the standard's quiet zone). */
const BORDER = 4;

/** @param {string} id */
function element(id) {
  const found = document.getElementById(id);
  if (!found) {
    throw new Error(`#${id} is missing`);
  }
  return found;
}

const holder = element("code");
const field = /** @type {HTMLTextAreaElement} */ (element("text"));
const info = element("info");
const pageTitle = element("page-title");
const pageHost = element("page-host");
const reset = /** @type {HTMLButtonElement} */ (element("reset"));
let pageUrl = "";

function draw() {
  const text = field.value;
  const qr = text ? encode(text) : null;
  holder.replaceChildren();
  holder.classList.toggle("empty", !qr);
  if (!qr) {
    info.textContent = text ? tooLong(text) : "Type something to encode.";
    info.classList.add("warning");
    return;
  }
  const side = qr.size + 2 * BORDER;
  const svg = document.createElementNS(SVG, "svg");
  svg.setAttribute("viewBox", `0 0 ${side} ${side}`);
  svg.setAttribute("role", "img");
  svg.setAttribute("aria-label", `QR code for ${text}`);
  svg.setAttribute("shape-rendering", "crispEdges");
  const paper = document.createElementNS(SVG, "rect");
  paper.setAttribute("width", String(side));
  paper.setAttribute("height", String(side));
  paper.setAttribute("fill", "#ffffff");
  const ink = document.createElementNS(SVG, "path");
  ink.setAttribute("d", svgPath(qr, BORDER));
  ink.setAttribute("fill", "#000000");
  svg.append(paper, ink);
  holder.append(svg);
  const bytes = new TextEncoder().encode(text).length;
  info.textContent = `Version ${qr.version} · ${bytes} of ${byteCapacity(MAX_VERSION)} bytes · scan it with a phone's camera`;
  info.classList.remove("warning");
  reset.disabled = text === pageUrl;
}

field.addEventListener("input", draw);
field.addEventListener("keydown", (event) => {
  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
  }
});
reset.addEventListener("click", () => {
  field.value = pageUrl;
  draw();
  field.focus();
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
    const page = await littleJoan.pageInfo();
    if (page) {
      pageUrl = page.url;
      pageTitle.textContent = page.title || page.url;
      pageHost.textContent = new URL(page.url).host;
    } else {
      pageTitle.textContent = "No page";
      pageHost.textContent = "Open QR code on a web page to see its address here.";
    }
  } catch {
    pageTitle.textContent = "No page";
  }
  field.value = pageUrl;
  draw();
  field.focus();
  field.setSelectionRange(field.value.length, field.value.length);
}

void start();
