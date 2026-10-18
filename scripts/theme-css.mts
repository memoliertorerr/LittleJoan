// Writes ui/lib/theme-defaults.css: the default theme's CSS variables in light and dark, so a
// page looks right before its script hands it the user's theme (and the page webview's plain
// first page never gets one). ui/lib/theme.test.ts fails when the file is out of date.
//
//   node scripts/theme-css.mts
import { writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { defaultsCss } from "../ui/lib/theme-css.ts";

const file = resolve(import.meta.dirname, "..", "ui", "lib", "theme-defaults.css");
writeFileSync(file, defaultsCss());
console.log(`wrote ${file}`);
