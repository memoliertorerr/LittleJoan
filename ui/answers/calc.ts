// The calculator quick answer (pure, unit-tested): a real parser — a tokenizer and a
// precedence-climbing parser over numbers, + − × ÷ ^ %, parentheses, `pi`, `e` and a few
// functions — never eval or Function. "= <expression>" always asks it; without the "=", a query
// is taken as math only when it has a digit and an operator (or a function) and nothing else.
import type { Answer } from "./types.ts";

type Token =
  | { kind: "number"; value: number }
  | { kind: "name"; value: string }
  | { kind: "op"; value: "+" | "-" | "*" | "/" | "^" | "%" }
  | { kind: "open" }
  | { kind: "close" };

/** The longest expression read. */
export const MAX_LENGTH = 200;

const CONSTANTS: Record<string, number> = { pi: Math.PI, π: Math.PI, e: Math.E, tau: 2 * Math.PI };

const FUNCTIONS: Record<string, (x: number) => number> = {
  sqrt: Math.sqrt,
  cbrt: Math.cbrt,
  abs: Math.abs,
  sin: Math.sin,
  cos: Math.cos,
  tan: Math.tan,
  asin: Math.asin,
  acos: Math.acos,
  atan: Math.atan,
  log: Math.log10,
  ln: Math.log,
  exp: Math.exp,
  round: Math.round,
  floor: Math.floor,
  ceil: Math.ceil,
};

/** Splits `text` into tokens; `null` for any character it doesn't know. */
export function tokenize(text: string): Token[] | null {
  const tokens: Token[] = [];
  let i = 0;
  while (i < text.length) {
    const char = text[i] ?? "";
    if (/\s/.test(char)) {
      i++;
      continue;
    }
    const number = /^(\d+(?:\.\d+)?|\.\d+)(?:e[+-]?\d+)?/i.exec(text.slice(i));
    if (number) {
      tokens.push({ kind: "number", value: Number(number[0]) });
      i += number[0].length;
      continue;
    }
    const name = /^[a-zπ]+/i.exec(text.slice(i));
    if (name) {
      const value = name[0].toLowerCase();
      // "3 x 4": a lone x multiplies.
      tokens.push(value === "x" ? { kind: "op", value: "*" } : { kind: "name", value });
      i += name[0].length;
      continue;
    }
    const op: Record<string, "+" | "-" | "*" | "/" | "^" | "%"> = {
      "+": "+",
      "-": "-",
      "−": "-",
      "*": "*",
      "×": "*",
      "/": "/",
      "÷": "/",
      "^": "^",
      "%": "%",
    };
    const operator = op[char];
    if (operator) {
      tokens.push({ kind: "op", value: operator });
    } else if (char === "(") {
      tokens.push({ kind: "open" });
    } else if (char === ")") {
      tokens.push({ kind: "close" });
    } else {
      return null;
    }
    i++;
  }
  return tokens;
}

class ParseError extends Error {}

/** Precedence climbing: + − < × ÷ % (modulo) < unary − < ^ (right-associative), postfix % (percent). */
class Parser {
  private at = 0;
  private readonly tokens: readonly Token[];

  constructor(tokens: readonly Token[]) {
    this.tokens = tokens;
  }

  parse(): number {
    const value = this.sum();
    if (this.at !== this.tokens.length) {
      throw new ParseError("trailing input");
    }
    return value;
  }

  private peek(offset = 0): Token | undefined {
    return this.tokens[this.at + offset];
  }

  private isOp(value: string, offset = 0): boolean {
    const token = this.peek(offset);
    return token?.kind === "op" && token.value === value;
  }

  /** Whether the token at `offset` can start an operand (so a `%` before it is modulo). */
  private startsOperand(offset: number): boolean {
    const token = this.peek(offset);
    return token?.kind === "number" || token?.kind === "name" || token?.kind === "open" || (token?.kind === "op" && token.value === "-");
  }

  private sum(): number {
    let value = this.product();
    while (this.isOp("+") || this.isOp("-")) {
      const op = this.peek();
      this.at++;
      const right = this.product();
      value = op?.kind === "op" && op.value === "+" ? value + right : value - right;
    }
    return value;
  }

