// @ts-check
// 2048's panel: the board as DOM tiles that slide (keyed by tile id), colored from Little Joan's
// theme; the rules are logic.js. The game in progress and the best score are kept with
// littleJoan.storage, so closing the panel loses nothing.
import { GOAL, bestScore, keyAction, move, newGame, palette, restore, save, tileColors } from "./logic.js";

/** @typedef {import("./logic.js").Game} Game */
/** @typedef {import("./logic.js").Move} Move */

/** How long a slide takes (style.css has the same). */
const SLIDE_MS = 110;
/** Saves wait this long after the last move (storage takes a few writes a second at most). */
const SAVE_AFTER_MS = 400;

/** An integer in [0, n), from the web's crypto randomness. */
const random = (/** @type {number} */ n) => {
  const value = new Uint32Array(1);
  crypto.getRandomValues(value);
  return (value[0] ?? 0) % n;
};

/** @param {string} id */
function element(id) {
  const found = document.getElementById(id);
  if (!found) {
    throw new Error(`#${id} is missing`);
  }
  return found;
}

const box = element("board-box");
const boardElement = element("board");
const layer = element("tiles");
const scoreText = element("score");
const bestText = element("best");
const message = element("message");
const messageTitle = element("message-title");
const messageText = element("message-text");
const continueButton = element("message-continue");

/** @type {Game} */
let game = newGame(random);
let best = 0;
let savedBest = 0;
/** The win message was shown (once per game that reaches 2048). */
let winSeen = false;
/** @type {ReturnType<typeof setTimeout> | undefined} */
let saveTimer;
/** @type {Map<number, HTMLElement>} */
const tiles = new Map();

/** The tiles' colors, from the theme once it answers. */
let colors = palette("light", {});

/**
 * @param {HTMLElement} tile
 * @param {number} index
 */
function place(tile, index) {
  tile.style.setProperty("--x", String(index % game.size));
  tile.style.setProperty("--y", String(Math.floor(index / game.size)));
}

/**
 * @param {HTMLElement} tile
 * @param {number} value
 */
function paint(tile, value) {
  const { background, color } = tileColors(value, colors);
  tile.textContent = String(value);
  tile.dataset.digits = String(Math.min(5, String(value).length));
  tile.style.background = background;
  tile.style.color = color;
}

/**
 * Restarts a CSS animation class on a tile.
 * @param {HTMLElement} tile
 * @param {string} name
 */
function animate(tile, name) {
  tile.classList.remove(name);
  void tile.offsetWidth;
  tile.classList.add(name);
}

/**
 * Draws the board after `result` (or from scratch with null): tiles slide to their cells, merged
 * ones slide under their partner and go, the new one appears.
 * @param {Move | null} result
 */
function render(result) {
  const absorbed = new Map((result?.absorbed ?? []).map((a) => [a.id, a.into]));
  const merged = new Set(result?.merged ?? []);
  /** @type {Map<number, number>} */
  const where = new Map();
  game.cells.forEach((tile, index) => {
    if (tile) {
      where.set(tile.id, index);
    }
  });
  for (const [id, tile] of tiles) {
    const into = absorbed.get(id);
    const target = into === undefined ? undefined : where.get(into);
    if (target !== undefined) {
      place(tile, target);
      tile.classList.add("gone");
      setTimeout(() => tile.remove(), SLIDE_MS + 20);
      tiles.delete(id);
    } else if (!where.has(id)) {
      tile.remove();
      tiles.delete(id);
    }
  }
  game.cells.forEach((cell, index) => {
    if (!cell) {
      return;
    }
    let tile = tiles.get(cell.id);
    if (!tile) {
      tile = document.createElement("div");
      tile.className = "tile";
      tiles.set(cell.id, tile);
      place(tile, index);
      paint(tile, cell.value);
      layer.append(tile);
      animate(tile, "new");
      return;
    }
    place(tile, index);
    if (merged.has(cell.id)) {
      const value = cell.value;
      const shown = tile;
      setTimeout(() => {
        paint(shown, value);
        animate(shown, "merged");
      }, SLIDE_MS);
    } else {
      paint(tile, cell.value);
    }
  });
  show();
}

