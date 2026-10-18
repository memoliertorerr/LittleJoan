// @ts-check
// Snake's panel: draws the game on a canvas in Little Joan's colors and feeds the rules
// (logic.js) its keys and ticks. The best score is kept with littleJoan.storage.
import { COLS, ROWS, bestScore, complementOf, keyAction, mix, newGame, step, tickMs, togglePause, turn } from "./logic.js";

/** @typedef {import("./logic.js").Game} Game */
/** @typedef {import("./logic.js").Direction} Direction */

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

const canvas = /** @type {HTMLCanvasElement} */ (element("board"));
const box = element("board-box");
const scoreText = element("score");
const bestText = element("best");
const message = element("message");
const messageTitle = element("message-title");
const messageText = element("message-text");
const context = canvas.getContext("2d");

/** @type {Game} */
let game = newGame(random);
let best = 0;
let savedBest = 0;
let lastStep = 0;
let looping = false;
/** One cell, in CSS pixels. */
let cell = 16;

/** The canvas's colors, from the --lj-* variables the theme set (these until it answers). */
const colors = { board: "rgba(127,127,127,0.10)", check: "rgba(127,127,127,0.05)", snake: "#485b7d", tail: "#a4adbf", food: "#7d6a48", eye: "#f0f3f8" };

function readColors() {
  const style = getComputedStyle(document.documentElement);
  const read = (/** @type {string} */ name, /** @type {string} */ fallback) => style.getPropertyValue(name).trim() || fallback;
  colors.board = read("--lj-field", colors.board);
  colors.check = read("--lj-row-hover", colors.check);
  colors.snake = read("--lj-accent", colors.snake);
  colors.eye = read("--lj-surface", colors.eye);
  colors.tail = mix(colors.snake, colors.eye, 0.45);
  colors.food = complementOf(colors.snake);
}

/**
 * A rounded rectangle's path.
 * @param {CanvasRenderingContext2D} ctx
 * @param {number} x
 * @param {number} y
 * @param {number} w
 * @param {number} h
 * @param {number} r
 */
function roundRect(ctx, x, y, w, h, r) {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.arcTo(x + w, y, x + w, y + h, r);
  ctx.arcTo(x + w, y + h, x, y + h, r);
  ctx.arcTo(x, y + h, x, y, r);
  ctx.arcTo(x, y, x + w, y, r);
  ctx.closePath();
}

/** @type {Record<Direction, [number, number]>} */
const LOOK = { up: [0, -1], down: [0, 1], left: [-1, 0], right: [1, 0] };

function draw() {
  if (!context) {
    return;
  }
  const ratio = window.devicePixelRatio || 1;
  const width = cell * COLS;
  const height = cell * ROWS;
  context.setTransform(ratio, 0, 0, ratio, 0, 0);
  context.clearRect(0, 0, width, height);
  roundRect(context, 0, 0, width, height, Math.min(14, cell));
  context.fillStyle = colors.board;
  context.fill();
  context.save();
  context.clip();
  context.fillStyle = colors.check;
  context.globalAlpha = 0.7;
  for (let y = 0; y < ROWS; y += 1) {
    for (let x = y % 2; x < COLS; x += 2) {
      context.fillRect(x * cell, y * cell, cell, cell);
    }
  }
  context.globalAlpha = 1;
  context.restore();

  if (game.food) {
    context.fillStyle = colors.food;
    context.beginPath();
    context.arc((game.food.x + 0.5) * cell, (game.food.y + 0.5) * cell, cell * 0.34, 0, Math.PI * 2);
    context.fill();
  }

  // The body from the tail up, each segment joined to the one ahead, fading toward the tail.
  const inset = Math.max(1, Math.round(cell * 0.12));
  const length = game.snake.length;
  for (let i = length - 1; i >= 0; i -= 1) {
    const part = game.snake[i];
    if (!part) {
      continue;
    }
    context.fillStyle = mix(colors.snake, colors.tail, length > 1 ? i / (length - 1) : 0);
    roundRect(context, part.x * cell + inset, part.y * cell + inset, cell - 2 * inset, cell - 2 * inset, cell * 0.3);
    context.fill();
    const ahead = game.snake[i - 1];
    if (ahead) {
      const left = Math.min(part.x, ahead.x) * cell + inset;
      const top = Math.min(part.y, ahead.y) * cell + inset;
      context.fillRect(left, top, (Math.abs(part.x - ahead.x) + 1) * cell - 2 * inset, (Math.abs(part.y - ahead.y) + 1) * cell - 2 * inset);
    }
  }

  // Eyes, looking the way the snake goes next.
  const head = game.snake[0];
  if (head) {
    const [dx, dy] = LOOK[game.turns[0] ?? game.heading];
    const cx = (head.x + 0.5) * cell + dx * cell * 0.14;
    const cy = (head.y + 0.5) * cell + dy * cell * 0.14;
    const spread = cell * 0.2;
    context.fillStyle = colors.eye;
    for (const side of [-1, 1]) {
      context.beginPath();
      context.arc(cx + dy * spread * side, cy + dx * spread * side, Math.max(1.2, cell * 0.09), 0, Math.PI * 2);
      context.fill();
    }
  }
}

