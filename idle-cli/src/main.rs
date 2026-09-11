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
mod doctor;
mod doctor_checks;
mod doctor_env;
mod doctor_fs;
mod doctor_pkg;
mod doctor_rules;
mod doctor_service;
mod doctor_sys;
mod interactive;
mod interactive_io;
mod pkg_query;
mod self_update;
mod self_update_backend;

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
    const SHORTS: &[char] = &['h', 'V', 'f', 'j', 't', 'l'];
    for a in &args {
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
    let cmd = cli.cmd;

    // Standalone commands dispatch without a daemon connection.
    if !cmd.needs_daemon() {
        return match cmd {
            Cmd::Version { long } => {
                print_version(long);
                Ok(())
            }
            Cmd::About => {
                print_version(true);
                Ok(())
            }
            Cmd::Doctor { fix, json } => doctor::run_doctor(fix, json),
            Cmd::Clean => clean::handle_clean(),
            Cmd::Completion { shell } => completion::handle_completion(shell),
            Cmd::BugReport => bug_report::handle_bug_report(),
            Cmd::SelfUpdate => self_update::handle_self_update(),
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
        Cmd::Config { op } => config::handle_config(&client, op),
        Cmd::Interactive => interactive::run_interactive(&client),
        Cmd::Enable => client
            .enable()
            .context("enabling idle screensaver")
            .inspect(|_| println!("Idle screensaver enabled.")),
        Cmd::Disable => client
            .disable()
            .context("disabling idle screensaver")
            .inspect(|_| println!("Idle screensaver disabled.")),
        Cmd::Timeout { minutes } => cmd_timeout(&client, minutes),
        Cmd::Saver { op } => match op {
            None => commands::cmd_saver_show(&client),
            Some(cli::SaverOp::Set { name }) => commands::cmd_saver_set(&client, &name),
            Some(cli::SaverOp::List { json }) => cmd_list(&client, json),
        },
        Cmd::List { json } => cmd_list(&client, json),
        Cmd::Inhibitors { json } => cmd_inhibitors(&client, json),
        Cmd::Preview { name, timeout } => cmd_preview(&client, &name, timeout),
        Cmd::Stop => client
            .stop_preview()
            .context("stopping preview or idle presentation")
            .inspect(|_| println!("Presentation stopped.")),
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
