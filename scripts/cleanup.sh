#!/bin/bash

# cleanup.sh - Run cargo clean for all subprojects

set -e

echo "Cleaning pot-head workspace..."

# Root project
echo "  [1/5] Root project"
cargo clean

# Examples
echo "  [2/5] examples/filtering"
(cd examples/filtering && cargo clean)

echo "  [3/5] examples/interactive"
(cd examples/interactive && cargo clean)

# Tools
# echo "  [4/5] tools/sizeof-calculator"
# (cd tools/sizeof-calculator && cargo clean)

# Remove tmp directory if it exists
if [ -d "tmp" ]; then
    echo "  [5/5] Removing tmp directory"
    rm -rf tmp
else
    echo "  [5/5] tmp directory (not present)"
fi

echo ""
echo "Cleanup complete!"
