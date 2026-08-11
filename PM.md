# PM.md — IdleScreen project management

**Sources:** `DESIGN.md` is the vision. PM.md lists what is done and
what is not.

**Last pass:** 2026-08-10 (Sprint 04 close-out: GPU budget, render-loop watchdog, signature verification, presentation refactor onto `Arc<dyn OverlaySurface>`, OODA sweep including F-101 frame-loop panic fix, F-102 ABI version required, F-202 audit log JSON escape. Mac/Windows shims remain out of scope per user.)

---

## What ships today — done / not

| Product / capability | Status | Latest | Date |
|----------------------|--------|--------|------|
| `idle-daemon` (Linux) | done | v3.1.0 | 2026-08-10 |
| `idle-cli`, `idle-runner`, `idle-api` | done | bundled (v3.1.0) | 2026-08-10 |
| savers (`idle-saver-*` × 10) | done | v2.1.0 | 2026-08-10 |
| `idle-tui` (Linux) | done | v3.0.1 | — |
| `idle-cosmic` (applet) | done | v3.0.2 | — |
| `idle-studio`, `render` | done | studio v0.3.4 / engine v1.1.0 | 2026-08-10 |
| Channel (RPM / DEB / Arch) | done | v4.0.5 | 2026-08-10 |
| Engine (platform-agnostic core) | partial | — | Linux-only today |
| macOS shim | **not** | — | — |
| Windows shim | **not** | — | — |
| WASM plugin host | **deferred** (DECISION-WASM-01 = Option C) | — | — |
| Capability-declaring plugin manifest | **done** | manifest host + 10/10 savers + audit log | 2026-08-10 |
| Headless render mode (PNG/MP4/stdout) | **done** | merged to render/master (0b7a11c) | 2026-08-10 |
| Homebrew / scoop / winget / MSI channels | **deferred** (DECISION-CHANNEL-MORE = Option A) | — | — |
| Audio capture capability gate | **done** | per-capability opt-in (`IDLE_PERMIT_AUDIO_CAPTURE` / `_OUTPUT`); profile-aware network gate | 2026-08-10 |
| Per-saver GPU/CPU budget enforcement | **done** | CPU via cgroup v2 done; GPU via 3 vendor backends (nvidia-smi / intel_gpu_top / amdgpu_top) with health watchdog | 2026-08-10 |
| Watchdog on render loop + per-plugin | **done** | per-plugin tick watchdog (wall-clock); render-loop heartbeat (AtomicU64) → AtomicBool escalation; IPC timeout → `kill_child()` on hung saver | 2026-08-10 |
| GPG manifest signature verification | **done** (opt-in) | `IDLE_REQUIRE_MANIFEST_SIGNATURE=1` → refuse unsig'd; `~/.config/idle/trusted-keys.d/` keyring | 2026-08-10 |
| Subprocess plugin isolation | **done** | IPC child + SIGKILL on hang; `kill_child()` idempotent | 2026-08-10 |
| Install-time audit log (F-202 escape fix) | **done** | JSONL `~/.config/idle/install-audit.jsonl`; full RFC 8259 control-char escape | 2026-08-10 |
| macOS / Windows sandbox (Seatbelt / AppContainer) | **not** | — | out of scope per user |
| Multi-platform idle detection (IOKit, GetLastInputInfo) | **not** | — | out of scope per user |
| ABI version enforcement (F-102) | **done** | loader requires `idle_api_version` symbol; 10/10 savers export it | 2026-08-10 |
| Frame-loop panic fix (F-101) | **done** | `saturating_sub` on `frame_duration - elapsed` at 2 callsites | 2026-08-10 |

## Channel — last cut vs Pages

- Last `packages` cut: **v4.0.5** (2026-08-10) — seats `idle` v3.1.0
  + 10× `idle-saver-*` v2.1.0 + the install-time plugin capability
  audit log in the public channel.
- Local pool equals Pages. **F-002 closed.** Sprint 02 manifest host
  and v4.0.5 cut re-validated the release pipeline.
- (Side effect: the prior v4.0.4 tag was cut without bumping
  `Cargo.toml` from 4.0.3 — that gap is closed by v4.0.5.)

## Privacy — open residuals

