// SPDX-License-Identifier: MIT

//! Plugin screensaver presentation on Wayland layer-shell overlays.
//!
//! A dedicated thread loads the selected plugin, renders frames at the target
//! refresh rate, and submits BGRA buffers per output. Display modes (expand,
//! mirror, primary-only, span) are handled in the frame loop submodule.

mod frame_loop;
mod frame_pacing;
mod hw_scaling;
mod ipc_init;
mod ipc_lifecycle;
mod ipc_peer;
mod ipc_raster;
mod ipc_session;
mod layout;
mod overlays;
mod plugin_loop;
mod refresh;
mod render;
pub mod topology;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

use idle_runner::launcher::LaunchMode;
use wayland_present::OverlayPresenter;

pub use plugin_loop::run_plugin_loop;

#[derive(Clone)]
pub struct PresentationOptions {
    pub gpu_enabled: bool,
    pub show_fps_overlay: bool,
    pub render_scale: Option<f32>,
    pub launch_mode: LaunchMode,
}

pub struct PluginPresentation {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl PluginPresentation {
    pub fn start(
        presenter: Arc<OverlayPresenter>,
        saver_name: String,
        options: PresentationOptions,
    ) -> Result<Self, String> {
        if !idle_runner::launcher::is_allowed_saver(&saver_name) {
            return Err(format!("invalid or disallowed saver name: {saver_name}"));
        }

        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        let presenter_for_thread = presenter.clone();

        let thread = thread::spawn(move || {
            if let Err(error) =
                run_plugin_loop(&presenter_for_thread, &saver_name, &stop_flag, options)
            {
                tracing::error!("plugin presentation ended: {error}");
                presenter_for_thread.hide();
            }
        });

        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }

    pub fn is_running(&self) -> bool {
        self.thread.as_ref().is_some_and(|t| !t.is_finished())
    }

    pub fn stop(&mut self, presenter: &OverlayPresenter) {
        self.stop.store(true, Ordering::Relaxed);
        presenter.hide();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_presentation_start_rejects_invalid_saver() {
        let presenter = match OverlayPresenter::new() {
            Some(p) => Arc::new(p),
            None => return,
        };
        let options = PresentationOptions {
            gpu_enabled: false,
            show_fps_overlay: false,
            render_scale: None,
            launch_mode: LaunchMode::Preview,
        };
        let result = PluginPresentation::start(
            presenter,
            "nonexistent_invalid_saver_123".to_string(),
            options,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_presentation_is_running_returns_false_when_thread_finished() {
        let stop = Arc::new(AtomicBool::new(false));
        let handle = thread::spawn(|| {});
        thread::sleep(std::time::Duration::from_millis(20));

        let plugin = PluginPresentation {
            stop,
            thread: Some(handle),
        };
        thread::sleep(std::time::Duration::from_millis(10));
        assert!(!plugin.is_running());
    }

    #[test]
    fn test_active_presentation_check_liveness_clears_state_on_finished_thread() {
        let stop = Arc::new(AtomicBool::new(false));
        let handle = thread::spawn(|| {});
        thread::sleep(std::time::Duration::from_millis(20));

        let plugin = PluginPresentation {
            stop,
            thread: Some(handle),
        };
        thread::sleep(std::time::Duration::from_millis(10));

        let mut active = crate::daemon::presentation::ActivePresentation::Plugin(plugin);
        let mut preview_name = Some("beams".to_string());
        let mut current_saver = "beams".to_string();

        active.check_liveness(&mut preview_name, &mut current_saver);

        assert!(!active.is_active());
        assert_eq!(current_saver, "");
        assert_eq!(preview_name, None);
    }
}
