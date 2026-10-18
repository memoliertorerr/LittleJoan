import { describe, expect, it } from "vitest";
import {
  contrast,
  fitGamut,
  hexToOklab,
  hexToOklch,
  inGamut,
  mixOklab,
  normalizeHex,
  oklabToOklch,
  oklabToRgb,
  oklchToHex,
  oklchToOklab,
  over,
  rgba,
  rgbToOklab,
  toHex,
} from "./color.ts";

const close = (actual: number, expected: number, digits = 4) => expect(actual).toBeCloseTo(expected, digits);

describe("hex ↔ OKLab / OKLCH", () => {
  it("matches known values", () => {
    // Björn Ottosson's reference values (and CSS Color 4's) for the sRGB primaries.
    const known: [string, number, number, number][] = [
      ["#ff0000", 0.627955, 0.224863, 0.125846],
      ["#00ff00", 0.86644, -0.233888, 0.179498],
      ["#0000ff", 0.452014, -0.032457, -0.311528],
      ["#ffffff", 1, 0, 0],
      ["#000000", 0, 0, 0],
    ];
    for (const [hex, l, a, b] of known) {
      const lab = hexToOklab(hex);
      close(lab?.l ?? NaN, l);
      close(lab?.a ?? NaN, a);
      close(lab?.b ?? NaN, b);
    }
    const red = hexToOklch("#ff0000");
    close(red?.c ?? NaN, 0.257683);
    close(red?.h ?? NaN, 29.2339, 3);
    // A grey has no hue: 0, not NaN.
    expect(hexToOklch("#808080")?.h).toBe(0);
    close(hexToOklch("#808080")?.c ?? NaN, 0, 6);
  });

  it("round-trips every #rgb color", () => {
    const digits = "0123456789abcdef";
    for (const r of digits) {
      for (const g of digits) {
        for (const b of digits) {
          const hex = normalizeHex(`#${r}${g}${b}`) ?? "";
          const lab = hexToOklab(hex);
          expect(lab && toHex(oklabToRgb(lab))).toBe(hex);
          expect(lab && oklchToHex(oklabToOklch(lab))).toBe(hex);
        }
      }
    }
  });

  it("goes between OKLab and OKLCH both ways", () => {
    const lab = { l: 0.7, a: -0.05, b: 0.08 };
    const back = oklchToOklab(oklabToOklch(lab));
    close(back.l, lab.l, 9);
    close(back.a, lab.a, 9);
    close(back.b, lab.b, 9);
    const rgb = { r: 0.2, g: 0.4, b: 0.6 };
    const again = oklabToRgb(rgbToOklab(rgb));
    close(again.r, rgb.r, 6);
    close(again.g, rgb.g, 6);
    close(again.b, rgb.b, 6);
  });

  it("reads #rgb and #rrggbb in any case, nothing else", () => {
    expect(normalizeHex("#ABC")).toBe("#aabbcc");
    expect(normalizeHex(" #A1b2C3 ")).toBe("#a1b2c3");
    for (const bad of ["abc", "#abcd", "#12345g", "", "#", null, 7, "red", "#1234567"]) {
      expect(normalizeHex(bad)).toBeNull();
    }
  });
});

describe("gamut, mixing, contrast", () => {
  it("lowers chroma until sRGB can show the color, keeping lightness and hue", () => {
    const wild = { l: 0.9, c: 0.3, h: 260 };
    expect(inGamut(oklabToRgb(oklchToOklab(wild)))).toBe(false);
    const fitted = fitGamut(wild);
    expect(inGamut(oklabToRgb(oklchToOklab(fitted)))).toBe(true);
    expect(fitted.l).toBe(0.9);
    expect(fitted.h).toBe(260);
    expect(fitted.c).toBeLessThan(0.3);
    expect(fitted.c).toBeGreaterThan(0.01);
    const tame = { l: 0.6, c: 0.05, h: 120 };
    expect(fitGamut(tame)).toEqual(tame);
  });

  it("mixes in OKLab", () => {
    const a = { l: 0.2, a: 0.1, b: -0.1 };
    const b = { l: 0.8, a: -0.1, b: 0.1 };
    expect(mixOklab(a, b, 0)).toEqual(a);
    expect(mixOklab(a, b, 1)).toEqual(b);
    const mid = mixOklab(a, b, 0.5);
    close(mid.l, 0.5, 9);
    close(mid.a, 0, 9);
  });

  it("computes WCAG contrast", () => {
    close(contrast("#000000", "#ffffff"), 21, 6);
    close(contrast("#ffffff", "#000000"), 21, 6);
    close(contrast("#777777", "#777777"), 1, 9);
    // #767676 on white is the classic 4.54:1 grey.
    close(contrast("#767676", "#ffffff"), 4.54, 2);
  });

  it("composites and writes rgba() with commas", () => {
    expect(over({ r: 1, g: 1, b: 1 }, 0.5, { r: 0, g: 0, b: 0 })).toEqual({ r: 0.5, g: 0.5, b: 0.5 });
    expect(rgba({ r: 1, g: 0.5, b: 0 }, 0.25)).toBe("rgba(255, 128, 0, 0.25)");
    expect(toHex({ r: 1.2, g: -0.1, b: 0.5 })).toBe("#ff0080");
  });
});
