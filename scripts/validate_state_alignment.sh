#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Validation script for Requirement R2 / Acceptance Criterion 4: State Alignment

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/.."

echo "=== Running State Alignment Integration Tests ==="
cd "$PROJECT_ROOT/idle"
cargo test --test state_sync_tests

echo "=== Verifying File Line Count Constraints (<250 lines) ==="
"$PROJECT_ROOT/scripts/check_file_lines.sh"

echo "=== All Milestone 2 State Alignment Checks Passed Successfully ==="
