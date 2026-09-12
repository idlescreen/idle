// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use cli::Cmd;
use commands::{
    cmd_fps_overlay, cmd_inhibitors, cmd_list, cmd_preview, cmd_render_scale, cmd_status,
    cmd_timeout, print_version,
};
use idle_dbus::{TranceClient, daemon_available};

mod bug_report;
mod clean;
mod cli;
mod commands;
mod completion;
mod config;
mod config_file;
mod doctor;
mod doctor_checks;
mod doctor_env;
mod doctor_fs;
mod doctor_pkg;
mod doctor_pkg_fmt;
mod doctor_rules;
mod doctor_service;
mod doctor_sys;
mod interactive;
mod interactive_io;
mod pkg_query;
mod self_update;
mod self_update_backend;
mod self_update_check;
mod service;

#[cfg(test)]
mod cli_parse_tests;
#[cfg(test)]
mod tests;

fn main() -> ExitCode {
    // Restore default SIGPIPE: Rust ignores it, so `idlescreen X | head`
    // panics on EPIPE instead of exiting quietly like every other CLI.
    // SAFETY: single-threaded startup, before any output is produced.
    unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL) };
    init_tracing();
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if error.downcast_ref::<UsageError>().is_some() {
                // clap already printed the usage text.
                return ExitCode::from(2);
            }
            tracing::error!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

/// Marker: the command line itself was invalid (clap usage error → exit 2).
#[derive(Debug)]
struct UsageError;

impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("usage error")
    }
}
impl std::error::Error for UsageError {}

/// `-q/--quiet`: suppress confirmations, not primary output or errors.
static QUIET: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(crate) fn quiet() -> bool {
    QUIET.load(std::sync::atomic::Ordering::Relaxed)
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    #[cfg(feature = "journald")]
    {
        use tracing_subscriber::prelude::*;
        if let Ok(layer) = tracing_journald::layer() {
            let _ = tracing_subscriber::registry()
                .with(env_filter.clone())
                .with(layer)
                .try_init();
            return;
        }
    }

    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .try_init();
}

#[tracing::instrument(skip_all)]
fn run() -> Result<()> {
    run_from(std::env::args().skip(1).collect())
}

/// Parse + dispatch. Takes bare args (no argv[0]); test-injectable. clap's
/// help/version "errors" print and exit by default — here they map to Ok.
pub(crate) fn run_from(args: Vec<String>) -> Result<()> {
    use clap::Parser;
    use clap::error::ErrorKind;

    // Reject single-dash long options (`-help`, `-version`) — clap would
    // silently split them into short flags; users must write `--help`.
    // Args after a passthrough command (tui/inhibit) belong to the child.
    const SHORTS: &[char] = &['h', 'V', 'f', 'j', 't', 'l', 'q', 'n', 'r', 'c'];
    let stop = args
        .iter()
        .position(|a| matches!(a.as_str(), "tui" | "ui" | "inhibit" | "hold"))
        .unwrap_or(args.len());
    for a in &args[..stop] {
        if let Some(rest) = a.strip_prefix('-')
            && !rest.is_empty()
            && !rest.starts_with('-')
            && !rest.chars().next().is_some_and(|c| c.is_ascii_digit())
            && !rest.chars().all(|c| SHORTS.contains(&c))
        {
            eprintln!("error: invalid option '{a}' — long options use two dashes (try --{rest})");
            return Err(UsageError.into());
        }
    }

    let argv = std::iter::once("idlescreen".to_string()).chain(args);
    let cli = match cli::Cli::try_parse_from(argv) {
        Ok(c) => c,
        Err(e) => match e.kind() {
            ErrorKind::DisplayHelp
            | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            | ErrorKind::DisplayVersion => {
                let _ = e.print();
                return Ok(());
            }
            _ => {
                let _ = e.print();
                return Err(UsageError.into());
            }
        },
    };
    QUIET.store(cli.quiet, std::sync::atomic::Ordering::Relaxed);
    let cmd = cli.cmd;

    // Standalone commands dispatch without a daemon connection.
    if !cmd.needs_daemon() {
        return match cmd {
            Cmd::Version { long, json } => {
                print_version(long, json);
                Ok(())
            }
            Cmd::About => {
                print_version(true, false);
                Ok(())
            }
            Cmd::Doctor { fix, json } => doctor::run_doctor(fix, json),
            Cmd::Clean { dry_run } => clean::handle_clean(dry_run),
            Cmd::Completion { shell } => completion::handle_completion(shell),
            Cmd::BugReport => bug_report::handle_bug_report(),
            Cmd::SelfUpdate { check } => self_update::handle_self_update(check),
            Cmd::Restart => service::handle_restart(),
            Cmd::Logs { follow, lines } => service::handle_logs(follow, lines),
            Cmd::Config { op, json } => config_file::handle_config_local(op, json),
            Cmd::Tui { args } => {
                let mut c = std::process::Command::new("idle-tui");
                c.args(&args);
                let status = c.status().context("failed to execute idle-tui")?;
                if status.success() {
                    Ok(())
                } else {
                    std::process::exit(status.code().unwrap_or(1));
                }
            }
            other => bail!("internal error: {other:?} should not need daemon"),
        };
    }

    if !daemon_available() {
        bail!("idle-daemon is not running; start it with: systemctl --user start idle-daemon");
    }
    let client = TranceClient::connect().context("failed to connect to daemon")?;

    match cmd {
        Cmd::Status { json } => cmd_status(&client, json),
        Cmd::Config { op, json } => config::handle_config(&client, op, json),
        Cmd::Interactive => interactive::run_interactive(&client),
        Cmd::Enable => client
            .enable()
            .context("enabling idle screensaver")
            .inspect(|_| {
                if !quiet() {
                    println!("Idle screensaver enabled.")
                }
            }),
        Cmd::Disable => client
            .disable()
            .context("disabling idle screensaver")
            .inspect(|_| {
                if !quiet() {
                    println!("Idle screensaver disabled.")
                }
            }),
        Cmd::Timeout { minutes, json } => cmd_timeout(&client, minutes, json),
        Cmd::Saver { op, json } => match op {
            None => commands::cmd_saver_show(&client, json),
            Some(cli::SaverOp::Set { name }) => commands::cmd_saver_set(&client, &name, json),
            Some(cli::SaverOp::List) => cmd_list(&client, json),
        },
        Cmd::Inhibit { reason, command } => {
            commands::cmd_inhibit(&client, reason.as_deref(), &command)
        }
        Cmd::List { json } => cmd_list(&client, json),
        Cmd::Inhibitors { json } => cmd_inhibitors(&client, json),
        Cmd::Start => client
            .activate()
            .context("activating screensaver via d-bus")
            .inspect(|_| {
                if !quiet() {
                    println!("Screensaver activated — `idlescreen stop` ends it.")
                }
            }),
        Cmd::Preview { name, timeout } => cmd_preview(&client, &name, timeout),
        Cmd::Stop => client
            .stop_preview()
            .context("stopping preview or idle presentation")
            .inspect(|_| {
                if !quiet() {
                    println!("Presentation stopped.")
                }
            }),
        Cmd::FpsOverlay { state } => {
            let s = state.map(|s| match s {
                cli::OverlayState::On => "on",
                cli::OverlayState::Off => "off",
                cli::OverlayState::Status => "status",
            });
            cmd_fps_overlay(&client, s)
        }
        Cmd::RenderScale { value } => cmd_render_scale(&client, value.as_deref()),
        other => bail!("internal error: {other:?} needs daemon but was dispatched early"),
    }
}
