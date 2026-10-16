import { inflateSync } from "node:zlib";
import { describe, expect, it } from "vitest";
import { appIconSample, circle, render, roundedRect, trayTemplateSample } from "./icon-geometry.mts";
import { crc32, encodePng } from "./png.mts";

function alphaAt(pixels: Uint8Array, size: number, x: number, y: number): number {
  return pixels[(y * size + x) * 4 + 3] ?? -1;
}

describe("shapes", () => {
  it("rounded rect is negative inside and positive outside", () => {
    expect(roundedRect(0.5, 0.5, 0.5, 0.5, 0.2, 0.2, 0.05)).toBeLessThan(0);
    expect(roundedRect(0.9, 0.5, 0.5, 0.5, 0.2, 0.2, 0.05)).toBeGreaterThan(0);
    // The rounded corner cuts the square's corner point off.
    expect(roundedRect(0.7, 0.7, 0.5, 0.5, 0.2, 0.2, 0.05)).toBeGreaterThan(0);
  });

  it("circle distance is zero on the rim", () => {
    expect(circle(0.6, 0.5, 0.5, 0.5, 0.1)).toBeCloseTo(0);
  });
});

describe("app icon", () => {
  const size = 64;
  const pixels = render(size, appIconSample);

  it("has transparent corners and an opaque centre", () => {
    expect(alphaAt(pixels, size, 0, 0)).toBe(0);
    expect(alphaAt(pixels, size, size - 1, size - 1)).toBe(0);
    expect(alphaAt(pixels, size, size / 2, size / 2)).toBe(255);
  });
});

describe("tray template", () => {
  const size = 32;
  const pixels = render(size, trayTemplateSample, 8);

  it("is pure black wherever it is visible, so macOS can tint it", () => {
    let visible = 0;
    for (let i = 0; i < pixels.length; i += 4) {
      if ((pixels[i + 3] ?? 0) > 0) {
        visible++;
        expect([pixels[i], pixels[i + 1], pixels[i + 2]]).toEqual([0, 0, 0]);
      }
    }
    expect(visible).toBeGreaterThan(size * 2);
  });

  it("leaves the inside of the window outline empty", () => {
    expect(alphaAt(pixels, size, Math.round(0.42 * size), Math.round(0.68 * size))).toBe(0);
  });
});

describe("png encoder", () => {
  it("computes the standard CRC-32", () => {
    expect(crc32(new TextEncoder().encode("123456789"))).toBe(0xcbf43926);
  });

  it("writes a decodable RGBA image", () => {
    const rgba = new Uint8Array([255, 0, 0, 255, 0, 255, 0, 128]);
    const png = encodePng(2, 1, rgba);
    expect([...png.subarray(0, 8)]).toEqual([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
    expect(png.readUInt32BE(16)).toBe(2); // IHDR width
    expect(png.readUInt32BE(20)).toBe(1); // IHDR height
    const idatLength = png.readUInt32BE(33);
    expect(png.toString("ascii", 37, 41)).toBe("IDAT");
    const raw = inflateSync(png.subarray(41, 41 + idatLength));
    expect([...raw]).toEqual([0, ...rgba]);
  });

  it("rejects a buffer of the wrong size", () => {
    expect(() => encodePng(2, 2, new Uint8Array(4))).toThrow();
  });
});
