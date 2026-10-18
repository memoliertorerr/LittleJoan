// @ts-check
// 2048's rules, pure: no DOM, no randomness of its own (the caller passes it in), so
// logic.test.ts can check every move. Tiles keep an id while they slide, so ui.js can animate
// them; the tile a merge keeps is the one nearer the edge the move goes to.

/** @typedef {"up" | "down" | "left" | "right"} Direction */
/** @typedef {{ id: number, value: number }} Tile */
/**
 * @typedef {object} Game
 * @property {number} size the board is size × size
 * @property {(Tile | null)[]} cells row by row
 * @property {number} score the sum of every merge's new tile
 * @property {number} nextId
 * @property {boolean} won a tile reached `GOAL` (the player may keep playing)
 * @property {boolean} over no move is left
 */
/**
 * @typedef {object} Move
 * @property {Game} game
 * @property {boolean} moved whether any tile moved (only then does a new one appear)
 * @property {number} gained points this move
 * @property {{ id: number, into: number }[]} absorbed tiles merged into another (`into`), gone now
 * @property {number[]} merged ids of the tiles that doubled
 * @property {number | null} spawned the new tile's id
 */
/** @typedef {(n: number) => number} RandomInt an integer in [0, n) */
/** @typedef {Direction | "new" | "continue" | "escape"} Action */

export const SIZE = 4;
export const GOAL = 2048;
/** Tiles above this aren't believed when a saved game is read (2^17 is the largest possible). */
const MAX_TILE = 2 ** 17;

/**
 * The cell indices of each line in the order a move packs them: the cell at the edge the tiles
 * slide toward first.
 * @param {number} size
 * @param {Direction} direction
 * @returns {number[][]}
 */
export function lines(size, direction) {
  /** @type {number[][]} */
  const all = [];
  for (let a = 0; a < size; a += 1) {
    /** @type {number[]} */
    const line = [];
    for (let b = 0; b < size; b += 1) {
      line.push(direction === "left" || direction === "right" ? a * size + b : b * size + a);
    }
    all.push(direction === "right" || direction === "down" ? line.reverse() : line);
  }
  return all;
}

/**
 * Puts a new tile — a 2, or one time in ten a 4 — on a random empty cell.
 * @param {Game} game
 * @param {RandomInt} random
 * @returns {{ game: Game, id: number | null }}
 */
export function spawn(game, random) {
  const empty = game.cells.flatMap((tile, index) => (tile ? [] : [index]));
  const at = empty.length > 0 ? empty[random(empty.length)] : undefined;
  if (at === undefined) {
    return { game, id: null };
  }
  const value = random(10) === 0 ? 4 : 2;
  const cells = game.cells.slice();
  cells[at] = { id: game.nextId, value };
  return { game: { ...game, cells, nextId: game.nextId + 1 }, id: game.nextId };
}

/**
 * A new game: an empty board with two tiles.
 * @param {RandomInt} random
 * @param {number} [size]
 * @returns {Game}
 */
export function newGame(random, size = SIZE) {
  /** @type {Game} */
  const empty = { size, cells: Array.from({ length: size * size }, () => null), score: 0, nextId: 1, won: false, over: false };
  return spawn(spawn(empty, random).game, random).game;
}

/**
 * Whether any move is left: an empty cell, or two equal tiles side by side.
 * @param {(Tile | null)[]} cells
 * @param {number} size
 */
export function canMove(cells, size) {
  for (let index = 0; index < cells.length; index += 1) {
    const tile = cells[index];
    if (!tile) {
      return true;
    }
    const right = (index + 1) % size !== 0 ? cells[index + 1] : null;
    const below = cells[index + size];
    if (right?.value === tile.value || below?.value === tile.value) {
      return true;
    }
  }
  return false;
}

/**
 * Slides every tile toward `direction`. Two equal tiles meeting merge into one of double the
 * value, and a tile merges at most once per move ([2, 2, 2, 2] → [4, 4], [4, 4, 8] → [8, 8]).
 * When anything moved, a new tile appears.
 * @param {Game} game
 * @param {Direction} direction
 * @param {RandomInt} random
 * @returns {Move}
 */
export function move(game, direction, random) {
  /** @type {Move} */
  const result = { game, moved: false, gained: 0, absorbed: [], merged: [], spawned: null };
  if (game.over) {
    return result;
  }
  const cells = game.cells.slice();
  for (const line of lines(game.size, direction)) {
    /** @type {{ tile: Tile, merged: boolean }[]} */
    const packed = [];
    for (const index of line) {
      const tile = game.cells[index];
      if (!tile) {
        continue;
      }
      const last = packed.at(-1);
      if (last && !last.merged && last.tile.value === tile.value) {
        last.tile = { id: last.tile.id, value: last.tile.value * 2 };
        last.merged = true;
        result.absorbed.push({ id: tile.id, into: last.tile.id });
        result.merged.push(last.tile.id);
        result.gained += last.tile.value;
      } else {
        packed.push({ tile, merged: false });
      }
    }
    line.forEach((index, position) => {
      const next = packed[position]?.tile ?? null;
      const before = game.cells[index] ?? null;
      if (next?.id !== before?.id || next?.value !== before?.value) {
        result.moved = true;
      }
      cells[index] = next;
    });
  }
  if (!result.moved) {
    return result;
  }
  const slid = { ...game, cells, score: game.score + result.gained };
  const spawned = spawn(slid, random);
  const won = game.won || spawned.game.cells.some((tile) => (tile?.value ?? 0) >= GOAL);
  result.game = { ...spawned.game, won, over: !canMove(spawned.game.cells, game.size) };
  result.spawned = spawned.id;
  return result;
}

