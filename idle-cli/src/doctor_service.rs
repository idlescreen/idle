// SPDX-License-Identifier: MIT

//! D-Bus, systemd, and process checks for doctor.

use super::doctor_checks::{CheckResult, fail, ok, warn};
use idle_dbus::TranceClient;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn check_dbus() -> CheckResult {
    use super::doctor_rules::{dbus_disconnected_check, dbus_status_check};
    if let Ok(client) = TranceClient::connect() {
        match client.get_status() {
            Ok(status) => dbus_status_check(
                status.idle_enabled,
                status.idle_timeout_mins,
                &status.active_saver,
            ),
            Err(e) => fail("D-Bus Service", format!("GetStatus error: {e}")),
        }
    } else {
        dbus_disconnected_check()
    }
}

/// Savers on disk — and each `.so` must have a `.idleplugin.toml` manifest
/// beside it or the runner will refuse to load it.
pub fn check_savers() -> CheckResult {
    const DIRS: &[&str] = &[
        "/usr/libexec/idle/screensavers",
        "/usr/local/libexec/idle/screensavers",
    ];
    let mut found_so = 0usize;
    let mut missing_manifest = Vec::new();
    let mut found_dir: Option<&str> = None;
    for dir in DIRS {
        let path = Path::new(dir);
        if !path.is_dir() {
            continue;
        }
        found_dir = Some(dir);
        if let Ok(rd) = fs::read_dir(path) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) == Some("so") {
                    found_so += 1;
                    let manifest = p.with_extension("idleplugin.toml");
                    if !manifest.is_file() {
                        missing_manifest.push(
                            p.file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| p.display().to_string()),
                        );
                    }
                }
            }
        }
    }

    if found_so > 0 {
        let detail = format!(
            "{found_so} plugin(s) under {}",
            found_dir.unwrap_or("libexec")
        );
        if missing_manifest.is_empty() {
            return ok("Savers", detail);
        }
        return warn(
            "Savers",
            format!(
                "{detail}; missing manifests: {}",
                missing_manifest.join(", ")
            ),
        )
        .with_fix("each plugin needs a <name>.idleplugin.toml beside the .so");
    }

    if package_installed("idle-savers") || package_installed("idle-saver-beams") {
        return ok("Savers", "idle-savers / idle-saver-* package present");
    }

    if let Ok(home) = std::env::var("HOME") {
        let local = PathBuf::from(home).join(".local/share/idle/screensavers");
        if local.is_dir()
            && fs::read_dir(&local)
                .map(|rd| {
                    rd.flatten()
                        .any(|e| e.path().extension().and_then(|x| x.to_str()) == Some("so"))
                })
                .unwrap_or(false)
        {
            return ok("Savers", "plugins under ~/.local/share/idle/screensavers");
        }
    }

    fail("Savers", "no screensaver plugins found")
        .with_fix("install idle-savers or an idle-saver-* package")
}

/// Optional TUI binary — a WARN, not a silent pass.
pub fn check_tui_optional() -> CheckResult {
    if which_exists("idle-tui") || which_exists("idlescreen-tui") {
        ok("TUI", "idle-tui available on PATH")
    } else {
        warn("TUI", "not installed — optional terminal control UI")
            .with_fix("install idle-tui for the terminal UI")
    }
}

fn which_exists(name: &str) -> bool {
    // .output() captures child stdout — .status() would leak the resolved
    // path into `doctor --json` output.
    Command::new("sh")
        .args(["-c", &format!("command -v {name}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn package_installed(pkg: &str) -> bool {
    // Same leak as which_exists — rpm/dpkg-query print to stdout.
    if Command::new("rpm")
        .args(["-q", pkg])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return true;
    }
    Command::new("dpkg-query")
        .args(["-W", pkg])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `is-active` and `is-enabled` are separate facts — an active service that
/// is not enabled silently dies at next login.
pub fn check_systemd_service() -> CheckResult {
    let active = Command::new("systemctl")
        .args(["--user", "is-active", "idle-daemon.service"])
        .output();
    let enabled = Command::new("systemctl")
        .args(["--user", "is-enabled", "idle-daemon.service"])
        .output();

    let active_state = active
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let enabled_state = enabled
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    match (active_state.as_deref(), enabled_state.as_deref()) {
        (Some("active"), Some("enabled")) => ok("Systemd Service", "active + enabled"),
        (Some("active"), _) => warn(
            "Systemd Service",
            format!(
                "active but enabled='{}' — will not start at login",
                enabled_state.as_deref().unwrap_or("unknown")
            ),
        )
        .with_fix("systemctl --user enable idle-daemon"),
        (Some(state), _) => fail("Systemd Service", format!("is-active reports '{state}'"))
            .with_fix("systemctl --user start idle-daemon  (or: idlescreen doctor --fix)"),
        (None, _) => fail("Systemd Service", "systemctl --user failed")
            .with_fix("check that a user systemd session exists (loginctl)"),
    }
}

/// PID file + liveness + pending-restart detection. After a package upgrade
/// the running daemon is still the old binary — `/proc/<pid>/exe` ends with
/// " (deleted)" and the daemon must be restarted to run the new code.
pub fn check_running_pid() -> CheckResult {
    let pid_path = pid_file_path();
    let dbus_ok = TranceClient::connect().is_ok();

    if pid_path.exists() {
        if let Ok(content) = fs::read_to_string(&pid_path) {
            let pid_str = content.trim();
            if let Ok(pid) = pid_str.parse::<i32>() {
                // SAFETY: kill(pid, 0) only checks process existence.
                if unsafe { libc::kill(pid, 0) } == 0 {
                    if let Some(stale) = exe_deleted_marker(pid) {
                        return fail(
                            "Process Status",
                            format!("PID {pid} running a deleted binary ({stale}) — pre-upgrade daemon still live"),
                        )
                        .with_fix("systemctl --user restart idle-daemon");
                    }
                    return ok("Process Status", format!("PID {pid} running"));
                }
                return fail(
                    "Process Status",
                    format!("stale PID file ({pid} not running)"),
                )
                .with_fix(format!(
                    "rm {}; idlescreen doctor --fix",
                    pid_path.display()
                ));
            }
        }
        return warn("Process Status", "pid file unreadable");
    }
    if dbus_ok {
        return warn("Process Status", "no pid file but D-Bus responds");
    }
    fail("Process Status", "missing pid file").with_fix("systemctl --user start idle-daemon")
}

/// Read `/proc/<pid>/exe`; returns the target when it is marked deleted
/// (the kernel appends " (deleted)" to the link for replaced binaries).
fn exe_deleted_marker(pid: i32) -> Option<String> {
    let target = fs::read_link(format!("/proc/{pid}/exe")).ok()?;
    let s = target.to_string_lossy();
    s.strip_suffix(" (deleted)").map(str::to_owned)
}

pub fn check_inhibitor() -> CheckResult {
    use super::doctor_rules::inhibitor_status_check;
    if let Ok(client) = TranceClient::connect()
        && let Ok(status) = client.get_status()
    {
        let sources: Vec<String> = if status.inhibited {
            client
                .list_inhibitors()
                .unwrap_or_default()
                .into_iter()
                .map(|(_, who, why)| format!("{who}: {why}"))
                .collect()
        } else {
            Vec::new()
        };
        return inhibitor_status_check(true, status.inhibited, &sources);
    }
    inhibitor_status_check(false, false, &[])
}

pub fn pid_file_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("idle-daemon.pid")
    } else {
        std::env::temp_dir().join("idle-daemon.pid")
    }
}
