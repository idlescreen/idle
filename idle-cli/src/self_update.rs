// SPDX-License-Identifier: MIT

//! Check whether a newer *system package* is available.

use anyhow::Result;
use std::path::Path;
use std::process::Command;

use super::self_update_backend::{
    Backend, PKG_CANDIDATES, detect_backend, installed_packages, run_privileged, stdout_trim,
};

fn rpm_installed_version(pkg: &str) -> Option<String> {
    stdout_trim("rpm", &["-q", pkg, "--qf", "%{VERSION}-%{RELEASE}"])
}

fn dnf_available_version(pkg: &str) -> Option<String> {
    stdout_trim(
        "dnf",
        &[
            "repoquery",
            "--available",
            "--latest-limit=1",
            "--qf",
            "%{version}-%{release}",
            pkg,
        ],
    )
    .or_else(|| {
        let out = Command::new("dnf")
            .args(["list", "--available", pkg])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        parse_dnf_list_version(&text, true)
    })
}

fn parse_dnf_list_version(text: &str, want_available: bool) -> Option<String> {
    let mut section = "";
    let mut last = None;
    for line in text.lines() {
        let line = line.trim();
        if line.contains("Installed Packages") || line == "Installed packages" {
            section = "installed";
            continue;
        }
        if line.contains("Available Packages") || line == "Available packages" {
            section = "available";
            continue;
        }
        let looks_like_pkg = PKG_CANDIDATES
            .iter()
            .any(|p| line.starts_with(&format!("{p}.")) || line.starts_with(&format!("{p} ")));
        if !looks_like_pkg {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let ver = parts[1].to_string();
        if (want_available && (section == "available" || section.is_empty()))
            || (!want_available && section == "installed")
        {
            last = Some(ver);
        }
    }
    last
}

fn first_installed_rpm() -> Option<(String, String)> {
    for pkg in PKG_CANDIDATES {
        if let Some(ver) = rpm_installed_version(pkg) {
            return Some((pkg.to_string(), ver));
        }
    }
    None
}

fn handle_dnf_update() -> Result<()> {
    println!("Checking for updates with DNF/RPM...");

    let Some((pkg, installed)) = first_installed_rpm() else {
        println!(" [!] No IdleScreen RPM packages detected (idle-cli / idle-daemon).");
        println!("     -> curl -fsSL https://idlescreen.github.io/packages/install.sh | sh");
        return Ok(());
    };
    let available = dnf_available_version(&pkg);

    match available {
        Some(cand) if versions_equalish(&installed, &cand) => {
            println!(" [✔] {pkg} is up to date (version {installed}).");
            println!("     -> Upgrade anytime with: sudo dnf upgrade");
        }
        Some(cand) => {
            println!(" [!] A new version is available: {pkg} {installed} → {cand}");
            println!("     -> Run: sudo dnf upgrade {pkg}");
        }
        None => {
            println!(" [✔] Installed: {pkg}-{installed}");
            let repo_present = Path::new("/etc/yum.repos.d/idlescreen.repo").exists()
                || Path::new("/etc/yum.repos.d/_copr:idlescreen.repo").exists();
            if repo_present {
                // Repo file exists but query failed — almost always a stale
                // metadata cache, not a real problem.
                println!(" [i] Repo configured; version check skipped (stale dnf metadata).");
                println!("     -> Refresh: sudo dnf clean all && sudo dnf upgrade");
            } else {
                println!(" [!] Could not query the latest package from the repo.");
                println!("     -> Confirm the idlescreen repo is in /etc/yum.repos.d/");
            }
        }
    }
    Ok(())
}

fn apt_policy_versions(pkg: &str) -> Option<(String, String)> {
    let out = stdout_trim("apt-cache", &["policy", pkg])?;
    let mut inst = None;
    let mut cand = None;
    for line in out.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Installed:") {
            inst = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("Candidate:") {
            cand = Some(rest.trim().to_string());
        }
    }
    match (inst, cand) {
        (Some(i), Some(c)) if i != "(none)" && c != "(none)" => Some((i, c)),
        (None, Some(c)) if c != "(none)" => Some(("(none)".to_string(), c)),
        _ => None,
    }
}

