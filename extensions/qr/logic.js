// @ts-check
// A QR code encoder, pure and written for Little Joan from the QR code standard (ISO/IEC 18004):
// byte mode (UTF-8), versions 1–10, error correction level M (about 15 % of the code can be lost
// and it still reads). encode() picks the smallest version that fits, lays out the function
// patterns, the data and its Reed–Solomon error correction, and the mask with the lowest penalty.
// logic.test.ts compares it with reference matrices.

/**
 * @typedef {object} Qr
 * @property {number} version 1–10
 * @property {number} size modules per side (4 × version + 17)
 * @property {number} mask 0–7
 * @property {boolean[][]} modules [y][x], true = dark
 */

export const MAX_VERSION = 10;
/** Error-correction codewords per block at level M, by version (index 0 unused). */
const ECC_PER_BLOCK = [0, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26];
/** Blocks at level M, by version. */
const BLOCKS = [0, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5];
/** Level M's two format bits. */
const LEVEL_M = 0;

/**
 * Modules a version has for data and error correction (everything but the function patterns).
 * @param {number} version
 */
function rawModules(version) {
  let modules = (16 * version + 128) * version + 64;
  if (version >= 2) {
    const align = Math.floor(version / 7) + 2;
    modules -= (25 * align - 10) * align - 55;
    if (version >= 7) {
      modules -= 36;
    }
  }
  return modules;
}

/**
 * Data codewords a version holds at level M.
 * @param {number} version
 */
export function dataCodewords(version) {
  return Math.floor(rawModules(version) / 8) - (ECC_PER_BLOCK[version] ?? 0) * (BLOCKS[version] ?? 0);
}

/**
 * The most bytes a version holds in byte mode at level M (4 mode bits, 8 or 16 count bits).
 * @param {number} version
 */
export function byteCapacity(version) {
  return Math.floor((dataCodewords(version) * 8 - 4 - (version <= 9 ? 8 : 16)) / 8);
}

/**
 * Multiplication in GF(2^8) modulo x^8 + x^4 + x^3 + x^2 + 1.
 * @param {number} x
 * @param {number} y
 */
function gfMultiply(x, y) {
  let z = 0;
  for (let i = 7; i >= 0; i -= 1) {
    z = (z << 1) ^ ((z >>> 7) * 0x11d);
    z ^= ((y >>> i) & 1) * x;
  }
  return z & 0xff;
}

/**
 * The Reed–Solomon generator polynomial of a degree (its coefficients, highest first, the leading
 * 1 left out).
 * @param {number} degree
 */
function rsGenerator(degree) {
  const result = Array.from({ length: degree }, () => 0);
  result[degree - 1] = 1;
  let root = 1;
  for (let i = 0; i < degree; i += 1) {
    for (let j = 0; j < degree; j += 1) {
      result[j] = gfMultiply(result[j] ?? 0, root);
      if (j + 1 < degree) {
        result[j] = (result[j] ?? 0) ^ (result[j + 1] ?? 0);
      }
    }
    root = gfMultiply(root, 0x02);
  }
  return result;
}

/**
 * The error-correction codewords of a block: the remainder of its data divided by the generator.
 * @param {number[]} data
 * @param {number[]} generator
 */
export function rsRemainder(data, generator) {
  const result = generator.map(() => 0);
  for (const byte of data) {
    const factor = byte ^ (result.shift() ?? 0);
    result.push(0);
    generator.forEach((coefficient, i) => {
      result[i] = (result[i] ?? 0) ^ gfMultiply(coefficient, factor);
    });
  }
  return result;
}

/**
 * The data codewords for `bytes` in byte mode: mode, count, the bytes, a terminator, padding.
 * @param {Uint8Array} bytes
 * @param {number} version
 */
function dataFor(bytes, version) {
  /** @type {number[]} */
  const bits = [];
  const push = (/** @type {number} */ value, /** @type {number} */ length) => {
    for (let i = length - 1; i >= 0; i -= 1) {
      bits.push((value >>> i) & 1);
    }
  };
  push(0b0100, 4);
  push(bytes.length, version <= 9 ? 8 : 16);
  for (const byte of bytes) {
    push(byte, 8);
  }
  const capacity = dataCodewords(version) * 8;
  push(0, Math.min(4, capacity - bits.length));
  push(0, (8 - (bits.length % 8)) % 8);
  for (let pad = 0xec; bits.length < capacity; pad ^= 0xec ^ 0x11) {
    push(pad, 8);
  }
  /** @type {number[]} */
  const codewords = [];
  for (let i = 0; i < bits.length; i += 8) {
    codewords.push(bits.slice(i, i + 8).reduce((byte, bit) => (byte << 1) | bit, 0));
  }
  return codewords;
}

