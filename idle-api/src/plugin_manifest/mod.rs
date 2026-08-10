// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! `.idleplugin.toml` capability manifest (schema v1, `DECISION-MANIFEST-01`).
//!
//! Every plugin library ships a sibling manifest declaring its identity,
//! entry point, requested capabilities and sandbox profile. The host refuses
//! to load a plugin whose manifest is missing, unparseable, or inconsistent
//! with the resolved library — see `idle-runner`'s `plugin_session::loading`.
//!
//! The manifest is named `<stem>.idleplugin.toml` rather than a bare
//! `.idleplugin.toml` because every saver installs into one shared directory
//! (`/usr/libexec/idle/screensavers/`), where a single fixed name would collide.

mod schema;

pub mod host;

pub use schema::{Capabilities, Dependencies, Entry, HeadlessRender, Manifest, Sandbox};

use std::path::{Path, PathBuf};

/// Manifest schema version understood by this host.
pub const SCHEMA_VERSION: u32 = 1;

/// Sandbox profiles recognised by the host, loosest last.
pub const PROFILES: &[&str] = &["minimal", "renderer", "asset-author", "experimental"];

/// Why a manifest was rejected. Every variant is fail-closed at the loader.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("no manifest at {0} (plugin refused; set IDLE_ALLOW_UNSIGNED_PLUGINS=1 to override)")]
    Missing(PathBuf),
    #[error("malformed manifest {path}: {source}")]
    Parse {
        path: PathBuf,
        source: Box<toml::de::Error>,
    },
    #[error("manifest {path} declares schema_version {found}, host supports {SCHEMA_VERSION}")]
    UnsupportedSchemaVersion { path: PathBuf, found: u32 },
    #[error("manifest {0} has invalid plugin_id '{1}' (expected reverse-DNS, e.g. io.github.x.y)")]
    InvalidPluginId(PathBuf, String),
    #[error("manifest {1} invalid: {0}")]
    Invalid(String, PathBuf),
}

/// Path of the manifest that belongs to `plugin_path`.
pub fn sibling_path(plugin_path: &Path) -> PathBuf {
    let stem = plugin_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("plugin");
    plugin_path.with_file_name(format!("{stem}.idleplugin.toml"))
}

/// Read and parse the manifest belonging to `plugin_path`. Does not validate.
pub fn load_for(plugin_path: &Path) -> Result<Manifest, ManifestError> {
    let path = sibling_path(plugin_path);
    let text = std::fs::read_to_string(&path).map_err(|_| ManifestError::Missing(path.clone()))?;
    let mut manifest = parse_str(&text, &path)?;
    manifest.source_path = path;
    Ok(manifest)
}

/// Parse manifest text. `path` is used only for error reporting.
pub fn parse_str(text: &str, path: &Path) -> Result<Manifest, ManifestError> {
    toml::from_str(text).map_err(|source| ManifestError::Parse {
        path: path.to_path_buf(),
        source: Box::new(source),
    })
}

/// Enforce schema, identity and entry-point invariants.
pub fn validate(manifest: &Manifest) -> Result<(), ManifestError> {
    let path = manifest.source_path.clone();
    if manifest.schema_version != SCHEMA_VERSION {
        return Err(ManifestError::UnsupportedSchemaVersion {
            path,
            found: manifest.schema_version,
        });
    }
    if !is_reverse_dns(&manifest.plugin_id) {
        return Err(ManifestError::InvalidPluginId(
            path,
            manifest.plugin_id.clone(),
        ));
    }
    let invalid = |m: &str| Err(ManifestError::Invalid(m.to_string(), path.clone()));
    if manifest.plugin_version.trim().is_empty() {
        return invalid("plugin_version must not be empty");
    }
    if !matches!(manifest.entry.runtime.as_str(), "native" | "wasm") {
        return invalid("entry.runtime must be \"native\" or \"wasm\"");
    }
    if manifest.entry.library.trim().is_empty() || manifest.entry.library.contains('/') {
        return invalid("entry.library must be a bare file name");
    }
    if !PROFILES.contains(&manifest.sandbox.profile.as_str()) {
        return invalid("sandbox.profile is not a known profile");
    }
    Ok(())
}

/// Reverse-DNS check: three or more non-empty dot-separated labels.
fn is_reverse_dns(id: &str) -> bool {
    let labels: Vec<&str> = id.split('.').collect();
    labels.len() >= 3
        && labels.iter().all(|l| {
            !l.is_empty()
                && l.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
}
