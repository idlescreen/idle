// SPDX-License-Identifier: MIT

//! Platform-agnostic runtime initialization and liveness checks.
//!
//! On Linux the idle source is `wayland_idle::IdleMonitor` (the `ext-idle-notify-v1`
//! implementation); on other targets `idle_api::platform_idle()` returns a
//! `Box<dyn IdleSource>` stub that fails closed (Sprint 05 will replace it).
//!
//! Overlay surface: Linux returns `Arc<dyn OverlaySurface>` via the
//! `WaylandOverlay` adapter that lifts `OverlayPresenter` onto the
//! platform-agnostic trait. Sprint 05 H1/H2 drop in macOS/Windows impls
//! without changing daemon call sites.

use anyhow::anyhow;
use std::sync::Arc;
use std::time::Duration;

use crate::controller::DaemonController;
use idle_api::{IdleSource, OverlaySurface};

pub use super::recovery::*;

pub fn initialize_runtime(
    controller: &DaemonController,
) -> anyhow::Result<(Box<dyn IdleSource>, Arc<dyn OverlaySurface>)> {
    let idle_timeout = controller
        .config
        .lock()
        .unwrap_or_else(|p| crate::locks::poison_or_exit("config", p))
        .idle_timeout_mins;

    #[cfg(target_os = "linux")]
    let idle_monitor: Box<dyn IdleSource> = {
        use wayland_idle::IdleMonitor;
        let timeout = Duration::from_secs(idle_timeout.saturating_mul(60) as u64);
        Box::new(IdleMonitor::new_timeout(timeout).ok_or_else(|| {
            anyhow!(
                "DEGRADED: Wayland idle monitoring unavailable (need ext-idle-notify-v1). \
                 IdleScreen is a compositor client — this DE/compositor does not expose the \
                 idle protocol. See docs/BOUNDARIES.md. Run: idle doctor --json"
            )
        })?)
    };
    #[cfg(not(target_os = "linux"))]
    let idle_monitor: Box<dyn IdleSource> = idle_api::platform_idle(Duration::from_secs(
        idle_timeout.saturating_mul(60) as u64,
    ))
    .ok_or_else(|| {
        anyhow!(
            "DEGRADED: idle source unavailable on this platform. Sprint 05 will land \
             macOS / Windows impls; see SPRINT.md."
        )
    })?;

    tracing::info!("using platform idle source");
    if !idle_monitor.is_alive() {
        return Err(anyhow!(
            "DEGRADED: idle source reports dead at startup; refusing to load"
        ));
    }

    if !idle_runner::cell_renderer::font_available() {
        return Err(anyhow!(
            "DEGRADED: no monospace font found; install fonts-dejavu-core (or equivalent) before running idle. Run: idle doctor"
        ));
    }
    if let Some(path) = idle_runner::cell_renderer::resolve_font_path() {
        tracing::info!("using monospace font: {path}");
    }

    // Trait seam: prefer `WaylandOverlay::new()` so the daemon can later
    // hold `Arc<dyn OverlaySurface>` instead of `Arc<OverlayPresenter>`.
    // For now we unwrap to the concrete presenter (the trait's `is_visible`,
    // `submit_frame`, `is_alive` are all the same signature as the
    // presenter's, except for Arc<Vec<u8>> vs Vec<u8>). The concrete path
    // is used by the presentation pipeline (which has additional methods
    // like `show_screensaver`); Sprint 05 will move those onto the trait.
    let overlay_presenter: Arc<dyn OverlaySurface> = idle_api::WaylandOverlay::new()
        .map(|w| Arc::new(w) as Arc<dyn OverlaySurface>)
        .ok_or_else(|| {
            anyhow!(
                "DEGRADED: Wayland layer-shell presenter unavailable (need zwlr_layer_shell_v1). \
                 IdleScreen presents as a guest overlay — compositors without layer-shell cannot \
                 host it (e.g. some GNOME configurations). See docs/BOUNDARIES.md. Run: idle doctor --json"
            )
        })?;
    tracing::info!("using Wayland layer-shell presenter");
    Ok((idle_monitor, overlay_presenter))
}

pub fn check_runtime_alive(
    idle_monitor: &dyn IdleSource,
    overlay_presenter: &dyn OverlaySurface,
) -> Result<(), RuntimeFault> {
    classify_runtime(idle_monitor.is_alive(), overlay_presenter.is_alive())
}
