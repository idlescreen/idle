# D-Bus ABI (dual-publish migration)

Product control plane for IdleScreen live runtime. The daemon **dual-exports**
primary and legacy well-known names during the rebrand window.

## Primary (prefer)

| Constant | Value |
|----------|--------|
| Service | `io.github.idlescreen.Idle` |
| Object path | `/io/github/idlescreen/Idle` |

## Legacy (still claimed)

| Constant | Value |
|----------|--------|
| Service | `io.github.ubermetroid.trance` |
| Object path | `/io/github/crateria/trance` |

## Interface

| Constant | Value |
|----------|--------|
| Interface | `io.github.ubermetroid.trance` |

Method shapes are identical on both endpoints. Interface name stays historical
so one method set serves both bus names until a future major drops the legacy
endpoint.

Defined in `crates/idle-dbus` as `SERVICE_NAME` / `OBJECT_PATH` (primary) and
`SERVICE_NAME_LEGACY` / `OBJECT_PATH_LEGACY`.

## Clients

- `idle-cli` / `idlescreen` — tries **primary first**, then legacy
- `idle-tui`
- COSMIC applet (`idle-cosmic` / `idlescreen-applet`)

## Stability

- Method and property shapes used by the above clients are **stable**.
- Adding optional methods is preferred over changing existing signatures.
- Removing the **legacy** bus name is a coordinated major (after dual-publish window).
- Removing or changing the **interface** method set is a coordinated major.

## Activation

- `usr/share/dbus-1/services/io.github.idlescreen.Idle.service` (primary)
- `usr/share/dbus-1/services/io.github.ubermetroid.trance.service` (legacy)

Both start `idle-daemon` / `idle-daemon.service`.

## Boundaries

D-Bus is the **product control plane**. Display presentation uses Wayland, not
D-Bus. Plugins do not speak D-Bus.
