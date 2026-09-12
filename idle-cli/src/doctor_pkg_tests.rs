// SPDX-License-Identifier: MIT

use super::package_rank;
use crate::doctor_pkg_fmt::{display_entry, major_minor};

#[test]
fn major_minor_parses_release_versions() {
    assert_eq!(major_minor("3.5.2-1"), Some((3, 5)));
    assert_eq!(major_minor("3.5.2"), Some((3, 5)));
    assert_eq!(major_minor("2.3.1-1.x86_64"), Some((2, 3)));
    assert_eq!(major_minor("abc"), None);
}

#[test]
fn display_entry_maps_roles() {
    assert_eq!(
        display_entry("idle-daemon-3.5.2-1.x86_64"),
        "daemon 3.5.2-1"
    );
    assert_eq!(
        display_entry("idle-cli-3.5.4-1.x86_64"),
        "cli (the 'idlescreen' command) 3.5.4-1"
    );
    assert_eq!(
        display_entry("idlescreen-3.0.3-1.noarch"),
        "metapackage 3.0.3-1"
    );
    assert_eq!(
        display_entry("idle-cosmic-3.2.1-1.x86_64"),
        "cosmic-applet 3.2.1-1"
    );
    assert_eq!(
        display_entry("idle-saver-beams-2.2.1-1.x86_64"),
        "saver-beams 2.2.1-1"
    );
    assert_eq!(display_entry("unmanaged-thing"), "unmanaged-thing");
    assert_eq!(display_entry("idle-tui-3.2.1-1.aarch64"), "tui 3.2.1-1");
}

#[test]
fn package_rank_prefers_core() {
    assert!(package_rank("idle-daemon-2.3.1-1.x86_64") < package_rank("idle-cli-2.3.1-1.x86_64"));
    assert!(package_rank("idle-cli-2.3.1-1.x86_64") < package_rank("idle-tui-2.2.0-1.x86_64"));
    assert!(
        package_rank("idle-savers-2.3.1-1.x86_64") < package_rank("idle-cosmic-2.1.2-1.x86_64")
    );
}
