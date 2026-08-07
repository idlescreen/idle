// SPDX-License-Identifier: MIT

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]

pub mod config;
pub mod config_parse;
pub mod config_watcher;
pub mod controller;
pub mod daemon;
pub mod dbus_server;
pub mod failsafe;
pub mod inhibit;
pub mod ipc_runner;
pub mod lock_monitor;
pub mod locks;
pub mod ooda;
pub mod presentation;
