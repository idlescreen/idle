use std::path::Path;
use std::process::Command;

pub fn query_dpkg(name: &str) -> Option<String> {
    let o = Command::new("dpkg-query")
        .args(["-W", "-f=${Package} ${Version}\\n", name])
        .output()
        .ok()?;
    if !o.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
    if s.is_empty() || s.contains("no packages found") {
        None
    } else {
        Some(s)
    }
}

pub fn query_rpm_file(path: &Path) -> Option<String> {
    let o = Command::new("rpm")
        .args(["-qf", path.to_str()?])
        .output()
        .ok()?;
    if !o.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
    if s.is_empty() || s.contains("is not owned") {
        None
    } else {
        Some(s)
    }
}

pub fn query_dpkg_file(path: &Path) -> Option<String> {
    let o = Command::new("dpkg")
        .args(["-S", path.to_str()?])
        .output()
        .ok()?;
    if !o.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&o.stdout);
    let pkg = line.split(':').next()?.trim();
    if pkg.is_empty() || pkg.contains("no path found") {
        return None;
    }
    query_dpkg(pkg).or_else(|| Some(pkg.to_string()))
}