  private product(): number {
    let value = this.unary();
    for (;;) {
      if (this.isOp("*") || this.isOp("/")) {
        const op = this.peek();
        this.at++;
        const right = this.unary();
        if (op?.kind === "op" && op.value === "/") {
          if (right === 0) {
            throw new ParseError("division by zero");
          }
          value /= right;
        } else {
          value *= right;
        }
      } else if (this.isOp("%") && this.startsOperand(1)) {
        this.at++;
        const right = this.unary();
        if (right === 0) {
          throw new ParseError("division by zero");
        }
        value %= right;
      } else if (this.peek()?.kind === "open" || this.peek()?.kind === "name") {
        // Implicit multiplication: 2(3+4), 2pi, 3 sqrt(4).
        value *= this.unary();
      } else {
        return value;
      }
    }
  }

  private unary(): number {
    if (this.isOp("-")) {
      this.at++;
      return -this.unary();
    }
    if (this.isOp("+")) {
      this.at++;
      return this.unary();
    }
    return this.power();
  }

  private power(): number {
    const base = this.postfix();
    if (this.isOp("^")) {
      this.at++;
      // Right-associative, and binds tighter than a unary minus on its left: -2^2 = -4.
      return base ** this.unary();
    }
    return base;
  }

  private postfix(): number {
    let value = this.primary();
    while (this.isOp("%") && !this.startsOperand(1)) {
      this.at++;
      value /= 100;
    }
    return value;
  }

  private primary(): number {
    const token = this.peek();
    if (!token) {
      throw new ParseError("missing operand");
    }
    this.at++;
    switch (token.kind) {
      case "number":
        return token.value;
      case "open": {
        const value = this.sum();
        if (this.peek()?.kind !== "close") {
          throw new ParseError("missing )");
        }
        this.at++;
        return value;
      }
      case "name": {
        const fn = FUNCTIONS[token.value];
        if (fn) {
          return fn(this.postfix());
        }
        const constant = CONSTANTS[token.value];
        if (constant !== undefined) {
          return constant;
        }
        throw new ParseError(`unknown name ${token.value}`);
      }
      default:
        throw new ParseError("unexpected token");
    }
  }
}

/** The value of `expression`, or `null` when it doesn't parse or has no finite value. */
export function evaluate(expression: string): number | null {
  const tokens = tokenize(expression);
  if (!tokens || tokens.length === 0) {
    return null;
  }
  try {
    const value = new Parser(tokens).parse();
    return Number.isFinite(value) ? value : null;
  } catch (error) {
    if (error instanceof ParseError) {
      return null;
    }
    throw error;
  }
}

/** A result as people write it: 12 significant digits, no float noise (0.1 + 0.2 = 0.3). */
export function formatNumber(value: number): string {
  const rounded = Number(value.toPrecision(12));
  if (Object.is(rounded, -0)) {
    return "0";
  }
  const abs = Math.abs(rounded);
  if (abs !== 0 && (abs >= 1e15 || abs < 1e-9)) {
    return rounded.toExponential().replace(/\.?0+e/, "e");
  }
  return String(rounded);
}

/** Whether a query without "=" looks like math: a digit, and an operator or a function — but not a date. */
function looksLikeMath(text: string, tokens: readonly Token[]): boolean {
  return (
    /\d/.test(text) &&
    !/^\d{4}-\d{1,2}-\d{1,2}$/.test(text) &&
    tokens.some((t) => (t.kind === "op" && !(t.value === "-" && tokens.indexOf(t) === 0)) || (t.kind === "name" && t.value in FUNCTIONS))
  );
}

export function answer(query: string): Answer | null {
  const trimmed = query.trim();
  if (trimmed.length === 0 || trimmed.length > MAX_LENGTH) {
    return null;
  }
  const asked = trimmed.startsWith("=");
  const expression = asked ? trimmed.slice(1).trim() : trimmed;
  const tokens = tokenize(expression);
  if (!tokens || tokens.length === 0 || (!asked && !looksLikeMath(expression, tokens))) {
    return null;
  }
  const value = evaluate(expression);
  if (value === null) {
    return null;
  }
  const result = formatNumber(value);
  return { title: result, subtitle: `= ${expression}`, copy: result };
}
