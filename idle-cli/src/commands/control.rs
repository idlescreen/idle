// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Control subcommands: timeout, saver, preview, overlays, inhibitors.

use anyhow::{Context, Result, bail};
use idle_dbus::TranceClient;

fn json_esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn cmd_timeout(client: &TranceClient, minutes: Option<u32>, json: bool) -> Result<()> {
    if let Some(m) = minutes {
        client.set_timeout(m).context("setting idle timeout")?;
    }
    let status = client.get_status().context("querying daemon status")?;
    if json {
        println!("{{\"idle_timeout_mins\":{}}}", status.idle_timeout_mins);
    } else if minutes.is_some() || !crate::quiet() {
        println!("idle timeout: {} min", status.idle_timeout_mins);
    }
    Ok(())
}

pub fn cmd_saver_show(client: &TranceClient, json: bool) -> Result<()> {
    let status = client.get_status().context("querying daemon status")?;
    let shown = if status.active_saver.is_empty() {
        "random"
    } else {
        &status.active_saver
    };
    if json {
        println!("{{\"active_saver\":\"{}\"}}", json_esc(shown));
    } else {
        println!("active saver: {shown}");
    }
    Ok(())
}

pub fn cmd_saver_set(client: &TranceClient, name: &str, json: bool) -> Result<()> {
    let dbus_name = if name.is_empty()
        || name.eq_ignore_ascii_case("random")
        || name.eq_ignore_ascii_case("none")
        || name.eq_ignore_ascii_case("shuffle")
    {
        ""
    } else {
        name
    };
    client
        .set_saver(dbus_name)
        .context("setting active saver via d-bus")?;
    let shown = if dbus_name.is_empty() {
        "random"
    } else {
        dbus_name
    };
    if json {
        println!("{{\"active_saver\":\"{}\"}}", json_esc(shown));
    } else if !crate::quiet() {
        println!("active saver: {shown}");
    }
    Ok(())
}

/// systemd-inhibit pattern: hold an idle inhibitor while `argv` runs, then
/// release it. The daemon also drops the inhibitor if this process dies and
/// the D-Bus connection closes.
pub fn cmd_inhibit(client: &TranceClient, reason: Option<&str>, argv: &[String]) -> Result<()> {
    let reason = reason.unwrap_or("inhibited by idlescreen CLI");
    let cookie = client
        .inhibit("idlescreen-cli", reason)
        .context("creating idle inhibitor via d-bus")?;
    if !crate::quiet() {
        println!("Idle inhibited (cookie {cookie}) while: {}", argv.join(" "));
    }
    let spawned = std::process::Command::new(&argv[0])
        .args(&argv[1..])
        .status();
    // Always release, even when the child could not be spawned.
    let release = client.un_inhibit(cookie);
    let status = spawned.with_context(|| format!("failed to run '{}'", argv[0]))?;
    release.context("releasing idle inhibitor")?;
    if status.success() {
        Ok(())
    } else {
        std::process::exit(status.code().unwrap_or(1));
    }
}

pub fn cmd_list(client: &TranceClient, json: bool) -> Result<()> {
    let savers = client
        .list_savers()
        .context("listing installed savers via d-bus")?;
    if json {
        let items: Vec<String> = savers
            .iter()
            .map(|s| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")))
            .collect();
        println!("[{}]", items.join(","));
    } else {
        for saver in savers {
            println!("{saver}");
        }
    }
    Ok(())
}

pub fn cmd_inhibitors(client: &TranceClient, json: bool) -> Result<()> {
    use super::inhibitors_fmt::format_inhibitors_report;
    let inhibitors = client
        .list_inhibitors()
        .context("listing active inhibitors via d-bus")?;
    let status = client.get_status().context("querying daemon status")?;
    if json {
        fn esc(s: &str) -> String {
            s.replace('\\', "\\\\").replace('"', "\\\"")
        }
        let items: Vec<String> = inhibitors
            .iter()
            .map(|(pid, who, why)| {
                format!(
                    "{{\"pid\":{},\"who\":\"{}\",\"why\":\"{}\"}}",
                    pid,
                    esc(who),
                    esc(why)
                )
            })
            .collect();
        println!(
            "{{\"inhibited\":{},\"inhibitors\":[{}]}}",
            status.inhibited,
            items.join(",")
        );
    } else {
        print!(
            "{}",
            format_inhibitors_report(status.inhibited, &inhibitors)
        );
    }
    Ok(())
}

pub fn cmd_preview(client: &TranceClient, name: &str, timeout: Option<u64>) -> Result<()> {
    client.preview(name).context("starting preview via d-bus")?;
    if timeout.is_none() && !crate::quiet() {
        println!("Previewing '{name}' — run `idlescreen stop` or provide input to end.");
    }
    if let Some(secs) = timeout {
        if !crate::quiet() {
            println!("Preview started. Auto-stopping in {secs} seconds...");
        }
        std::thread::sleep(std::time::Duration::from_secs(secs));
        let _ = client.stop_preview();
        if !crate::quiet() {
            println!("Preview stopped.");
        }
    }
    Ok(())
}

pub fn cmd_fps_overlay(client: &TranceClient, state: Option<&str>) -> Result<()> {
    match state {
        None | Some("status") => {
            let status = client.get_status().context("querying daemon status")?;
            println!(
                "fps overlay: {}",
                if status.show_fps_overlay { "on" } else { "off" }
            );
            Ok(())
        }
        Some("on") => client
            .set_show_fps_overlay(true)
            .context("enabling fps overlay via d-bus")
            .inspect(|_| {
                if !crate::quiet() {
                    println!("fps overlay: on")
                }
            }),
        Some("off") => client
            .set_show_fps_overlay(false)
            .context("disabling fps overlay via d-bus")
            .inspect(|_| {
                if !crate::quiet() {
                    println!("fps overlay: off")
                }
            }),
        Some(value) => Err(anyhow::anyhow!(
            "unknown fps-overlay subcommand: {value} (use on, off, status)"
        )),
    }
}

fn parse_render_scale_value(value: &str) -> Result<f32> {
    let scale = value
        .parse::<f32>()
        .context("render-scale requires a number between 0.25 and 1.0")?;
    if !(0.25..=1.0).contains(&scale) {
        bail!("render-scale must be between 0.25 and 1.0");
    }
    Ok(scale)
}

pub fn cmd_render_scale(client: &TranceClient, value: Option<&str>) -> Result<()> {
    match value {
        None | Some("status") => {
            let status = client.get_status().context("querying daemon status")?;
            println!(
                "render scale: {}",
                if status.render_scale.is_empty() {
                    "default"
                } else {
                    &status.render_scale
                }
            );
            Ok(())
        }
        Some("default") => client
            .set_render_scale(0.0)
            .context("resetting render scale via d-bus")
            .inspect(|_| {
                if !crate::quiet() {
                    println!("render scale: default")
                }
            }),
        Some(value) => {
            let scale = parse_render_scale_value(value)?;
            client
                .set_render_scale(scale)
                .context("setting render scale via d-bus")
                .inspect(|_| {
                    if !crate::quiet() {
                        println!("render scale: {scale}")
                    }
                })
        }
    }
}
