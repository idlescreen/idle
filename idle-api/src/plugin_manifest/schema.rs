// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Serde mirror of the `.idleplugin.toml` surface (schema v1).
//!
//! Unknown fields are ignored by design (forward-compat): a v1 host tolerates
//! manifests written against a later minor revision instead of failing closed.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Parsed `.idleplugin.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub plugin_id: String,
    pub plugin_version: String,
    pub api_version: u32,
    pub entry: Entry,
    #[serde(default)]
    pub capabilities: Capabilities,
    #[serde(default)]
    pub sandbox: Sandbox,
    #[serde(default)]
    pub dependencies: Dependencies,
    #[serde(default)]
    pub headless_render: HeadlessRender,
    /// Path the manifest was read from. Not part of the TOML surface.
    #[serde(skip)]
    pub source_path: PathBuf,
}

/// Plugin entry point: which runtime loads it, and which library file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub runtime: String,
    pub library: String,
}

/// Capabilities the plugin requests. Absent = denied.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub audio_capture: bool,
    #[serde(default)]
    pub audio_output: bool,
    #[serde(default)]
    pub filesystem_read: Vec<String>,
    #[serde(default)]
    pub filesystem_write: Vec<String>,
}

/// Requested sandbox profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sandbox {
    #[serde(default = "default_profile")]
    pub profile: String,
}

/// Advisory dependency lists (not enforced in Sprint 02).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependencies {
    #[serde(default)]
    pub native: Vec<String>,
    #[serde(default)]
    pub wasm: Vec<String>,
}

/// Hints for the offline/headless renderer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessRender {
    #[serde(default = "default_fps")]
    pub default_fps: u32,
    #[serde(default = "default_true")]
    pub deterministic_seed: bool,
    #[serde(default = "default_true")]
    pub gpu_optional: bool,
}

fn default_profile() -> String {
    "minimal".to_string()
}

fn default_fps() -> u32 {
    60
}

fn default_true() -> bool {
    true
}

impl Default for Sandbox {
    fn default() -> Self {
        Self {
            profile: default_profile(),
        }
    }
}

impl Default for HeadlessRender {
    fn default() -> Self {
        Self {
            default_fps: default_fps(),
            deterministic_seed: true,
            gpu_optional: true,
        }
    }
}
