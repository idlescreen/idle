// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Pure doctor decision rules (unit-tested; no D-Bus).
//!
//! Regression guards for: daemon-down, idle disabled, inhibited marked FAIL
//! so "ALL SYSTEMS NOMINAL" cannot lie while savers cannot run.

use super::doctor_checks::{CheckResult, Severity, fail, ok};

/// D-Bus status interpretation after a successful GetStatus.
pub fn dbus_status_check(
    idle_enabled: bool,
    idle_timeout_mins: u32,
    active_saver: &str,
) -> CheckResult {
    if !idle_enabled {
        return fail(
            "D-Bus Service",
            format!(
                "connected but idle is DISABLED — savers will not start (timeout={}m saver='{}')",
                idle_timeout_mins, active_saver
            ),
        )
        .with_fix("idlescreen enable");
    }
    ok(
        "D-Bus Service",
        format!(
            "connected (io.github.idlescreen.Idle) idle_enabled=true timeout={}m saver='{}'",
            idle_timeout_mins, active_saver
        ),
    )
}

/// When the daemon is not reachable on the session bus.
pub fn dbus_disconnected_check() -> CheckResult {
    if std::env::var("DBUS_SESSION_BUS_ADDRESS").is_err()
        && std::env::var("XDG_RUNTIME_DIR").is_err()
    {
        return fail(
            "D-Bus Service",
            "no session bus (DBUS_SESSION_BUS_ADDRESS and XDG_RUNTIME_DIR unset) — likely not a graphical session",
        )
        .with_fix("run inside the desktop session, or export DBUS_SESSION_BUS_ADDRESS");
    }
    fail(
        "D-Bus Service",
        "cannot connect — idle-daemon is not running",
    )
    .with_fix("systemctl --user start idle-daemon  (or: idlescreen doctor --fix)")
}

/// Inhibitor / readiness for idle savers.
///
/// - Not connected → FAIL (cannot claim idle path is healthy)
/// - Connected + inhibited → FAIL (savers will not start)
/// - Connected + uninhibited → ok
pub fn inhibitor_status_check(
    daemon_connected: bool,
    inhibited: bool,
    sources: &[String],
) -> CheckResult {
    if !daemon_connected {
        return fail(
            "Inhibitor Status",
            "cannot check — idle-daemon not connected",
        )
        .with_fix("systemctl --user start idle-daemon");
    }
    if inhibited {
        let detail = if sources.is_empty() {
            "INHIBITED — an app/system is blocking idle; savers will not start".to_string()
        } else {
            format!("INHIBITED — {}; savers will not start", sources.join("; "))
        };
        return fail("Inhibitor Status", detail).with_fix("idlescreen inhibitors");
    }
    ok("Inhibitor Status", "uninhibited (idle can trigger savers)")
}

/// True only when no check failed — WARNs do not block NOMINAL.
pub fn all_systems_nominal(results: &[CheckResult]) -> bool {
    results.iter().all(|r| r.severity != Severity::Fail)
}

