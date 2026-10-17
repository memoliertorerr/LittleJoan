// Original Little Joan artwork, drawn from simple shapes: a small browser window on a night-blue
// tile with a crescent moon (the app sleeps until you call it). Pure functions: shapes are signed
// distance functions in unit coordinates (0..1), rendered with supersampling.

export type Rgba = readonly [number, number, number, number];

/** Signed distance to a rounded rectangle centred at (cx, cy); negative inside. */
export function roundedRect(
  x: number,
  y: number,
  cx: number,
  cy: number,
  halfW: number,
  halfH: number,
  radius: number,
): number {
  const qx = Math.abs(x - cx) - halfW + radius;
  const qy = Math.abs(y - cy) - halfH + radius;
  const outside = Math.hypot(Math.max(qx, 0), Math.max(qy, 0));
  const inside = Math.min(Math.max(qx, qy), 0);
  return outside + inside - radius;
}

/** Signed distance to a circle; negative inside. */
export function circle(x: number, y: number, cx: number, cy: number, r: number): number {
  return Math.hypot(x - cx, y - cy) - r;
}

/** A crescent: one circle minus another (both given as [cx, cy, r]). */
export function crescent(
  x: number,
  y: number,
  outer: readonly [number, number, number],
  bite: readonly [number, number, number],
): number {
  return Math.max(circle(x, y, ...outer), -circle(x, y, ...bite));
}

/** The colour of one sample point of the app icon, or null when transparent. */
export function appIconSample(x: number, y: number): Rgba | null {
  // Crescent moon, drawn last so it sits on top of the window's corner.
  if (crescent(x, y, [0.73, 0.29, 0.135], [0.785, 0.245, 0.115]) <= 0) {
    return [247, 201, 72, 255];
  }
  const tile = roundedRect(x, y, 0.5, 0.5, 0.41, 0.41, 0.19);
  if (tile > 0) {
    return null;
  }
  const windowBody = roundedRect(x, y, 0.47, 0.56, 0.255, 0.205, 0.05);
  if (windowBody <= 0) {
    const titleBarBottom = 0.56 - 0.205 + 0.085;
    if (y < titleBarBottom) {
      return [205, 214, 232, 255];
    }
    if (roundedRect(x, y, 0.47, 0.515, 0.185, 0.026, 0.026) <= 0) {
      return [196, 205, 223, 255];
    }
    return [245, 247, 251, 255];
  }
  // Night-blue tile with a gentle vertical gradient.
  const t = Math.min(Math.max((y - 0.09) / 0.82, 0), 1);
  return [Math.round(43 - 12 * t), Math.round(58 - 16 * t), Math.round(95 - 27 * t), 255];
}

/** The tray glyph: black on transparent, so macOS can tint it as a template image. */
export function trayTemplateSample(x: number, y: number): Rgba | null {
  const black: Rgba = [0, 0, 0, 255];
  const moon = crescent(x, y, [0.78, 0.24, 0.2], [0.87, 0.165, 0.17]);
  if (moon <= 0) {
    return black;
  }
  // A thin transparent halo keeps the moon from merging with the window outline.
  if (moon <= 0.075) {
    return null;
  }
  const outer = roundedRect(x, y, 0.42, 0.6, 0.36, 0.3, 0.1);
  const inner = roundedRect(x, y, 0.42, 0.6, 0.36 - 0.085, 0.3 - 0.085, 0.04);
  if (outer <= 0 && inner > 0) {
    return black;
  }
  // Solid title bar across the top of the window outline.
  if (outer <= 0 && y < 0.6 - 0.3 + 0.19) {
    return black;
  }
  return null;
}

/**
 * Renders `sample` into a size×size RGBA buffer with grid×grid supersampling per pixel.
 * Colours are averaged with alpha weighting so edges blend into transparency cleanly.
 */
export function render(
  size: number,
  sample: (x: number, y: number) => Rgba | null,
  grid = 4,
): Uint8Array {
  const out = new Uint8Array(size * size * 4);
  const n = grid * grid;
  for (let py = 0; py < size; py++) {
    for (let px = 0; px < size; px++) {
      let r = 0;
      let g = 0;
      let b = 0;
      let a = 0;
      for (let sy = 0; sy < grid; sy++) {
        for (let sx = 0; sx < grid; sx++) {
          const c = sample((px + (sx + 0.5) / grid) / size, (py + (sy + 0.5) / grid) / size);
          if (c) {
            const alpha = c[3] / 255;
            r += c[0] * alpha;
            g += c[1] * alpha;
            b += c[2] * alpha;
            a += alpha;
          }
        }
      }
      const i = (py * size + px) * 4;
      if (a > 0) {
        out[i] = Math.round(r / a);
        out[i + 1] = Math.round(g / a);
        out[i + 2] = Math.round(b / a);
        out[i + 3] = Math.round((a / n) * 255);
      }
    }
  }
  return out;
}
