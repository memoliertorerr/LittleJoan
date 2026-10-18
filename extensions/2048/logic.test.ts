import { describe, expect, it } from "vitest";
import {
  GOAL,
  SIZE,
  bestScore,
  canMove,
  contrast,
  highest,
  keyAction,
  lines,
  mix,
  move,
  palette,
  newGame,
  restore,
  save,
  spawn,
  tileColors,
  type Game,
  type Tile,
} from "./logic.js";

/** The new tile goes on the first empty cell, and is a 2 (random(10) = 0 would make a 4). */
const first = (n: number) => (n === 10 ? 1 : 0);
/** The new tile goes on the last empty cell (out of the way of row 0), and is a 2. */
const last = (n: number) => n - 1;

/** A board from values (0 = empty), ids in reading order. */
function board(values: number[], options: Partial<Game> = {}): Game {
  const size = Math.round(Math.sqrt(values.length));
  const cells = values.map((value, index): Tile | null => (value ? { id: index + 1, value } : null));
  return { size, cells, score: 0, nextId: values.length + 1, won: false, over: false, ...options };
}

const values = (game: Game) => game.cells.map((tile) => tile?.value ?? 0);
const empty = (n: number): null[] => Array.from({ length: n }, () => null);

describe("2048", () => {
  it("starts with two tiles on an empty board", () => {
    const game = newGame(first);
    expect(game.size).toBe(SIZE);
    expect(game.cells.filter(Boolean)).toHaveLength(2);
    expect(values(game).slice(0, 2)).toEqual([2, 2]);
    expect(game.score).toBe(0);
  });

  it("packs each line toward the edge the move goes to", () => {
    expect(lines(2, "left")).toEqual([[0, 1], [2, 3]]);
    expect(lines(2, "right")).toEqual([[1, 0], [3, 2]]);
    expect(lines(2, "up")).toEqual([[0, 2], [1, 3]]);
    expect(lines(2, "down")).toEqual([[2, 0], [3, 1]]);
  });

  it("merges each tile at most once per move", () => {
    const cases: [number[], number[], number][] = [
      [[2, 2, 2, 2], [4, 4, 0, 0], 8],
      [[2, 2, 4, 0], [4, 4, 0, 0], 4],
      [[4, 4, 8, 0], [8, 8, 0, 0], 8],
      [[2, 0, 2, 0], [4, 0, 0, 0], 4],
      [[2, 2, 2, 0], [4, 2, 0, 0], 4],
      [[8, 4, 4, 4], [8, 8, 4, 0], 8],
      [[2, 4, 8, 16], [2, 4, 8, 16], 0],
    ];
    for (const [row, packed, gained] of cases) {
      const game = board([...row, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
      const result = move(game, "left", last);
      expect(values(result.game).slice(0, 4)).toEqual(packed);
      expect(result.gained).toBe(gained);
      expect(result.game.score).toBe(gained);
    }
    // To the right, the tiles nearer the right edge merge first.
    const right = move(board([2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), "right", last);
    expect(values(right.game).slice(0, 4)).toEqual([0, 0, 2, 4]);
  });

  it("keeps the edge tile of a merge (for the slide), and says which went into which", () => {
    const result = move(board([2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), "left", first);
    expect(result.game.cells[0]).toEqual({ id: 1, value: 4 });
    expect(result.absorbed).toEqual([{ id: 2, into: 1 }]);
    expect(result.merged).toEqual([1]);
    const down = move(board([4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), "down", first);
    expect(down.game.cells[12]).toEqual({ id: 5, value: 8 });
    expect(down.absorbed).toEqual([{ id: 1, into: 5 }]);
  });

  it("adds one new tile after a move, never after a non-move", () => {
    const game = board([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    const stuck = move(game, "left", first);
    expect(stuck.moved).toBe(false);
    expect(stuck.game).toBe(game);
    expect(stuck.spawned).toBeNull();
    const moved = move(game, "right", first);
    expect(moved.moved).toBe(true);
    expect(moved.game.cells.filter(Boolean)).toHaveLength(2);
    expect(moved.spawned).toBe(game.nextId);
    expect(moved.game.cells[0]).toEqual({ id: game.nextId, value: 2 });
    // One time in ten a 4, on the cell the randomness picks among the empty ones.
    const four = spawn(board([2, 0, 0, 0]), (n) => (n === 10 ? 0 : 2));
    expect(values(four.game)).toEqual([2, 0, 0, 4]);
    expect(spawn(board([2, 4, 8, 16]), first).id).toBeNull();
  });

  it("is over when the board is full and no two neighbors match", () => {
    const full = [2, 4, 2, 4, 4, 2, 4, 2, 2, 4, 2, 4, 4, 2, 4, 2];
    expect(canMove(board(full).cells, 4)).toBe(false);
    expect(canMove(board([...full.slice(0, 15), 0]).cells, 4)).toBe(true);
    expect(canMove(board([2, 2, ...full.slice(2)]).cells, 4)).toBe(true);
    // Equal tiles at the end of one row and the start of the next aren't neighbors.
    expect(canMove(board([2, 4, 4, 2]).cells, 2)).toBe(false);
    // The move that fills the board for good ends the game.
    const last = board([2, 4, 2, 4, 4, 2, 4, 2, 2, 4, 2, 4, 0, 4, 2, 8]);
    const ended = move(last, "left", first);
    expect(values(ended.game)).toEqual([2, 4, 2, 4, 4, 2, 4, 2, 2, 4, 2, 4, 4, 2, 8, 2]);
    expect(ended.game.over).toBe(true);
    expect(move(ended.game, "right", first).moved).toBe(false);
  });

  it("is won when a tile reaches 2048, and play can go on", () => {
    const result = move(board([1024, 1024, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), "left", first);
    expect(result.game.won).toBe(true);
    expect(highest(result.game)).toBe(GOAL);
    expect(result.game.over).toBe(false);
    const next = move(result.game, "right", first);
    expect(next.moved).toBe(true);
    expect(next.game.won).toBe(true);
  });

  it("keeps the best score", () => {
    expect(bestScore(undefined, 120)).toBe(120);
    expect(bestScore(4000, 120)).toBe(4000);
    expect(bestScore(100.7, 120)).toBe(120);
    expect(bestScore({ best: 9 }, 0)).toBe(0);
  });

  it("saves a game and reads it back, refusing anything odd", () => {
    const game = move(board([2, 2, 4, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16]), "left", first).game;
    const saved = save(game);
    expect(saved.cells).toEqual(values(game).map((v) => v || null));
    const back = restore(JSON.parse(JSON.stringify(saved)));
    expect(back && values(back)).toEqual(values(game));
    expect(back?.score).toBe(game.score);
    expect(new Set(back?.cells.filter(Boolean).map((t) => t?.id)).size).toBe(game.cells.filter(Boolean).length);
    for (const bad of [
      null,
      "x",
      { cells: [2], score: 0 },
      { cells: empty(16), score: 0 },
      { cells: [3, ...empty(15)], score: 0 },
      { cells: [2 ** 18, ...empty(15)], score: 0 },
      { cells: [2, ...empty(15)], score: -1 },
      { cells: [2, ...empty(15)], score: 1.5 },
      { cells: ["2", ...empty(15)], score: 0 },
    ]) {
      expect(restore(bad)).toBeNull();
    }
    const stuck = restore({ cells: [2, 4, 2, 4, 4, 2, 4, 2, 2, 4, 2, 4, 4, 2, 4, 2], score: 10 });
    expect(stuck?.over).toBe(true);
  });

  it("slides with the arrows and WASD (by position); N starts again", () => {
    expect(keyAction({ key: "ArrowLeft" })).toBe("left");
    expect(keyAction({ key: "ArrowUp" })).toBe("up");
    expect(keyAction({ key: "q", code: "KeyA" })).toBe("left");
    expect(keyAction({ key: "s", code: "KeyS" })).toBe("down");
    expect(keyAction({ key: "d", code: "KeyD" })).toBe("right");
    expect(keyAction({ key: "w", code: "KeyW" })).toBe("up");
    expect(keyAction({ key: "n", code: "KeyN" })).toBe("new");
    expect(keyAction({ key: "Enter" })).toBe("continue");
    expect(keyAction({ key: "Escape" })).toBe("escape");
    expect(keyAction({ key: "ArrowLeft", metaKey: true })).toBeNull();
    expect(keyAction({ key: "x", code: "KeyX" })).toBeNull();
  });

  it("colors tiles from the theme, readable in light and dark", () => {
    const light = {
      "--lj-surface": "#f0f3f8",
      "--lj-fg": "#1d1f23",
      "--lj-row-selected": "#617599",
      "--lj-on-selected": "#ffffff",
      "--lj-accent": "#485b7d",
      "--lj-field": "rgba(29, 31, 35, 0.06)",
    };
    const dark = { ...light, "--lj-surface": "#191d23", "--lj-fg": "#eff2f7", "--lj-accent": "#96acd2" };
    for (const [scheme, vars] of [["light", light], ["dark", dark]] as const) {
      const colors = palette(scheme, vars);
      let previous = 1;
      for (let value = 2; value <= 2 ** 17; value *= 2) {
        const { background, color } = tileColors(value, colors);
        // Tile numbers are large bold text: 3:1 is the bar (WCAG AA for large text).
        expect(contrast(color, background), `${scheme} ${value}: ${color} on ${background}`).toBeGreaterThanOrEqual(3);
        const away = contrast(background, colors.base);
        expect(away).toBeGreaterThanOrEqual(previous);
        previous = away;
      }
      expect(tileColors(GOAL, colors).background).toBe(colors.strong);
    }
    expect(palette("light", light).strong).toBe("#617599");
    expect(palette("dark", dark).strong).toBe("#96acd2");
    // Anything that isn't #rrggbb falls back to the default theme's color.
    expect(palette("light", { "--lj-surface": "oklch(0.9 0 0)" }).base).toBe("#f0f3f8");
    expect(mix("#000000", "#ffffff", 0.5)).toBe("#808080");
    expect(contrast("#000000", "#ffffff")).toBeCloseTo(21, 5);
    expect(contrast("bad", "#ffffff")).toBe(1);
  });
});
