// SPDX-License-Identifier: MIT

//! Idle inhibitors: IdleScreen cookies + external (logind idle / MPRIS).

mod external;

use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use zbus::names::UniqueName;

use external::list_external;

#[derive(Debug, Clone)]
pub struct Inhibitor {
    pub cookie: u32,
    #[allow(dead_code)]
    pub application_name: String,
    #[allow(dead_code)]
    pub reason: String,
    pub client: UniqueName<'static>,
}

#[derive(Debug)]
pub struct InhibitorState {
    inhibitors: Mutex<Vec<Inhibitor>>,
    last_cookie: AtomicU32,
    #[cfg(not(test))]
    logind_cache: Mutex<(bool, std::time::Instant)>,
}

impl InhibitorState {
    pub fn new() -> Self {
        Self {
            inhibitors: Mutex::new(Vec::new()),
            last_cookie: AtomicU32::new(0),
            #[cfg(not(test))]
            logind_cache: Mutex::new((
                false,
                std::time::Instant::now()
                    .checked_sub(std::time::Duration::from_secs(5))
                    .unwrap_or_else(std::time::Instant::now),
            )),
        }
    }

    pub fn is_inhibited(&self) -> bool {
        if !self
            .inhibitors
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty()
        {
            return true;
        }

        #[cfg(test)]
        {
            false
        }
        #[cfg(not(test))]
        {
            let mut cache = self.logind_cache.lock().unwrap_or_else(|e| e.into_inner());
            if cache.1.elapsed() >= std::time::Duration::from_secs(2) {
                cache.0 = check_logind_inhibited() || check_mpris_playing();
                cache.1 = std::time::Instant::now();
            }
            cache.0
        }
    }

    pub fn add(
        &self,
        application_name: String,
        reason: String,
        client: UniqueName<'static>,
    ) -> Result<u32, &'static str> {
        let mut inhibitors = self.inhibitors.lock().unwrap_or_else(|e| e.into_inner());
        let count = inhibitors
            .iter()
            .filter(|entry| entry.client == client)
            .count();
        if count >= 32 {
            return Err("too many concurrent inhibitors for this client");
        }
        let cookie = self.last_cookie.fetch_add(1, Ordering::Relaxed) + 1;
        inhibitors.push(Inhibitor {
            cookie,
            application_name,
            reason,
            client,
        });
        Ok(cookie)
    }

    pub fn add_with_cookie(
        &self,
        application_name: String,
        reason: String,
        client: UniqueName<'static>,
        cookie: u32,
    ) {
        let mut inhibitors = self.inhibitors.lock().unwrap_or_else(|e| e.into_inner());
        if inhibitors
            .iter()
            .any(|entry| entry.cookie == cookie && entry.client == client)
        {
            return;
        }
        tracing::info!(
            "Adding external inhibitor for client {} (cookie {}): {}",
            client,
            cookie,
            application_name
        );
        inhibitors.push(Inhibitor {
            cookie,
            application_name,
            reason,
            client,
        });
    }

    /// Remove an inhibitor only when `cookie` belongs to `client`.
    pub fn remove_for_client(&self, cookie: u32, client: &UniqueName<'_>) -> bool {
        let mut inhibitors = self.inhibitors.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(index) = inhibitors
            .iter()
            .position(|entry| entry.cookie == cookie && entry.client == *client)
        {
            inhibitors.remove(index);
            true
        } else {
            false
        }
    }

    pub fn remove_client(&self, client: &UniqueName<'_>) {
        let mut inhibitors = self.inhibitors.lock().unwrap_or_else(|e| e.into_inner());
        inhibitors.retain(|entry| entry.client != *client);
    }

    /// IdleScreen cookies only (D-Bus UnInhibit targets these).
    pub fn list(&self) -> Vec<(u32, String, String)> {
        let inhibitors = self.inhibitors.lock().unwrap_or_else(|e| e.into_inner());
        inhibitors
            .iter()
            .map(|entry| {
                (
                    entry.cookie,
                    entry.application_name.clone(),
                    entry.reason.clone(),
                )
            })
            .collect()
    }

    /// Full picture for `idlescreen inhibitors`: cookies + logind idle + MPRIS.
    ///
    /// External rows use cookie `0` and `application` prefixed with source
    /// (`logind:…`, `mpris:…`) so the CLI can print them clearly.
    pub fn list_all(&self) -> Vec<(u32, String, String)> {
        let mut out = self.list();
        for ext in list_external() {
            out.push((0, format!("{}:{}", ext.source, ext.who), ext.why));
        }
        out
    }
}

#[cfg(test)]
#[path = "inhibit/tests.rs"]
mod tests;
