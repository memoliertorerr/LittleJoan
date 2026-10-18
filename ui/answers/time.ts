// The time-zone quick answer (pure, unit-tested; the clock and the locale are passed in): "time in
// Tokyo" (also "tokyo time"), and "3pm london to new york" — that time today in the first city,
// in the second. Cities come from cities.ts; all zone math goes through Intl, so daylight saving
// time follows the engine's own time-zone data.
import { CITIES } from "./cities.ts";
import type { Answer } from "./types.ts";

/** The city's time zone, by name (any case, extra spaces and dots ignored). */
export function zoneFor(city: string): string | undefined {
  const key = city.trim().toLowerCase().replace(/\./g, "").replace(/\s+/g, " ");
  return Object.hasOwn(CITIES, key) ? CITIES[key] : undefined;
}

/** A city as the answer names it: words capitalized, short ones (NYC, UTC) in capitals. */
export function cityName(typed: string): string {
  const words = typed.trim().replace(/\s+/g, " ").split(" ");
  if (words.length === 1 && (words[0] ?? "").length <= 3) {
    return (words[0] ?? "").toUpperCase();
  }
  return words.map((word) => (word ? word.charAt(0).toUpperCase() + word.slice(1).toLowerCase() : word)).join(" ");
}

interface WallTime {
  year: number;
  month: number;
  day: number;
  hour: number;
  minute: number;
  second: number;
}

/** The wall-clock time in `zone` at `instant` (unix ms). */
export function wallTime(zone: string, instant: number): WallTime {
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
  const get = (type: Intl.DateTimeFormatPartTypes): number => Number(parts.find((part) => part.type === type)?.value ?? 0);
  return { year: get("year"), month: get("month"), day: get("day"), hour: get("hour") % 24, minute: get("minute"), second: get("second") };
}

/** How far `zone` is ahead of UTC at `instant`, in minutes. */
export function offsetMinutes(zone: string, instant: number): number {
  const wall = wallTime(zone, instant);
  const asUtc = Date.UTC(wall.year, wall.month - 1, wall.day, wall.hour, wall.minute, wall.second);
  return Math.round((asUtc - Math.floor(instant / 1000) * 1000) / 60_000);
}

/** The instant when `zone`'s clock shows `hour:minute` on the day it shows at `now`. */
export function instantAt(zone: string, now: number, hour: number, minute: number): number {
  const today = wallTime(zone, now);
  const asUtc = Date.UTC(today.year, today.month - 1, today.day, hour, minute);
  let instant = asUtc - offsetMinutes(zone, asUtc) * 60_000;
  // Once more with the offset at that instant (it differs across a daylight-saving change).
  instant = asUtc - offsetMinutes(zone, instant) * 60_000;
  return instant;
}

/** "UTC+9", "UTC−4", "UTC+5:30". */
export function utcOffset(minutes: number): string {
  if (minutes === 0) {
    return "UTC";
  }
  const sign = minutes > 0 ? "+" : "−";
  const abs = Math.abs(minutes);
  const hours = Math.floor(abs / 60);
  const rest = abs % 60;
  return `UTC${sign}${hours}${rest ? `:${String(rest).padStart(2, "0")}` : ""}`;
}

function clock(zone: string, instant: number, locale: string | undefined): string {
  return new Intl.DateTimeFormat(locale, { timeZone: zone, hour: "numeric", minute: "2-digit" }).format(instant);
}

function day(zone: string, instant: number, locale: string | undefined): string {
  return new Intl.DateTimeFormat(locale, { timeZone: zone, weekday: "short", month: "short", day: "numeric" }).format(instant);
}

/** "3pm", "3 pm", "15:30", "9:05am", "noon", "midnight" → [hour, minute]; anything else null. */
export function parseClock(text: string): [number, number] | null {
  const typed = text.trim().toLowerCase();
  if (typed === "noon") {
    return [12, 0];
  }
  if (typed === "midnight") {
    return [0, 0];
  }
  const match = /^(\d{1,2})(?::(\d{2}))?\s*(am|pm)?$/.exec(typed);
  if (!match?.[1]) {
    return null;
  }
  let hour = Number(match[1]);
  const minute = Number(match[2] ?? "0");
  const half = match[3];
  if (minute > 59 || (half ? hour < 1 || hour > 12 : hour > 23) || (!half && !match[2])) {
    // A bare number ("3 london …") is too vague: it needs am/pm or minutes.
    return null;
  }
  if (half === "pm" && hour !== 12) {
    hour += 12;
  } else if (half === "am" && hour === 12) {
    hour = 0;
  }
  return [hour, minute];
}

const NOW_QUERY = /^(?:(?:what\s+)?time\s+(?:is\s+it\s+)?in\s+(.+)|(.+?)\s+time)$/i;
const CONVERT_QUERY = /^(\d{1,2}(?::\d{2})?\s*(?:am|pm)?|noon|midnight)\s+(.+?)\s+(?:to|in)\s+(.+)$/i;

/** `now`: unix ms. `locale`: how times are written (the system's when undefined). */
export function answer(query: string, now: number, locale?: string): Answer | null {
  const typed = query.trim();
  const converting = CONVERT_QUERY.exec(typed);
  if (converting?.[1] && converting[2] && converting[3]) {
    const at = parseClock(converting[1]);
    const from = zoneFor(converting[2]);
    const to = zoneFor(converting[3]);
    if (at && from && to) {
      const instant = instantAt(from, now, at[0], at[1]);
      return {
        title: `${clock(to, instant, locale)} in ${cityName(converting[3])}`,
        subtitle: `${clock(from, instant, locale)} in ${cityName(converting[2])} · ${day(to, instant, locale)} · ${utcOffset(offsetMinutes(to, instant))}`,
        copy: clock(to, instant, locale),
      };
    }
  }
  const asking = NOW_QUERY.exec(typed);
  const city = asking?.[1] ?? asking?.[2];
  const zone = city ? zoneFor(city) : undefined;
  if (!city || !zone) {
    return null;
  }
  return {
    title: `${clock(zone, now, locale)} in ${cityName(city)}`,
    subtitle: `${day(zone, now, locale)} · ${utcOffset(offsetMinutes(zone, now))}`,
    copy: clock(zone, now, locale),
  };
}
