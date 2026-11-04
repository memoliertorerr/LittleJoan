//! The per-OS readers' pure parts (LJ46), compiled and tested on every OS (browsers/tests.rs):
//! Windows' `StartMenuInternet` entries and command lines, Linux's desktop files and `xdg-mime`'s
//! answer, Little Joan's own ids. platform/{win,mac,linux}/browsers.rs feed them what the OS says.

use std::path::{Path, PathBuf};

use super::target::{Browser, Launch};

/// Little Joan's ids on each OS (src-tauri/windows/hooks.nsh, tauri.conf.json, the desktop files).
pub const BUNDLE_ID: &str = "io.github.memoliertorerr.littlejoan";
pub const CLIENT_ID: &str = "LittleJoan";
pub const PROGID: &str = "LittleJoanURL";
pub const DESKTOP_IDS: [&str; 2] = [
    "Little Joan.desktop",
    "io.github.memoliertorerr.littlejoan.desktop",
];
const BINARY: &str = "little-joan";

/// Programs a browser's command may never start (phase 21's security review): shells, script
/// hosts and launchers that would run whatever their arguments say.
const INTERPRETERS: [&str; 12] = [
    "cmd.exe",
    "powershell.exe",
    "pwsh.exe",
    "wscript.exe",
    "cscript.exe",
    "mshta.exe",
    "rundll32.exe",
    "regsvr32.exe",
    "msiexec.exe",
    "wsl.exe",
    "bash.exe",
    "conhost.exe",
];

/// Windows: whether a browser's command may start `program` — an `.exe`, and no interpreter.
pub fn launchable_program(program: &str) -> bool {
    let name = program
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(program)
        .trim()
        .to_ascii_lowercase();
    name.ends_with(".exe") && name.len() > 4 && !INTERPRETERS.contains(&name.as_str())
}

/// Windows: a command line whose program may be started (`launchable_program`).
pub fn launchable_command(line: &str) -> bool {
    split_command(line).is_some_and(|(program, _)| launchable_program(&program))
}

/// A command line that is there and may be started.
fn usable_command(command: Option<&str>) -> Option<&str> {
    command
        .map(str::trim)
        .filter(|c| !c.is_empty() && launchable_command(c))
}

/// Windows: one `StartMenuInternet\<id>` entry as a browser — its name (the key's default value,
/// else Capabilities' ApplicationName unless it is an indirect `@…` string, else the id) and how to
/// open a URL with it: the `shell\open\command` of the ProgId its Capabilities give https to
/// (`progid_command`, `%1` = the URL), else its own `shell\open\command` with the URL appended.
/// `None` without a command, or when the command's program isn't launchable.
pub fn start_menu_entry(
    id: &str,
    default_name: Option<&str>,
    app_name: Option<&str>,
    command: Option<&str>,
    progid_command: Option<&str>,
) -> Option<Browser> {
    let command = usable_command(progid_command).or_else(|| usable_command(command))?;
    let name = default_name
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .or_else(|| {
            app_name
                .map(str::trim)
                .filter(|n| !n.is_empty() && !n.starts_with('@'))
        })
        .unwrap_or(id);
    Some(Browser {
        id: id.to_owned(),
        name: name.to_owned(),
        launch: Launch::Command(command.to_owned()),
    })
}

/// Windows: the effective choice — `UserChoiceLatest`'s (newer Windows 11) over `UserChoice`'s.
pub fn pick_progid(latest: Option<String>, legacy: Option<String>) -> Option<String> {
    latest
        .filter(|p| !p.trim().is_empty())
        .or(legacy.filter(|p| !p.trim().is_empty()))
}

/// Windows: the browser a `UserChoice` ProgId means — the entry whose Capabilities give https to
/// it (`(id, https ProgId)` pairs) — or the ProgId itself when no entry claims it.
pub fn default_from_progid(progid: &str, associations: &[(String, String)]) -> String {
    if progid.eq_ignore_ascii_case(PROGID) {
        return CLIENT_ID.to_owned();
    }
    associations
        .iter()
        .find(|(_, https)| https.eq_ignore_ascii_case(progid))
        .map_or_else(|| progid.to_owned(), |(id, _)| id.clone())
}

