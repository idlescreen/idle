#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Product closed-loop QA — the "territory" the unit package gate cannot prove.
#
# Principles covered:
#   1. Install / unit / bus name present
#   2. Control plane: Preview → status → Stop over real D-Bus
#   3. Healthy preview: presentation stays up for HOLD_SECS
#   4. Fullscreen geometry (when compositor logs it)
#   5. Fault injection: kill plugin children; daemon must stay up and accept
#      a new preview
#   6. Grok agent-turn not listed as inhibitor
#
# Requirements:
#   - User session with systemd --user
#   - WAYLAND_DISPLAY set (for preview presentation)
#   - idle-daemon.service active (or startable)
#   - idlescreen CLI installed
#
# Usage:
#   ./scripts/qa_product_loop.sh
#   SAVER=cosmos HOLD_SECS=4 ./scripts/qa_product_loop.sh
#
# Exit 0 = pass, 1 = fail. Not run during headless package builds.

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

# ---------------------------------------------------------------------------
# 1. Install / unit / binaries
# ---------------------------------------------------------------------------
echo "--- install / service ---"

if command -v rpm >/dev/null 2>&1; then
  if rpm -q idle-daemon >/dev/null 2>&1; then
    ver="$(rpm -q --qf '%{VERSION}' idle-daemon)"
    ok "idle-daemon package installed (version $ver)"
    oldest=$(printf '%s\n%s\n' "$MIN_DAEMON_VER" "$ver" | sort -V | head -n1)
    if [[ "$oldest" == "$ver" && "$ver" != "$MIN_DAEMON_VER" ]]; then
      bad "idle-daemon $ver older than minimum $MIN_DAEMON_VER"
    else
      ok "daemon version meets minimum $MIN_DAEMON_VER"
    fi
  else
    bad "idle-daemon RPM not installed"
  fi
  if rpm -q idle-cli >/dev/null 2>&1 || command -v idlescreen >/dev/null; then
    ok "idle-cli / idlescreen available"
  else
    bad "idle-cli / idlescreen missing"
  fi
else
  info "rpm not available — skipping package version checks"
fi

if [[ -x /usr/bin/idle-daemon ]]; then
  ok "/usr/bin/idle-daemon present"
else
  bad "/usr/bin/idle-daemon missing"
fi

if systemctl --user cat idle-daemon.service >/dev/null 2>&1; then
  ok "idle-daemon.service unit visible to user systemd"
else
  bad "idle-daemon.service unit not found"
fi

if ! systemctl --user is-active --quiet idle-daemon.service; then
  info "daemon inactive — attempting enable --now"
  systemctl --user enable --now idle-daemon.service 2>/dev/null || true
  sleep 1
fi

if systemctl --user is-active --quiet idle-daemon.service; then
  ok "idle-daemon.service is active"
else
  bad "idle-daemon.service is not active"
  echo "Cannot continue closed-loop without a running daemon."
  echo "=== summary: $pass pass, $fail fail ==="
  exit 1
fi

R0="$(systemctl --user show idle-daemon.service -p NRestarts --value)"
PID0="$(systemctl --user show idle-daemon.service -p MainPID --value)"
info "baseline NRestarts=$R0 MainPID=$PID0"

# ---------------------------------------------------------------------------
# 2. Bus name (control plane presence)
# ---------------------------------------------------------------------------
echo "--- D-Bus control plane ---"

if command -v busctl >/dev/null 2>&1; then
  if busctl --user status "$BUS_NAME" >/dev/null 2>&1; then
    ok "bus name $BUS_NAME is on the session bus"
  else
    # Some systems need list + grep
    if busctl --user list 2>/dev/null | grep -qF "$BUS_NAME"; then
      ok "bus name $BUS_NAME listed on session bus"
    else
      bad "bus name $BUS_NAME not on session bus"
    fi
  fi
else
  info "busctl missing — relying on idlescreen status for bus check"
fi

if idlescreen status >/dev/null 2>&1; then
  ok "idlescreen status reaches the daemon"
else
  bad "idlescreen status cannot reach the daemon"
fi

# ---------------------------------------------------------------------------
# 3. Control plane: Preview → healthy status → Stop
# ---------------------------------------------------------------------------
echo "--- control plane closed loop ---"

