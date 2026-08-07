// SPDX-License-Identifier: MIT

use landlock::Access;
use landlock::RulesetAttr;
use landlock::{ABI, AccessFs, Ruleset};

/// Enforce a strict Landlock filesystem sandbox on the current process.
///
/// Fails closed unless the caller has explicitly opted out via
/// `IDLE_DISABLE_SANDBOX=1` (only used by offline export / `render`).
///
/// Returns an error rather than logging-and-continuing, so plugin loading
/// paths propagate the failure and refuse to load the plugin when the kernel
/// cannot enforce the sandbox.
pub fn enforce_sandbox_or_skip_for_render() -> Result<(), String> {
    if idle_api::env_truthy(&["IDLE_DISABLE_SANDBOX"]) {
        tracing::warn!(
            "Landlock sandbox DISABLED via IDLE_DISABLE_SANDBOX — \
             only the offline render pipeline should set this"
        );
        return Ok(());
    }
    // Use ABI::V1 which is the baseline Landlock version supported since 5.13.
    // We handle all filesystem access rights to ensure a total lockdown.
    let ruleset = Ruleset::default()
        .handle_access(AccessFs::from_all(ABI::V1))
        .map_err(|e| format!("Failed to initialize ruleset: {e}"))?;

    let ruleset = ruleset
        .create()
        .map_err(|e| format!("Failed to create ruleset: {e}"))?;

    let status = ruleset
        .restrict_self()
        .map_err(|e| format!("Failed to enforce Landlock sandbox: {e}"))?;

    tracing::info!("Landlock filesystem sandbox enforced: {:?}", status);
    Ok(())
}

#[cfg(test)]
#[path = "sandbox_tests.rs"]
mod sandbox_tests;