/**
 * Splits the data into blocks, adds each block's error correction and interleaves them.
 * @param {number[]} data
 * @param {number} version
 */
export function withErrorCorrection(data, version) {
  const blocks = BLOCKS[version] ?? 1;
  const eccLength = ECC_PER_BLOCK[version] ?? 0;
  const raw = Math.floor(rawModules(version) / 8);
  const shortBlocks = blocks - (raw % blocks);
  const shortLength = Math.floor(raw / blocks);
  const generator = rsGenerator(eccLength);
  /** @type {number[][]} */
  const all = [];
  for (let i = 0, k = 0; i < blocks; i += 1) {
    const part = data.slice(k, k + shortLength - eccLength + (i < shortBlocks ? 0 : 1));
    k += part.length;
    const ecc = rsRemainder(part, generator);
    if (i < shortBlocks) {
      part.push(0);
    }
    all.push([...part, ...ecc]);
  }
  /** @type {number[]} */
  const result = [];
  for (let i = 0; i < (all[0]?.length ?? 0); i += 1) {
    all.forEach((block, j) => {
      if (i !== shortLength - eccLength || j >= shortBlocks) {
        result.push(block[i] ?? 0);
      }
    });
  }
  return result;
}

/**
 * Where a version's alignment patterns are centered (each coordinate, on both axes).
 * @param {number} version
 */
export function alignmentPositions(version) {
  if (version === 1) {
    return [];
  }
  const count = Math.floor(version / 7) + 2;
  const step = Math.ceil((version * 4 + 4) / (count * 2 - 2)) * 2;
  const result = [6];
  for (let position = version * 4 + 17 - 7; result.length < count; position -= step) {
    result.splice(1, 0, position);
  }
  return result;
}

/**
 * The 15 format bits for level M and a mask (BCH-coded, XOR 0x5412).
 * @param {number} mask
 */
export function formatBits(mask) {
  const data = (LEVEL_M << 3) | mask;
  let remainder = data;
  for (let i = 0; i < 10; i += 1) {
    remainder = (remainder << 1) ^ ((remainder >>> 9) * 0x537);
  }
  return ((data << 10) | remainder) ^ 0x5412;
}

/**
 * The 18 version bits (versions 7 and up).
 * @param {number} version
 */
export function versionBits(version) {
  let remainder = version;
  for (let i = 0; i < 12; i += 1) {
    remainder = (remainder << 1) ^ ((remainder >>> 11) * 0x1f25);
  }
  return (version << 12) | remainder;
}

/** @type {((x: number, y: number) => boolean)[]} */
const MASKS = [
  (x, y) => (x + y) % 2 === 0,
  (_x, y) => y % 2 === 0,
  (x) => x % 3 === 0,
  (x, y) => (x + y) % 3 === 0,
  (x, y) => (Math.floor(x / 3) + Math.floor(y / 2)) % 2 === 0,
  (x, y) => ((x * y) % 2) + ((x * y) % 3) === 0,
  (x, y) => (((x * y) % 2) + ((x * y) % 3)) % 2 === 0,
  (x, y) => (((x + y) % 2) + ((x * y) % 3)) % 2 === 0,
];

/**
 * A matrix with everything but the data: finders, separators, timing, alignment, the dark module
 * and the version bits, plus room for the format bits. `isFunction` marks those modules.
 * @param {number} version
 */
