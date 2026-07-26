//! Environment helpers for IdleScreen host and plugins.
//!
//! Protocol keys are `IDLE_*` only (hard cut; no `TRANCE_*` dual-read).

/// First non-empty value among `keys` (left to right).
pub fn env_var_first(keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Ok(v) = std::env::var(key)
            && !v.is_empty()
        {
            return Some(v);
        }
    }
    None
}

/// True if any of `keys` is set (including empty string).
pub fn env_is_set(keys: &[&str]) -> bool {
    keys.iter().any(|k| std::env::var_os(k).is_some())
}

/// True if any key equals `"1"`, `"true"`, or `"TRUE"` (case-sensitive for true).
pub fn env_truthy(keys: &[&str]) -> bool {
    matches!(
        env_var_first(keys).as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

/// Set a single environment key (plugin spawn / host setup).
///
/// # Safety
/// Same constraints as [`std::env::set_var`]: not concurrent with other
/// env access from other threads in undefined ways; used at plugin spawn.
pub fn set_env(key: &str, value: impl AsRef<std::ffi::OsStr>) {
    let v = value.as_ref();
    // SAFETY: host single-threaded spawn path / export setup.
    unsafe {
        std::env::set_var(key, v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_prefers_left() {
        // SAFETY: test process isolation for env keys we own.
        unsafe {
            std::env::set_var("IDLE_TEST_A", "new");
            std::env::set_var("IDLE_TEST_B", "old");
        }
        assert_eq!(
            env_var_first(&["IDLE_TEST_A", "IDLE_TEST_B"]).as_deref(),
            Some("new")
        );
        unsafe {
            std::env::remove_var("IDLE_TEST_A");
            std::env::remove_var("IDLE_TEST_B");
        }
    }
}
