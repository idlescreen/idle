//! Secure plugin path resolution for screensaver `.so` libraries.

use std::path::{Path, PathBuf};

use crate::launcher_resolve::{cleaned_allowed_name, search_trusted_plugin};

/// Errors that can occur during plugin loading and initialization.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("plugin name '{0}' is not in the allowlist")]
    NotAllowed(String),
    #[error("plugin path contains '..' (path traversal attempt)")]
    PathTraversal,
    #[error("invalid plugin name: {0}")]
    InvalidName(String),
    #[error("failed to load library: {0}")]
    LoadFailure(#[from] libloading::Error),
    #[error("symbol '{0}' not found in plugin")]
    SymbolMissing(&'static str),
    #[error("plugin API version {found} incompatible with host {expected}")]
    ApiVersionMismatch { found: u32, expected: u32 },
    #[error("plugin does not export the required 'idle_api_version' symbol")]
    MissingVersion,
    #[error("sandbox error: {0}")]
    Sandbox(String),
    #[error("plugin '{0}' has no .idleplugin.toml manifest")]
    ManifestMissing(String),
    #[error("plugin manifest unsupported: {0}")]
    ManifestUnsupported(String),
    #[error("plugin capability refused: {0}")]
    CapabilityMismatch(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<idle_api::plugin_manifest::ManifestError> for PluginError {
    fn from(err: idle_api::plugin_manifest::ManifestError) -> Self {
        use idle_api::plugin_manifest::ManifestError as ME;
        match err {
            ME::Missing(path) => Self::ManifestMissing(path.display().to_string()),
            other => Self::ManifestUnsupported(other.to_string()),
        }
    }
}

/// The canonical list of allowed saver basenames.
pub const ALLOWED_SAVERS: &[&str] = &[
    "aurora", "beams", "bursts", "chaos", "cosmos", "glyphs", "gnats", "radar", "storm", "hearth",
    "ripple",
];

/// Controls which directories [`resolve_saver_binary`] may search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchMode {
    /// Installed system paths only.
    Daemon,
    /// Installed paths plus local development build trees.
    Preview,
}

/// Whether `name` resolves to a built-in screensaver package.
pub fn is_allowed_saver(name: &str) -> bool {
    if name.contains('/') || name.contains('\\') {
        return false;
    }
    sanitize_saver_name(name)
        .as_deref()
        .is_some_and(|clean| ALLOWED_SAVERS.contains(&clean))
}

/// Reduce a raw name or path to a clean basename, if valid.
pub fn sanitize_saver_name(raw: &str) -> Option<String> {
    let mut stem = Path::new(raw)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(raw)
        .to_string();

    if let Some(stripped) = stem.strip_prefix("libscreensaver_") {
        stem = stripped.to_string();
    } else if let Some(stripped) = stem.strip_prefix("lib") {
        stem = stripped.to_string();
    }

    if let Some(stripped) = stem.strip_prefix("screensaver-") {
        stem = stripped.to_string();
    }
    // Package name form: idle-saver-beams → beams
    if let Some(stripped) = stem.strip_prefix("idle-saver-") {
        stem = stripped.to_string();
    }

    if !stem.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return None;
    }

    if stem.is_empty() {
        return None;
    }

    Some(stem)
}

pub use crate::launcher_trust::is_trusted_plugin_path;

/// Resolve a saver name to a trusted plugin library path.
pub fn resolve_saver_binary(name: &str, mode: &LaunchMode) -> std::io::Result<PathBuf> {
    let clean = cleaned_allowed_name(name)?;
    search_trusted_plugin(&clean, mode)
}

#[cfg(test)]
#[path = "launcher_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "launcher_proptest.rs"]
mod proptests;
