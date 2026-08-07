// SPDX-License-Identifier: MIT

//! Landlock filesystem sandbox for plugin host processes.
//!
//! Policy: handle all FS rights, then allow **only** explicit path trees
//! (plugin directory + optional font dirs). Empty allowlists are rejected —
//! that would deny-all and break `dlopen`, or force operators into escape hatches.

use landlock::{
    ABI, Access, AccessFs, PathBeneath, PathFd, Ruleset, RulesetAttr, RulesetCreatedAttr,
};
use std::path::Path;

/// True when sandbox may be skipped.
///
/// - Debug builds: `IDLE_DISABLE_SANDBOX=1` alone is enough (dev loop).
/// - Release: requires **both** `IDLE_DISABLE_SANDBOX=1` and
///   `IDLE_RENDER_PIPELINE=1` (offline export only). IPC children clear these.
pub fn sandbox_skip_allowed() -> bool {
    if !idle_api::env_truthy(&["IDLE_DISABLE_SANDBOX"]) {
        return false;
    }
    if cfg!(debug_assertions) {
        return true;
    }
    idle_api::env_truthy(&["IDLE_RENDER_PIPELINE"])
}

/// Strip ambient sandbox escape env from the process (call at `run-ipc-runner`).
pub fn clear_sandbox_escape_env() {
    // SAFETY: single-threaded startup of the IPC child before other threads.
    unsafe {
        std::env::remove_var("IDLE_DISABLE_SANDBOX");
        std::env::remove_var("IDLE_DEV_PLUGINS");
        // Do not clear IDLE_RENDER_PIPELINE here — render host may set it intentionally
        // only on the offline binary, never on daemon IPC children (those should not set it).
    }
}

/// Enforce Landlock with read access to `plugin_path`'s directory (and parents as needed).
pub fn enforce_sandbox_for_plugin(plugin_path: &Path) -> Result<(), String> {
    if sandbox_skip_allowed() {
        tracing::warn!(
            "Landlock sandbox DISABLED (IDLE_DISABLE_SANDBOX) — offline/render or debug only"
        );
        return Ok(());
    }

    let plugin_path = plugin_path
        .canonicalize()
        .map_err(|e| format!("canonicalize plugin path: {e}"))?;
    let parent = plugin_path
        .parent()
        .ok_or_else(|| "plugin path has no parent directory".to_string())?;

    let abi = ABI::V1;
    let read_exec = AccessFs::from_read(abi);

    let mut ruleset = Ruleset::default()
        .handle_access(AccessFs::from_all(abi))
        .map_err(|e| format!("Failed to initialize ruleset: {e}"))?
        .create()
        .map_err(|e| format!("Failed to create ruleset: {e}"))?;

    // Plugin dir: ReadFile|ReadDir|Execute so `dlopen` of the .so works.
    let plugin_dir_fd =
        PathFd::new(parent).map_err(|e| format!("PathFd plugin dir {}: {e}", parent.display()))?;
    ruleset = ruleset
        .add_rule(PathBeneath::new(plugin_dir_fd, read_exec))
        .map_err(|e| format!("add_rule plugin dir: {e}"))?;

    // Fonts (caption path may still touch them after init in some code paths).
    for font_root in ["/usr/share/fonts", "/usr/share/fontconfig"] {
        if Path::new(font_root).is_dir() {
            match PathFd::new(font_root) {
                Ok(fd) => {
                    ruleset = ruleset
                        .add_rule(PathBeneath::new(fd, read_exec))
                        .map_err(|e| format!("add_rule {font_root}: {e}"))?;
                }
                Err(e) => tracing::debug!("skip font root {font_root}: {e}"),
            }
        }
    }

    let status = ruleset
        .restrict_self()
        .map_err(|e| format!("Failed to enforce Landlock sandbox: {e}"))?;

    tracing::info!(
        plugin = %plugin_path.display(),
        "Landlock filesystem sandbox enforced: {:?}",
        status
    );
    Ok(())
}

/// Backward-compatible entry used by tests / call sites without a path.
/// Prefer [`enforce_sandbox_for_plugin`].
pub fn enforce_sandbox_or_skip_for_render() -> Result<(), String> {
    // Without a plugin path we cannot build a safe allowlist. Only allow skip
    // when the render/debug escape is set; otherwise fail closed.
    if sandbox_skip_allowed() {
        tracing::warn!("Landlock sandbox DISABLED via escape hatch (no path)");
        return Ok(());
    }
    Err(
        "enforce_sandbox_or_skip_for_render requires a plugin path; use enforce_sandbox_for_plugin"
            .into(),
    )
}

#[cfg(test)]
#[path = "sandbox_tests.rs"]
mod sandbox_tests;
