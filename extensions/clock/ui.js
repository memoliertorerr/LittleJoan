// @ts-check
// The world clock's panel: your time, then the cities you picked — each with its time, how far
// ahead or behind it is, and which day it is there. Add a city in the field at the top (type,
// ↑ ↓, Enter); on a row, Delete removes it and Alt+↑ ↓ moves it. The list is kept in
// littleJoan.storage. The rules are logic.js.
import { BY_ID, addCity, clockText, dayLabel, daypart, difference, differenceText, moveCity, offsetMinutes, parseList, removeCity, rowKey, search, utcText, wallTime, zoneName } from "./logic.js";

/** @param {string} id */
function element(id) {
  const found = document.getElementById(id);
  if (!found) {
    throw new Error(`#${id} is missing`);
  }
  return found;
}

const field = /** @type {HTMLInputElement} */ (element("add"));
const suggestions = element("suggestions");
const list = element("cities");
const hereTime = element("here-time");
const herePlace = element("here-place");
const hereDate = element("here-date");
const here = Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
const longDate = new Intl.DateTimeFormat(undefined, { timeZone: here, weekday: "long", day: "numeric", month: "long" });

/** @type {string[]} */
let cities = [];
/** @type {{ id: string, city: import("./cities.js").City }[]} */
let matches = [];
let picked = 0;
/** The row to focus after the next draw (by city id). */
/** @type {string | null} */
let focusAfter = null;

function persist() {
  littleJoan.storage.set("cities", cities).catch(() => {
    // Without storage the list lasts while the panel is open.
  });
}

/** @param {string[]} next */
function update(next) {
  cities = next;
  persist();
  draw();
}

function draw() {
  const now = Date.now();
  hereTime.textContent = clockText(here, now);
  const name = zoneName(here);
  const utc = utcText(offsetMinutes(here, now));
  herePlace.textContent = name === utc ? name : `${name} · ${utc}`;
  hereDate.textContent = longDate.format(now);
  /** @type {HTMLElement[]} */
  const rows = [];
  for (const id of cities) {
    const city = BY_ID.get(id);
    if (!city) {
      continue;
    }
    const row = document.createElement("li");
    row.tabIndex = 0;
    row.dataset.id = id;
    row.className = `city is-${daypart(wallTime(city.zone, now).hour)}`;
    row.setAttribute("aria-label", `${city.name}, ${clockText(city.zone, now)}, ${dayLabel(city.zone, here, now)}`);
    const sky = document.createElement("span");
    sky.className = "sky";
    sky.setAttribute("aria-hidden", "true");
    const place = document.createElement("div");
    place.className = "place";
    const name = document.createElement("strong");
    name.textContent = city.name;
    const detail = document.createElement("span");
    detail.className = "detail";
    detail.textContent = `${city.country} · ${differenceText(difference(city.zone, here, now))}`;
    place.append(name, detail);
    const time = document.createElement("div");
    time.className = "time";
    const clock = document.createElement("span");
    clock.className = "clock";
    clock.textContent = clockText(city.zone, now);
    const day = document.createElement("span");
    day.className = "day";
    day.textContent = dayLabel(city.zone, here, now);
    time.append(clock, day);
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "btn quiet remove";
    remove.textContent = "×";
    remove.tabIndex = -1;
    remove.setAttribute("aria-label", `Remove ${city.name}`);
    remove.addEventListener("click", () => update(removeCity(cities, id)));
    row.append(sky, place, time, remove);
    rows.push(row);
  }
  list.replaceChildren(...rows);
  element("empty").hidden = cities.length > 0;
  if (focusAfter) {
    const target = [...list.children].find((row) => row instanceof HTMLElement && row.dataset.id === focusAfter);
    if (target instanceof HTMLElement) {
      target.focus();
    }
    focusAfter = null;
  }
}

