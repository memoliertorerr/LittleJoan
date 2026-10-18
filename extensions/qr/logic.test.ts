import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  MAX_VERSION,
  alignmentPositions,
  byteCapacity,
  dataCodewords,
  encode,
  formatBits,
  penalty,
  rsRemainder,
  svgPath,
  tooLong,
  versionBits,
  withErrorCorrection,
} from "./logic.js";

/**
 * Reference matrices from another encoder: macOS Core Image's CIQRCodeGenerator at level M (made
 * on 2026-10-18; rows of 0/1, the quiet zone left out), with the mask Core Image chose. Every code
 * ours makes for these texts was also read back with Core Image's QR detector.
 */
const REFERENCES = JSON.parse(readFileSync(new URL("./reference.test-data.json", import.meta.url), "utf8")) as {
  text: string;
  mask: number;
  rows: string[];
}[];

const rows = (modules: boolean[][]) => modules.map((row) => row.map((dark) => (dark ? "1" : "0")).join(""));

describe("QR code encoder", () => {
  it("knows each version's room at level M", () => {
    expect([1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map(dataCodewords)).toEqual([16, 28, 44, 64, 86, 108, 124, 154, 182, 216]);
    expect([1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map(byteCapacity)).toEqual([14, 26, 42, 62, 84, 106, 122, 152, 180, 213]);
  });

  it("matches the reference matrices, versions 1 to 10", () => {
    const versions = new Set<number>();
    for (const reference of REFERENCES) {
      const qr = encode(reference.text);
      expect(qr, reference.text).not.toBeNull();
      versions.add(qr!.version);
      expect(qr!.size).toBe(reference.rows.length);
      // Where both chose the same mask, every module matches; masks are a judgement call between
      // encoders (penalty rules), so with Core Image's mask the matrices must match too.
      if (qr!.mask === reference.mask) {
        expect(rows(qr!.modules), reference.text).toEqual(reference.rows);
      }
      expect(rows(encode(reference.text, reference.mask)!.modules), `${reference.text} (mask ${reference.mask})`).toEqual(reference.rows);
    }
    expect([...versions].sort((a, b) => a - b)).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    expect(REFERENCES.filter((r) => encode(r.text)?.mask === r.mask).length).toBeGreaterThanOrEqual(10);
  });

  it("encodes UTF-8 in byte mode and refuses what doesn't fit version 10", () => {
    const text = "café ☕";
    expect(encode(text)?.version).toBe(1);
    expect(encode("x".repeat(byteCapacity(MAX_VERSION)))?.version).toBe(10);
    expect(encode("x".repeat(byteCapacity(MAX_VERSION) + 1))).toBeNull();
    // Multi-byte characters count as their bytes.
    expect(encode("é".repeat(107))).toBeNull();
    expect(tooLong("x".repeat(300))).toBe("This is 300 bytes long; a QR code here holds at most 213.");
  });

  it("computes Reed–Solomon error correction", () => {
    // The standard's worked example (ISO/IEC 18004 annex I): "01234567" at 1-M.
    const data = [0x10, 0x20, 0x0c, 0x56, 0x61, 0x80, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11];
    const all = withErrorCorrection(data, 1);
    expect(all.slice(16)).toEqual([0xa5, 0x24, 0xd4, 0xc1, 0xed, 0x36, 0xc7, 0x87, 0x2c, 0x55]);
    // "HELLO WORLD" at 1-M (alphanumeric data; the error correction doesn't care how it was made).
    const hello = [0x20, 0x5b, 0x0b, 0x78, 0xd1, 0x72, 0xdc, 0x4d, 0x43, 0x40, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11];
    expect(withErrorCorrection(hello, 1).slice(16)).toEqual([0xc4, 0x23, 0x27, 0x77, 0xeb, 0xd7, 0xe7, 0xe2, 0x5d, 0x17]);
    expect(rsRemainder([], [1, 2, 3])).toEqual([0, 0, 0]);
  });

  it("places alignment patterns, format and version bits as the standard does", () => {
    expect(alignmentPositions(1)).toEqual([]);
    expect(alignmentPositions(2)).toEqual([6, 18]);
    expect(alignmentPositions(7)).toEqual([6, 22, 38]);
    expect(alignmentPositions(10)).toEqual([6, 28, 50]);
    // Level M, mask 0..7 (the standard's table of format information).
    expect([0, 1, 2, 3, 4, 5, 6, 7].map(formatBits)).toEqual([0x5412, 0x5125, 0x5e7c, 0x5b4b, 0x45f9, 0x40ce, 0x4f97, 0x4aa0]);
    expect(versionBits(7)).toBe(0x07c94);
    expect(versionBits(10)).toBe(0x0a4d3);
  });

  it("chooses the mask with the lowest penalty", () => {
    const qr = encode("https://example.com/")!;
    const chosen = penalty(qr.modules);
    for (let mask = 0; mask < 8; mask += 1) {
      expect(penalty(encode("https://example.com/", mask)!.modules)).toBeGreaterThanOrEqual(chosen);
    }
  });

  it("draws the dark modules as an SVG path with a quiet zone", () => {
    const qr = encode("hi")!;
    const path = svgPath(qr);
    // The top-left finder's first row: seven dark modules from (4, 4).
    expect(path.startsWith("M4 4h7v1h-7z")).toBe(true);
    const dark = qr.modules.flat().filter(Boolean).length;
    const drawn = [...path.matchAll(/h(\d+)v1/g)].reduce((sum, m) => sum + Number(m[1]), 0);
    expect(drawn).toBe(dark);
  });
});