fn handle_apt_update() -> Result<()> {
    let pkg = PKG_CANDIDATES
        .iter()
        .find(|p| {
            stdout_trim("dpkg-query", &["-W", "-f=${Version}", p]).is_some()
                || apt_policy_versions(p).is_some()
        })
        .copied()
        .unwrap_or("idle-cli");

    println!(" Checking APT package status for '{pkg}'...");
    match apt_policy_versions(pkg) {
        Some((inst, cand)) => {
            println!(" [✔] Installed version: {inst}");
            println!(" [✔] Repository version: {cand}");

            if inst == "(none)" {
                println!(" [!] Package is not currently installed.");
                println!(
                    "     -> curl -fsSL https://idlescreen.github.io/packages/install.sh | sh"
                );
            } else if !versions_equalish(&inst, &cand) {
                println!(" [!] Upgrade available: {inst} -> {cand}");
                println!("     -> Run: sudo apt update && sudo apt install --only-upgrade {pkg}");
            } else {
                println!(" [✔] IdleScreen is up to date.");
                println!("     -> Upgrade anytime with: sudo apt update && sudo apt upgrade");
            }
        }
        None => {
            if let Some(inst) = stdout_trim("dpkg-query", &["-W", "-f=${Version}", pkg]) {
                println!(" [✔] Installed version: {inst}");
                println!(" [!] Could not read APT candidate (is the idlescreen repo configured?).");
                println!("     -> sudo apt update && sudo apt upgrade");
            } else {
                println!(" [!] Could not determine package status.");
                println!(
                    "     -> curl -fsSL https://idlescreen.github.io/packages/install.sh | sh"
                );
            }
        }
    }
    Ok(())
}

fn versions_equalish(a: &str, b: &str) -> bool {
    let norm = |s: &str| {
        s.trim()
            .trim_end_matches(".x86_64")
            .trim_end_matches(".noarch")
            .to_string()
    };
    norm(a) == norm(b)
}

/// `update`/`upgrade`/`self-update` all do the same thing: upgrade every
/// installed IdleScreen package (`idle-*` / `idlescreen*`) via the system
/// package manager. Status is printed first so the user sees what changed.
/// `check_only` stops after the status report — nothing is installed.
#[tracing::instrument]
pub fn handle_self_update(check_only: bool) -> Result<()> {
    let Some(backend) = detect_backend() else {
        println!(" [!] No supported package manager detected (need DNF/RPM or APT).");
        println!("     -> Fedora: sudo dnf update");
        println!("     -> Debian/Ubuntu: sudo apt update && sudo apt upgrade");
        return Ok(());
    };

    match backend {
        Backend::Dnf => handle_dnf_update()?,
        Backend::Apt => handle_apt_update()?,
    }
    if check_only {
        return Ok(());
    }

    let pkgs = installed_packages(backend);
    if pkgs.is_empty() {
        return Ok(()); // status handler already printed the installer hint
    }
    println!(" Upgrading {} package(s): {}", pkgs.len(), pkgs.join(" "));

    // Refresh metadata, then upgrade only the IdleScreen set — never the rest
    // of the system. Streams the package manager's own progress/output.
    let steps: Vec<Vec<&str>> = match backend {
        Backend::Dnf => {
            let mut v = vec!["dnf", "upgrade", "-y"];
            v.extend(pkgs.iter().map(String::as_str));
            vec![v]
        }
        Backend::Apt => vec![vec!["apt-get", "update", "-y"], {
            let mut v = vec!["apt-get", "install", "--only-upgrade", "-y"];
            v.extend(pkgs.iter().map(String::as_str));
            v
        }],
    };
    for step in &steps {
        println!(" $ {}", step.join(" "));
        match run_privileged(step) {
            Ok(s) if s.success() => {}
            Ok(s) => {
                println!(" [!] {} exited with {}", step[0], s);
                println!("     -> retry manually: sudo {}", step.join(" "));
                anyhow::bail!("package upgrade failed: {} exited {}", step[0], s);
            }
            Err(e) => {
                println!(" [!] Upgrade needs root: {e}");
                println!("     -> sudo {}", step.join(" "));
                anyhow::bail!("package upgrade needs root: {e}");
            }
        }
    }
    println!(" [✔] IdleScreen packages upgraded.");
    Ok(())
}

#[cfg(test)]
#[path = "self_update_tests.rs"]
mod tests;
