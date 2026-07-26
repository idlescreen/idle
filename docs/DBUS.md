# D-Bus ABI

Product control plane for IdleScreen live runtime. Primary well-known names only
(legacy dual-export removed in 2.5.0 / idle-dbus 0.6.0).

## Names

| Constant | Value |
|----------|--------|
| Service | `io.github.idlescreen.Idle` |
| Object path | `/io/github/idlescreen/Idle` |
| Interface | `io.github.idlescreen.Idle` |

Defined in `crates/idle-dbus` as `SERVICE_NAME` / `OBJECT_PATH` / `INTERFACE_NAME`.

## Clients

- `idle-cli` / `idlescreen` (`idle` binary)
- `idle-tui`
- COSMIC applet (`idle-cosmic` / `idlescreen-applet`)

## Stability

- Method and property shapes used by the above clients are **stable**.
- Adding optional methods is preferred over changing existing signatures.
- Removing or changing the interface method set is a coordinated major.

## Activation

- `usr/share/dbus-1/services/io.github.idlescreen.Idle.service`

Starts `idle-daemon` / `idle-daemon.service` (`BusName=io.github.idlescreen.Idle`).

## Boundaries

D-Bus is the **product control plane**. Display presentation uses Wayland, not
D-Bus. Plugins do not speak D-Bus.
