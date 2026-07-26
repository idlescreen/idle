// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! D-Bus API for the IdleScreen screensaver daemon.
//!
//! ## Well-known names (primary only)
//!
//! - Service: [`SERVICE_NAME`] (`io.github.idlescreen.Idle`)
//! - Path: [`OBJECT_PATH`] (`/io/github/idlescreen/Idle`)
//! - Interface: [`INTERFACE_NAME`] (`io.github.idlescreen.Idle`)

pub mod client;
pub mod status;

pub use client::{TranceClient, daemon_available};
pub use status::DaemonStatus;

/// Well-known bus name.
pub const SERVICE_NAME: &str = "io.github.idlescreen.Idle";
/// Object path.
pub const OBJECT_PATH: &str = "/io/github/idlescreen/Idle";
/// D-Bus interface for control methods.
pub const INTERFACE_NAME: &str = "io.github.idlescreen.Idle";
