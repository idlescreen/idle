// SPDX-License-Identifier: MIT

#![cfg(test)]

//! clap surface falsification: every subcommand and alias must parse, and
//! hostile input must fail with a usage error — never a panic or a silent
//! misdispatch.

use crate::cli::{Cli, Cmd, ConfigOp, SaverOp};
use clap::Parser;

fn parse(args: &[&str]) -> Result<Cmd, clap::Error> {
    Cli::try_parse_from(std::iter::once("idlescreen").chain(args.iter().copied())).map(|c| c.cmd)
}

// ---- alias round-trips -------------------------------------------------

#[test]
fn all_aliases_resolve_to_canonical_variants() {
    for (alias, canonical) in [
        ("st", "status"),
        ("cfg list", "config list"),
        ("on", "enable"),
        ("off", "disable"),
        ("t", "timeout"),
        ("ls", "list"),
        ("p storm", "preview storm"),
        ("sv", "saver"),
        ("sv set storm", "saver set storm"),
        ("sv list", "saver list"),
        ("inhib", "inhibitors"),
        ("x", "stop"),
        ("fps", "fps-overlay"),
        ("scale", "render-scale"),
        ("i", "interactive"),
        ("doc", "doctor"),
        ("cl", "clean"),
        ("comp bash", "completion bash"),
        ("bug", "bug-report"),
        ("ui", "tui"),
        ("v", "version"),
        ("info", "about"),
        ("update", "self-update"),
        ("upgrade", "self-update"),
        ("hold true", "inhibit true"),
        ("rs", "restart"),
        ("log", "logs"),
        ("activate", "start"),
    ] {
        let a = parse(&alias.split(' ').collect::<Vec<_>>());
        let c = parse(&canonical.split(' ').collect::<Vec<_>>());
        assert!(
            a.is_ok() && c.is_ok(),
            "alias '{alias}' or '{canonical}' failed to parse"
        );
        assert_eq!(
            std::mem::discriminant(&a.unwrap()),
            std::mem::discriminant(&c.unwrap()),
            "alias '{alias}' must equal '{canonical}'"
        );
    }
}

#[test]
fn subcommand_shapes_parse() {
    assert!(matches!(
        parse(&["status", "--json"]),
        Ok(Cmd::Status { json: true })
    ));
    assert!(matches!(
        parse(&["config", "get", "timeout"]),
        Ok(Cmd::Config {
            op: Some(ConfigOp::Get { .. }),
            ..
        })
    ));
    assert!(matches!(
        parse(&["config", "set", "timeout", "5"]),
        Ok(Cmd::Config {
            op: Some(ConfigOp::Set { .. }),
            ..
        })
    ));
    assert!(matches!(
        parse(&["saver", "set", "storm"]),
        Ok(Cmd::Saver {
            op: Some(SaverOp::Set { .. }),
            ..
        })
    ));
    assert!(matches!(
        parse(&["preview", "storm", "-t", "30"]),
        Ok(Cmd::Preview {
            timeout: Some(30),
            ..
        })
    ));
    assert!(matches!(
        parse(&["doctor", "--fix", "-j"]),
        Ok(Cmd::Doctor {
            fix: true,
            json: true
        })
    ));
    assert!(matches!(
        parse(&["timeout"]),
        Ok(Cmd::Timeout {
            minutes: None,
            json: false
        })
    ));
    assert!(matches!(
        parse(&["timeout", "30"]),
        Ok(Cmd::Timeout {
            minutes: Some(30),
            ..
        })
    ));
    assert!(matches!(parse(&["restart"]), Ok(Cmd::Restart)));
    assert!(matches!(
        parse(&["logs", "-f", "-n", "50"]),
        Ok(Cmd::Logs {
            follow: true,
            lines: 50
        })
    ));
    assert!(matches!(
        parse(&["config", "path"]),
        Ok(Cmd::Config {
            op: Some(ConfigOp::Path),
            ..
        })
    ));
    assert!(matches!(
        parse(&["config", "reset", "--yes"]),
        Ok(Cmd::Config {
            op: Some(ConfigOp::Reset { yes: true }),
            ..
        })
    ));
    assert!(matches!(
        parse(&["self-update", "--check"]),
        Ok(Cmd::SelfUpdate { check: true })
    ));
    assert!(matches!(
        parse(&["clean", "-n"]),
        Ok(Cmd::Clean { dry_run: true })
    ));
}

