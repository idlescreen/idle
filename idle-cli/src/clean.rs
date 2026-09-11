// SPDX-License-Identifier: MIT

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use idle_dbus::daemon_available;

struct CleanPlan {
    stale_pid: Option<PathBuf>,
    active_pid: Option<PathBuf>,
    cache_dir: Option<PathBuf>,
}

fn plan() -> CleanPlan {
    let pid_path = std::env::var("XDG_RUNTIME_DIR")
        .map(|d| PathBuf::from(d).join("idle-daemon.pid"))
        .unwrap_or_else(|_| std::env::temp_dir().join("idle-daemon.pid"));
    let cache_dir = std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".cache").join("idle"))
        .filter(|p| p.exists());
    let pid_exists = pid_path.exists();
    CleanPlan {
        stale_pid: (pid_exists && !daemon_available()).then_some(pid_path.clone()),
        active_pid: (pid_exists && daemon_available()).then_some(pid_path),
        cache_dir,
    }
}

#[tracing::instrument]
pub fn handle_clean(dry_run: bool) -> Result<()> {
    let plan = plan();

    if dry_run {
        println!("Dry run — nothing will be removed.");
        let mut found = false;
        if let Some(p) = &plan.stale_pid {
            println!(" would remove stale PID file: {}", p.display());
            found = true;
        }
        if let Some(p) = &plan.active_pid {
            println!(" would keep PID file (daemon running): {}", p.display());
        }
        if let Some(p) = &plan.cache_dir {
            println!(" would remove cache directory: {}", p.display());
            found = true;
        }
        if !found && plan.active_pid.is_none() {
            println!(" nothing to clean.");
        }
        return Ok(());
    }

    if !crate::quiet() {
        println!("Cleaning IdleScreen workspace files...");
    }
    if let Some(pid_path) = &plan.stale_pid {
        fs::remove_file(pid_path)
            .with_context(|| format!("failed to delete stale PID file: {}", pid_path.display()))?;
        if !crate::quiet() {
            println!(" [✔] Removed stale PID file: '{}'", pid_path.display());
        }
    } else if !crate::quiet() {
        match &plan.active_pid {
            Some(p) => println!(
                " [!] Daemon is currently running. Keeping PID file: {}",
                p.display()
            ),
            None => println!(" [✔] No stale PID files found."),
        }
    }

    match &plan.cache_dir {
        Some(dir) => match fs::remove_dir_all(dir) {
            Ok(()) => {
                if !crate::quiet() {
                    println!(" [✔] Cleared cache directory: '{}'", dir.display())
                }
            }
            Err(e) => println!(" [!] Warning: Failed to clear cache directory: {e}"),
        },
        None => {
            if !crate::quiet() {
                println!(" [✔] Cache directory is already clean.")
            }
        }
    }
    if !crate::quiet() {
        println!("Cleanup completed successfully.");
    }
    Ok(())
}
