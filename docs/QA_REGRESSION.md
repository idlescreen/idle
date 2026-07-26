# QA regression checklist (host / preview)

Manual / integration checks that unit tests cannot fully cover. Run after
daemon/cli upgrades or before a release cut.

**Critical rule:** TUI **`p` must never stop `idle-daemon`**.  
`systemctl --user show idle-daemon -p NRestarts` must not increase on preview.

**Minimum shipped daemon for this checklist:** `idle-daemon >= 2.5.6`.

## Automated gates (run first)

```bash
cd idle

# Pure unit regression suite (no display required)
just qa-unit
# equivalent:
# cargo test -p idle-cli -p idle-daemon -p idle-ipc -p wayland-present

# Live smoke against the user service (Wayland session required)
./scripts/qa_preview_smoke.sh beams
# LOOPS=3 HOLD_SECS=3 ./scripts/qa_preview_smoke.sh cosmos
```

| Gate | What it catches |
|------|-----------------|
| `just qa-unit` | Doctor false NOMINAL, inhibitors merge/fmt, SHM allowlist, recovery never exits, preview-while-inhibited, viewporter default off |
| `scripts/qa_preview_smoke.sh` | **P5**: NRestarts / MainPID stable across multi-preview + double-preview; no systemd process exit |

## Quick smoke (5 minutes)

```bash
# 0. Version
rpm -q idle-daemon idle-cli
# expect idle-daemon >= 2.5.6

# 1. Baseline
systemctl --user is-active idle-daemon
R0=$(systemctl --user show idle-daemon -p NRestarts --value)
echo "NRestarts=$R0"

# 2. Preview from CLI
idlescreen preview beams
sleep 3
idlescreen status | grep -E 'preview|presentation|running'
# Prefer preview_active=true while overlay is up

# 3. Daemon must not have bounced
R1=$(systemctl --user show idle-daemon -p NRestarts --value)
test "$R0" = "$R1" && echo PASS: no restart || echo FAIL: NRestarts $R0 '->' $R1
systemctl --user is-active idle-daemon   # must be active

# 4. Stop
idlescreen stop

# 5. TUI: open idle-tui, select a saver, press p — same NRestarts check
# Or: ./scripts/qa_preview_smoke.sh
```

## Doctor honesty

| ID | Steps | Pass criteria |
|----|--------|----------------|
| D1 | `systemctl --user stop idle-daemon` then `idlescreen doctor` | Exit ≠ 0; **FAIL** D-Bus + systemd + inhibitor; **not** “ALL SYSTEMS NOMINAL” |
| D2 | Start daemon; force inhibit (MPRIS or logind idle) then doctor | **FAIL** Inhibitor Status; not NOMINAL |
| D3 | Clear inhibit; daemon active; idle enabled | NOMINAL only if uninhibited |
| D4 | `idlescreen disable` then doctor | **FAIL** D-Bus (idle DISABLED) |

## Inhibitors list

| ID | Steps | Pass criteria |
|----|--------|----------------|
| I1 | While Grok/agent holds logind idle only: `idlescreen inhibitors` | **Does not** list grok; inhibited false (agent-turn ignored) |
| I2 | Play MPRIS media: `idlescreen inhibitors` | Shows `mpris:…` with Playing |
| I3 | No external blocks, no cookies | `inhibited: false` + No active inhibitors |
| I4 | `status` inhibited=true ⇔ list names sources **or** honest “unknown external” |
| I5 | Real logind idle (vlc/fullscreen) | Listed; blocks **idle** savers only |
| I6 | Forced preview while MPRIS/logind inhibited | Preview still starts; header says preview ignores |

## Preview / TUI `p` (critical)

| ID | Steps | Pass criteria |
|----|--------|----------------|
| P1 | Uninhibited: `idlescreen preview beams` | Overlay visible; **no** `invalid shm name` |
| P2 | TUI: select saver, press `p` | Overlay visible (or brief flash + recovery, never process exit) |
| P3 | Inhibited (Grok logind): still preview | Preview **allowed**; inhibitors still list the block; idle savers blocked |
| P4 | `idlescreen stop` | `preview_active` false |
| **P5** | Note `NRestarts` before/after TUI `p` | **Unchanged** — no `Main process exited` / `Failed with result` |
| **P6** | If presenter dies | Journal: `recovering without exiting` + `presenter recreated`; **service stays active** |
| P7 | Journal after `p` | Prefer **no** `failed to read Wayland events`; if present, still P5/P6 |
| P8 | Without `IDLE_HW_VIEWPORT` | No log line enabling viewporter; scaling stays CPU |
| P9 | Press `p` twice in a row | Second preview works; daemon still active |
| **P10** | After recovery (fault log), press `p` again | Second queue starts; NRestarts still unchanged |
| P11 | Preview three different savers back-to-back | No restart; stop leaves daemon active |
| P12 | `./scripts/qa_preview_smoke.sh` | Prints `QA_PREVIEW_SMOKE_PASS` |

