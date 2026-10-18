// Random quick answers (pure, unit-tested; the randomness is passed in): "uuid", "password 20",
// "roll 2d6", "flip a coin", "random 1-100". In the bar the randomness comes from the web crypto
// API (`cryptoInt`), never Math.random: passwords and UUIDs must be unguessable.
import type { Answer } from "./types.ts";

/** A uniform random integer in [0, n). */
export type RandomInt = (n: number) => number;

/** The web crypto API's randomness, without modulo bias (rejection sampling). */
export const cryptoInt: RandomInt = (n) => {
  if (!Number.isInteger(n) || n <= 0 || n > 2 ** 32) {
    throw new RangeError(`can't draw from ${n}`);
  }
  const limit = 2 ** 32 - (2 ** 32 % n);
  const buffer = new Uint32Array(1);
  for (;;) {
    crypto.getRandomValues(buffer);
    const value = buffer[0] ?? 0;
    if (value < limit) {
      return value % n;
    }
  }
};

/** A version 4 UUID from `random`. */
export function uuid(random: RandomInt): string {
  const bytes = Array.from({ length: 16 }, () => random(256));
  bytes[6] = ((bytes[6] ?? 0) & 0x0f) | 0x40;
  bytes[8] = ((bytes[8] ?? 0) & 0x3f) | 0x80;
  const hex = bytes.map((b) => b.toString(16).padStart(2, "0")).join("");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

const LOWER = "abcdefghijkmnopqrstuvwxyz";
const UPPER = "ABCDEFGHJKLMNPQRSTUVWXYZ";
const DIGITS = "23456789";
const SYMBOLS = "!@#$%^&*-_=+?";

/** Password lengths allowed (16 when none is given). */
export const PASSWORD_LENGTHS = { min: 8, max: 128, default: 16 } as const;

/**
 * A password of `length` characters: letters, digits and symbols, without look-alikes (l, I, O,
 * 0, 1), with at least one of each kind.
 */
export function password(length: number, random: RandomInt): string {
  const all = LOWER + UPPER + DIGITS + SYMBOLS;
  const pick = (set: string): string => set.charAt(random(set.length));
  const chars = [pick(LOWER), pick(UPPER), pick(DIGITS), pick(SYMBOLS)];
  while (chars.length < length) {
    chars.push(pick(all));
  }
  // Shuffle (Fisher–Yates), so the guaranteed kinds aren't always first.
  for (let i = chars.length - 1; i > 0; i--) {
    const j = random(i + 1);
    [chars[i], chars[j]] = [chars[j] ?? "", chars[i] ?? ""];
  }
  return chars.join("");
}

export function answer(query: string, random: RandomInt = cryptoInt): Answer | null {
  const typed = query.trim().toLowerCase().replace(/\s+/g, " ");
  if (/^(uuid|guid|uuid v4|uuid4)$/.test(typed)) {
    const id = uuid(random);
    return { title: id, subtitle: "A random UUID (v4)", copy: id };
  }
  const pass = /^(?:password|pass|pw)(?: (\d{1,3}))?$/.exec(typed);
  if (pass) {
    const length = pass[1] ? Number(pass[1]) : PASSWORD_LENGTHS.default;
    if (length < PASSWORD_LENGTHS.min || length > PASSWORD_LENGTHS.max) {
      return null;
    }
    const made = password(length, random);
    return { title: made, subtitle: `A random password, ${length} characters`, copy: made };
  }
  const roll = /^(?:roll )?(\d{0,2})d(\d{1,4})$/.exec(typed);
  if (roll?.[2] && (typed.startsWith("roll") || roll[1])) {
    const count = roll[1] ? Number(roll[1]) : 1;
    const sides = Number(roll[2]);
    if (count < 1 || count > 20 || sides < 2) {
      return null;
    }
    const dice = Array.from({ length: count }, () => random(sides) + 1);
    const total = dice.reduce((sum, die) => sum + die, 0);
    return {
      title: String(total),
      subtitle: count > 1 ? `${count}d${sides}: ${dice.join(" + ")}` : `1d${sides}`,
      copy: String(total),
    };
  }
  if (/^(flip a coin|flip coin|coin flip|flip|toss a coin|heads or tails)$/.test(typed)) {
    const side = random(2) === 0 ? "Heads" : "Tails";
    return { title: side, subtitle: "A coin flip", copy: side };
  }
  const range = /^(?:random|rand|pick)(?: (-?\d{1,15}) ?(?:-|to|\.\.) ?(-?\d{1,15}))?$/.exec(typed);
  if (range) {
    const low = range[1] ? Number(range[1]) : 1;
    const high = range[2] ? Number(range[2]) : 100;
    const [min, max] = low <= high ? [low, high] : [high, low];
    const span = max - min + 1;
    if (span > 2 ** 32) {
      return null;
    }
    const value = String(min + random(span));
    return { title: value, subtitle: `A random number from ${min} to ${max}`, copy: value };
  }
  return null;
}
