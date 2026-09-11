// SPDX-License-Identifier: MIT

//! Shell completion generation — bash/zsh/fish/powershell/elvish are
//! generated from the clap definition (can never drift); nushell ships a
//! maintained static script.

use anyhow::Result;
use clap::CommandFactory;
use clap_complete::generate;

use crate::cli::{Cli, CompletionShell};

pub fn handle_completion(shell: CompletionShell) -> Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    match shell {
        CompletionShell::Bash => generate(
            clap_complete::shells::Bash,
            &mut cmd,
            &name,
            &mut std::io::stdout(),
        ),
        CompletionShell::Zsh => generate(
            clap_complete::shells::Zsh,
            &mut cmd,
            &name,
            &mut std::io::stdout(),
        ),
        CompletionShell::Fish => generate(
            clap_complete::shells::Fish,
            &mut cmd,
            &name,
            &mut std::io::stdout(),
        ),
        CompletionShell::PowerShell => generate(
            clap_complete::shells::PowerShell,
            &mut cmd,
            &name,
            &mut std::io::stdout(),
        ),
        CompletionShell::Elvish => generate(
            clap_complete::shells::Elvish,
            &mut cmd,
            &name,
            &mut std::io::stdout(),
        ),
        CompletionShell::Nushell => {
            // clap_complete has no nu generator; ship the maintained script.
            print!("{}", include_str!("completions/idle.nu"));
        }
    }
    Ok(())
}
