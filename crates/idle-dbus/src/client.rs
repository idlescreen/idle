// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use zbus::zvariant::OwnedValue;

use crate::status::DaemonStatus;
use crate::{OBJECT_PATH, OBJECT_PATH_LEGACY, SERVICE_NAME, SERVICE_NAME_LEGACY};

#[zbus::proxy(
    interface = "io.github.ubermetroid.trance",
    default_service = "io.github.idlescreen.Idle",
    default_path = "/io/github/idlescreen/Idle",
    gen_blocking = true
)]
trait IdlePrimary {
    fn get_status(&self) -> zbus::Result<HashMap<String, OwnedValue>>;
    fn enable(&self) -> zbus::Result<()>;
    fn disable(&self) -> zbus::Result<()>;
    fn set_timeout(&self, minutes: u32) -> zbus::Result<()>;
    fn set_saver(&self, name: &str) -> zbus::Result<()>;
    fn list_savers(&self) -> zbus::Result<Vec<String>>;
    fn preview(&self, name: &str) -> zbus::Result<()>;
    fn stop_preview(&self) -> zbus::Result<()>;
    fn inhibit(&self, application: &str, reason: &str) -> zbus::Result<u32>;
    fn un_inhibit(&self, cookie: u32) -> zbus::Result<()>;
    fn list_inhibitors(&self) -> zbus::Result<Vec<(u32, String, String)>>;
    fn set_gpu_enabled(&self, enabled: bool) -> zbus::Result<()>;
    fn set_show_fps_overlay(&self, enabled: bool) -> zbus::Result<()>;
    fn set_render_scale(&self, scale: f64) -> zbus::Result<()>;
}

#[zbus::proxy(
    interface = "io.github.ubermetroid.trance",
    default_service = "io.github.ubermetroid.trance",
    default_path = "/io/github/crateria/trance",
    gen_blocking = true
)]
trait TranceLegacy {
    fn get_status(&self) -> zbus::Result<HashMap<String, OwnedValue>>;
    fn enable(&self) -> zbus::Result<()>;
    fn disable(&self) -> zbus::Result<()>;
    fn set_timeout(&self, minutes: u32) -> zbus::Result<()>;
    fn set_saver(&self, name: &str) -> zbus::Result<()>;
    fn list_savers(&self) -> zbus::Result<Vec<String>>;
    fn preview(&self, name: &str) -> zbus::Result<()>;
    fn stop_preview(&self) -> zbus::Result<()>;
    fn inhibit(&self, application: &str, reason: &str) -> zbus::Result<u32>;
    fn un_inhibit(&self, cookie: u32) -> zbus::Result<()>;
    fn list_inhibitors(&self) -> zbus::Result<Vec<(u32, String, String)>>;
    fn set_gpu_enabled(&self, enabled: bool) -> zbus::Result<()>;
    fn set_show_fps_overlay(&self, enabled: bool) -> zbus::Result<()>;
    fn set_render_scale(&self, scale: f64) -> zbus::Result<()>;
}

#[derive(Clone, Copy)]
enum EndpointKind {
    Primary,
    Legacy,
}

/// Blocking D-Bus client for the IdleScreen daemon (primary, then legacy).
pub struct TranceClient {
    connection: zbus::blocking::Connection,
    kind: EndpointKind,
}

impl TranceClient {
    pub fn connect() -> zbus::Result<Self> {
        let connection = zbus::blocking::Connection::session()?;
        // Prefer primary well-known name; fall back to legacy for older daemons.
        let kind = if let Ok(proxy) = IdlePrimaryProxyBlocking::new(&connection)
            && proxy.get_status().is_ok()
        {
            EndpointKind::Primary
        } else {
            // Validate legacy is actually up.
            let legacy = TranceLegacyProxyBlocking::new(&connection)?;
            legacy.get_status()?;
            EndpointKind::Legacy
        };
        Ok(Self { connection, kind })
    }

