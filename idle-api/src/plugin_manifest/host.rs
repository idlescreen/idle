// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Host-side helpers over a parsed [`Manifest`].
//!
//! Sprint 02 enforces ambient capabilities by *refusal*, not by OS mediation:
//! the host declines to load a plugin that asks for network or audio unless the
//! operator opts in. Seccomp-based enforcement is Sprint 03+ work.

use super::Manifest;
use std::path::Path;

/// Operator opt-in for plugins declaring network/audio capabilities.
pub const PERMIT_NETWORK_ENV: &str = "IDLE_PERMIT_NETWORK_PLUGINS";

/// Operator opt-in for loading a bare `.so` with no manifest.
pub const ALLOW_UNSIGNED_ENV: &str = "IDLE_ALLOW_UNSIGNED_PLUGINS";

/// Operator opt-in for the `experimental` sandbox profile.
pub const ALLOW_EXPERIMENTAL_ENV: &str = "IDLE_ALLOW_EXPERIMENTAL_PROFILES";

impl Manifest {
    /// Ambient capabilities that Sprint 02 cannot mediate at the OS level.
    ///
    /// Returns the names of every such capability the plugin requested, so the
    /// caller can log precisely what was asked for before refusing.
    pub fn ambient_capabilities(&self) -> Vec<&'static str> {
        let c = &self.capabilities;
        [
            ("network", c.network),
            ("audio_capture", c.audio_capture),
            ("audio_output", c.audio_output),
        ]
        .into_iter()
        .filter_map(|(name, requested)| requested.then_some(name))
        .collect()
    }

    /// True when the plugin requests a capability the host cannot enforce.
    pub fn requests_ambient_capabilities(&self) -> bool {
        !self.ambient_capabilities().is_empty()
    }

    /// True when `entry.library` names the library actually resolved on disk.
    ///
    /// Compared by file name: the manifest declares a bare name, while the
    /// resolved path is absolute and may traverse a symlinked install root.
    pub fn library_matches(&self, resolved: &Path) -> bool {
        resolved
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| name == self.entry.library)
    }

    /// True when this plugin targets the in-process native runtime.
    pub fn is_native(&self) -> bool {
        self.entry.runtime == "native"
    }
}

/// Whether the operator has opted into network/audio-capable plugins.
pub fn network_plugins_permitted() -> bool {
    crate::env_truthy(&[PERMIT_NETWORK_ENV])
}

/// Whether the operator has opted into manifest-less `.so` loading.
pub fn unsigned_plugins_allowed() -> bool {
    crate::env_truthy(&[ALLOW_UNSIGNED_ENV])
}

/// Whether the operator has opted into the `experimental` sandbox profile.
pub fn experimental_profiles_allowed() -> bool {
    crate::env_truthy(&[ALLOW_EXPERIMENTAL_ENV])
}
