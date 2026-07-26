#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Post-upgrade / pre-release smoke for idle-daemon preview stability.
#
# Critical rule: preview (CLI or TUI `p`) must never stop idle-daemon.
# NRestarts must not increase; service must stay active.
#
# Usage:
#   ./scripts/qa_preview_smoke.sh
#   ./scripts/qa_preview_smoke.sh cosmos
#   SAVER=bursts LOOPS=3 ./scripts/qa_preview_smoke.sh
#
# Exit 0 = pass, 1 = fail.

set -euo pipefail

SAVER="${1:-${SAVER:-beams}}"
LOOPS="${LOOPS:-2}"
HOLD_SECS="${HOLD_SECS:-2}"
MIN_DAEMON_VER="${MIN_DAEMON_VER:-2.5.6}"

pass=0
fail=0

ok() {
  echo "PASS: $*"
  pass=$((pass + 1))
}

bad() {
  echo "FAIL: $*" >&2
  fail=$((fail + 1))
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "FAIL: missing command: $1" >&2
    exit 1
  }
}

need_cmd systemctl
need_cmd idlescreen

if ! systemctl --user is-active --quiet idle-daemon.service; then
  bad "idle-daemon.service is not active (start it first)"
  exit 1
fi

if command -v rpm >/dev/null 2>&1; then
  ver="$(rpm -q --qf '%{VERSION}' idle-daemon 2>/dev/null || true)"
  if [[ -n "$ver" ]]; then
    echo "INFO: idle-daemon rpm version=$ver (want >= $MIN_DAEMON_VER)"
  fi
fi

nrestarts() {
  systemctl --user show idle-daemon.service -p NRestarts --value
}

main_pid() {
  systemctl --user show idle-daemon.service -p MainPID --value
}

R0="$(nrestarts)"
PID0="$(main_pid)"
echo "INFO: baseline NRestarts=$R0 MainPID=$PID0 saver=$SAVER loops=$LOOPS"

# Stop any leftover preview so status is clean.
idlescreen stop >/dev/null 2>&1 || true
sleep 0.3

for i in $(seq 1 "$LOOPS"); do
  echo "--- loop $i/$LOOPS ---"
  if ! idlescreen preview "$SAVER"; then
    bad "idlescreen preview $SAVER failed (loop $i)"
    continue
  fi
  sleep "$HOLD_SECS"

  if ! systemctl --user is-active --quiet idle-daemon.service; then
    bad "daemon inactive after preview (loop $i)"
  else
    ok "daemon active after preview (loop $i)"
  fi

  R="$(nrestarts)"
  PID="$(main_pid)"
  if [[ "$R" != "$R0" ]]; then
    bad "NRestarts rose $R0 -> $R (loop $i) — preview killed/restarted daemon"
  else
    ok "NRestarts still $R0 (loop $i)"
  fi
  if [[ "$PID" != "$PID0" ]]; then
    # PID change with NRestarts bump is fatal; PID change alone is still bad.
    bad "MainPID changed $PID0 -> $PID (loop $i)"
  else
    ok "MainPID stable $PID0 (loop $i)"
  fi

  idlescreen stop >/dev/null 2>&1 || true
  sleep 0.4
done

# Double-preview without explicit stop (P9): second should still not kill daemon.
echo "--- double-preview without stop ---"
idlescreen preview "$SAVER" >/dev/null 2>&1 || true
sleep 0.5
idlescreen preview "$SAVER" >/dev/null 2>&1 || true
sleep "$HOLD_SECS"
R="$(nrestarts)"
if [[ "$R" != "$R0" ]]; then
  bad "NRestarts rose on double-preview: $R0 -> $R"
else
  ok "double-preview NRestarts unchanged"
fi
if systemctl --user is-active --quiet idle-daemon.service; then
  ok "daemon active after double-preview"
else
  bad "daemon inactive after double-preview"
fi
idlescreen stop >/dev/null 2>&1 || true

# Journal: recovery is allowed; process exit is not.
if command -v journalctl >/dev/null 2>&1; then
  recent="$(journalctl --user -u idle-daemon.service -n 80 --no-pager 2>/dev/null || true)"
  if echo "$recent" | grep -q 'Main process exited'; then
    bad "journal shows Main process exited"
  else
    ok "no Main process exited in recent journal"
  fi
  if echo "$recent" | grep -q 'Failed with result'; then
    bad "journal shows Failed with result"
  else
    ok "no Failed with result in recent journal"
  fi
  if echo "$recent" | grep -q 'recovering without exiting'; then
    echo "INFO: recovery path exercised (presenter fault recovered without exit) — ok if NRestarts stable"
    if echo "$recent" | grep -q 'presenter recreated successfully'; then
      ok "presenter recreated after fault"
    fi
  fi
fi

echo
echo "=== QA smoke summary: $pass pass, $fail fail ==="
if [[ "$fail" -gt 0 ]]; then
  exit 1
fi
echo "QA_PREVIEW_SMOKE_PASS"
exit 0
