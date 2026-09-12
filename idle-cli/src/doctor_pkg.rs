// SPDX-License-Identifier: MIT

//! Installed-package checks for doctor (rpm/dpkg probing).

use super::doctor_checks::{CheckResult, fail, ok, warn};
use crate::pkg_query::{query_dpkg, query_dpkg_file, query_rpm_file};
use std::path::PathBuf;
use std::process::Command;

/// Probe IdleScreen product packages by NEVRA and by ownership of known binaries.
///
/// Do **not** query `cosmic-idle` (System76 COSMIC) or treat `/usr/bin/idle` as
/// IdleScreen — that path is Fedora's `python3-idle` (Python IDE).
pub fn check_package_install() -> CheckResult {
    // Published IdleScreen package names (order = report preference).
    const CANDIDATES: &[&str] = &[
        "idle-daemon",
        "idle-cli",
        "idle-savers",
        "idle-tui",
        "idle-cosmic",
        "idle-studio",
        // Meta product package from packages repo.
        "idlescreen",
    ];

    // Binaries that imply an IdleScreen install when owned by a package.
    // Never include bare `idle` — that is python3-idle on Fedora.
    const BINARIES: &[&str] = &["idle-daemon", "idlescreen", "idle-tui", "idlescreen-applet"];

    let mut found: Vec<String> = Vec::new();

    // 1) Package that owns the binaries on PATH (most reliable after renames).
    for bin in BINARIES {
        if let Some(path) = which_path(bin) {
            if let Some(ver) = query_rpm_file(&path) {
                push_unique(&mut found, ver);
            } else if let Some(ver) = query_dpkg_file(&path) {
                push_unique(&mut found, ver);
            }
        }
    }

    // 2) Direct package-name queries (modular idle-* set).
    for pkg in CANDIDATES {
        if let Some(ver) = query_rpm(pkg) {
            push_unique(&mut found, ver);
            continue;
        }
        if let Some(ver) = query_dpkg(pkg) {
            push_unique(&mut found, ver);
        }
    }

    // 3) Meta / modular product package capabilities.
    for capability in ["idlescreen", "idle-savers"] {
        if let Some(ver) = query_rpm_whatprovides(capability) {
            push_unique(&mut found, ver);
        }
    }

    // Prefer engine/product packages first in the summary.
    found.sort_by(|a, b| package_rank(a).cmp(&package_rank(b)).then(a.cmp(b)));

    if found.is_empty() {
        let has_bin = BINARIES.iter().any(|b| binary_on_path(b));
        if has_bin {
            return ok(
                "Package",
                "unmanaged install (binaries present; not owned by RPM/DEB) — see https://idlescreen.github.io/packages/",
            );
        }
        return fail(
            "Package",
            "idle-daemon / idle-cli not installed — curl -fsSL https://idlescreen.github.io/packages/install.sh | sh",
        );
    }

    // Render role labels, not raw NEVRAs: package names (idle-cli vs the
    // idlescreen metapackage) confuse users; roles say what each thing is.
    let summary = found
        .iter()
        .map(|e| display_entry(e))
        .collect::<Vec<_>>()
        .join(", ");
    // Version skew that actually matters: a CLI *newer* than the daemon
    // can call D-Bus methods the daemon lacks; different majors can drift
    // either way. Same major.minor, different patch is the normal state —
    // crates version independently inside a release train.
    let daemon_mm = found
        .iter()
        .find(|f| f.starts_with("idle-daemon"))
        .and_then(|e| extract_version(e))
        .and_then(major_minor);
    let cli_mm = found
        .iter()
        .find(|f| f.starts_with("idle-cli"))
        .and_then(|e| extract_version(e))
        .and_then(major_minor);
    if let (Some(d), Some(c)) = (daemon_mm, cli_mm)
        && (d.0 != c.0 || c > d)
    {
        return warn(
            "Package",
            format!(
                "{summary} — version skew: daemon {}.{} vs cli {}.{}",
                d.0, d.1, c.0, c.1
            ),
        )
        .with_fix("idlescreen self-update to align component versions");
    }
    ok("Package", summary)
}

/// `idle-daemon-3.5.2-1.x86_64` → `daemon 3.5.2-1`. Labels map package
/// names to user-facing roles — `idle-cli` is the `idlescreen` command,
/// `idlescreen` is the metapackage — so the report reads as components.
fn display_entry(nevra: &str) -> String {
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
fn major_minor(version: &str) -> Option<(u64, u64)> {
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

fn package_rank(nevra_or_line: &str) -> u8 {
    // NEVRA / dpkg lines start with the package name.
    if nevra_or_line.starts_with("idle-daemon") {
        0
    } else if nevra_or_line.starts_with("idle-cli") {
        1
    } else if nevra_or_line.starts_with("idle-savers") {
        2
    } else if nevra_or_line.starts_with("idle-tui") {
        3
    } else if nevra_or_line.starts_with("idle-cosmic") {
        4
    } else if nevra_or_line.starts_with("idle-studio") {
        5
    } else {
        9
    }
}

fn push_unique(found: &mut Vec<String>, ver: String) {
    if !found.iter().any(|v| v == &ver) {
        found.push(ver);
    }
}

fn query_rpm(pkg: &str) -> Option<String> {
    let o = Command::new("rpm")
        .args(["-q", pkg, "--qf", "%{NAME}-%{VERSION}-%{RELEASE}.%{ARCH}"])
        .output()
        .ok()?;
    if !o.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
    if s.is_empty() || s.contains("is not installed") || s.contains("not installed") {
        None
    } else {
        Some(s)
    }
}

fn query_rpm_whatprovides(capability: &str) -> Option<String> {
    let o = Command::new("rpm")
        .args([
            "-q",
            "--whatprovides",
            capability,
            "--qf",
            "%{NAME}-%{VERSION}-%{RELEASE}.%{ARCH}\\n",
        ])
        .output()
        .ok()?;
    if !o.status.success() {
        return None;
    }
    String::from_utf8_lossy(&o.stdout)
        .lines()
        .map(str::trim)
        .find(|l| {
            !l.is_empty()
                && !l.contains("no package provides")
                && !l.contains("is not installed")
                && !l.contains("not installed")
        })
        .map(str::to_string)
}

/// Owning RPM for a filesystem path (`rpm -qf`).
fn which_path(name: &str) -> Option<PathBuf> {
    let o = Command::new("sh")
        .args(["-c", &format!("command -v {name}")])
        .output()
        .ok()?;
    if !o.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(PathBuf::from(s))
    }
}

fn binary_on_path(name: &str) -> bool {
    which_path(name).is_some()
}

#[cfg(test)]
#[path = "doctor_pkg_tests.rs"]
mod tests;
