import { describe, expect, it } from "vitest";
import {
  contrast,
  fromHsl,
  fromOklch,
  levels,
  over,
  parse,
  ratioText,
  textOn,
  toHex,
  toHsl,
  toHslString,
  toOklch,
  toOklchString,
  toRgbString,
  type Rgb,
} from "./logic.js";

const hex = (text: string) => {
  const color = parse(text);
  return color ? toHex(color) : null;
};
const rgb = (r: number, g: number, b: number, a = 1): Rgb => ({ r, g, b, a });

describe("color tools", () => {
  it("reads hex, rgb(), hsl(), oklch() and names", () => {
    expect(hex("#f80")).toBe("#ff8800");
    expect(hex("F80")).toBe("#ff8800");
    expect(hex("#FF8800AA")).toBe("#ff8800aa");
    expect(hex("#f80a")).toBe("#ff8800aa");
    expect(hex("rgb(255, 0, 0)")).toBe("#ff0000");
    expect(hex("rgb(255 0 0 / 50%)")).toBe("#ff000080");
    expect(hex("rgba(0,0,255,.5)")).toBe("#0000ff80");
    expect(hex("rgb(100% 50% 0%)")).toBe("#ff8000");
    expect(hex("hsl(120 100% 25%)")).toBe("#008000");
    expect(hex("hsl(120deg, 100%, 25%)")).toBe("#008000");
    expect(hex("hsla(0.5turn 100% 50% / 1)")).toBe("#00ffff");
    expect(hex("oklch(62.8% 0.2577 29.23)")).toBe("#ff0000");
    expect(hex("oklch(0.628 0.2577 29.23)")).toBe("#ff0000");
    expect(hex("Red")).toBe("#ff0000");
    expect(hex(" rebeccapurple ")).toBe("#663399");
    for (const bad of ["nope", "#12", "#12345", "rgb(1, 2)", "rgb(1 2 3 4 5)", "hsl(x 1% 1%)", "rgb(1 2 3 / 1 / 2)", ""]) {
      expect(parse(bad), bad).toBeNull();
    }
  });

  it("converts to RGB, HSL and OKLCH as CSS writes them", () => {
    const color = rgb(51, 102, 153);
    expect(toRgbString(color)).toBe("rgb(51 102 153)");
    const hsl = toHsl(color);
    expect(hsl.h).toBeCloseTo(210, 9);
    expect(hsl.s).toBeCloseTo(50, 9);
    expect(hsl.l).toBeCloseTo(40, 9);
    expect(toHslString(color)).toBe("hsl(210 50% 40%)");
    expect(toHslString(rgb(255, 0, 0, 0.5))).toBe("hsl(0 100% 50% / 0.5)");
    expect(toRgbString(rgb(255, 0, 0, 0.25))).toBe("rgb(255 0 0 / 0.25)");
    // Björn Ottosson's reference values.
    const red = toOklch(rgb(255, 0, 0));
    expect(red.l).toBeCloseTo(0.62796, 4);
    expect(red.c).toBeCloseTo(0.25768, 4);
    expect(red.h).toBeCloseTo(29.2339, 2);
    expect(toOklchString(rgb(255, 0, 0))).toBe("oklch(0.628 0.258 29.23)");
    expect(toOklchString(rgb(255, 255, 255))).toBe("oklch(1 0 0)");
    expect(toOklchString(rgb(0, 0, 0))).toBe("oklch(0 0 0)");
    expect(toOklch(rgb(0, 0, 255)).h).toBeCloseTo(264.05, 1);
  });

  it("round-trips through HSL and OKLCH", () => {
    for (const color of [rgb(51, 102, 153), rgb(255, 136, 0), rgb(12, 200, 99), rgb(128, 128, 128), rgb(0, 0, 0), rgb(255, 255, 255)]) {
      expect(toHex(fromHsl(toHsl(color)))).toBe(toHex(color));
      expect(toHex(fromOklch(toOklch(color)))).toBe(toHex(color));
    }
  });

  it("fits colors outside sRGB by lowering chroma, keeping lightness", () => {
    const vivid = fromOklch({ l: 0.7, c: 0.4, h: 150 });
    for (const channel of [vivid.r, vivid.g, vivid.b]) {
      expect(channel).toBeGreaterThanOrEqual(0);
      expect(channel).toBeLessThanOrEqual(255);
    }
    expect(toOklch(vivid).l).toBeCloseTo(0.7, 2);
    expect(toOklch(vivid).c).toBeLessThan(0.4);
  });

  it("measures contrast the WCAG way", () => {
    expect(contrast(rgb(0, 0, 0), rgb(255, 255, 255))).toBeCloseTo(21, 5);
    expect(contrast(rgb(255, 255, 255), rgb(255, 255, 255))).toBe(1);
    expect(ratioText(contrast(rgb(119, 119, 119), rgb(255, 255, 255)))).toBe("4.47:1");
    expect(ratioText(contrast(rgb(118, 118, 118), rgb(255, 255, 255)))).toBe("4.54:1");
    // Order doesn't matter for opaque colors.
    expect(contrast(rgb(51, 102, 153), rgb(240, 240, 240))).toBeCloseTo(contrast(rgb(240, 240, 240), rgb(51, 102, 153)), 10);
    // See-through text is painted over its background first.
    expect(contrast(rgb(0, 0, 0, 0.5), rgb(255, 255, 255))).toBeCloseTo(contrast(rgb(127.5, 127.5, 127.5), rgb(255, 255, 255)), 5);
    expect(over(rgb(0, 0, 0, 0), rgb(10, 20, 30))).toEqual(rgb(10, 20, 30));
  });

  it("names the levels a ratio passes", () => {
    expect(levels(21)).toEqual({ aa: true, aaLarge: true, aaa: true, aaaLarge: true });
    expect(levels(4.5)).toEqual({ aa: true, aaLarge: true, aaa: false, aaaLarge: true });
    expect(levels(3.2)).toEqual({ aa: false, aaLarge: true, aaa: false, aaaLarge: false });
    expect(levels(2)).toEqual({ aa: false, aaLarge: false, aaa: false, aaaLarge: false });
    expect(ratioText(4.4999)).toBe("4.49:1");
  });

  it("picks black or white text for a swatch", () => {
    expect(textOn(rgb(255, 255, 0))).toBe("#000000");
    expect(textOn(rgb(20, 30, 90))).toBe("#ffffff");
  });
});
