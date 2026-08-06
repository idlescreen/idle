## 2026-08-06T07:30:35Z

Task: Fix the Asynchronous Sticky Preview State Desync Bug flagged by challenger_m3_4 in `idle-daemon`:
1. In `idle/idle-daemon/src/presentation/mod.rs` and `idle/idle-daemon/src/ooda/act/mod.rs`:
   - `PluginPresentation::start` spawns a background thread that may fail or exit asynchronously when given an invalid saver name or missing binary.
   - Update `ActivePresentation::is_active` or add liveness detection (e.g. checking `thread.as_ref().map_or(false, |t| !t.is_finished())` or clearing `preview_name` when thread finishes) so exited presentation threads transition `ActivePresentation` to `None` and clear `preview_name`.
   - Optionally perform pre-flight saver binary / plugin validation in `start_presentation` before spawning the thread so `start_presentation` returns `false` synchronously if the saver plugin cannot be started.
   - Ensure `ooda::act::tests::test_ooda_actor_failed_preview_clears_preview_state` (and all other tests) pass cleanly.

MANDATORY INTEGRITY WARNING: DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_preview_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Requirements & Verification:
- Run `cargo test --workspace` from `/home/jeryd/Projects/idlescreen/idle` to ensure 100% tests pass.
- Run `bash scripts/check_file_lines.sh` from `/home/jeryd/Projects/idlescreen` to ensure 0 files > 250 lines.
- Run `bash scripts/validate_state_alignment.sh` from `/home/jeryd/Projects/idlescreen` to ensure state validation passes.
- Write handoff report to `/home/jeryd/Projects/idlescreen/.agents/worker_m3_3/handoff.md` and `progress.md` in your working directory.
- Send a message to parent (`a435a8b8-5564-4870-9ad8-967af452de96` / current orchestrator) reporting completion and handoff path.
