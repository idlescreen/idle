# Product closed-loop gate (session)

**Goal:** Prove the **installed, running** product still closes the loop:

```text
install/unit → bus name → Preview → healthy status → hold → Stop
                 ↘ kill plugin child → daemon stays up → re-preview
```

Unit tests and `qa_package_gate.sh` prove the **map**. This gate walks the **territory**.

## Requirements

- Graphical user session (`WAYLAND_DISPLAY`)
- `systemctl --user` and `idle-daemon.service`
- Packaged `idlescreen` + `idle-daemon` (or equivalent paths)

## Run

```bash
cd idle
./scripts/qa_product_loop.sh
# or
just qa-product
```

Optional:

```bash
SAVER=cosmos HOLD_SECS=5 MIN_DAEMON_VER=2.5.10 ./scripts/qa_product_loop.sh
```

## What it checks

| Principle | Check |
|-----------|--------|
| Install | RPM present, `/usr/bin/idle-daemon`, unit file |
| Service | `idle-daemon.service` active |
| Bus | `io.github.idlescreen.Idle` on session bus + `idlescreen status` |
| Control | Preview accepted → `preview_active` + `presentation_active` |
| Healthy | Still active after `HOLD_SECS`; NRestarts/MainPID stable |
| Stop | `preview_active` false after stop |
| Fullscreen | Journal `fullscreen saver geometry` or full-height output |
| Fault | Kill plugin children → daemon up → re-preview works |
| Inhibit | No Grok/agent-turn in `idlescreen inhibitors` |
| Doctor | Coherent report; not NOMINAL while INHIBITED |

## Relation to packaging

| Script | When |
|--------|------|
| `qa_package_gate.sh` | **Every** package build (headless) |
| `qa_product_loop.sh` | After install on a real desktop / release checklist |
| `qa_preview_smoke.sh` | Lighter live smoke (subset of product loop) |

```bash
just qa              # package gate + preview smoke
just qa-all          # package gate + full product loop
```

Do **not** require `qa_product_loop.sh` on headless CI builders without Wayland.