/// (fails, warns) for the summary line.
pub fn tally(results: &[CheckResult]) -> (usize, usize) {
    let fails = results
        .iter()
        .filter(|r| r.severity == Severity::Fail)
        .count();
    let warns = results
        .iter()
        .filter(|r| r.severity == Severity::Warn)
        .count();
    (fails, warns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dbus_disabled_is_fail_not_nominal() {
        let r = dbus_status_check(false, 5, "ripple");
        assert!(!r.passed());
        assert!(r.detail.contains("DISABLED"));
        assert!(!all_systems_nominal(&[r]));
    }

    #[test]
    fn dbus_enabled_is_ok() {
        let r = dbus_status_check(true, 5, "ripple");
        assert!(r.passed());
        assert!(r.detail.contains("idle_enabled=true"));
    }

    #[test]
    fn dbus_disconnected_is_fail() {
        let r = dbus_disconnected_check();
        assert!(!r.passed());
        assert!(r.detail.contains("not running"));
    }

    #[test]
    fn inhibitor_blocked_is_fail() {
        // Regression: previously marked [ok] with INHIBITED text → false NOMINAL.
        let r = inhibitor_status_check(true, true, &[]);
        assert!(!r.passed(), "inhibited must not pass doctor");
        assert!(r.detail.contains("INHIBITED"));
        assert!(!all_systems_nominal(&[r]));
    }

    #[test]
    fn inhibitor_clear_is_ok() {
        let r = inhibitor_status_check(true, false, &[]);
        assert!(r.passed());
    }

    #[test]
    fn inhibitor_daemon_down_is_fail() {
        // Regression: "idle daemon not connected" was previously [ok].
        let r = inhibitor_status_check(false, false, &[]);
        assert!(!r.passed());
        assert!(r.detail.contains("not connected"));
    }

    #[test]
    fn nominal_requires_all_pass() {
        let ok = ok("A", "fine");
        let bad = fail("B", "nope");
        assert!(all_systems_nominal(std::slice::from_ref(&ok)));
        assert!(!all_systems_nominal(&[ok, bad]));
    }

    #[test]
    fn inhibited_alone_blocks_nominal_footer() {
        // Regression: morning bug — doctor printed ALL SYSTEMS NOMINAL while
        // inhibited (Grok logind / media) because inhibitor check was [ok].
        let results = [
            dbus_status_check(true, 5, "beams"),
            inhibitor_status_check(true, true, &[]),
            ok("Wayland", "ok"),
            ok("systemd", "active"),
        ];
        assert!(results[0].passed());
        assert!(!results[1].passed());
        assert!(
            !all_systems_nominal(&results),
            "must not claim NOMINAL while inhibited"
        );
    }

    #[test]
    fn daemon_down_blocks_nominal_even_if_other_ok() {
        let results = [
            dbus_disconnected_check(),
            inhibitor_status_check(false, false, &[]),
            ok("Package", "installed"),
        ];
        assert!(!all_systems_nominal(&results));
    }

    #[test]
    fn healthy_uninhibited_can_be_nominal() {
        let results = [
            dbus_status_check(true, 10, "ripple"),
            inhibitor_status_check(true, false, &[]),
            ok("Wayland", "session"),
        ];
        assert!(all_systems_nominal(&results));
    }

    #[test]
    fn golden_dbus_disconnected_message() {
        let r = dbus_disconnected_check();
        assert_eq!(r.name, "D-Bus Service");
        assert!(!r.passed());
        // Session-bus state decides which detail/fix we get; both are valid.
        let fix = r.fix.as_deref().unwrap_or("");
        assert!(
            r.detail.contains("idle-daemon is not running") || r.detail.contains("no session bus")
        );
        assert!(fix.contains("systemctl --user start idle-daemon") || fix.contains("DBUS"));
    }

    #[test]
    fn golden_inhibitor_blocked_message() {
        let r = inhibitor_status_check(true, true, &[]);
        assert_eq!(r.name, "Inhibitor Status");
        assert!(!r.passed());
        assert!(r.detail.contains("INHIBITED"));
        assert_eq!(r.fix.as_deref(), Some("idlescreen inhibitors"));
    }

    #[test]
    fn inhibited_names_listed_sources() {
        // Regression: doctor blamed "an app/system" while the daemon's own
        // battery policy was the cause and `inhibitors` listed nothing.
        let sources = vec!["battery: on battery power — savers suppressed".to_string()];
        let r = inhibitor_status_check(true, true, &sources);
        assert!(!r.passed());
        assert!(r.detail.contains("battery"));
        assert!(r.detail.contains("on battery power"));
    }

    #[test]
    fn golden_dbus_disabled_message() {
        let r = dbus_status_check(false, 5, "beams");
        assert!(!r.passed());
        assert!(r.detail.contains("DISABLED"));
        assert_eq!(r.fix.as_deref(), Some("idlescreen enable"));
    }

    #[test]
    fn empty_results_is_nominal() {
        assert!(all_systems_nominal(&[]));
    }
}
