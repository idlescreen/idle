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
    if PKG_CANDIDATES.iter().any(|p| command_ok("rpm", &["-q", p])) {
        return Some(Backend::Dnf);
    }
    // Package counts as apt-installed when a status query succeeds and says
    // "install ok installed", or when the query fails but `dpkg -s` knows it.
    let apt_has = |p: &str| {
        (command_ok("dpkg-query", &["-W", "-f=${Status}", p]) || command_ok("dpkg", &["-s", p]))
            && stdout_trim("dpkg-query", &["-W", "-f=${Status}", p])
                .map(|s| s.contains("install ok installed"))
                .unwrap_or(true)
    };
    if PKG_CANDIDATES.iter().any(|p| apt_has(p)) {
        return Some(Backend::Apt);
    }

    if let Ok(os) = std::fs::read_to_string("/etc/os-release") {
        let field = |key: &str| {
            os.lines()
                .find_map(|l| l.strip_prefix(key))
                .unwrap_or("")
                .trim_matches('"')
                .to_string()
        };
        let (id, like) = (field("ID="), field("ID_LIKE="));
        let like_has = |t: &str| like.split_whitespace().any(|x| x == t);
        if (matches!(
            id.as_str(),
            "fedora" | "rhel" | "centos" | "rocky" | "almalinux"
        ) || like_has("fedora")
            || like_has("rhel")
            || like_has("centos"))
            && (which("dnf") || which("rpm"))
        {
            return Some(Backend::Dnf);
        }
        if (matches!(id.as_str(), "debian" | "ubuntu" | "pop")
            || like_has("debian")
            || like_has("ubuntu"))
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

/// Parse `dnf check-update` output: `name.arch version repo` rows.
/// Only rows whose name matches an installed IdleScreen package count;
/// a package listed by several repos is reported once.
pub fn parse_dnf_check_update(text: &str, installed: &[String]) -> Vec<(String, String)> {
    let mut seen = std::collections::HashSet::new();
    text.lines()
        .filter_map(|l| {
            let mut parts = l.split_whitespace();
            let token = parts.next()?;
            let name = token.rsplit_once('.').map(|(n, _)| n).unwrap_or(token);
            let ver = parts.next()?;
            (installed.iter().any(|p| p == name) && seen.insert(name.to_string()))
                .then(|| (name.to_string(), ver.to_string()))
        })
        .collect()
}

/// Parse `apt list --upgradable`: `name/repo ver arch [upgradable from: x]`.
/// A package listed by several suites is reported once.
pub fn parse_apt_upgradable(text: &str, installed: &[String]) -> Vec<(String, String)> {
    let mut seen = std::collections::HashSet::new();
    text.lines()
        .filter_map(|l| {
            let mut parts = l.split_whitespace();
            let name = parts.next()?.split('/').next()?;
            let ver = parts.next()?;
            (installed.iter().any(|p| p == name) && seen.insert(name.to_string()))
                .then(|| (name.to_string(), ver.to_string()))
        })
        .collect()
}

/// Which of `installed` have a pending upgrade, per the package manager's
/// cached view. `None` = could not determine; caller should still attempt
/// the upgrade (fail-open — the package manager is authoritative anyway).
pub fn upgradable_packages(
    backend: Backend,
    installed: &[String],
) -> Option<Vec<(String, String)>> {
    if installed.is_empty() {
        return Some(Vec::new());
    }
    match backend {
        Backend::Dnf => {
            // check-update exits 100 when updates exist, 0 when current.
            // `-y` auto-accepts repo key imports into the per-user keyring —
            // without it a repo_gpgcheck repo never loads for a non-root
            // user and exit 0 falsely reports "no updates". Belt-and-braces:
            // if a repo still failed to load, treat the result as unknown.
            let out = Command::new("dnf")
                .args(["-y", "check-update", "--refresh", "--repo=idlescreen"])
                .args(installed)
                .output()
                .ok()?;
            if dnf_repo_load_failed(&out) {
                return None;
            }
            match out.status.code() {
                Some(0) => Some(Vec::new()),
                Some(100) => Some(parse_dnf_check_update(
                    &String::from_utf8_lossy(&out.stdout),
                    installed,
                )),
                _ => None,
            }
        }
        Backend::Apt => {
            let out = Command::new("apt")
                .args(["list", "--upgradable"])
                .output()
                .ok()?;
            out.status
                .success()
                .then(|| parse_apt_upgradable(&String::from_utf8_lossy(&out.stdout), installed))
        }
    }
}

/// True when dnf reported that a repo's metadata could not be verified or
/// downloaded — in which case a "no updates" answer cannot be trusted.
fn dnf_repo_load_failed(out: &std::process::Output) -> bool {
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    [
        "verification error",
        "Signing key not found",
        "Failed to download metadata",
        "Cannot download repomd",
    ]
    .iter()
    .any(|m| all.contains(m))
}

/// Currently installed version of the given package (None = not installed).
pub fn installed_version(backend: Backend, pkg: &str) -> Option<String> {
    match backend {
        Backend::Dnf => stdout_trim("rpm", &["-q", pkg, "--qf", "%{VERSION}-%{RELEASE}"]),
        Backend::Apt => stdout_trim("dpkg-query", &["-W", "-f=${Version}", pkg]),
    }
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
