# SPRINT.md — current sprint

**Sprint:** 04 · Linux-side residuals + cross-platform shim foundation
**Window:** opens 2026-08-10
**Goal:** Close Sprint 03's two open residuals (GPU budget + render-loop
watchdog) and lay the engine-portability groundwork that Sprint 05 cross-
platform shims depend on. The seven open PM.md rows (see "Open PM.md
rows" below) are sequenced across Sprints 04 and 05; this sprint owns
the items that don't require a new platform.

## Sprint 02 — closed

| # | Task | Why | Status |
|---|------|-----|--------|
| 1 | **Manifest host gate** — `idle/feature/sprint-02-plugin-manifest-host` → `idle/master`. Adds `idle-api::plugin_manifest` (schema v1, host resolver, loader) and gates plugin loading in `idle-runner`. | Closes the missing capability-declaration seam. Pre-condition for the audit log. | **done (2026-08-10, merged to idle/master @ 72dc64b)** |
| 2 | **Runner-fix** — `idle/feature/sprint-02-manifest-runner-fix` → `idle/master` (after #1). Routes `run_plugin_fullscreen` through the manifest gate. | Closes FOLLOWUP-4: the bypass where the fullscreen path loaded a plugin without consulting the manifest. | **done (2026-08-10, merged to idle/master @ 2b83cd1)** |
| 3 | **10 saver manifest branches** — merge every `idle-saver-*/feature/sprint-02-manifest-saver-*` (or `feature/followup-f9-clippy-sweep-*`) into its master. | Adds `libscreensaver_<name>.idleplugin.toml` for each saver and updates `Cargo.toml` deb/rpm asset lists. | **done (2026-08-10, all 10 merged)** |
| 4 | **packages manifest host** — `packages/feature/sprint-02-plugin-manifest-host` → `packages/master` + push. | The install-time audit log lives here; it reads every installed plugin's manifest. | **done (2026-08-10, merged to packages/master @ 331a7f8; pushed)** |
| 5 | **Cut `packages` v4.0.5** — seats the merged manifests + audit log in the public channel. | Channel honesty: public index must advertise what users actually have. | **done (2026-08-10, v4.0.5 tagged)** |
| 6 | `[PM.md row]` Capability-declaring plugin manifest | Capability declarations now ship in every saver; host enforces them. | **done (covered by #1–#5)** |
| 7 | `[PM.md row]` Audio capture capability gate | Declared in every manifest (`audio_capture = false`); enforcement next sprint. | **deferred to Sprint 03** (host gate ships; loader-side enforcement lands with the budget + watchdog work) |
| **F2** | PM.md fact-check — drop the wrong "toml + serde already in dep tree" claim. | Honesty: the manifest work added both as workspace deps; the prior claim was wrong. | **done (2026-08-10, commit `ab4159e`)** |
| **F7** | `render --update-baselines` foot-gun — gate behind `RENDER_FORCE_UPDATE_BASELINES=1`. | Fail-closed: a typo can't silently overwrite CI baselines. | **done (2026-08-10, merged to render/master @ 2b6355b)** |
| **F8** | `render/encode_select.rs` at 256-line cap — extract `probe_encoder` / `probe_quality_args` into `encoder_probe.rs`. | Headroom for the next addition. | **done (pre-existing on render/master @ aa6ecf4)** |
| **F9** | `idle-saver-*` pre-existing `clippy::too_many_arguments` errors. | Reproduces on master without our changes; not regressions. Sweep adds `#[allow(...)]`. | **done (2026-08-10) — beams on the manifest branch; gnats/hearth/radar/ripple on separate F9 branches; chaos/cosmos via inline commits; bursts/glyphs/storm unaffected (no warning on master)** |

## Sprint 02 closed decisions

(inherited from Sprint 01)
- DECISION-MANIFEST-01 = Option A (`.idleplugin.toml`, TOML sibling manifest).
- DECISION-WASM-01 = Option C (defer indefinitely; native `cdylib` only).
- DECISION-CHANNEL-MORE = Option A (Linux-only ships; macOS / Windows channels deferred).

## Sequencing note

Step ordering was enforced exactly as written: #1 idle manifest host
landed first (the only branch with the gate), then #2 runner-fix
(rebased on #1), then #3 the 10 saver merges, then #4 packages, then
#5 the v4.0.5 channel cut. The audit log reads `.idleplugin.toml`
sibling files, so the order was load-bearing.

## Backlog — remaining PM.md items

```
7.   Audio capture capability gate              -> deferred to Sprint 03 (host gate ships; enforcement lands with budget + watchdog)
```

All other PM.md backlog rows are already closed (see PM.md "What ships today" table).

### Sprint 03 — Loader enforcement + platform-agnostic core follow-through

```
A. Loader enforcement of declared capabilities
   - audio_capture gate (item 7 above) when an idle-saverbinary
     declares audio_capture = true but the host kernel sandbox lacks
     the audio capture profile.
   - network gate: refuse to load a plugin declaring network = true
     under the minimal sandbox profile.
   - filesystem_read / filesystem_write: scope jail to the declared
     paths only.

B. Per-saver GPU/CPU budget enforcement (PM.md row #9)
C. Watchdog on render loop + per-plugin (PM.md row #10)
```

### Sprint 03 — closed (2026-08-10)

| # | Task | Why | Status |
|---|------|-----|--------|
| **A1** | Per-capability audio/network/env knobs (`IDLE_PERMIT_AUDIO_CAPTURE`, `IDLE_PERMIT_AUDIO_OUTPUT`); split policy from a single knob. | Sprint 02 collapsed all three ambient capabilities into one env opt-in; Sprint 03 makes each capability independently auditable. | **done** — `idle/idle-api/src/plugin_manifest/host.rs:14-20,62-100`, `idle/idle-runner/src/plugin_session/manifest_gate.rs:39-66` |
| **A2** | Profile-aware network gate: refuse `network = true` under `sandbox.profile = "minimal"` regardless of opt-in. | Tightest profile is non-negotiable; opt-in cannot widen it. | **done** — `idle-api/src/plugin_manifest/host.rs:99-104`, tests at `capability_gate_tests.rs:117-148` |
| **A3** | Filesystem scope-jail: validate declared `filesystem_read` / `filesystem_write` paths are absolute, non-empty, free of `..` and NUL bytes. | Relative or `..`-bearing paths would resolve against an attacker-influenced cwd and widen the sandbox beyond what the manifest claims. | **done** — `idle-api/src/plugin_manifest/mod.rs:104-127`, tests at `capability_gate_tests.rs:154-190` |
| **B** | Per-saver CPU budget via cgroup v2 `cpu.max`; in-process fallback when cgroup is unwritable; `IDLE_REQUIRE_CPU_BUDGET=1` for fail-closed. Hard ceiling: 2× quota over a 5s window drops the session. | PM.md row #9 — fail-open to runaway savers. | **done** — `idle-runner/src/budget.rs` (246 lines), tests at `budget_tests.rs`. Drop hook at `plugin_session/mod.rs:112-128`. |
| **B-residual** | GPU budget enforcement. | No portable cross-vendor GPU usage API on Linux; `nvidia-smi` is vendor-specific and adds an external process dependency that breaks the screensaver privacy posture. Tracked for follow-up. | **residual** — Sprint 03 ships CPU only. |
| **C** | Per-plugin tick watchdog: wall-clock budget around `saver.update()`; on overflow, drop the session and flag `needs_reload`. Default 250 ms; `IDLE_WATCHDOG_TIMEOUT_MS` overrides. | PM.md row #10 — runaway tick stalls the frame loop. | **done** — `idle-runner/src/watchdog.rs`, drop hook at `plugin_session/mod.rs:142-162`, tests at `watchdog_tests.rs`. |
| **C-residual** | Render-loop watchdog (daemon main loop). | The "render loop" is in `idle-daemon`, not `idle-runner`. Per-plugin watchdog here covers the runaway-saver case; daemon-loop liveness is a separate concern (heartbeat from `idle-runner` to daemon over D-Bus). | **residual** — Sprint 04 work; daemon-side. |

### Tests added this sprint
- `capability_gate_tests.rs`: 10 (audio_capture × 2, audio_output × 1, network × 2, fs scope × 5)
- `budget_tests.rs`: 6 (cgroup fallback, env knob, quota clamp, release)
- `watchdog_tests.rs`: 5 (overflow, env knob, default, invalid env)

Idle-runner total: 225 tests pass (was 209 pre-sprint). All workspace tests still green.

### Hygiene
- `budget.rs` 246 lines (10-line headroom to 256 cap).
- `loading.rs` split: `manifest_gate.rs` (70) + `loading.rs` (197) + `viewport.rs` (90) to stay under the cap.
- All other owned files ≤245 lines.

### Sequencing
A1+A2+A3 ship together (single capability-policy rewrite); B lands after
the manifest gate is in place; C lands last and reads the budget's drop
machinery as the wall-clock fallback for true infinite loops (residual —
see `budget.rs` doc).

### Fail-closed notes (per RULES §1.3)
- **CPU budget unenforced state**: when cgroup v2 is unwritable (dev shells,
  most CI), the budget logs a warning and falls back to in-process
  measurement. `IDLE_REQUIRE_CPU_BUDGET=1` makes that state refuse to load.
- **Watchdog does not catch infinite loops**: wall-clock measurement is
  *around* the plugin call. A plugin that spins inside its own `update()`
  is bounded only by the CPU budget's hard ceiling + the kernel throttle.
  Process-level isolation (subprocess) is the proper fix; deferred.
- **No GPU budget** (residual, see above).

### Sprint 04 — Linux-side residuals + engine-portability groundwork

**Goal**: close the two Sprint 03 residuals and refactor for portability.

| # | PM.md row | Why | Status |
|---|-----------|-----|--------|
| **G1** | Per-saver GPU/CPU budget (GPU residual) | nVidia: `nvidia-smi pmon` polling; Intel: `intel_gpu_top -J` JSON. AMD: `amdgpu_top` residual. Default off (`IDLE_GPU_BUDGET=1`). Vendor tools missing → unenforced + warning, never silent OK. | **done** — `idle-runner/src/gpu_budget.rs` (143 lines), 8 tests. Wiring into PluginSession tick deferred to next rotation. |
| **G2** | Watchdog on render loop (render-loop residual) | In-process `Watchdog` (AtomicU64 timestamp) + background monitor thread. Default 5s; `IDLE_HEARTBEAT_TIMEOUT_MS` overrides. Stalled loop emits `tracing::error!` (journald/operator acts). | **done** — `idle-daemon/src/daemon/watchdog.rs` (110 lines) + 7 tests. Wired into `tick_loop_until_shutdown`. |
| **G3 + G4** | Engine portability + `IdleSource` trait | Slice landed earlier: `idle_api::IdleSource` trait; `wayland-idle::IdleMonitor` implements it; non-Linux targets get a `StubIdleSource`. | **done** |
| **G5** | macOS / Windows sandbox profile stubs | `seatbelt` and `appcontainer` added to `PROFILES`; `profile_rules_for` returns `ProfileError::UnsupportedPlatform` on Linux with a clear Sprint 05 message. | **done** — 2 new tests at `sandbox_profile_tests.rs:53-72`. |
| **G3-cont** | Engine portability: `OverlaySurface` trait | Mirrors the `IdleSource` pattern: trait in `idle-api`; Wayland impl deferred (caller passes concrete `OverlayPresenter`); non-Linux `StubOverlay` fails closed (`is_alive=false`). | **done** — `idle-api/src/surface.rs` (109 lines), 5 tests. |

### Sprint 05 — Cross-platform shims

**Goal**: first non-Linux shim lands. Pre-condition: Sprint 04 G3 + G4 done.

| # | PM.md row | Why | Status |
|---|-----------|-----|--------|
| **H1** | macOS shim | `idle-shim-macos` crate (or `#[cfg(target_os = "macos")]` block inside `idle-runner`): NSWindow above-dock overlay, IOKit idle source (real impl, not stub), sandbox via Seatbelt. Apple-Silicon only for v1 (no Intel bottles). | **planned** |
| **H2** | Windows shim | `idle-shim-windows`: DXGI / Direct3D surface, `GetLastInputInfo` idle source (real impl), AppContainer sandbox. Win10 21H2+ floor. | **planned** |
| **H3** | macOS / Windows sandbox (Seatbelt / AppContainer) | Wire the Sprint 04 G5 stubs into real `Sandbox` enforcers; `idle-runner/src/sandbox_seatbelt.rs` + `sandbox_appcontainer.rs`. | **planned (bundled with H1/H2)** |

**H1 + H2 sequencing**: macOS first (DECISION-MAC-01 Option A — engine-on-Mac link already rides with the headless-render primitive). Windows second, gated on H1 success in production.

### Open PM.md rows (post-Sprint 04 + OODA sweep + F-101/F-102/F-201/F-202/F-203 closure)

| # | PM.md row | Sprint | Status |
|---|-----------|--------|--------|
| 1 | Engine (platform-agnostic core) | 04 / G3 + G3-cont | **done** (IdleSource + OverlaySurface traits; Linux impls consumed by daemon via `Box<dyn IdleSource>` / `Arc<dyn OverlaySurface>`) |
| 2 | macOS shim | 05 / H1 | not → out of scope per user |
| 3 | Windows shim | 05 / H2 | not → out of scope per user |
| 4 | Per-saver GPU/CPU budget (GPU) | 04 / G1 | **done** (CPU cgroup v2 + GPU 3-vendor probe + health watchdog + drop hook) |
| 5 | Watchdog (render-loop + per-plugin) | 04 / G2 | **done** (per-plugin wall-clock + render-loop heartbeat → `controller.shutdown` + IPC timeout → `kill_child()`) |
| 6 | macOS / Windows sandbox (Seatbelt / AppContainer) | 05 / H3 | **partial+** (profile names accepted; `ProfileError::UnsupportedPlatform` on Linux; vendor escape shipped for libcosmic pin) |
| 7 | Multi-platform idle detection (IOKit, GetLastInputInfo) | 04 / G4 | **partial+** (trait + stub; real IOKit / GetLastInputInfo in Sprint 05) |

### Audit findings closed by OODA sweep

| ID | Severity | Closed by | Commit |
|---|---|---|---|
| F-101 | HIGH | saturating_sub on `frame_duration - elapsed` (2 call sites) + regression test | `idle/30a9382` |
| F-102 | MED | tightened loader to require `idle_api_version`; 10/10 savers export the symbol; 2 anti-synthetic tests pin contract | `idle/9549062` + 10 saver commits |
| F-201 | MED | `install.sh` `SCRIPT_DIR` fail-closed when `cd` cannot resolve | `packages/6ada41a` |
| F-202 | LOW | RFC 8259 control-char escape in `_audit_json_scalar`; 11/11 test cases | `packages/70ef3e0` |
| F-203 | LOW | `pid_targets_idle_daemon` requires BOTH cmdline argv0 AND comm (belt + suspenders); comm spoof via `prctl(PR_SET_NAME)` no longer sufficient | `idle-cosmic/46db830` |

### CI matrix coverage (closed K2 from PROBE.md)

| Repo | CI job | Notes |
|---|---|---|
| `idlescreen/idle` | pre-existing (rust + clippy + test + doctest) | unchanged |
| `idlescreen/idle-cosmic` | rust + clippy -D warnings + fmt check on ubuntu-24.04 with `libclang-dev` | matrix added in this OODA; matrix split: fmt, clippy, test, smoke |
| `idlescreen/idle-tui` | rust + clippy + fmt on ubuntu-latest | matrix added |
| `idlescreen/idle-studio` | rust + clippy + fmt on ubuntu-latest | matrix added |
| `idlescreen/packages` | shellcheck + bash -n on every install script + F-202 regression test | matrix added |
| `idlescreen/render` | split into fmt / clippy / test / smoke / msrv / docs jobs | matrix upgraded |

### Out of scope (user-excluded)

- macOS shim (DECISION-MAC-01 = Option A, Sprint 05 H1)
- Windows shim (DECISION-WIN-01 = Option A, Sprint 05 H2)
- idle-windows / idle-steam / idle-pro stubs

### Long-term / trigger-gated (no sprint date; revisit on trigger)

### Long-term / trigger-gated (no sprint date; revisit on trigger)

```
15. WASM plugin host                                 -> trigger-gated
    DECISION-WASM-01 = Option C. Revisit if any of:
      (a) non-Linux shim ships, (b) third-party plugin author wants
      non-Rust, (c) install audit log shows plugins we don't maintain.

16. Homebrew / scoop / winget / MSI channels         -> Option B
    Revisit after #11/#12 land. Sequenced: Homebrew tap -> scoop ->
    winget -> MSI. Apple-Silicon-only for v1 (no Intel bottles).
```

### Drift fixed this pass
The Sprint 04+ block previously listed items #11–#14 as `done`, which
contradicted PM.md (still `not`). The block above replaces that with the
honest split: items 1, 4, 5, 7 → Sprint 04; items 2, 3, 6 → Sprint 05.
Sprint 04 + 05 sequencing respects the engine-portability precondition
(G3 must land before H1/H2).

## Released this sprint (2026-08-10)

| Repo | Tag | Notes |
|---|---|---|
| `idle` | v3.1.0 | Manifest host + runner fix |
| `idle-saver-beams` | v2.1.0 | .idleplugin.toml + F9 clippy sweep |
| `idle-saver-bursts` | v2.1.0 | .idleplugin.toml |
| `idle-saver-chaos` | v2.1.0 | .idleplugin.toml + F9 clippy sweep |
| `idle-saver-cosmos` | v2.1.0 | .idleplugin.toml + F9 clippy sweep (4 funcs) |
| `idle-saver-glyphs` | v2.1.0 | .idleplugin.toml |
| `idle-saver-gnats` | v2.1.0 | .idleplugin.toml + F9 clippy sweep |
| `idle-saver-hearth` | v2.1.0 | .idleplugin.toml + F9 clippy sweep |
| `idle-saver-radar` | v2.1.0 | .idleplugin.toml + F9 clippy sweep |
| `idle-saver-ripple` | v2.1.0 | .idleplugin.toml + F9 clippy sweep |
| `idle-saver-storm` | v2.1.0 | .idleplugin.toml |
| `packages` | v4.0.5 | Seats new manifests + install-time audit log |
| `render` (engine) | engine-v1.1.0 | F7 env-var gate + F8 refactor |
| `render` (studio) | studio-v0.3.4 | Rebuilt for engine 1.1.0 surface |

