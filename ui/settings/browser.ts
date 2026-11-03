// Settings → General's "Browser" section (LJ46; DOM glue): whether Little Joan is the default
// browser, "Open default apps settings" (it opens the system's own settings and nothing else), and
// "Main browser" — System default or one the system lists. browser-logic.ts writes every text; the
// names come from the system and are drawn as text. The main browser is a draft like the rest of
// General: Save keeps it, Cancel drops it. Coming back to the window (from the system's settings,
// say) reads the system again.
import { SYSTEM_DEFAULT, browserChoices, browsersInfoFrom, defaultText, goesToNote, settingsNote, showsSettingsButton, type BrowsersInfo } from "./browser-logic.ts";

export interface BrowserSection {
  /** The list's choice ("" = System default); `undefined` until the system answered (Save leaves it). */
  draft(): string | undefined;
  /** Reads the system now (`browsers_info`) and draws it, keeping the choice shown. */
  load(saved: string | null): Promise<void>;
}

export function browserSection(
  root: HTMLElement,
  call: <T>(command: string) => Promise<T>,
  onChange: () => void,
  onError: (message: string) => void,
): BrowserSection {
  const defaultLine = root.querySelector<HTMLElement>("#browser-default");
  const openButton = root.querySelector<HTMLButtonElement>("#browser-open-settings");
  const settingsLine = root.querySelector<HTMLElement>("#browser-settings-note");
  const list = root.querySelector<HTMLSelectElement>("#main-browser");
  const goesTo = root.querySelector<HTMLElement>("#main-browser-note");
  let info: BrowsersInfo | null = null;

  function draw(selected: string): void {
    if (!info || !list) {
      return;
    }
    if (defaultLine) {
      defaultLine.textContent = defaultText(info);
    }
    if (openButton) {
      openButton.hidden = !showsSettingsButton(info);
    }
    if (settingsLine) {
      settingsLine.textContent = settingsNote(info);
    }
    list.replaceChildren(
      ...browserChoices(info).map((choice) => {
        const option = document.createElement("option");
        option.value = choice.id;
        option.textContent = choice.name;
        option.selected = choice.id === selected;
        return option;
      }),
    );
    if (goesTo) {
      goesTo.textContent = goesToNote(info, list.value);
    }
  }

  list?.addEventListener("change", () => {
    if (info && goesTo) {
      goesTo.textContent = goesToNote(info, list.value);
    }
    onChange();
  });
  openButton?.addEventListener("click", () => {
    call<string>("open_default_apps_settings").catch((err: unknown) => onError(err instanceof Error ? err.message : String(err)));
  });

  async function load(saved: string | null): Promise<void> {
    const shown = info && list ? list.value : (saved ?? SYSTEM_DEFAULT);
    info = browsersInfoFrom(await call<unknown>("browsers_info"));
    draw(shown);
  }

  return {
    draft: () => (info && list ? list.value : undefined),
    load,
  };
}
