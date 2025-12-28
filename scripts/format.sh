#!/bin/bash

# format.sh - Run cargo fmt for all subprojects

set -e

echo "Formatting pot-head workspace..."

# Root project
echo "  [1/7] Root project"
cargo fmt

# Examples
echo "  [2/7] examples/filtering"
(cd examples/filtering && cargo fmt)

echo "  [3/7] examples/interactive"
(cd examples/interactive && cargo fmt)

# Tools
echo "  [4/7] tools/sizeof-calculator"
(cd tools/sizeof-calculator && cargo fmt)

echo "  [5/7] tools/binary-analyzer/test-binary"
(cd tools/binary-analyzer/test-binary && cargo fmt)

echo "  [6/7] tools/benchmark/rp2040"
(cd tools/benchmark/rp2040 && cargo fmt)

echo "  [7/7] tools/benchmark/rp2350"
(cd tools/benchmark/rp2350 && cargo fmt)

echo ""
echo "Format complete!"
