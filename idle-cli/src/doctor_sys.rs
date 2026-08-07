// SPDX-License-Identifier: MIT

use super::doctor_checks::{CheckResult, chk};
use crate::pkg_query::{query_dpkg, query_dpkg_file, query_rpm_file};
use std::path::PathBuf;
use std::process::Command;

pub fn check_fonts() -> CheckResult {
    if font_check_via_fc_list() {
        chk("System Fonts", true, "monospace font found")
    } else {
        chk(
            "System Fonts",
            false,
            "monospace font missing — install fonts-dejavu-core or another mono font",
        )
    }
}

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
            return chk(
                "Package",
                true,
                "unmanaged install (binaries present; not owned by RPM/DEB) — see https://idlescreen.github.io/packages/",
            );
        }
        return chk(
            "Package",
            false,
            "idle-daemon / idle-cli not installed — curl -fsSL https://idlescreen.github.io/packages/install.sh | sh",
        );
    }

    let summary = found.join(", ");
    chk("Package", true, summary)
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

fn font_check_via_fc_list() -> bool {
    let output = Command::new("fc-list").args([":mono"]).output();
    match output {
        Ok(out) => out.status.success() && !out.stdout.is_empty(),
        Err(_) => {
            let common_dirs = ["/usr/share/fonts", "/usr/local/share/fonts"];
            common_dirs.iter().any(|dir| PathBuf::from(dir).exists())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::package_rank;

    #[test]
    fn package_rank_prefers_core() {
        assert!(
            package_rank("idle-daemon-2.3.1-1.x86_64") < package_rank("idle-cli-2.3.1-1.x86_64")
        );
        assert!(package_rank("idle-cli-2.3.1-1.x86_64") < package_rank("idle-tui-2.2.0-1.x86_64"));
        assert!(
            package_rank("idle-savers-2.3.1-1.x86_64") < package_rank("idle-cosmic-2.1.2-1.x86_64")
        );
    }
}
