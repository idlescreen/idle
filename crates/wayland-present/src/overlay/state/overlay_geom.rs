// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Geometry helpers for layer-shell overlays (fullscreen savers vs panel inset).

use std::collections::HashMap;

use wayland_client::protocol::wl_surface;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1;

use super::types::SessionState;

impl SessionState {
    /// Buffer / layout size for presentation.
    ///
    /// Prefer the **native output mode** when it is larger than the layer-shell
    /// configure (COSMIC often configures 1920×1040 under a panel). Savers must
    /// cover the full display including the panel bar.
    pub(crate) fn render_dimensions(
        output_id: u32,
        configured_w: u32,
        configured_h: u32,
        mode_sizes: &HashMap<u32, (u32, u32)>,
        fullscreen: bool,
    ) -> (u32, u32) {
        if !fullscreen {
            return (configured_w, configured_h);
        }
        let Some((native_w, native_h)) = mode_sizes.get(&output_id).copied() else {
            return (configured_w, configured_h);
        };
        (native_w.max(configured_w), native_h.max(configured_h))
    }

    /// Expand under the panel with **negative** margins when fullscreen and the
    /// configure is inset vs native mode.
    ///
    /// Does **not** commit — caller must `ack_configure` then commit once.
    #[allow(clippy::cast_possible_wrap)]
    pub(crate) fn apply_tiling_margins(
        layer_surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        _surface: &wl_surface::WlSurface,
        output_id: u32,
        configured_w: u32,
        configured_h: u32,
        mode_sizes: &HashMap<u32, (u32, u32)>,
        fullscreen: bool,
    ) {
        if !fullscreen {
            layer_surface.set_margin(0, 0, 0, 0);
            return;
        }
        let Some((native_w, native_h)) = mode_sizes.get(&output_id).copied() else {
            layer_surface.set_margin(0, 0, 0, 0);
            return;
        };

        // Half the missing pixels on each side (panel often top/bottom).
        let inset_x = native_w.saturating_sub(configured_w) / 2;
        let inset_y = native_h.saturating_sub(configured_h) / 2;
        let m_y = if inset_y > 0 {
            -(inset_y as i32)
        } else {
            0
        };
        let m_x = if inset_x > 0 {
            -(inset_x as i32)
        } else {
            0
        };
        layer_surface.set_margin(m_y, m_x, m_y, m_x);
    }

    /// Exclusive zone for layer-shell: `-1` = surface wants full exclusive area
    /// (covers panel); `0` = no exclusive claim (panel may remain).
    pub(crate) fn exclusive_zone_for(fullscreen: bool) -> i32 {
        if fullscreen { -1 } else { 0 }
    }
}

#[cfg(test)]
mod geom_tests {
    use super::SessionState;
    use std::collections::HashMap;

    #[test]
    fn fullscreen_expands_to_native_mode() {
        let mut modes = HashMap::new();
        modes.insert(1u32, (1920u32, 1080u32));
        assert_eq!(
            SessionState::render_dimensions(1, 1920, 1040, &modes, true),
            (1920, 1080)
        );
    }

    #[test]
    fn non_fullscreen_keeps_configure() {
        let mut modes = HashMap::new();
        modes.insert(1u32, (1920u32, 1080u32));
        assert_eq!(
            SessionState::render_dimensions(1, 1920, 1040, &modes, false),
            (1920, 1040)
        );
    }

    #[test]
    fn exclusive_zone_fullscreen_is_minus_one() {
        assert_eq!(SessionState::exclusive_zone_for(true), -1);
        assert_eq!(SessionState::exclusive_zone_for(false), 0);
    }
}
