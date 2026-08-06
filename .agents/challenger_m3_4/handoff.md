# Handoff Report — challenger_m3_4

**Verdict**: REQUEST_CHANGES

## 1. Observation

- Executed `cargo test --workspace` from `/home/jeryd/Projects/idlescreen/idle`.
- Executed `bash scripts/check_file_lines.sh` from `/home/jeryd/Projects/idlescreen` (Exit code 0; all files under 250 lines).
- Executed `bash scripts/validate_state_alignment.sh` from `/home/jeryd/Projects/idlescreen` (Exit code 0; 4 integration tests passed).
- Inspected openOODA Pillars in `idle/idle-daemon/src/ooda/act/mod.rs` and `idle/idle-daemon/src/ooda/decide/mod.rs`.
- Discovered an asynchronous state desync bug in `idle/idle-daemon/src/ooda/act/mod.rs` and `idle/idle-daemon/src/presentation/mod.rs`:
  - `PluginPresentation::start` (`presentation/mod.rs:46-68`) spawns a thread via `thread::spawn` and immediately returns `Ok(Self { ... })` synchronously.
  - `start_presentation` (`presentation/mod.rs:24-59`) returns `true` synchronously whenever `PluginPresentation::start` returns `Ok(...)`.
  - In `OodaActor::execute` (`act/mod.rs:56-69`), `let started = start_presentation(...)` always evaluates `started == true`.
  - Consequently, `if !started && reason == "preview"` (`act/mod.rs:65`) is dead code.
  - If `run_plugin_loop` in the spawned thread fails (e.g. invalid saver name, missing binary, or initialization crash), the thread exits, but `presentation` remains set to `ActivePresentation::Plugin(plugin)`.
  - `ActivePresentation::is_active()` (`presentation.rs:18-20`) returns `true` (only matching `ActivePresentation::Plugin`), and `preview_name` remains `Some(...)`.
  - Subsequent openOODA ticks publish `presentation_active: true` and `preview_active: true` to canonical D-Bus state (`OodaStateManager::sync_state`), creating a sticky preview state desync where `idle status` claims a preview is active when no thread or process is running.
- Empirically demonstrated the bug via unit test `ooda::act::tests::test_ooda_actor_failed_preview_clears_preview_state`:
  ```
  thread 'ooda::act::tests::test_ooda_actor_failed_preview_clears_preview_state' panicked at idle-daemon/src/ooda/act/mod.rs:197:9:
  assertion `left == right` failed
    left: Some("nonexistent_invalid_saver_123")
   right: None
  ```

## 2. Logic Chain

1. **Premise**: `OodaActor::execute` (`act/mod.rs`) attempts to clear sticky preview state when a preview plugin fails to launch: `if !started && reason == "preview" { *preview_name = None; }`.
2. **Observation**: `start_presentation` calls `PluginPresentation::start(...)`.
3. **Observation**: `PluginPresentation::start` (`presentation/mod.rs:46-68`) spawns a background thread for `run_plugin_loop` and immediately returns `Ok(Self { ... })`.
4. **Deduction**: `PluginPresentation::start` NEVER returns an `Err` synchronously. `start_presentation` ALWAYS returns `true`.
5. **Observation**: `started` in `OodaActor::execute` is ALWAYS `true`.
6. **Deduction**: `if !started && reason == "preview"` never executes on plugin launch failure.
7. **Observation**: When `run_plugin_loop` in the spawned thread fails (e.g. invalid saver name), the thread logs an error and exits, but `ActivePresentation` stays `ActivePresentation::Plugin(plugin)`.
8. **Deduction**: `presentation.is_active()` returns `true` continuously, and `preview_name` remains `Some(...)`.
9. **Deduction**: `OodaStateManager::sync_state` continues publishing `presentation_active: true` and `preview_active: true` to D-Bus/CLI, causing a sticky preview state desync.
10. **Conclusion**: `idle-daemon` requires fixes to detect presentation thread exit / synchronous pre-flight plugin validation so failed preview launches clear `preview_name` and `ActivePresentation`.

## 3. Caveats

- Pure decision policy in `OodaDecisionEngine` (`decide/mod.rs`) is robust and correctly handles lock/inhibit/preview/idle priority ordering.
- Script constraints (`check_file_lines.sh`, `validate_state_alignment.sh`) pass cleanly.
- The bug only manifests when a screensaver plugin fails to launch or exits unexpectedly during preview execution.

## 4. Conclusion

Milestone 3 Iteration 2 openOODA refactoring is structurally sound and satisfies line count limits, but contains a CRITICAL ASYNCHRONOUS STICKY PREVIEW BUG in `idle-daemon/src/ooda/act/mod.rs` and `idle-daemon/src/presentation/mod.rs`.
Verdict: **REQUEST_CHANGES**.

Recommended remediations for worker agent:
1. Update `ActivePresentation::is_active` or add a liveness check (e.g. `thread.as_ref().map_or(false, |t| !t.is_finished())`) so exited presentation threads transition `ActivePresentation` to `None` and clear `preview_name`.
2. Alternatively, perform synchronous saver binary validation in `start_presentation` prior to spawning the thread so `start_presentation` returns `false` synchronously when launching an invalid saver.

## 5. Verification Method

To independently verify:
1. Run `cargo test --workspace` from `/home/jeryd/Projects/idlescreen/idle`.
   - Test `ooda::act::tests::test_ooda_actor_failed_preview_clears_preview_state` currently fails, reproducing the bug.
2. Run `bash scripts/check_file_lines.sh` from `/home/jeryd/Projects/idlescreen`.
3. Run `bash scripts/validate_state_alignment.sh` from `/home/jeryd/Projects/idlescreen`.
