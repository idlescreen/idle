// SPDX-License-Identifier: MIT

//! Background idle daemon: Wayland idle detection, overlay presentation, D-Bus API.
//!
//! `run_daemon` is the orchestrator; setup helpers stay here and the runtime
//! tick loop lives in sibling modules.

pub mod battery;
pub(crate) mod idle_decision;
#[cfg(test)]
mod liveness_validation_tests;
#[cfg(test)]
mod m2_concurrency_stress_tests;
pub(crate) mod pidfile;
pub(crate) mod presentation;
pub(crate) mod preview_queue;
pub(crate) mod recovery;
pub(crate) mod runtime;
pub(crate) mod tick_loop;
pub(crate) mod watchdog;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use anyhow::{Context, anyhow};

use crate::config::DaemonConfig;
use crate::controller::DaemonController;

pub use tick_loop::tick_loop_until_shutdown;

#[tracing::instrument(skip_all)]
pub fn run_daemon() -> anyhow::Result<()> {
    check_wayland_env()?;
    let Some(pidfile) = pidfile::acquire_pidfile()? else {
        return Ok(());
    };
    let config = DaemonConfig::load();
    // Honor config.yaml strict_control for D-Bus auth (env is the auth check surface).
    if config.strict_control {
        // SAFETY: single-threaded startup before other threads read the flag.
        unsafe {
            std::env::set_var("IDLE_STRICT_CONTROL", "1");
        }
    }
    let controller = Arc::new(DaemonController::new(config));
    crate::config_watcher::start_config_watcher(controller.clone());
    install_signal_handlers(&controller)?;
    log_daemon_startup();
    runtime::log_posture();
    let dbus_handle = spawn_dbus_thread(Arc::clone(&controller))?;
    let result = tick_loop_until_shutdown(Arc::clone(&controller));
    controller.shutdown.store(true, Ordering::Relaxed);
    let _ = dbus_handle.join();
    pidfile::release_pidfile(&pidfile);
    result
}

fn check_wayland_env() -> anyhow::Result<()> {
    if std::env::var("WAYLAND_DISPLAY").is_err() {
        return Err(anyhow!(
            "WAYLAND_DISPLAY is not set; IdleScreen requires a Wayland session"
        ));
    }
    Ok(())
}

fn install_signal_handlers(controller: &Arc<DaemonController>) -> anyhow::Result<()> {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }
    signal_hook::flag::register(
        signal_hook::consts::SIGINT,
        Arc::clone(&controller.shutdown),
    )
    .context("registering SIGINT handler")?;
    signal_hook::flag::register(
        signal_hook::consts::SIGTERM,
        Arc::clone(&controller.shutdown),
    )
    .context("registering SIGTERM handler")?;
    Ok(())
}

fn log_daemon_startup() {
    tracing::info!("idle-daemon running (pid {})...", std::process::id());
    if cfg!(debug_assertions) {
        tracing::warn!(
            "WARNING — debug build is very slow (~1 FPS). \
             Use target/release/idle-daemon for real performance."
        );
    }
}

fn spawn_dbus_thread(
    controller: Arc<DaemonController>,
) -> anyhow::Result<std::thread::JoinHandle<()>> {
    let ctrl = controller.clone();
    let handle = std::thread::spawn(move || {
        let mut retries = 0;
        while !controller.shutdown.load(Ordering::Relaxed) && retries < 3 {
            let start = std::time::Instant::now();
            let ctrl_loop = ctrl.clone();
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                crate::dbus_server::run(ctrl_loop)
            }));
            if start.elapsed() > std::time::Duration::from_secs(5) {
                retries = 0;
            }
            match res {
                Ok(Err(error)) => {
                    tracing::error!("D-Bus server stopped: {error}");
                    retries += 1;
                }
                Err(payload) => {
                    let msg = payload
                        .downcast_ref::<&str>()
                        .copied()
                        .or_else(|| payload.downcast_ref::<String>().map(|s| s.as_str()))
                        .unwrap_or("unknown panic");
                    tracing::error!("D-Bus server thread panicked: {msg}");
                    retries += 1;
                }
                Ok(Ok(())) => break,
            }
            if controller.shutdown.load(Ordering::Relaxed) {
                break;
            }
            tracing::warn!("Restarting D-Bus server after error or panic...");
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        if retries >= 3 {
            tracing::error!("D-Bus server thread exceeded max retries (3); stopping D-Bus thread.");
        }
    });
    Ok(handle)
}
