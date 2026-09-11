// SPDX-License-Identifier: MIT

//! Filesystem and config checks for doctor.

use super::doctor_checks::{CheckResult, fail, ok, warn};
use std::fs;
use std::path::PathBuf;

/// Keys the daemon's line-parser understands (`config_parse.rs`), plus
/// keys consumed by sibling tools (idle-cosmic reads the same file).
const KNOWN_KEYS: &[&str] = &[
    "idle_timeout_mins",
    "active_saver",
    "idle_enabled",
    "show_fps_overlay",
    "render_scale",
    "theme",
    "strict_control",
    // Legacy — parsed but ignored (GPU upscaler removed).
    "gpu_enabled",
];
/// Ecosystem keys owned by idle-cosmic — valid but not daemon-applied.
const SIBLING_KEYS: &[&str] = &["accent_color", "theme_idx", "dark_mode"];
/// `idle_api::Theme` variants (FromStr names, lowercase).
const THEMES: &[&str] = &[
    "synthwave",
    "cyberpunk",
    "neon",
    "aurora",
    "monokai",
    "matrix",
];

/// Real validation: every non-comment line must be `key: value` with a
/// known key and a value the daemon can parse. Replaces the old
/// "count the colons" check, which passed garbage files.
pub fn check_config() -> CheckResult {
    let Some(path) = get_config_path() else {
        return fail("Configuration", "cannot resolve config path (no HOME)");
    };
    if !path.exists() {
        return ok("Configuration", "defaults active (no config file)");
    }
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => return fail("Configuration", format!("unreadable: {e}")),
    };
    let (problems, unknown) = validate_config_text(&content);
    if !problems.is_empty() {
        return fail(
            "Configuration",
            format!("{} — {}", path.display(), problems.join("; ")),
        )
        .with_fix(format!("fix or delete {}", path.display()));
    }
    if !unknown.is_empty() {
        return warn(
            "Configuration",
            format!(
                "{} valid; ignored keys: {}",
                path.display(),
                unknown.join(", ")
            ),
        );
    }
    ok("Configuration", format!("valid at {}", path.display()))
}

/// Pure config validator: (hard problems, tolerated unknown keys).
/// Separated from the file read so tests can feed hostile text directly.
fn validate_config_text(content: &str) -> (Vec<String>, Vec<String>) {
    // Track [saver] / [saver.<name>] sections — keys inside are free-form
    // saver params (saver_params map), not daemon keys.
    let mut in_saver_section = false;
    let mut problems = Vec::new();
    let mut unknown = Vec::new();
    for (i, raw) in content.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_saver_section =
                line == "[saver]" || (line.starts_with("[saver.") && line.ends_with(']'));
            if !line.ends_with(']') {
                problems.push(format!("line {}: malformed section header", i + 1));
            }
            continue;
        }
        let Some((key, val)) = line.split_once(':') else {
            problems.push(format!("line {}: not 'key: value'", i + 1));
            continue;
        };
        if in_saver_section {
            continue; // free-form saver params
        }
        let key = key.trim();
        let val = val.trim().trim_matches('"').trim_matches('\'');
        if SIBLING_KEYS.contains(&key) {
            continue; // consumed by idle-cosmic, not the daemon
        }
        if !KNOWN_KEYS.contains(&key) {
            unknown.push(format!("'{key}' (line {})", i + 1));
            continue;
        }
        let bad = match key {
            "idle_timeout_mins" => val
                .parse::<u32>()
                .ok()
                .is_none_or(|n| !(1..=240).contains(&n)),
            "idle_enabled" | "show_fps_overlay" | "gpu_enabled" | "strict_control" => {
                val.parse::<bool>().is_err()
            }
            "render_scale" => {
                !val.is_empty()
                    && !val.eq_ignore_ascii_case("null")
                    && !val.eq_ignore_ascii_case("default")
                    && val.parse::<f32>().is_err()
            }
            "theme" => !THEMES.contains(&val.to_ascii_lowercase().as_str()),
            _ => false,
        };
        if bad {
            problems.push(format!(
                "line {}: '{key}: {val}' has an invalid value",
                i + 1
            ));
        }
    }
    (problems, unknown)
}

/// /dev/shm probe — a missing dir is a WARN (memfd fallback), not a failure.
pub fn check_shm_permissions() -> CheckResult {
    let shm_dir = PathBuf::from("/dev/shm");
    if shm_dir.exists() {
        let test_file = shm_dir.join(format!(".idle-doctor-test-{}", std::process::id()));
        if fs::write(&test_file, b"test").is_ok() {
            let _ = fs::remove_file(&test_file);
            ok("Shared Memory", "/dev/shm writable")
        } else {
            fail("Shared Memory", "/dev/shm permission denied")
                .with_fix("check /dev/shm mount and permissions")
        }
    } else {
        warn("Shared Memory", "/dev/shm absent — memfd fallback in use")
    }
}

/// Idle first, legacy `trance` second (matches idle-daemon). Shared with
/// `config path`/`edit`/`reset` so both tools resolve the same file.
pub(crate) fn get_config_path() -> Option<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::validate_config_text;

    #[test]
    fn clean_config_passes() {
        let (p, u) =
            validate_config_text("idle_timeout_mins: 5\nactive_saver: storm\nidle_enabled: true\n");
        assert!(p.is_empty() && u.is_empty());
    }

    #[test]
    fn unknown_key_warns_not_fails() {
        let (p, u) = validate_config_text("mystery_key: 1\n");
        assert!(p.is_empty());
        assert_eq!(u.len(), 1);
    }

    #[test]
    fn sibling_keys_tolerated() {
        let (p, u) = validate_config_text("accent_color: \"#fff\"\ntheme_idx: 0\n");
        assert!(p.is_empty() && u.is_empty(), "cosmic keys are legal");
    }

    #[test]
    fn malformed_line_fails() {
        let (p, _) = validate_config_text("garbage line without colon\n");
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn bad_typed_values_fail() {
        for line in [
            "idle_timeout_mins: 999",
            "idle_timeout_mins: abc",
            "idle_enabled: maybe",
            "strict_control: yes",
            "render_scale: fast",
            "theme: ultraviolet",
        ] {
            let (p, _) = validate_config_text(line);
            assert!(!p.is_empty(), "'{line}' must fail");
        }
    }

    #[test]
    fn null_and_default_render_scale_valid() {
        for line in [
            "render_scale: null",
            "render_scale: default",
            "render_scale: 0.5",
            "render_scale:",
        ] {
            let (p, _) = validate_config_text(line);
            assert!(p.is_empty(), "'{line}' must pass");
        }
    }

    #[test]
    fn saver_section_params_are_freeform() {
        let (p, _) = validate_config_text("[saver]\nanything_goes: x\n[saver.storm]\nwind: 9\n");
        assert!(p.is_empty());
    }

    #[test]
    fn quoted_values_validate() {
        let (p, _) = validate_config_text("active_saver: \"hearth\"\nidle_enabled: 'true'\n");
        assert!(p.is_empty());
    }

    #[test]
    fn oversized_input_does_not_panic() {
        let huge = "key: value\n".repeat(200_000);
        let _ = validate_config_text(&huge);
    }
}
