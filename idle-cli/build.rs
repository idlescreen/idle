// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 IdleScreen

//! Generate shell completions into `target/<profile>/completions/` so the
//! deb/rpm packaging assets can ship them. The clap definition is the same
//! `cli.rs` the binary compiles — the generated files cannot drift.

use std::fs;
use std::path::Path;

// Only `Cli::command()` is used here; the rest of the surface belongs to
// the binary.
#[allow(dead_code)]
#[path = "src/cli.rs"]
mod cli;

fn emit<G: clap_complete::Generator>(shell: G, cmd: &mut clap::Command, dir: &Path, file: &str) {
    let path = dir.join(file);
    let mut f = fs::File::create(&path)
        .unwrap_or_else(|e| fail(&format!("create {}: {e}", path.display())));
    clap_complete::generate(shell, cmd, "idlescreen", &mut f);
}

fn fail(msg: &str) -> ! {
    eprintln!("idle-cli build.rs: {msg}");
    std::process::exit(1)
}

fn main() {
    use clap::CommandFactory;

    // OUT_DIR is target/<profile>/build/<pkg>-<hash>/out — walk up to the
    // profile dir so assets land at target/release/completions/.
    let dir = std::env::var_os("OUT_DIR")
        .and_then(|out| {
            Path::new(&out)
                .ancestors()
                .nth(3)
                .map(|p| p.join("completions"))
        })
        .unwrap_or_else(|| fail("unexpected OUT_DIR layout"));
    fs::create_dir_all(&dir).unwrap_or_else(|e| fail(&format!("create {}: {e}", dir.display())));

    let mut cmd = cli::Cli::command();
    emit(clap_complete::shells::Bash, &mut cmd, &dir, "idlescreen");
    emit(clap_complete::shells::Zsh, &mut cmd, &dir, "_idlescreen");
    emit(
        clap_complete::shells::Fish,
        &mut cmd,
        &dir,
        "idlescreen.fish",
    );
    emit(
        clap_complete::shells::Elvish,
        &mut cmd,
        &dir,
        "idlescreen.elv",
    );
    emit(
        clap_complete::shells::PowerShell,
        &mut cmd,
        &dir,
        "_idlescreen.ps1",
    );
}
