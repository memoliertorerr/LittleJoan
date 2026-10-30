// Settings → General's "Quit Little Joan" (LJ19.7): the same quit as the tray's and the command
// bar's (Rust's joan::quit — history saved, on macOS the window closed first), through a command of
// its own that only the settings window may call (`quit_from_settings`; the bar's `quit_app` isn't
// Settings'). What Settings hasn't saved is dropped, as with Cancel; the note under the button
// says so. The button stays disabled once pressed, unless the call fails.

/** The command the button calls (capabilities/settings.json grants it to Settings only). */
export const QUIT_COMMAND = "quit_from_settings";

/** What the wiring needs of the button (an HTMLButtonElement in the page; a fake in tests). */
export interface QuitControl {
  disabled: boolean;
  addEventListener(type: "click", listener: () => void): void;
}

/**
 * Wires the button: a click runs `before` (a key recording stops), disables the button and calls
 * `call(QUIT_COMMAND)`; if that fails, the button works again and `onError` gets the reason.
 */
export function quitButton(button: QuitControl, call: (command: string) => Promise<unknown>, before: () => void, onError: (message: string) => void): void {
  button.addEventListener("click", () => {
    if (button.disabled) {
      return;
    }
    before();
    button.disabled = true;
    call(QUIT_COMMAND).catch((err: unknown) => {
      button.disabled = false;
      onError(err instanceof Error ? err.message : String(err));
    });
  });
}