if [[ -z "${WAYLAND_DISPLAY:-}" ]]; then
  bad "WAYLAND_DISPLAY unset — presentation cannot be verified"
  echo "Set up a Wayland session and re-run."
  echo "=== summary: $pass pass, $fail fail ==="
  exit 1
fi
ok "WAYLAND_DISPLAY=${WAYLAND_DISPLAY}"

idlescreen stop >/dev/null 2>&1 || true
sleep 0.3

if ! idlescreen preview "$SAVER"; then
  bad "idlescreen preview $SAVER failed"
else
  ok "idlescreen preview $SAVER accepted"
fi

# Poll for healthy preview (plugin start can take ~1s)
healthy=0
for _ in $(seq 1 "$((HOLD_SECS * 2))"); do
  st="$(idlescreen status 2>&1 || true)"
  if echo "$st" | grep -q 'preview_active:[[:space:]]*true' \
    && echo "$st" | grep -q 'presentation_active:[[:space:]]*true' \
    && echo "$st" | grep -q 'running:[[:space:]]*true'; then
    healthy=1
    break
  fi
  sleep 0.5
done

if [[ "$healthy" -eq 1 ]]; then
  ok "healthy preview (preview_active + presentation_active + running)"
else
  bad "preview never became healthy within ${HOLD_SECS}s"
  idlescreen status 2>&1 | sed 's/^/  | /' || true
fi

# Hold duration without death
sleep "$HOLD_SECS"
st="$(idlescreen status 2>&1 || true)"
if echo "$st" | grep -q 'preview_active:[[:space:]]*true'; then
  ok "preview still active after ${HOLD_SECS}s hold"
else
  bad "preview_active lost during ${HOLD_SECS}s hold (broken-but-up?)"
  echo "$st" | sed 's/^/  | /' || true
fi

R1="$(systemctl --user show idle-daemon.service -p NRestarts --value)"
PID1="$(systemctl --user show idle-daemon.service -p MainPID --value)"
if [[ "$R1" != "$R0" ]]; then
  bad "NRestarts rose during preview: $R0 -> $R1"
else
  ok "NRestarts unchanged during preview ($R0)"
fi
if [[ "$PID1" != "$PID0" ]]; then
  bad "MainPID changed during preview: $PID0 -> $PID1"
else
  ok "MainPID stable during preview ($PID0)"
fi

if ! idlescreen stop; then
  bad "idlescreen stop failed"
else
  ok "idlescreen stop accepted"
fi
sleep 0.5
st="$(idlescreen status 2>&1 || true)"
if echo "$st" | grep -q 'preview_active:[[:space:]]*false'; then
  ok "preview_active false after stop"
else
  bad "preview_active still true after stop"
fi

# ---------------------------------------------------------------------------
# 4. Fullscreen (journal)
# ---------------------------------------------------------------------------
echo "--- fullscreen / journal ---"
if command -v journalctl >/dev/null 2>&1; then
  # Re-run a short preview to capture geometry lines
  idlescreen preview "$SAVER" >/dev/null 2>&1 || true
  sleep 2
  recent="$(journalctl --user -u idle-daemon.service -n 60 --no-pager 2>/dev/null || true)"
  if echo "$recent" | grep -q 'fullscreen saver geometry'; then
    ok "journal: fullscreen saver geometry"
  elif echo "$recent" | grep -qE 'output .* — [0-9]+x(10[8-9][0-9]|1[1-9][0-9]{2}|[2-9][0-9]{3})'; then
    ok "journal: full-height-class output geometry"
  else
    bad "journal missing fullscreen/full-height geometry (panel inset regression?)"
  fi
  if echo "$recent" | grep -q 'Main process exited'; then
    bad "journal: Main process exited"
  else
    ok "journal: no Main process exited"
  fi
  thrash="$(echo "$recent" | grep -c 'recovering without exiting' || true)"
  if [[ "${thrash:-0}" -gt 8 ]]; then
    bad "journal: thrash ($thrash recoveries)"
  else
    ok "journal: recovery count bounded ($thrash)"
  fi
  idlescreen stop >/dev/null 2>&1 || true
else
  info "journalctl missing — skip fullscreen journal checks"
fi

