// @ts-check
// Text tools, pure: counts, case changes, Base64 and URL encoding (UTF-8), and line operations.
// Each transform takes the text and gives the new text (or throws a TextToolError the panel
// shows). ui.js applies them to the selection, or to everything when nothing is selected.

export class TextToolError extends Error {}

/**
 * Words in a text (letters and digits in any script; apostrophes inside words count with them).
 * @param {string} text
 */
function words(text) {
  return text.match(/[\p{L}\p{N}]+(?:['’][\p{L}\p{N}]+)*/gu) ?? [];
}

/**
 * Characters (as people count them: one per code point, emoji included), words, lines, bytes in
 * UTF-8, sentences and the reading time at 230 words a minute.
 * @param {string} text
 */
export function counts(text) {
  const wordCount = words(text).length;
  return {
    characters: [...text].length,
    charactersNoSpaces: [...text.replace(/\s/g, "")].length,
    words: wordCount,
    lines: text === "" ? 0 : text.split(/\r\n|\r|\n/).length,
    bytes: new TextEncoder().encode(text).length,
    sentences: (text.match(/[^.!?…]*[\p{L}\p{N}][^.!?…]*(?:[.!?…]+|$)/gu) ?? []).length,
    readingMinutes: wordCount === 0 ? 0 : Math.max(1, Math.round(wordCount / 230)),
  };
}

/** @param {string} word */
const capital = (word) => (word ? word.charAt(0).toLocaleUpperCase() + word.slice(1).toLocaleLowerCase() : word);

/** Small words Title Case leaves lower case (unless first or last). */
const SMALL = new Set(["a", "an", "and", "as", "at", "but", "by", "for", "in", "nor", "of", "on", "or", "per", "the", "to", "vs", "via"]);

/** @type {Record<string, (text: string) => string>} */
export const CASES = {
  upper: (text) => text.toLocaleUpperCase(),
  lower: (text) => text.toLocaleLowerCase(),
  title: (text) =>
    text.replace(/[\p{L}\p{N}]+(?:['’][\p{L}\p{N}]+)*/gu, (/** @type {string} */ word, /** @type {number} */ offset) => {
      const first = offset === 0 || /[.:!?]\s*$/.test(text.slice(0, offset));
      const last = !/[\p{L}\p{N}]/u.test(text.slice(offset + word.length));
      return !first && !last && SMALL.has(word.toLowerCase()) ? word.toLowerCase() : capital(word);
    }),
  sentence: (text) =>
    text.toLocaleLowerCase().replace(/(^\s*|[.!?…]\s+)(\p{L})/gu, (_match, before, letter) => `${String(before)}${String(letter).toLocaleUpperCase()}`),
  camel: (text) =>
    splitWords(text)
      .map((word, i) => (i === 0 ? word.toLocaleLowerCase() : capital(word)))
      .join(""),
  pascal: (text) => splitWords(text).map(capital).join(""),
  snake: (text) => splitWords(text).map((w) => w.toLocaleLowerCase()).join("_"),
  kebab: (text) => splitWords(text).map((w) => w.toLocaleLowerCase()).join("-"),
  constant: (text) => splitWords(text).map((w) => w.toLocaleUpperCase()).join("_"),
};

/**
 * Words for identifier cases: split at spaces, punctuation and lower→upper changes
 * ("parseHTTPResponse" → parse, HTTP, Response).
 * @param {string} text
 */
export function splitWords(text) {
  return text
    .replace(/(\p{Ll}|\p{N})(\p{Lu})/gu, "$1 $2")
    .replace(/(\p{Lu}+)(\p{Lu}\p{Ll})/gu, "$1 $2")
    .split(/[^\p{L}\p{N}]+/u)
    .filter(Boolean);
}

/**
 * Base64 of the text's UTF-8 bytes.
 * @param {string} text
 */
export function base64Encode(text) {
  const bytes = new TextEncoder().encode(text);
  let binary = "";
  for (let i = 0; i < bytes.length; i += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(binary);
}

/**
 * The text whose UTF-8 bytes the Base64 (standard or URL-safe, padding optional, whitespace
 * ignored) holds.
 * @param {string} text
 */
export function base64Decode(text) {
  const clean = text.replace(/\s+/g, "").replace(/-/g, "+").replace(/_/g, "/");
  if (!/^[A-Za-z0-9+/]*={0,2}$/.test(clean) || clean.replace(/=+$/, "").length % 4 === 1) {
    throw new TextToolError("That isn't Base64.");
  }
  const padded = clean.replace(/=+$/, "").padEnd(Math.ceil(clean.replace(/=+$/, "").length / 4) * 4, "=");
  const binary = atob(padded);
  const bytes = Uint8Array.from(binary, (c) => c.charCodeAt(0));
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    throw new TextToolError("That Base64 isn't text (its bytes aren't UTF-8).");
  }
}

/**
 * Percent-encoding for a URL component (everything but letters, digits and `-_.!~*'()`).
 * @param {string} text
 */
export function urlEncode(text) {
  return encodeURIComponent(text);
}

/**
 * Percent-decoding (and `+` as a space, as forms send it).
 * @param {string} text
 */
export function urlDecode(text) {
  try {
    return decodeURIComponent(text.replace(/\+/g, " "));
  } catch {
    throw new TextToolError("That has a % that isn't followed by two hex digits.");
  }
}

/** @param {string} text */
const splitLines = (text) => text.split(/\r\n|\r|\n/);
const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

/** @type {Record<string, (text: string) => string>} */
export const LINES = {
  sort: (text) => splitLines(text).sort((a, b) => collator.compare(a, b)).join("\n"),
  reverseSort: (text) => splitLines(text).sort((a, b) => collator.compare(b, a)).join("\n"),
  reverse: (text) => splitLines(text).reverse().join("\n"),
  unique: (text) => [...new Set(splitLines(text))].join("\n"),
  trim: (text) => splitLines(text).map((line) => line.trim()).join("\n"),
  removeEmpty: (text) => splitLines(text).filter((line) => line.trim() !== "").join("\n"),
};

/** @type {Record<string, (text: string) => string>} */
export const TRANSFORMS = {
  ...Object.fromEntries(Object.entries(CASES).map(([name, run]) => [`case.${name}`, run])),
  ...Object.fromEntries(Object.entries(LINES).map(([name, run]) => [`lines.${name}`, run])),
  "base64.encode": base64Encode,
  "base64.decode": base64Decode,
  "url.encode": urlEncode,
  "url.decode": urlDecode,
};

/**
 * Applies the transform `name` to the part of `text` from `start` to `end` (all of it when
 * nothing is selected); returns the new text and the new selection.
 * @param {string} name
 * @param {string} text
 * @param {number} start
 * @param {number} end
 */
export function apply(name, text, start, end) {
  const run = TRANSFORMS[name];
  if (!run) {
    throw new TextToolError(`No such tool: ${name}`);
  }
  const [from, to] = start === end ? [0, text.length] : [Math.min(start, end), Math.max(start, end)];
  const changed = run(text.slice(from, to));
  return { text: text.slice(0, from) + changed + text.slice(to), start: from, end: from + changed.length };
}