| ID | Status | One-line |
|----|--------|----------|
| F-002 (channel lag) | **closed** | v4.0.4 cut sealed 3.0.3 in Pages |
| F-007 (comm fallback) | documented residual | opt-out `IDLE_STRICT_CONTROL=1` |
| F-004 / F-005 / F-006 / F-008 | closed | resolutions in commit messages; no separate audit/ dir (deleted with the stale `audit/AUDIT-FINDINGS.md` in this session) |
| F-101 (frame-loop panic) | **closed** | `saturating_sub` fix at 2 callsites |
| F-102 (ABI version fallback) | **closed** | all 10 savers export `idle_api_version`; loader requires it |
| F-201 (install.sh SCRIPT_DIR fallback) | **closed** | fail-closed when `cd` cannot resolve |
| F-202 (audit log JSON escape) | **closed** | RFC 8259 control-char escape |
| F-203 (idle-tui comm spoof) | noted residual | narrow attack surface (daemon `O_NOFOLLOW` pidfile); combined mitigation deferred |

## Recent releases

- **v4.0.5** · 2026-08-10 · `packages` channel · seats `idle` v3.1.0 + 10× `idle-saver-*` v2.1.0 + install-time audit log
- **v3.1.0** · 2026-08-10 · `idle` capability-declaring plugin manifest host (DECISION-MANIFEST-01 Option A)
- **idle-saver-* v2.1.0** · 2026-08-10 · 10 savers · `.idleplugin.toml` + F9 clippy sweep (where applicable)
- **render engine v1.1.0** · 2026-08-10 · FOLLOWUP-7 env-var gate + FOLLOWUP-8 refactor
- **render studio v0.3.4** · 2026-08-10 · rebuilt for engine v1.1.0 surface
- **v4.0.4** · 2026-08-10 · `packages` channel · idlescreen + idle-daemon 3.0.3
- **v3.0.3** · 2026-08-10 · `idle` audit-harden (F-009, F-012, F-016)
- **v3.0.2** · 2026-08-07 · `idle` D-Bus activation
- **v4.0.3** · packages channel · idlescreen + idle-cosmic 3.0.2

## Decisions made

- 2026-08-10 · **DECISION-MAC-01 = Option A** (engine-on-Mac first).
  Folded into Sprint 01 task 2 (headless render). Engine-on-Mac link
  rides with the headless-render testability primitive; full shim
  (`IOKit` + `NSWindow` + sandbox + IPC) deferred. Open follow-up
  questions: macOS version floor 13 (Ventura) recommended; Apple-
  Silicon-only for v1; user-agent only (no signing this turn).

- 2026-08-10 · **DECISION-WIN-01 = Option A** (engine-on-Windows first).
  Same rationale as MAC-01. Folded into Sprint 01 task 2. Full shim
  (`GetLastInputInfo` + DXGI/D3D + AppContainer + named-pipes) deferred.
  Open follow-up questions: Win10 21H2 vs Win11 floor; classical
  `.scr` vs UWP/.appx; sandbox choice (AppContainer / AppLocker /
  none); signing cert (EV / OV / none).

- 2026-08-10 · **DECISION-MANIFEST-01 = Option A** (`.idleplugin.toml`,
  TOML sibling manifest). Decoupled from artifact so WASM plugins
  use the same format. (Note: the manifest work that landed added
  `toml = "0.8"` and `serde = { version = "1", features = ["derive"] }`
  as direct workspace deps in `idle/Cargo.toml`; the prior claim
  that they were already in the dep tree was wrong — they were
  transitive-only or absent.) Schema carries `plugin.id` (reverse-DNS),
  `entry.runtime = "native" | "wasm"`, declared capabilities
  (network / audio_capture / audio_output / filesystem_read /
  filesystem_write), `sandbox.profile` (named; host maps per-platform),
  `dependencies`, `headless_render`
  knobs. Migration work: `idle-api` gains `pub mod plugin_manifest`,
  loader reads it before `libloading::Library::new`, install-time
  audit log, all 10 `idle-saver-*` get a `.idleplugin.toml`. Schema
  sketch captured in the Sprint 01 task 5 memo.

## Open decisions

- `DECISION-WASM-01`: see Decisions made (resolved 2026-08-10, Option C).
- `DECISION-CHANNEL-MORE`: see Decisions made (resolved 2026-08-10, Option A).

## Decisions made

- (continued)

- 2026-08-10 · **DECISION-WASM-01 = Option C** (defer WASM indefinitely;
  stay on native `cdylib` only). Schema declares `runtime = "wasm"`
  parseable so the format is forward-compat; loader returns
  `PluginError::RuntimeUnsupported` when "wasm" is encountered today.
  Triggers to re-open: (a) non-Linux shim ships, (b) third-party
  plugin author wants non-Rust, (c) install audit log shows plugins we
  don't maintain. Reversible cheaply — the format seam stays.

