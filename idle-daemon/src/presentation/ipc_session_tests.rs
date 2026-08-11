// Test files legitimately panic; suppress the lint at file scope.
#![allow(clippy::panic)]
// SPDX-License-Identifier: MIT

//! Adversarial tests for the subprocess-isolation primitive.

use super::ipc_session::IpcPluginSession;
use idle_runner::launcher::LaunchMode;

#[test]
fn kill_child_is_idempotent_without_child() {
    let mut s = IpcPluginSession::load_with_options(
        "beams",
        &LaunchMode::Daemon,
        Some(false),
        None,
    )
    .expect("load");
    // No child yet; kill must not panic.
    s.kill_child();
    s.kill_child();
}

#[test]
fn kill_child_clears_handle() {
    let mut s = IpcPluginSession::load_with_options(
        "beams",
        &LaunchMode::Daemon,
        Some(false),
        None,
    )
    .expect("load");
    // Simulate a live child by inserting a dummy process handle would
    // require spawning; we instead assert that the kill path on a
    // `None` child clears state correctly (the slot stays None).
    s.kill_child();
    assert!(s.child.is_none(), "kill on None must leave child slot empty");
}

#[test]
fn child_is_dead_true_without_child() {
    let mut s = IpcPluginSession::load_with_options(
        "beams",
        &LaunchMode::Daemon,
        Some(false),
        None,
    )
    .expect("load");
    assert!(s.child_is_dead(), "no child → reports dead");
}

#[test]
fn expected_stop_is_set_after_kill() {
    let mut s = IpcPluginSession::load_with_options(
        "beams",
        &LaunchMode::Daemon,
        Some(false),
        None,
    )
    .expect("load");
    s.kill_child();
    assert!(
        s.expected_stop.load(std::sync::atomic::Ordering::Acquire),
        "kill_child must mark expected_stop to suppress failsafe spurious liveness events"
    );
}

/// Spawn a real long-running child process (sleep 30), inject it into
/// the session, kill it, and assert the kernel reaped it. This exercises
/// the actual `Child::kill` path — the docstring-stub tests above only
/// cover state transitions.
#[test]
fn kill_child_reaps_real_process() {
    use std::process::Command;

    // `sleep` is in coreutils; available on every Unix CI we target.
    let child = Command::new("sleep")
        .arg("30")
        .spawn()
        .expect("spawn sleep");
    let pid = child.id() as i32;
    assert!(pid > 0, "spawn returned a real pid");

    // Inject the live child into the session.
    let mut s = IpcPluginSession::load_with_options(
        "beams",
        &LaunchMode::Daemon,
        Some(false),
        None,
    )
    .expect("load");
    s.child = Some(child);

    // Sanity: process is alive before kill.
    let before = unsafe { libc::kill(pid, 0) };
    assert_eq!(before, 0, "child must be alive before kill_child()");

    s.kill_child();

    // The reap must have happened: try_wait reports the exit, the
    // process is no longer alive, and the slot is cleared.
    assert!(s.child.is_none(), "kill_child must consume the Child handle");
    let after = unsafe { libc::kill(pid, 0) };
    assert_ne!(
        after, 0,
        "process pid {} must be reaped (kill returned {})",
        pid, after
    );
    // ESRCH is the canonical "no such process" errno; the child of
    // that on Linux is -1 from libc.
    if after == -1 {
        let err = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        assert_eq!(
            err, 3 /* ESRCH */,
            "kill(pid, 0) post-reap must return ESRCH"
        );
    }

    // Second kill is a no-op (idempotent).
    s.kill_child();
}