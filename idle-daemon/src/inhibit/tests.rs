// SPDX-License-Identifier: MIT

use super::*;
use zbus::names::UniqueName;

fn client(name: &str) -> UniqueName<'static> {
    UniqueName::try_from(name.to_string()).unwrap()
}

#[test]
fn inhibitor_state_starts_uninhibited() {
    let s = InhibitorState::new();
    assert!(!s.is_inhibited());
}

#[test]
fn add_inhibitor_marks_inhibited() {
    let s = InhibitorState::new();
    let c = client(":test.app.Inhibitor");
    let cookie = s
        .add("app".to_string(), "reason".to_string(), c.clone())
        .unwrap();
    assert!(cookie > 0);
    assert!(s.is_inhibited());
}

#[test]
fn remove_for_client_clears_inhibition() {
    let s = InhibitorState::new();
    let c = client(":test.app.Inhibitor");
    let cookie = s
        .add("app".to_string(), "reason".to_string(), c.clone())
        .unwrap();
    assert!(s.remove_for_client(cookie, &c));
    assert!(!s.is_inhibited());
}

#[test]
fn remove_for_client_wrong_cookie_returns_false() {
    let s = InhibitorState::new();
    let c = client(":test.app.Inhibitor");
    let _ = s.add("app".to_string(), "reason".to_string(), c.clone());
    assert!(!s.remove_for_client(9999, &c));
    assert!(s.is_inhibited());
}

#[test]
fn remove_for_client_wrong_client_returns_false() {
    let s = InhibitorState::new();
    let c1 = client(":test.one.Client");
    let c2 = client(":test.two.Client");
    let cookie = s
        .add("app".to_string(), "reason".to_string(), c1.clone())
        .unwrap();
    assert!(!s.remove_for_client(cookie, &c2));
    assert!(s.is_inhibited());
}

#[test]
fn remove_client_clears_all_for_that_client() {
    let s = InhibitorState::new();
    let c1 = client(":test.one.Client");
    let c2 = client(":test.two.Client");
    let _ = s
        .add("app".to_string(), "reason1".to_string(), c1.clone())
        .unwrap();
    let _ = s
        .add("app".to_string(), "reason2".to_string(), c1.clone())
        .unwrap();
    let _ = s
        .add("app".to_string(), "reason3".to_string(), c2.clone())
        .unwrap();
    s.remove_client(&c1);
    assert!(s.is_inhibited()); // c2 still holds an inhibitor
    s.remove_client(&c2);
    assert!(!s.is_inhibited());
}

#[test]
fn cookies_are_unique_and_increasing() {
    let s = InhibitorState::new();
    let c = client(":test.app.Cookie");
    let k1 = s.add("a".to_string(), "r".to_string(), c.clone()).unwrap();
    let k2 = s.add("a".to_string(), "r".to_string(), c.clone()).unwrap();
    let k3 = s.add("a".to_string(), "r".to_string(), c.clone()).unwrap();
    assert!(k1 < k2);
    assert!(k2 < k3);
}

#[test]
fn add_rejects_when_at_capacity_for_one_client() {
    let s = InhibitorState::new();
    let c = client(":test.app.Capacity");
    for i in 0..32 {
        assert!(
            s.add("a".to_string(), format!("r{i}"), c.clone()).is_ok(),
            "expected add {i} to succeed"
        );
    }
    // 33rd should be rejected (per-cap of 32)
    assert!(s.add("a".to_string(), "r".to_string(), c.clone()).is_err());
}

#[test]
fn list_all_includes_local_cookies() {
    let s = InhibitorState::new();
    let c = client(":test.app.List");
    let cookie = s.add("myapp".into(), "fullscreen".into(), c).expect("add");
    let rows = s.list_all();
    assert!(
        rows.iter()
            .any(|(k, app, why)| *k == cookie && app == "myapp" && why == "fullscreen"),
        "local cookie missing from list_all: {rows:?}"
    );
}

#[test]
fn merge_includes_logind_grok_style_external() {
    // Regression: status.inhibited true while CLI showed empty list.
    use super::external::ExternalInhibitor;
    use super::merge_inhibitor_rows;

    let local = vec![(1u32, "app".into(), "reason".into())];
    let external = vec![ExternalInhibitor {
        source: "logind".into(),
        who: "grok (block)".into(),
        why: "agent turn in progress".into(),
    }];
    let rows = merge_inhibitor_rows(local, &external);
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter().any(|(k, app, why)| {
            *k == 0 && app == "logind:grok (block)" && why == "agent turn in progress"
        }),
        "grok logind row missing: {rows:?}"
    );
}

#[test]
fn merge_external_only_not_empty() {
    use super::external::ExternalInhibitor;
    use super::merge_inhibitor_rows;

    let rows = merge_inhibitor_rows(
        vec![],
        &[ExternalInhibitor {
            source: "mpris".into(),
            who: "firefox".into(),
            why: "PlaybackStatus=Playing".into(),
        }],
    );
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, 0);
    assert!(rows[0].1.starts_with("mpris:"));
}