/// Windows: a command line as CreateProcess's callers write it — the program, quoted or not, then
/// arguments split by the MSVC rules (quotes group, `\"` is a quote, backslashes before a quote
/// halve). `None` for an empty line.
pub fn split_command(line: &str) -> Option<(String, Vec<String>)> {
    let line = line.trim();
    let (program, rest) = if let Some(quoted) = line.strip_prefix('"') {
        let end = quoted.find('"')?;
        (&quoted[..end], &quoted[end + 1..])
    } else {
        let end = line.find(char::is_whitespace).unwrap_or(line.len());
        (&line[..end], &line[end..])
    };
    if program.is_empty() {
        return None;
    }
    Some((program.to_owned(), split_args(rest)))
}

fn split_args(text: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut started = false;
    let mut quoting = false;
    let mut slashes = 0usize;
    for c in text.chars() {
        match c {
            '\\' => {
                slashes += 1;
                started = true;
            }
            '"' => {
                current.extend(std::iter::repeat_n('\\', slashes / 2));
                if slashes % 2 == 1 {
                    current.push('"');
                } else {
                    quoting = !quoting;
                }
                slashes = 0;
                started = true;
            }
            c if c.is_whitespace() && !quoting => {
                current.extend(std::iter::repeat_n('\\', slashes));
                slashes = 0;
                if started {
                    args.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            c => {
                current.extend(std::iter::repeat_n('\\', slashes));
                slashes = 0;
                current.push(c);
                started = true;
            }
        }
    }
    current.extend(std::iter::repeat_n('\\', slashes));
    if started {
        args.push(current);
    }
    args
}

/// The arguments to start a browser with for `url`: `%1` / `%L` placeholders become the URL; with
/// none, the URL is one more argument. Always exactly one argument holds the URL, never a shell.
pub fn with_url(args: &[String], url: &str) -> Vec<String> {
    let placeholder = |arg: &str| matches!(arg, "%1" | "%L" | "%l");
    if args.iter().any(|arg| placeholder(arg)) {
        args.iter()
            .map(|arg| {
                if placeholder(arg) {
                    url.to_owned()
                } else {
                    arg.clone()
                }
            })
            .collect()
    } else {
        args.iter()
            .cloned()
            .chain(std::iter::once(url.to_owned()))
            .collect()
    }
}

/// Linux: a desktop file's `[Desktop Entry]` group, key → value.
pub fn desktop_entry(text: &str) -> Vec<(String, String)> {
    let mut entry = Vec::new();
    let mut in_group = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            in_group = line == "[Desktop Entry]";
        } else if in_group && !line.starts_with('#') {
            if let Some((key, value)) = line.split_once('=') {
                entry.push((key.trim().to_owned(), value.trim().to_owned()));
            }
        }
    }
    entry
}

/// What a desktop file is for browsing: `None` when it isn't an application, or is hidden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopFile {
    /// A shown application whose MimeType has x-scheme-handler/https.
    Browser(Browser),
    /// Hidden (`Hidden` / `NoDisplay`) or no browser: it still masks a file of the same id in a
    /// later data directory.
    Other,
}

/// Linux: desktop file `id` at `path` as a browser, from its text.
pub fn desktop_browser(id: &str, path: &Path, text: &str) -> DesktopFile {
    let entry = desktop_entry(text);
    let get = |key: &str| {
        entry
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    };
    let shown = get("NoDisplay") != Some("true") && get("Hidden") != Some("true");
    let web = get("MimeType")
        .unwrap_or_default()
        .split(';')
        .any(|t| t == "x-scheme-handler/https");
    match (get("Type"), get("Exec"), get("Name")) {
        (Some("Application"), Some(exec), Some(name)) if shown && web && !exec.is_empty() => {
            DesktopFile::Browser(Browser {
                id: id.to_owned(),
                name: name.to_owned(),
                launch: Launch::Desktop {
                    path: path.to_owned(),
                    exec: exec.to_owned(),
                },
            })
        }
        _ => DesktopFile::Other,
    }
}

