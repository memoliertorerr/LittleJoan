// Typing on CI's macOS runners (pure, unit-tested; scripts/ci/smoke.mts writes the Swift that
// posts the events): key events that carry only a Unicode string never reached the app there,
// while events with a real virtual key code did, so characters go as key codes where the US
// layout has one, and as a Unicode string otherwise.

/** macOS virtual key codes (kVK_ANSI_*, US layout) for typing; `s` = with Shift. */
const MAC_TYPE_KEYS: Record<string, string> = {
  ...Object.fromEntries([..."asdfhgzxcv"].map((c, i) => [c, String(i)])),
  b: "11",
  q: "12",
  w: "13",
  e: "14",
  r: "15",
  y: "16",
  t: "17",
  o: "31",
  u: "32",
  i: "34",
  p: "35",
  l: "37",
  j: "38",
  k: "40",
  n: "45",
  m: "46",
  "1": "18",
  "2": "19",
  "3": "20",
  "4": "21",
  "6": "22",
  "5": "23",
  "=": "24",
  "9": "25",
  "7": "26",
  "-": "27",
  "8": "28",
  "0": "29",
  ".": "47",
  "/": "44",
  ",": "43",
  " ": "49",
  "*": "28s",
  "+": "24s",
  "(": "25s",
  ")": "29s",
};

/** A character as type.swift takes it: `k<code>[s]` for a key, `u<char>` for a Unicode string. */
export function macKeyToken(c: string): string {
  const lower = c.toLowerCase();
  const code = MAC_TYPE_KEYS[c] ?? (c !== lower && /^[a-z]$/.test(lower) ? `${MAC_TYPE_KEYS[lower] ?? ""}s` : undefined);
  return code && code !== "s" ? `k${code}` : `u${c}`;
}

