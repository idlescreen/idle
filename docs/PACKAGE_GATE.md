# Package build gate (headless)

**Goal:** Fail packaging early when host/CLI/presenter regressions reappear.  
**Constraint:** No Wayland session, no live `idle-daemon`, no interactive TUI.

## What runs

```bash
./scripts/qa_package_gate.sh
# or: just qa-package-gate
# or automatically: ./package.rs / just package
```

### Step 1 — Full unit suites (selected crates)

| Crate | Why in the gate |
|-------|-----------------|
| `idle-daemon` | Recovery, idle policy, inhibitors, battery, auth, presentation helpers |
| `idle-cli` | Doctor rules, inhibitors report, status text/json |
| `idle-ipc` | SHM / socket path safety (preview hard-cut) |
| `idle-dbus` | Status map / D-Bus types |
| `idle-upscaler` | CPU upscale path used at 50% scale |
| `idle-runner` | Plugin load / launcher trust |
| `idle-api` | Saver API contracts |
| `wayland-present` | Geometry, fullscreen, EAGAIN, frame safety |
| `wayland-idle` | Idle monitor helpers (unit only) |

### Step 2 — Named regression filters

Hard filters for bugs already fixed in production (preview kill, thrash,
Grok filter, fullscreen panel, doctor NOMINAL, SHM, etc.). See
`docs/QA_REGRESSION.md` issue map.

## What does **not** run at package time

| Check | Why excluded | When to run |
|-------|--------------|-------------|
| `scripts/qa_preview_smoke.sh` | Needs user session + compositor | After install on a real desktop |
| COSMIC applet UI | Separate package / desktop | Manual |
| Nested Wayland CI | Infra-heavy | Optional later |

## Escape hatch

```bash
SKIP_TESTS=1 ./package.rs   # emergency only — not for release cuts
```

## Extending the gate

Prefer **pure functions + unit tests** over live checks:

1. Extract policy/format logic (no I/O).
2. Add `#[test]` in the same crate.
3. If it guards a past bug, add a name token to the named filter list in
   `scripts/qa_package_gate.sh` and `just qa-unit-named`.

Do **not** add tests that open Wayland, need `sudo`, or require a running
daemon to the package gate.
