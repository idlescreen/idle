// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use zbus::zvariant::OwnedValue;

use crate::status::DaemonStatus;
use crate::{OBJECT_PATH, SERVICE_NAME};

#[zbus::proxy(
    interface = "io.github.idlescreen.Idle",
    default_service = "io.github.idlescreen.Idle",
    default_path = "/io/github/idlescreen/Idle",
    gen_blocking = true
)]
trait Idle {
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

/// Blocking D-Bus client for the IdleScreen daemon.
pub struct TranceClient {
    connection: zbus::blocking::Connection,
}

impl TranceClient {
    pub fn connect() -> zbus::Result<Self> {
        let connection = zbus::blocking::Connection::session()?;
        let proxy = IdleProxyBlocking::new(&connection)?;
        proxy.get_status()?;
        Ok(Self { connection })
    }

    pub fn get_status(&self) -> zbus::Result<DaemonStatus> {
        let map = IdleProxyBlocking::new(&self.connection)?.get_status()?;
        parse_status(map)
    }

    pub fn enable(&self) -> zbus::Result<()> {
        IdleProxyBlocking::new(&self.connection)?.enable()
    }

    pub fn disable(&self) -> zbus::Result<()> {
        IdleProxyBlocking::new(&self.connection)?.disable()
    }

    pub fn set_timeout(&self, minutes: u32) -> zbus::Result<()> {
        IdleProxyBlocking::new(&self.connection)?.set_timeout(minutes)
    }

    pub fn set_saver(&self, name: &str) -> zbus::Result<()> {
        IdleProxyBlocking::new(&self.connection)?.set_saver(name)
    }

    pub fn list_savers(&self) -> zbus::Result<Vec<String>> {
        IdleProxyBlocking::new(&self.connection)?.list_savers()
    }

    pub fn preview(&self, name: &str) -> zbus::Result<()> {
        IdleProxyBlocking::new(&self.connection)?.preview(name)
    }

    pub fn stop_preview(&self) -> zbus::Result<()> {
        IdleProxyBlocking::new(&self.connection)?.stop_preview()
    }

    pub fn inhibit(&self, application: &str, reason: &str) -> zbus::Result<u32> {
        IdleProxyBlocking::new(&self.connection)?.inhibit(application, reason)
    }

    pub fn un_inhibit(&self, cookie: u32) -> zbus::Result<()> {
        IdleProxyBlocking::new(&self.connection)?.un_inhibit(cookie)
    }

    pub fn list_inhibitors(&self) -> zbus::Result<Vec<(u32, String, String)>> {
        IdleProxyBlocking::new(&self.connection)?.list_inhibitors()
    }

    pub fn set_gpu_enabled(&self, enabled: bool) -> zbus::Result<()> {
        IdleProxyBlocking::new(&self.connection)?.set_gpu_enabled(enabled)
    }

    pub fn set_show_fps_overlay(&self, enabled: bool) -> zbus::Result<()> {
        IdleProxyBlocking::new(&self.connection)?.set_show_fps_overlay(enabled)
    }

    pub fn set_render_scale(&self, scale: f32) -> zbus::Result<()> {
        IdleProxyBlocking::new(&self.connection)?.set_render_scale(f64::from(scale))
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

/// Returns whether the IdleScreen daemon is reachable on the session bus.
pub fn daemon_available() -> bool {
    let connection = match zbus::blocking::Connection::session() {
        Ok(connection) => connection,
        Err(_) => return false,
    };
    let dbus = match zbus::blocking::fdo::DBusProxy::new(&connection) {
        Ok(dbus) => dbus,
        Err(_) => return false,
    };

    if let Ok(bus) = zbus::names::BusName::try_from(SERVICE_NAME)
        && dbus.name_has_owner(bus).unwrap_or(false)
    {
        return true;
    }
    let _ = OBJECT_PATH;
    false
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
