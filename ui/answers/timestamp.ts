// The Unix-timestamp quick answer (pure, unit-tested; the clock is passed in): a 10-digit number
// is seconds, a 13-digit one milliseconds ("1700000000" → its date and time), and "now unix" (or
// "unix now", "timestamp") gives the current one.
import type { Answer } from "./types.ts";

/** "2023-11-14 22:13:20 UTC". */
export function utcText(ms: number): string {
  return `${new Date(ms).toISOString().slice(0, 19).replace("T", " ")} UTC`;
}

export function answer(query: string, now: number): Answer | null {
  const typed = query.trim().toLowerCase().replace(/\s+/g, " ");
  if (/^(now unix|unix now|unix time|unix timestamp|timestamp|now timestamp|epoch|now epoch)$/.test(typed)) {
    const seconds = String(Math.floor(now / 1000));
    return { title: seconds, subtitle: `Unix time now · ${utcText(now)}`, copy: seconds };
  }
  const match = /^(\d{10}|\d{13})$/.exec(typed);
  if (!match?.[1]) {
    return null;
  }
  const millis = match[1].length === 13;
  const ms = millis ? Number(match[1]) : Number(match[1]) * 1000;
  if (!Number.isFinite(ms) || Number.isNaN(new Date(ms).getTime())) {
    return null;
  }
  const iso = new Date(ms).toISOString();
  return {
    title: utcText(ms),
    subtitle: `Unix time in ${millis ? "milliseconds" : "seconds"} · ${iso}`,
    copy: iso,
  };
}
