#!/usr/bin/env bash
set -euo pipefail

echo "=== Updating all dependencies to latest ==="
cargo update

echo
echo "=== Checking for duplicate dependencies ==="
cargo tree -d > target/duplicates.txt

if [ -s target/duplicates.txt ]; then
  echo "WARNING: Duplicate versions detected"
else
  echo "No duplicate versions detected"
fi

echo
echo "=== Generating dependency report ==="
cargo metadata --format-version=1 \
| jq -r '
.packages[]
| select(.source != null)
| "      - \(.name) → resolved \(.version)"
' \
| sort \
> target/deps-report.txt

echo
echo "=== Report written to target/deps-report.txt ==="

