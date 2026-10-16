// Generates Little Joan's original icons. Run via `npm run icons`, which then feeds the 1024 px
// app icon to `tauri icon` to produce every platform format under src-tauri/icons/.
import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { appIconSample, render, trayTemplateSample } from "./icon-geometry.mts";
import { encodePng } from "./png.mts";

const root = resolve(import.meta.dirname, "..");
const cacheDir = resolve(root, ".cache", "icons");
const iconsDir = resolve(root, "src-tauri", "icons");
mkdirSync(cacheDir, { recursive: true });
mkdirSync(iconsDir, { recursive: true });

function write(path: string, size: number, pixels: Uint8Array): void {
  writeFileSync(path, encodePng(size, size, pixels));
  console.log(`wrote ${path}`);
}

write(resolve(cacheDir, "app-icon.png"), 1024, render(1024, appIconSample, 3));
write(resolve(iconsDir, "tray-template.png"), 32, render(32, trayTemplateSample, 8));
write(resolve(iconsDir, "tray-template@2x.png"), 64, render(64, trayTemplateSample, 8));
