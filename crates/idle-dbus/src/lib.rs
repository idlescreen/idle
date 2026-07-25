// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! D-Bus API for the IdleScreen screensaver daemon.
//!
//! ## Primary (current) well-known names
//!
//! - Service: [`SERVICE_NAME`] (`io.github.idlescreen.Idle`)
//! - Path: [`OBJECT_PATH`] (`/io/github/idlescreen/Idle`)
//!
//! ## Legacy (still dual-registered for upgrades)
//!
//! - Service: [`SERVICE_NAME_LEGACY`] (`io.github.ubermetroid.trance`)
//! - Path: [`OBJECT_PATH_LEGACY`] (`/io/github/crateria/trance`)
//!
//! Interface name remains [`INTERFACE_NAME`] on both endpoints so one method set
//! works during migration. Clients try primary first, then legacy.

pub mod client;
pub mod status;

pub use client::{TranceClient, daemon_available};
pub use status::DaemonStatus;

/// Current well-known bus name (prefer this).
pub const SERVICE_NAME: &str = "io.github.idlescreen.Idle";
/// Current object path (prefer this).
pub const OBJECT_PATH: &str = "/io/github/idlescreen/Idle";

/// Historical bus name — dual-claimed by the daemon.
pub const SERVICE_NAME_LEGACY: &str = "io.github.ubermetroid.trance";
/// Historical object path — dual-exported by the daemon.
pub const OBJECT_PATH_LEGACY: &str = "/io/github/crateria/trance";

/// D-Bus interface for control methods (same on primary and legacy endpoints).
pub const INTERFACE_NAME: &str = "io.github.ubermetroid.trance";
