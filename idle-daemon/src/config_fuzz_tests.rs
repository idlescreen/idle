// SPDX-License-Identifier: MIT

//! Fuzz tests for the config parser: every byte pattern that can appear in a
//! user-editable config.yaml must parse without panic and leave the config in
//! a valid state (bounded timeout, finite in-range scale, charset-clean saver).

use crate::config::DaemonConfig;
use crate::config_parse::apply_config_line;
use idle_api::LcgRng;

/// Post-parse invariants the parser must preserve regardless of input.
fn assert_config_valid(c: &DaemonConfig) {
    assert!((1..=240).contains(&c.idle_timeout_mins));
    if let Some(s) = c.render_scale {
        assert!(
            s.is_finite() && (0.25..=1.0).contains(&s),
            "scale {s} out of range"
        );
    }
    if let Some(name) = &c.active_saver {
        assert!(!name.is_empty());
        assert!(
            name.chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-'),
            "active_saver not charset-clean: {name:?}"
        );
    }
}

#[test]
fn hostile_key_value_lines() {
    // Targeted hostile lines: extreme numerics, injection, section tricks.
    let lines = [
        "idle_timeout_mins: 0",
        "idle_timeout_mins: 240",
        "idle_timeout_mins: 241",
        "idle_timeout_mins: 4294967295",
        "idle_timeout_mins: -5",
        "idle_timeout_mins: abc",
        "idle_timeout_mins: 5extra",
        "idle_timeout_mins:  5  ",
        "idle_timeout_mins: 005",
        "idle_timeout_mins:",
        "render_scale: NaN",
        "render_scale: nan",
        "render_scale: inf",
        "render_scale: -inf",
        "render_scale: 1e999",
        "render_scale: -0.5",
        "render_scale: 0.25",
        "render_scale: 1.0",
        "render_scale: 0.0",
        "active_saver: ../../etc/passwd",
        "active_saver: evil;sudo reboot",
        "active_saver: beams\u{0}x",
        "active_saver: ",
        "active_saver: RANDOM",
        "active_saver: Shuffle",
        "idle_enabled: TRUE",
        "idle_enabled: 1",
        "idle_enabled: notabool",
        "key_without_colon",
        ":",
        "::",
        "[",
        "]",
        "[]",
        "[saver",
        "saver]",
        "[saver]",
        "[saver.params]",
        "theme: notatheme",
    ];
    for line in lines {
        let mut c = DaemonConfig::default();
        let mut section = String::new();
        apply_config_line(&mut c, &mut section, line);
        assert_config_valid(&c);
    }
}

#[test]
fn seeded_line_soup() {
    let mut rng = LcgRng::new(0xBEEF);
    for _ in 0..20_000 {
        let mut c = DaemonConfig::default();
        let mut section = String::new();
        // Hostile alphabet incl. unicode + control + structural chars.
        const ALPHABET: &[u8] = b"abz019-./\\\0\x7f~:[]\"' \t".as_slice();
        let len = rng.next_usize(80);
        let mut line = String::with_capacity(len);
        for _ in 0..len {
            line.push(ALPHABET[rng.next_usize(ALPHABET.len())] as char);
        }
        apply_config_line(&mut c, &mut section, &line);
        assert_config_valid(&c);
    }
}

#[test]
fn oversized_and_edge_lines() {
    let huge_val = "x".repeat(1_000_000);
    let huge_key = "k".repeat(100_000);
    for line in [
        format!("render_scale: {huge_val}"),
        format!("{huge_key}: v"),
        format!("active_saver: {huge_val}"),
        format!("idle_timeout_mins: {}", "9".repeat(500)),
        "a".repeat(1_000_000),
    ] {
        let mut c = DaemonConfig::default();
        let mut section = String::new();
        apply_config_line(&mut c, &mut section, &line);
        assert_config_valid(&c);
    }
}

