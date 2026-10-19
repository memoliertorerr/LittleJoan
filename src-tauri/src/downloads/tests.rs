use std::collections::HashSet;
use std::path::{Path, PathBuf};

use url::Url;

use super::*;

fn url(s: &str) -> Url {
    Url::parse(s).expect("valid")
}

fn origin() -> Url {
    url("tauri://localhost/")
}

#[test]
fn web_blob_and_data_downloads_are_accepted() {
    for ok in [
        "https://example.com/report.pdf",
        "http://127.0.0.1:8123/lj-smoke.bin",
        "blob:https://example.com/4f7e2a1c-0000-4000-8000-000000000000",
        "blob:http://127.0.0.1:8123/uuid",
        "data:text/plain,hello",
        "data:application/octet-stream;base64,AAAA",
    ] {
        assert!(accepts(&url(ok), &origin()), "{ok}");
    }
}

#[test]
fn extension_app_and_other_urls_never_download() {
    for refused in [
        "ljext://hello.localhost/index.html",
        "http://ljext.hello.localhost/index.html",
        "https://ljext.hello.localhost/a.bin",
        "http://LJEXT.hello.localhost/a.bin",
        "blob:ljext://hello.localhost/uuid",
        "blob:http://ljext.hello.localhost/uuid",
        "blob:tauri://localhost/uuid",
        "blob:null/uuid",
        "tauri://localhost/index.html",
        "file:///etc/passwd",
        "ftp://example.com/a.bin",
        "javascript:alert(1)",
        "about:blank",
    ] {
        assert!(!accepts(&url(refused), &origin()), "{refused}");
    }
    // Windows' app origin is an http one.
    let windows = url("http://tauri.localhost/");
    assert!(!accepts(
        &url("http://tauri.localhost/index.html"),
        &windows
    ));
    assert!(!accepts(&url("blob:http://tauri.localhost/uuid"), &windows));
    assert!(accepts(&url("http://localhost:8000/a.zip"), &windows));
}

#[test]
fn the_name_comes_from_the_engine_else_the_url_else_download() {
    let page = url("https://example.com/files/Annual%20Report.pdf?x=1");
    assert_eq!(file_name(Some("report.pdf"), &page), "report.pdf");
    assert_eq!(file_name(None, &page), "Annual Report.pdf");
    assert_eq!(file_name(Some(""), &page), "Annual Report.pdf");
    assert_eq!(file_name(Some("..."), &page), "Annual Report.pdf");
    assert_eq!(file_name(None, &url("https://example.com/a/b/")), "b");
    assert_eq!(file_name(None, &url("https://example.com/")), FALLBACK_NAME);
    assert_eq!(
        file_name(None, &url("https://example.com/%2E%2E")),
        FALLBACK_NAME
    );
    assert_eq!(
        file_name(None, &url("https://example.com/%2Fetc%2Fpasswd")),
        "_etc_passwd"
    );
    // blob: and data: URLs name nothing; WebKitGTK's payload "name" for data: is no name.
    assert_eq!(
        file_name(None, &url("blob:https://example.com/uuid")),
        FALLBACK_NAME
    );
    assert_eq!(
        file_name(Some("plain,hello"), &url("data:text/plain,hello")),
        FALLBACK_NAME
    );
    assert_eq!(
        file_name(Some("notes.txt"), &url("data:text/plain,hello")),
        "notes.txt"
    );
}

#[test]
fn names_lose_separators_controls_and_hidden_characters() {
    assert_eq!(clean("../../etc/passwd").as_deref(), Some("_.._etc_passwd"));
    assert_eq!(clean("..\\..\\boot.ini").as_deref(), Some("_.._boot.ini"));
    assert_eq!(
        clean("a/b\\c:d*e?f\"g<h>i|j").as_deref(),
        Some("a_b_c_d_e_f_g_h_i_j")
    );
    assert_eq!(clean("line\nbreak\t.txt").as_deref(), Some("linebreak.txt"));
    assert_eq!(clean("bell\u{7}.txt").as_deref(), Some("bell.txt"));
    // A right-to-left override (U+202E) would show "report<RLO>fdp.exe" as "reportexe.pdf".
    assert_eq!(
        clean("report\u{202E}fdp.exe").as_deref(),
        Some("reportfdp.exe")
    );
    assert_eq!(
        clean("zero\u{200B}width.txt").as_deref(),
        Some("zerowidth.txt")
    );
    assert_eq!(
        clean("café ünïcode 文件.txt").as_deref(),
        Some("café ünïcode 文件.txt")
    );
}

