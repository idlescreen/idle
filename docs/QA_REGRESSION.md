# QA regression checklist (host / preview)

Manual / integration checks that unit tests cannot fully cover. Run after
daemon/cli upgrades or before a release cut.

**Critical rule:** TUI **`p` must never stop `idle-daemon`**.  
`systemctl --user show idle-daemon -p NRestarts` must not increase on preview.

**Minimum shipped daemon for this checklist:** `idle-daemon >= 2.5.9`.

## Automated gates (run first)

```bash
cd idle

# Pure unit regression suite (no display required)
just qa-unit
# Named filters for this release’s fixes:
just qa-unit-named

# Live smoke against the user service (Wayland session required)
./scripts/qa_preview_smoke.sh beams
# LOOPS=3 HOLD_SECS=4 ./scripts/qa_preview_smoke.sh cosmos
```

**Packaging default:** `./package.rs` and `just package` run the **headless
package gate** (`scripts/qa_package_gate.sh` — see [`PACKAGE_GATE.md`](PACKAGE_GATE.md))
before building RPMs/DEBs. Failures abort packaging. Emergency only:
`SKIP_TESTS=1 ./package.rs`. Live smoke is **not** part of packaging (needs a
Wayland session); run `just qa-smoke` after install.

| Gate | What it catches |
|------|-----------------|
| `just qa-unit` / **package gate** | Full package unit suites for cli/daemon/ipc/wayland-present |
| `just qa-unit-named` | Doctor, inhibitors, recovery, cooldown, preview policy, fullscreen geom, EAGAIN, SHM |
| `scripts/qa_preview_smoke.sh` | Live NRestarts/MainPID, Grok not listed, multi-p, multi-saver, thrash bound, fullscreen log |
| `just package` / `./package.rs` | qa-unit then release build + deb/rpm |

## Issue → test map (must not regress)

| Issue (what broke) | Unit / pure tests | Live / manual |
|--------------------|-------------------|---------------|
| TUI `p` killed daemon | `runtime` recovery_plan `exit_process=false` (all faults) | smoke P5; journal no `Main process exited` |
| Preview died on EAGAIN | `wayland-present` `is_wayland_would_block` | smoke: `preview_active` after HOLD_SECS |
| Commit before configure | `layer_surface_configured`, zero geometry | smoke durability |
| Idle thrash after fault | `present_cooldown_after_fault`, `cooldown_as_inhibit_blocks_idle` | smoke thrash count ≤ 8 |
| Grok listed as inhibitor | `ignore_logind_idle_hold`, `merge_drops_grok` | smoke inhibitors section |
| Preview blocked by inhibit | `preview_starts_even_when_inhibited`, cooldown force preview | P3/I6 |
| Panel still visible | `exclusive_zone_for(-1)`, `render_dimensions` expand, `panel_expand_margins` | journal `fullscreen saver geometry` / 1920×1080 |
| Doctor false NOMINAL | `doctor_rules` composite inhibited / daemon down | D1–D4 |
| Empty inhibitors while blocked | `inhibitors_fmt` empty-but-inhibited, merge external | I2–I4 |
| Invalid `/idle-shm-*` | `path_safety` + proptest | P1 |
| Viewporter default on | `hw_scaling` default off | P8 |
| Bare `/usr/bin/idle` trusted | `auth_tests` | — |

## Quick smoke (5 minutes)

```bash
rpm -q idle-daemon idle-cli   # expect idle-daemon >= 2.5.9
systemctl --user is-active idle-daemon
./scripts/qa_preview_smoke.sh beams
# TUI: open idle-tui, select saver, press p — same NRestarts check
```

## Doctor honesty

