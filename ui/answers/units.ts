// The unit-conversion quick answer (pure, unit-tested): "<number> <unit> in|to|as <unit>" for
// length, mass, temperature, volume, speed and data sizes ("10 km in mi", "72f to c",
// "1.5 GiB in MB"). Units are matched by symbol or name, in any case (data sizes keep "b" as bytes;
// bits aren't converted).
import { formatNumber } from "./calc.ts";
import type { Answer } from "./types.ts";

type Kind = "length" | "mass" | "temperature" | "volume" | "speed" | "data";

interface Unit {
  kind: Kind;
  /** Shown in the answer. */
  symbol: string;
  /** How many base units one of it is (meters, grams, liters, m/s, bytes); temperatures convert on their own. */
  factor: number;
  names: string[];
}

const UNITS: Unit[] = [
  // Length, in meters.
  { kind: "length", symbol: "mm", factor: 0.001, names: ["mm", "millimeter", "millimeters", "millimetre", "millimetres"] },
  { kind: "length", symbol: "cm", factor: 0.01, names: ["cm", "centimeter", "centimeters", "centimetre", "centimetres"] },
  { kind: "length", symbol: "m", factor: 1, names: ["m", "meter", "meters", "metre", "metres"] },
  { kind: "length", symbol: "km", factor: 1000, names: ["km", "kilometer", "kilometers", "kilometre", "kilometres"] },
  { kind: "length", symbol: "in", factor: 0.0254, names: ["in", "inch", "inches", '"'] },
  { kind: "length", symbol: "ft", factor: 0.3048, names: ["ft", "foot", "feet", "'"] },
  { kind: "length", symbol: "yd", factor: 0.9144, names: ["yd", "yard", "yards"] },
  { kind: "length", symbol: "mi", factor: 1609.344, names: ["mi", "mile", "miles"] },
  { kind: "length", symbol: "nmi", factor: 1852, names: ["nmi", "nautical mile", "nautical miles"] },
  // Mass, in grams.
  { kind: "mass", symbol: "mg", factor: 0.001, names: ["mg", "milligram", "milligrams"] },
  { kind: "mass", symbol: "g", factor: 1, names: ["g", "gram", "grams", "gramme", "grammes"] },
  { kind: "mass", symbol: "kg", factor: 1000, names: ["kg", "kilo", "kilos", "kilogram", "kilograms"] },
  { kind: "mass", symbol: "t", factor: 1_000_000, names: ["t", "tonne", "tonnes", "metric ton", "metric tons"] },
  { kind: "mass", symbol: "oz", factor: 28.349523125, names: ["oz", "ounce", "ounces"] },
  { kind: "mass", symbol: "lb", factor: 453.59237, names: ["lb", "lbs", "pound", "pounds"] },
  { kind: "mass", symbol: "st", factor: 6350.29318, names: ["st", "stone", "stones"] },
  // Temperature (factor unused).
  { kind: "temperature", symbol: "°C", factor: 1, names: ["c", "°c", "celsius", "degc"] },
  { kind: "temperature", symbol: "°F", factor: 1, names: ["f", "°f", "fahrenheit", "degf"] },
  { kind: "temperature", symbol: "K", factor: 1, names: ["k", "kelvin"] },
  // Volume, in liters (US customary for cups, pints, quarts, gallons, fluid ounces).
  { kind: "volume", symbol: "ml", factor: 0.001, names: ["ml", "milliliter", "milliliters", "millilitre", "millilitres"] },
  { kind: "volume", symbol: "l", factor: 1, names: ["l", "liter", "liters", "litre", "litres"] },
  { kind: "volume", symbol: "tsp", factor: 0.00492892159375, names: ["tsp", "teaspoon", "teaspoons"] },
  { kind: "volume", symbol: "tbsp", factor: 0.01478676478125, names: ["tbsp", "tablespoon", "tablespoons"] },
  { kind: "volume", symbol: "fl oz", factor: 0.0295735295625, names: ["fl oz", "floz", "fluid ounce", "fluid ounces"] },
  { kind: "volume", symbol: "cup", factor: 0.2365882365, names: ["cup", "cups"] },
  { kind: "volume", symbol: "pt", factor: 0.473176473, names: ["pt", "pint", "pints"] },
  { kind: "volume", symbol: "qt", factor: 0.946352946, names: ["qt", "quart", "quarts"] },
  { kind: "volume", symbol: "gal", factor: 3.785411784, names: ["gal", "gallon", "gallons"] },
  // Speed, in meters per second.
  { kind: "speed", symbol: "m/s", factor: 1, names: ["m/s", "mps", "meters per second", "metres per second"] },
  { kind: "speed", symbol: "km/h", factor: 1 / 3.6, names: ["km/h", "kmh", "kph", "kmph", "kilometers per hour", "kilometres per hour"] },
  { kind: "speed", symbol: "mph", factor: 0.44704, names: ["mph", "mi/h", "miles per hour"] },
  { kind: "speed", symbol: "kn", factor: 1852 / 3600, names: ["kn", "kt", "kts", "knot", "knots"] },
  // Data sizes, in bytes: decimal (kB = 1000 B) and binary (KiB = 1024 B).
  { kind: "data", symbol: "B", factor: 1, names: ["b", "byte", "bytes"] },
  { kind: "data", symbol: "kB", factor: 1e3, names: ["kb", "kilobyte", "kilobytes"] },
  { kind: "data", symbol: "MB", factor: 1e6, names: ["mb", "megabyte", "megabytes"] },
  { kind: "data", symbol: "GB", factor: 1e9, names: ["gb", "gigabyte", "gigabytes"] },
  { kind: "data", symbol: "TB", factor: 1e12, names: ["tb", "terabyte", "terabytes"] },
  { kind: "data", symbol: "PB", factor: 1e15, names: ["pb", "petabyte", "petabytes"] },
  { kind: "data", symbol: "KiB", factor: 1024, names: ["kib", "kibibyte", "kibibytes"] },
  { kind: "data", symbol: "MiB", factor: 1024 ** 2, names: ["mib", "mebibyte", "mebibytes"] },
  { kind: "data", symbol: "GiB", factor: 1024 ** 3, names: ["gib", "gibibyte", "gibibytes"] },
  { kind: "data", symbol: "TiB", factor: 1024 ** 4, names: ["tib", "tebibyte", "tebibytes"] },
];

