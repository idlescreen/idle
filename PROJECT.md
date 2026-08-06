# Project: idlescreen openOODA Audit & Refactoring

## Architecture
idlescreen is a modular Wayland idle management & screensaver architecture consisting of workspace crates (`idle-daemon`, `idle-ipc`, `idle-runner`, `wayland-idle`, `wayland-present`, `idle-cli`, `idle-dbus`).

Under openOODA refactoring, `idle-daemon` is structured around 5 openOODA pillars:
- **Observe (Sensors)**: `wayland-idle` monitor, D-Bus inhibit listener, battery state monitor, config watcher, session lock monitor.
- **Orient (Situation Assessment)**: Inhibit merger (`inhibit/merge.rs`), Wayland runtime fault detection & backoff cooldown (`runtime.rs`).
- **Decide (Policy Engine)**: Pure idle decision matrix (`idle_decision.rs`), preview queue prioritization policy (`preview_queue.rs`).
- **Act (Executors)**: Wayland layer-shell presenter (`wayland-present`), OOP plugin IPC session runner (`idle-runner`), D-Bus status contract publisher (`controller/status.rs`).
- **State (Unified Store)**: `DaemonController` central state store (`controller/mod.rs`), D-Bus status contract (`status_contract.rs`).

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | R1-ImmuneRail-IPC | Enforce strict payload limits & fail-closed validation on IPC socket deserialization | M1 | ORIGINAL_REQUEST §R1 |
| 2 | R1-ImmuneRail-DBus | Reject oversized strings, malformed D-Bus method calls, and invalid parameter inputs | M1 | ORIGINAL_REQUEST §R1 |
| 3 | R1-NegativeTests | Add at least 3 negative selection unit/integration tests targeting fail-closed handlers | M1 | ORIGINAL_REQUEST §AC2 |
| 4 | R2-InhibitStateSync | Expose battery & fault cooldown inhibition in `DaemonStatus` so `idle status` matches runtime behavior | M2 | ORIGINAL_REQUEST §R2 |
| 5 | R2-PreviewStateFix | Clear `preview_name` on plugin launch errors to prevent sticky `preview_active: true` state desync | M2 | ORIGINAL_REQUEST §R2 |
| 6 | R2-CliSaverAliasSync | Standardize `"random"`, `"none"`, and `""` saver alias validation between `idle saver set` and `idle config set` | M2 | ORIGINAL_REQUEST §R2 |
| 7 | R2-DbusDoubleWriteFix | Fix dual config mutation double disk save in D-Bus handler thread vs tick loop | M2 | ORIGINAL_REQUEST §R2 |
| 8 | R2-StateValidationScript | Add independent validation script/test confirming `idle-cli` queries return exact internal state of `idle-daemon` | M2 | ORIGINAL_REQUEST §AC4 |
| 9 | R3-OODAModuleExtract | Extract `idle-daemon/src/ooda/` hierarchy (`observe/`, `orient/`, `decide/`, `act/`, `state/`) | M3 | ORIGINAL_REQUEST §R3 |
| 10 | R3-FileLengthEntropy | Decompose monolithic tick & status loops to ensure 0 files exceed 250 lines (`scripts/check_file_lines.sh`) | M3 | ORIGINAL_REQUEST §AC3 |
| 11 | R3-ArchitectureDoc | Document openOODA port mapping in `openOODA-architecture-mapping.md` | M3 | ORIGINAL_REQUEST §AC5 |
| 12 | Final-WorkspaceValidation | Full `cargo test` workspace pass, line count check, state sync check, forensic audit | M4 | ORIGINAL_REQUEST §AC1-5 |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Security & Immune Rail Tests | Fail-closed IPC/DBus validation + 3+ negative selection tests | None | DONE |
| M2 | State Alignment & CLI Validation | Resolve state desyncs (battery/cooldown, sticky preview, saver aliases, double writes) + validation script | None | DONE |
| M3 | openOODA Module Extraction | Extract `ooda/` module hierarchy, resolve file length risks (<250 lines), document architecture | M1, M2 | DONE |
| M4 | Final Integration & Gate Audit | Workspace `cargo test`, line check, state validation, and forensic integrity audit | M1, M2, M3 | DONE |

## Interface Contracts
### D-Bus / CLI ↔ DaemonController State Contract
- `DaemonStatus`: 13-field canonical status representation (`state`, `current_saver`, `presentation_active`, `inhibited`, `inhibit_reasons`, `preview_active`, `preview_saver`, `idle_seconds`, `lock_active`, `battery_active`, `cooldown_active`, etc.)
- All state queries must read atomically from `ooda/state` / `DaemonController` live status snapshot.

### IPC ↔ Daemon / Runner Contract
- Maximum payload limit: 64 KB (65,536 bytes) per IPC frame. Oversized payloads rejected immediately with `Error::PayloadTooLarge`.
- Plugin names restricted to sanitized alphanumeric/hyphen/underscore identifiers. Path traversal rejected with `Error::InvalidPluginName`.

## Code Layout
- `idle/idle-daemon/src/`: Core daemon runtime & controller
- `idle/crates/idle-ipc/src/`: IPC socket communication & SHM
- `idle/crates/idle-dbus/src/`: D-Bus interface definitions & contract structs
- `idle/idle-runner/src/`: Out-of-process screensaver plugin host
- `idle/crates/wayland-idle/src/`: Wayland ext-idle-notifier protocol client
- `idle/crates/wayland-present/src/`: Wayland layer-shell presentation overlay
- `idle/idle-cli/src/`: Command-line management tool
