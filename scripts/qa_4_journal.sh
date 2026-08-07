check_journal() {
  echo "--- fullscreen / journal ---"
  if command -v journalctl >/dev/null 2>&1; then
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
}
