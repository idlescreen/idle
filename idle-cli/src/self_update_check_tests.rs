// SPDX-License-Identifier: MIT

use super::*;

#[test]
fn parse_dnf5_list_available() {
    let text = "\
Installed packages
idle-daemon.x86_64 2.4.0-1 idlescreen

Available packages
idle-daemon.x86_64 2.3.0-1 idlescreen
idle-daemon.x86_64 2.5.0-1 idlescreen
";
    assert_eq!(
        parse_dnf_list_version(text, true).as_deref(),
        Some("2.5.0-1")
    );
    assert_eq!(
        parse_dnf_list_version(text, false).as_deref(),
        Some("2.4.0-1")
    );
}

#[test]
fn version_cmp_orders_release_and_downgrade() {
    use std::cmp::Ordering::*;
    assert_eq!(version_cmp("3.5.1-1", "3.5.3-1"), Less);
    assert_eq!(version_cmp("3.5.3-1", "3.5.1-1"), Greater); // stale-candidate case
    assert_eq!(version_cmp("3.5.1-1", "3.5.1-2"), Less); // same version, newer release
    assert_eq!(version_cmp("0:3.5.1-1", "3.5.1-1"), Equal); // epoch ignored
    assert_eq!(version_cmp("3.5.1-1.x86_64", "3.5.1-1"), Equal);
    assert_eq!(version_cmp("3.10.0-1", "3.9.9-9"), Greater); // not lexical
}

#[test]
fn versions_equalish_ignores_arch() {
    assert!(versions_equalish("2.5.0-1", "2.5.0-1"));
    assert!(versions_equalish("2.5.0-1.x86_64", "2.5.0-1"));
    assert!(!versions_equalish("2.4.0-1", "2.5.0-1"));
}

#[test]
fn versions_equalish_ignores_epoch() {
    // dnf reports "0:3.5.1-1"; rpm -q answers "3.5.1-1".
    assert!(versions_equalish("0:3.5.1-1", "3.5.1-1"));
    assert!(versions_equalish("3.5.1-1", "0:3.5.1-1"));
    assert!(versions_equalish("1:3.5.1-1", "2:3.5.1-1")); // epoch ignored
    assert!(!versions_equalish("0:3.5.0-1", "3.5.1-1"));
}

#[test]
fn parse_apt_policy_extracts_installed_and_candidate() {
    let text = "idle-cli:\n  Installed: 3.5.0-1\n  Candidate: 3.5.1-1\n  Version table:\n";
    assert_eq!(
        parse_apt_policy(text),
        Some(("3.5.0-1".to_string(), "3.5.1-1".to_string()))
    );
}

#[test]
fn parse_apt_policy_not_installed_reports_none_placeholder() {
    let text = "idle-cli:\n  Installed: (none)\n  Candidate: 3.5.1-1\n";
    assert_eq!(
        parse_apt_policy(text),
        Some(("(none)".to_string(), "3.5.1-1".to_string()))
    );
}

#[test]
fn parse_apt_policy_rejects_garbage() {
    assert!(parse_apt_policy("").is_none());
    assert!(parse_apt_policy("garbage\nmore garbage\n").is_none());
    // No candidate available anywhere.
    assert!(parse_apt_policy("  Installed: 3.5.0-1\n  Candidate: (none)\n").is_none());
    // Missing Installed line is treated like a not-installed package.
    assert_eq!(
        parse_apt_policy("  Candidate: 3.5.1-1\n"),
        Some(("(none)".to_string(), "3.5.1-1".to_string()))
    );
}
