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
fn cookies_are_unique_and_increasing_for_distinct_reasons() {
    let s = InhibitorState::new();
    let c = client(":test.app.Cookie");
    let k1 = s.add("a".to_string(), "r1".to_string(), c.clone()).unwrap();
    let k2 = s.add("a".to_string(), "r2".to_string(), c.clone()).unwrap();
    let k3 = s.add("a".to_string(), "r3".to_string(), c.clone()).unwrap();
    assert!(k1 < k2);
    assert!(k2 < k3);
}

#[test]
fn add_coalesces_same_client_app_reason() {
    let s = InhibitorState::new();
    let c = client(":test.app.Coalesce");
    let k1 = s
        .add("firefox".into(), "Playing video".into(), c.clone())
        .unwrap();
    let k2 = s
        .add("firefox".into(), "Playing video".into(), c.clone())
        .unwrap();
    assert_eq!(k1, k2, "duplicate Inhibit must reuse cookie");
    assert_eq!(s.len(), 1);
}

#[test]
fn prune_not_in_live_set_drops_dead_peers() {
    let s = InhibitorState::new();
    let live = client(":1.100");
    let dead = client(":1.999");
    let _ = s
        .add("firefox".into(), "Playing video".into(), live.clone())
        .unwrap();
    let _ = s
        .add("firefox".into(), "Playing video".into(), dead.clone())
        .unwrap();
    assert_eq!(s.len(), 2);
    let mut set = std::collections::HashSet::new();
    set.insert(":1.100".to_string());
    let n = s.prune_not_in_live_set(&set);
    assert_eq!(n, 1);
    assert_eq!(s.len(), 1);
    assert!(s.is_inhibited());
    set.clear();
    let n = s.prune_not_in_live_set(&set);
    assert_eq!(n, 1);
    assert!(!s.is_inhibited());
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
fn merge_includes_real_logind_external() {
    // Real media/fullscreen holds still list (not Grok agent-turn).
    use super::external::ExternalInhibitor;
    use super::merge_inhibitor_rows;

    let local = vec![(1u32, "app".into(), "reason".into())];
    let external = vec![ExternalInhibitor {
        source: "logind".into(),
        who: "vlc (block)".into(),
        why: "playing video".into(),
    }];
    let rows = merge_inhibitor_rows(local, &external);
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter().any(|(k, app, why)| {
            *k == 0 && app == "logind:vlc (block)" && why == "playing video"
        }),
        "vlc logind row missing: {rows:?}"
    );
}

#[test]
fn merge_drops_grok_agent_turn_logind() {
    use super::external::ExternalInhibitor;
    use super::merge_inhibitor_rows;

    let rows = merge_inhibitor_rows(
        vec![],
        &[
            ExternalInhibitor {
                source: "logind".into(),
                who: "grok (block)".into(),
                why: "agent turn in progress".into(),
            },
            ExternalInhibitor {
                source: "mpris".into(),
                who: "spotify".into(),
                why: "PlaybackStatus=Playing".into(),
            },
        ],
    );
    assert_eq!(rows.len(), 1, "only mpris should remain: {rows:?}");
    assert!(rows[0].1.starts_with("mpris:"));
    assert!(!rows.iter().any(|(_, app, _)| app.contains("grok")));
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

#[test]
fn merge_preserves_local_then_external_order() {
    use super::external::ExternalInhibitor;
    use super::merge_inhibitor_rows;

    let rows = merge_inhibitor_rows(
        vec![
            (2u32, "app-a".into(), "r1".into()),
            (5u32, "app-b".into(), "r2".into()),
        ],
        &[
            ExternalInhibitor {
                source: "logind".into(),
                who: "agent".into(),
                why: "busy".into(),
            },
            ExternalInhibitor {
                source: "mpris".into(),
                who: "player".into(),
                why: "Playing".into(),
            },
        ],
    );
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].0, 2);
    assert_eq!(rows[1].0, 5);
    assert_eq!(rows[2].1, "logind:agent");
    assert_eq!(rows[3].1, "mpris:player");
}

#[test]
fn merge_empty_local_and_external_is_empty() {
    use super::merge_inhibitor_rows;
    let rows = merge_inhibitor_rows(vec![], &[]);
    assert!(rows.is_empty());
}
