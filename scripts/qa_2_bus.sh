check_bus() {
  echo "--- D-Bus control plane ---"

  if command -v busctl >/dev/null 2>&1; then
    if busctl --user status "$BUS_NAME" >/dev/null 2>&1; then
      ok "bus name $BUS_NAME is on the session bus"
    else
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
}