function functionPatterns(version) {
  const size = version * 4 + 17;
  const modules = Array.from({ length: size }, () => Array.from({ length: size }, () => false));
  const isFunction = Array.from({ length: size }, () => Array.from({ length: size }, () => false));
  const set = (/** @type {number} */ x, /** @type {number} */ y, /** @type {boolean} */ dark) => {
    const row = modules[y];
    const marks = isFunction[y];
    if (row && marks && x >= 0 && x < size) {
      row[x] = dark;
      marks[x] = true;
    }
  };
  for (let i = 0; i < size; i += 1) {
    set(6, i, i % 2 === 0);
    set(i, 6, i % 2 === 0);
  }
  for (const [cx, cy] of [
    [3, 3],
    [size - 4, 3],
    [3, size - 4],
  ]) {
    for (let dy = -4; dy <= 4; dy += 1) {
      for (let dx = -4; dx <= 4; dx += 1) {
        const distance = Math.max(Math.abs(dx), Math.abs(dy));
        const x = (cx ?? 0) + dx;
        const y = (cy ?? 0) + dy;
        if (y >= 0 && y < size) {
          set(x, y, distance !== 2 && distance !== 4);
        }
      }
    }
  }
  const centers = alignmentPositions(version);
  const last = centers.length - 1;
  centers.forEach((cx, i) => {
    centers.forEach((cy, j) => {
      const corner = (i === 0 && j === 0) || (i === 0 && j === last) || (i === last && j === 0);
      if (!corner) {
        for (let dy = -2; dy <= 2; dy += 1) {
          for (let dx = -2; dx <= 2; dx += 1) {
            set(cx + dx, cy + dy, Math.max(Math.abs(dx), Math.abs(dy)) !== 1);
          }
        }
      }
    });
  });
  // Room for the format bits (drawn per mask), and the dark module.
  placeFormat(set, size, 0);
  if (version >= 7) {
    const bits = versionBits(version);
    for (let i = 0; i < 18; i += 1) {
      const dark = ((bits >>> i) & 1) === 1;
      const a = size - 11 + (i % 3);
      const b = Math.floor(i / 3);
      set(a, b, dark);
      set(b, a, dark);
    }
  }
  return { size, modules, isFunction };
}

/**
 * Draws the format bits for `mask` (both copies) and the dark module.
 * @param {(x: number, y: number, dark: boolean) => void} set
 * @param {number} size
 * @param {number} mask
 */
function placeFormat(set, size, mask) {
  const bits = formatBits(mask);
  const bit = (/** @type {number} */ i) => ((bits >>> i) & 1) === 1;
  for (let i = 0; i <= 5; i += 1) {
    set(8, i, bit(i));
  }
  set(8, 7, bit(6));
  set(8, 8, bit(7));
  set(7, 8, bit(8));
  for (let i = 9; i < 15; i += 1) {
    set(14 - i, 8, bit(i));
  }
  for (let i = 0; i < 8; i += 1) {
    set(size - 1 - i, 8, bit(i));
  }
  for (let i = 8; i < 15; i += 1) {
    set(8, size - 15 + i, bit(i));
  }
  set(8, size - 8, true);
}

/**
 * Puts the codewords' bits in the zigzag order of the standard, around the function patterns.
 * @param {boolean[][]} modules
 * @param {boolean[][]} isFunction
 * @param {number[]} codewords
 */
function placeData(modules, isFunction, codewords) {
  const size = modules.length;
  let i = 0;
  for (let right = size - 1; right >= 1; right -= 2) {
    if (right === 6) {
      right = 5;
    }
    for (let vertical = 0; vertical < size; vertical += 1) {
      for (let j = 0; j < 2; j += 1) {
        const x = right - j;
        const upward = ((right + 1) & 2) === 0;
        const y = upward ? size - 1 - vertical : vertical;
        const row = modules[y];
        if (row && !isFunction[y]?.[x] && i < codewords.length * 8) {
          row[x] = (((codewords[i >>> 3] ?? 0) >>> (7 - (i & 7))) & 1) === 1;
          i += 1;
        }
      }
    }
  }
}

/**
 * The standard's penalty for a finished matrix (lower is easier to read): runs of five or more,
 * 2 × 2 blocks, finder-like patterns, and an uneven share of dark modules.
 * @param {boolean[][]} modules
 */
