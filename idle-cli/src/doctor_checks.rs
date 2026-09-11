// SPDX-License-Identifier: MIT

//! Doctor check result type — three severities like brew/flutter/mise.
//!
//! `Fail` blocks NOMINAL and exits 1. `Warn` prints but does not fail —
//! degraded or optional capabilities (missing TUI, memfd fallback,
//! un-delegated cgroup controllers).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: &'static str,
    pub severity: Severity,
    pub detail: String,
    /// Remediation hint — rendered as a `-> fix:` line and emitted in JSON.
    pub fix: Option<String>,
}

impl CheckResult {
    pub fn passed(&self) -> bool {
        self.severity != Severity::Fail
    }

    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }
}

pub fn ok(name: &'static str, detail: impl Into<String>) -> CheckResult {
    CheckResult {
        name,
        severity: Severity::Ok,
        detail: detail.into(),
        fix: None,
    }
}

pub fn warn(name: &'static str, detail: impl Into<String>) -> CheckResult {
    CheckResult {
        name,
        severity: Severity::Warn,
        detail: detail.into(),
        fix: None,
    }
}

pub fn fail(name: &'static str, detail: impl Into<String>) -> CheckResult {
    CheckResult {
        name,
        severity: Severity::Fail,
        detail: detail.into(),
        fix: None,
    }
}