/** The unit called `name` (any case, `°` and spaces as written). */
export function findUnit(name: string): Unit | undefined {
  const wanted = name.trim().toLowerCase().replace(/\s+/g, " ");
  return UNITS.find((unit) => unit.names.includes(wanted));
}

function toCelsius(value: number, from: string): number {
  return from === "°F" ? ((value - 32) * 5) / 9 : from === "K" ? value - 273.15 : value;
}

function fromCelsius(value: number, to: string): number {
  return to === "°F" ? (value * 9) / 5 + 32 : to === "K" ? value + 273.15 : value;
}

/** `value` in `from`, converted to `to`; `null` when they measure different things. */
export function convert(value: number, from: Unit, to: Unit): number | null {
  if (from.kind !== to.kind) {
    return null;
  }
  if (from.kind === "temperature") {
    return fromCelsius(toCelsius(value, from.symbol), to.symbol);
  }
  return (value * from.factor) / to.factor;
}

const QUERY = /^(-?(?:\d+(?:\.\d+)?|\.\d+))\s*([a-z°"'/ ]+?)\s+(?:in|to|as|into|=)\s+([a-z°"'/ ]+?)\s*$/i;

export function answer(query: string): Answer | null {
  const match = QUERY.exec(query.trim());
  if (!match?.[1] || !match[2] || !match[3]) {
    return null;
  }
  const from = findUnit(match[2]);
  const to = findUnit(match[3]);
  if (!from || !to) {
    return null;
  }
  const value = Number(match[1]);
  const result = convert(value, from, to);
  if (result === null || !Number.isFinite(result)) {
    return null;
  }
  const shown = formatNumber(Number(result.toPrecision(10)));
  return {
    title: `${shown} ${to.symbol}`,
    subtitle: `${formatNumber(value)} ${from.symbol} in ${to.symbol}`,
    copy: shown,
  };
}