export function penalty(modules) {
  const size = modules.length;
  let score = 0;
  /** @param {number[]} history */
  const patterns = (history) => {
    const n = history[1] ?? 0;
    const core = n > 0 && history[2] === n && history[3] === n * 3 && history[4] === n && history[5] === n;
    return (core && (history[0] ?? 0) >= n * 4 && (history[6] ?? 0) >= n ? 1 : 0) + (core && (history[6] ?? 0) >= n * 4 && (history[0] ?? 0) >= n ? 1 : 0);
  };
  /**
   * @param {number} run
   * @param {number[]} history
   */
  const addHistory = (run, history) => {
    history.pop();
    history.unshift(history[0] === 0 ? run + size : run);
  };
  /** @param {(a: number, b: number) => boolean} at */
  const lines = (at) => {
    for (let a = 0; a < size; a += 1) {
      let color = false;
      let run = 0;
      const history = [0, 0, 0, 0, 0, 0, 0];
      for (let b = 0; b < size; b += 1) {
        if (at(a, b) === color) {
          run += 1;
          if (run === 5) {
            score += 3;
          } else if (run > 5) {
            score += 1;
          }
        } else {
          addHistory(run, history);
          if (!color) {
            score += patterns(history) * 40;
          }
          color = at(a, b);
          run = 1;
        }
      }
      // The light border after the line.
      if (color) {
        addHistory(run, history);
        run = 0;
      }
      addHistory(run + size, history);
      score += patterns(history) * 40;
    }
  };
  lines((y, x) => modules[y]?.[x] === true);
  lines((x, y) => modules[y]?.[x] === true);
  let dark = 0;
  for (let y = 0; y < size; y += 1) {
    for (let x = 0; x < size; x += 1) {
      const color = modules[y]?.[x] === true;
      if (color) {
        dark += 1;
      }
      if (x < size - 1 && y < size - 1 && color === (modules[y]?.[x + 1] === true) && color === (modules[y + 1]?.[x] === true) && color === (modules[y + 1]?.[x + 1] === true)) {
        score += 3;
      }
    }
  }
  const total = size * size;
  score += (Math.ceil(Math.abs(dark * 20 - total * 10) / total) - 1) * 10;
  return score;
}

/**
 * The QR code of `text` (UTF-8, byte mode, level M), or null when it needs more than version 10
 * (`byteCapacity(10)` bytes). `forceMask` picks a mask instead of the best one (for tests).
 * @param {string} text
 * @param {number} [forceMask]
 * @returns {Qr | null}
 */
export function encode(text, forceMask) {
  const bytes = new TextEncoder().encode(text);
  let version = 1;
  while (version <= MAX_VERSION && bytes.length > byteCapacity(version)) {
    version += 1;
  }
  if (version > MAX_VERSION) {
    return null;
  }
  const codewords = withErrorCorrection(dataFor(bytes, version), version);
  /** @type {Qr | null} */
  let best = null;
  let bestScore = Infinity;
  for (let mask = 0; mask < 8; mask += 1) {
    if (forceMask !== undefined && mask !== forceMask) {
      continue;
    }
    const { size, modules, isFunction } = functionPatterns(version);
    placeData(modules, isFunction, codewords);
    const flip = MASKS[mask] ?? (() => false);
    for (let y = 0; y < size; y += 1) {
      const row = modules[y];
      for (let x = 0; row && x < size; x += 1) {
        if (!isFunction[y]?.[x] && flip(x, y)) {
          row[x] = !row[x];
        }
      }
    }
    placeFormat(
      (x, y, dark) => {
        const row = modules[y];
        if (row) {
          row[x] = dark;
        }
      },
      size,
      mask,
    );
    const score = penalty(modules);
    if (score < bestScore) {
      best = { version, size, mask, modules };
      bestScore = score;
    }
  }
  return best;
}

/**
 * An SVG path drawing the dark modules (one `h1v1h-1z` square each, merged per row run), offset
 * by `border` modules of quiet zone.
 * @param {Qr} qr
 * @param {number} [border]
 */
export function svgPath(qr, border = 4) {
  /** @type {string[]} */
  const parts = [];
  qr.modules.forEach((row, y) => {
    let x = 0;
    while (x < qr.size) {
      if (!row[x]) {
        x += 1;
        continue;
      }
      const start = x;
      while (x < qr.size && row[x]) {
        x += 1;
      }
      parts.push(`M${start + border} ${y + border}h${x - start}v1h-${x - start}z`);
    }
  });
  return parts.join("");
}

/**
 * What the panel says about a text that is too long for a code.
 * @param {string} text
 */
export function tooLong(text) {
  const bytes = new TextEncoder().encode(text).length;
  return `This is ${bytes} bytes long; a QR code here holds at most ${byteCapacity(MAX_VERSION)}.`;
}
