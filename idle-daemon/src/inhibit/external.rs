// SPDX-License-Identifier: MIT

//! External idle blocks (logind + MPRIS) — same sources as [`super::InhibitorState::is_inhibited`].

/// One external block for CLI/status listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalInhibitor {
    /// e.g. `logind`, `mpris`
    pub source: String,
    /// Application / player name
    pub who: String,
    /// Human reason
    pub why: String,
}

#[cfg(all(target_os = "linux", not(test)))]
type LogindInhibitorInfo = (String, String, String, String, u32, u32);

/// True when logind has any inhibitor with `what` containing `idle`.
#[cfg(all(target_os = "linux", not(test)))]
pub fn check_logind_inhibited() -> bool {
    !list_logind_idle().is_empty()
}

#[cfg(all(target_os = "linux", not(test)))]
pub fn list_logind_idle() -> Vec<ExternalInhibitor> {
    let run_blocking = || {
        let Ok(conn) = zbus::blocking::Connection::system() else {
            return Vec::new();
        };
        let Ok(reply) = conn.call_method(
            Some("org.freedesktop.login1"),
            "/org/freedesktop/login1",
            Some("org.freedesktop.login1.Manager"),
            "ListInhibitors",
            &(),
        ) else {
            return Vec::new();
        };
        let Ok(inhibitors): Result<Vec<LogindInhibitorInfo>, _> = reply.body().deserialize() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (what, who, why, mode, _uid, _pid) in inhibitors {
            if what.split(':').any(|w| w == "idle") {
                out.push(ExternalInhibitor {
                    source: "logind".into(),
                    who: format!("{who} ({mode})"),
                    why,
                });
            }
        }
        out
    };

    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::task::block_in_place(run_blocking)
    } else {
        run_blocking()
    }
}

/// True when any MPRIS player reports PlaybackStatus=Playing.
#[cfg(all(target_os = "linux", not(test)))]
pub fn check_mpris_playing() -> bool {
    !list_mpris_playing().is_empty()
}

#[cfg(all(target_os = "linux", not(test)))]
pub fn list_mpris_playing() -> Vec<ExternalInhibitor> {
    let run_blocking = || {
        let Ok(conn) = zbus::blocking::Connection::session() else {
            return Vec::new();
        };
        let Ok(names_reply) = conn.call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "ListNames",
            &(),
        ) else {
            return Vec::new();
        };
        let Ok(names): Result<Vec<String>, _> = names_reply.body().deserialize() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for name in names {
            if !name.starts_with("org.mpris.MediaPlayer2.") {
                continue;
            }
            // Prefer Player path used by most apps.
            let playing = mpris_status_playing(&conn, &name, "/org/mpris/MediaPlayer2/Player")
                || mpris_status_playing(&conn, &name, "/org/mpris/MediaPlayer2");
            if playing {
                let short = name
                    .strip_prefix("org.mpris.MediaPlayer2.")
                    .unwrap_or(name.as_str());
                out.push(ExternalInhibitor {
                    source: "mpris".into(),
                    who: short.to_string(),
                    why: "PlaybackStatus=Playing".into(),
                });
            }
        }
        out
    };

    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::task::block_in_place(run_blocking)
    } else {
        run_blocking()
    }
}

#[cfg(all(target_os = "linux", not(test)))]
fn mpris_status_playing(conn: &zbus::blocking::Connection, name: &str, path: &str) -> bool {
    let Ok(prop_reply) = conn.call_method(
        Some(name),
        path,
        Some("org.freedesktop.DBus.Properties"),
        "Get",
        &("org.mpris.MediaPlayer2.Player", "PlaybackStatus"),
    ) else {
        return false;
    };
    let body = prop_reply.body();
    let Ok(val) = body.deserialize::<zbus::zvariant::Value>() else {
        return false;
    };
    match val.downcast::<String>() {
        Ok(s) => s == "Playing",
        Err(_) => false,
    }
}

#[cfg(any(not(target_os = "linux"), test))]
pub fn check_logind_inhibited() -> bool {
    false
}

#[cfg(any(not(target_os = "linux"), test))]
pub fn check_mpris_playing() -> bool {
    false
}

#[cfg(any(not(target_os = "linux"), test))]
pub fn list_logind_idle() -> Vec<ExternalInhibitor> {
    Vec::new()
}

#[cfg(any(not(target_os = "linux"), test))]
pub fn list_mpris_playing() -> Vec<ExternalInhibitor> {
    Vec::new()
}

/// All external blocks currently considered by IdleScreen.
pub fn list_external() -> Vec<ExternalInhibitor> {
    let mut out = list_logind_idle();
    out.extend(list_mpris_playing());
    out
}
