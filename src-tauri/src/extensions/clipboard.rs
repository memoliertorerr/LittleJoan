//! littleJoan.clipboard.writeText: puts text on the system clipboard natively (the panel's own
//! `navigator.clipboard` needs a click first and differs per engine; this works the same
//! everywhere, which is why it needs the `clipboard` permission). Runs on the main thread, where
//! AppKit, GTK and the Win32 clipboard want it, and waits at most a second for the answer.

use std::sync::mpsc;
use std::time::Duration;

use tauri::{AppHandle, Runtime};

use crate::platform;

/// The most text written at once, in bytes.
pub const MAX_TEXT_BYTES: usize = 1024 * 1024;

pub fn write_text<R: Runtime>(app: &AppHandle<R>, text: String) -> Result<(), String> {
    if text.len() > MAX_TEXT_BYTES {
        return Err("The text is too long for the clipboard (1 MB).".to_owned());
    }
    let (done, result) = mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = done.send(platform::write_clipboard(&text));
    })
    .map_err(|err| err.to_string())?;
    result
        .recv_timeout(Duration::from_secs(1))
        .map_err(|_| "The clipboard didn't answer.".to_owned())?
}
