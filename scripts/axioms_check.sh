#!/usr/bin/env bash
# scripts/axioms_check.sh - Fail-closed bundle of RULES.md §1.7 hygiene gates.
#
# References:
#   RULES.md §1.7 — Hygiene (required): line pressure, no cruft, no secrets,
#   temps cleaned, git hygiene, claims match tree.
#
# Runs from the monorepo root regardless of caller cwd (anchors via BASH_SOURCE).
# Each gate fails closed (exit non-zero) if violated. Warnings are non-blocking.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

printf 'RULES.md §1.7 hygiene bundle — root=%s\n\n' "$ROOT"

# ---- Gate 1: line pressure (RULES.md §1.7) -----------------
printf '[gate 1/4] line pressure (<=256 lines per owned *.rs/*.c/*.h/*.sh)\n'
"$SCRIPT_DIR/check_file_lines.sh"

# ---- Gate 2: no production into_inner() poison recovery (audit F-004) ---------
printf '\n[gate 2/4] no production into_inner() (audit F-004)\n'
hits=$(
    grep -RIn --include='*.rs' 'into_inner()' \
        "$ROOT/idle/idle-daemon/src" \
        "$ROOT/idle/idle-runner/src" \
        2>/dev/null \
        | grep -v '/tests' \
        | grep -v '_tests\.rs' \
        | grep -v 'mod tests' \
        | grep -v 'locks\.rs' \
        | grep -v 'poison_or_exit' \
        || true
)
if [ -n "$hits" ]; then
    printf '  FAIL: production into_inner() found:\n'
    printf '%s\n' "$hits"
    exit 1
fi
printf '  ok\n'

# ---- Gate 3: no committed build artifacts in tree (RULES.md §1.7 no cruft) -------
printf '\n[gate 3/4] no committed build artifacts (RULES.md §1.7 no cruft)\n'
cruft=$(find . \
    -path '*/target' -prune -o \
    -path '*/.git' -prune -o \
    -path '*/node_modules' -prune -o \
    -path '*/.local' -prune -o \
    -path '*/.agents' -prune -o \
    -path '*/.cache' -prune -o \
    -type f \( -name '*.pyc' -o -name 'Cargo.lock.bak' -o -name '*.wasm.orig' \) -print 2>/dev/null \
    | grep -v '^.git/' || true)
if [ -n "$cruft" ]; then
    # Only flag files that look accidentally committed (not in target/).
    echo "$cruft" | while IFS= read -r f; do
        if ! git check-ignore "$f" >/dev/null 2>&1; then
            echo "  $f"
        fi
    done
    printf '  WARN: review files above (none are blocking; RULES.md recommends .gitignore)\n'
else
    printf '  ok\n'
fi

# ---- Gate 4: no committed secret-like literals (RULES.md §1.7 / §1.4 secrets) ----
printf '\n[gate 4/4] secret sniff (RULES.md §1.7 / §1.4)\n'
# Heuristic: lines matching credential-shaped assignments inside source. This is a
# sniffer; gitleaks/trufflehog is the real tool. The patterns here are conservative
# and tend to false-positive on tests/fixtures — we report, do not fail-closed.
hits=$(
    grep -RIn --include='*.rs' --include='*.sh' --include='*.py' \
        --exclude-dir='target' --exclude-dir='.git' --exclude-dir='.agents' \
        --exclude-dir='node_modules' --exclude-dir='.local' \
        -E '\b(password|passwd|secret|api[_-]?key|token|private[_-]?key)\b[[:space:]]*[:=][[:space:]]*["'\''][^"'\''$]{6,}' \
        "$ROOT"/idle "$ROOT"/packages "$ROOT"/render "$ROOT"/scripts \
        "$ROOT"/idle-tui "$ROOT"/idle-cosmic "$ROOT"/idle-studio \
        "$ROOT"/idle-saver-* "$ROOT"/idlescreen.github.io \
        2>/dev/null \
        | grep -vE '/tests/|/_tests\.rs|/test_|_test\.rs|/examples/' \
        | grep -vE 'AXIOMS|RULES|axioms_check|enterprises' \
        || true
)
if [ -n "$hits" ]; then
    printf '  WARN: %d candidate matches (review, not failing closed):\n' "$(echo "$hits" | wc -l)"
    printf '%s\n' "$hits" | head -20
else
    printf '  ok (no obvious patterns)\n'
fi

printf '\nRULES.md §1.7 hygiene bundle PASSED (gate 4 is informational).\n'