# ---------------------------------------------------------------------------
# 5. Fault injection: kill plugin children, daemon must survive
# ---------------------------------------------------------------------------
echo "--- fault injection (kill plugin children) ---"
idlescreen preview "$SAVER" >/dev/null 2>&1 || true
sleep 2
main_pid="$(systemctl --user show idle-daemon.service -p MainPID --value)"
# Children of the daemon (IPC runners / plugins), not the main PID
mapfile -t kids < <(pgrep -P "$main_pid" 2>/dev/null || true)
if [[ "${#kids[@]}" -eq 0 ]]; then
  # Fallback: any process with idle-runner / libscreensaver parented loosely
  mapfile -t kids < <(pgrep -f 'idle-runner|libscreensaver_|IPC Runner' 2>/dev/null | head -5 || true)
fi

if [[ "${#kids[@]}" -gt 0 ]]; then
  info "killing plugin/helper PIDs: ${kids[*]}"
  for k in "${kids[@]}"; do
    if [[ "$k" != "$main_pid" ]]; then
      kill "$k" 2>/dev/null || true
    fi
  done
  sleep 1.5
  if systemctl --user is-active --quiet idle-daemon.service; then
    ok "daemon still active after killing plugin children"
  else
    bad "daemon died after plugin child kill"
  fi
  R2="$(systemctl --user show idle-daemon.service -p NRestarts --value)"
  if [[ "$R2" != "$R0" ]]; then
    bad "NRestarts rose after plugin kill: $R0 -> $R2"
  else
    ok "NRestarts unchanged after plugin kill"
  fi
  # Must accept a new preview (control plane still alive)
  idlescreen stop >/dev/null 2>&1 || true
  sleep 0.3
  if idlescreen preview "$SAVER" >/dev/null 2>&1; then
    sleep 1.5
    st="$(idlescreen status 2>&1 || true)"
    if echo "$st" | grep -q 'preview_active:[[:space:]]*true'; then
      ok "re-preview after fault succeeded"
    else
      bad "re-preview after fault did not become active"
    fi
  else
    bad "idlescreen preview failed after plugin kill"
  fi
  idlescreen stop >/dev/null 2>&1 || true
else
  info "no plugin child PIDs found to kill — soft-skip fault injection"
fi

# ---------------------------------------------------------------------------
# 6. Inhibitors: Grok must not appear
# ---------------------------------------------------------------------------
echo "--- inhibitors ---"
inh="$(idlescreen inhibitors 2>&1 || true)"
if echo "$inh" | grep -qiE 'logind:grok|agent turn'; then
  bad "inhibitors still surface Grok/agent-turn"
else
  ok "inhibitors do not list Grok/agent-turn"
fi

# ---------------------------------------------------------------------------
# 7. Doctor honesty when daemon up (not necessarily NOMINAL if inhibited)
# ---------------------------------------------------------------------------
echo "--- doctor ---"
if idlescreen doctor >/tmp/idle-doctor.out 2>&1; then
  doctor_rc=0
else
  doctor_rc=$?
fi
# Case-sensitive: fail line is "INHIBITED — ..."; ok line is "uninhibited ..."
if grep -q 'ALL SYSTEMS NOMINAL' /tmp/idle-doctor.out 2>/dev/null; then
  if grep -E 'Inhibitor Status: INHIBITED' /tmp/idle-doctor.out >/dev/null 2>&1; then
    bad "doctor claims NOMINAL while Inhibitor Status is INHIBITED"
  else
    ok "doctor NOMINAL and not reporting Inhibitor Status: INHIBITED"
  fi
else
  # Non-zero or non-nominal is fine if daemon is up and report is coherent
  if grep -qiE 'D-Bus|Inhibitor|idle-daemon' /tmp/idle-doctor.out; then
    ok "doctor produced a coherent report (rc=$doctor_rc)"
  else
    bad "doctor output unrecognizable (rc=$doctor_rc)"
  fi
fi

# Final daemon health
if systemctl --user is-active --quiet idle-daemon.service \
  && [[ "$(systemctl --user show idle-daemon.service -p NRestarts --value)" == "$R0" ]]; then
  ok "final: daemon active, NRestarts still $R0"
else
  bad "final: daemon unstable or restarted"
fi

echo ""
echo "=== product loop summary: $pass pass, $fail fail ==="
if [[ "$fail" -gt 0 ]]; then
  exit 1
fi
echo "QA_PRODUCT_LOOP_PASS"
exit 0
