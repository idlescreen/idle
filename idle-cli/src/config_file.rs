// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! File-level config ops: `config path`, `config edit`, `config reset`.
//! These act on config.yaml itself and do not need a running daemon.

use std::io::IsTerminal;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::cli::ConfigOp;
use crate::doctor_fs::get_config_path;

pub fn handle_config_local(op: Option<ConfigOp>, json: bool) -> Result<()> {
    match op {
        Some(ConfigOp::Path) => cmd_path(json),
        Some(ConfigOp::Edit) => cmd_edit(),
        Some(ConfigOp::Reset { yes }) => cmd_reset(yes),
        _ => bail!("internal error: {op:?} needs a daemon connection"),
    }
}

fn cmd_path(json: bool) -> Result<()> {
    let path = get_config_path().context("could not resolve config directory")?;
    if json {
        let esc = path
            .display()
            .to_string()
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        println!("{{\"path\":\"{esc}\"}}");
    } else {
        println!("{}", path.display());
    }
    Ok(())
}

fn cmd_edit() -> Result<()> {
    let path = get_config_path().context("could not resolve config directory")?;
    let editor = std::env::var("VISUAL")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("EDITOR").ok().filter(|s| !s.is_empty()))
        .context("no editor configured — set $VISUAL or $EDITOR")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let status = Command::new(&editor)
        .arg(&path)
        .status()
        .with_context(|| format!("failed to run editor '{editor}'"))?;
    if !status.success() {
        bail!("editor '{editor}' exited {status}");
    }
    Ok(())
}

fn cmd_reset(yes: bool) -> Result<()> {
    let Some(path) = get_config_path().filter(|p| p.is_file()) else {
        println!("No configuration file exists; defaults are already in effect.");
        return Ok(());
    };
    if !yes {
        if !std::io::stdin().is_terminal() {
            bail!(
                "refusing to remove {} without confirmation — pass --yes",
                path.display()
            );
        }
        eprint!("Remove {} and restore defaults? [y/N] ", path.display());
        let mut answer = String::new();
        std::io::stdin()
            .read_line(&mut answer)
            .context("reading confirmation")?;
        if !matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
            println!("Aborted; configuration unchanged.");
            return Ok(());
        }
    }
    std::fs::remove_file(&path).with_context(|| format!("failed to remove {}", path.display()))?;
    println!("Removed {} — defaults restored.", path.display());
    if idle_dbus::daemon_available() {
        println!("Note: the running daemon keeps its live values until `idlescreen restart`.");
    }
    Ok(())
}
