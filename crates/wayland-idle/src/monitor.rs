// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use crate::wayland;

/// Tracks user inactivity through the Wayland `ext-idle-notify-v1` protocol.
///
/// Returns `None` from [`Self::new`] when `WAYLAND_DISPLAY` is unset or the
/// compositor does not expose the idle notifier.
pub struct IdleMonitor {
    is_idle: Arc<AtomicBool>,
    timeout_tx: mpsc::Sender<u32>,
    shutdown: Arc<AtomicBool>,
    is_alive: Arc<AtomicBool>,
}

impl IdleMonitor {
    /// Connect to the current Wayland session and begin monitoring idle state.
    ///
    /// `timeout_mins` is the initial inactivity threshold. Use [`Self::new_timeout`]
    /// to pass a [`Duration`]; this constructor is preserved for callers that
    /// already pass minutes.
    pub fn new(timeout_mins: u32) -> Option<Self> {
        Self::new_timeout(Duration::from_secs(timeout_mins.saturating_mul(60) as u64))
    }

    /// Connect using a [`Duration`]. Internally the Wayland compositor's
    /// `ext-idle-notify-v1` only accepts a minute granularity, so the
    /// duration is rounded down to whole minutes.
    pub fn new_timeout(timeout: Duration) -> Option<Self> {
        if !Self::is_available() {
            return None;
        }

        let timeout_mins = (timeout.as_secs() / 60).min(u32::MAX as u64) as u32;

        let is_idle = Arc::new(AtomicBool::new(false));
        let (timeout_tx, timeout_rx) = mpsc::channel();
        let shutdown = Arc::new(AtomicBool::new(false));
        let is_alive = Arc::new(AtomicBool::new(true));

        wayland::spawn_event_thread(
            is_idle.clone(),
            shutdown.clone(),
            timeout_rx,
            timeout_mins,
            is_alive.clone(),
        );

        Some(Self {
            is_idle,
            timeout_tx,
            shutdown,
            is_alive,
        })
    }

    /// Whether `WAYLAND_DISPLAY` is set in the environment.
    pub fn is_available() -> bool {
        std::env::var("WAYLAND_DISPLAY").is_ok()
    }

    /// Returns `true` when the user has been idle longer than the configured timeout.
    pub fn is_idle(&self) -> bool {
        self.is_idle.load(Ordering::SeqCst)
    }

    /// Returns `true` if the Wayland event monitoring thread is still running.
    pub fn is_alive(&self) -> bool {
        self.is_alive.load(Ordering::SeqCst)
    }

    /// Update the idle timeout. The compositor is re-notified on the next event-loop tick.
    pub fn set_timeout(&self, timeout_mins: u32) {
        let _ = self.timeout_tx.send(timeout_mins);
    }

    /// Update the idle timeout from a [`Duration`]. Rounded down to whole minutes.
    pub fn set_timeout_duration(&self, timeout: Duration) {
        let mins = (timeout.as_secs() / 60).min(u32::MAX as u64) as u32;
        self.set_timeout(mins);
    }
}

impl Drop for IdleMonitor {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

impl idle_api::IdleSource for IdleMonitor {
    fn is_available() -> bool {
        IdleMonitor::is_available()
    }

    fn new(timeout: Duration) -> Option<Self> {
        IdleMonitor::new_timeout(timeout)
    }

    fn is_idle(&self) -> bool {
        IdleMonitor::is_idle(self)
    }

    fn is_alive(&self) -> bool {
        IdleMonitor::is_alive(self)
    }

    fn set_timeout(&self, timeout: Duration) {
        IdleMonitor::set_timeout_duration(self, timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_starts_unavailable_without_wayland() {
        let _lock = crate::get_test_mutex().lock().unwrap();
        let backup = std::env::var("WAYLAND_DISPLAY").ok();
        unsafe {
            std::env::remove_var("WAYLAND_DISPLAY");
        }
        assert!(!IdleMonitor::is_available());
        assert!(IdleMonitor::new(5).is_none());
        if let Some(val) = backup {
            unsafe {
                std::env::set_var("WAYLAND_DISPLAY", val);
            }
        }
    }

    #[test]
    fn monitor_is_available_matches_env() {
        let _lock = crate::get_test_mutex().lock().unwrap();
        let backup = std::env::var("WAYLAND_DISPLAY").ok();
        unsafe {
            std::env::set_var("WAYLAND_DISPLAY", "wayland-mock-monitor-0");
        }
        assert!(IdleMonitor::is_available());
        unsafe {
            std::env::remove_var("WAYLAND_DISPLAY");
        }
        assert!(!IdleMonitor::is_available());
        if let Some(val) = backup {
            unsafe {
                std::env::set_var("WAYLAND_DISPLAY", val);
            }
        }
    }
}
