// @ts-check
// Snake's rules, pure: no DOM, no clock, no randomness of its own (the caller passes it in), so
// logic.test.ts can play whole games. ui.js draws the game and feeds it keys and ticks.

/** @typedef {"up" | "down" | "left" | "right"} Direction */
/** @typedef {{ x: number, y: number }} Cell */
/** @typedef {"ready" | "running" | "paused" | "over" | "won"} Status */
/**
 * @typedef {object} Game
 * @property {number} cols
 * @property {number} rows
 * @property {Cell[]} snake the head first
 * @property {Direction} heading the way the snake last moved
 * @property {Direction[]} turns turns pressed but not yet taken (at most `MAX_TURNS`)
 * @property {Cell | null} food null once the snake fills the board
 * @property {number} score food eaten
 * @property {Status} status
 */
/** @typedef {(n: number) => number} RandomInt an integer in [0, n) */
/** @typedef {Direction | "pause" | "start" | "escape"} Action */

export const COLS = 18;
export const ROWS = 18;
export const START_LENGTH = 3;
/** Milliseconds per step: `BASE_TICK` at first, `SPEEDUP` less per food, never under `MIN_TICK`. */
export const BASE_TICK = 140;
export const MIN_TICK = 60;
export const SPEEDUP = 4;
/** Turns that can wait for the next steps, so two quick presses (a U-turn) both count. */
export const MAX_TURNS = 2;

/** @type {Record<Direction, Cell>} */
const DELTAS = { up: { x: 0, y: -1 }, down: { x: 0, y: 1 }, left: { x: -1, y: 0 }, right: { x: 1, y: 0 } };
/** @type {Record<Direction, Direction>} */
const OPPOSITE = { up: "down", down: "up", left: "right", right: "left" };

/**
 * @param {Cell} a
 * @param {Cell} b
 */
const same = (a, b) => a.x === b.x && a.y === b.y;

/**
 * A random free cell of the board, or null when the snake covers all of it.
 * @param {number} cols
 * @param {number} rows
 * @param {Cell[]} snake
 * @param {RandomInt} random
 * @returns {Cell | null}
 */
export function placeFood(cols, rows, snake, random) {
  const taken = new Set(snake.map((c) => c.y * cols + c.x));
  const free = cols * rows - taken.size;
  if (free <= 0) {
    return null;
  }
  let pick = random(free);
  for (let index = 0; index < cols * rows; index += 1) {
    if (!taken.has(index)) {
      if (pick === 0) {
        return { x: index % cols, y: Math.floor(index / cols) };
      }
      pick -= 1;
    }
  }
  return null;
}

/**
 * A new game: a short snake in the middle, heading right, waiting for the first key.
 * @param {RandomInt} random
 * @param {number} [cols]
 * @param {number} [rows]
 * @returns {Game}
 */
export function newGame(random, cols = COLS, rows = ROWS) {
  const y = Math.floor(rows / 2);
  const head = Math.floor(cols / 2);
  /** @type {Cell[]} */
  const snake = [];
  for (let i = 0; i < START_LENGTH; i += 1) {
    snake.push({ x: head - i, y });
  }
  return { cols, rows, snake, heading: "right", turns: [], food: placeFood(cols, rows, snake, random), score: 0, status: "ready" };
}

/**
 * The player pressed `direction`. It waits for the next step (at most `MAX_TURNS` wait); a turn
 * back onto itself (180°) or onto the way it already goes is ignored. A turn starts a game that
 * is ready and resumes one that is paused.
 * @param {Game} game
 * @param {Direction} direction
 * @returns {Game}
 */
export function turn(game, direction) {
  if (game.status === "over" || game.status === "won") {
    return game;
  }
  const status = "running";
  const last = game.turns.at(-1) ?? game.heading;
  if (direction === last || direction === OPPOSITE[last] || game.turns.length >= MAX_TURNS) {
    return game.status === status ? game : { ...game, status };
  }
  return { ...game, turns: [...game.turns, direction], status };
}

/**
 * Space: pause a running game, resume a paused one, start one that is ready.
 * @param {Game} game
 * @returns {Game}
 */
export function togglePause(game) {
  switch (game.status) {
    case "running":
      return { ...game, status: "paused" };
    case "paused":
    case "ready":
      return { ...game, status: "running" };
    default:
      return game;
  }
}

/**
 * One step of a running game: the snake moves one cell (taking the next waiting turn), grows when
 * it eats, and the game is over when its head leaves the board or meets its body. The cell its
 * tail leaves in this step is free (unless it grows).
 * @param {Game} game
 * @param {RandomInt} random
 * @returns {Game}
 */
