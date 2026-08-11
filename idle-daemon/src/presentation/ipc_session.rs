// SPDX-License-Identifier: MIT

use idle_api::TerminalCell;
use idle_ipc::{IpcCommand, IpcResponse, SharedMemory};
use idle_runner::cell_renderer::CellRenderer;
use idle_runner::launcher::LaunchMode;
use idle_upscaler::{FilterMode, FrameUpscaler, resolve_render_scale};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Child;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use super::ipc_init::initialize_ipc_session;
use super::ipc_raster::raster_viewport_into;

pub struct IpcPluginSession {
    pub(crate) saver_name: String,
    gpu_enabled: bool,
    render_scale: f32,
    renderer: CellRenderer,
    upscaler: FrameUpscaler,
    pub(crate) grid: Vec<TerminalCell>,
    content_buf: Vec<u8>,
    hardware_scaling: bool,

    pub(crate) child: Option<Child>,
    pub(crate) socket: Option<UnixStream>,
    pub(crate) shm: Option<SharedMemory>,
    pub(crate) socket_path: Option<PathBuf>,
    pub(crate) expected_stop: Arc<AtomicBool>,
    /// When true, an unexpected child exit may arm the failsafe locker once.
    pub(crate) failsafe_armed: Arc<AtomicBool>,
}

impl IpcPluginSession {
    pub fn load_with_options(
        saver_name: &str,
        _launch_mode: &LaunchMode,
        gpu_enabled: Option<bool>,
        render_scale: Option<f32>,
    ) -> Result<Self, String> {
        let renderer = CellRenderer::new().map_err(|e| e.to_string())?;
        let use_gpu = gpu_enabled.unwrap_or_else(idle_upscaler::gpu_enabled);
        let render_scale = resolve_render_scale(use_gpu, render_scale);
        let upscaler = FrameUpscaler::new(use_gpu, FilterMode::from_env());

        Ok(Self {
            saver_name: saver_name.to_string(),
            gpu_enabled: use_gpu,
            render_scale,
            renderer,
            upscaler,
            grid: Vec::new(),
            content_buf: Vec::new(),
            hardware_scaling: false,
            child: None,
            socket: None,
            shm: None,
            socket_path: None,
            expected_stop: Arc::new(AtomicBool::new(false)),
            failsafe_armed: Arc::new(AtomicBool::new(true)),
        })
    }

    pub fn render_scale(&self) -> f32 {
        self.render_scale
    }

    pub fn using_gpu_upscale(&self) -> bool {
        self.upscaler.using_gpu()
    }

    pub fn set_hardware_scaling(&mut self, enabled: bool) {
        self.hardware_scaling = enabled;
    }

    pub fn content_width(&self, cols: usize) -> u32 {
        self.renderer.content_width(cols)
    }

    pub fn content_height(&self, rows: usize) -> u32 {
        self.renderer.content_height(rows)
    }

    pub fn grid_for_pixels(&self, width: u32, height: u32) -> (usize, usize) {
        self.renderer
            .grid_for_pixels_scaled(width, height, self.render_scale)
    }

    pub fn init(&mut self, cols: usize, rows: usize) -> Result<(), String> {
        let cells = cols
            .checked_mul(rows)
            .ok_or_else(|| format!("grid size overflow: {cols}x{rows}"))?;
        self.grid = vec![TerminalCell::default(); cells];

        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(path) = self.socket_path.take() {
            let _ = std::fs::remove_file(path);
        }

        let init_res = initialize_ipc_session(
            &self.saver_name,
            cols,
            rows,
            self.gpu_enabled,
            self.render_scale,
        )?;

        self.child = Some(init_res.child);
        self.socket = Some(init_res.socket);
        self.shm = Some(init_res.shm);
        self.socket_path = Some(init_res.socket_path);

        Ok(())
    }

    pub fn set_simulation_rate(&mut self, fps: f32) {
        if let Some(ref mut socket) = self.socket {
            let cmd = IpcCommand::SetSimulationRate { hz: fps };
            if let Err(e) = cmd.write_to(&mut *socket) {
                tracing::error!("failed to send SetSimulationRate: {}", e);
                self.socket = None;
                return;
            }
            match IpcResponse::read_from(&mut *socket) {
                Ok(IpcResponse::Ack) => {}
                Ok(resp) => {
                    tracing::error!("unexpected response to SetSimulationRate: {:?}", resp);
                    self.socket = None;
                }
                Err(e) => {
                    tracing::error!("failed to read SetSimulationRate Ack: {}", e);
                    self.socket = None;
                }
            }
        }
    }

