// SPDX-License-Identifier: MIT

use crate::self_update_backend::{
    parse_apt_upgradable, parse_dnf_check_update, parse_installed_lines,
};

fn inst() -> Vec<String> {
    vec![
        "idle-cli".to_string(),
        "idle-daemon".to_string(),
        "idle-saver-beams".to_string(),
    ]
}

#[test]
fn parse_installed_keeps_only_installed_rows() {
    let dpkg = "idle-cli\tinstalled\nidle-daemon\tinstalled\nidle-savers\tdeinstall\nidle-tui\tconfig-files\n";
    assert_eq!(
        parse_installed_lines(dpkg),
        vec!["idle-cli".to_string(), "idle-daemon".to_string()]
    );

    let rpm = "idle-cli\nidle-daemon\nidle-saver-beams\n";
    assert_eq!(parse_installed_lines(rpm).len(), 3);

    assert!(parse_installed_lines("").is_empty());
    assert!(parse_installed_lines("\n\t\n").is_empty());
}

#[test]
fn dnf_check_update_parses_upgrade_rows() {
    let text = "\
Last metadata expiration check: 0:05:12 ago on Fri 12 Sep 2025 09:30:00.
idle-cli.x86_64                    3.5.1-1                 idlescreen
idle-daemon.x86_64                 0:3.5.1-1               idlescreen
unrelated-pkg.noarch               9.9-1                   fedora
";
    assert_eq!(
        parse_dnf_check_update(text, &inst()),
        vec![
            ("idle-cli".to_string(), "3.5.1-1".to_string()),
            ("idle-daemon".to_string(), "0:3.5.1-1".to_string()),
        ]
    );
}

#[test]
fn dnf_check_update_dedupes_multi_repo_rows() {
    let text = "idle-cli.x86_64 3.5.1-1 idlescreen\nidle-cli.x86_64 3.5.1-1 updates\n";
    assert_eq!(
        parse_dnf_check_update(text, &inst()).len(),
        1,
        "same package from two repos must count once"
    );
}

#[test]
fn dnf_check_update_rejects_garbage() {
    let i = inst();
    assert!(parse_dnf_check_update("", &i).is_empty());
    assert!(parse_dnf_check_update("\n\n\n", &i).is_empty());
    assert!(parse_dnf_check_update("single-word\n", &i).is_empty());
    assert!(parse_dnf_check_update("idle-cli\n", &i).is_empty());
    assert!(parse_dnf_check_update("idle-cli.x86_64\n", &i).is_empty());
    // Well-formed row for a package we do not have installed.
    assert!(parse_dnf_check_update("evil-pkg.x86_64 1.0-1 repo\n", &i).is_empty());
    // Bare name without arch suffix still parses.
    assert_eq!(
        parse_dnf_check_update("idle-cli 3.5.1-1 idlescreen\n", &i),
        vec![("idle-cli".to_string(), "3.5.1-1".to_string())]
    );
    // Oversized input must terminate, not hang or panic.
    let big = "x.y 1.0 r\n".repeat(100_000);
    assert!(parse_dnf_check_update(&big, &i).is_empty());
}

#[test]
fn apt_upgradable_parses_rows_and_skips_header() {
    let text = "\
Listing... Done
idle-cli/stable 3.5.1-1 amd64 [upgradable from: 3.5.0-1]
idle-daemon/stable,stable 3.5.1-1 amd64 [upgradable from: 3.5.0-1]
unrelated/stable 1.0 amd64 [upgradable from: 0.9]
";
    assert_eq!(
        parse_apt_upgradable(text, &inst()),
        vec![
            ("idle-cli".to_string(), "3.5.1-1".to_string()),
            ("idle-daemon".to_string(), "3.5.1-1".to_string()),
        ]
    );
}

#[test]
fn apt_upgradable_dedupes_multi_suite_rows() {
    let text = "idle-cli/stable 3.5.1-1 amd64\nidle-cli/testing 3.5.1-2 amd64\n";
    assert_eq!(parse_apt_upgradable(text, &inst()).len(), 1);
}

#[test]
fn apt_upgradable_rejects_garbage() {
    let i = inst();
    assert!(parse_apt_upgradable("", &i).is_empty());
    assert!(parse_apt_upgradable("Listing... Done\n", &i).is_empty());
    assert!(parse_apt_upgradable("idle-cli\n", &i).is_empty());
    assert!(parse_apt_upgradable("notinstalled/foo 1.0 amd64\n", &i).is_empty());
    let big = "x/y 1.0 amd64\n".repeat(100_000);
    assert!(parse_apt_upgradable(&big, &i).is_empty());
}
