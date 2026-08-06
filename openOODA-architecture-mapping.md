# openOODA Architecture Mapping: idlescreen Engine & Subsystems

**Document Version:** 1.0.0  
**Project:** `idlescreen` (Wayland Idle Management & Screensaver Architecture)  
**Target Architecture:** openOODA (Observe, Orient, Decide, Act, State)  
**Compliance Standard:** File Length Limit < 250 lines (`scripts/check_file_lines.sh`)  
**Status:** Acceptance Criterion 5 Reference Architecture Specification  

---

## 1. Executive Summary & openOODA Vision

`idlescreen` is a modular, high-reliability Wayland idle management and screensaver framework. To guarantee deterministic behavior, prevent state desynchronization between daemon runtime and CLI tools, and isolate fault domains (such as Wayland compositor crashes or plugin process exits), the `idle-daemon` crate is refactored around the **openOODA** control loop paradigm.

The openOODA architecture decomposes `idle-daemon` execution into five strict operational pillars:
1. **Observe (Sensors & Input Ingestion)**: Gathers asynchronous sensor signals (Wayland idle notifier, D-Bus inhibit requests, system power/battery status, session lock signals, config updates, CLI IPC commands).
2. **Orient (Situation Assessment & Fault Management)**: Normalizes raw inputs, merges inhibition state, tracks sub-system health, and enforces fault backoff cooldowns without terminating the daemon process.
3. **Decide (Pure Policy Engine)**: Evaluates a side-effect-free, deterministic decision matrix to compute target presentation states (Hold, Start, Stop, Preview).
4. **Act (Side-Effect Presentation & Plugin Host)**: Executes presentation state transitions via Wayland layer-shell surfaces (`wayland-present`) and out-of-process SHM plugin runners (`idle-runner`).
5. **State (Centralized Canonical State Engine)**: Provides a single atomic source of truth (`DaemonStatus`) accessible to D-Bus clients (`idle-cli`, `idle-cosmic`) and internal controllers.

---

## 2. Architectural Pillar Mapping

The table below maps `idlescreen` workspace crates and modules to the 5 openOODA pillars:

| openOODA Pillar | Architectural Responsibility | `idle-daemon` Refactored Submodule | Underlying Workspace Dependencies | Primary Data Types |
|---|---|---|---|---|
| **Observe** | Ingestion of external events, system power, lock state, idle timers, & IPC commands | `ooda/observe/mod.rs` | `crates/wayland-idle`<br>`idle-daemon/src/inhibit/external.rs`<br>`idle-daemon/src/daemon/battery.rs`<br>`idle-daemon/src/config_watcher.rs`<br>`idle-daemon/src/lock_monitor.rs` | `RawObservation`<br>`DaemonCommand`<br>`DaemonConfig` |
| **Orient** | Signal synthesis, battery policy, Wayland fault detection & backoff cooldown | `ooda/orient/mod.rs` | `idle-daemon/src/inhibit/merge.rs`<br>`idle-daemon/src/inhibit/state.rs`<br>`idle-daemon/src/daemon/runtime.rs`<br>`idle-daemon/src/daemon/recovery.rs` | `SituationAssessment`<br>`RuntimeFault`<br>`RecoveryPlan` |
| **Decide** | Deterministic state machine & preview queue policy evaluation | `ooda/decide/mod.rs` | `idle-daemon/src/daemon/idle_decision.rs`<br>`idle-daemon/src/daemon/preview_queue.rs`<br>`idle-daemon/src/failsafe/mod.rs` | `IdlePolicyInput`<br>`PresentationDecision` |
| **Act** | Layer-shell surface rendering, OOP plugin lifecycle, D-Bus contract publication | `ooda/act/mod.rs` | `crates/wayland-present`<br>`idle-runner`<br>`idle-daemon/src/presentation/ipc_session.rs`<br>`idle-daemon/src/controller/status.rs` | `ActivePresentation`<br>`OverlayPresenter`<br>`PluginSession` |
| **State** | Unified live state container & atomic D-Bus status contract snapshot | `ooda/state/mod.rs` | `idle-daemon/src/controller/mod.rs`<br>`idle-daemon/src/controller/status.rs`<br>`crates/idle-dbus/src/status_contract.rs` | `DaemonStatus`<br>`DaemonController`<br>`LiveStateSnapshot` |

---

## 3. End-to-End openOODA Data Flow

The diagram below illustrates the flow of data through the 5 openOODA phases on every tick of the daemon main loop (`MAIN_LOOP_INTERVAL = 250ms`).

