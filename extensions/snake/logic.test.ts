import { describe, expect, it } from "vitest";
import {
  BASE_TICK,
  COLS,
  MIN_TICK,
  ROWS,
  START_LENGTH,
  bestScore,
  complementOf,
  keyAction,
  mix,
  newGame,
  placeFood,
  step,
  tickMs,
  togglePause,
  turn,
  type Game,
} from "./logic.js";

/** Always the first free cell. */
const first = () => 0;

/** A running game with the snake and food where a test wants them. */
function game(snake: { x: number; y: number }[], options: Partial<Game> = {}): Game {
  return { cols: 6, rows: 6, snake, heading: "right", turns: [], food: { x: 5, y: 5 }, score: 0, status: "running", ...options };
}

describe("snake", () => {
  it("starts in the middle, heading right, waiting for a key", () => {
    const start = newGame(first);
    expect(start.cols).toBe(COLS);
    expect(start.rows).toBe(ROWS);
    expect(start.snake).toHaveLength(START_LENGTH);
    expect(start.snake[0]).toEqual({ x: COLS / 2, y: ROWS / 2 });
    expect(start.status).toBe("ready");
    expect(step(start, first)).toBe(start);
    expect(start.food).toEqual({ x: 0, y: 0 });
  });

  it("moves one cell per step on the grid", () => {
    let g = game([{ x: 2, y: 2 }, { x: 1, y: 2 }, { x: 0, y: 2 }]);
    g = step(g, first);
    expect(g.snake).toEqual([{ x: 3, y: 2 }, { x: 2, y: 2 }, { x: 1, y: 2 }]);
    g = step(turn(g, "down"), first);
    expect(g.snake).toEqual([{ x: 3, y: 3 }, { x: 3, y: 2 }, { x: 2, y: 2 }]);
    expect(g.heading).toBe("down");
  });

  it("never turns back on itself, also through quick presses", () => {
    const g = game([{ x: 2, y: 2 }, { x: 1, y: 2 }, { x: 0, y: 2 }]);
    expect(turn(g, "left")).toBe(g);
    expect(turn(g, "right")).toBe(g);
    // Up, then down at once: down is the reverse of the waiting up, so it is dropped.
    const quick = turn(turn(g, "up"), "down");
    expect(quick.turns).toEqual(["up"]);
    // Up, then left: a U-turn in two steps is fine.
    const uTurn = turn(turn(g, "up"), "left");
    expect(uTurn.turns).toEqual(["up", "left"]);
    const moved = step(step(uTurn, first), first);
    expect(moved.snake[0]).toEqual({ x: 1, y: 1 });
    expect(moved.status).toBe("running");
    // At most two turns wait.
    expect(turn(uTurn, "down").turns).toEqual(["up", "left"]);
  });

  it("grows when it eats, scores, and puts new food on a free cell", () => {
    const g = game([{ x: 2, y: 2 }, { x: 1, y: 2 }, { x: 0, y: 2 }], { food: { x: 3, y: 2 } });
    const ate = step(g, first);
    expect(ate.snake).toEqual([{ x: 3, y: 2 }, { x: 2, y: 2 }, { x: 1, y: 2 }, { x: 0, y: 2 }]);
    expect(ate.score).toBe(1);
    expect(ate.food).toEqual({ x: 0, y: 0 });
    expect(ate.snake.some((c) => c.x === ate.food?.x && c.y === ate.food.y)).toBe(false);
    const after = step(ate, first);
    expect(after.snake).toHaveLength(4);
  });

  it("ends at the walls", () => {
    for (const [heading, head] of [
      ["right", { x: 5, y: 2 }],
      ["left", { x: 0, y: 2 }],
      ["up", { x: 2, y: 0 }],
      ["down", { x: 2, y: 5 }],
    ] as const) {
      const g = game([head], { heading });
      const over = step(g, first);
      expect(over.status).toBe("over");
      expect(over.snake).toEqual([head]);
      expect(step(over, first)).toBe(over);
      expect(turn(over, "up")).toBe(over);
    }
  });

  it("ends when it runs into itself, but may follow its own tail", () => {
    // A square of five: turning down runs into the body.
    const body = [
      { x: 2, y: 2 },
      { x: 3, y: 2 },
      { x: 3, y: 3 },
      { x: 2, y: 3 },
      { x: 1, y: 3 },
    ];
    expect(step(game(body, { heading: "left", turns: ["down"] }), first).status).toBe("over");
    // A ring of four: the head moves into the cell the tail leaves.
    const ring = [
      { x: 2, y: 2 },
      { x: 3, y: 2 },
      { x: 3, y: 3 },
      { x: 2, y: 3 },
    ];
    const chased = step(game(ring, { heading: "left", turns: ["down"] }), first);
    expect(chased.status).toBe("running");
    expect(chased.snake[0]).toEqual({ x: 2, y: 3 });
    // …unless it eats there and grows.
    const grown = step(game(ring, { heading: "left", turns: ["down"], food: { x: 2, y: 3 } }), first);
    expect(grown.status).toBe("over");
  });

  it("speeds up with every food, to a limit", () => {
    expect(tickMs(0)).toBe(BASE_TICK);
    expect(tickMs(1)).toBeLessThan(tickMs(0));
    expect(tickMs(5)).toBeLessThan(tickMs(1));
    expect(tickMs(1000)).toBe(MIN_TICK);
  });

  it("pauses on Space and resumes; a turn resumes too", () => {
    const g = game([{ x: 2, y: 2 }]);
    const paused = togglePause(g);
    expect(paused.status).toBe("paused");
    expect(step(paused, first)).toBe(paused);
    expect(togglePause(paused).status).toBe("running");
    expect(turn(paused, "up").status).toBe("running");
    expect(togglePause(newGame(first)).status).toBe("running");
    expect(turn(newGame(first), "left").status).toBe("running");
    const over = { ...g, status: "over" as const };
    expect(togglePause(over)).toBe(over);
  });

  it("steers with the arrows and WASD (by position), pauses on Space", () => {
    expect(keyAction({ key: "ArrowUp" })).toBe("up");
    expect(keyAction({ key: "ArrowDown" })).toBe("down");
    expect(keyAction({ key: "ArrowLeft" })).toBe("left");
    expect(keyAction({ key: "ArrowRight" })).toBe("right");
    expect(keyAction({ key: "w", code: "KeyW" })).toBe("up");
    expect(keyAction({ key: "A", code: "KeyA" })).toBe("left");
    expect(keyAction({ key: "s", code: "KeyS" })).toBe("down");
    expect(keyAction({ key: "d", code: "KeyD" })).toBe("right");
    // AZERTY: the key in W's place says "z".
    expect(keyAction({ key: "z", code: "KeyW" })).toBe("up");
    expect(keyAction({ key: " ", code: "Space" })).toBe("pause");
    expect(keyAction({ key: "Enter" })).toBe("start");
    expect(keyAction({ key: "Escape" })).toBe("escape");
    expect(keyAction({ key: "x", code: "KeyX" })).toBeNull();
    expect(keyAction({ key: "ArrowUp", ctrlKey: true })).toBeNull();
    expect(keyAction({ key: "w", code: "KeyW", metaKey: true })).toBeNull();
  });

  it("places food only on free cells, and none on a full board (a win)", () => {
    const snake = [
      { x: 0, y: 0 },
      { x: 1, y: 0 },
    ];
    expect(placeFood(2, 2, snake, first)).toEqual({ x: 0, y: 1 });
    expect(placeFood(2, 2, snake, (n) => n - 1)).toEqual({ x: 1, y: 1 });
    expect(placeFood(2, 1, snake, first)).toBeNull();
    const almost = game([{ x: 1, y: 0 }, { x: 0, y: 0 }, { x: 0, y: 1 }], { cols: 2, rows: 2, heading: "down", food: { x: 1, y: 1 } });
    const won = step(almost, first);
    expect(won.status).toBe("won");
    expect(won.food).toBeNull();
    expect(won.score).toBe(1);
  });

  it("keeps the best score", () => {
    expect(bestScore(undefined, 3)).toBe(3);
    expect(bestScore(10, 3)).toBe(10);
    expect(bestScore(2, 7)).toBe(7);
    expect(bestScore("12", 1)).toBe(1);
    expect(bestScore(-4, 0)).toBe(0);
  });

  it("colors the food opposite the accent, a little livelier", () => {
    expect(complementOf("#ff0000")).toBe("#13ecec");
    expect(complementOf("#485b7d")).toBe("#8d6e38");
    expect(complementOf("#777777")).toBe("#777777");
    expect(complementOf("nope", "#123456")).toBe("#123456");
  });

  it("fades the body toward the background", () => {
    expect(mix("#000000", "#ffffff", 0)).toBe("#000000");
    expect(mix("#000000", "#ffffff", 1)).toBe("#ffffff");
    expect(mix("#000000", "#ffffff", 0.5)).toBe("#808080");
    expect(mix("#485b7d", "#f0f3f8", 2)).toBe("#f0f3f8");
    expect(mix("rgba(0,0,0,0.1)", "#ffffff", 0.5)).toBe("rgba(0,0,0,0.1)");
  });
});
