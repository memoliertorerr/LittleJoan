// @ts-check
// The world clock's rules, pure: finding cities, the time in a zone at an instant (Intl does the
// daylight-saving work), how far ahead or behind it is, which day it is there, and the list of
// cities the panel keeps (checked from scratch when read). The clock is passed in, so
// logic.test.ts can stand on either side of a daylight-saving change.
import { CITIES } from "./cities.js";

/** @typedef {import("./cities.js").City} City */
/** @typedef {{ year: number, month: number, day: number, hour: number, minute: number, second: number }} WallTime */
/** @typedef {"night" | "morning" | "day" | "evening"} Daypart */

/** At most this many cities in the list. */
export const MAX_CITIES = 24;
/** The first list, for someone who hasn't picked any yet. */
const STARTERS = ["london", "new-york", "tokyo", "sydney"];

/**
 * Text for matching: lower case, without accents ("sao" finds "São Paulo").
 * @param {string} text
 */
export function fold(text) {
  return text.normalize("NFD").replace(/[̀-ͯ]/g, "").replace(/ı/g, "i").toLowerCase();
}

/**
 * A city's id: its folded name in dashes ("new-york", "st-john-s").
 * @param {City} city
 */
export function cityId(city) {
  return fold(city.name)
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

/** @type {Map<string, City>} */
export const BY_ID = new Map(CITIES.map((city) => [cityId(city), city]));

/**
 * Cities matching what was typed: every word must start a word of the city's name or country.
 * Names that start with the query come first, then other name matches, then country matches.
 * @param {string} query
 * @param {readonly string[]} [exclude] ids already in the list
 * @param {number} [limit]
 * @returns {{ id: string, city: City }[]}
 */
export function search(query, exclude = [], limit = 8) {
  const words = fold(query).split(/[\s,]+/).filter(Boolean);
  if (words.length === 0) {
    return [];
  }
  const typed = words.join(" ");
  /** @type {{ id: string, city: City, rank: number }[]} */
  const found = [];
  for (const [id, city] of BY_ID) {
    if (exclude.includes(id)) {
      continue;
    }
    const name = fold(city.name);
    const nameWords = name.split(/[\s'.-]+/);
    const countryWords = fold(city.country).split(/[\s'.-]+/);
    const every = words.every((word) => nameWords.some((w) => w.startsWith(word)) || countryWords.some((w) => w.startsWith(word)));
    if (!every) {
      continue;
    }
    const rank = name.startsWith(typed) ? 0 : words.every((word) => nameWords.some((w) => w.startsWith(word))) ? 1 : 2;
    found.push({ id, city, rank });
  }
  found.sort((a, b) => a.rank - b.rank || a.city.name.localeCompare(b.city.name));
  return found.slice(0, limit).map(({ id, city }) => ({ id, city }));
}

/**
 * The wall-clock time in `zone` at `instant` (unix ms).
 * @param {string} zone
 * @param {number} instant
 * @returns {WallTime}
 */
export function wallTime(zone, instant) {
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone: zone,
    hourCycle: "h23",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  }).formatToParts(instant);
  const get = (/** @type {Intl.DateTimeFormatPartTypes} */ type) => Number(parts.find((part) => part.type === type)?.value ?? 0);
  return { year: get("year"), month: get("month"), day: get("day"), hour: get("hour") % 24, minute: get("minute"), second: get("second") };
}

/**
 * How far `zone` is ahead of UTC at `instant`, in minutes.
 * @param {string} zone
 * @param {number} instant
 */
export function offsetMinutes(zone, instant) {
  const wall = wallTime(zone, instant);
  const asUtc = Date.UTC(wall.year, wall.month - 1, wall.day, wall.hour, wall.minute, wall.second);
  return Math.round((asUtc - Math.floor(instant / 1000) * 1000) / 60_000);
}

/**
 * How far `zone` is ahead of `here` at `instant`, in minutes (negative: behind).
 * @param {string} zone
 * @param {string} here
 * @param {number} instant
 */
export function difference(zone, here, instant) {
  return offsetMinutes(zone, instant) - offsetMinutes(here, instant);
}

/**
 * "Same time", "+9 h", "−4 h", "+5 h 30 min".
 * @param {number} minutes
 */
export function differenceText(minutes) {
  if (minutes === 0) {
    return "Same time";
  }
  const sign = minutes > 0 ? "+" : "−";
  const hours = Math.floor(Math.abs(minutes) / 60);
  const rest = Math.abs(minutes) % 60;
  return rest === 0 ? `${sign}${hours} h` : hours === 0 ? `${sign}${rest} min` : `${sign}${hours} h ${rest} min`;
}

/**
 * "UTC", "UTC+9", "UTC−4", "UTC+5:30".
 * @param {number} minutes
 */
export function utcText(minutes) {
  if (minutes === 0) {
    return "UTC";
  }
  const sign = minutes > 0 ? "+" : "−";
  const hours = Math.floor(Math.abs(minutes) / 60);
  const rest = Math.abs(minutes) % 60;
  return `UTC${sign}${hours}${rest ? `:${String(rest).padStart(2, "0")}` : ""}`;
}

/**
 * Which day it is in `zone` compared with `here`: "Today", "Tomorrow" or "Yesterday".
 * @param {string} zone
 * @param {string} here
 * @param {number} instant
 */
export function dayLabel(zone, here, instant) {
  const there = wallTime(zone, instant);
  const local = wallTime(here, instant);
  const days = Math.round((Date.UTC(there.year, there.month - 1, there.day) - Date.UTC(local.year, local.month - 1, local.day)) / 86_400_000);
  return days > 0 ? "Tomorrow" : days < 0 ? "Yesterday" : "Today";
}

/**
 * The part of the day an hour falls in (for the little sun or moon).
 * @param {number} hour 0–23
 * @returns {Daypart}
 */
export function daypart(hour) {
  if (hour >= 6 && hour < 12) {
    return "morning";
  }
  if (hour >= 12 && hour < 18) {
    return "day";
  }
  if (hour >= 18 && hour < 21) {
    return "evening";
  }
  return "night";
}

/**
 * The time shown on a clock, the way `locale` writes it ("14:05", "2:05 PM").
 * @param {string} zone
 * @param {number} instant
 * @param {string} [locale]
 */
export function clockText(zone, instant, locale) {
  return new Intl.DateTimeFormat(locale, { timeZone: zone, hour: "numeric", minute: "2-digit" }).format(instant);
}

/**
 * A zone's own name for "your time" when it isn't in the list: `Europe/Istanbul` → "Istanbul".
 * @param {string} zone
 */
export function zoneName(zone) {
  const known = CITIES.find((city) => city.zone === zone);
  return known ? known.name : (zone.split("/").at(-1) ?? zone).replace(/_/g, " ");
}

/**
 * The saved list, checked from scratch: known city ids, each once, at most `MAX_CITIES`. Nothing
 * saved yet (undefined) gives the starters, minus the zone you're in.
 * @param {unknown} saved
 * @param {string} here
 * @param {number} instant
 * @returns {string[]}
 */
export function parseList(saved, here, instant) {
  if (saved === undefined) {
    const local = offsetMinutes(here, instant);
    return STARTERS.filter((id) => {
      const city = BY_ID.get(id);
      return city && city.zone !== here && offsetMinutes(city.zone, instant) !== local;
    }).slice(0, 3);
  }
  if (!Array.isArray(saved)) {
    return [];
  }
  /** @type {string[]} */
  const ids = [];
  for (const id of /** @type {unknown[]} */ (saved)) {
    if (typeof id === "string" && BY_ID.has(id) && !ids.includes(id)) {
      ids.push(id);
    }
  }
  return ids.slice(0, MAX_CITIES);
}

/**
 * The list with `id` at the end (once; known cities only; at most `MAX_CITIES`).
 * @param {readonly string[]} list
 * @param {string} id
 */
export function addCity(list, id) {
  return BY_ID.has(id) && !list.includes(id) && list.length < MAX_CITIES ? [...list, id] : [...list];
}

/**
 * The list without `id`.
 * @param {readonly string[]} list
 * @param {string} id
 */
export function removeCity(list, id) {
  return list.filter((other) => other !== id);
}

/**
 * The list with `id` moved one place up (-1) or down (+1).
 * @param {readonly string[]} list
 * @param {string} id
 * @param {-1 | 1} step
 */
export function moveCity(list, id, step) {
  const from = list.indexOf(id);
  const to = from + step;
  if (from < 0 || to < 0 || to >= list.length) {
    return [...list];
  }
  const copy = [...list];
  copy.splice(from, 1);
  copy.splice(to, 0, id);
  return copy;
}

/** @typedef {"up" | "down" | "move-up" | "move-down" | "remove" | "add" | "escape"} RowAction */

/**
 * What a key press means on a city's row: ↑ ↓ move between rows, Alt+↑ ↓ move the city,
 * Delete remove it, A or / add a city, Esc leave.
 * @param {{ key: string, code?: string, ctrlKey?: boolean, altKey?: boolean, metaKey?: boolean }} event
 * @returns {RowAction | null}
 */
export function rowKey(event) {
  if (event.ctrlKey || event.metaKey) {
    return null;
  }
  if (event.altKey) {
    return event.key === "ArrowUp" ? "move-up" : event.key === "ArrowDown" ? "move-down" : null;
  }
  /** @type {Record<string, RowAction>} */
  const keys = { ArrowUp: "up", ArrowDown: "down", Delete: "remove", Backspace: "remove", "/": "add", Escape: "escape" };
  return keys[event.key] ?? (event.code === "KeyA" ? "add" : null);
}
