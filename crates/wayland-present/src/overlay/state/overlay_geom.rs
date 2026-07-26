// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Geometry helpers for layer-shell overlays.

use super::types::SessionState;

impl SessionState {
    /// Trust the compositor configure size (panel-aware). Do not expand to
    /// native mode with negative margins — that forced extra commits before
    /// `ack_configure` and caused unstable sessions on COSMIC.
    pub(crate) fn render_dimensions(
        _output_id: u32,
        configured_w: u32,
        configured_h: u32,
        _mode_sizes: &std::collections::HashMap<u32, (u32, u32)>,
    ) -> (u32, u32) {
        (configured_w, configured_h)
    }

    /// Apply zero margins only (no surface commit). Caller owns the commit
    /// after `ack_configure`.
    pub(crate) fn apply_tiling_margins(
        layer_surface: &wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _output_id: u32,
        _configured_w: u32,
        _configured_h: u32,
        _mode_sizes: &std::collections::HashMap<u32, (u32, u32)>,
    ) {
        layer_surface.set_margin(0, 0, 0, 0);
    }
}

#[cfg(test)]
mod geom_tests {
    use super::SessionState;
    use std::collections::HashMap;

    #[test]
    fn render_dimensions_trusts_configure() {
        let mut modes = HashMap::new();
        modes.insert(1u32, (1920u32, 1080u32));
        // Configure is panel-inset; do not expand to full mode.
        assert_eq!(
            SessionState::render_dimensions(1, 1920, 1040, &modes),
            (1920, 1040)
        );
    }
}