#[test]
fn names_have_no_leading_dots_nor_trailing_dots_and_spaces() {
    assert_eq!(clean(".bashrc").as_deref(), Some("bashrc"));
    assert_eq!(clean("..").as_deref(), None);
    assert_eq!(clean(".").as_deref(), None);
    assert_eq!(clean("   ").as_deref(), None);
    assert_eq!(clean("").as_deref(), None);
    assert_eq!(clean("report.pdf. . ").as_deref(), Some("report.pdf"));
    assert_eq!(clean("  spaced.txt  ").as_deref(), Some("spaced.txt"));
    assert_eq!(clean("\u{202E}").as_deref(), None);
}

#[test]
fn windows_device_names_are_never_used() {
    for device in [
        "CON",
        "con",
        "nul.txt",
        "NUL.tar.gz",
        "Com1.log",
        "lpt9",
        "aux .txt",
        "prn",
    ] {
        let cleaned = clean(device).expect("a name");
        assert!(cleaned.starts_with('_'), "{device} → {cleaned}");
    }
    for fine in ["console.log", "nullable.txt", "com10", "auxiliary"] {
        assert_eq!(clean(fine).as_deref(), Some(fine));
    }
}

#[test]
fn long_names_are_capped_at_200_bytes_keeping_the_extension() {
    let long = format!("{}.pdf", "a".repeat(300));
    let capped = clean(&long).expect("a name");
    assert_eq!(capped.len(), MAX_NAME_BYTES);
    assert!(capped.ends_with(".pdf"));
    // Never in the middle of a character.
    let wide = format!("{}.tar.gz", "é".repeat(150));
    let capped = clean(&wide).expect("a name");
    assert!(capped.len() <= MAX_NAME_BYTES);
    assert!(capped.ends_with(".tar.gz"));
    let no_extension = "日".repeat(100);
    let capped = clean(&no_extension).expect("a name");
    assert!(capped.len() <= MAX_NAME_BYTES && capped.len() > MAX_NAME_BYTES - 3);
    // A cut that ends in spaces is trimmed.
    let spaced = format!("{}  {}", "b".repeat(198), "c".repeat(20));
    assert_eq!(clean(&spaced), Some("b".repeat(198)));
}

#[test]
fn extensions_are_short_plain_and_may_include_tar() {
    assert_eq!(split_extension("report.pdf"), ("report", ".pdf"));
    assert_eq!(split_extension("archive.tar.gz"), ("archive", ".tar.gz"));
    assert_eq!(split_extension("archive.TAR.xz"), ("archive", ".TAR.xz"));
    assert_eq!(
        split_extension("v1.2.3-setup.exe"),
        ("v1.2.3-setup", ".exe")
    );
    assert_eq!(split_extension("notes"), ("notes", ""));
    assert_eq!(
        split_extension("Chapter 1. Intro"),
        ("Chapter 1. Intro", "")
    );
    assert_eq!(split_extension("trailing."), ("trailing.", ""));
    assert_eq!(split_extension(".tar.gz"), (".tar", ".gz"));
    assert_eq!(
        split_extension("x.verylongextension"),
        ("x.verylongextension", "")
    );
}

#[test]
fn taken_names_get_a_number_before_the_extension() {
    let on_disk: HashSet<&str> = ["report.pdf", "report (1).pdf", "archive.tar.gz", "notes"]
        .into_iter()
        .collect();
    let taken = |name: &str| on_disk.contains(name);
    assert_eq!(
        unique("report.pdf", taken).as_deref(),
        Some("report (2).pdf")
    );
    assert_eq!(
        unique("archive.tar.gz", taken).as_deref(),
        Some("archive (1).tar.gz")
    );
    assert_eq!(unique("notes", taken).as_deref(), Some("notes (1)"));
    assert_eq!(unique("fresh.txt", taken).as_deref(), Some("fresh.txt"));
    assert_eq!(unique("full.bin", |_| true), None);
}

