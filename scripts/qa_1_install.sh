check_install() {
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
}
