// @ts-check
// Color tools' panel: type a color in any CSS form (or pick one), see it as hex, RGB, HSL and
// OKLCH with a Copy for each, and its contrast as text on white, on black and on a second color,
// with the WCAG levels it passes. Little Joan's own theme colors are a click away. The last colors
// are kept in littleJoan.storage. The color math is logic.js.
import { contrast, levels, parse, ratioText, textOn, toHex, toHslString, toOklchString, toRgbString } from "./logic.js";

/** @typedef {import("./logic.js").Rgb} Rgb */

/** @param {string} id */
function element(id) {
  const found = document.getElementById(id);
  if (!found) {
    throw new Error(`#${id} is missing`);
  }
  return found;
}

const field = /** @type {HTMLInputElement} */ (element("color"));
const picker = /** @type {HTMLInputElement} */ (element("picker"));
const swatch = element("swatch");
const formats = element("formats");
const against = /** @type {HTMLInputElement} */ (element("against"));
const checks = element("checks");
const sample = element("sample");
const themeChips = element("theme");
const status = element("status");
const WHITE = { r: 255, g: 255, b: 255, a: 1 };
const BLACK = { r: 0, g: 0, b: 0, a: 1 };
/** @type {Rgb} */
let color = { r: 72, g: 91, b: 125, a: 1 };
/** @type {Rgb} */
let second = WHITE;
/** @type {ReturnType<typeof setTimeout> | undefined} */
let saveTimer;

/** @param {string} text */
function say(text) {
  status.textContent = text;
}

/**
 * @param {string} label
 * @param {string} value
 */
function formatRow(label, value) {
  const row = document.createElement("div");
  row.className = "format";
  const name = document.createElement("span");
  name.className = "name";
  name.textContent = label;
  const code = document.createElement("code");
  code.textContent = value;
  const copy = document.createElement("button");
  copy.type = "button";
  copy.className = "btn quiet";
  copy.textContent = "Copy";
  copy.setAttribute("aria-label", `Copy ${label}: ${value}`);
  copy.addEventListener("click", () => {
    littleJoan.clipboard.writeText(value).then(
      () => say(`Copied ${value}`),
      () => say("Couldn't copy."),
    );
  });
  row.append(name, code, copy);
  return row;
}

/**
 * @param {string} label
 * @param {Rgb} fg
 * @param {Rgb} bg
 */
function checkRow(label, fg, bg) {
  const ratio = contrast(fg, bg);
  const passes = levels(ratio);
  const row = document.createElement("div");
  row.className = "check";
  const name = document.createElement("span");
  name.textContent = label;
  const value = document.createElement("strong");
  value.textContent = ratioText(ratio);
  const badges = document.createElement("span");
  badges.className = "badges";
  for (const [text, ok] of /** @type {[string, boolean][]} */ ([
    ["AA", passes.aa],
    ["AA large", passes.aaLarge],
    ["AAA", passes.aaa],
  ])) {
    const badge = document.createElement("span");
    badge.className = ok ? "badge pass" : "badge fail";
    badge.textContent = `${ok ? "✓" : "✕"} ${text}`;
    badges.append(badge);
  }
  row.append(name, value, badges);
  return row;
}

function render() {
  const hex = toHex(color);
  swatch.style.background = toRgbString(color);
  swatch.style.color = textOn(color);
  swatch.textContent = hex;
  picker.value = hex.slice(0, 7);
  formats.replaceChildren(formatRow("HEX", hex), formatRow("RGB", toRgbString(color)), formatRow("HSL", toHslString(color)), formatRow("OKLCH", toOklchString(color)));
  checks.replaceChildren(checkRow("On white", color, WHITE), checkRow("On black", color, BLACK), checkRow(`On ${toHex(second)}`, color, second));
  sample.style.background = toRgbString(second);
  sample.style.color = toRgbString(color);
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    littleJoan.storage.set("colors", { color: hex, against: toHex(second) }).catch(() => {
      // Not kept, then.
    });
  }, 500);
}

/** @param {string} text */
function useColor(text) {
  const parsed = parse(text);
  if (parsed) {
    color = parsed;
    field.classList.remove("invalid");
    say("");
    render();
  } else {
    field.classList.add("invalid");
    say(text.trim() ? "Not a color I know — try #ff8800, rgb(255 136 0), hsl(32 100% 50%) or oklch(0.75 0.18 60)." : "");
  }
}

field.addEventListener("input", () => useColor(field.value));
picker.addEventListener("input", () => {
  field.value = picker.value;
  useColor(picker.value);
});
against.addEventListener("input", () => {
  const parsed = parse(against.value);
  against.classList.toggle("invalid", !parsed && against.value.trim() !== "");
  if (parsed) {
    second = parsed;
    render();
  }
});
window.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && !event.ctrlKey && !event.altKey && !event.metaKey) {
    event.preventDefault();
    void littleJoan.close();
  }
});

/**
 * Little Joan's theme colors as chips that load into the field.
 * @param {Record<string, string>} vars
 */
function showTheme(vars) {
  const chips = /** @type {[string, string][]} */ ([
    ["Background", "--lj-bg-0"],
    ["Text", "--lj-fg"],
    ["Accent", "--lj-accent"],
    ["Selected", "--lj-row-selected"],
  ]).flatMap(([label, name]) => {
    const parsed = parse(vars[name] ?? "");
    if (!parsed) {
      return [];
    }
    const chip = document.createElement("button");
    chip.type = "button";
    chip.className = "chip";
    chip.title = `${label}: ${toHex(parsed)}`;
    const dot = document.createElement("span");
    dot.className = "dot";
    dot.style.background = toRgbString(parsed);
    const text = document.createElement("span");
    text.textContent = label;
    chip.append(dot, text);
    chip.addEventListener("click", () => {
      field.value = toHex(parsed);
      useColor(field.value);
      field.focus();
    });
    return [chip];
  });
  themeChips.replaceChildren(...chips);
  themeChips.hidden = chips.length === 0;
}

async function start() {
  try {
    const theme = await littleJoan.theme.apply();
    showTheme(theme.vars);
  } catch {
    themeChips.hidden = true;
  }
  try {
    const saved = /** @type {{ color?: unknown, against?: unknown } | undefined} */ (await littleJoan.storage.get("colors"));
    const first = typeof saved?.color === "string" ? parse(saved.color) : null;
    const other = typeof saved?.against === "string" ? parse(saved.against) : null;
    if (first) {
      color = first;
    }
    if (other) {
      second = other;
    }
  } catch {
    // The defaults stay.
  }
  field.value = toHex(color);
  against.value = toHex(second);
  render();
  field.focus();
  field.select();
}

void start();
