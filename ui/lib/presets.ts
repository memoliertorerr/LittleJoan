// Theme presets: muted palettes of one to three colors, Little Joan's own. A preset sets the
// colors, the intensity and the grain; light / dark / system stays the user's choice. The first
// one is the default theme (settings.rs has the same values: a test compares them).

export interface Preset {
  id: string;
  name: string;
  colors: readonly string[];
  intensity: number;
  grain: number;
}

export const PRESETS: readonly Preset[] = [
  { id: "mist", name: "Mist", colors: ["#8c9bb5"], intensity: 0.45, grain: 0.15 },
  { id: "sand", name: "Sand", colors: ["#c9a27e", "#d8c49a"], intensity: 0.55, grain: 0.3 },
  { id: "sage", name: "Sage", colors: ["#8fae96", "#b9b98c"], intensity: 0.55, grain: 0.25 },
  { id: "lagoon", name: "Lagoon", colors: ["#6fa8c4", "#7fc2ab"], intensity: 0.6, grain: 0.25 },
  { id: "dusk", name: "Dusk", colors: ["#9d8fd0", "#e0a2a0"], intensity: 0.6, grain: 0.3 },
  { id: "ember", name: "Ember", colors: ["#d98f74", "#d6b067"], intensity: 0.55, grain: 0.35 },
  { id: "orchid", name: "Orchid", colors: ["#c294cf", "#8ea3dc"], intensity: 0.55, grain: 0.2 },
  { id: "glacier", name: "Glacier", colors: ["#94bce2", "#a9d8d2", "#c7bfe8"], intensity: 0.5, grain: 0.2 },
  { id: "meadow", name: "Meadow", colors: ["#a3c27a", "#78b8a6", "#dcc67d"], intensity: 0.55, grain: 0.3 },
  { id: "berry", name: "Berry", colors: ["#cf86a5", "#958acf", "#e8ae96"], intensity: 0.6, grain: 0.3 },
  { id: "graphite", name: "Graphite", colors: ["#7d828c"], intensity: 0.2, grain: 0.35 },
];

/** The default theme's colors, intensity and grain. */
export const DEFAULT_PRESET: Preset = PRESETS[0] ?? { id: "mist", name: "Mist", colors: ["#8c9bb5"], intensity: 0.45, grain: 0.15 };
