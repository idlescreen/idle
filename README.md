# idle — the IdleScreen engine

A Wayland screensaver for Linux. We don't lock, dim, or authenticate — we
render content while you're idle and yield back cleanly on input.

**Website:** [idlescreen.github.io](https://idlescreen.github.io) ·
**Packages:** signed APT/RPM channel via the
[`packages`](https://github.com/idlescreen/packages) repo

## What's in this repo

This is the runtime workspace — the engine, daemon, and plugin host.

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

The CLI lives in [`cli`](https://github.com/idlescreen/cli), the router in
[`idlescreen`](https://github.com/idlescreen/idlescreen), savers in
[`savers`](https://github.com/idlescreen/savers), the TUI in `tui`, the
COSMIC applet in `cosmic`, and the render engine + studio TUI in
[`studio`](https://github.com/idlescreen/studio).

## Using it

```sh
idlescreen status          # daemon state, active saver, inhibitors
idlescreen preview hearth  # fullscreen preview
idlescreen saver set beams # pick a saver
idlescreen timeout 10      # idle timeout in minutes
idlescreen tui             # interactive console
idlescreen doctor          # diagnostics
```

Config lives at `~/.config/idle/config.yaml` — edits hot-reload, comments
and unknown keys survive saves, and `.bak` snapshots precede every write.

## Process kit

Root `*.md` files are the working kit, not user docs — load together:
`DESIGN.md` (product contract), `RULES.md` (axioms).
`DEPLOYMENT.md` covers packaging/release ops.
Hygiene, chaos, and QA scripts live in `scripts/`.