| ID | Steps | Pass criteria |
|----|--------|----------------|
| D1 | `systemctl --user stop idle-daemon` then `idlescreen doctor` | Exit ≠ 0; **FAIL** D-Bus; **not** “ALL SYSTEMS NOMINAL” |
| D2 | Real media/logind inhibit (not Grok): doctor | **FAIL** Inhibitor Status |
| D3 | Clear inhibit; daemon active; idle enabled | NOMINAL only if uninhibited |
| D4 | `idlescreen disable` then doctor | **FAIL** D-Bus (idle DISABLED) |

## Inhibitors list

| ID | Steps | Pass criteria |
|----|--------|----------------|
| I1 | Grok/agent logind idle only | **Does not** list grok; not “agent turn” |
| I2 | MPRIS Playing | Shows `mpris:…` |
| I3 | No blocks | `inhibited: false` + No active inhibitors |
| I4 | Real logind idle (vlc) | Listed; blocks **idle** only |
| I5 | Forced preview while MPRIS inhibited | Preview starts; header says forced preview ignores |

## Preview / TUI `p` (critical)

| ID | Steps | Pass criteria |
|----|--------|----------------|
| P1 | `idlescreen preview beams` | Overlay; **no** `invalid shm name` |
| P2 | TUI `p` | Overlay (or flash + recovery, never process exit) |
| P3 | Grok agent holding idle | Preview **allowed**; Grok **not** listed |
| P4 | `idlescreen stop` | `preview_active` false |
| **P5** | NRestarts before/after `p` | **Unchanged** |
| **P6** | Presenter dies | `recovering without exiting` + recreated; service active |
| P7 | Journal after `p` | Prefer no fatal Wayland read error; if present still P5/P6 |
| P8 | No `IDLE_HW_VIEWPORT` | CPU scale; no HW viewport enable line |
| P9 | `p` twice | Second works; daemon active |
| **P10** | After recovery, `p` again | Starts; NRestarts unchanged |
| P11 | Three savers back-to-back | No restart |
| P12 | `./scripts/qa_preview_smoke.sh` | `QA_PREVIEW_SMOKE_PASS` |
| **P13** | Preview on COSMIC with panel | **Fullscreen** — panel covered; journal `fullscreen saver geometry` or `1920x1080` (not stuck at panel-inset height only) |
| P14 | Hold preview ≥ 3s | Still presenting or clean user dismiss; no unit failure |

### Journal watch

```bash
journalctl --user -u idle-daemon -f
# other terminal: idlescreen preview cosmos
```

**FAIL (old):**
```text
Main process exited, code=exited, status=1/FAILURE
Scheduled restart job, restart counter is at N
```

**PASS (2.5.9+):**
```text
queued preview command
starting Wayland screensaver '…' (preview)...
wayland-present: fullscreen saver geometry …
output … — 1920x1080 …
achieved … FPS
# optional: recovering without exiting (NRestarts still 0)
```

## Packaging / dual icons

| ID | Pass |
|----|------|
| A1 | `idlescreen-applet` prefers `/usr/bin` |
| A2 | Applet uses `io.github.idlescreen.Idle` |
| A3 | Single IdleScreen app launcher |
| A4 | No orphan CosmicAppletIdle desktop |

## Channel upgrade

| ID | Pass |
|----|------|
| U1–U3 | Channel has ≥ shipped; signed RPM |
| U4 | `./scripts/qa_preview_smoke.sh` after upgrade |

## Run commands

```bash
cd idle
just qa-unit
just qa-unit-named
just qa-smoke          # or ./scripts/qa_preview_smoke.sh
just qa                # units + live smoke
```

### Named unit filter (CI-friendly)

```bash
cargo test -p idle-cli -p idle-daemon -p idle-ipc -p wayland-present -- \
  doctor_rules inhibitors_fmt ignore_logind merge_drops merge_includes \
  recovery_plan present_cooldown thrash hold_idle exit_process \
  preview_starts idle_decision path_safety hw_scaling \
  frame_geometry layer_not would_block eagain exclusive_zone \
  panel_expand fullscreen_expands geom_tests
```
