// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Pure doctor decision rules (unit-tested; no D-Bus).
//!
//! Regression guards for: daemon-down, idle disabled, inhibited marked FAIL
//! so "ALL SYSTEMS NOMINAL" cannot lie while savers cannot run.

use super::doctor_checks::{CheckResult, chk};

/// D-Bus status interpretation after a successful GetStatus.
pub fn dbus_status_check(
    idle_enabled: bool,
    idle_timeout_mins: u32,
    active_saver: &str,
) -> CheckResult {
    if !idle_enabled {
        return chk(
            "D-Bus Service",
            false,
            format!(
                "connected but idle is DISABLED — savers will not start (timeout={}m saver='{}'); enable with: idlescreen enable",
                idle_timeout_mins, active_saver
            ),
        );
    }
    chk(
        "D-Bus Service",
        true,
        format!(
            "connected (io.github.idlescreen.Idle) idle_enabled=true timeout={}m saver='{}'",
            idle_timeout_mins, active_saver
        ),
    )
}

/// When the daemon is not reachable on the session bus.
pub fn dbus_disconnected_check() -> CheckResult {
    chk(
        "D-Bus Service",
        false,
        "cannot connect — idle-daemon is not running; start it: systemctl --user start idle-daemon  (or: idlescreen doctor --fix)",
    )
}

/// Inhibitor / readiness for idle savers.
///
/// - Not connected → FAIL (cannot claim idle path is healthy)
/// - Connected + inhibited → FAIL (savers will not start)
/// - Connected + uninhibited → ok
pub fn inhibitor_status_check(daemon_connected: bool, inhibited: bool) -> CheckResult {
    if !daemon_connected {
        return chk(
            "Inhibitor Status",
            false,
            "cannot check — idle-daemon not connected (start the daemon first)",
        );
    }
    if inhibited {
        return chk(
            "Inhibitor Status",
            false,
            "INHIBITED — an app/system is blocking idle; savers will not start (try: idlescreen inhibitors)",
        );
    }
    chk(
        "Inhibitor Status",
        true,
        "uninhibited (idle can trigger savers)",
    )
}

/// True only when every check passed — used for NOMINAL footer / exit code.
pub fn all_systems_nominal(results: &[CheckResult]) -> bool {
    results.iter().all(|r| r.passed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dbus_disabled_is_fail_not_nominal() {
        let r = dbus_status_check(false, 5, "ripple");
        assert!(!r.passed);
        assert!(r.detail.contains("DISABLED"));
        assert!(!all_systems_nominal(&[r]));
    }

    #[test]
    fn dbus_enabled_is_ok() {
        let r = dbus_status_check(true, 5, "ripple");
        assert!(r.passed);
        assert!(r.detail.contains("idle_enabled=true"));
    }

    #[test]
    fn dbus_disconnected_is_fail() {
        let r = dbus_disconnected_check();
        assert!(!r.passed);
        assert!(r.detail.contains("not running"));
    }

    #[test]
    fn inhibitor_blocked_is_fail() {
        // Regression: previously marked [ok] with INHIBITED text → false NOMINAL.
        let r = inhibitor_status_check(true, true);
        assert!(!r.passed, "inhibited must not pass doctor");
        assert!(r.detail.contains("INHIBITED"));
        assert!(!all_systems_nominal(&[r]));
    }

    #[test]
    fn inhibitor_clear_is_ok() {
        let r = inhibitor_status_check(true, false);
        assert!(r.passed);
    }

    #[test]
    fn inhibitor_daemon_down_is_fail() {
        // Regression: "idle daemon not connected" was previously [ok].
        let r = inhibitor_status_check(false, false);
        assert!(!r.passed);
        assert!(r.detail.contains("not connected"));
    }

    #[test]
    fn nominal_requires_all_pass() {
        let ok = chk("A", true, "fine");
        let bad = chk("B", false, "nope");
        assert!(all_systems_nominal(&[ok.clone()]));
        assert!(!all_systems_nominal(&[ok, bad]));
    }
}
