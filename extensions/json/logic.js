// @ts-check
// The JSON tool's rules, pure: a JSON parser that says where an error is (line and column, the
// same on every engine — JSON.parse's messages differ), and printers that format or minify
// without changing a value: numbers and strings keep their exact spelling (no rounding of big
// numbers, escapes left as written). ui.js is the panel.

/**
 * @typedef {{ type: "object", entries: { key: string, value: Node }[] }
 *   | { type: "array", items: Node[] }
 *   | { type: "string", raw: string }
 *   | { type: "number", raw: string }
 *   | { type: "literal", raw: "true" | "false" | "null" }} Node
 * Object keys are kept as written (`key` is the raw string with its quotes).
 */
/** @typedef {{ message: string, offset: number, line: number, column: number }} ParseError */
/** @typedef {{ ok: true, value: Node } | { ok: false, error: ParseError }} Parsed */
/** @typedef {"key" | "string" | "number" | "literal" | "punctuation" | "space"} TokenKind */
/** @typedef {[TokenKind, string]} Token */

/** Deeper nesting is refused (and keeps the parser's recursion bounded). */
export const MAX_DEPTH = 256;

class JsonError extends Error {
  /**
   * @param {string} message
   * @param {number} offset
   */
  constructor(message, offset) {
    super(message);
    this.offset = offset;
  }
}

/**
 * Line and column (1-based) of an offset in `text`.
 * @param {string} text
 * @param {number} offset
 */
export function position(text, offset) {
  const before = text.slice(0, offset);
  const line = before.split("\n").length;
  const column = offset - (before.lastIndexOf("\n") + 1) + 1;
  return { line, column };
}

/**
 * A character for a message: `'x'`, or a name for what can't be seen.
 * @param {string | undefined} c
 */
function describe(c) {
  if (c === undefined) {
    return "the end";
  }
  if (c === "\n") {
    return "a line break";
  }
  if (/\s/.test(c)) {
    return "a space";
  }
  return `'${c}'`;
}

/**
 * Parses JSON (RFC 8259; a leading byte-order mark is allowed). On error, says what and where.
 * @param {string} text
 * @returns {Parsed}
 */
export function parse(text) {
  let i = text.charCodeAt(0) === 0xfeff ? 1 : 0;
  const skip = () => {
    while (i < text.length && " \t\n\r".includes(text[i] ?? "")) {
      i += 1;
    }
  };
  /** @param {number} depth @returns {Node} */
  const value = (depth) => {
    skip();
    const c = text[i];
    if (c === undefined) {
      throw new JsonError("The JSON ends where a value should be", i);
    }
    if (depth > MAX_DEPTH) {
      throw new JsonError(`Nested deeper than ${MAX_DEPTH} levels`, i);
    }
    if (c === "{") {
      return object(depth);
    }
    if (c === "[") {
      return array(depth);
    }
    if (c === '"') {
      return { type: "string", raw: string() };
    }
    if (c === "-" || (c >= "0" && c <= "9")) {
      return { type: "number", raw: number() };
    }
    for (const word of /** @type {const} */ (["true", "false", "null"])) {
      if (text.startsWith(word, i)) {
        i += word.length;
        return { type: "literal", raw: word };
      }
    }
    if (c === "'") {
      throw new JsonError("Strings need double quotes (\"), not single ones", i);
    }
    throw new JsonError(`Expected a value, found ${describe(c)}`, i);
  };
  /** @param {number} depth @returns {Node} */
  const object = (depth) => {
    i += 1;
    /** @type {{ key: string, value: Node }[]} */
    const entries = [];
    skip();
    if (text[i] === "}") {
      i += 1;
      return { type: "object", entries };
    }
    for (;;) {
      skip();
      if (text[i] !== '"') {
        throw new JsonError(text[i] === "}" ? "A comma before '}' (JSON has no trailing commas)" : `Expected a key in double quotes, found ${describe(text[i])}`, i);
      }
      const key = string();
      skip();
      if (text[i] !== ":") {
        throw new JsonError(`Expected ':' after the key, found ${describe(text[i])}`, i);
      }
      i += 1;
      entries.push({ key, value: value(depth + 1) });
      skip();
      if (text[i] === ",") {
        i += 1;
        continue;
      }
      if (text[i] === "}") {
        i += 1;
        return { type: "object", entries };
      }
      throw new JsonError(`Expected ',' or '}' after a value, found ${describe(text[i])}`, i);
    }
  };
  /** @param {number} depth @returns {Node} */
  const array = (depth) => {
    i += 1;
    /** @type {Node[]} */
    const items = [];
    skip();
    if (text[i] === "]") {
      i += 1;
      return { type: "array", items };
    }
    for (;;) {
      skip();
      if (text[i] === "]") {
        throw new JsonError("A comma before ']' (JSON has no trailing commas)", i);
      }
      items.push(value(depth + 1));
      skip();
      if (text[i] === ",") {
        i += 1;
        continue;
      }
      if (text[i] === "]") {
        i += 1;
        return { type: "array", items };
      }
      throw new JsonError(`Expected ',' or ']' after an item, found ${describe(text[i])}`, i);
    }
  };
  /** @returns {string} the raw string, quotes included */
  const string = () => {
    const start = i;
    i += 1;
    for (;;) {
      const c = text[i];
      if (c === undefined) {
        throw new JsonError("A string that never ends (no closing \")", start);
      }
      if (c === '"') {
        i += 1;
        return text.slice(start, i);
      }
      if (c === "\\") {
        const next = text[i + 1];
        if (next === "u") {
          if (!/^[0-9a-fA-F]{4}$/.test(text.slice(i + 2, i + 6))) {
            throw new JsonError("\\u needs four hex digits", i);
          }
          i += 6;
          continue;
        }
        if (next === undefined || !'"\\/bfnrt'.includes(next)) {
          throw new JsonError(`Unknown escape \\${next ?? ""} in a string`, i);
        }
        i += 2;
        continue;
      }
      if (c.charCodeAt(0) < 0x20) {
        throw new JsonError(c === "\n" ? "A line break inside a string (write \\n)" : "A control character inside a string", i);
      }
      i += 1;
    }
  };
  const number = () => {
    const match = /^-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?/.exec(text.slice(i));
    if (!match || match[0] === "-") {
      throw new JsonError("A number that isn't one", i);
    }
    const after = text[i + match[0].length];
    if (after !== undefined && /[0-9.eE]/.test(after)) {
      throw new JsonError(match[1] === "0" && /\d/.test(after) ? "Numbers can't start with 0" : "A number that isn't one", i);
    }
    i += match[0].length;
    return match[0];
  };
  try {
    const result = value(0);
    skip();
    if (i < text.length) {
      throw new JsonError(`More after the JSON value: ${describe(text[i])}`, i);
    }
    return { ok: true, value: result };
  } catch (error) {
    if (error instanceof JsonError) {
      const at = Math.min(error.offset, text.length);
      return { ok: false, error: { message: error.message, offset: at, ...position(text, at) } };
    }
    throw error;
  }
}

