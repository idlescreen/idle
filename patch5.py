import sys

def replace_in_file(path, old, new):
    with open(path, "r") as f:
        content = f.read()
    if old not in content:
        print("OLD not found in content!")
    with open(path, "w") as f:
        f.write(content.replace(old, new))

replace_in_file(
    "idle/idle-daemon/src/presentation/render.rs",
    "let mut pixels = s.session.raster_viewport(\n                    0, 0, s.cols, s.rows, s.cols, s.rows, target_w, target_h, scanlines,\n                );\n                apply_fade_in(\n                    std::sync::Arc::make_mut(&mut pixels).as_mut_slice(),\n                    state.frame_start.duration_since(state.session_start),\n                );\n                maybe_draw_overlays(\n                    std::sync::Arc::make_mut(&mut pixels).as_mut_slice(),\n                    target_w,\n                    target_h,\n                    layout.id == state.primary.id,\n                    state.options.show_fps_overlay,\n                    state.achieved_fps,\n                );\n                state\n                    .presenter\n                    .submit_frame(layout.id, target_w, target_h, pixels);",
    "let mut pixels = state.presenter.get_frame_buffer((target_w * target_h * 4) as usize);\n                s.session.raster_viewport(\n                    0, 0, s.cols, s.rows, s.cols, s.rows, target_w, target_h, scanlines, &mut pixels\n                );\n                apply_fade_in(\n                    &mut pixels,\n                    state.frame_start.duration_since(state.session_start),\n                );\n                maybe_draw_overlays(\n                    &mut pixels,\n                    target_w,\n                    target_h,\n                    layout.id == state.primary.id,\n                    state.options.show_fps_overlay,\n                    state.achieved_fps,\n                );\n                state\n                    .presenter\n                    .submit_frame(layout.id, target_w, target_h, pixels);"
)

replace_in_file(
    "idle/idle-daemon/src/presentation/render.rs",
    "let mut pixels = s.session.raster_viewport(\n                bounds.start_col,\n                bounds.start_row,\n                col_w,\n                row_h,\n                s.cols,\n                s.rows,\n                target_w,\n                target_h,\n                scanlines,\n            );\n            apply_fade_in(\n                std::sync::Arc::make_mut(&mut pixels).as_mut_slice(),\n                state.frame_start.duration_since(state.session_start),\n            );\n            maybe_draw_overlays(\n                std::sync::Arc::make_mut(&mut pixels).as_mut_slice(),\n                target_w,\n                target_h,\n                layout.id == state.primary.id,\n                state.options.show_fps_overlay,\n                state.achieved_fps,\n            );\n            state\n                .presenter\n                .submit_frame(layout.id, target_w, target_h, pixels);",
    "let mut pixels = state.presenter.get_frame_buffer((target_w * target_h * 4) as usize);\n            s.session.raster_viewport(\n                bounds.start_col,\n                bounds.start_row,\n                col_w,\n                row_h,\n                s.cols,\n                s.rows,\n                target_w,\n                target_h,\n                scanlines,\n                &mut pixels,\n            );\n            apply_fade_in(\n                &mut pixels,\n                state.frame_start.duration_since(state.session_start),\n            );\n            maybe_draw_overlays(\n                &mut pixels,\n                target_w,\n                target_h,\n                layout.id == state.primary.id,\n                state.options.show_fps_overlay,\n                state.achieved_fps,\n            );\n            state\n                .presenter\n                .submit_frame(layout.id, target_w, target_h, pixels);"
)
