use super::*;

fn start(query: &str) -> FindAction {
    FindAction::Start {
        query: query.to_owned(),
    }
}

/// An open find on `engine` with `query` started.
fn finding(engine: Engine, query: &str) -> FindState {
    let mut state = FindState::new(engine);
    state.open();
    assert_eq!(
        state.act(start(query)),
        Some(EngineOp::Start(query.to_owned()))
    );
    state
}

#[test]
fn the_bar_sends_one_of_four_actions_and_nothing_else() {
    let parse = |json: serde_json::Value| serde_json::from_value::<FindAction>(json);
    assert_eq!(
        parse(serde_json::json!({ "kind": "start", "query": "lj-needle" })).expect("start"),
        start("lj-needle")
    );
    assert_eq!(
        parse(serde_json::json!({ "kind": "next" })).expect("next"),
        FindAction::Next {}
    );
    assert_eq!(
        parse(serde_json::json!({ "kind": "previous" })).expect("previous"),
        FindAction::Previous {}
    );
    assert_eq!(
        parse(serde_json::json!({ "kind": "stop" })).expect("stop"),
        FindAction::Stop {}
    );
    for bad in [
        serde_json::json!({ "kind": "regex", "query": "a.*" }),
        serde_json::json!({ "kind": "start" }),
        serde_json::json!({ "kind": "start", "query": "a", "caseSensitive": true }),
        serde_json::json!({ "kind": "next", "wholeWord": true }),
        serde_json::json!("next"),
    ] {
        assert!(parse(bad.clone()).is_err(), "{bad}");
    }
}

#[test]
fn a_query_is_at_most_a_thousand_characters() {
    let long = "é".repeat(MAX_QUERY + 50);
    let mut state = FindState::new(Engine::WebView2);
    state.open();
    let Some(EngineOp::Start(sent)) = state.act(start(&long)) else {
        panic!("a start");
    };
    assert_eq!(sent.chars().count(), MAX_QUERY);
    assert_eq!(state.query().map(|q| q.chars().count()), Some(MAX_QUERY));
    assert_eq!(cut_query("lj-needle"), "lj-needle");
    // The log says how long, never what.
    assert_eq!(start("lj-needle").log_detail(), "start length=9");
    assert_eq!(FindAction::Next {}.log_detail(), "next");
}

#[test]
fn every_find_ignores_case_and_wraps() {
    const { assert!(!CASE_SENSITIVE) };
    const { assert!(WRAPS) };
}

#[test]
fn nothing_is_found_while_the_strip_is_closed() {
    let mut state = FindState::new(Engine::WebView2);
    assert_eq!(state.act(start("late")), None, "a query typed as it closed");
    assert_eq!(state.act(FindAction::Next {}), None);
    assert_eq!(state.act(FindAction::Previous {}), None);
    assert_eq!(state.act(FindAction::Stop {}), None);
    assert_eq!(state.report(None, Report::Count(3)), None, "no find runs");
}

#[test]
fn next_and_previous_need_a_query_and_carry_it() {
    let mut state = FindState::new(Engine::WebKit);
    state.open();
    assert_eq!(state.act(FindAction::Next {}), None, "nothing typed yet");
    assert_eq!(
        state.act(start("needle")),
        Some(EngineOp::Start("needle".into()))
    );
    assert_eq!(
        state.act(FindAction::Next {}),
        Some(EngineOp::Next("needle".into()))
    );
    assert_eq!(
        state.act(FindAction::Previous {}),
        Some(EngineOp::Previous("needle".into()))
    );
}

#[test]
fn a_new_query_restarts_and_drops_what_the_old_one_found() {
    let mut state = finding(Engine::WebView2, "lj");
    state.report(None, Report::Count(7));
    state.report(None, Report::Active(4));
    let old = state.generation();
    assert_eq!(
        state.act(start("lj-needle")),
        Some(EngineOp::Start("lj-needle".into()))
    );
    assert!(state.generation() > old);
    assert!(!state.known(), "nothing said about the new query yet");
    assert_eq!(state.result(), FindResult::default());
    // A report that says it is for the old find is dropped.
    assert_eq!(state.report(Some(old), Report::Found(true)), None);
}

#[test]
fn an_empty_query_clears_the_find_and_keeps_the_strip() {
    let mut state = finding(Engine::WebKitGtk, "needle");
    assert_eq!(state.act(start("")), Some(EngineOp::Stop));
    assert!(state.is_open());
    assert_eq!(state.query(), None);
    assert_eq!(state.act(start("")), None, "nothing to clear twice");
}

#[test]
fn stop_clears_everything_and_closes_the_strip() {
    let mut state = finding(Engine::WebView2, "needle");
    state.report(None, Report::Count(3));
    assert_eq!(state.act(FindAction::Stop {}), Some(EngineOp::Stop));
    assert!(!state.is_open());
    assert_eq!(state.query(), None);
    assert_eq!(state.result(), FindResult::default());
    assert_eq!(state.report(None, Report::Count(3)), None, "late reports");
}

#[test]
fn a_new_page_ends_the_find() {
    let mut state = finding(Engine::WebKitGtk, "needle");
    assert_eq!(
        state.end(),
        Ended {
            was_open: true,
            stop_engine: true
        }
    );
    assert_eq!(
        state.end(),
        Ended {
            was_open: false,
            stop_engine: false
        },
        "once"
    );
    // The strip open with nothing typed: it closes, the engine has nothing to stop.
    let mut open = FindState::new(Engine::WebView2);
    open.open();
    assert_eq!(
        open.end(),
        Ended {
            was_open: true,
            stop_engine: false
        }
    );
}

