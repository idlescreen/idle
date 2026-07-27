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

// ---------------------------------------------------------------------------
// Regression: Firefox stale ScreenSaver inhibitors (idle-daemon 2.5.12)
//
// Old bug: ScreenSaverService.add (real cookie 1..) AND bus sniffer
// add_with_cookie (phantom 10000+) for the same Inhibit. Firefox UnInhibits
// only the real cookie → phantoms block idle forever after Firefox exits.
// ---------------------------------------------------------------------------

#[test]
fn firefox_playing_video_coalesce_under_spam() {
    // Firefox often calls Inhibit many times for the same reason.
    let s = InhibitorState::new();
    let c = client(":1.145");
    let mut cookies = Vec::new();
    for _ in 0..50 {
        cookies.push(
            s.add(
                "org.mozilla.firefox".into(),
                "Playing video".into(),
                c.clone(),
            )
            .unwrap(),
        );
    }
    assert_eq!(s.len(), 1, "spam Inhibit must not stack holds");
    assert!(cookies.iter().all(|k| *k == cookies[0]));
    assert!(s.is_inhibited());
    assert!(s.remove_for_client(cookies[0], &c));
    assert!(!s.is_inhibited());
}

#[test]
fn firefox_double_count_uninhibit_leaves_phantom() {
    // Documents the *old* failure mode if a sniffer still added phantoms.
    // After UnInhibit of the real cookie, prune must clear orphans.
    let s = InhibitorState::new();
    let c = client(":1.145");
    let real = s
        .add(
            "org.mozilla.firefox".into(),
            "Playing video".into(),
            c.clone(),
        )
        .unwrap();
    // Simulated sniffer phantom (cookie Firefox never sees / never UnInhibits).
    s.add_with_cookie(
        "org.mozilla.firefox".into(),
        "Playing video".into(),
        c.clone(),
        10000,
    );
    // With coalesce, add_with_cookie of same app/reason must NOT stack.
    assert_eq!(
        s.len(),
        1,
        "phantom with same app/reason must coalesce away: len={}",
        s.len()
    );
    // Distinct phantom reason (old sniffer always used same reason — covered above).
    // Force a second entry as if sniffer used a different cookie-only path without coalesce:
    // use audio reason then prune.
    s.add_with_cookie(
        "org.mozilla.firefox".into(),
        "Playing audio".into(),
        c.clone(),
        10001,
    );
    assert_eq!(s.len(), 2);
    assert!(s.remove_for_client(real, &c));
    // Still inhibited by phantom audio hold.
    assert!(s.is_inhibited());
    // Browser exited: unique name gone from bus → prune clears rest.
    let live = std::collections::HashSet::new();
    let n = s.prune_not_in_live_set(&live);
    assert!(n >= 1);
    assert!(!s.is_inhibited(), "after exit+prune must be free to idle");
}

#[test]
fn firefox_prune_after_exit_clears_all_holds() {
    let s = InhibitorState::new();
    let ff = client(":1.200");
    let other = client(":1.50");
    for reason in ["Playing video", "Playing audio"] {
        let _ = s
            .add("org.mozilla.firefox".into(), reason.into(), ff.clone())
            .unwrap();
    }
    let _ = s
        .add("vlc".into(), "fullscreen".into(), other.clone())
        .unwrap();
    assert_eq!(s.len(), 3);
    // Firefox gone; VLC still live.
    let mut live = std::collections::HashSet::new();
    live.insert(":1.50".to_string());
    let n = s.prune_not_in_live_set(&live);
    assert_eq!(n, 2);
    assert_eq!(s.len(), 1);
    assert!(s.is_inhibited());
    // VLC also gone.
    let n = s.prune_not_in_live_set(&std::collections::HashSet::new());
    assert_eq!(n, 1);
    assert!(!s.is_inhibited());
}

#[test]
fn firefox_uninhibit_real_cookie_clears_when_no_phantom() {
    // Correct single-path (service only): UnInhibit is enough.
    let s = InhibitorState::new();
    let c = client(":1.145");
    let cookie = s
        .add(
            "org.mozilla.firefox".into(),
            "Playing video".into(),
            c.clone(),
        )
        .unwrap();
    assert!(
        cookie < 10000,
        "service cookies start at 1, not sniffer range"
    );
    assert!(s.remove_for_client(cookie, &c));
    assert_eq!(s.len(), 0);
    assert!(!s.is_inhibited());
}

#[test]
fn remove_client_by_unique_name_string_eq() {
    // NameOwnerChanged path: UniqueName compare via as_str.
    let s = InhibitorState::new();
    let c = client(":1.145");
    let _ = s
        .add(
            "org.mozilla.firefox".into(),
            "Playing video".into(),
            c.clone(),
        )
        .unwrap();
    s.remove_client(&client(":1.145"));
    assert!(!s.is_inhibited());
}
