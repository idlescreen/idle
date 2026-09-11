// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! clap surface for `idlescreen`. All legacy spellings survive as
//! `visible_alias` so they show up in `--help` instead of hiding.

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "idlescreen",
    version,
    about = "Control the IdleScreen daemon",
    long_about = "Control the IdleScreen Wayland screensaver daemon.\n\
                  Run `idlescreen <command> --help` for subcommand details.",
    infer_subcommands = false,
    arg_required_else_help = true
)]
pub struct Cli {
    /// Suppress confirmations and other non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,
    #[command(subcommand)]
    pub cmd: Cmd,
}

/// Commands that do not need a running daemon are marked standalone.
impl Cmd {
    pub fn needs_daemon(&self) -> bool {
        !matches!(
            self,
            Cmd::Version { .. }
                | Cmd::About
                | Cmd::Doctor { .. }
                | Cmd::Clean { .. }
                | Cmd::Completion { .. }
                | Cmd::BugReport
                | Cmd::SelfUpdate { .. }
                | Cmd::Tui { .. }
                | Cmd::Restart
                | Cmd::Logs { .. }
                | Cmd::Config {
                    op: Some(ConfigOp::Path | ConfigOp::Edit | ConfigOp::Reset { .. }),
                    ..
                }
        )
    }
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Show daemon state (running, idle, saver, inhibitors)
    #[command(visible_alias = "st")]
    Status {
        /// Machine-readable JSON output
        #[arg(short, long)]
        json: bool,
    },
    /// View or change daemon configuration
    #[command(visible_alias = "cfg")]
    Config {
        #[command(subcommand)]
        op: Option<ConfigOp>,
        /// Machine-readable JSON output
        #[arg(short, long, global = true)]
        json: bool,
    },
    /// Turn the idle screensaver on
    #[command(visible_alias = "on")]
    Enable,
    /// Turn the idle screensaver off
    #[command(visible_alias = "off")]
    Disable,
    /// Set or show the idle timeout in minutes (1–240)
    #[command(visible_alias = "t")]
    Timeout {
        minutes: Option<u32>,
        /// Machine-readable JSON output
        #[arg(short, long)]
        json: bool,
    },
    /// Show or change the active saver
    #[command(visible_alias = "sv")]
    Saver {
        #[command(subcommand)]
        op: Option<SaverOp>,
        /// Machine-readable JSON output
        #[arg(short, long, global = true)]
        json: bool,
    },
    /// List installed savers
    #[command(visible_alias = "ls")]
    List {
        /// Machine-readable JSON output
        #[arg(short, long)]
        json: bool,
    },
    /// List active idle inhibitors
    #[command(visible_alias = "inhib")]
    Inhibitors {
        /// Machine-readable JSON output
        #[arg(short, long)]
        json: bool,
    },
    /// Run a command while holding an idle inhibitor
    #[command(visible_alias = "hold")]
    Inhibit {
        /// Reason recorded for the inhibitor
        #[arg(short, long)]
        reason: Option<String>,
        /// Command to run while idle is inhibited
        #[arg(
            trailing_var_arg = true,
            allow_hyphen_values = true,
            required = true,
            num_args = 1..
        )]
        command: Vec<String>,
    },
    /// Activate the screensaver now (uses your configured saver)
    #[command(visible_alias = "activate")]
    Start,
    /// Preview a saver fullscreen
    #[command(visible_alias = "p")]
    Preview {
        /// Saver name (see `idlescreen list`)
        name: String,
        /// Auto-stop after N seconds
        #[arg(short, long)]
        timeout: Option<u64>,
    },
    /// Stop the running preview or idle presentation
    #[command(visible_alias = "x")]
    Stop,
    /// FPS overlay: on, off, or status
    #[command(visible_alias = "fps")]
    FpsOverlay { state: Option<OverlayState> },
    /// Render scale: 0.25–1.0, 'default', or status
    #[command(visible_alias = "scale")]
    RenderScale { value: Option<String> },
    /// Interactive console panel
    #[command(visible_alias = "i")]
    Interactive,
    /// Restart the idle-daemon user service
    #[command(visible_alias = "rs")]
    Restart,
    /// Show daemon logs (journalctl --user -u idle-daemon)
    #[command(visible_alias = "log")]
    Logs {
        /// Follow new entries (Ctrl-C to stop)
        #[arg(short, long)]
        follow: bool,
        /// Number of recent entries to show
        #[arg(short = 'n', long, default_value_t = 100)]
        lines: u32,
    },
    /// Run diagnostics (—fix reloads the user service, —json prints a report)
    #[command(visible_alias = "doc")]
    Doctor {
        /// Attempt to repair common issues
        #[arg(short, long)]
        fix: bool,
        /// Machine-readable JSON output
        #[arg(short, long)]
        json: bool,
    },
    /// Remove stale run state and log caches
    #[command(visible_alias = "cl")]
    Clean {
        /// Show what would be removed without deleting anything
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    /// Print a shell completion script to stdout
    #[command(visible_alias = "comp")]
    Completion { shell: CompletionShell },
    /// Print a sanitized diagnostics bundle for bug reports
    #[command(visible_alias = "bug")]
    BugReport,
    /// Check for updates and upgrade installed IdleScreen packages
    #[command(visible_aliases = ["update", "upgrade"])]
    SelfUpdate {
        /// Report update status without upgrading anything
        #[arg(short, long)]
        check: bool,
    },
    /// Launch the full-screen TUI
    #[command(visible_alias = "ui")]
    Tui {
        /// Arguments forwarded to idle-tui
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Print CLI version
    #[command(visible_alias = "v")]
    Version {
        /// Extended info (same as `about`)
        #[arg(short, long)]
        long: bool,
        /// Machine-readable JSON output
        #[arg(short, long)]
        json: bool,
    },
    /// Print version plus project info
    #[command(visible_aliases = ["info"])]
    About,
}

#[derive(Debug, Subcommand)]
pub enum ConfigOp {
    /// Show all configuration keys
    List,
    /// Show one configuration value
    Get { key: String },
    /// Set a configuration value
    Set { key: String, value: String },
    /// Print the configuration file path
    Path,
    /// Open the configuration file in $EDITOR
    Edit,
    /// Restore defaults by removing the configuration file
    Reset {
        /// Skip the confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum SaverOp {
    /// Set the active saver (name, or 'random'/'none' for rotation)
    Set { name: String },
    /// List installed savers
    List,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OverlayState {
    On,
    Off,
    Status,
}

/// Shells with generated completions (nu ships a static script).
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    #[value(alias = "nu")]
    Nushell,
    #[value(name = "powershell", alias = "pwsh", alias = "power-shell")]
    PowerShell,
    Elvish,
}