#[test]
fn webview2_counts_and_knows_the_active_match() {
    let mut state = finding(Engine::WebView2, "lj-needle");
    let counted = state.report(None, Report::Count(3)).expect("changed");
    assert_eq!(
        counted,
        FindResult {
            matches: Some(3),
            active: None,
            found: true,
            unavailable: false
        }
    );
    let active = state.report(None, Report::Active(1)).expect("changed");
    assert_eq!(active.log_line(), "matches=3 active=1");
    assert_eq!(state.report(None, Report::Active(1)), None, "no change");
    // Next: the engine moves and says so.
    assert_eq!(
        state.act(FindAction::Next {}),
        Some(EngineOp::Next("lj-needle".into()))
    );
    assert_eq!(state.result().active, Some(1), "not until the engine says");
    let next = state.report(None, Report::Active(2)).expect("changed");
    assert_eq!(next.log_line(), "matches=3 active=2");
    // No matches.
    let none = state.report(None, Report::Count(0)).expect("changed");
    assert_eq!(
        none,
        FindResult {
            matches: Some(0),
            active: None,
            found: false,
            unavailable: false
        }
    );
    assert_eq!(none.log_line(), "matches=0");
}

#[test]
fn an_old_webview2_has_no_find() {
    let mut state = finding(Engine::WebView2, "lj-needle");
    let result = state.report(None, Report::Unavailable).expect("changed");
    assert!(result.unavailable);
    assert!(!result.found);
    assert_eq!(result.log_line(), "unavailable");
    assert_eq!(
        serde_json::to_value(result).expect("json"),
        serde_json::json!({ "matches": null, "active": null, "found": false, "unavailable": true })
    );
}

#[test]
fn webkitgtk_counts_and_the_active_match_is_tracked_here() {
    let mut state = finding(Engine::WebKitGtk, "lj-needle");
    // found-text, then counted-matches (either order).
    assert_eq!(
        state
            .report(None, Report::Found(true))
            .expect("changed")
            .log_line(),
        "found=true"
    );
    assert_eq!(
        state
            .report(None, Report::Count(3))
            .expect("changed")
            .log_line(),
        "matches=3 active=1"
    );
    let mut other = finding(Engine::WebKitGtk, "lj-needle");
    other.report(None, Report::Count(3));
    other.report(None, Report::Found(true));
    assert_eq!(other.result().log_line(), "matches=3 active=1");
    // Next and previous step it here, wrapping at both ends.
    for want in [2, 3, 1] {
        state.act(FindAction::Next {});
        assert_eq!(state.result().active, Some(want));
    }
    state.act(FindAction::Previous {});
    assert_eq!(state.result().active, Some(3), "back past the first");
    // The engine's found-text after search_next keeps the step.
    assert_eq!(state.report(None, Report::Found(true)), None);
    assert_eq!(state.result().active, Some(3));
    // failed-to-find-text: no active match.
    let failed = state.report(None, Report::Found(false)).expect("changed");
    assert_eq!(failed.active, None);
    assert!(!failed.found);
}

#[test]
fn webkitgtk_beyond_its_count_says_found_without_a_number() {
    let mut state = finding(Engine::WebKitGtk, "e");
    let result = state
        .report(None, Report::Count(u32::MAX))
        .expect("changed");
    assert_eq!(result.matches, None);
    assert!(result.found);
    assert_eq!(result.log_line(), "found=true");
}

#[test]
fn wkwebview_says_only_found_or_not() {
    let mut state = finding(Engine::WebKit, "lj-needle");
    let generation = state.generation();
    let found = state
        .report(Some(generation), Report::Found(true))
        .expect("changed");
    assert_eq!(
        found,
        FindResult {
            matches: None,
            active: None,
            found: true,
            unavailable: false
        }
    );
    assert_eq!(found.log_line(), "found=true");
    // Next finds again: the same result, nothing new to tell.
    state.act(FindAction::Next {});
    assert_eq!(state.report(Some(generation), Report::Found(true)), None);
    let gone = state
        .report(Some(generation), Report::Found(false))
        .expect("changed");
    assert_eq!(gone.log_line(), "found=false");
    // A count from nowhere changes nothing it shows.
    assert_eq!(state.result().matches, None);
}

#[test]
fn the_result_goes_to_the_bar_as_json() {
    let result = FindResult {
        matches: Some(12),
        active: Some(3),
        found: true,
        unavailable: false,
    };
    assert_eq!(
        serde_json::to_value(result).expect("json"),
        serde_json::json!({ "matches": 12, "active": 3, "found": true })
    );
}

#[test]
fn stepping_wraps_only_when_the_count_is_known() {
    assert_eq!(step(3, Some(3), true), 1);
    assert_eq!(step(1, Some(3), false), 3);
    assert_eq!(step(2, Some(3), true), 3);
    assert_eq!(step(2, Some(3), false), 1);
    assert_eq!(step(5, None, true), 6);
    assert_eq!(step(1, None, false), 1);
    assert_eq!(step(4, Some(0), true), 5);
}

#[test]
fn each_os_has_its_engine() {
    let want = if cfg!(windows) {
        Engine::WebView2
    } else if cfg!(target_os = "macos") {
        Engine::WebKit
    } else {
        Engine::WebKitGtk
    };
    assert_eq!(Engine::current(), want);
}