function show() {
  best = bestScore(best, game.score);
  scoreText.textContent = String(game.score);
  bestText.textContent = String(best);
  boardElement.setAttribute("aria-label", `2048 board, score ${game.score}`);
  if (game.won && !winSeen && !game.over) {
    messageTitle.textContent = `You made ${GOAL}!`;
    messageText.textContent = "Keep going for a bigger tile, or start again.";
    continueButton.textContent = "Keep going";
    message.hidden = false;
  } else if (game.over) {
    messageTitle.textContent = game.score >= best && game.score > savedBest ? "New best!" : "No moves left";
    messageText.textContent = `You scored ${game.score}.`;
    continueButton.textContent = "Try again";
    message.hidden = false;
  } else {
    message.hidden = true;
  }
}

/** Saves the game and the best score a moment after the last move. */
function scheduleSave() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => void flush(), SAVE_AFTER_MS);
}

/** Saves now; resolves when storage answered (or failed). */
async function flush() {
  clearTimeout(saveTimer);
  saveTimer = undefined;
  const writes = [littleJoan.storage.set("game", save(game))];
  if (best > savedBest) {
    savedBest = best;
    writes.push(littleJoan.storage.set("best", best));
  }
  try {
    await Promise.all(writes);
  } catch {
    // Without storage the game lasts while the panel is open.
  }
}

function restart() {
  game = newGame(random);
  winSeen = false;
  render(null);
  scheduleSave();
  boardElement.focus();
}

/** Enter (or the message's first button): keep playing after the win, or try again. */
function proceed() {
  if (game.over) {
    restart();
  } else if (game.won && !winSeen) {
    winSeen = true;
    show();
    boardElement.focus();
  }
}

window.addEventListener(
  "keydown",
  (event) => {
    const action = keyAction(event);
    if (!action) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    if (action === "escape") {
      void flush().then(() => littleJoan.close());
    } else if (action === "new") {
      restart();
    } else if (action === "continue") {
      proceed();
    } else if (message.hidden) {
      const result = move(game, action, random);
      if (result.moved) {
        game = result.game;
        render(result);
        scheduleSave();
      }
    }
  },
  true,
);

element("new-game").addEventListener("click", restart);
element("message-new").addEventListener("click", restart);
continueButton.addEventListener("click", proceed);
window.addEventListener("pagehide", () => {
  if (saveTimer !== undefined) {
    void flush();
  }
});

/** The board: the largest square that fits its box (at most 460 px), cells and gaps from it. */
function resize() {
  const room = box.getBoundingClientRect();
  const side = Math.max(200, Math.min(room.width, room.height, 460));
  const gap = Math.round(Math.max(6, side * 0.026));
  const cell = (side - gap * (game.size + 1)) / game.size;
  boardElement.style.setProperty("--side", `${side}px`);
  boardElement.style.setProperty("--gap", `${gap}px`);
  boardElement.style.setProperty("--cell", `${cell}px`);
}

async function start() {
  try {
    const theme = await littleJoan.theme.apply();
    colors = palette(theme.scheme, theme.vars);
  } catch {
    // The fallback colors in style.css stay.
  }
  document.documentElement.classList.add("ready");
  try {
    const [saved, stored] = await Promise.all([littleJoan.storage.get("game"), littleJoan.storage.get("best")]);
    savedBest = bestScore(stored, 0);
    best = savedBest;
    const resumed = restore(saved);
    if (resumed) {
      game = resumed;
      winSeen = resumed.won;
    }
  } catch {
    // Without storage the game lasts while the panel is open.
  }
  new ResizeObserver(resize).observe(box);
  resize();
  render(null);
  boardElement.focus();
}

void start();
