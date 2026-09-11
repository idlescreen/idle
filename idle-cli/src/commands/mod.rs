// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! CLI command handlers. Public surface is re-exported for `main` stability.

mod control;
mod inhibitors_fmt;
mod status;

pub use control::{
    cmd_fps_overlay, cmd_inhibit, cmd_inhibitors, cmd_list, cmd_preview, cmd_render_scale,
    cmd_saver_set, cmd_saver_show, cmd_timeout,
};
pub use status::{cmd_status, print_version};