function drawSuggestions() {
  matches = search(field.value, cities);
  picked = Math.min(picked, Math.max(0, matches.length - 1));
  suggestions.replaceChildren(
    ...matches.map((match, index) => {
      const option = document.createElement("li");
      option.id = `match-${match.id}`;
      option.setAttribute("role", "option");
      option.setAttribute("aria-selected", String(index === picked));
      const name = document.createElement("strong");
      name.textContent = match.city.name;
      const detail = document.createElement("span");
      const now = Date.now();
      detail.textContent = `${match.city.country} · ${clockText(match.city.zone, now)}`;
      option.append(name, detail);
      option.addEventListener("mousedown", (event) => {
        event.preventDefault();
        add(match.id);
      });
      return option;
    }),
  );
  const open = matches.length > 0;
  suggestions.hidden = !open;
  field.setAttribute("aria-expanded", String(open));
  if (open) {
    field.setAttribute("aria-activedescendant", `match-${matches[picked]?.id ?? ""}`);
  } else {
    field.removeAttribute("aria-activedescendant");
  }
}

/** @param {string} id */
function add(id) {
  update(addCity(cities, id));
  field.value = "";
  picked = 0;
  drawSuggestions();
  field.focus();
}

field.addEventListener("input", () => {
  picked = 0;
  drawSuggestions();
});
field.addEventListener("keydown", (event) => {
  if (event.key === "ArrowDown" && matches.length > 0) {
    event.preventDefault();
    picked = Math.min(matches.length - 1, picked + 1);
    drawSuggestions();
  } else if (event.key === "ArrowDown" && cities.length > 0) {
    event.preventDefault();
    /** @type {HTMLElement | null} */ (list.firstElementChild)?.focus();
  } else if (event.key === "ArrowUp" && matches.length > 0) {
    event.preventDefault();
    picked = Math.max(0, picked - 1);
    drawSuggestions();
  } else if (event.key === "Enter") {
    event.preventDefault();
    const match = matches[picked];
    if (match) {
      add(match.id);
    }
  } else if (event.key === "Escape") {
    event.preventDefault();
    if (field.value) {
      field.value = "";
      drawSuggestions();
    } else {
      void littleJoan.close();
    }
  }
});
field.addEventListener("blur", () => {
  suggestions.hidden = true;
});
field.addEventListener("focus", drawSuggestions);

list.addEventListener("keydown", (event) => {
  const row = event.target instanceof HTMLElement ? event.target.closest("li") : null;
  const id = row?.dataset.id;
  const action = rowKey(event);
  if (!row || !id || !action) {
    return;
  }
  event.preventDefault();
  switch (action) {
    case "up":
      if (row.previousElementSibling instanceof HTMLElement) {
        row.previousElementSibling.focus();
      } else {
        field.focus();
      }
      break;
    case "down":
      if (row.nextElementSibling instanceof HTMLElement) {
        row.nextElementSibling.focus();
      }
      break;
    case "move-up":
    case "move-down":
      focusAfter = id;
      update(moveCity(cities, id, action === "move-up" ? -1 : 1));
      break;
    case "remove": {
      const neighbour = row.nextElementSibling ?? row.previousElementSibling;
      focusAfter = neighbour instanceof HTMLElement ? (neighbour.dataset.id ?? null) : null;
      update(removeCity(cities, id));
      if (!focusAfter) {
        field.focus();
      }
      break;
    }
    case "add":
      field.focus();
      break;
    case "escape":
      void littleJoan.close();
      break;
  }
});

// The clocks move on: redraw when the minute changes (checked every second).
let shownMinute = -1;
setInterval(() => {
  const minute = Math.floor(Date.now() / 60_000);
  if (minute !== shownMinute) {
    shownMinute = minute;
    const focused = document.activeElement instanceof HTMLElement ? document.activeElement.dataset.id : undefined;
    focusAfter = focused ?? null;
    draw();
  }
}, 1000);

async function start() {
  try {
    await littleJoan.theme.apply();
  } catch {
    // The fallback colors in style.css stay.
  }
  let saved;
  try {
    saved = await littleJoan.storage.get("cities");
  } catch {
    saved = undefined;
  }
  cities = parseList(saved, here, Date.now());
  shownMinute = Math.floor(Date.now() / 60_000);
  draw();
  field.focus();
}

void start();
