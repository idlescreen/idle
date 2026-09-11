// SPDX-License-Identifier: MIT

//! Environment and protocol soft-checks for doctor.

use super::doctor_checks::{CheckResult, fail, ok, warn};

pub fn check_wayland() -> CheckResult {
    match std::env::var("WAYLAND_DISPLAY") {
        Ok(val) if !val.is_empty() => ok("Environment", format!("WAYLAND_DISPLAY={val}")),
        _ => fail(
            "Environment",
            "WAYLAND_DISPLAY missing; IdleScreen needs a Wayland session",
        )
        .with_fix("run inside a Wayland session (not ssh/VT)"),
    }
}

/// Soft protocol/DE hints — an unrecognized DE warns rather than fails.
pub fn check_protocol_hints() -> CheckResult {
    let de = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    if std::env::var("WAYLAND_DISPLAY").is_err() {
        return fail(
            "Protocols",
            "WAYLAND_DISPLAY unset; need ext-idle-notify-v1 and zwlr_layer_shell_v1",
        )
        .with_fix("run inside a Wayland session");
    }
    let known = [
        "cosmic", "hyprland", "sway", "niri", "river", "wayfire", "kde", "plasma",
    ];
    let lower = de.to_ascii_lowercase();
    let friendly = known.iter().any(|k| lower.contains(k));
    let de_label = if de.is_empty() {
        "unknown"
    } else {
        de.as_str()
    };
    if friendly || de.is_empty() {
        ok(
            "Protocols",
            format!("WAYLAND_DISPLAY set; DE='{de_label}' (need idle-notify + layer-shell)"),
        )
    } else {
        warn(
            "Protocols",
            format!("DE='{de_label}' unrecognized — may lack layer-shell/idle-notify"),
        )
        .with_fix("see docs/BOUNDARIES.md for the compositor support matrix")
    }
}
