// SPDX-License-Identifier: MIT

//! Display/format helpers for the doctor package check: role labels for
//! the summary line and the version-skew verdict.

use super::doctor_checks::{CheckResult, warn};

/// Warn only on skew that matters: a CLI *newer* than the daemon can call
/// D-Bus methods the daemon lacks; different majors can drift either way.
/// Same major.minor, different patch is the normal state — crates version
/// independently inside a release train.
pub fn skew_result(found: &[String], summary: &str) -> Option<CheckResult> {
    let mm_of = |prefix: &str| {
        found
            .iter()
            .find(|f| f.starts_with(prefix))
            .and_then(|e| extract_version(e))
            .and_then(major_minor)
    };
    let (Some(d), Some(c)) = (mm_of("idle-daemon"), mm_of("idle-cli")) else {
        return None;
    };
    if d.0 == c.0 && c <= d {
        return None;
    }
    Some(
        warn(
            "Package",
            format!(
                "{summary} — version skew: daemon {}.{} vs cli {}.{}",
                d.0, d.1, c.0, c.1
            ),
        )
        .with_fix("idlescreen self-update to align component versions"),
    )
}

/// `idle-daemon-3.5.2-1.x86_64` → `daemon 3.5.2-1`. Labels map package
/// names to user-facing roles — `idle-cli` is the `idlescreen` command,
/// `idlescreen` is the metapackage — so the report reads as components.
pub fn display_entry(nevra: &str) -> String {
    let split = nevra
        .match_indices('-')
        .find(|(i, _)| {
            nevra[i + 1..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
        })
        .map(|(i, _)| i);
    let Some(idx) = split else {
        return nevra.to_string();
    };
    let (name, ver) = nevra.split_at(idx);
    let ver = ver[1..]
        .trim_end_matches(".x86_64")
        .trim_end_matches(".noarch")
        .trim_end_matches(".aarch64");
    let label = match name {
        "idle-daemon" => "daemon",
        "idle-cli" => "cli (the 'idlescreen' command)",
        "idle-savers" => "savers",
        "idle-tui" => "tui",
        "idle-cosmic" => "cosmic-applet",
        "idle-studio" => "studio",
        "idlescreen" => "metapackage",
        n if n.starts_with("idle-saver-") => &n["idle-".len()..],
        n => n,
    };
    format!("{label} {ver}")
}

/// `3.5.2-1` → `(3, 5)` — major.minor pair for skew checks.
pub fn major_minor(version: &str) -> Option<(u64, u64)> {
    let mut it = version
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty());
    Some((
        it.next()?.parse().ok()?,
        it.next().unwrap_or("0").parse().ok()?,
    ))
}

/// `idle-daemon-3.4.0-1.x86_64` → `3.4.0` — the version is the first
/// dash-segment starting with a digit (package names contain dashes).
fn extract_version(entry: &str) -> Option<&str> {
    entry
        .split('-')
        .find(|s| s.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .map(|v| v.trim_end_matches(".x86_64").trim_end_matches(".noarch"))
}
