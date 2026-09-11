// SPDX-License-Identifier: MIT

use super::doctor_checks::{CheckResult, fail, ok, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn check_fonts() -> CheckResult {
    if font_check_via_fc_list() {
        ok("System Fonts", "monospace font found")
    } else {
        fail(
            "System Fonts",
            "monospace font missing — install fonts-dejavu-core or another mono font",
        )
    }
}

/// cgroup v2 delegation — the runner's budget enforcement needs `cpu` (and
/// ideally `memory`) controllers in the user's subtree.
pub fn check_cgroup() -> CheckResult {
    let subtree = Path::new("/sys/fs/cgroup/user.slice/cgroup.subtree_control");
    let controllers = Path::new("/sys/fs/cgroup/cgroup.controllers");
    if !controllers.exists() {
        return warn(
            "Cgroup Budget",
            "cgroup v2 not mounted — saver CPU/memory limits inactive",
        );
    }
    let delegated = fs::read_to_string(controllers).unwrap_or_default();
    let subtree_ctrls = fs::read_to_string(subtree).unwrap_or_default();
    let has_cpu = delegated.contains("cpu") || subtree_ctrls.contains("cpu");
    let has_mem = delegated.contains("memory") || subtree_ctrls.contains("memory");
    match (has_cpu, has_mem) {
        (true, true) => ok("Cgroup Budget", "cpu+memory controllers available"),
        (true, false) => warn(
            "Cgroup Budget",
            "cpu only — memory.max delegation unavailable (saver OOM uncapped)",
        ),
        _ => warn(
            "Cgroup Budget",
            "no delegated controllers — saver resource limits inactive",
        )
        .with_fix("delegate cpu/memory: systemctl edit --user or cgroup2 delegation"),
    }
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