    /// Which bus endpoint this client is using (`primary` or `legacy`).
    pub fn endpoint_label(&self) -> &'static str {
        match self.kind {
            EndpointKind::Primary => "primary",
            EndpointKind::Legacy => "legacy",
        }
    }

    pub fn get_status(&self) -> zbus::Result<DaemonStatus> {
        let map = match self.kind {
            EndpointKind::Primary => {
                IdlePrimaryProxyBlocking::new(&self.connection)?.get_status()?
            }
            EndpointKind::Legacy => {
                TranceLegacyProxyBlocking::new(&self.connection)?.get_status()?
            }
        };
        parse_status(map)
    }

    pub fn enable(&self) -> zbus::Result<()> {
        match self.kind {
            EndpointKind::Primary => IdlePrimaryProxyBlocking::new(&self.connection)?.enable(),
            EndpointKind::Legacy => TranceLegacyProxyBlocking::new(&self.connection)?.enable(),
        }
    }

    pub fn disable(&self) -> zbus::Result<()> {
        match self.kind {
            EndpointKind::Primary => IdlePrimaryProxyBlocking::new(&self.connection)?.disable(),
            EndpointKind::Legacy => TranceLegacyProxyBlocking::new(&self.connection)?.disable(),
        }
    }

    pub fn set_timeout(&self, minutes: u32) -> zbus::Result<()> {
        match self.kind {
            EndpointKind::Primary => {
                IdlePrimaryProxyBlocking::new(&self.connection)?.set_timeout(minutes)
            }
            EndpointKind::Legacy => {
                TranceLegacyProxyBlocking::new(&self.connection)?.set_timeout(minutes)
            }
        }
    }

    pub fn set_saver(&self, name: &str) -> zbus::Result<()> {
        match self.kind {
            EndpointKind::Primary => {
                IdlePrimaryProxyBlocking::new(&self.connection)?.set_saver(name)
            }
            EndpointKind::Legacy => {
                TranceLegacyProxyBlocking::new(&self.connection)?.set_saver(name)
            }
        }
    }

    pub fn list_savers(&self) -> zbus::Result<Vec<String>> {
        match self.kind {
            EndpointKind::Primary => IdlePrimaryProxyBlocking::new(&self.connection)?.list_savers(),
            EndpointKind::Legacy => TranceLegacyProxyBlocking::new(&self.connection)?.list_savers(),
        }
    }

    pub fn preview(&self, name: &str) -> zbus::Result<()> {
        match self.kind {
            EndpointKind::Primary => IdlePrimaryProxyBlocking::new(&self.connection)?.preview(name),
            EndpointKind::Legacy => TranceLegacyProxyBlocking::new(&self.connection)?.preview(name),
        }
    }

    pub fn stop_preview(&self) -> zbus::Result<()> {
        match self.kind {
            EndpointKind::Primary => {
                IdlePrimaryProxyBlocking::new(&self.connection)?.stop_preview()
            }
            EndpointKind::Legacy => {
                TranceLegacyProxyBlocking::new(&self.connection)?.stop_preview()
            }
        }
    }

    pub fn inhibit(&self, application: &str, reason: &str) -> zbus::Result<u32> {
        match self.kind {
            EndpointKind::Primary => {
                IdlePrimaryProxyBlocking::new(&self.connection)?.inhibit(application, reason)
            }
            EndpointKind::Legacy => {
                TranceLegacyProxyBlocking::new(&self.connection)?.inhibit(application, reason)
            }
        }
    }

    pub fn un_inhibit(&self, cookie: u32) -> zbus::Result<()> {
        match self.kind {
            EndpointKind::Primary => {
                IdlePrimaryProxyBlocking::new(&self.connection)?.un_inhibit(cookie)
            }
            EndpointKind::Legacy => {
                TranceLegacyProxyBlocking::new(&self.connection)?.un_inhibit(cookie)
            }
        }
    }

    pub fn list_inhibitors(&self) -> zbus::Result<Vec<(u32, String, String)>> {
        match self.kind {
            EndpointKind::Primary => {
                IdlePrimaryProxyBlocking::new(&self.connection)?.list_inhibitors()
            }
            EndpointKind::Legacy => {
                TranceLegacyProxyBlocking::new(&self.connection)?.list_inhibitors()
            }
        }
    }

    pub fn set_gpu_enabled(&self, enabled: bool) -> zbus::Result<()> {
        match self.kind {
            EndpointKind::Primary => {
                IdlePrimaryProxyBlocking::new(&self.connection)?.set_gpu_enabled(enabled)
            }
            EndpointKind::Legacy => {
                TranceLegacyProxyBlocking::new(&self.connection)?.set_gpu_enabled(enabled)
            }
        }
    }

    pub fn set_show_fps_overlay(&self, enabled: bool) -> zbus::Result<()> {
        match self.kind {
            EndpointKind::Primary => {
                IdlePrimaryProxyBlocking::new(&self.connection)?.set_show_fps_overlay(enabled)
            }
            EndpointKind::Legacy => {
                TranceLegacyProxyBlocking::new(&self.connection)?.set_show_fps_overlay(enabled)
            }
        }
    }

    pub fn set_render_scale(&self, scale: f32) -> zbus::Result<()> {
        match self.kind {
            EndpointKind::Primary => {
                IdlePrimaryProxyBlocking::new(&self.connection)?.set_render_scale(f64::from(scale))
            }
            EndpointKind::Legacy => {
                TranceLegacyProxyBlocking::new(&self.connection)?.set_render_scale(f64::from(scale))
            }
        }
    }
}

