#!/usr/bin/env bash
# chaos_test_fs.sh - Read-only /tmp & unwritable XDG_RUNTIME_DIR test (sourced by chaos_test.sh)
test_readonly_runtime() {
    echo ""
    echo "[CHAOS TEST 3/3] Simulating Read-Only /tmp & Unwriteable Runtime Dirs..."
    cleanup
    local tmp_base
    tmp_base=$(mktemp -d /tmp/idle_chaos_XXXXXX)
    CHAOS_TMP_DIRS+=("$tmp_base")

    local ro_runtime="$tmp_base/ro_runtime"
    mkdir -p "$ro_runtime" && chmod 555 "$ro_runtime"
    echo "[PROOF] Testing unwriteable XDG_RUNTIME_DIR handling..."
    local out1 code1=0
    out1=$(XDG_RUNTIME_DIR="$ro_runtime" timeout 2s "$IDLE_DAEMON" 2>&1) || code1=$?
    chmod 755 "$ro_runtime"

    if echo "$out1" | grep -q -i "panicked at" || [ "$code1" -eq 0 ]; then
        echo "[CHAOS ERROR] Daemon failed unwriteable XDG_RUNTIME_DIR test (out=$out1, code=$code1)"
        cleanup; return 1
    fi

    local ro_tmp="$tmp_base/ro_tmp"
    mkdir -p "$ro_tmp" && chmod 444 "$ro_tmp"
    echo "[PROOF] Testing read-only TMPDIR handling..."
    local out2 code2=0
    out2=$(unset XDG_RUNTIME_DIR; TMPDIR="$ro_tmp" timeout 2s "$IDLE_DAEMON" 2>&1) || code2=$?
    chmod 755 "$ro_tmp"

    if echo "$out2" | grep -q -i "panicked at" || [ "$code2" -eq 0 ]; then
        echo "[CHAOS ERROR] Daemon failed read-only TMPDIR test (out=$out2, code=$code2)"
        cleanup; return 1
    fi

    echo "[PROOF] Testing invalid XDG_RUNTIME_DIR path..."
    local out3 code3=0
    out3=$(XDG_RUNTIME_DIR="/proc/invalid_idle_path_test" timeout 2s "$IDLE_DAEMON" 2>&1) || code3=$?

    if echo "$out3" | grep -q -i "panicked at" || [ "$code3" -eq 0 ]; then
        echo "[CHAOS ERROR] Daemon failed invalid XDG_RUNTIME_DIR path test (out=$out3, code=$code3)"
        cleanup; return 1
    fi

    cleanup
    echo "✅ [PASS] Read-only / unwriteable runtime dir test passed (fail-closed and safely handled)."
    return 0
}
