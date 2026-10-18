import { describe, expect, it } from "vitest";
import { answer, evaluate, formatNumber, tokenize } from "./calc.ts";

describe("the calculator", () => {
  it("does + − × ÷ ^ and precedence", () => {
    expect(evaluate("12*7")).toBe(84);
    expect(evaluate("2 + 3 * 4")).toBe(14);
    expect(evaluate("(2 + 3) * 4")).toBe(20);
    expect(evaluate("10 − 4 − 3")).toBe(3);
    expect(evaluate("12 × 3 ÷ 4")).toBe(9);
    expect(evaluate("2 ^ 10")).toBe(1024);
    expect(evaluate("2^3^2")).toBe(512);
    expect(evaluate("-2^2")).toBe(-4);
    expect(evaluate("2^-1")).toBe(0.5);
    expect(evaluate("--3")).toBe(3);
    expect(evaluate("3 x 4")).toBe(12);
    expect(evaluate("((1+2)*(3+4))")).toBe(21);
  });

  it("reads % as percent after a number and as modulo between two", () => {
    expect(evaluate("50%")).toBe(0.5);
    expect(evaluate("50% * 30")).toBe(15);
    expect(evaluate("10 % 3")).toBe(1);
    expect(evaluate("(7 + 3) % 4")).toBe(2);
  });

  it("knows decimals, constants and functions", () => {
    expect(evaluate("1.5 * 2")).toBe(3);
    expect(evaluate(".5 + .25")).toBe(0.75);
    expect(evaluate("1e3 + 1")).toBe(1001);
    expect(evaluate("pi")).toBe(Math.PI);
    expect(evaluate("2pi")).toBeCloseTo(2 * Math.PI);
    expect(evaluate("e")).toBe(Math.E);
    expect(evaluate("sqrt(16)")).toBe(4);
    expect(evaluate("sqrt 2")).toBeCloseTo(Math.SQRT2);
    expect(evaluate("sin(pi / 2)")).toBe(1);
    expect(evaluate("cos(0)")).toBe(1);
    expect(evaluate("log(1000)")).toBe(3);
    expect(evaluate("ln(e)")).toBe(1);
    expect(evaluate("2(3+4)")).toBe(14);
    expect(evaluate("abs(-3) + round(2.5)")).toBe(6);
  });

  it("has no answer for division by zero, nonsense or non-finite results", () => {
    for (const bad of ["1/0", "5 % 0", "1/(2-2)", "sqrt(-1)", "2 +", "(1 + 2", "1 + 2)", "foo(2)", "", "*3", "10^1000", "alert(1)"]) {
      expect(evaluate(bad), bad).toBeNull();
    }
    expect(tokenize("2 & 3")).toBeNull();
    expect(tokenize("1;2")).toBeNull();
  });

  it("writes results without float noise", () => {
    expect(formatNumber(0.1 + 0.2)).toBe("0.3");
    expect(formatNumber(1 / 3)).toBe("0.333333333333");
    expect(formatNumber(2 / 3)).toBe("0.666666666667");
    expect(formatNumber(-0)).toBe("0");
    expect(formatNumber(123456789)).toBe("123456789");
    expect(formatNumber(2 ** 60)).toBe("1.15292150461e+18");
    expect(formatNumber(1e-12)).toBe("1e-12");
    expect(answer("= 0.1 + 0.2")?.title).toBe("0.3");
  });

  it("answers '= …' always, and bare math only when it looks like math", () => {
    expect(answer("= 12*7")).toEqual({ title: "84", subtitle: "= 12*7", copy: "84" });
    expect(answer("12*7")).toEqual({ title: "84", subtitle: "= 12*7", copy: "84" });
    expect(answer("= pi")?.title).toBe("3.14159265359");
    expect(answer("sqrt(2)")?.title).toBe("1.41421356237");
    for (const notMath of ["pi", "2024", "1700000000", "-5", "hello", "10 km in mi", "2024-10-18", "192.168.1.1", "1.2.3", "random 1-100", "= ", "= nonsense"]) {
      expect(answer(notMath), notMath).toBeNull();
    }
    expect(answer(`= ${"1+".repeat(200)}1`)).toBeNull();
  });
});
