// SPDX-License-Identifier: MIT

#![cfg(test)]

//! Parse coverage for the second-wave commands: inhibit passthrough,
//! lifecycle verbs, config file ops, and the global quiet/json flags.

use crate::cli::{Cli, Cmd, ConfigOp};
use clap::Parser;

fn parse(args: &[&str]) -> Result<Cmd, clap::Error> {
    Cli::try_parse_from(std::iter::once("idlescreen").chain(args.iter().copied())).map(|c| c.cmd)
}

#[test]
fn inhibit_forwards_child_argv() {
    let parsed = parse(&["inhibit", "--", "make", "-j4"]);
    assert!(
        matches!(parsed, Ok(Cmd::Inhibit { .. })),
        "inhibit passthrough broken: {parsed:?}"
    );
    if let Ok(Cmd::Inhibit { command, .. }) = parsed {
        assert_eq!(command, ["make", "-j4"]);
    }
    // Hyphen args reach the child even without the `--` separator.
    let parsed = parse(&["inhibit", "make", "-j4"]);
    assert!(
        matches!(parsed, Ok(Cmd::Inhibit { .. })),
        "inhibit make -j4 must parse: {parsed:?}"
    );
    if let Ok(Cmd::Inhibit { command, reason }) = parsed {
        assert_eq!(command, ["make", "-j4"]);
        assert!(reason.is_none());
    }
    // Missing command is a usage error.
    assert!(parse(&["inhibit"]).is_err());
}

#[test]
fn lifecycle_and_file_ops_parse() {
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
        parse(&["config", "edit"]),
        Ok(Cmd::Config {
            op: Some(ConfigOp::Edit),
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

#[test]
fn file_ops_do_not_need_daemon() {
    for args in [
        vec!["config", "path"],
        vec!["config", "edit"],
        vec!["config", "reset", "-y"],
        vec!["restart"],
        vec!["logs"],
        vec!["clean", "-n"],
    ] {
        let cmd = parse(&args).expect("parse failed");
        assert!(!cmd.needs_daemon(), "{args:?} should be daemon-free");
    }
    assert!(parse(&["inhibit", "true"]).unwrap().needs_daemon());
    assert!(parse(&["start"]).unwrap().needs_daemon());
    assert!(parse(&["activate"]).unwrap().needs_daemon());
}

#[test]
fn quiet_and_json_flags_parse_everywhere() {
    // Global -q works before or after the subcommand.
    assert!(parse(&["-q", "status"]).is_ok());
    assert!(parse(&["status", "-q"]).is_ok());
    assert!(parse(&["config", "set", "timeout", "5", "-q"]).is_ok());
    // --json on the newly-covered commands.
    assert!(matches!(
        parse(&["saver", "--json"]),
        Ok(Cmd::Saver { json: true, .. })
    ));
    assert!(matches!(
        parse(&["saver", "set", "storm", "--json"]),
        Ok(Cmd::Saver { json: true, .. })
    ));
    assert!(matches!(
        parse(&["timeout", "--json"]),
        Ok(Cmd::Timeout { json: true, .. })
    ));
    assert!(matches!(
        parse(&["config", "list", "--json"]),
        Ok(Cmd::Config { json: true, .. })
    ));
    assert!(matches!(
        parse(&["version", "--json"]),
        Ok(Cmd::Version { json: true, .. })
    ));
}