use crate::config_parse::merge_config_body;
use std::collections::BTreeMap;

fn fields() -> Vec<(&'static str, String)> {
    vec![
        ("idle_timeout_mins", "10".into()),
        ("active_saver", "\"beams\"".into()),
        ("idle_enabled", "true".into()),
        ("show_fps_overlay", "false".into()),
        ("render_scale", "null".into()),
        ("theme", "\"synthwave\"".into()),
        ("strict_control", "false".into()),
    ]
}

#[test]
fn merge_preserves_foreign_keys_and_comments() {
    let existing = "# my header\naccent_color: \"#FF0000\"\ntheme_idx: 3\nidle_timeout_mins: 5\nmy_note: keep me\n";
    let out = merge_config_body(existing, &mut fields(), &BTreeMap::new());
    assert!(out.contains("accent_color: \"#FF0000\""));
    assert!(out.contains("theme_idx: 3"));
    assert!(out.contains("# my header"));
    assert!(out.contains("my_note: keep me"));
    assert!(out.contains("idle_timeout_mins: 10"));
    assert!(!out.contains("idle_timeout_mins: 5"));
    // Every daemon-owned field lands even when absent from the old file.
    for k in ["active_saver", "theme", "strict_control", "render_scale"] {
        assert!(out.contains(&format!("{k}:")), "missing {k}");
    }
}

#[test]
fn merge_rewrites_saver_sections() {
    let existing = "idle_enabled: false\n[saver]\nbeams.speed: 9\n[saver.storm]\ndensity: 4\n";
    let mut params = BTreeMap::new();
    params.insert("beams.speed".to_string(), "2".to_string());
    let out = merge_config_body(existing, &mut fields(), &params);
    assert!(out.contains("[saver]\nbeams.speed: 2"));
    assert!(!out.contains("density"), "removed params must not linger");
    assert!(out.contains("idle_enabled: true"));
}

#[test]
fn merge_leaves_keys_inside_other_sections_alone() {
    let existing = "idle_enabled: false\n[plugin]\ntheme: \"other\"\n";
    let out = merge_config_body(existing, &mut fields(), &BTreeMap::new());
    assert!(out.contains("[plugin]\ntheme: \"other\""));
    // top-level theme still emitted (it was absent at top level)
    assert!(out.contains("theme: \"synthwave\""));
}

#[test]
fn merge_empty_file_emits_template() {
    let out = merge_config_body("", &mut fields(), &BTreeMap::new());
    assert!(out.contains("accent_color"));
    assert!(out.contains("idle_timeout_mins: 10"));
    assert!(out.contains("theme: \"synthwave\""));
}

#[test]
fn save_roundtrip_preserves_foreign_keys() {
    let dir = std::env::temp_dir().join(format!("idle-cfg-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("config.yaml");
    std::fs::write(
        &file,
        "# user header\naccent_color: \"#AABBCC\"\ntheme_idx: 7\nidle_timeout_mins: 5\n\
         active_saver: \"beams\"\nstrict_control: true\ncustom_key: 42\n[saver]\nbeams.speed: 1\n",
    )
    .unwrap();
    unsafe { std::env::set_var("IDLE_CONFIG_DIR", &dir) };
    let mut cfg = DaemonConfig::load();
    cfg.idle_timeout_mins = 20;
    cfg.saver_params.clear();
    cfg.save().unwrap();
    let out = std::fs::read_to_string(&file).unwrap();
    unsafe { std::env::remove_var("IDLE_CONFIG_DIR") };
    assert!(out.contains("accent_color: \"#AABBCC\""));
    assert!(out.contains("theme_idx: 7"));
    assert!(out.contains("custom_key: 42"));
    assert!(out.contains("# user header"));
    assert!(out.contains("idle_timeout_mins: 20"));
    assert!(out.contains("strict_control: true"));
    assert!(!out.contains("beams.speed"), "cleared params must drop");
}
