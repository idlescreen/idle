// SPDX-License-Identifier: MIT

//! Check whether a newer *system package* is available, then upgrade the
//! installed IdleScreen package set via the system package manager.

use anyhow::Result;

use super::self_update_backend::{
    Backend, detect_backend, installed_packages, installed_version, run_privileged,
    upgradable_packages,
};
use super::self_update_check::{
    handle_apt_update, handle_dnf_update, version_cmp, versions_equalish,
};

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

    let tracked = match backend {
        Backend::Dnf => handle_dnf_update()?,
        Backend::Apt => handle_apt_update()?,
    };
    if check_only {
        return Ok(());
    }

    let pkgs = installed_packages(backend);
    if pkgs.is_empty() {
        return Ok(()); // status handler already printed the installer hint
    }

    // APT's lists dir is root-owned: `apt list --upgradable` only sees the
    // last refresh, so an empty result there is a stale-cache guess until
    // `apt-get update` runs. DNF refreshes per-user metadata inside
    // `check-update` for free — no privilege needed.
    if backend == Backend::Apt {
        run_step(&["apt-get", "update", "-y"], 0)?;
    }

    // Pre-flight: skip the privileged upgrade entirely when nothing pending.
    let pending = upgradable_packages(backend, &pkgs);
    match &pending {
        Some(v) if v.is_empty() => {
            println!(" [✔] All IdleScreen packages are up to date.");
            return Ok(());
        }
        Some(v) => {
            println!(" {} update(s) available:", v.len());
            for (name, ver) in v {
                println!("    {name} → {ver}");
            }
        }
        // Indeterminate — run the upgrade and let the package manager decide.
        None => println!(" Checking {} IdleScreen package(s)...", pkgs.len()),
    }

    // Upgrade only what is actually pending when the manager told us; the
    // full IdleScreen set otherwise. Never touches the rest of the system.
    let names: Vec<&str> = match &pending {
        Some(v) => v.iter().map(|(n, _)| n.as_str()).collect(),
        None => pkgs.iter().map(String::as_str).collect(),
    };
    // --refresh on the privileged side: root's dnf metadata cache is
    // separate from the user's and can be older — without this the
    // upgrade sees nothing while the preflight just saw a candidate.
    let mut step: Vec<&str> = match backend {
        Backend::Dnf => vec!["dnf", "upgrade", "-y", "--refresh"],
        Backend::Apt => vec!["apt-get", "install", "--only-upgrade", "-y"],
    };
    step.extend(&names);
    run_step(&step, names.len())?;

    // Post-check: report what actually changed, not what the command claimed.
    match &pending {
        Some(v) => {
            let mut missed = 0usize;
            for (name, cand) in v {
                match installed_version(backend, name) {
                    // after >= cand: a newer build may have been published
                    // between the preflight check and the upgrade — that
                    // is success, not a miss.
                    Some(after) if !version_cmp(&after, cand).is_lt() => {
                        println!(" [✔] {name} → {after}");
                    }
                    other => {
                        let found = other.unwrap_or_else(|| "not installed".into());
                        println!(" [!] {name}: expected {cand}, found {found}");
                        missed += 1;
                    }
                }
            }
            if missed > 0 {
                anyhow::bail!("{missed} package(s) did not reach the candidate version");
            }
        }
        None => match tracked.and_then(|(pkg, before)| {
            installed_version(backend, &pkg).map(|after| (pkg, before, after))
        }) {
            Some((pkg, before, after)) if !versions_equalish(&before, &after) => {
                println!(" [✔] Upgraded: {pkg} {before} → {after}");
            }
            Some(_) => println!(" [✔] IdleScreen packages already up to date."),
            None => {
                println!(" [i] Upgrade command finished; could not verify versions.")
            }
        },
    }
    Ok(())
}

/// Echo `step` (eliding a long trailing package list) and run it privileged.
fn run_step(step: &[&str], tail_pkgs: usize) -> Result<()> {
    let shown = if tail_pkgs > 4 {
        format!(
            "{} <{tail_pkgs} packages>",
            step[..step.len() - tail_pkgs].join(" ")
        )
    } else {
        step.join(" ")
    };
    println!(" $ {shown}");
    match run_privileged(step) {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => {
            println!(" [!] {} exited with {}", step[0], s);
            println!("     -> retry manually: sudo {}", step.join(" "));
            anyhow::bail!("package upgrade failed: {} exited {}", step[0], s)
        }
        Err(e) => {
            println!(" [!] Upgrade needs root: {e}");
            println!("     -> sudo {}", step.join(" "));
            anyhow::bail!("package upgrade needs root: {e}")
        }
    }
}

#[cfg(test)]
#[path = "self_update_tests.rs"]
mod tests;
