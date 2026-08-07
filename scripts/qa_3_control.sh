check_control() {
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
}
