import { describe, expect, it } from "vitest";
import { answer, cryptoInt, password, uuid, type RandomInt } from "./random.ts";

/** A repeatable stand-in for the crypto randomness: a simple LCG. */
function seeded(seed: number): RandomInt {
  let state = seed;
  return (n) => {
    state = (state * 1103515245 + 12345) % 2 ** 31;
    return state % n;
  };
}

describe("random answers", () => {
  it("make version 4 UUIDs", () => {
    const id = uuid(seeded(1));
    expect(id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
    expect(answer("uuid", seeded(1))).toEqual({ title: id, subtitle: "A random UUID (v4)", copy: id });
    expect(answer("UUID", seeded(2))?.title).not.toBe(id);
  });

  it("make passwords of the asked length with every kind of character", () => {
    for (const length of [8, 20, 64]) {
      const made = password(length, seeded(length));
      expect(made).toHaveLength(length);
      expect(made).toMatch(/[a-z]/);
      expect(made).toMatch(/[A-Z]/);
      expect(made).toMatch(/[2-9]/);
      expect(made).toMatch(/[!@#$%^&*\-_=+?]/);
      expect(made).not.toMatch(/[lIO01]/);
    }
    expect(answer("password 20", seeded(3))?.title).toHaveLength(20);
    expect(answer("password", seeded(3))?.title).toHaveLength(16);
    expect(answer("password 4", seeded(3))).toBeNull();
    expect(answer("password 500", seeded(3))).toBeNull();
  });

  it("roll dice, flip coins and pick numbers in range", () => {
    const roll = answer("roll 2d6", seeded(4));
    expect(roll?.subtitle).toMatch(/^2d6: [1-6] \+ [1-6]$/);
    const [a, b] = (roll?.subtitle ?? "").replace("2d6: ", "").split(" + ").map(Number);
    expect(Number(roll?.title)).toBe((a ?? 0) + (b ?? 0));
    expect(answer("roll d20", seeded(5))?.subtitle).toBe("1d20");
    expect(answer("3d8", seeded(5))?.subtitle).toMatch(/^3d8: /);
    expect(answer("d20", seeded(5))).toBeNull();
    expect(["Heads", "Tails"]).toContain(answer("flip a coin", seeded(6))?.title);
    for (let seed = 0; seed < 50; seed++) {
      const value = Number(answer("random 1-100", seeded(seed))?.title);
      expect(value).toBeGreaterThanOrEqual(1);
      expect(value).toBeLessThanOrEqual(100);
    }
    expect(answer("random 10 to 5", () => 0)?.title).toBe("5");
    expect(answer("random", (n) => n - 1)?.title).toBe("100");
    expect(answer("random -5..5", () => 0)?.title).toBe("-5");
  });

  it("draw from crypto without bias, and refuse bad ranges", () => {
    for (let i = 0; i < 200; i++) {
      const value = cryptoInt(6);
      expect(value).toBeGreaterThanOrEqual(0);
      expect(value).toBeLessThan(6);
    }
    expect(() => cryptoInt(0)).toThrow(RangeError);
    expect(() => cryptoInt(1.5)).toThrow(RangeError);
  });

  it("have no answer for other text", () => {
    for (const bad of ["random words", "uuids", "roll the dice", "coin", "pw 3", "10 km in mi"]) {
      expect(answer(bad, seeded(7)), bad).toBeNull();
    }
  });
});