    pub fn tick(&mut self, frame_dt: Duration) {
        if let Some(ref mut socket) = self.socket {
            let cmd = IpcCommand::TickAndDraw {
                dt_micros: frame_dt.as_micros() as u64,
            };
            if let Err(e) = cmd.write_to(&mut *socket) {
                if is_timeout(&e) {
                    tracing::error!(
                        saver = %self.saver_name,
                        "IPC write timed out — saver hung inside TickAndDraw; killing child"
                    );
                    self.kill_child();
                    return;
                }
                tracing::error!("failed to send TickAndDraw: {}", e);
                self.socket = None;
            }
        }
    }

    /// Kill the saver child process. Idempotent: a second call after the
    /// child has already exited is a no-op. The caller is responsible for
    /// clearing `socket` / `shm` / `socket_path` after kill — `init` will
    /// recover them on the next session start.
    ///
    /// Subprocess isolation primitive: when the saver hangs inside an IPC
    /// command, the daemon calls this from the per-plugin watchdog. We send
    /// `SIGKILL` (not `SIGTERM`) because plugin code that ignores signals
    /// inside its own `update()` cannot be reasoned with politely.
    pub fn kill_child(&mut self) {
        if let Some(mut child) = self.child.take() {
            tracing::warn!(
                saver = %self.saver_name,
                "subprocess isolation: killing hung saver child (pid {})",
                child.id()
            );
            let _ = child.kill();
            let _ = child.wait();
        }
        self.expected_stop.store(true, std::sync::atomic::Ordering::Release);
    }

    /// True when the saver subprocess has exited (either cleanly or after
    /// `kill_child`). The IPC socket closing is the parent-side signal.
    #[allow(dead_code)]
    pub fn child_is_dead(&mut self) -> bool {
        match self.child.as_mut() {
            Some(child) => matches!(child.try_wait(), Ok(Some(_))),
            None => true,
        }
    }

    pub fn draw_frame(&mut self, grid_cols: usize, grid_rows: usize) -> (bool, bool) {
        if let Some(ref mut socket) = self.socket {
            match IpcResponse::read_from(&mut *socket) {
                Ok(IpcResponse::FrameReady { scanlines, dirty }) => {
                    if let Some(ref shm) = self.shm {
                        // SAFETY: SHM mapped for session lifetime; dims set at init.
                        match unsafe { shm.cells_mut() } {
                            Ok(cells) => {
                                if let Some(need) = grid_cols.checked_mul(grid_rows) {
                                    if self.grid.len() != need {
                                        self.grid = vec![TerminalCell::default(); need];
                                    }
                                } else {
                                    tracing::error!(
                                        "grid resize overflow: {grid_cols}x{grid_rows}"
                                    );
                                    return (false, false);
                                }
                                if dirty {
                                    // Zip avoids bounds checks on the destination grid.
                                    for (dst, src) in self.grid.iter_mut().zip(cells.iter()) {
                                        *dst = TerminalCell::from(*src);
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
                    self.socket = None;
                }
                Err(e) => {
                    if is_timeout(&e) {
                        tracing::error!(
                            saver = %self.saver_name,
                            "IPC read timed out — saver hung inside TickAndDraw; killing child"
                        );
                        self.kill_child();
                        self.socket = None;
                        return (false, false);
                    }
                    tracing::error!("failed to read response to TickAndDraw: {}", e);
                    self.socket = None;
                }
            }
        }
        (false, false)
    }

    pub fn raster_viewport(
        &mut self,
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
        let using_gpu = self.using_gpu_upscale();
        let hardware_scaling = self.hardware_scaling;
        raster_viewport_into(
            &mut self.renderer,
            &mut self.upscaler,
            &self.grid,
            hardware_scaling,
            using_gpu,
            &mut self.content_buf,
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
}

/// Default per-IPC-call read timeout. The platform layer sets a
/// `read_timeout` / `write_timeout` on the UnixStream at session init;
/// if the saver doesn't respond in time, we treat it as hung and kill
/// the child. Operators can tighten via `IDLE_IPC_READ_TIMEOUT_MS`.
pub const DEFAULT_IPC_READ_TIMEOUT: Duration = Duration::from_millis(500);

pub(crate) fn read_timeout() -> Duration {
    std::env::var("IDLE_IPC_READ_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(DEFAULT_IPC_READ_TIMEOUT)
}

/// Classify an `std::io::Error` as a socket timeout (read/write deadline
/// elapsed). The platform layer sets a `read_timeout` / `write_timeout`
/// on the UnixStream at session init; on deadline the kernel returns
/// `TimedOut` (Unix) or `WouldBlock` (fallback). When we see either,
/// the peer hung inside an IPC command — `kill_child()` is the right
/// call.
pub(crate) fn is_timeout(err: &std::io::Error) -> bool {
    matches!(
        err.kind(),
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
    )
}
