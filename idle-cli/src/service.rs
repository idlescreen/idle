// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Daemon service lifecycle: `restart` and `logs` via systemd user units.
//! Both work without a live daemon — `restart` is how you fix a dead one.

use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use idle_dbus::daemon_available;

use crate::self_update_backend::which;

/// Current unit plus the legacy name so old installs still show logs.
const UNITS: &[&str] = &["idle-daemon", "trance-daemon"];

pub fn handle_restart() -> Result<()> {
    let status = Command::new("systemctl")
        .args(["--user", "restart", "idle-daemon"])
        .status()
        .context("failed to run systemctl --user restart idle-daemon")?;
    if !status.success() {
        bail!("systemctl --user restart idle-daemon exited {status}");
    }
    // The unit restart returns before the D-Bus name is claimed; wait for it.
    for _ in 0..30 {
        if daemon_available() {
            if !crate::quiet() {
                println!("Daemon restarted.");
            }
            return Ok(());
        }
        sleep(Duration::from_millis(100));
    }
    println!("Restart issued; idle-daemon is not reachable on D-Bus yet.");
    Ok(())
}

pub fn handle_logs(follow: bool, lines: u32) -> Result<()> {
    if !which("journalctl") {
        bail!("journalctl not found; read the daemon log under ~/.cache/idle/ instead");
    }
    let mut args = vec![
        "--user".to_string(),
        "--output".to_string(),
        "short".to_string(),
    ];
    for unit in UNITS {
        args.push("-u".to_string());
        args.push(format!("{unit}.service"));
    }
    args.push("-n".to_string());
    args.push(lines.to_string());
    if follow {
        args.push("-f".to_string());
    }
    let status = Command::new("journalctl")
        .args(&args)
        .status()
        .context("failed to run journalctl")?;
    if status.success() {
        Ok(())
    } else {
        std::process::exit(status.code().unwrap_or(1));
    }
}
