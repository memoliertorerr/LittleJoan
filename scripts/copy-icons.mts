// Copies the desktop icon set produced by `tauri icon` (into .cache/icons/generated) to
// src-tauri/icons. Mobile and Windows Store icons are left behind: Little Joan is desktop-only.
import { copyFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const from = resolve(root, ".cache", "icons", "generated");
const to = resolve(root, "src-tauri", "icons");

for (const name of ["32x32.png", "64x64.png", "128x128.png", "128x128@2x.png", "icon.png", "icon.icns", "icon.ico"]) {
  copyFileSync(resolve(from, name), resolve(to, name));
  console.log(`copied ${name}`);
}
