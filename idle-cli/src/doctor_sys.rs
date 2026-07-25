// SPDX-License-Identifier: MIT

use super::doctor_checks::{CheckResult, chk};
use std::path::PathBuf;
use std::process::Command;

pub fn check_fonts() -> CheckResult {
    if font_check_via_fc_list() {
        println!(" [✔] System Fonts: Monospace font is installed.");
        chk("System Fonts", true, "monospace font found")
    } else {
        println!(" [✗] System Fonts: Monospace font not found on system!");
        println!("     -> Fix: Please install fonts-dejavu-core or a system monospace font.");
        chk("System Fonts", false, "monospace font missing")
    }
}

/// Probe IdleScreen product/engine packages by real package *name* (NEVRA).
///
/// Do **not** query `cosmic-idle`: on Fedora that is System76's COSMIC DE
/// package, not IdleScreen. IdleScreen's wrapper binary of that name is
/// shipped inside `idle-cli`.
pub fn check_package_install() -> CheckResult {
    // Prefer engine + product packages actually published to the IdleScreen repo.
    const CANDIDATES: &[&str] = &[
        "idle-daemon",
        "idle-cli",
        "idle-cosmic",
        "idle-tui",
        "idle-savers",
        "idlescreen",
        "idle",
        "trance",
    ];

    let mut found: Vec<String> = Vec::new();
    for pkg in CANDIDATES {
        if let Some(ver) = query_rpm(pkg) {
            push_unique(&mut found, ver);
            continue;
        }
        if let Some(ver) = query_dpkg(pkg) {
            push_unique(&mut found, ver);
        }
    }

    // Virtual Provides (e.g. idle-daemon Provides: idlescreen) — name-only
    // rpm -q misses these.
    for capability in ["idlescreen", "idle", "trance"] {
        if let Some(ver) = query_rpm_whatprovides(capability) {
            push_unique(&mut found, ver);
        }
    }

    if found.is_empty() {
        if binary_on_path("idle-daemon") || binary_on_path("idlescreen") || binary_on_path("idle")
        {
            println!(
                " [!] Package: not tracked by RPM/DEB (binaries present; source or manual install)."
            );
            println!("     -> System packages: https://idlescreen.github.io/packages/");
            return chk("Package", true, "unmanaged install (binaries present)");
        }
        println!(" [!] Package: no IdleScreen RPM/DEB packages detected.");
        println!(
            "     -> Install: curl -fsSL https://idlescreen.github.io/packages/install.sh | sh"
        );
        return chk("Package", true, "not a system package");
    }

    let summary = found.join(", ");
    println!(" [✔] Package (system): {summary}");
    println!("     -> Upgrade with: sudo dnf upgrade  OR  sudo apt update && sudo apt upgrade");
    chk("Package", true, summary)
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
    if s.is_empty() || s.contains("is not installed") {
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
    // First line only; may list multiple providers.
    String::from_utf8_lossy(&o.stdout)
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.contains("no package provides"))
        .map(str::to_string)
}

fn query_dpkg(pkg: &str) -> Option<String> {
    let o = Command::new("dpkg-query")
        .args(["-W", "-f=${Package} ${Version}", pkg])
        .output()
        .ok()?;
    if !o.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn binary_on_path(name: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {name} >/dev/null 2>&1")])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
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