/**
 * The highest tile on the board.
 * @param {Game} game
 */
export function highest(game) {
  return game.cells.reduce((top, tile) => Math.max(top, tile?.value ?? 0), 0);
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
 * A saved game (from storage) checked from scratch, or null: a square board of `SIZE`, tiles
 * that are powers of two with unique ids, a score that is a whole number.
 * @param {unknown} saved
 * @returns {Game | null}
 */
export function restore(saved) {
  if (typeof saved !== "object" || saved === null) {
    return null;
  }
  const value = /** @type {Record<string, unknown>} */ (saved);
  const { cells, score } = value;
  if (!Array.isArray(cells) || cells.length !== SIZE * SIZE || typeof score !== "number" || !Number.isInteger(score) || score < 0) {
    return null;
  }
  /** @type {(Tile | null)[]} */
  const tiles = [];
  for (const cell of /** @type {unknown[]} */ (cells)) {
    if (cell === null) {
      tiles.push(null);
    } else if (typeof cell === "number" && Number.isInteger(Math.log2(cell)) && cell >= 2 && cell <= MAX_TILE) {
      tiles.push({ id: tiles.length + 1, value: cell });
    } else {
      return null;
    }
  }
  if (tiles.every((tile) => tile === null)) {
    return null;
  }
  const won = value.won === true || tiles.some((tile) => (tile?.value ?? 0) >= GOAL);
  return { size: SIZE, cells: tiles, score, nextId: tiles.length + 1, won, over: !canMove(tiles, SIZE) };
}

/**
 * What storage keeps of a game: the values and the score.
 * @param {Game} game
 */
export function save(game) {
  return { cells: game.cells.map((tile) => tile?.value ?? null), score: game.score, won: game.won };
}

/** @type {Record<string, Action>} */
const KEYS = { ArrowUp: "up", ArrowDown: "down", ArrowLeft: "left", ArrowRight: "right", Enter: "continue", Escape: "escape" };
/** WASD and N by position (`KeyboardEvent.code`), so they sit in the same place on every layout. */
/** @type {Record<string, Action>} */
const CODES = { KeyW: "up", KeyA: "left", KeyS: "down", KeyD: "right", KeyN: "new" };

/**
 * What a key press means: arrows and WASD slide, N starts a new game, Enter keeps playing after
 * a win, Escape leaves. Presses with Ctrl, Alt or Cmd belong to Little Joan and the system.
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
 * `a` mixed with `b` in sRGB: `t` = 0 is `a`, 1 is `b`; `a` when either isn't `#rrggbb`.
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
 * WCAG contrast of two `#rrggbb` colors (1–21; 1 when either isn't one).
 * @param {string} a
 * @param {string} b
 */
export function contrast(a, b) {
  const luminance = (/** @type {[number, number, number]} */ [r, g, b2]) =>
    [r, g, b2]
      .map((v) => v / 255)
      .map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4))
      .reduce((sum, v, i) => sum + v * ([0.2126, 0.7152, 0.0722][i] ?? 0), 0);
  const x = rgb(a);
  const y = rgb(b);
  if (!x || !y) {
    return 1;
  }
  const [hi, lo] = [luminance(x), luminance(y)].sort((p, q) => q - p);
  return ((hi ?? 0) + 0.05) / ((lo ?? 0) + 0.05);
}

/**
 * @typedef {object} Palette the colors tiles are made of (all `#rrggbb`)
 * @property {string} base a 2 (lighter than an empty cell)
 * @property {string} strong 2048 and above
 * @property {string[]} inks the text colors to choose from, per tile
 */

/** The default theme's colors, for anything the theme doesn't give as `#rrggbb`. */
const DEFAULTS = {
  light: { "--lj-surface": "#f0f3f8", "--lj-fg": "#1d1f23", "--lj-row-selected": "#617599", "--lj-on-selected": "#ffffff", "--lj-accent": "#485b7d" },
  dark: { "--lj-surface": "#191d23", "--lj-fg": "#eff2f7", "--lj-row-selected": "#617599", "--lj-on-selected": "#ffffff", "--lj-accent": "#96acd2" },
};

/**
 * The tiles' colors from the theme: in light mode from the surface to the selected-row color
 * (dark enough for white text); in dark mode from a lifted surface to the accent (light there),
 * so tiles always stand out from the empty cells.
 * @param {"light" | "dark"} scheme
 * @param {Record<string, string>} vars the theme's `--lj-*` variables
 * @returns {Palette}
 */
export function palette(scheme, vars) {
  const defaults = DEFAULTS[scheme];
  const color = (/** @type {keyof typeof defaults} */ name) => (rgb(vars[name] ?? "") ? (vars[name] ?? "").trim() : defaults[name]);
  const surface = color("--lj-surface");
  const fg = color("--lj-fg");
  const inks = [fg, color("--lj-on-selected"), surface];
  return scheme === "dark"
    ? { base: mix(surface, fg, 0.2), strong: color("--lj-accent"), inks }
    : { base: surface, strong: color("--lj-row-selected"), inks };
}

/**
 * A tile's colors: from the palette's base toward its strong color as the value grows (2048 is
 * fully strong), with the ink that reads best on it.
 * @param {number} value
 * @param {Palette} colors
 */
export function tileColors(value, colors) {
  const level = Math.min(1, Math.max(0, Math.log2(Math.max(2, value)) / Math.log2(GOAL)));
  const background = mix(colors.base, colors.strong, value >= GOAL ? 1 : 0.04 + 0.96 * level ** 1.25);
  let color = colors.inks[0] ?? "#000000";
  for (const ink of colors.inks) {
    if (contrast(ink, background) > contrast(color, background)) {
      color = ink;
    }
  }
  return { background, color };
}