/**
 * The value printed as tokens (for highlighting): `indent` = "" prints it on one line with no
 * spaces (minified); otherwise one item per line, nested by `indent`.
 * @param {Node} node
 * @param {string} indent
 * @returns {Token[]}
 */
export function tokens(node, indent) {
  /** @type {Token[]} */
  const out = [];
  const line = (/** @type {number} */ level) => {
    if (indent) {
      out.push(["space", `\n${indent.repeat(level)}`]);
    }
  };
  /** @param {Node} n @param {number} level */
  const walk = (n, level) => {
    switch (n.type) {
      case "object":
        if (n.entries.length === 0) {
          out.push(["punctuation", "{}"]);
          return;
        }
        out.push(["punctuation", "{"]);
        n.entries.forEach((entry, index) => {
          if (index > 0) {
            out.push(["punctuation", ","]);
          }
          line(level + 1);
          out.push(["key", entry.key], ["punctuation", ":"]);
          if (indent) {
            out.push(["space", " "]);
          }
          walk(entry.value, level + 1);
        });
        line(level);
        out.push(["punctuation", "}"]);
        return;
      case "array":
        if (n.items.length === 0) {
          out.push(["punctuation", "[]"]);
          return;
        }
        out.push(["punctuation", "["]);
        n.items.forEach((item, index) => {
          if (index > 0) {
            out.push(["punctuation", ","]);
          }
          line(level + 1);
          walk(item, level + 1);
        });
        line(level);
        out.push(["punctuation", "]"]);
        return;
      default:
        out.push([n.type, n.raw]);
    }
  };
  walk(node, 0);
  return out;
}

/**
 * Pretty-printed JSON.
 * @param {Node} node
 * @param {string} [indent] two spaces by default
 */
export function format(node, indent = "  ") {
  return tokens(node, indent)
    .map(([, text]) => text)
    .join("");
}

/**
 * JSON on one line, without spaces.
 * @param {Node} node
 */
export function minify(node) {
  return format(node, "");
}

/**
 * The value with every object's keys sorted (by the key's text; arrays keep their order).
 * @param {Node} node
 * @returns {Node}
 */
export function sortKeys(node) {
  if (node.type === "array") {
    return { type: "array", items: node.items.map(sortKeys) };
  }
  if (node.type !== "object") {
    return node;
  }
  const text = (/** @type {string} */ raw) => {
    try {
      return String(JSON.parse(raw));
    } catch {
      return raw;
    }
  };
  const entries = node.entries.map((e) => ({ key: e.key, value: sortKeys(e.value) }));
  entries.sort((a, b) => (text(a.key) < text(b.key) ? -1 : text(a.key) > text(b.key) ? 1 : 0));
  return { type: "object", entries };
}

/**
 * What the value holds: objects, arrays, other values, and how deep it nests.
 * @param {Node} node
 */
export function stats(node) {
  const result = { objects: 0, arrays: 0, values: 0, depth: 0 };
  /** @param {Node} n @param {number} level */
  const walk = (n, level) => {
    result.depth = Math.max(result.depth, level);
    if (n.type === "object") {
      result.objects += 1;
      n.entries.forEach((e) => walk(e.value, level + 1));
    } else if (n.type === "array") {
      result.arrays += 1;
      n.items.forEach((item) => walk(item, level + 1));
    } else {
      result.values += 1;
    }
  };
  walk(node, 0);
  return result;
}

/**
 * The line an error is on and a caret under its column, for the panel.
 * @param {string} text
 * @param {ParseError} error
 */
export function excerpt(text, error) {
  const lineText = (text.split("\n")[error.line - 1] ?? "").replace(/\t/g, " ");
  const start = Math.max(0, error.column - 40);
  const shown = lineText.slice(start, start + 80);
  return `${shown}\n${" ".repeat(error.column - 1 - start)}^`;
}

/** @typedef {"two" | "four" | "tab" | "minify"} Style */
/** @type {Record<Style, string>} */
export const INDENTS = { two: "  ", four: "    ", tab: "\t", minify: "" };
