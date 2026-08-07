#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Product closed-loop QA — the "territory" the unit package gate cannot prove.

set -euo pipefail

SAVER="${SAVER:-beams}"
HOLD_SECS="${HOLD_SECS:-4}"
MIN_DAEMON_VER="${MIN_DAEMON_VER:-2.5.10}"
BUS_NAME="io.github.idlescreen.Idle"

pass=0
fail=0

ok()  { echo "PASS: $*"; pass=$((pass + 1)); }
bad() { echo "FAIL: $*" >&2; fail=$((fail + 1)); }
info(){ echo "INFO: $*"; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "FAIL: missing command: $1" >&2
    exit 1
  }
}

need_cmd systemctl
need_cmd idlescreen

echo "=========================================="
echo "IdleScreen product closed-loop QA"
echo "=========================================="
echo "saver=$SAVER hold=${HOLD_SECS}s bus=$BUS_NAME"
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
source "$SCRIPT_DIR/qa_1_install.sh"
source "$SCRIPT_DIR/qa_2_bus.sh"
source "$SCRIPT_DIR/qa_3_control.sh"
source "$SCRIPT_DIR/qa_4_journal.sh"
source "$SCRIPT_DIR/qa_5_faults.sh"

check_install
check_bus
check_control
check_journal
check_faults

echo ""
echo "=== product loop summary: $pass pass, $fail fail ==="
if [[ "$fail" -gt 0 ]]; then
  exit 1
fi
echo "QA_PRODUCT_LOOP_PASS"
exit 0