```
+-------------------------------------------------------------------------------------------------------+
|                                           1. OBSERVE                                                  |
|  +---------------------+  +----------------------+  +--------------------+  +----------------------+  |
|  | Wayland Idle        |  | D-Bus Inhibitor      |  | Power / Battery    |  | Session Lock         |  |
|  | (ext-idle-notifier) |  | (org.freedesktop...) |  | (/sys/class/power) |  | (org.freedesktop...) |  |
|  +----------+----------+  +----------+-----------+  +---------+----------+  +----------+-----------+  |
|             |                        |                        |                        |              |
|             +------------------------+-----------+------------+------------------------+              |
|                                                  |                                                    |
|                                                  v                                                    |
|                                        [ RawObservation ]                                             |
+--------------------------------------------------+----------------------------------------------------+
                                                   |
                                                   v
+-------------------------------------------------------------------------------------------------------+
|                                           2. ORIENT                                                   |
|  - Check Wayland subsystem health (IdleMonitor & OverlayPresenter alive?)                             |
|  - If Fault: execute recover_runtime(), update consecutive_faults, set cooldown timer                 |
|  - Merge active inhibitors: D-Bus external + Battery policy + Presenter fault cooldown                |
|  - Reset fault streak on user activity (!system_idle)                                                 |
|                                                  |                                                    |
|                                                  v                                                    |
|                                     [ SituationAssessment ]                                           |
+--------------------------------------------------+----------------------------------------------------+
                                                   |
                                                   v
+-------------------------------------------------------------------------------------------------------+
|                                           3. DECIDE                                                   |
|  - Evaluate Pure Decision Matrix (decide_presentation):                                               |
|      * Precedence 1: Session Locked -> PresentationDecision::Stop { clear_preview: true }            |
|      * Precedence 2: Forced Preview -> PresentationDecision::Start { name, reason: "preview" }       |
|      * Precedence 3: Inhibited/Cooldown -> PresentationDecision::Stop { clear_preview: false }        |
|      * Precedence 4: System Idle -> PresentationDecision::Start { name, reason: "idle" }             |
|      * Precedence 5: Activity -> PresentationDecision::Stop { clear_preview: false }                 |
|                                                  |                                                    |
|                                                  v                                                    |
|                                     [ PresentationDecision ]                                         |
+--------------------------------------------------+----------------------------------------------------+
                                                   |
                                                   v
+-------------------------------------------------------------------------------------------------------+
|                                             4. ACT                                                    |
|  - If Start: spawn/switch wayland-present overlay surface & idle-runner OOP plugin process            |
|  - If Stop: terminate overlay presentation, free SHM buffers, clear current_saver                     |
|  - Drain IPC command queue & handle timeout reloads                                                   |
|                                                  |                                                    |
|                                                  v                                                    |
|                                       [ Presentation Transition ]                                     |
+--------------------------------------------------+----------------------------------------------------+
                                                   |
                                                   v
+-------------------------------------------------------------------------------------------------------+
|                                            5. STATE                                                   |
|  - Update DaemonControllerlive state atomically (DaemonStatus)                                       |
|  - Mark status_dirty flag -> emit D-Bus signal if changed                                            |
|  - Expose single source of truth for `idle status` / `idle-cli` queries                               |
+-------------------------------------------------------------------------------------------------------+
```

---

## 4. Subsystem Boundary Specifications & Code Hierarchy

The `idle-daemon` codebase extracts all openOODA logic into a clean module hierarchy under `idle-daemon/src/ooda/`:

```
idle/idle-daemon/src/
├── ooda/
│   ├── mod.rs                # Phase Coordinator: OodaLoopController
│   ├── observe/
│   │   └── mod.rs            # Pillar 1: OodaObserver & RawObservation
│   ├── orient/
│   │   └── mod.rs            # Pillar 2: OodaOrientator & SituationAssessment
│   ├── decide/
│   │   └── mod.rs            # Pillar 3: OodaDecisionEngine & Policy Matrix
│   ├── act/
│   │   └── mod.rs            # Pillar 4: OodaActor & Presentation Executor
│   └── state/
│       └── mod.rs            # Pillar 5: OodaStateManager & Status Contract Publisher
```

### 4.1 Locking Precedence & Atomicity Guarantees

To prevent deadlocks across multi-threaded operations (D-Bus worker thread vs daemon tick thread), state mutations in `ooda/state` observe a strict top-down locking hierarchy:

```
Config Lock (Mutex<DaemonConfig>)
  └─► Inhibitor State Lock (InhibitorState)
        └─► DaemonStatus Lock (Mutex<DaemonStatus>)
```

*Rule*: Locks are acquired strictly in order. No thread holding `DaemonStatus` may acquire `Config` or system D-Bus connections.

---

## 5. File Length Entropy Mitigation Strategy

Prior to openOODA extraction, monolithic control modules represented high line-count risk:
- `tick_loop.rs`: 240 lines (Red Zone)
- `status.rs`: 248 lines (Red Zone)

### Decomposed Module Line-Count Target Matrix

| Module File | Former / Baseline Lines | Post-Extraction Target Lines | Decomposed Responsibilities |
|---|---|---|---|
| `idle-daemon/src/daemon/tick_loop.rs` | 240 lines | ~40 lines | Outer loop framework & thread sleep timing |
| `idle-daemon/src/controller/status.rs` | 248 lines | ~45 lines | Thin Controller delegates to `ooda::state` |
| `idle-daemon/src/ooda/mod.rs` | New | ~85 lines | 5-phase loop orchestration (`step_tick`) |
| `idle-daemon/src/ooda/observe/mod.rs` | New | ~75 lines | Sensor aggregation & raw state snapshotting |
| `idle-daemon/src/ooda/orient/mod.rs` | New | ~110 lines | Fault backoff & situation normalization |
| `idle-daemon/src/ooda/decide/mod.rs` | New | ~65 lines | Pure decision policy engine invocation |
| `idle-daemon/src/ooda/act/mod.rs` | New | ~95 lines | Presentation side-effect execution |
| `idle-daemon/src/ooda/state/mod.rs` | New | ~100 lines | Atomic status application & D-Bus publishing |

Every module in the refactored architecture remains strictly under 120 lines, well below the **250-line limit** enforced by `scripts/check_file_lines.sh`.

---

## 6. Verification & Test Plan

1. **Deterministic Unit Testing (Pillar 3 Decide)**:  
   `ooda/decide` is tested purely in-memory with zero Wayland display or D-Bus daemon dependencies (`idle_decision_tests.rs`).
2. **Negative Selection & Immune Rail Tests (Pillar 1 & 4)**:  
   IPC frame limit validation (64KB) and invalid plugin string sanitization are validated in `idle-ipc` and `idle-runner`.
3. **State Alignment Verification (Pillar 5 State)**:  
   `idle-cli status` output is verified against `DaemonController` live status to ensure 100% field equality.
4. **Entropy Verification**:  
   Run `scripts/check_file_lines.sh` to confirm 0 line count violations.
