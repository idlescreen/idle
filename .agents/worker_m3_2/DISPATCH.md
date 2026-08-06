## 2026-08-06T07:22:54Z
Task: Implement 2 specific fixes for Milestone 3 (openOODA Module Extraction) in `idle-daemon`:
1. In `idle/idle-daemon/src/ooda/act/mod.rs`, update `OodaActor::execute` to consume and execute `decision: PresentationDecision` directly rather than ignoring `_decision` and delegating to `update_presentation_state`.
2. In `idle/idle-daemon/src/ooda/decide/mod.rs`, update `OodaDecisionEngine::decide` to pass live `current_saver` string into `IdlePolicyInput.current_saver` instead of `config.active_saver`.