### Journal watch recipe

```bash
# terminal A
journalctl --user -u idle-daemon -f

# terminal B — or use TUI p
idlescreen preview cosmos
```

**FAIL signals (old bug):**
```text
Error: Wayland presenter connection lost
Main process exited, code=exited, status=1/FAILURE
Scheduled restart job, restart counter is at N
```

**PASS signals (2.5.5+):**
```text
queued preview command
starting Wayland screensaver '…' (preview)...
# optional recovery without death:
wayland runtime fault — recovering without exiting the daemon
overlay presenter recreated successfully
# NRestarts unchanged
```

## Packaging / dual icons / applet

| ID | Steps | Pass criteria |
|----|--------|----------------|
| A1 | `which -a idlescreen-applet` | Prefer `/usr/bin`; no stale `~/.local/bin` with trance bus |
| A2 | Packaged applet | Speaks `io.github.idlescreen.Idle` |
| A3 | App menu | Single IdleScreen launcher (tui); CosmicApplet `NoDisplay` |
| A4 | No orphan `com.system76.CosmicAppletIdle.desktop` |

## Channel upgrade

| ID | Steps | Pass criteria |
|----|--------|----------------|
| U1 | `dnf list idle-daemon` | Available ≥ shipped |
| U2 | `dnf upgrade idle-daemon` / install.sh | Version rises; service active after |
| U3 | `rpm -K` on pool RPM | `signatures OK` |
| U4 | After upgrade: `./scripts/qa_preview_smoke.sh` | Pass before calling the cut good |

## Automated coverage map

| Issue | Code tests | Run |
|-------|------------|-----|
| Doctor false NOMINAL | `idle-cli` `doctor_rules` (+ composite inhibited) | `cargo test -p idle-cli doctor_rules` |
| Inhibitors empty vs blocked | `inhibitors_fmt` + `merge_inhibitor_rows` | `cargo test -p idle-cli inhibitors_fmt` / `cargo test -p idle-daemon inhibit` |
| Preview shm invalid | `idle-ipc` path_safety unit + proptest | `cargo test -p idle-ipc path_safety` |
| Preview while inhibited | `idle_decision` preview_starts_even_when_inhibited | `cargo test -p idle-daemon idle_decision` |
| **Presenter death ≠ process exit** | `runtime::recovery_plan` exit_process=false (all faults) | `cargo test -p idle-daemon runtime` |
| **Re-queue after recovery clear** | `idle_decision` after_preview_cleared / requeued | `cargo test -p idle-daemon idle_decision` |
| **Viewporter default off** | `hw_scaling::should_use_hw_viewport` | `cargo test -p idle-daemon hw_scaling` |
| Zero frame geometry | `wayland-present` frame_geometry_ok | `cargo test -p wayland-present geometry` |
| Bare `idle` not trusted | `auth_tests` | `cargo test -p idle-daemon auth` |
| **Live NRestarts / multi-p** | `scripts/qa_preview_smoke.sh` | `./scripts/qa_preview_smoke.sh` |

### Full unit suite

```bash
cd idle
cargo test -p idle-cli -p idle-daemon -p idle-ipc -p wayland-present
# or
just qa-unit
```

### Post-release one-liner

```bash
R0=$(systemctl --user show idle-daemon -p NRestarts --value)
idlescreen preview beams; sleep 2
idlescreen stop
R1=$(systemctl --user show idle-daemon -p NRestarts --value)
test "$R0" = "$R1" && systemctl --user is-active --quiet idle-daemon && echo QA_P5_PASS || echo QA_P5_FAIL
```

### Preferred post-upgrade

```bash
cd ~/Jeryd/Documents/Workspace/idlescreen/idle
just qa-unit && ./scripts/qa_preview_smoke.sh
```
