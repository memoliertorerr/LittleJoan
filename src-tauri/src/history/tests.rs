use super::*;

const DAY: u64 = 86_400_000;
/// A fixed clock: day 1000 of the unix epoch.
const NOW: u64 = 1_000 * DAY;

fn url(s: &str) -> Url {
    Url::parse(s).expect("valid")
}

fn origin() -> Url {
    url("tauri://localhost/")
}

fn visit(at: u64, kind: VisitKind) -> Visit {
    Visit { at, kind }
}

/// A history with one record per (url, title, visits).
fn history(records: &[(&str, &str, &[Visit])]) -> History {
    let mut history = History::default();
    for (address, title, visits) in records {
        for v in *visits {
            assert!(history.record(&url(address), Some(title), v.kind, v.at, &origin()));
        }
    }
    history
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

#[test]
fn fragments_are_dropped_and_queries_kept() {
    let mut h = History::default();
    h.record(
        &url("https://example.com/a?q=1#top"),
        None,
        VisitKind::Link,
        NOW,
        &origin(),
    );
    h.record(
        &url("https://example.com/a?q=1#end"),
        None,
        VisitKind::Link,
        NOW,
        &origin(),
    );
    let [entry] = h.entries() else {
        panic!("one record, got {:?}", h.entries());
    };
    assert_eq!(entry.url, "https://example.com/a?q=1");
    assert_eq!(entry.visit_count, 2);
}

#[test]
fn only_web_pages_are_recorded() {
    let mut h = History::default();
    for (address, app) in [
        ("about:blank", "tauri://localhost/"),
        ("file:///etc/hosts", "tauri://localhost/"),
        ("data:text/html,hi", "tauri://localhost/"),
        ("ljext://localhost/snake/index.html", "tauri://localhost/"),
        ("tauri://localhost/index.html", "tauri://localhost/"),
        // Windows: the app's pages and Tauri's custom schemes are plain http there.
        (
            "http://tauri.localhost/index.html",
            "http://tauri.localhost/",
        ),
        (
            "http://ljext.localhost/snake/index.html",
            "http://tauri.localhost/",
        ),
        ("http://ipc.localhost/open_input", "http://tauri.localhost/"),
        ("http://localhost:1420/index.html", "http://localhost:1420/"),
    ] {
        assert!(
            !h.record(&url(address), None, VisitKind::Link, NOW, &url(app)),
            "{address}"
        );
    }
    let long = format!("https://example.com/?q={}", "a".repeat(MAX_URL_BYTES));
    assert!(!h.record(&url(&long), None, VisitKind::Link, NOW, &origin()));
    assert!(h.entries().is_empty());
    // A dev server on localhost is an ordinary web page.
    assert!(h.record(
        &url("http://localhost:3000/"),
        None,
        VisitKind::Link,
        NOW,
        &origin()
    ));
}

#[test]
fn a_title_change_updates_the_record_of_that_page() {
    let mut h = history(&[
        ("https://a.test/", "A", &[visit(NOW, VisitKind::Link)]),
        ("https://b.test/", "", &[visit(NOW, VisitKind::Link)]),
    ]);
    assert!(h.set_title(&url("https://b.test/#x"), "  B page \n", &origin()));
    assert_eq!(h.entries()[1].title, "B page");
    assert_eq!(h.entries()[0].title, "A");
    assert!(
        !h.set_title(&url("https://b.test/"), "B page", &origin()),
        "unchanged"
    );
    assert!(
        !h.set_title(&url("https://b.test/"), "   ", &origin()),
        "empty"
    );
    assert!(
        !h.set_title(&url("https://c.test/"), "C", &origin()),
        "not recorded"
    );
    // A later visit without a known title keeps the one it has; long titles are cut.
    h.record(
        &url("https://a.test/"),
        None,
        VisitKind::Link,
        NOW,
        &origin(),
    );
    assert_eq!(h.entries()[0].title, "A");
    h.set_title(&url("https://a.test/"), &"x".repeat(1000), &origin());
    assert_eq!(h.entries()[0].title.chars().count(), MAX_TITLE_CHARS);
}

#[test]
fn records_keep_the_last_20_visits_and_count_them_all() {
    let mut h = History::default();
    for i in 0..25 {
        h.record(
            &url("https://a.test/"),
            None,
            VisitKind::Link,
            NOW + i,
            &origin(),
        );
    }
    let entry = &h.entries()[0];
    assert_eq!(entry.visit_count, 25);
    assert_eq!(entry.visits.len(), MAX_VISITS);
    assert_eq!(entry.visits[0].at, NOW + 5);
}

#[test]
fn hosts_lose_www_and_keep_a_port() {
    assert_eq!(host_key(&url("https://WWW.Example.com/a")), "example.com");
    assert_eq!(
        host_key(&url("http://127.0.0.1:8123/smoke.html")),
        "127.0.0.1:8123"
    );
    assert_eq!(
        host_key(&url("https://www2.example.com:443/")),
        "www2.example.com"
    );
}

#[test]
fn visit_kinds_have_their_weights_and_names() {
    assert_eq!(VisitKind::Typed.weight(), 2.0);
    assert_eq!(VisitKind::Link.weight(), 1.0);
    assert_eq!(VisitKind::Reload.weight(), 0.0);
    let kinds = serde_json::to_string(&[VisitKind::Typed, VisitKind::Link, VisitKind::Reload]);
    assert_eq!(kinds.expect("json"), r#"["typed","link","reload"]"#);
}

#[test]
fn frecency_follows_the_decided_formula() {
    let visits = [
        visit(NOW, VisitKind::Typed),            // 2
        visit(NOW - 14 * DAY, VisitKind::Link),  // 1 · 2^-1
        visit(NOW - DAY, VisitKind::Reload),     // 0
        visit(NOW - 28 * DAY, VisitKind::Link),  // 1 · 2^-2
        visit(NOW - 91 * DAY, VisitKind::Typed), // older than 90 days
        visit(NOW + DAY, VisitKind::Link),       // a clock that went back: age 0
    ];
    assert!(close(frecency(&visits, NOW), 2.0 + 0.5 + 0.25 + 1.0));
    assert!(close(
        frecency(&[visit(NOW - 90 * DAY, VisitKind::Link)], NOW),
        (-90.0f64 / 14.0).exp2()
    ));
    assert_eq!(
        frecency(&[visit(NOW - 90 * DAY - 1, VisitKind::Link)], NOW),
        0.0
    );
    assert_eq!(frecency(&[], NOW), 0.0);
}

#[test]
fn frecency_counts_only_the_last_20_visits() {
    let mut visits: Vec<Visit> = (0..20).map(|_| visit(NOW, VisitKind::Link)).collect();
    // Five older typed visits fall outside the last 20.
    visits.extend((1..=5).map(|i| visit(NOW - i * DAY, VisitKind::Typed)));
    assert!(close(frecency(&visits, NOW), 20.0));
}

#[test]
fn top_sites_rank_hosts_by_frecency() {
    let h = history(&[
        (
            "https://www.example.com/",
            "Example",
            &[visit(NOW, VisitKind::Link)],
        ),
        (
            "https://example.com/news",
            "News",
            &[visit(NOW, VisitKind::Typed)],
        ),
        (
            "https://other.test/",
            "Other",
            &[visit(NOW, VisitKind::Typed)],
        ),
        (
            "https://old.test/",
            "Old",
            &[visit(NOW - 100 * DAY, VisitKind::Typed)],
        ),
    ]);
    let top = h.top_sites(8, &[], NOW);
    let hosts: Vec<_> = top.iter().map(|r| (r.host.as_str(), r.score)).collect();
    // www.example.com and example.com are one site (1 + 2); old.test has no recent visit.
    assert_eq!(hosts, [("example.com", 3.0), ("other.test", 2.0)]);
    assert_eq!(h.top_sites(1, &[], NOW).len(), 1);
    let without = h.top_sites(8, &["WWW.example.com"], NOW);
    assert_eq!(
        without.iter().map(|r| r.host.as_str()).collect::<Vec<_>>(),
        ["other.test"]
    );
}

#[test]
fn top_site_ties_go_to_the_latest_visit() {
    let two_weeks = NOW - 14 * DAY;
    let older = history(&[
        ("https://a.test/", "A", &[visit(NOW, VisitKind::Link)]),
        (
            "https://b.test/",
            "B",
            &[
                visit(two_weeks, VisitKind::Link),
                visit(two_weeks, VisitKind::Link),
            ],
        ),
    ]);
    let order = |h: &History| {
        h.top_sites(8, &[], NOW)
            .into_iter()
            .map(|r| r.host)
            .collect::<Vec<_>>()
    };
    assert_eq!(order(&older), ["a.test", "b.test"]);
    let newer = history(&[
        (
            "https://a.test/",
            "A",
            &[
                visit(two_weeks, VisitKind::Link),
                visit(two_weeks, VisitKind::Link),
            ],
        ),
        ("https://b.test/", "B", &[visit(NOW, VisitKind::Link)]),
    ]);
    assert_eq!(order(&newer), ["b.test", "a.test"]);
}

#[test]
fn a_top_site_opens_its_most_visited_short_url_else_its_origin() {
    let links = |n: u64| {
        (0..n)
            .map(|i| visit(NOW - i, VisitKind::Link))
            .collect::<Vec<_>>()
    };
    let h = history(&[
        ("https://example.com/", "Home", &links(3)),
        ("https://example.com/feed/", "Feed", &links(5)),
        ("https://example.com/a/b", "Deep", &links(10)),
        ("https://example.com/search?q=x", "Search", &links(7)),
        (
            "http://127.0.0.1:8123/smoke.html?persist=set",
            "Smoke",
            &links(1),
        ),
        ("http://127.0.0.1:8123/a/b.html", "", &links(1)),
    ]);
    let top = h.top_sites(8, &[], NOW);
    let example = top
        .iter()
        .find(|r| r.host == "example.com")
        .expect("example.com");
    assert_eq!(
        (example.url.as_str(), example.title.as_str()),
        ("https://example.com/feed/", "Feed")
    );
    let local = top
        .iter()
        .find(|r| r.host == "127.0.0.1:8123")
        .expect("127.0.0.1");
    // No short URL without a query: the origin, titled with the host.
    assert_eq!(
        (local.url.as_str(), local.title.as_str()),
        ("http://127.0.0.1:8123/", "127.0.0.1:8123")
    );
}

#[test]
fn search_awards_the_decided_points() {
    // One typed visit now: frecency 2 for every record.
    let typed: &[Visit] = &[visit(NOW, VisitKind::Typed)];
    let h = history(&[
        (
            "https://example.com/docs/guide",
            "Rust Programming Language",
            typed,
        ),
        ("https://www.github.com/", "GitHub", typed),
    ]);
    let score = |q: &str| {
        h.search(q, 10, NOW)
            .first()
            .map(|r| (r.host.clone(), r.score))
    };
    assert_eq!(
        score("exa"),
        Some(("example.com".into(), 20.0)),
        "host prefix 10"
    );
    assert_eq!(
        score("com").map(|s| s.1),
        Some(16.0),
        "host word boundary 8"
    );
    assert_eq!(
        score("prog"),
        Some(("example.com".into(), 12.0)),
        "title word prefix 6"
    );
    assert_eq!(
        score("guide"),
        Some(("example.com".into(), 8.0)),
        "path word boundary 4"
    );
    assert_eq!(
        score("uid"),
        Some(("example.com".into(), 2.0)),
        "substring 1"
    );
    // Every token must match; points add up.
    assert_eq!(score("git hub"), Some(("github.com".into(), 22.0)));
    assert_eq!(score("git zebra"), None);
    assert_eq!(score("   "), None);
}

#[test]
fn search_ranks_by_points_times_frecency_then_latest_visit() {
    let h = history(&[
        (
            "https://news.test/",
            "News",
            &[visit(NOW - 14 * DAY, VisitKind::Link)],
        ),
        (
            "https://newer.test/",
            "Newer",
            &[visit(NOW - 14 * DAY + 1, VisitKind::Link)],
        ),
        (
            "https://often.test/news",
            "Often",
            &[visit(NOW, VisitKind::Typed), visit(NOW, VisitKind::Typed)],
        ),
        (
            "https://old.test/news",
            "Old",
            &[visit(NOW - 200 * DAY, VisitKind::Typed)],
        ),
    ]);
    let rows = h.search("new", 10, NOW);
    let hosts: Vec<_> = rows.iter().map(|r| r.host.as_str()).collect();
    // often.test: path 4 × 4; newer/news: host prefix 10 × ~0.5; old.test still matches, last.
    assert_eq!(hosts, ["often.test", "newer.test", "news.test", "old.test"]);
    assert_eq!(rows[3].score, 0.0);
    assert_eq!(h.search("new", 2, NOW).len(), 2);
}

#[test]
fn history_is_capped_at_5000_urls_least_frecent_first() {
    let mut h = History::default();
    for i in 0..=MAX_URLS as u64 {
        h.record(
            &url(&format!("https://site{i}.test/")),
            None,
            VisitKind::Link,
            NOW - (MAX_URLS as u64 - i) * 1000,
            &origin(),
        );
    }
    assert_eq!(h.entries().len(), MAX_URLS);
    assert!(
        h.entries().iter().all(|e| e.url != "https://site0.test/"),
        "the oldest went first"
    );
    assert!(
        h.entries()
            .iter()
            .any(|e| e.url == format!("https://site{MAX_URLS}.test/"))
    );
}

#[test]
fn a_bad_or_missing_file_is_empty_history() {
    for text in [
        "",
        "not json",
        "[1, 2]",
        r#"{"entries": 5}"#,
        r#"{"version": 1}"#,
    ] {
        assert_eq!(
            History::parse(text, NOW, &origin()),
            History::default(),
            "{text}"
        );
    }
}

#[test]
fn bad_entries_and_visits_are_skipped_alone() {
    let text = r#"{"version": 1, "entries": [
        {"url": "https://ok.test/#x", "title": "OK", "host": "wrong", "visitCount": 1,
         "visits": [{"at": 5, "kind": "typed"}, {"at": "soon", "kind": "link"}, {"at": 7, "kind": "teleport"}, {"at": 3, "kind": "link"}]},
        {"url": "javascript:alert(1)", "visits": [{"at": 1, "kind": "link"}]},
        {"url": "tauri://localhost/index.html", "visits": [{"at": 1, "kind": "link"}]},
        {"url": "https://novisits.test/", "visits": []},
        {"title": "no url", "visits": [{"at": 1, "kind": "link"}]},
        {"url": "https://ok.test/", "title": "duplicate", "visits": [{"at": 9, "kind": "link"}]},
        42
    ]}"#;
    let h = History::parse(text, NOW, &origin());
    let [entry] = h.entries() else {
        panic!("one entry, got {:?}", h.entries());
    };
    assert_eq!(entry.url, "https://ok.test/");
    assert_eq!(entry.title, "OK");
    assert_eq!(entry.host, "ok.test", "recomputed");
    assert_eq!(
        entry.visits,
        [visit(3, VisitKind::Link), visit(5, VisitKind::Typed)]
    );
    assert_eq!(entry.visit_count, 2, "at least the visits it has");
}

#[test]
fn the_file_round_trips() {
    let h = history(&[
        (
            "https://a.test/x?y=1",
            "A \"quoted\"",
            &[
                visit(NOW - DAY, VisitKind::Typed),
                visit(NOW, VisitKind::Reload),
            ],
        ),
        ("https://www.b.test/", "B", &[visit(NOW, VisitKind::Link)]),
    ]);
    let json = h.to_json();
    assert!(json.contains(r#""version":1"#), "{json}");
    assert!(json.contains(r#""visitCount":2"#), "{json}");
    assert!(
        json.contains(r#"{"at":86400000000,"kind":"reload"}"#),
        "{json}"
    );
    assert_eq!(History::parse(&json, NOW, &origin()), h);
}
