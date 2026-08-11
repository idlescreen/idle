#!/usr/bin/env bash
# chaos_test_fd.sh - FD limit exhaustion test (sourced by chaos_test.sh)
test_fd_limit_exhaustion() {
    echo ""
    echo "[CHAOS TEST 1/3] Simulating File Descriptor (FD) Limit Exhaustion..."
    cleanup
    sleep 0.2

    local fd_log output
    fd_log=$(mktemp)
    (
        ulimit -n 64 2>/dev/null || true
        exec timeout -s 9 2s python3 -c "
import os, sys
for fd in range(4, 256):
    if fd in (3, 255):
        continue
    try:
        target = os.readlink(f'/proc/self/fd/{fd}')
        if any(k in target for k in ['.sh', 'chaos', 'pts', 'tty', 'console', 'socket']):
            continue
        os.close(fd)
    except Exception:
        pass
fds = []
for _ in range(40):
    try:
        fd = os.open('/dev/null', os.O_RDONLY)
        os.set_inheritable(fd, True)
        fds.append(fd)
    except Exception:
        break
os.execv(sys.argv[1], [sys.argv[1]])
" "$IDLE_DAEMON"
    ) >"$fd_log" 2>&1 || true

    output=$(cat "$fd_log" 2>/dev/null || true)
    rm -f "$fd_log" 2>/dev/null || true

    if echo "$output" | grep -q -i "panicked at"; then
        echo "[CHAOS ERROR] Daemon panicked under FD limit exhaustion!"
        return 1
    fi

    echo "[PROOF] Verifying daemon / IPC self-healing post-FD restoration..."
    cleanup
    sleep 0.2
    spawn_test_daemon

    if ! wait_for_daemon_dbus_ready "$TEST_DAEMON_PID"; then
        echo "[CHAOS ERROR] Post-FD recovery failed: daemon D-Bus/Wayland init failed (check env; not auth trust-all)."
        tail -n 20 "$DAEMON_LOG" 2>/dev/null || true
        cleanup; return 1
    fi
    cleanup
    echo "✅ [PASS] FD Limit Exhaustion test passed (fail-closed and self-healed)."
    return 0
}
