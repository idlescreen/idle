// SPDX-License-Identifier: MIT

//! D-Bus, systemd, and process checks for doctor.

use super::doctor_checks::{CheckResult, chk};
use idle_dbus::TranceClient;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn check_dbus() -> CheckResult {
    if let Ok(client) = TranceClient::connect() {
        match client.get_status() {
            Ok(status) => {
                if !status.idle_enabled {
                    return chk(
                        "D-Bus Service",
                        false,
                        format!(
                            "connected but idle is DISABLED — savers will not start (timeout={}m saver='{}'); enable with: idlescreen enable",
                            status.idle_timeout_mins, status.active_saver
                        ),
                    );
                }
                chk(
                    "D-Bus Service",
                    true,
                    format!(
                        "connected (io.github.idlescreen.Idle) idle_enabled=true timeout={}m saver='{}'",
                        status.idle_timeout_mins, status.active_saver
                    ),
                )
            }
            Err(e) => chk("D-Bus Service", false, format!("GetStatus error: {e}")),
        }
    } else {
        chk(
            "D-Bus Service",
            false,
            "cannot connect — idle-daemon is not running; start it: systemctl --user start idle-daemon  (or: idlescreen doctor --fix)",
        )
    }
}

/// Savers on disk or via `idle-savers` / modular `idle-saver-*` packages.
pub fn check_savers() -> CheckResult {
    const DIRS: &[&str] = &[
        "/usr/libexec/idle/screensavers",
        "/usr/local/libexec/idle/screensavers",
    ];
    let mut found_so = 0usize;
    let mut found_dir: Option<&str> = None;
    for dir in DIRS {
        let path = Path::new(dir);
        if !path.is_dir() {
            continue;
        }
        found_dir = Some(dir);
        if let Ok(rd) = fs::read_dir(path) {
            found_so += rd
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|x| x.to_str())
                        .is_some_and(|ext| ext == "so")
                })
                .count();
        }
    }

    if found_so > 0 {
        return chk(
            "Savers",
            true,
            format!(
                "{found_so} plugin(s) under {}",
                found_dir.unwrap_or("libexec")
            ),
        );
    }

    // Package presence is enough when plugins are not yet expanded on disk.
    if package_installed("idle-savers") || package_installed("idle-saver-beams") {
        return chk("Savers", true, "idle-savers / idle-saver-* package present");
    }

    // User-local installs
    if let Ok(home) = std::env::var("HOME") {
        let local = PathBuf::from(home).join(".local/share/idle/screensavers");
        if local.is_dir()
            && fs::read_dir(&local)
                .map(|rd| {
                    rd.filter_map(|e| e.ok()).any(|e| {
                        e.path()
                            .extension()
                            .and_then(|x| x.to_str())
                            .is_some_and(|ext| ext == "so")
                    })
                })
                .unwrap_or(false)
        {
            return chk(
                "Savers",
                true,
                "plugins under ~/.local/share/idle/screensavers",
            );
        }
    }

    chk(
        "Savers",
        false,
        "no screensaver plugins found — install idle-savers or idle-saver-* packages",
    )
}

/// Optional TUI binary (product front-end, not required for daemon).
pub fn check_tui_optional() -> CheckResult {
    if which_exists("idle-tui") || which_exists("idlescreen-tui") {
        chk("TUI", true, "idle-tui available on PATH")
    } else {
        chk(
            "TUI",
            true,
            "optional — install idle-tui for terminal control UI",
        )
    }
}

fn which_exists(name: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {name}")])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn package_installed(pkg: &str) -> bool {
    if Command::new("rpm")
        .args(["-q", pkg])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return true;
    }
    Command::new("dpkg-query")
        .args(["-W", pkg])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn check_systemd_service() -> CheckResult {
    let output = Command::new("systemctl")
        .args(["--user", "is-active", "idle-daemon.service"])
        .output();

    match output {
        Ok(out) => {
            let status = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if status == "active" {
                chk("Systemd Service", true, "active")
            } else {
                chk(
                    "Systemd Service",
                    false,
                    format!("status '{status}'; run systemctl --user start idle-daemon"),
                )
            }
        }
        Err(e) => chk("Systemd Service", false, format!("systemctl error: {e}")),
    }
}

pub fn check_running_pid() -> CheckResult {
    let pid_path = pid_file_path();
    let dbus_ok = TranceClient::connect().is_ok();

    if pid_path.exists() {
        if let Ok(content) = fs::read_to_string(&pid_path) {
            let pid_str = content.trim();
            if let Ok(pid) = pid_str.parse::<i32>() {
                // SAFETY: kill(pid, 0) only checks process existence; no signal delivered.
                if unsafe { libc::kill(pid, 0) } == 0 {
                    return chk("Process Status", true, format!("PID {pid} running"));
                }
                return chk("Process Status", false, format!("stale PID {pid}"));
            }
        }
        chk("Process Status", true, "pid file unreadable")
    } else if dbus_ok {
        chk("Process Status", true, "missing pid but d-bus ok")
    } else {
        chk("Process Status", false, "missing pid file")
    }
}

pub fn check_inhibitor() -> CheckResult {
    if let Ok(client) = TranceClient::connect()
        && let Ok(status) = client.get_status()
    {
        if status.inhibited {
            // Not "ok" — inhibited means savers will not appear.
            return chk(
                "Inhibitor Status",
                false,
                "INHIBITED — an app/system is blocking idle; savers will not start (try: idlescreen inhibitors)",
            );
        }
        return chk(
            "Inhibitor Status",
            true,
            "uninhibited (idle can trigger savers)",
        );
    }
    // Cannot claim nominal idle path without a live daemon.
    chk(
        "Inhibitor Status",
        false,
        "cannot check — idle-daemon not connected (start the daemon first)",
    )
}

fn pid_file_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("idle-daemon.pid")
    } else {
        std::env::temp_dir().join("idle-daemon.pid")
    }
}
