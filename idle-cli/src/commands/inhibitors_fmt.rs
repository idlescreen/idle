// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Pure formatting for `idlescreen inhibitors` (unit-tested).

/// Format the inhibitors report printed by the CLI.
///
/// Rows: `(cookie, application, reason)`. Cookie `0` means external
/// (`logind:…` / `mpris:…` prefixes in `application`).
pub fn format_inhibitors_report(inhibited: bool, rows: &[(u32, String, String)]) -> String {
    let mut out = String::new();
    if rows.is_empty() {
        if inhibited {
            out.push_str("inhibited: true\n");
            out.push_str("No listed inhibitors (unknown external block — see doctor).\n");
        } else {
            out.push_str("inhibited: false\n");
            out.push_str("No active inhibitors.\n");
        }
        return out;
    }
    let n = rows.len();
    out.push_str(&format!(
        "inhibited: {}  ({} source{})\n",
        inhibited,
        n,
        if n == 1 { "" } else { "s" }
    ));
    out.push_str("Active inhibitors (block idle savers; forced preview ignores these):\n");
    for (cookie, app, reason) in rows {
        if *cookie == 0 {
            out.push_str(&format!("  [{app}] {reason}\n"));
        } else {
            out.push_str(&format!("  [idlescreen cookie {cookie}] {app}: {reason}\n"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_uninhibited() {
        let s = format_inhibitors_report(false, &[]);
        assert!(s.contains("inhibited: false"));
        assert!(s.contains("No active inhibitors"));
    }

    #[test]
    fn empty_but_inhibited_is_honest() {
        // Must not claim "no inhibitors" without admitting blocked.
        let s = format_inhibitors_report(true, &[]);
        assert!(s.contains("inhibited: true"));
        assert!(!s.contains("No active inhibitors."));
        assert!(s.contains("unknown external") || s.contains("listed"));
    }

    #[test]
    fn logind_media_row_and_preview_note() {
        // Real external holds list; forced preview is not blocked by them.
        let rows = vec![(0u32, "logind:vlc (block)".into(), "playing video".into())];
        let s = format_inhibitors_report(true, &rows);
        assert!(s.contains("inhibited: true"));
        assert!(s.contains("logind:vlc"));
        assert!(s.contains("playing video"));
        assert!(s.contains("forced preview ignores"));
        assert!(!s.contains("can clear preview"));
        assert!(!s.contains("cookie 0"));
    }

    #[test]
    fn idlescreen_cookie_row() {
        let rows = vec![(3u32, "myapp".into(), "fullscreen".into())];
        let s = format_inhibitors_report(true, &rows);
        assert!(s.contains("idlescreen cookie 3"));
        assert!(s.contains("myapp"));
    }

    #[test]
    fn mixed_sources_plural() {
        let rows = vec![
            (1u32, "app".into(), "r".into()),
            (
                0u32,
                "mpris:firefox".into(),
                "PlaybackStatus=Playing".into(),
            ),
        ];
        let s = format_inhibitors_report(true, &rows);
        assert!(s.contains("2 sources"));
        assert!(s.contains("mpris:firefox"));
    }

    #[test]
    fn single_source_singular_label() {
        let rows = vec![(0u32, "logind:vlc (block)".into(), "playing video".into())];
        let s = format_inhibitors_report(true, &rows);
        assert!(s.contains("1 source"));
        assert!(!s.contains("1 sources"));
    }

    #[test]
    fn inhibited_false_with_stale_row_still_lists() {
        // Defensive: list truth even if inhibited flag disagrees.
        let rows = vec![(1u32, "app".into(), "why".into())];
        let s = format_inhibitors_report(false, &rows);
        assert!(s.contains("inhibited: false"));
        assert!(s.contains("idlescreen cookie 1"));
    }

    #[test]
    fn golden_empty_uninhibited_exact() {
        let s = format_inhibitors_report(false, &[]);
        assert_eq!(s, "inhibited: false\nNo active inhibitors.\n");
    }

    #[test]
    fn golden_empty_inhibited_exact() {
        let s = format_inhibitors_report(true, &[]);
        assert_eq!(
            s,
            "inhibited: true\nNo listed inhibitors (unknown external block — see doctor).\n"
        );
    }

    #[test]
    fn golden_single_external_includes_preview_ignore_header() {
        let rows = [(0u32, "mpris:mpv".into(), "PlaybackStatus=Playing".into())];
        let s = format_inhibitors_report(true, &rows);
        assert!(s.starts_with("inhibited: true  (1 source)\n"));
        assert!(s.contains("forced preview ignores these"));
        assert!(s.contains("[mpris:mpv] PlaybackStatus=Playing\n"));
        assert!(!s.contains("cookie 0"));
    }
}
