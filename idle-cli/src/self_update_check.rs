// SPDX-License-Identifier: MIT

//! Version-check/report half of `self-update`: query installed + repo
//! candidate versions and print the status block before any upgrade runs.

use anyhow::Result;
use std::path::Path;
use std::process::Command;

use super::self_update_backend::{Backend, PKG_CANDIDATES, installed_version, stdout_trim};

fn dnf_available_version(pkg: &str) -> Option<String> {
    // `-y` auto-accepts repo key imports; without it repo_gpgcheck repos
    // can't load for a non-root user and every query comes back empty.
    // Scoped to the idlescreen repo: refreshes ~4KB of our metadata, not
    // every enabled Fedora repo, and it's the only repo that carries us.
    stdout_trim(
        "dnf",
        &[
            "-y",
            "repoquery",
            "--refresh",
            "--repo=idlescreen",
            "--available",
            "--latest-limit=1",
            "--qf",
            "%{version}-%{release}",
            pkg,
        ],
    )
    .or_else(|| {
        let out = Command::new("dnf")
            .args(["-y", "--repo=idlescreen", "list", "--available", pkg])
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

fn first_installed(backend: Backend) -> Option<(String, String)> {
    for pkg in PKG_CANDIDATES {
        if let Some(ver) = installed_version(backend, pkg) {
            return Some((pkg.to_string(), ver));
        }
    }
    None
}

pub fn handle_dnf_update() -> Result<Option<(String, String)>> {
    println!("Checking for updates with DNF/RPM...");

    let Some((pkg, installed)) = first_installed(Backend::Dnf) else {
        println!(" [!] No IdleScreen RPM packages detected (idle-cli / idle-daemon).");
        println!("     -> curl -fsSL https://idlescreen.github.io/packages/install.sh | sh");
        return Ok(None);
    };
    let available = dnf_available_version(&pkg);

    match available {
        Some(cand) if versions_equalish(&installed, &cand) => {
            println!(" [✔] {pkg} is up to date (version {installed}).");
        }
        Some(cand) if version_cmp(&installed, &cand).is_lt() => {
            println!(" [!] Update available: {pkg} {installed} → {cand}");
        }
        Some(cand) => {
            // installed > candidate: repo metadata is behind the local
            // install (fresh release not yet mirrored, or stale cache).
            println!(" [i] Installed {installed} is newer than repo candidate {cand}.");
        }
        None => {
            println!(" [✔] Installed: {pkg}-{installed}");
            let repo_present = Path::new("/etc/yum.repos.d/idlescreen.repo").exists()
                || Path::new("/etc/yum.repos.d/_copr:idlescreen.repo").exists();
            if repo_present {
                println!(" [i] Could not query repo for candidate version.");
                println!("     -> Refresh metadata: sudo dnf clean all && sudo dnf upgrade");
            } else {
                println!(" [!] Could not query the latest package from the repo.");
                println!("     -> Confirm the idlescreen repo is in /etc/yum.repos.d/");
            }
        }
    }
    Ok(Some((pkg, installed)))
}

fn apt_policy_versions(pkg: &str) -> Option<(String, String)> {
    let out = stdout_trim("apt-cache", &["policy", pkg])?;
    parse_apt_policy(&out)
}

fn parse_apt_policy(out: &str) -> Option<(String, String)> {
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
        // Keep a "(none)" Installed value — the caller renders it as
        // "package not installed" rather than "status unknown".
        (Some(i), Some(c)) if c != "(none)" => Some((i, c)),
        (None, Some(c)) if c != "(none)" => Some(("(none)".to_string(), c)),
        _ => None,
    }
}

pub fn handle_apt_update() -> Result<Option<(String, String)>> {
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
                return Ok(None);
            } else if version_cmp(&inst, &cand).is_lt() {
                println!(" [!] Update available: {inst} → {cand}");
            } else if version_cmp(&inst, &cand).is_gt() {
                println!(" [i] Installed {inst} is newer than repo candidate {cand}.");
            } else {
                println!(" [✔] IdleScreen is up to date.");
            }
            Ok(Some((pkg.to_string(), inst)))
        }
        None => {
            if let Some(inst) = stdout_trim("dpkg-query", &["-W", "-f=${Version}", pkg]) {
                println!(" [✔] Installed version: {inst}");
                println!(" [!] Could not read APT candidate (is the idlescreen repo configured?).");
                println!("     -> sudo apt update && sudo apt upgrade");
                Ok(Some((pkg.to_string(), inst)))
            } else {
                println!(" [!] Could not determine package status.");
                println!(
                    "     -> curl -fsSL https://idlescreen.github.io/packages/install.sh | sh"
                );
                Ok(None)
            }
        }
    }
}

/// Strip RPM epoch prefix and arch suffix — DNF reports `0:3.5.1-1`
/// while `rpm -q` answers `3.5.1-1`.
fn norm_version(s: &str) -> String {
    s.trim()
        .rsplit(':')
        .next()
        .unwrap_or("")
        .trim_end_matches(".x86_64")
        .trim_end_matches(".noarch")
        .to_string()
}

pub fn versions_equalish(a: &str, b: &str) -> bool {
    norm_version(a) == norm_version(b)
}

/// Ordering on `X.Y.Z-R`-style versions: split on non-alphanumerics,
/// compare numeric segments numerically, missing segments as zero.
/// Enough for IdleScreen's `N.N.N-N` package versions — not full rpmvercmp.
pub fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    // split into alphanumeric runs, e.g. "3.5.1-2" -> [3,5,1,2]
    let seg = |s: &str| -> Vec<String> {
        norm_version(s)
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect()
    };
    let (pa, pb) = (seg(a), seg(b));
    for i in 0..pa.len().max(pb.len()) {
        let (x, y) = (
            pa.get(i).map_or("0", String::as_str),
            pb.get(i).map_or("0", String::as_str),
        );
        let ord = match (x.parse::<u64>(), y.parse::<u64>()) {
            (Ok(nx), Ok(ny)) => nx.cmp(&ny),
            _ => x.cmp(y),
        };
        if ord != Ordering::Equal {
            return ord;
        }
    }
    Ordering::Equal
}

#[cfg(test)]
#[path = "self_update_check_tests.rs"]
mod tests;