export function step(game, random) {
  if (game.status !== "running") {
    return game;
  }
  const [next, ...turns] = game.turns;
  const heading = next ?? game.heading;
  const delta = DELTAS[heading];
  const current = game.snake[0];
  if (!current) {
    return { ...game, status: "over" };
  }
  const head = { x: current.x + delta.x, y: current.y + delta.y };
  if (head.x < 0 || head.y < 0 || head.x >= game.cols || head.y >= game.rows) {
    return { ...game, heading, turns, status: "over" };
  }
  const eats = game.food !== null && same(head, game.food);
  const body = eats ? game.snake : game.snake.slice(0, -1);
  if (body.some((cell) => same(cell, head))) {
    return { ...game, heading, turns, status: "over" };
  }
  const snake = [head, ...body];
  if (!eats) {
    return { ...game, snake, heading, turns };
  }
  const food = placeFood(game.cols, game.rows, snake, random);
  return { ...game, snake, heading, turns, food, score: game.score + 1, status: food ? "running" : "won" };
}

/**
 * Milliseconds between steps for a score: faster with every food, down to `MIN_TICK`.
 * @param {number} score
 */
export function tickMs(score) {
  return Math.max(MIN_TICK, BASE_TICK - SPEEDUP * score);
}

/** @type {Record<string, Action>} */
const KEYS = {
  ArrowUp: "up",
  ArrowDown: "down",
  ArrowLeft: "left",
  ArrowRight: "right",
  " ": "pause",
  Spacebar: "pause",
  Enter: "start",
  Escape: "escape",
};
/** WASD by position (`KeyboardEvent.code`), so they sit in the same place on every layout. */
/** @type {Record<string, Direction>} */
const CODES = { KeyW: "up", KeyA: "left", KeyS: "down", KeyD: "right" };

/**
 * What a key press means: arrows and WASD steer, Space pauses, Enter starts, Escape leaves.
 * Presses with Ctrl, Alt or Cmd belong to Little Joan and the system (null).
 * @param {{ key: string, code?: string, ctrlKey?: boolean, altKey?: boolean, metaKey?: boolean }} event
 * @returns {Action | null}
 */
export function keyAction(event) {
  if (event.ctrlKey || event.altKey || event.metaKey) {
    return null;
  }
  return KEYS[event.key] ?? (event.code ? CODES[event.code] : undefined) ?? null;
}

/**
 * A new best score, or the old one.
 * @param {unknown} best what storage held (anything)
 * @param {number} score
 */
export function bestScore(best, score) {
  const old = typeof best === "number" && Number.isFinite(best) && best > 0 ? Math.floor(best) : 0;
  return Math.max(old, score);
}

/**
 * `#rrggbb` → [r, g, b] (0–255), or null.
 * @param {string} hex
 * @returns {[number, number, number] | null}
 */
function rgb(hex) {
  const match = /^#([0-9a-f]{6})$/i.exec(hex.trim());
  if (!match?.[1]) {
    return null;
  }
  const value = parseInt(match[1], 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255];
}

/**
 * `a` mixed with `b`: `t` = 0 is `a`, 1 is `b` (in sRGB; for the body fading toward the
 * background). `a` when either isn't `#rrggbb`.
 * @param {string} a
 * @param {string} b
 * @param {number} t
 */
export function mix(a, b, t) {
  const from = rgb(a);
  const to = rgb(b);
  if (!from || !to) {
    return a;
  }
  const part = Math.min(1, Math.max(0, t));
  return `#${from.map((v, i) => Math.round(v + ((to[i] ?? v) - v) * part).toString(16).padStart(2, "0")).join("")}`;
}

/**
 * The food's color: the theme's accent turned halfway around the color wheel, a little more
 * saturated (the same lightness, so it is as visible as the snake on the theme's background).
 * @param {string} hex `#rrggbb`
 * @param {string} fallback used when `hex` isn't one
 */
export function complementOf(hex, fallback = "#c8743a") {
  const channels = rgb(hex);
  if (!channels) {
    return fallback;
  }
  const [r, g, b] = /** @type {[number, number, number]} */ (channels.map((v) => v / 255));
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const light = (max + min) / 2;
  const chroma = max - min;
  const sat = chroma === 0 ? 0 : Math.min(0.85, (1.6 * chroma) / (1 - Math.abs(2 * light - 1)));
  let hue = 0;
  if (chroma !== 0) {
    if (max === r) {
      hue = ((g - b) / chroma) % 6;
    } else if (max === g) {
      hue = (b - r) / chroma + 2;
    } else {
      hue = (r - g) / chroma + 4;
    }
  }
  const turned = ((hue * 60 + 180) % 360 + 360) % 360;
  // Back from HSL.
  const c = (1 - Math.abs(2 * light - 1)) * sat;
  const x = c * (1 - Math.abs(((turned / 60) % 2) - 1));
  const m = light - c / 2;
  const sector = Math.floor(turned / 60);
  const [r1, g1, b1] = [
    [c, x, 0],
    [x, c, 0],
    [0, c, x],
    [0, x, c],
    [x, 0, c],
    [c, 0, x],
  ][sector] ?? [0, 0, 0];
  const channel = (/** @type {number} */ v) =>
    Math.round((v + m) * 255)
      .toString(16)
      .padStart(2, "0");
  return `#${channel(r1 ?? 0)}${channel(g1 ?? 0)}${channel(b1 ?? 0)}`;
}
