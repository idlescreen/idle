use anyhow::Result;
use std::process::Command;

use super::doctor_checks::{CheckResult, Severity};
use super::doctor_env::{check_protocol_hints, check_wayland};
use super::doctor_fs::{check_config, check_shm_permissions};
use super::doctor_pkg::check_package_install;
use super::doctor_rules::all_systems_nominal;
use super::doctor_rules::tally;
use super::doctor_service::{
    check_dbus, check_inhibitor, check_running_pid, check_savers, check_systemd_service,
    check_tui_optional,
};
use super::doctor_sys::{check_cgroup, check_fonts};

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
        check_config(),
        check_shm_permissions(),
        check_cgroup(),
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
    remove_stale_pid();
    if let Ok(home) = std::env::var("HOME") {
        let config_dir = std::path::PathBuf::from(home).join(".config").join("idle");
        let _ = std::fs::create_dir_all(&config_dir);
    }
    // .output() — child stdout must not leak into --json output.
    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output();
    let enable = Command::new("systemctl")
        .args(["--user", "enable", "--now", "idle-daemon.service"])
        .output();
    if enable.map(|o| !o.status.success()).unwrap_or(true) {
        let _ = Command::new("systemctl")
            .args(["--user", "restart", "idle-daemon.service"])
            .output();
    }
    Ok(())
}

/// Drop a pid file whose process no longer exists.
fn remove_stale_pid() {
    let path = super::doctor_service::pid_file_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return;
    };
    let Ok(pid) = content.trim().parse::<i32>() else {
        let _ = std::fs::remove_file(&path);
        return;
    };
    // SAFETY: kill(pid, 0) probes existence only.
    if unsafe { libc::kill(pid, 0) } != 0 {
        let _ = std::fs::remove_file(&path);
    }
}

/// Best-effort recovery after package upgrade or a dead session service.
fn fix_user_service() -> Result<()> {
    println!("--fix: reloading and ensuring idle-daemon user service...");
    remove_stale_pid();

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
        let marker = match result.severity {
            Severity::Ok => "ok",
            Severity::Warn => "warn",
            Severity::Fail => "FAIL",
        };
        println!("  [{marker}] {}: {}", result.name, result.detail);
        if let Some(fix) = &result.fix {
            println!("       -> fix: {fix}");
        }
    }
    println!("==========================================");
    let (fails, warns) = tally(results);
    println!(
        "Summary: {} checks — {} failed, {} warning(s)",
        results.len(),
        fails,
        warns
    );
    if fails == 0 {
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
        let sev = match r.severity {
            Severity::Ok => "ok",
            Severity::Warn => "warn",
            Severity::Fail => "fail",
        };
        let fix = match &r.fix {
            Some(f) => format!(", \"fix\": \"{}\"", escape_json(f)),
            None => String::new(),
        };
        out.push_str(&format!(
            "    {{\"name\": \"{}\", \"severity\": \"{sev}\", \"passed\": {}, \"detail\": \"{detail}\"{fix}}}",
            r.name,
            if r.passed() { "true" } else { "false" }
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
