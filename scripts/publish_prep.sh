#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Prepare channel for GitHub publish. Does NOT push (needs operator auth/GPG).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PKG="$ROOT/packages"

echo "=== Publish prep (local) ==="
echo "ROOT=$ROOT"
echo ""
echo "1) Publish gate..."
bash "$ROOT/scripts/publish_gate.sh"
echo ""
echo "2) Line lock..."
bash "$ROOT/scripts/check_file_lines.sh"
echo ""
echo "3) Channel honesty note:"
echo "   Local pool latest is idle-* 3.0.0 / meta 3.0.0."
echo "   GitHub Pages only updates after you push packages master + Pages deploy."
echo ""
echo "4) Suggested operator steps (not run by this script):"
echo "   cd $PKG"
echo "   # optional: cargo run --release --bin sign   # if IDLESCREEN_GPG_NAME set"
echo "   # optional: ./update.sh                      # re-index if you add pool files"
echo "   git status"
echo "   git push origin master"
echo "   # Confirm Pages serves https://idlescreen.github.io/packages/"
echo ""
echo "5) After push, verify remote has 3.0.0:"
echo "   curl -s 'https://api.github.com/repos/idlescreen/packages/contents/rpm/pool?ref=master' | grep 3.0.0"
echo ""
echo "=== Publish prep complete (no remote changes made) ==="