/// Linux: a desktop file's `Exec` as arguments for `url` (the Desktop Entry spec): double quotes
/// group, a backslash escapes `"`, `` ` ``, `$` and `\` in them; `%u %U %f %F` become the URL,
/// `%%` a percent sign, the other field codes go. `None` for an unbalanced quote, or a URL code in
/// a quoted argument (the spec forbids it).
pub fn exec_args(exec: &str, url: &str) -> Option<Vec<String>> {
    let mut words: Vec<(String, bool)> = Vec::new();
    let mut current: Option<(String, bool)> = None;
    let mut quoting = false;
    let mut chars = exec.chars().peekable();
    while let Some(c) = chars.next() {
        if quoting {
            match c {
                '\\' if chars.peek().is_some_and(|n| "\"`$\\".contains(*n)) => {
                    let next = chars.next().unwrap_or('\\');
                    current
                        .get_or_insert_with(|| (String::new(), true))
                        .0
                        .push(next);
                }
                '"' => quoting = false,
                c => current
                    .get_or_insert_with(|| (String::new(), true))
                    .0
                    .push(c),
            }
        } else if c == '"' {
            quoting = true;
            current.get_or_insert_with(|| (String::new(), true)).1 = true;
        } else if c == ' ' || c == '\t' {
            if let Some(word) = current.take() {
                words.push(word);
            }
        } else {
            current
                .get_or_insert_with(|| (String::new(), false))
                .0
                .push(c);
        }
    }
    if quoting {
        return None;
    }
    words.extend(current);
    let mut args = Vec::new();
    for (text, quoted) in words {
        let url_code = |t: &str| matches!(t, "%u" | "%U" | "%f" | "%F");
        if quoted {
            if ["%u", "%U", "%f", "%F"]
                .iter()
                .any(|code| text.contains(code))
            {
                return None;
            }
            args.push(text);
        } else if url_code(&text) {
            args.push(url.to_owned());
        } else if !matches!(text.as_str(), "%i" | "%c" | "%k") {
            args.push(field_codes_out(&text));
        }
    }
    (!args.is_empty()).then_some(args)
}

/// `%%` → `%`; any other `%x` field code inside a word goes.
fn field_codes_out(word: &str) -> String {
    let mut out = String::new();
    let mut chars = word.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
        } else if let Some(next) = chars.next()
            && next == '%'
        {
            out.push('%');
        }
    }
    out
}

/// Linux: whether desktop file `id`, running `exec`, is Little Joan — by its id, or by the program
/// it runs (an AppImage's integration names its file differently): `little-joan`, or one of
/// `own_paths` (this executable, `$APPIMAGE`).
pub fn is_own_desktop(id: &str, exec: &str, own_paths: &[PathBuf]) -> bool {
    if DESKTOP_IDS.iter().any(|own| own.eq_ignore_ascii_case(id)) {
        return true;
    }
    let Some(program) = exec_args(exec, "").and_then(|args| args.into_iter().next()) else {
        return false;
    };
    let program = PathBuf::from(program);
    program.file_name().is_some_and(|name| name == BINARY) || own_paths.contains(&program)
}

/// Windows: whether a `StartMenuInternet` entry is Little Joan — its key, or a command that runs
/// this executable.
pub fn is_own_client(id: &str, command: &str, own_exe: Option<&Path>) -> bool {
    if id.eq_ignore_ascii_case(CLIENT_ID) {
        return true;
    }
    let Some((program, _)) = split_command(command) else {
        return false;
    };
    own_exe.is_some_and(|own| {
        own.to_string_lossy()
            .eq_ignore_ascii_case(program.trim_matches('"'))
    })
}

/// Linux: `xdg-mime query default …`'s answer — one desktop file id, or nothing.
pub fn xdg_answer(stdout: &str) -> Option<String> {
    let line = stdout.lines().map(str::trim).find(|l| !l.is_empty())?;
    (line.ends_with(".desktop") && !line.contains('/')).then(|| line.to_owned())
}

/// Linux, without `xdg-mime` (some systems lack xdg-utils; CI's ubuntu-22.04-arm runner did): the
/// default for https from mimeapps.list files, as the XDG MIME Applications spec orders them (the
/// caller passes their texts in that order). The first file whose `[Default Applications]` names a
/// desktop file for x-scheme-handler/https wins, with its first entry.
pub fn mimeapps_default<'a>(lists: impl IntoIterator<Item = &'a str>) -> Option<String> {
    lists.into_iter().find_map(|text| {
        let mut in_group = false;
        text.lines().find_map(|raw| {
            let line = raw.trim();
            if line.starts_with('[') {
                in_group = line == "[Default Applications]";
                return None;
            }
            let value = line
                .strip_prefix("x-scheme-handler/https")?
                .trim_start()
                .strip_prefix('=')?;
            let first = value.split(';').map(str::trim).find(|id| !id.is_empty())?;
            (in_group && first.ends_with(".desktop") && !first.contains('/'))
                .then(|| first.to_owned())
        })
    })
}
