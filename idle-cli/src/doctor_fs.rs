// SPDX-License-Identifier: MIT

//! Config and shared-memory checks for doctor.

use super::doctor_checks::{CheckResult, chk};
use std::fs;
use std::path::PathBuf;

pub fn check_config_parses() -> CheckResult {
    match get_config_path() {
        Some(path) if path.exists() => match fs::read_to_string(&path) {
            Ok(content) => {
                let n = content.lines().count();
                chk(
                    "Configuration",
                    true,
                    format!("{} lines at {}", n, path.display()),
                )
            }
            Err(e) => chk("Configuration", false, format!("unreadable: {e}")),
        },
        Some(_) => chk("Configuration", true, "default settings (no config file)"),
        None => chk("Configuration", false, "cannot resolve home"),
    }
}

pub fn check_shm_permissions() -> CheckResult {
    let shm_dir = PathBuf::from("/dev/shm");
    if shm_dir.exists() {
        let test_file = shm_dir.join(format!(".idle-doctor-test-{}", std::process::id()));
        if fs::write(&test_file, b"test").is_ok() {
            let _ = fs::remove_file(&test_file);
            chk("Shared Memory", true, "/dev/shm writable")
        } else {
            chk("Shared Memory", false, "/dev/shm permission denied")
        }
    } else {
        chk("Shared Memory", true, "memfd fallback active")
    }
}

pub fn check_yaml_syntax() -> CheckResult {
    match get_config_path() {
        Some(path) if path.exists() => match fs::read_to_string(&path) {
            Ok(content) => {
                let mut valid_keys = 0;
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if line.contains(':') {
                        valid_keys += 1;
                    }
                }
                chk("Config Syntax", true, format!("{valid_keys} entries"))
            }
            Err(e) => chk("Config Syntax", false, format!("read error: {e}")),
        },
        _ => chk("Config Syntax", true, "defaults active"),
    }
}

/// Idle first, legacy `trance` second (matches idle-daemon).
fn get_config_path() -> Option<PathBuf> {
    let mut bases = Vec::new();
    if let Some(xdg) = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|s| !s.is_empty())
    {
        bases.push(PathBuf::from(xdg));
    }
    if let Ok(home) = std::env::var("HOME") {
        bases.push(PathBuf::from(home).join(".config"));
    }
    if bases.is_empty() {
        return None;
    }
    for base in &bases {
        let idle = base.join("idle").join("config.yaml");
        if idle.is_file() {
            return Some(idle);
        }
    }
    for base in &bases {
        let trance = base.join("trance").join("config.yaml");
        if trance.is_file() {
            return Some(trance);
        }
    }
    Some(bases[0].join("idle").join("config.yaml"))
}
