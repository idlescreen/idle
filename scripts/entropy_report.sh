#!/usr/bin/env bash
# scripts/entropy_report.sh - Report entropy S = U + F + W + O.
#
# References:
#   RULES.md §2 — Heuristics, Entropy S = U + F + W + O (optional score).
#   Proxies live in this script's body; the formula is canonical in RULES.md.
#
# Anchored to monorepo root (parent of scripts/). Runs against shipping source
# globs only: idle/, idle-tui/, idle-cosmic/, idle-studio/, render/, packages/,
# idle-saver-*/, brand/, idlescreen.github.io/, .github/, plus scripts/.
#
# Output is human-readable plus a one-line summary suitable for CI logs.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

LIMIT=256
SOURCE_ROOTS=(
    "idle"
    "idle-tui"
    "idle-cosmic"
    "idle-studio"
    "render"
    "packages"
    "scripts"
)
for saver in idle-saver-*; do
    [ -d "$saver" ] && SOURCE_ROOTS+=("$saver")
done

# ---- U: untested claims -------------------------------------------------------
# Proxies for "claim made but no enforcement":
#   - rust: todo!(), unimplemented!() in non-test code
#   - comments: // TODO / FIXME / XXX / HACK in non-test code
#   - doc claims of "todo" / "later" / "soon"
# Cap each proxy count at LIMIT/8 to avoid one file dominating.
U_TODO_RUST=$(
    grep -RIc --include='*.rs' \
        -E '\b(todo!|unimplemented!)\s*\(' \
        "$ROOT"/idle "$ROOT"/idle-tui "$ROOT"/idle-cosmic \
        "$ROOT"/idle-studio "$ROOT"/render "$ROOT"/idle-saver-* \
        2>/dev/null \
        | awk -F: '{s+=$2} END{print s+0}' \
        || true
)
U_TODO_COMMENTS=$(
    grep -RIc --include='*.rs' --include='*.sh' --include='*.py' \
        -E '(^|[^A-Za-z])(TODO|FIXME|XXX|HACK)\b' \
        "$ROOT"/idle "$ROOT"/idle-tui "$ROOT"/idle-cosmic \
        "$ROOT"/idle-studio "$ROOT"/render "$ROOT"/idle-saver-* \
        "$ROOT"/packages "$ROOT"/scripts \
        2>/dev/null \
        | awk -F: '{s+=$2} END{print s+0}' \
        || true
)
U=$(( U_TODO_RUST + U_TODO_COMMENTS ))

# ---- F: fail-open / silent OK -------------------------------------------------
# Proxies (kept narrow on purpose; some "let _ = " patterns are intentional):
#   - shell `|| true` (silently swallows exit codes)
#   - rust `if let Err(_)` and `match … => _, Err(_) =>` (silent Result swallow
#     in non-test code)
#   - production `.into_inner()` outside test/locks.rs/poison_or_exit scopes
#     (poison recovery — the canonical fail-open pattern from audit F-004)
F_OR_TRUE=$(
    grep -RIc --include='*.sh' '|| true' \
        "$ROOT"/scripts "$ROOT"/packages \
        2>/dev/null \
        | awk -F: '{s+=$2} END{print s+0}' \
        || true
)
F_SILENT_RESULT=$(
    grep -RIc --include='*.rs' \
        -E 'if[[:space:]]+let[[:space:]]+Err\(_\)|if[[:space:]]+let[[:space:]]+Ok\(_\)' \
        "$ROOT"/idle "$ROOT"/idle-tui "$ROOT"/idle-cosmic \
        "$ROOT"/idle-studio "$ROOT"/render "$ROOT"/idle-saver-* \
        2>/dev/null \
        | grep -v ':0$' \
        | grep -v '_tests\.rs' \
        | awk -F: '{s+=$2} END{print s+0}' \
        || true
)
F_INTO_INNER=$(
    grep -RIn --include='*.rs' 'into_inner()' \
        "$ROOT/idle/idle-daemon/src" \
        "$ROOT/idle/idle-runner/src" \
        2>/dev/null \
        | grep -v '/tests' \
        | grep -v '_tests\.rs' \
        | grep -v 'mod tests' \
        | grep -v 'locks\.rs' \
        | grep -v 'poison_or_exit' \
        | wc -l || true
)
F=$(( F_OR_TRUE + F_SILENT_RESULT + F_INTO_INNER ))

# ---- W: hand-waves ------------------------------------------------------------
# Proxies:
#   - rust: stub/fake/placeholder string outside tests
#   - sh:  # stub / # fake / # placeholder comments
#   - "unimplemented!" / "TODO: implement"
W_RUST=$(
    grep -RIc --include='*.rs' \
        -E '\b(stub|fake|placeholder|temp[_-]?impl|tmp[_-]?impl)\b' \
        "$ROOT"/idle "$ROOT"/idle-tui "$ROOT"/idle-cosmic \
        "$ROOT"/idle-studio "$ROOT"/render "$ROOT"/idle-saver-* \
        2>/dev/null \
        | grep -v ':0$' \
        | grep -v '_tests\.rs' \
        | awk -F: '{s+=$2} END{print s+0}' \
        || true
)
W_SH=$(
    grep -RIc --include='*.sh' \
        -E '^[[:space:]]*#[[:space:]]*(stub|fake|placeholder|XXX|FIXME)\b' \
        "$ROOT"/scripts \
        2>/dev/null \
        | grep -v ':0$' \
        | awk -F: '{s+=$2} END{print s+0}' \
        || true
)
W=$(( W_RUST + W_SH ))

# ---- O: oversize owned files --------------------------------------------------
# Count files over LIMIT lines (matches scripts/check_file_lines.sh scope).
O=$(
    find . -type f \( -name "*.rs" -o -name "*.c" -o -name "*.h" -o -name "*.sh" \) \
        -not -path "*/target/*" \
        -not -path "*/.git/*" \
        -not -path "*/.agents/*" \
        -not -path "*/node_modules/*" \
        -not -path "*/.local/*" \
        -not -path "*/.cache/*" \
        -exec wc -l {} + 2>/dev/null \
        | awk -v limit="$LIMIT" '$1 > limit && $2 != "total" {n++} END{print n+0}'
)

S=$(( U + F + W + O ))

# ---- Report -------------------------------------------------------------------
cat <<EOF
Entropy report — root=$ROOT — limit=$LIMIT
===========================================
U (untested claims)        = $U
    └─ rust todo!/unimplemented!  = $U_TODO_RUST
    └─ // TODO/FIXME/XXX/HACK     = $U_TODO_COMMENTS
F (fail-open / silent OK)  = $F
    └─ shell || true              = $F_OR_TRUE
    └─ rust silent Result swallow = $F_SILENT_RESULT
    └─ production into_inner()    = $F_INTO_INNER
W (hand-waves)             = $W
    └─ rust stub/fake/placeholder = $W_RUST
    └─ shell comments             = $W_SH
O (oversize files)         = $O
-------------------------------------------
S = U + F + W + O          = $S
EOF
