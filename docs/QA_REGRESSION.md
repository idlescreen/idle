# QA regression checklist (morning issues)

Manual / integration checks that unit tests cannot fully cover. Run after
daemon/cli upgrades or before a release cut.

## Doctor honesty

| ID | Steps | Pass criteria |
|----|--------|----------------|
| D1 | `systemctl --user stop idle-daemon` then `idlescreen doctor` | Exit ≠ 0; **FAIL** D-Bus + systemd + inhibitor; **not** “ALL SYSTEMS NOMINAL” |
| D2 | Start daemon; force inhibit (play media with MPRIS, or hold logind idle) then `idlescreen doctor` | **FAIL** Inhibitor Status; not NOMINAL |
| D3 | Clear inhibit; daemon active; idle enabled | Can reach NOMINAL only if uninhibited |
| D4 | `idlescreen disable` then doctor | **FAIL** D-Bus (idle DISABLED) |

## Inhibitors list

| ID | Steps | Pass criteria |
|----|--------|----------------|
| I1 | While Grok/agent holds logind idle: `idlescreen inhibitors` | Shows `logind:…` and reason text (e.g. agent turn) |
| I2 | Play MPRIS media: `idlescreen inhibitors` | Shows `mpris:…` with Playing |
| I3 | No external blocks, no cookies | `inhibited: false` + No active inhibitors |
| I4 | `status` inhibited=true ⇔ inhibitors list non-empty **or** honest “unknown external” |

## Preview / TUI `p`

| ID | Steps | Pass criteria |
|----|--------|----------------|
| P1 | Uninhibited: `idlescreen preview beams` | Overlay appears; journal has **no** `invalid shm name` |
| P2 | TUI: select saver, press `p` | Same as P1 |
| P3 | Inhibited: preview may clear (current policy); inhibitors must name the block |
| P4 | After stop: `idlescreen stop`; preview_active false |

## Packaging / dual icons / applet

| ID | Steps | Pass criteria |
|----|--------|----------------|
| A1 | `which -a idlescreen-applet` | Prefer `/usr/bin`; no stale `~/.local/bin` with trance bus |
| A2 | Packaged applet strings / bus | `io.github.idlescreen.Idle` only for control plane |
| A3 | App menu | Single IdleScreen launcher (tui); CosmicApplet `NoDisplay` |
| A4 | No orphan `com.system76.CosmicAppletIdle.desktop` |

## Channel upgrade

| ID | Steps | Pass criteria |
|----|--------|----------------|
| U1 | Ship signed RPM; `dnf list idle-daemon` | Available version matches ship |
| U2 | `install.sh` or `dnf upgrade idle-daemon` | Installed version rises; service restarts |
| U3 | `rpm -K` on pool package | `signatures OK` |

## Automated coverage map

| Issue | Code tests |
|-------|------------|
| Doctor false NOMINAL | `idle-cli` `doctor_rules::tests` |
| Inhibitors empty vs inhibited | `idle-cli` `inhibitors_fmt::tests`; `idle-daemon` `inhibit::tests` merge |
| Preview shm invalid | `idle-ipc` `path_safety` unit + proptest |
| Inhibit clears preview | `idle-daemon` `idle_decision_tests` |
| Job file (render) | `render` `job_spec::tests` |

Run unit suite:

```bash
cd idle
cargo test -p idle-cli -p idle-daemon -p idle-ipc
```
