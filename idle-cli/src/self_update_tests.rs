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
fn versions_equalish_ignores_arch() {
    assert!(versions_equalish("2.5.0-1", "2.5.0-1"));
    assert!(versions_equalish("2.5.0-1.x86_64", "2.5.0-1"));
    assert!(!versions_equalish("2.4.0-1", "2.5.0-1"));
}
