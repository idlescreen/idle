# Project Plan: idlescreen openOODA Audit & Refactoring

## Phase 0: Survey & Scoping
- Spawn 3 Explorers:
  - Explorer 1 (IPC & Security): Investigate IPC handlers, Wayland rendering, malicious payload handling, negative selection tests opportunities.
  - Explorer 2 (State Management): Investigate idle-cli and idle-daemon state representations, desync points, and state query APIs.
  - Explorer 3 (Architecture & Code Quality): Investigate openOODA port mapping, workspace module boundaries, and file line counts (`scripts/check_file_lines.sh`).

## Phase 1: Milestone Decomposition (PROJECT.md)
- Synthesize Explorer reports into `PROJECT.md`.
- Establish milestone breakdown:
  - Milestone 1 (R1 Security & Negative Tests): Immune rail test coverage against malicious payloads & fail-closed logic.
  - Milestone 2 (R2 State Alignment): Fix state desyncs between idle-cli and idle-daemon.
  - Milestone 3 (R3 openOODA Mapping & Refactoring): Module extraction and line length compliance (<250 lines).
- Establish E2E / Validation plan for acceptance criteria.

## Phase 2: Execution & Gated Verification
- For each milestone:
  - Spawn Explorer for detailed fix proposal.
  - Spawn Worker for implementation.
  - Spawn 2 Reviewers for code quality & contract verification.
  - Spawn 2 Challengers for empirical test verification.
  - Spawn 1 Forensic Auditor for integrity verification.
  - Evaluate Gate (ALL criteria must pass).

## Phase 3: Final Verification & Sentinel Handoff
- Run/verify workspace `cargo test`.
- Run/verify `scripts/check_file_lines.sh`.
- Validate state synchronization.
- Verify openOODA architectural documentation.
- Deliver completion handoff to Sentinel.
