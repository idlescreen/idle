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
    out.push_str("Active inhibitors (block idle / can clear preview):\n");
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
    fn logind_grok_row() {
        // Regression: Grok logind idle must appear by name.
        let rows = vec![(
            0u32,
            "logind:grok (block)".into(),
            "agent turn in progress".into(),
        )];
        let s = format_inhibitors_report(true, &rows);
        assert!(s.contains("inhibited: true"));
        assert!(s.contains("logind:grok"));
        assert!(s.contains("agent turn in progress"));
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
}