// ---- hostile input -----------------------------------------------------

#[test]
fn unknown_subcommand_errors_not_panics() {
    assert!(parse(&["statuz"]).is_err());
    assert!(parse(&[""]).is_err());
    assert!(parse(&["--bogus-flag"]).is_err());
}

#[test]
fn missing_required_positionals_error() {
    assert!(parse(&["preview"]).is_err()); // name required
    assert!(parse(&["config", "get"]).is_err()); // key required
    assert!(parse(&["config", "set", "k"]).is_err()); // value required
    assert!(parse(&["completion"]).is_err()); // shell required
}

#[test]
fn invalid_flag_values_error() {
    assert!(parse(&["preview", "s", "-t", "abc"]).is_err());
    assert!(parse(&["timeout", "not-a-number"]).is_err());
    assert!(parse(&["completion", "tcsh"]).is_err());
    assert!(parse(&["fps-overlay", "maybe"]).is_err());
}

#[test]
fn extra_args_error() {
    assert!(parse(&["status", "extra"]).is_err());
    assert!(parse(&["enable", "extra"]).is_err());
    assert!(parse(&["timeout", "5", "extra"]).is_err());
}

#[test]
fn oversized_args_dont_panic() {
    let huge = "x".repeat(1 << 20);
    // Parser must return an error or a value — never panic or OOM-loop.
    let _ = parse(&["saver", "set", &huge]);
    let _ = parse(&[&huge]);
    let _ = parse(&["preview", &huge]);
}

#[test]
fn short_long_flag_parity() {
    for (short, long) in [
        ("status -j", "status --json"),
        ("list -j", "list --json"),
        ("inhibitors -j", "inhibitors --json"),
        ("saver list -j", "saver list --json"),
        ("doctor -j", "doctor --json"),
        ("doctor -f", "doctor --fix"),
        ("preview s -t 5", "preview s --timeout 5"),
        ("version -l", "version --long"),
        ("clean -n", "clean --dry-run"),
        ("self-update -c", "self-update --check"),
        ("logs -n 5", "logs --lines 5"),
        ("inhibit -r x true", "inhibit --reason x true"),
        ("config reset -y", "config reset --yes"),
    ] {
        let a = parse(&short.split(' ').collect::<Vec<_>>());
        let b = parse(&long.split(' ').collect::<Vec<_>>());
        assert!(
            a.is_ok() && b.is_ok(),
            "{short:?} / {long:?} must both parse"
        );
        assert_eq!(
            std::mem::discriminant(&a.unwrap()),
            std::mem::discriminant(&b.unwrap())
        );
    }
}

#[test]
fn flag_only_inputs_error() {
    assert!(parse(&["--"]).is_err());
    assert!(parse(&["-"]).is_err());
}

#[test]
fn tui_passthrough_keeps_hyphen_args() {
    let parsed = parse(&["tui", "--theme", "dark"]);
    assert!(
        matches!(parsed, Ok(Cmd::Tui { .. })),
        "tui passthrough broken: {parsed:?}"
    );
    if let Ok(Cmd::Tui { args }) = parsed {
        assert_eq!(args, ["--theme", "dark"]);
    }
}

#[test]
fn help_and_version_paths() {
    use clap::error::ErrorKind::*;
    // clap reports display paths as typed errors — run() maps them to Ok.
    assert_eq!(parse(&["--help"]).unwrap_err().kind(), DisplayHelp);
    assert_eq!(parse(&["help"]).unwrap_err().kind(), DisplayHelp);
    assert_eq!(parse(&["help", "saver"]).unwrap_err().kind(), DisplayHelp);
    assert_eq!(parse(&["--version"]).unwrap_err().kind(), DisplayVersion);
}

#[cfg(test)]
#[path = "cli_parse_tests2.rs"]
mod extra;
