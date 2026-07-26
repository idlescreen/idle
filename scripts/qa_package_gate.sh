#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Headless package gate — runs during packaging (no Wayland / no live daemon).
# Goal: fail the build early if host/CLI/presenter regressions reappear.
#
# Usage (from idle repo root):
#   ./scripts/qa_package_gate.sh
#   SKIP_TESTS=1 ./scripts/qa_package_gate.sh   # no-op success (emergency)
#
# Invoked by package.rs before release builds.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ "${SKIP_TESTS:-}" == "1" || "${SKIP_TESTS:-}" == "true" || "${SKIP_TESTS:-}" == "yes" ]]; then
  echo "SKIP_TESTS set — package gate skipped (not for release cuts)."
  exit 0
fi

echo "=========================================="
echo "IdleScreen package gate (headless)"
echo "=========================================="
echo "Repo: $ROOT"
echo ""

# Crates that ship in or under the host stack and have pure/unit tests.
# Do NOT include anything that requires a live Wayland session or root.
PACKAGES=(
  idle-cli
  idle-daemon
  idle-ipc
  idle-dbus
  idle-upscaler
  idle-runner
  idle-api
  wayland-present
  wayland-idle
)

echo ">>> cargo test: ${PACKAGES[*]}"
test_args=()
for pkg in "${PACKAGES[@]}"; do
  test_args+=(-p "$pkg")
done
cargo test "${test_args[@]}"
echo ""
echo ">>> package gate: unit suites passed"
echo ""

# Named regression filters — fail closed on the bugs we already fixed.
# Keeps the gate honest even if a new untested module is added elsewhere.
echo ">>> cargo test (named host/preview regressions)"
cargo test -p idle-cli -p idle-daemon -p idle-ipc -p idle-dbus -p wayland-present -- \
  doctor_rules inhibitors_fmt ignore_logind merge_drops merge_includes \
  recovery_plan present_cooldown thrash hold_idle exit_process \
  preview_starts idle_decision path_safety hw_scaling \
  frame_geometry layer_not would_block eagain exclusive_zone \
  panel_expand fullscreen_expands geom_tests battery_should \
  format_status status_text status_json golden_ \
  contract_ STATUS_FIELD control_methods bus_contract \
  queue_preview queue_stop requeue after_fault multi_preview \
  trusted_control untrusted_basename applet_comm security_reject \
  command_queue set_saver_rejects shm_rejects shm_accepts socket_rejects
echo ""
echo ">>> package gate: named regressions passed"
echo ""

echo "=========================================="
echo "PACKAGE_GATE_PASS"
echo "=========================================="
exit 0