#[test]
fn the_folder_is_the_os_downloads_else_downloads_at_home() {
    assert_eq!(
        folder(
            Some(PathBuf::from("/x/Downloads")),
            Some(PathBuf::from("/home/u"))
        ),
        Some(PathBuf::from("/x/Downloads"))
    );
    assert_eq!(
        folder(None, Some(PathBuf::from("/home/u"))),
        Some(PathBuf::from("/home/u/Downloads"))
    );
    assert_eq!(folder(None, None), None);
}

fn dir() -> PathBuf {
    PathBuf::from("/home/u/Downloads")
}

#[test]
fn a_second_identical_download_gets_its_own_name_even_before_the_first_lands() {
    let mut downloads = Downloads::new();
    let bin = url("http://127.0.0.1:8123/lj-smoke.bin");
    let nothing = |_: &Path| false;
    let first = downloads
        .decide(
            &bin,
            &origin(),
            false,
            Some("lj-smoke.bin"),
            &dir(),
            nothing,
        )
        .expect("saved");
    assert_eq!(first, dir().join("lj-smoke.bin"));
    downloads.started(bin.clone(), first.clone());
    // Not on disk yet, but under way: taken (case aside, as Windows and macOS compare names).
    let second = downloads
        .decide(
            &bin,
            &origin(),
            false,
            Some("LJ-SMOKE.bin"),
            &dir(),
            nothing,
        )
        .expect("saved");
    assert_eq!(second, dir().join("LJ-SMOKE (1).bin"));
    // Once the first is on disk and done, the disk says it is taken.
    downloads.finished(&bin, true);
    let on_disk = |path: &Path| path == dir().join("lj-smoke.bin");
    let third = downloads
        .decide(
            &bin,
            &origin(),
            false,
            Some("lj-smoke.bin"),
            &dir(),
            on_disk,
        )
        .expect("saved");
    assert_eq!(third, dir().join("lj-smoke (1).bin"));
}

#[test]
fn nothing_is_saved_while_a_panel_shows_nor_from_extensions() {
    let downloads = Downloads::new();
    let nothing = |_: &Path| false;
    let web = url("https://example.com/a.zip");
    assert_eq!(
        downloads.decide(&web, &origin(), true, Some("a.zip"), &dir(), nothing),
        Err(Refusal::Panel)
    );
    assert_eq!(
        downloads.decide(
            &url("ljext://hello.localhost/a.zip"),
            &origin(),
            false,
            Some("a.zip"),
            &dir(),
            nothing
        ),
        Err(Refusal::Scheme)
    );
    assert_eq!(
        downloads.decide(&web, &origin(), false, Some("a.zip"), &dir(), |_| true),
        Err(Refusal::Name)
    );
    assert_eq!(Refusal::Panel.as_str(), "panel");
}

#[test]
fn a_finished_download_is_reported_once_and_remembered_when_saved() {
    let mut downloads = Downloads::new();
    let a = url("https://example.com/a.zip");
    let b = url("https://example.com/b.zip");
    downloads.started(a.clone(), dir().join("a.zip"));
    downloads.started(a.clone(), dir().join("a (1).zip"));
    downloads.started(b.clone(), dir().join("b.zip"));
    // The same URL twice: the earlier one ends first.
    let done = downloads.finished(&a, true).expect("under way");
    assert_eq!(done.name(), "a.zip");
    assert_eq!(downloads.last_saved(), Some(dir().join("a.zip").as_path()));
    // A failed one isn't "last saved".
    assert_eq!(
        downloads.finished(&b, false).map(|d| d.name()).as_deref(),
        Some("b.zip")
    );
    assert_eq!(downloads.last_saved(), Some(dir().join("a.zip").as_path()));
    // Reported twice (or never started here): nothing.
    assert_eq!(downloads.finished(&b, true), None);
    assert_eq!(
        downloads.finished(&url("https://other.example/"), true),
        None
    );
    // Closing the window: what was under way goes, and nothing is kept.
    let open = downloads.closed();
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].name(), "a (1).zip");
    assert_eq!(downloads.last_saved(), None);
    assert_eq!(downloads.finished(&a, true), None);
}
