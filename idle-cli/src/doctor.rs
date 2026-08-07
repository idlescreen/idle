use anyhow::Result;
use std::process::Command;

use super::doctor_checks::CheckResult;
use super::doctor_env::{check_protocol_hints, check_wayland};
use super::doctor_fs::{check_config_parses, check_shm_permissions, check_yaml_syntax};
use super::doctor_rules::all_systems_nominal;
use super::doctor_service::{
    check_dbus, check_inhibitor, check_running_pid, check_savers, check_systemd_service,
    check_tui_optional,
};
use super::doctor_sys::{check_fonts, check_package_install};

/// Run diagnostics. When `fix` is true, attempt to reload/enable/restart the
/// user unit so upgrades do not require remembering systemctl flags.
/// When `json` is true, print a machine-readable report on stdout.
pub fn run_doctor(fix: bool, json: bool) -> Result<()> {
    if fix && !json {
        fix_user_service()?;
        println!();
    } else if fix && json {
        // Still apply fix; keep stdout JSON-only.
        let _ = fix_user_service_quiet();
    }

    let results = vec![
        check_wayland(),
        check_protocol_hints(),
        check_dbus(),
        check_systemd_service(),
        check_running_pid(),
        check_inhibitor(),
        check_savers(),
        check_tui_optional(),
        check_config_parses(),
        check_yaml_syntax(),
        check_shm_permissions(),
        check_fonts(),
        check_package_install(),
    ];

    if json {
        print_json(&results);
    } else {
        println!("==========================================");
        println!("IdleScreen System Diagnostics (Doctor)");
        println!("==========================================");
        print_results(&results);
        if !all_systems_nominal(&results) {
            if !fix {
                println!("Hint: try  idlescreen doctor --fix  to reload/enable the user service.");
            }
            std::process::exit(1);
        }
    }

    if json && !all_systems_nominal(&results) {
        std::process::exit(1);
    }
    Ok(())
}

fn fix_user_service_quiet() -> Result<()> {
    if let Ok(home) = std::env::var("HOME") {
        let config_dir = std::path::PathBuf::from(home).join(".config").join("idle");
        let _ = std::fs::create_dir_all(&config_dir);
    }
    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    let enable = Command::new("systemctl")
        .args(["--user", "enable", "--now", "idle-daemon.service"])
        .status();
    if enable.map(|s| !s.success()).unwrap_or(true) {
        let _ = Command::new("systemctl")
            .args(["--user", "restart", "idle-daemon.service"])
            .status();
    }
    Ok(())
}

/// Best-effort recovery after package upgrade or a dead session service.
fn fix_user_service() -> Result<()> {
    println!("--fix: reloading and ensuring idle-daemon user service...");

    if let Ok(home) = std::env::var("HOME") {
        let config_dir = std::path::PathBuf::from(home).join(".config").join("idle");
        if !config_dir.exists() && std::fs::create_dir_all(&config_dir).is_ok() {
            println!(
                "  [ok] Checked standard configuration directory {}",
                config_dir.display()
            );
        }
    }

    let reload = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    if let Ok(st) = reload
        && st.success()
    {
        println!("  [ok] systemctl --user daemon-reload");
    } else {
        println!("  [!] systemctl daemon-reload returned an error or was missing");
    }

    let enable = Command::new("systemctl")
        .args(["--user", "enable", "--now", "idle-daemon.service"])
        .status();
    if let Ok(st) = enable
        && st.success()
    {
        println!("  [ok] systemctl --user enable --now idle-daemon.service");
    } else {
        println!("  [!] enable --now failed; attempting restart...");
        let _ = Command::new("systemctl")
            .args(["--user", "restart", "idle-daemon.service"])
            .status();
    }
    Ok(())
}

fn print_results(results: &[CheckResult]) {
    for result in results {
        let marker = if result.passed { "ok" } else { "FAIL" };
        println!("  [{marker}] {}: {}", result.name, result.detail);
    }
    println!("==========================================");
    if all_systems_nominal(results) {
        println!("Diagnostics complete: ALL SYSTEMS NOMINAL.");
        println!("Daemon is up and idle is free to run savers.");
    } else {
        println!("Diagnostics complete: PROBLEMS DETECTED.");
        println!("Savers will not run reliably until every [FAIL] is fixed.");
        println!("Resolve issues marked FAIL. See docs/BOUNDARIES.md for platform limits.");
    }
}

fn print_json(results: &[CheckResult]) {
    let mut out = String::from("{\n  \"ok\": ");
    out.push_str(if all_systems_nominal(results) {
        "true"
    } else {
        "false"
    });
    out.push_str(",\n  \"checks\": [\n");
    for (i, r) in results.iter().enumerate() {
        let detail = escape_json(&r.detail);
        out.push_str(&format!(
            "    {{\"name\": \"{}\", \"passed\": {}, \"detail\": \"{detail}\"}}",
            r.name,
            if r.passed { "true" } else { "false" }
        ));
        if i + 1 < results.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    print!("{out}");
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