/** The canvas: the largest square of whole cells that fits its box, sharp at any scale. */
function resize() {
  const room = box.getBoundingClientRect();
  cell = Math.max(6, Math.floor(Math.max(120, Math.min(room.width, room.height)) / COLS));
  const ratio = window.devicePixelRatio || 1;
  canvas.style.width = `${cell * COLS}px`;
  canvas.style.height = `${cell * ROWS}px`;
  canvas.width = Math.round(cell * COLS * ratio);
  canvas.height = Math.round(cell * ROWS * ratio);
  draw();
}

/** @type {Partial<Record<import("./logic.js").Status, [string, string]>>} */
const MESSAGES = {
  ready: ["Snake", "Press an arrow key, WASD or Space to start."],
  paused: ["Paused", "Space or an arrow key to go on · Esc to close."],
  over: ["Game over", "Enter to play again · Esc to close."],
  won: ["You filled the board!", "Enter to play again."],
};

function show() {
  best = bestScore(best, game.score);
  scoreText.textContent = String(game.score);
  bestText.textContent = String(best);
  const text = MESSAGES[game.status];
  message.hidden = !text;
  if (text) {
    const record = game.status === "over" && game.score > 0 && game.score >= best && game.score > savedBest;
    messageTitle.textContent = record ? "New best!" : text[0];
    messageText.textContent = game.status === "over" ? `You scored ${game.score}. ${text[1]}` : text[1];
  }
  canvas.setAttribute("aria-label", `Snake, score ${game.score}, best ${best}`);
  if ((game.status === "over" || game.status === "won") && best > savedBest) {
    savedBest = best;
    littleJoan.storage.set("best", best).catch(() => {
      // Without storage the best score lasts while the panel is open.
    });
  }
  draw();
}

/** @param {number} now */
function frame(now) {
  if (game.status !== "running") {
    looping = false;
    return;
  }
  if (now - lastStep >= tickMs(game.score)) {
    lastStep = now;
    update(step(game, random));
  }
  requestAnimationFrame(frame);
}

/** @param {Game} next */
function update(next) {
  if (next === game) {
    return;
  }
  const started = game.status !== "running" && next.status === "running";
  game = next;
  show();
  if (started) {
    lastStep = performance.now();
    if (!looping) {
      looping = true;
      requestAnimationFrame(frame);
    }
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
    switch (action) {
      case "escape":
        if (game.status === "running") {
          update(togglePause(game));
        } else {
          void littleJoan.close();
        }
        break;
      case "pause":
        update(togglePause(game));
        break;
      case "start":
        if (game.status === "over" || game.status === "won") {
          update(togglePause(newGame(random)));
        } else if (game.status !== "running") {
          update(togglePause(game));
        }
        break;
      default:
        update(turn(game, action));
    }
  },
  true,
);

// A game in progress waits while the panel isn't in front.
const pause = () => {
  if (game.status === "running") {
    update(togglePause(game));
  }
};
window.addEventListener("blur", pause);
document.addEventListener("visibilitychange", () => {
  if (document.hidden) {
    pause();
  }
});

element("new-game").addEventListener("click", () => {
  update(newGame(random));
  canvas.focus();
});

async function start() {
  try {
    await littleJoan.theme.apply();
  } catch {
    // The fallback colors in style.css stay.
  }
  readColors();
  try {
    savedBest = bestScore(await littleJoan.storage.get("best"), 0);
    best = savedBest;
  } catch {
    // Without storage the best score lasts while the panel is open.
  }
  new ResizeObserver(resize).observe(box);
  resize();
  show();
  canvas.focus();
}

void start();
