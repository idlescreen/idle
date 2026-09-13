// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Status display and version reporting for the idle CLI.

use anyhow::{Context, Result};
use idle_dbus::{TranceClient, daemon_available};

fn display_saver(name: &str) -> String {
    if name.is_empty() {
        "random".into()
    } else {
        name.to_string()
    }
}

/// Pure text status report (unit-tested; no D-Bus).
pub fn format_status_text(status: &idle_dbus::DaemonStatus) -> String {
    let scale = if status.render_scale.is_empty() {
        "default"
    } else {
        status.render_scale.as_str()
    };
    format!(
        "running:              {}\n\
         idle_enabled:         {}\n\
         idle_timeout_mins:    {}\n\
         active_saver:         {}\n\
         show_fps_overlay:     {}\n\
         render_scale:         {}\n\
         presentation_active:  {}\n\
         preview_active:       {}\n\
         current_saver:        {}\n\
         system_idle:          {}\n\
         session_locked:       {}\n\
         inhibited:            {}\n",
        status.running,
        status.idle_enabled,
        status.idle_timeout_mins,
        display_saver(&status.active_saver),
        status.show_fps_overlay,
        scale,
        status.presentation_active,
        status.preview_active,
        status.current_saver,
        status.system_idle,
        status.session_locked,
        status.inhibited,
    )
}

/// Pure JSON status line (unit-tested; no D-Bus).
pub fn format_status_json(status: &idle_dbus::DaemonStatus) -> String {
    format!(
        "{{\"running\":{},\"idle_enabled\":{},\"idle_timeout_mins\":{},\"active_saver\":\"{}\",\"show_fps_overlay\":{},\"render_scale\":\"{}\",\"presentation_active\":{},\"preview_active\":{},\"current_saver\":\"{}\",\"system_idle\":{},\"session_locked\":{},\"inhibited\":{}}}",
        status.running,
        status.idle_enabled,
        status.idle_timeout_mins,
        status.active_saver,
        status.show_fps_overlay,
        status.render_scale,
        status.presentation_active,
        status.preview_active,
        status.current_saver,
        status.system_idle,
        status.session_locked,
        status.inhibited
    )
}

fn print_status_json(status: &idle_dbus::DaemonStatus) {
    print!("{}", format_status_json(status));
    println!();
}

fn print_status_text(status: &idle_dbus::DaemonStatus) {
    print!("{}", format_status_text(status));
}

pub fn cmd_status(client: &TranceClient, json: bool) -> Result<()> {
    let status = client.get_status().context("querying daemon status")?;
    if json {
        print_status_json(&status);
    } else {
        print_status_text(&status);
    }
    Ok(())
}

const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_daemon_version_line() {
    if !daemon_available() {
        println!("Daemon:  not running");
        return;
    }
    if let Ok(client) = TranceClient::connect()
        && let Ok(status) = client.get_status()
    {
        println!(
            "Daemon:  reachable ({})",
            if status.running {
                "running"
            } else {
                "connected"
            }
        );
        return;
    }
    println!("Daemon:  reachable");
}

pub fn print_version(verbose: bool, json: bool) {
    if json {
        println!("{{\"name\":\"idlescreen\",\"version\":\"{CLI_VERSION}\"}}");
        return;
    }
    println!("idlescreen {CLI_VERSION}");
    if !verbose {
        return;
    }
    println!("IdleScreen screensaver control CLI");
    println!("License: Apache-2.0");
    println!("Home:    https://github.com/idlescreen/idlescreen");
    if let Some(pkg) = package_version_hint() {
        println!("Package: {pkg}");
    }
    print_daemon_version_line();
}

fn package_hint_from_command(
    program: &str,
    args: &[&str],
    reject_substr: Option<&str>,
) -> Option<String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        return None;
    }
    if let Some(bad) = reject_substr
        && s.contains(bad)
    {
        return None;
    }
    Some(s)
}

fn package_version_hint() -> Option<String> {
    for pkg in ["idle-cli", "idle-daemon", "idlescreen"] {
        if let Some(s) = package_hint_from_command(
            "rpm",
            &["-q", pkg, "--qf", "%{NAME}-%{VERSION}-%{RELEASE}.%{ARCH}"],
            Some("is not installed"),
        ) {
            return Some(s);
        }
        if let Some(s) =
            package_hint_from_command("dpkg-query", &["-W", "-f=${Package} ${Version}", pkg], None)
        {
            return Some(s);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use idle_dbus::DaemonStatus;

    fn sample() -> DaemonStatus {
        DaemonStatus {
            running: true,
            idle_enabled: true,
            idle_timeout_mins: 5,
            active_saver: String::new(),
            presentation_active: true,
            preview_active: true,
            system_idle: false,
            session_locked: false,
            inhibited: false,
            current_saver: "beams".into(),
            show_fps_overlay: false,
            render_scale: String::new(),
        }
    }

    #[test]
    fn status_text_shows_preview_and_random_saver() {
        let s = format_status_text(&sample());
        assert!(s.contains("preview_active:       true"));
        assert!(s.contains("presentation_active:  true"));
        assert!(s.contains("active_saver:         random"));
        assert!(s.contains("current_saver:        beams"));
        assert!(s.contains("render_scale:         default"));
        assert!(s.contains("inhibited:            false"));
    }

    #[test]
    fn status_json_includes_critical_flags() {
        let j = format_status_json(&sample());
        assert!(j.contains("\"preview_active\":true"));
        assert!(j.contains("\"presentation_active\":true"));
        assert!(j.contains("\"inhibited\":false"));
        assert!(j.contains("\"current_saver\":\"beams\""));
    }

    #[test]
    fn status_text_shows_inhibited_when_blocked() {
        let mut st = sample();
        st.inhibited = true;
        st.preview_active = false;
        let s = format_status_text(&st);
        assert!(s.contains("inhibited:            true"));
        assert!(s.contains("preview_active:       false"));
    }
}
