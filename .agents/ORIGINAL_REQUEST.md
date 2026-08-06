# Original User Request

## 2026-08-06T08:50:07Z

Deep and wide openOODA audit and refactoring of the idlescreen codebase to eliminate Untested Claims (U) and State Disagreements (D), and prepare the architecture for a full port to openOODA.

Working directory: /home/jeryd/Projects/idlescreen
Integrity mode: development

## Requirements

### R1. Eliminate Untested Claims (U)
Audit critical paths (e.g., IPC, plugins, Wayland rendering) for untested assumptions. Implement strict "Immune Rail" test coverage (negative selection tests) to ensure the daemon securely rejects malicious injection strings and oversized payloads over IPC/DBus.

### R2. Resolve State Disagreements (D)
Audit and fix any desyncs between `idle-cli` outputs and `idle-daemon` internal states, ensuring absolute truth across binaries.

### R3. OpenOODA Port Architecture Mapping
Evaluate the codebase for the openOODA port. Identify and extract modules that can cleanly map to openOODA semantics, establishing new structural boundaries (the team may decide how far to go with the extraction based on feasibility).

## Verification Resources
- Existing test suite: `cargo test` across all workspaces.
- Entropy tracker: `scripts/check_file_lines.sh` (all files must remain under 250 lines).
- `openOODA-adoption-plan.md` artifact from previous steps for openOODA context.

## Acceptance Criteria

### Testing & Integrity (Programmatic Verification)
- [ ] `cargo test` passes across the entire workspace (`idle-daemon`, `idle-ipc`, `idle-runner`, `wayland-idle`, `wayland-present`).
- [ ] At least 3 new negative selection tests have been added to the test suite, specifically targeting fail-closed handlers.
- [ ] `scripts/check_file_lines.sh` runs and exits with a success code (no files > 250 lines).

### State Synchronization (Agent-as-Judge)
- [ ] A test script or independent validation confirms that `idle-cli` queries return the exact same internal state represented in the `idle-daemon` at runtime.

### Port Readiness
- [ ] A markdown document or clear structural changes in the codebase are provided that map extracted idlescreen modules to openOODA semantics.
