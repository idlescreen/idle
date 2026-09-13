# runtime

The engine — idle daemon, sandboxed plugin host, D-Bus API, and the saver
ABI every plugin builds against. Part of
[IdleScreen](https://idlescreen.github.io) — modular Wayland screensavers
for Linux.

## What's inside

| Path | Role |
|---|---|
| `idle-daemon/` | The service: idle monitoring, presentation sessions, config, D-Bus API, plugin orchestration |
| `idle-runner/` | Sandboxed plugin host: manifest + signature verification, capability gates, watchdog, cell/GPU raster |
| `idle-api/` | Plugin ABI: savers link against this (`param()`, palette/system-info callbacks) |
| `crates/wayland-idle` | `ext-idle-notify` idle detection |
| `crates/wayland-present` | `zwlr_layer_shell` presentation, output topology, overlays |
| `crates/idle-dbus` | D-Bus client helpers + systemd service lifecycle |
| `crates/idle-ipc` | Daemon↔runner wire protocol |
| `crates/idle-upscaler` | CPU frame upscaler |

## Use

Installed as `idle-daemon` — managed by `systemctl --user`. Drive it from
the [`idlescreen`](https://github.com/idlescreen/idlescreen) router:

```sh
idlescreen status          # daemon state, active saver, inhibitors
idlescreen preview hearth  # fullscreen preview
idlescreen saver set beams # pick a saver
idlescreen timeout 10      # idle timeout in minutes
idlescreen doctor          # diagnostics
```

## Develop

Standalone workspace — no sibling checkouts needed.

```sh
sudo dnf install libdbus-1-devel wayland-devel libxkbcommon-devel \
    openssl-devel libudev-devel pkgconf-pkg-config   # apt: -dev names
cargo build --workspace && cargo test --workspace
```

## License

Apache-2.0 · © 2026 IdleScreen
