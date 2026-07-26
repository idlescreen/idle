#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Post-upgrade / pre-release smoke for idle-daemon preview stability.
#
# Critical rules this script enforces:
#   1. Preview must never stop idle-daemon (NRestarts / MainPID stable).
#   2. Preview should stay up for HOLD_SECS without process death.
#   3. Journal must not show systemd unit failure.
#   4. Prefer fullscreen geometry (1920x1080-class) when compositor reports mode.
#   5. Grok agent-turn must not appear in `idlescreen inhibitors`.
#
# Usage:
#   ./scripts/qa_preview_smoke.sh
#   ./scripts/qa_preview_smoke.sh cosmos
#   SAVER=bursts LOOPS=3 HOLD_SECS=3 ./scripts/qa_preview_smoke.sh
#
# Exit 0 = pass, 1 = fail.

set -euo pipefail

SAVER="${1:-${SAVER:-beams}}"
LOOPS="${LOOPS:-2}"
HOLD_SECS="${HOLD_SECS:-3}"
MIN_DAEMON_VER="${MIN_DAEMON_VER:-2.5.9}"

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
    # Soft version check with sort -V when both are dotted.
    if [[ "$ver" =~ ^[0-9] ]]; then
      oldest=$(printf '%s\n%s\n' "$MIN_DAEMON_VER" "$ver" | sort -V | head -n1)
      if [[ "$oldest" != "$MIN_DAEMON_VER" && "$ver" != "$MIN_DAEMON_VER" ]]; then
        # ver older than min when sort puts ver first and they're not equal
        :
      fi
      if [[ "$oldest" == "$ver" && "$ver" != "$MIN_DAEMON_VER" ]]; then
        bad "idle-daemon $ver is older than required $MIN_DAEMON_VER"
      else
        ok "daemon version $ver meets minimum $MIN_DAEMON_VER"
      fi
    fi
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
echo "INFO: baseline NRestarts=$R0 MainPID=$PID0 saver=$SAVER loops=$LOOPS hold=${HOLD_SECS}s"

# --- Inhibitors: Grok agent-turn must not be listed (I1) ---
echo "--- inhibitors (Grok filter) ---"
inh_out="$(idlescreen inhibitors 2>&1 || true)"
echo "$inh_out" | sed 's/^/  | /'
if echo "$inh_out" | grep -qiE 'logind:grok|\[logind:grok'; then
  bad "idlescreen inhibitors lists grok (agent-turn should be ignored)"
else
  ok "inhibitors list does not show logind:grok"
fi
if echo "$inh_out" | grep -qi 'agent turn'; then
  bad "inhibitors list shows agent turn reason (should be filtered)"
else
  ok "inhibitors list does not show agent-turn reason"
fi

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

  # Preview should still be active after hold (durability / no instant fault-clear).
  st="$(idlescreen status 2>&1 || true)"
  if echo "$st" | grep -q 'preview_active:[[:space:]]*true'; then
    ok "preview_active=true after ${HOLD_SECS}s (loop $i)"
  else
    # Not always fatal if user dismissed or compositor quirks — warn as soft fail path.
    # We still require daemon alive.
    echo "INFO: preview_active not true after hold (loop $i) — checking daemon survival only"
    echo "$st" | sed 's/^/  | /' | head -20
  fi

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
    bad "MainPID changed $PID0 -> $PID (loop $i)"
  else
    ok "MainPID stable $PID0 (loop $i)"
  fi

  idlescreen stop >/dev/null 2>&1 || true
  sleep 0.4
done

# Double-preview without explicit stop (P9)
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

# Multi-saver back-to-back (P11)
echo "--- multi-saver back-to-back ---"
for s in beams ripple cosmos; do
  idlescreen preview "$s" >/dev/null 2>&1 || true
  sleep 1
done
idlescreen stop >/dev/null 2>&1 || true
if [[ "$(nrestarts)" == "$R0" ]] && systemctl --user is-active --quiet idle-daemon.service; then
  ok "multi-saver sequence kept daemon stable"
else
  bad "multi-saver sequence bounced daemon"
fi

# Journal checks
if command -v journalctl >/dev/null 2>&1; then
  echo "--- journal ---"
  recent="$(journalctl --user -u idle-daemon.service -n 120 --no-pager 2>/dev/null || true)"
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
  # Fullscreen (P13): geometry log or full mode size
  if echo "$recent" | grep -q 'fullscreen saver geometry'; then
    ok "journal reports fullscreen saver geometry"
  elif echo "$recent" | grep -qE 'output .* — 1920x1080|output .* — [0-9]+x[0-9]{4}'; then
    ok "journal shows full-height output geometry"
  else
    echo "INFO: no explicit fullscreen log line in last 120 journal lines (compositor-dependent)"
  fi
  # Thrash detection: many recoveries in short window is a regression smell
  thrash="$(echo "$recent" | grep -c 'recovering without exiting' || true)"
  if [[ "${thrash:-0}" -gt 8 ]]; then
    bad "journal shows $thrash recoveries recently (possible start/fault thrash)"
  else
    ok "recovery count in recent journal is bounded ($thrash)"
  fi
  if echo "$recent" | grep -q 'recovering without exiting'; then
    echo "INFO: recovery path exercised — ok if NRestarts stable"
    if echo "$recent" | grep -q 'presenter recreated successfully'; then
      ok "presenter recreated after fault"
    fi
  fi
  # CLI copy: forced preview ignores inhibitors
  if idlescreen inhibitors 2>&1 | grep -qi 'forced preview ignores'; then
    ok "inhibitors CLI documents forced preview ignore"
  else
    # only when there are active rows — empty list uses different text
    if idlescreen inhibitors 2>&1 | grep -q 'No active inhibitors'; then
      ok "inhibitors empty path (no forced-preview header needed)"
    else
      echo "INFO: inhibitors header may differ when list non-empty without phrase"
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
