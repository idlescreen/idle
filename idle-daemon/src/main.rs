// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

#[allow(clippy::wildcard_imports)]
use idle_daemon::*;

fn main() -> anyhow::Result<()> {
    use anyhow::Context;
    use tracing_subscriber::prelude::*;

    // Mark multi-monitor span presentation for plugins/layout helpers.
    idle_api::set_env("IDLE_SPAN_MODE", "1");

    // Initialize tracing with journald or stderr fallback
    if std::env::var("JOURNAL_STREAM").is_ok() {
        let filter = tracing_subscriber::EnvFilter::builder()
            .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
            .from_env_lossy();
        let registry = tracing_subscriber::registry()
            .with(filter)
            .with(tracing_journald::layer().context("initializing journald tracing layer")?);
        tracing::subscriber::set_global_default(registry)
            .context("installing journald tracing subscriber")?;
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .init();
    }

    // Register visual theme and system query callbacks for dynamically loaded screensaver plugins
    let _ = idle_api::SYSTEM_INFO_CALLBACK.set(idle_runner::toolkit::sys_info::get_system_info);
    let _ = idle_api::PALETTE_CALLBACK.set(idle_runner::toolkit::sys_info::query_current_palette);
    let _ = idle_api::MONITOR_BOUNDS_CALLBACK
        .set(idle_runner::toolkit::sys_info::get_primary_monitor_bounds);
    let _ = idle_api::IS_SECONDARY_MONITOR_CALLBACK
        .set(idle_runner::toolkit::sys_info::is_secondary_monitor);

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        let sub = &args[1];
        match sub.as_str() {
            "run-plugin" => run_plugin_subcmd(&args),
            "run-ipc-runner" => run_ipc_runner_subcmd(&args),
            "daemon" | "--daemon" => daemon::run_daemon(),
            "--help" | "-h" => {
                println!(
                    "idle-daemon — background idle monitoring service for IdleScreen

usage:
  idle-daemon                     run the background idle daemon (default)
  idle-daemon daemon | --daemon   run the background idle daemon
  idle-daemon run-plugin <saver>  run a trusted screensaver plugin fullscreen
  idle-daemon --help | -h         show this help message"
                );
                Ok(())
            }
            other => anyhow::bail!("unknown argument: {}\ntry --help", other),
        }
    } else {
        // Run the daemon by default
        daemon::run_daemon()
    }
}

fn run_plugin_subcmd(args: &[String]) -> anyhow::Result<()> {
    anyhow::ensure!(
        args.len() >= 3,
        "missing saver name.\nusage: idle-daemon run-plugin <saver>"
    );
    let name = &args[2];
    anyhow::ensure!(
        !name.contains('/') && !name.contains('\\'),
        "saver name must not be a path"
    );
    let path = idle_runner::launcher::resolve_saver_binary(
        name,
        &idle_runner::launcher::LaunchMode::Preview,
    )?;
    // run_plugin_fullscreen replaces this process image with the plugin;
    // exit the host with the plugin's status code on return.
    let code = idle_runner::idle_runner::run_plugin_fullscreen(path.to_string_lossy().as_ref())
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    std::process::exit(code as i32);
}

fn run_ipc_runner_subcmd(args: &[String]) -> anyhow::Result<()> {
    // IPC children must never inherit ambient sandbox/dev escapes from the session.
    idle_runner::sandbox::clear_sandbox_escape_env();
    anyhow::ensure!(
        args.len() >= 9,
        "missing arguments.\nusage: idle-daemon run-ipc-runner <saver> <socket_path> <shm_name> <cols> <rows> <gpu_enabled> <render_scale>"
    );
    let saver = &args[2];
    let socket_path = &args[3];
    let shm_name = &args[4];
    let cols: usize = args[5].parse().unwrap_or(80);
    let rows: usize = args[6].parse().unwrap_or(24);
    let gpu_enabled: bool = args[7].parse().unwrap_or(false);
    let render_scale: Option<f32> = if args[8] == "none" {
        None
    } else {
        args[8].parse().ok()
    };
    ipc_runner::run_ipc_runner(
        saver,
        socket_path,
        shm_name,
        cols,
        rows,
        gpu_enabled,
        render_scale,
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}
