// SPDX-License-Identifier: MIT

use super::ipc_session::IpcPluginSession;
use super::timeout::is_timeout;
use idle_ipc::IpcResponse;

/// Draw a frame by reading from the IPC socket and updating the grid.
/// Returns (scanlines, dirty) if frame was ready, or (false, false) on error/timeouts.
pub fn draw_frame(session: &mut IpcPluginSession, grid_cols: usize, grid_rows: usize) -> (bool, bool) {
    if let Some(ref mut socket) = session.socket {
        match IpcResponse::read_from(&mut *socket) {
            Ok(IpcResponse::FrameReady { scanlines, dirty }) => {
                if let Some(ref shm) = session.shm {
                    // SAFETY: SHM mapped for session lifetime; dims set at init.
                    match unsafe { shm.cells_mut() } {
                        Ok(cells) => {
                            if let Some(need) = grid_cols.checked_mul(grid_rows) {
                                if session.grid.len() != need {
                                    session.grid = vec![idle_api::TerminalCell::default(); need];
                                }
                            } else {
                                tracing::error!(
                                    "grid resize overflow: {grid_cols}x{grid_rows}"
                                );
                                return (false, false);
                            }
                            if dirty {
                                // Zip avoids bounds checks on the destination grid.
                                for (dst, src) in session.grid.iter_mut().zip(cells.iter()) {
                                    *dst = idle_api::TerminalCell::from(*src);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("shm cells view rejected: {e}");
                        }
                    }
                }
                return (scanlines, dirty);
            }
            Ok(resp) => {
                tracing::error!("unexpected response to TickAndDraw: {:?}", resp);
                session.socket = None;
            }
            Err(e) => {
                if is_timeout(&e) {
                    tracing::error!(
                        saver = %session.saver_name,
                        "IPC read timed out — saver hung inside TickAndDraw; killing child"
                    );
                    session.kill_child();
                    session.socket = None;
                    return (false, false);
                }
                tracing::error!("failed to read response to TickAndDraw: {}", e);
                session.socket = None;
            }
        }
    }
    (false, false)
}

/// Raster the current grid viewport into the pixel buffer.
/// Delegates to raster_viewport_into with the session's renderer, upscaler, and grid.
pub fn raster_viewport(
    session: &mut IpcPluginSession,
    col_start: usize,
    row_start: usize,
    cols: usize,
    rows: usize,
    grid_cols: usize,
    _grid_rows: usize,
    width: u32,
    height: u32,
    scanlines: bool,
    pixel_buf: &mut Vec<u8>,
) {
    let using_gpu = session.using_gpu_upscale();
    let hardware_scaling = session.hardware_scaling;
    super::ipc_raster::raster_viewport_into(
        &mut session.renderer,
        &mut session.upscaler,
        &session.grid,
        hardware_scaling,
        using_gpu,
        &mut session.content_buf,
        pixel_buf,
        col_start,
        row_start,
        cols,
        rows,
        grid_cols,
        width,
        height,
        scanlines,
    );
}