- 2026-08-10 · **DECISION-CHANNEL-MORE = Option A** for Sprint 02
  (Linux-only ships; macOS / Windows channels deferred). After
  headless-render proves the engine-on-Mac/Win link,
  re-open this memo and follow Option B: Homebrew tap → scoop →
  winget → MSI, gated on the actual binary existing for each
  platform. FORMULA-EMPTY-OPTION-C rejected — a formula pointing at
  a non-existent binary is the half-truth DESIGN §"Done bars" rail
  against.

## Follow-up tickets

- **`FOLLOWUP-1` — `idle/package.rs` requires `unsafe { set_var }`
  under current nightly toolchain.** **Resolved 2026-08-10** by commit
  `idle@c22b646` (patched `unsafe { … }` around the call, push to
  origin master landed). Close-out: idle v3.0.4 channel cut will
  re-pick-up the package.rs path naturally.

- **`FOLLOWUP-2` — PM.md claim "`toml` + `serde` already in dep tree`
  is wrong.** `toml` is not in `idle/Cargo.toml` at all (workspace or
  member); `serde` is transitive-only via `zbus`, not a direct
  dependency. The manifest spec adds both as workspace dependencies.
  Caught by the manifest spec subagent against `idle/Cargo.toml` and
  `idle/idle-api/Cargo.toml`. Fix this PM.md assertion when the
  manifest work lands, or now (one-line correction).

- **`FOLLOWUP-3` — `idle/idle-runner/src/apps/mod.rs` does not contain
  a `render` subcommand.** PM.md note flagged this; the headless
  render spec subagent confirmed: apps/mod.rs is 54 lines of identity
  helpers (username, hostname, refresh_rate_hz). The actual headless
  backend lives in the **sibling `render/` repo** at
  `render/engine/src/pipeline.rs`. Sprint 02 headless render work
  lives in `render/`, not in `idle/`. (Idle-side surface is just the
  `CellRenderer::disable_gpu()` plumb-through + the cross-platform
  contract test.)

- **`FOLLOWUP-4` — Wave 3 challenger found `run_plugin_fullscreen`
  bypassed the manifest gate at `idle-runner/src/idle_runner.rs:96`.**
  **Resolved 2026-08-10** by commit `idle@2b83cd1` (Merge
  feature/sprint-02-manifest-runner-fix). Merge order was enforced:
  manifest host landed first (`idle@72dc64b`), then runner-fix.

- **`FOLLOWUP-5` — Merge the 10 per-saver `.idleplugin.toml` branches.**
  **Resolved 2026-08-10.** All 10 merged to their respective masters
  (5 originally-correct: gnats/hearth/radar/ripple/storm; 5 amended:
  beams/bursts/chaos/cosmos/glyphs). Manifests re-verified after merge.

- **`FOLLOWUP-6` — After idle merges, cut `packages` v4.0.5 to seat the
  new manifests + audit log in the public channel.** **Resolved
  2026-08-10.** `packages@be78cbd` (v4.0.5) carries the audit log
  (`install_audit.sh`, 167 lines, line-lock compliant) and seats
  idle v3.1.0 + 10× saver v2.1.0 in the public channel.

- **`FOLLOWUP-7` — `render` `--update-baselines` flag silently rewrites
  CI baselines with no env-var gate (Wave 3 reviewer).** **Resolved
  2026-08-10** by commit `render@679f6e7` (gated behind
  `RENDER_FORCE_UPDATE_BASELINES=1` in `models.rs::validate()`; merged
  to render/master @ `2b6355b`). Fail-closed: validate() rejects the
  flag unless the env var is exactly `"1"`. Tests cover all 4 cases.

- **`FOLLOWUP-8` — `render/encode/src/encode_select.rs` is at exactly
  the 256-line cap.** **Resolved.** Commit `render@aa6ecf4` extracts
  `probe_encoder` / `detect_av1_encoder` / `detect_h264_encoder` /
  `push_quality_args` / `push_h264_quality_args` / `probe_quality_args`
  into `engine/src/encoder_probe.rs` (214 lines). `encode_select.rs`
  is now 76 lines (re-exports + candidate constants only).

- **`FOLLOWUP-9` — `idle-saver-*` packages have pre-existing
  `clippy::too_many_arguments` errors on master.** **Resolved
  2026-08-10.** Sweep applied to beams (on manifest branch), gnats,
  hearth, radar, ripple (on F9 branches, then merged), and chaos /
  cosmos (inline commits during merge). bursts, glyphs, storm unaffected
  (no `too_many_arguments` warning on master). All `#[allow]` blocks
  are scoped to single functions and noted with a refactor-to-context-
  struct comment for Sprint 03 housekeeping.

## Process

- PM.md mirrors DESIGN items with current status.
- Update on tag push: add a "Recent releases" entry.
- Update on DESIGN change: mirror the new section.
- "Last pass" date at the top updates whenever this file changes.
