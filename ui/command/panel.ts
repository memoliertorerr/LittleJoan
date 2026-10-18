// The welcome in the command bar's panel on the first launch (DOM glue only), with its
// start-at-login checkbox. What it says comes from welcome.ts; the facts from Rust's start_info
// (bar.ts asks for them when the bar loads).
import { invoke } from "@tauri-apps/api/core";
import { welcome, type ShortcutLine, type StartInfo, type Welcome } from "./welcome.ts";

function element<T extends HTMLElement>(selector: string): T | null {
  return document.querySelector<T>(selector);
}

function fillLine(target: HTMLElement, line: ShortcutLine): void {
  const parts: Node[] = [document.createTextNode(line.before)];
  if (line.code) {
    const code = document.createElement(line.kind === "keys" ? "kbd" : "code");
    code.textContent = line.code;
    parts.push(code);
  }
  parts.push(document.createTextNode(line.after));
  target.replaceChildren(...parts);
}

function showWelcome(shown: Welcome): void {
  const section = element<HTMLElement>("#welcome");
  const title = element<HTMLElement>("#welcome-title");
  const shortcut = element<HTMLElement>("#welcome-shortcut");
  const tray = element<HTMLElement>("#welcome-tray");
  const close = element<HTMLElement>("#welcome-close");
  if (!section || !title || !shortcut || !tray || !close) {
    return;
  }
  title.textContent = shown.title;
  fillLine(shortcut, shown.shortcut);
  tray.textContent = shown.tray;
  close.textContent = shown.close;
  section.hidden = false;
  if (shown.startAtLogin !== null) {
    setUpLoginSwitch(shown.startAtLogin);
  }
}

/** The checkbox adds or removes the OS login entry at once and shows what the OS reports. */
function setUpLoginSwitch(enabled: boolean): void {
  const row = element<HTMLElement>("#welcome-login-row");
  const box = element<HTMLInputElement>("#welcome-login");
  const error = element<HTMLElement>("#welcome-error");
  if (!row || !box || !error) {
    return;
  }
  box.checked = enabled;
  row.hidden = false;
  box.addEventListener("change", () => {
    const wanted = box.checked;
    box.disabled = true;
    error.hidden = true;
    invoke<boolean>("set_start_at_login", { enabled: wanted })
      .then((now) => {
        box.checked = now;
      })
      .catch((err: unknown) => {
        box.checked = !wanted;
        error.textContent = String(err);
        error.hidden = false;
      })
      .finally(() => {
        box.disabled = false;
      });
  });
}

/** On the first run, the welcome shows (`info`: start_info's answer). */
export function fillPanel(info: StartInfo): void {
  const shown = welcome(info);
  if (shown) {
    showWelcome(shown);
  }
}
