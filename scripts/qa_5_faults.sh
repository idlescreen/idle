check_faults() {
  echo "--- fault injection (kill plugin children) ---"
  idlescreen preview "$SAVER" >/dev/null 2>&1 || true
  sleep 2
  main_pid="$(systemctl --user show idle-daemon.service -p MainPID --value)"
  mapfile -t kids < <(pgrep -P "$main_pid" 2>/dev/null || true)
  if [[ "${#kids[@]}" -eq 0 ]]; then
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

  echo "--- inhibitors ---"
  inh="$(idlescreen inhibitors 2>&1 || true)"
  if echo "$inh" | grep -qiE 'logind:grok|agent turn'; then
    bad "inhibitors still surface Grok/agent-turn"
  else
    ok "inhibitors do not list Grok/agent-turn"
  fi

  echo "--- doctor ---"
  if idlescreen doctor >/tmp/idle-doctor.out 2>&1; then
    doctor_rc=0
  else
    doctor_rc=$?
  fi
  if grep -q 'ALL SYSTEMS NOMINAL' /tmp/idle-doctor.out 2>/dev/null; then
    if grep -E 'Inhibitor Status: INHIBITED' /tmp/idle-doctor.out >/dev/null 2>&1; then
      bad "doctor claims NOMINAL while Inhibitor Status is INHIBITED"
    else
      ok "doctor NOMINAL and not reporting Inhibitor Status: INHIBITED"
    fi
  else
    if grep -qiE 'D-Bus|Inhibitor|idle-daemon' /tmp/idle-doctor.out; then
      ok "doctor produced a coherent report (rc=$doctor_rc)"
    else
      bad "doctor output unrecognizable (rc=$doctor_rc)"
    fi
  fi

  if systemctl --user is-active --quiet idle-daemon.service \
    && [[ "$(systemctl --user show idle-daemon.service -p NRestarts --value)" == "$R0" ]]; then
    ok "final: daemon active, NRestarts still $R0"
  else
    bad "final: daemon unstable or restarted"
  fi
}
