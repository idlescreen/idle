// SPDX-License-Identifier: MIT

//! Main idle-detection tick loop delegates to the openOODA coordinator.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::controller::{DaemonController, MAIN_LOOP_INTERVAL};
use crate::ooda::OodaLoopController;

pub fn tick_loop_until_shutdown(controller: Arc<DaemonController>) -> anyhow::Result<()> {
    let (mut idle_monitor, mut overlay_presenter) =
        super::runtime::initialize_runtime(&controller)?;

    let mut ooda_loop = OodaLoopController::new();

    while !controller.shutdown.load(Ordering::Relaxed) {
        std::thread::sleep(MAIN_LOOP_INTERVAL);

        if let Err(err) = ooda_loop.step_tick(&controller, &mut idle_monitor, &mut overlay_presenter) {
            tracing::error!("error in openOODA tick cycle: {err:#}");
        }
    }

    ooda_loop.shutdown(&overlay_presenter);
    Ok(())
}
