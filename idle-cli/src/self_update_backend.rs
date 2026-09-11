// SPDX-License-Identifier: MIT

use std::process::Command;

/// Product/engine package names for update detection (NEVRA order preferred).
pub const PKG_CANDIDATES: &[&str] = &[
    "idle-cli",
    "idle-daemon",
    "idle-savers",
    "idle-tui",
    "idle-cosmic",
    "idlescreen",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Apt,
    Dnf,
}

pub fn command_ok(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn stdout_trim(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

pub fn detect_backend() -> Option<Backend> {
    for pkg in PKG_CANDIDATES {
        if command_ok("rpm", &["-q", pkg]) {
            return Some(Backend::Dnf);
        }
    }
    for pkg in PKG_CANDIDATES {
        if command_ok("dpkg-query", &["-W", "-f=${Status}", pkg])
            || command_ok("dpkg", &["-s", pkg])
        {
            if let Some(status) = stdout_trim("dpkg-query", &["-W", "-f=${Status}", pkg]) {
                if status.contains("install ok installed") {
                    return Some(Backend::Apt);
                }
            } else {
                return Some(Backend::Apt);
            }
        }
    }

    if let Ok(os) = std::fs::read_to_string("/etc/os-release") {
        let id = os
            .lines()
            .find_map(|l| l.strip_prefix("ID="))
            .unwrap_or("")
            .trim_matches('"');
        let like = os
            .lines()
            .find_map(|l| l.strip_prefix("ID_LIKE="))
            .unwrap_or("")
            .trim_matches('"');
        if (id == "fedora"
            || id == "rhel"
            || id == "centos"
            || id == "rocky"
            || id == "almalinux"
            || like
                .split_whitespace()
                .any(|t| matches!(t, "fedora" | "rhel" | "centos")))
            && (which("dnf") || which("rpm"))
        {
            return Some(Backend::Dnf);
        }
        if (id == "debian"
            || id == "ubuntu"
            || id == "pop"
            || like
                .split_whitespace()
                .any(|t| matches!(t, "debian" | "ubuntu")))
            && (which("apt-cache") || which("apt"))
        {
            return Some(Backend::Apt);
        }
    }

    if which("dnf") {
        return Some(Backend::Dnf);
    }
    if which("apt-cache") {
        return Some(Backend::Apt);
    }
    None
}

pub fn which(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|dir| dir.join(cmd).is_file()))
        .unwrap_or(false)
}

/// Installed IdleScreen packages on this host (every `idle-*`/`idlescreen*`
/// package known to the package DB, not just the update-check candidates).
pub fn installed_packages(backend: Backend) -> Vec<String> {
    let out = match backend {
        Backend::Apt => Command::new("dpkg-query")
            .args([
                "-W",
                "-f=${binary:Package}\t${db:Status-Status}\n",
                "idle-*",
                "idlescreen*",
            ])
            .output(),
        Backend::Dnf => Command::new("rpm")
            .args(["-qa", "--qf", "%{NAME}\n", "idle-*", "idlescreen*"])
            .output(),
    };
    let Ok(out) = out else { return Vec::new() };
    if !out.status.success() {
        return Vec::new();
    }
    parse_installed_lines(&String::from_utf8_lossy(&out.stdout))
}

/// Package-name-per-line parser. dpkg-query emits `name\tstatus` rows and
/// lists removed-but-configured packages too — only `installed` rows count.
/// `rpm -qa` emits bare names, so a line without a tab is taken as-is.
pub fn parse_installed_lines(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| {
            let name = l.split('\t').next()?.trim();
            if l.contains('\t') && !l.ends_with("installed") {
                return None;
            }
            (!name.is_empty()).then(|| name.to_string())
        })
        .collect()
}

/// Run `argv` as root: directly when euid==0, via sudo otherwise.
/// Returns Err only when the command could not be spawned at all.
pub fn run_privileged(argv: &[&str]) -> Result<std::process::ExitStatus, String> {
    let (prog, args) = if unsafe { libc::geteuid() } == 0 {
        (argv[0].to_string(), argv[1..].to_vec())
    } else if which("sudo") {
        ("sudo".to_string(), argv.to_vec())
    } else {
        return Err("needs root and sudo is not installed".to_string());
    };
    Command::new(prog)
        .args(args)
        .status()
        .map_err(|e| format!("failed to run {}: {e}", argv[0]))
}
