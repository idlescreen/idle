#!/usr/bin/env bash
# bootstrap.sh — one-command clean-clone → cargo test path.
# Idempotent. Safe to re-run.
#
# Source this from a fresh clone of any Rust repo in the idlescreen org that
# depends on the `idle` workspace (idle-cosmic, idle-tui, idle-studio,
# idle-saver-*). It will:
#   1. Ensure rustup + the pinned toolchain (rust-toolchain.toml).
#   2. Install cargo-audit / cargo-deny if missing.
#   3. Symlink ../idle if the repo's path deps expect it.
#   4. Print the verified next-step (cargo test --workspace).
#
# This is the *light-tier* local helper for hygiene; it never
# touches the network beyond `rustup`/`cargo install`.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || true)"
if [ -z "$REPO_ROOT" ] || [ ! -d "$REPO_ROOT" ]; then
    echo "bootstrap.sh: cannot resolve script directory" >&2
    exit 1
fi
cd "$REPO_ROOT"

log() { printf 'bootstrap: %s\n' "$*"; }
fail() { printf 'bootstrap: ERROR: %s\n' "$*" >&2; exit 1; }

# 1. rustup + pinned toolchain
if ! command -v rustup >/dev/null 2>&1; then
    fail "rustup not on PATH. Install: https://rustup.rs"
fi
if [ -f rust-toolchain.toml ]; then
    log "rustup show (pinned toolchain)"
    rustup show >/dev/null
else
    log "rustup show (no pin; using rustup default)"
    rustup show >/dev/null || true
fi

# 2. cargo-audit / cargo-deny (best-effort)
for bin in cargo-audit cargo-deny; do
    if ! command -v "$bin" >/dev/null 2>&1; then
        log "installing $bin (locked)"
        cargo install --locked "$bin" 2>/dev/null || log "  (skipped; offline?)"
    fi
done

# 3. sibling idle/ symlink if a path dep expects it.
if [ ! -e idle ] && grep -qE 'path *= *"\./?idle/' Cargo.toml 2>/dev/null; then
    if [ -d ../idle ]; then
        log "symlinking idle/ -> ../idle"
        ln -sfn ../idle idle
    else
        log "NOTE: idle/ missing and ../idle not present."
        log "      Either clone idlescreen/idle as a sibling or run"
        log "      'git submodule update --init' if this repo uses submodules."
    fi
fi

# 4. apt deps note (best-effort; do not auto-install without consent)
if command -v apt-get >/dev/null 2>&1; then
    for pkg in libdbus-1-dev libwayland-dev libxkbcommon-dev libssl-dev pkg-config libudev-dev; do
        if ! dpkg -s "$pkg" >/dev/null 2>&1; then
            log "apt: $pkg not installed (needed for build)"
        fi
    done
fi

log "ready. Next: cargo test --workspace"