#!/usr/bin/env bash
set -e

# Strict file size checking.
# Fail if any file exceeds 250 lines.
# If --ratchet is passed, we check against a baseline or just fail if anything is > 250.

LIMIT=250

echo "Checking file sizes..."
OVERSIZE=$(find . -type f \( -name "*.rs" -o -name "*.c" -o -name "*.h" -o -name "*.sh" \) -not -path "*/target/*" -not -path "*/dist/*" -not -path "*/.git/*" -exec wc -l {} + | awk -v limit="$LIMIT" '$1 > limit && $2 != "total" {print $1, $2}')

if [ -n "$OVERSIZE" ]; then
    echo "ERROR: The following files exceed the $LIMIT line limit:"
    echo "$OVERSIZE"
    echo ""
    echo "Entropy (S) is too high. Split these files."
    exit 1
fi

echo "Success! No oversized files found."
exit 0