fn parse_status(map: HashMap<String, OwnedValue>) -> zbus::Result<DaemonStatus> {
    Ok(DaemonStatus {
        running: read_bool(&map, "running"),
        idle_enabled: read_bool(&map, "idle_enabled"),
        idle_timeout_mins: read_u32(&map, "idle_timeout_mins"),
        active_saver: read_string(&map, "active_saver"),
        presentation_active: read_bool(&map, "presentation_active"),
        preview_active: read_bool(&map, "preview_active"),
        system_idle: read_bool(&map, "system_idle"),
        session_locked: read_bool(&map, "session_locked"),
        inhibited: read_bool(&map, "inhibited"),
        current_saver: read_string(&map, "current_saver"),
        gpu_enabled: read_bool(&map, "gpu_enabled"),
        show_fps_overlay: read_bool(&map, "show_fps_overlay"),
        render_scale: read_string(&map, "render_scale"),
    })
}

fn read_bool(map: &HashMap<String, OwnedValue>, key: &str) -> bool {
    map.get(key)
        .and_then(|value| value.downcast_ref::<bool>().ok())
        .unwrap_or(false)
}

fn read_u32(map: &HashMap<String, OwnedValue>, key: &str) -> u32 {
    map.get(key)
        .and_then(|value| value.downcast_ref::<u32>().ok())
        .unwrap_or(0)
}

fn read_string(map: &HashMap<String, OwnedValue>, key: &str) -> String {
    map.get(key)
        .and_then(|value| value.downcast_ref::<String>().ok())
        .unwrap_or_default()
}

/// Returns whether the IdleScreen daemon is reachable (primary or legacy name).
pub fn daemon_available() -> bool {
    let connection = match zbus::blocking::Connection::session() {
        Ok(connection) => connection,
        Err(_) => return false,
    };
    let dbus = match zbus::blocking::fdo::DBusProxy::new(&connection) {
        Ok(dbus) => dbus,
        Err(_) => return false,
    };

    for name in [SERVICE_NAME, SERVICE_NAME_LEGACY] {
        if let Ok(bus) = zbus::names::BusName::try_from(name)
            && dbus.name_has_owner(bus).unwrap_or(false)
        {
            return true;
        }
    }
    let _ = (OBJECT_PATH, OBJECT_PATH_LEGACY);
    false
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
