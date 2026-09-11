// SPDX-License-Identifier: MIT

use anyhow::{Context, Result, anyhow, bail};
use idle_dbus::TranceClient;

use crate::cli::ConfigOp;

pub fn handle_config(client: &TranceClient, op: Option<ConfigOp>, json: bool) -> Result<()> {
    match op.unwrap_or(ConfigOp::List) {
        ConfigOp::List => cmd_config_list(client, json),
        ConfigOp::Get { key } => cmd_config_get(client, &key, json),
        ConfigOp::Set { key, value } => cmd_config_set(client, &key, &value, json),
        _ => bail!("internal error: file-level config op reached daemon path"),
    }
}

fn json_esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// JSON value with the right type: booleans and numbers stay bare.
fn json_val(key: &str, v: &str) -> String {
    match key {
        "idle_enabled" | "enabled" | "show_fps_overlay" | "fps" | "idle_timeout_mins"
        | "timeout" => v.to_string(),
        _ => format!("\"{}\"", json_esc(v)),
    }
}

fn cmd_config_list(client: &TranceClient, json: bool) -> Result<()> {
    let status = client
        .get_status()
        .context("querying daemon status via d-bus")?;
    if json {
        let saver = if status.active_saver.is_empty() {
            "random"
        } else {
            &status.active_saver
        };
        let scale = if status.render_scale.is_empty() {
            "default"
        } else {
            &status.render_scale
        };
        println!(
            "{{\"idle_enabled\":{},\"idle_timeout_mins\":{},\"active_saver\":\"{}\",\"show_fps_overlay\":{},\"render_scale\":\"{}\"}}",
            status.idle_enabled,
            status.idle_timeout_mins,
            json_esc(saver),
            status.show_fps_overlay,
            json_esc(scale)
        );
        return Ok(());
    }
    println!("idle_enabled:      {}", status.idle_enabled);
    println!("idle_timeout_mins: {}", status.idle_timeout_mins);
    println!(
        "active_saver:      {}",
        if status.active_saver.is_empty() {
            "random"
        } else {
            &status.active_saver
        }
    );
    println!("show_fps_overlay:  {}", status.show_fps_overlay);
    println!(
        "render_scale:      {}",
        if status.render_scale.is_empty() {
            "default"
        } else {
            &status.render_scale
        }
    );
    Ok(())
}

fn config_value(status: &idle_dbus::DaemonStatus, key: &str) -> Result<String> {
    match key {
        "idle_enabled" | "enabled" => Ok(status.idle_enabled.to_string()),
        "idle_timeout_mins" | "timeout" => Ok(status.idle_timeout_mins.to_string()),
        "active_saver" | "saver" => Ok(if status.active_saver.is_empty() {
            "random".to_string()
        } else {
            status.active_saver.clone()
        }),
        "gpu_enabled" | "gpu" => Ok("false".to_string()),
        "show_fps_overlay" | "fps" => Ok(status.show_fps_overlay.to_string()),
        "render_scale" | "scale" => Ok(if status.render_scale.is_empty() {
            "default".to_string()
        } else {
            status.render_scale.clone()
        }),
        _ => Err(anyhow!("unknown configuration key: {key}")),
    }
}

fn cmd_config_get(client: &TranceClient, key: &str, json: bool) -> Result<()> {
    let status = client
        .get_status()
        .context("querying daemon status via d-bus")?;
    let v = config_value(&status, key)?;
    if key == "gpu_enabled" || key == "gpu" {
        println!("false (removed — GPU upscaler deleted)");
    } else if json {
        println!("{{\"{}\":{}}}", json_esc(key), json_val(key, &v));
    } else {
        println!("{v}");
    }
    Ok(())
}

fn cmd_config_set(client: &TranceClient, key: &str, val: &str, json: bool) -> Result<()> {
    match key {
        "idle_enabled" | "enabled" => set_idle_enabled(client, val)?,
        "idle_timeout_mins" | "timeout" => set_idle_timeout(client, val)?,
        "active_saver" | "saver" => set_active_saver(client, val)?,
        "gpu_enabled" | "gpu" => {
            // GPU upscaler removed (2026); do not claim the set succeeded.
            set_gpu_enabled_deprecated(client, val)?;
            return Ok(());
        }
        "show_fps_overlay" | "fps" => set_fps_overlay(client, val)?,
        "render_scale" | "scale" => set_render_scale(client, val)?,
        _ => return Err(anyhow!("unknown configuration key: {key}")),
    }
    if json {
        println!("{{\"{}\":{}}}", json_esc(key), json_val(key, val));
    } else if !crate::quiet() {
        println!("Set config key '{key}' to '{val}' successfully.");
    }
    Ok(())
}

fn set_idle_enabled(client: &TranceClient, val: &str) -> Result<()> {
    let b = val
        .parse::<bool>()
        .map_err(|_| anyhow!("value must be true or false"))?;
    if b { client.enable() } else { client.disable() }
        .context("toggling idle screensaver via d-bus")?;
    Ok(())
}

fn set_idle_timeout(client: &TranceClient, val: &str) -> Result<()> {
    let n = val
        .parse::<u32>()
        .map_err(|_| anyhow!("value must be an integer (1–240)"))?;
    if !(1..=240).contains(&n) {
        bail!("timeout must be between 1 and 240 minutes");
    }
    client
        .set_timeout(n)
        .with_context(|| format!("setting idle timeout to {n} minutes"))?;
    Ok(())
}

fn set_active_saver(client: &TranceClient, val: &str) -> Result<()> {
    let name = match val {
        "none" | "random" | "shuffle" | "" => "",
        s => s,
    };
    client
        .set_saver(name)
        .with_context(|| format!("setting active saver to '{name}'"))?;
    Ok(())
}

/// GPU upscaler was removed; D-Bus `SetGpuEnabled` is a no-op. Be honest in the CLI.
fn set_gpu_enabled_deprecated(client: &TranceClient, val: &str) -> Result<()> {
    let _b = val
        .parse::<bool>()
        .map_err(|_| anyhow!("value must be true or false"))?;
    // Still ping the daemon so old scripts do not hard-fail on AccessDenied paths
    // when the peer is trusted; ignore transport errors after messaging the user.
    let _ = client.set_gpu_enabled(false);
    println!(
        "gpu_enabled is deprecated and always off (GPU frame upscaler removed).\n\
         Cell raster may still use wgpu automatically when available; no action needed."
    );
    Ok(())
}

fn set_fps_overlay(client: &TranceClient, val: &str) -> Result<()> {
    let b = val
        .parse::<bool>()
        .map_err(|_| anyhow!("value must be true or false"))?;
    client
        .set_show_fps_overlay(b)
        .context("toggling fps overlay via d-bus")?;
    Ok(())
}

fn set_render_scale(client: &TranceClient, val: &str) -> Result<()> {
    let scale = if val == "default" {
        0.0f32
    } else {
        val.parse::<f32>()
            .map_err(|_| anyhow!("value must be between 0.25 and 1.0, or 'default'"))?
    };
    if scale != 0.0 && !(0.25..=1.0).contains(&scale) {
        bail!("scale must be between 0.25 and 1.0, or 'default'");
    }
    client
        .set_render_scale(scale)
        .with_context(|| format!("setting render scale to {scale}"))?;
    Ok(())
}
