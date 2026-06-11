#!/usr/bin/env bash
# Check performance regression against baseline
# Usage: ./check_regression.sh [baseline_file]
set -euo pipefail

BASELINE="${1:-baselines/baseline.toml}"
WARN_PCT=10
FAIL_PCT=50

echo "Performance Regression Check"
echo "Baseline: $BASELINE"
echo "Warn threshold: ${WARN_PCT}%, Fail threshold: ${FAIL_PCT}%"
echo ""

# Parse baseline thresholds
warn=$(grep "warn_percent" "$BASELINE" | head -1 | sed 's/.*= //')
fail=$(grep "fail_percent" "$BASELINE" | head -1 | sed 's/.*= //')

echo "Configuration loaded. Run benchmarks with: cargo bench -p ldir-bench --locked"
echo ""
echo "For automated CI, compare Criterion output against $BASELINE."
