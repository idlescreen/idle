// SPDX-License-Identifier: MIT

//! Main idle-detection tick loop and per-tick command dispatch.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use wayland_idle::IdleMonitor;
use wayland_present::OverlayPresenter;

use super::idle_logic::update_presentation_state;
use super::presentation::{
    ActivePresentation, stop_presentation,
};
use super::runtime::{
    check_runtime_alive, recovery_plan, RuntimeFault,
};
use crate::controller::{DaemonCommand, DaemonController, MAIN_LOOP_INTERVAL};

pub fn tick_loop_until_shutdown(controller: Arc<DaemonController>) -> anyhow::Result<()> {
    let (mut idle_monitor, mut overlay_presenter) =
        super::runtime::initialize_runtime(&controller)?;
    let mut presentation = ActivePresentation::None;
    let mut preview_name: Option<String> = None;
    let mut current_saver = String::new();
    let mut tick_counter = 0u32;

    while !controller.shutdown.load(Ordering::Relaxed) {
        std::thread::sleep(MAIN_LOOP_INTERVAL);
        tick_counter = tick_counter.saturating_add(1);

        if let Err(fault) = check_runtime_alive(&idle_monitor, &overlay_presenter) {
            recover_runtime(
                fault,
                &controller,
                &mut idle_monitor,
                &mut overlay_presenter,
                &mut presentation,
                &mut preview_name,
                &mut current_saver,
            );
            // Skip the rest of this tick; next tick uses recovered runtimes if any.
            continue;
        }

        dispatch_tick_commands(
            &controller,
            &overlay_presenter,
            &mut idle_monitor,
            &mut presentation,
            &mut preview_name,
            &mut current_saver,
        );
        if let Some(timeout) = controller.reload_config_if_due(tick_counter) {
            idle_monitor.set_timeout(timeout);
        }

        // One config snapshot per tick for presentation decisions.
        let config = controller
            .config
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let system_idle = idle_monitor.is_idle();
        let session_locked = controller.session_locked.load(Ordering::Relaxed);
        let mut inhibited = controller.inhibitors.is_inhibited();

        if is_on_battery() {
            inhibited = true;
        }

        // Skip presentation updates if presenter is dead (recovery pending).
        if overlay_presenter.is_alive() {
            update_presentation_state(
                &overlay_presenter,
                &mut presentation,
                &mut preview_name,
                &mut current_saver,
                &config,
                system_idle,
                session_locked,
                inhibited,
            );
        }

        controller.update_live_state(
            system_idle,
            presentation.is_active(),
            preview_name.is_some(),
            &current_saver,
        );
        controller.publish_status_if_dirty();
    }

    stop_presentation(Some(&overlay_presenter), &mut presentation);
    Ok(())
}

/// Handle Wayland subsystem death without exiting the daemon process.
#[tracing::instrument(skip_all, fields(?fault))]
fn recover_runtime(
    fault: RuntimeFault,
    controller: &DaemonController,
    idle_monitor: &mut IdleMonitor,
    overlay_presenter: &mut Arc<OverlayPresenter>,
    presentation: &mut ActivePresentation,
    preview_name: &mut Option<String>,
    current_saver: &mut String,
) {
    let plan = recovery_plan(fault);
    tracing::error!(
        ?fault,
        stop_presentation = plan.stop_presentation,
        clear_preview = plan.clear_preview,
        recreate_presenter = plan.recreate_presenter,
        recreate_idle_monitor = plan.recreate_idle_monitor,
        exit_process = plan.exit_process,
        "wayland runtime fault — recovering without exiting the daemon"
    );

    if plan.stop_presentation {
        stop_presentation(Some(overlay_presenter), presentation);
        current_saver.clear();
    }
    if plan.clear_preview {
        if let Some(name) = preview_name.take() {
            tracing::warn!(
                preview = %name,
                "cleared preview after wayland fault (try preview again after recovery)"
            );
        }
    }

    if plan.recreate_presenter {
        match OverlayPresenter::new() {
            Some(p) => {
                *overlay_presenter = Arc::new(p);
                tracing::info!("overlay presenter recreated successfully");
            }
            None => {
                tracing::error!(
                    "failed to recreate overlay presenter; previews/idle visuals unavailable until restart"
                );
            }
        }
    }

    if plan.recreate_idle_monitor {
        let mins = controller
            .config
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .idle_timeout_mins;
        match IdleMonitor::new(mins) {
            Some(m) => {
                *idle_monitor = m;
                tracing::info!("idle monitor recreated successfully");
            }
            None => {
                tracing::error!(
                    "failed to recreate idle monitor; idle detection unavailable until restart"
                );
            }
        }
    }

    // Never exit the process — systemd would bounce us, but we must stay up.
    debug_assert!(!plan.exit_process);
}

fn dispatch_tick_commands(
    controller: &DaemonController,
    overlay_presenter: &Arc<OverlayPresenter>,
    idle_monitor: &mut IdleMonitor,
    presentation: &mut ActivePresentation,
    preview_name: &mut Option<String>,
    current_saver: &mut String,
) {
    for command in controller.drain_commands() {
        match command {
            DaemonCommand::Preview(name) => {
                tracing::info!(saver = %name, "queued preview command");
                *preview_name = Some(name);
            }
            DaemonCommand::StopPresentation => {
                tracing::info!("queued stop-presentation command");
                *preview_name = None;
                stop_presentation(Some(overlay_presenter), presentation);
                current_saver.clear();
            }
            DaemonCommand::SetTimeout(minutes) => {
                let _ = controller.apply_command(DaemonCommand::SetTimeout(minutes));
                idle_monitor.set_timeout(minutes);
            }
            DaemonCommand::Enable
            | DaemonCommand::Disable
            | DaemonCommand::SetSaver(_)
            | DaemonCommand::SetShowFpsOverlay(_)
            | DaemonCommand::SetRenderScale(_) => {
                let _ = controller.apply_command(command);
            }
        }
    }
}

fn is_on_battery() -> bool {
    let path = std::path::Path::new("/sys/class/power_supply");
    if let Ok(entries) = std::fs::read_dir(path) {
        let mut has_ac = false;
        let mut ac_online = true;
        let mut battery_discharging = false;

        for entry in entries.flatten() {
            let p = entry.path();
            if let Ok(t) = std::fs::read_to_string(p.join("type")) {
                let type_str = t.trim();
                if type_str == "Mains" {
                    has_ac = true;
                    if let Ok(o) = std::fs::read_to_string(p.join("online")) {
                        ac_online = o.trim() != "0";
                    }
                } else if type_str == "Battery"
                    && let Ok(s) = std::fs::read_to_string(p.join("status"))
                    && s.trim() == "Discharging"
                {
                    battery_discharging = true;
                }
            }
        }

        if (has_ac && !ac_online) || battery_discharging {
            return true;
        }
    }
    false
}
