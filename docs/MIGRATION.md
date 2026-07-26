# Migrating to IdleScreen

## Hard cut (engine 2.5.0+)

As of idle-daemon / idle-cli **2.5.0** and idle-dbus **0.6.0**, this monorepo no
longer dual-exports legacy rebrand shims:

| Removed | Use instead |
|---------|-------------|
| D-Bus `io.github.ubermetroid.trance` / `/io/github/crateria/trance` | `io.github.idlescreen.Idle` / `/io/github/idlescreen/Idle` |
| Interface `io.github.ubermetroid.trance` | `io.github.idlescreen.Idle` |
| Env `TRANCE_*` host reads / dual-set | `IDLE_*` only |
| Binaries `trance`, `trance-daemon` | `idle` / `idlescreen`, `idle-daemon` |

Upgrade both daemon and CLI together. External clients (TUI, applet) must speak
the primary D-Bus names and interface.

For install names, keyrings, and package layout, see the packages repo:

- https://github.com/idlescreen/packages/blob/master/docs/MIGRATION.md

Local packages checkout: `../packages/docs/MIGRATION.md`.
