// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::time::Duration;

use idle_api::OutputId;

use super::frame_loop::FrameLoopState;
use super::layout::{monitor_cell_bounds, virtual_desktop};
use super::overlays::maybe_draw_overlays;

pub fn present_frame(state: &mut FrameLoopState) {
    let (min_x, min_y, total_w, total_h) = virtual_desktop(state.layouts);

    if state.independent_rendering {
        for s in state.sessions.iter_mut() {
            let (scanlines, dirty) = s.session.draw_frame(s.cols, s.rows);
            if !dirty
                && state.frame_start.duration_since(state.session_start)
                    >= Duration::from_millis(500)
            {
                continue;
            }
            if let Some(layout) = state.layouts.iter().find(|l| l.id == s.output_id) {
                let target_w = if state.use_hw_scaling {
                    s.session.content_width(s.cols)
                } else {
                    layout.width
                };
                let target_h = if state.use_hw_scaling {
                    s.session.content_height(s.rows)
                } else {
                    layout.height
                };

                let mut pixels = state
                    .presenter
                    .get_frame_buffer((target_w * target_h * 4) as usize);
                s.session.raster_viewport(
                    0,
                    0,
                    s.cols,
                    s.rows,
                    s.cols,
                    s.rows,
                    target_w,
                    target_h,
                    scanlines,
                    &mut pixels,
                );
                apply_fade_in(
                    &mut pixels,
                    state.frame_start.duration_since(state.session_start),
                );
                maybe_draw_overlays(
                    &mut pixels,
                    target_w,
                    target_h,
                    layout.id == state.primary.id,
                    state.options.show_fps_overlay,
                    state.achieved_fps,
                );
state
                .presenter
                .submit_frame(OutputId(layout.id), Arc::new(pixels), target_w, target_h);
            }
        }
    } else {
        if state.sessions.is_empty() {
            return;
        }
        let s = &mut state.sessions[0];
        let (scanlines, dirty) = s.session.draw_frame(s.cols, s.rows);
        if !dirty
            && state.frame_start.duration_since(state.session_start) >= Duration::from_millis(500)
        {
            return;
        }
        for layout in state.layouts {
            let bounds = monitor_cell_bounds(
                *layout,
                min_x,
                min_y,
                total_w,
                total_h,
                s.cols,
                s.rows,
                layout.id == state.primary.id,
            );
            let col_w = bounds.end_col.saturating_sub(bounds.start_col).max(1);
            let row_h = bounds.end_row.saturating_sub(bounds.start_row).max(1);

            let (target_w, target_h) = if state.use_hw_scaling {
                (
                    s.session.content_width(col_w),
                    s.session.content_height(row_h),
                )
            } else {
                (layout.width, layout.height)
            };

            let mut pixels = state
                .presenter
                .get_frame_buffer((target_w * target_h * 4) as usize);
            s.session.raster_viewport(
                bounds.start_col,
                bounds.start_row,
                col_w,
                row_h,
                s.cols,
                s.rows,
                target_w,
                target_h,
                scanlines,
                &mut pixels,
            );
            apply_fade_in(
                &mut pixels,
                state.frame_start.duration_since(state.session_start),
            );
            maybe_draw_overlays(
                &mut pixels,
                target_w,
                target_h,
                layout.id == state.primary.id,
                state.options.show_fps_overlay,
                state.achieved_fps,
            );
            state
                .presenter
                .submit_frame(OutputId(layout.id), Arc::new(pixels), target_w, target_h);
        }
    }
}

pub fn apply_fade_in(pixels: &mut [u8], elapsed: Duration) {
    let fade_duration = Duration::from_millis(500);
    if elapsed >= fade_duration {
        return;
    }

    let alpha_multiplier = elapsed.as_secs_f32() / fade_duration.as_secs_f32();
    let mult = (alpha_multiplier * 255.0) as u32;

    for chunk in pixels.chunks_exact_mut(4) {
        chunk[0] = ((chunk[0] as u32 * mult) / 255) as u8;
        chunk[1] = ((chunk[1] as u32 * mult) / 255) as u8;
        chunk[2] = ((chunk[2] as u32 * mult) / 255) as u8;
        chunk[3] = ((chunk[3] as u32 * mult) / 255) as u8;
    }
}
