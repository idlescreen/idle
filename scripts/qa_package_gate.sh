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
cargo test -p idle-daemon -p idle-ipc -p idle-dbus -p wayland-present -- \
  doctor_rules inhibitors_fmt ignore_logind merge_drops merge_includes \
  recovery_plan present_cooldown thrash hold_idle exit_process \
  preview_starts idle_decision path_safety hw_scaling \
  frame_geometry layer_not would_block eagain exclusive_zone \
  panel_expand fullscreen_expands geom_tests battery_should \
  format_status status_text status_json golden_ \
  contract_ STATUS_FIELD control_methods bus_contract \
  queue_preview queue_stop requeue after_fault multi_preview \
  trusted_control untrusted_basename applet_comm security_reject \
  command_queue set_saver_rejects shm_rejects shm_accepts socket_rejects \
  freeness_screensaver firefox_playing firefox_double_count firefox_prune \
  firefox_uninhibit add_coalesces prune_not_in_live sniffable_list \
  sniffable_list_never freeness_screensaver_must
echo ""
echo ">>> package gate: named regressions passed"
echo ""

# When sibling screensaver checkouts exist, require their tests too
# (GH host cuts often ship next to idle-savers meta). Set GATE_SAVERS=0 to skip.
if [[ "${GATE_SAVERS:-1}" != "0" ]]; then
  SAVERS_GATE="$(cd "$ROOT/.." && pwd)/packages/scripts/qa_savers_package_gate.sh"
  if [[ -x "$SAVERS_GATE" ]] || [[ -f "$SAVERS_GATE" ]]; then
    echo ">>> sibling savers package gate"
    bash "$SAVERS_GATE"
  elif compgen -G "$ROOT/../idle-saver-*/Cargo.toml" >/dev/null; then
    echo ">>> sibling savers (inline)"
    failed_s=0
    for dir in "$ROOT"/../idle-saver-*/; do
      name="$(basename "$dir")"
      echo ">>> $name"
      (
        cd "$dir"
        [[ -e runtime ]] || ln -sfn ../runtime runtime
        cargo test --quiet
      ) || failed_s=$((failed_s + 1))
    done
    if [[ "$failed_s" -gt 0 ]]; then
      echo "FAIL: $failed_s saver suite(s) failed" >&2
      exit 1
    fi
  else
    echo "INFO: no sibling idle-saver-* checkouts — host-only gate"
  fi
fi

echo "=========================================="
echo "PACKAGE_GATE_PASS"
echo "=========================================="
exit 0