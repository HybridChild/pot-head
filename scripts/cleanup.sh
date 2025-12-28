#!/bin/bash

# cleanup.sh - Run cargo clean for all subprojects

set -e

echo "Cleaning pot-head workspace..."

# Root project
echo "  [1/8] Root project"
cargo clean

# Examples
echo "  [2/8] examples/filtering"
(cd examples/filtering && cargo clean)

echo "  [3/8] examples/interactive"
(cd examples/interactive && cargo clean)

# Tools
echo "  [4/8] tools/sizeof-calculator"
(cd tools/sizeof-calculator && cargo clean)

echo "  [5/8] tools/binary-analyzer/test-binary"
(cd tools/binary-analyzer/test-binary && cargo clean)

echo "  [6/8] tools/benchmark/rp2040"
(cd tools/benchmark/rp2040 && cargo clean)

echo "  [7/8] tools/benchmark/rp2350"
(cd tools/benchmark/rp2350 && cargo clean)

# Remove tmp directory if it exists
if [ -d "tmp" ]; then
    echo "  [8/8] Removing tmp directory"
    rm -rf tmp
else
    echo "  [8/8] tmp directory (not present)"
fi

echo ""
echo "Cleanup complete!"
