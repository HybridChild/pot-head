#!/bin/bash

# format.sh - Run cargo fmt for all subprojects

set -e

echo "Formatting pot-head workspace..."

# Root project
echo "  [1/4] Root project"
cargo fmt

# Examples
echo "  [2/4] examples/filtering"
(cd examples/filtering && cargo fmt)

echo "  [3/4] examples/interactive"
(cd examples/interactive && cargo fmt)

# Tools
# echo "  [4/4] tools/sizeof-calculator"
# (cd tools/sizeof-calculator && cargo fmt)

echo ""
echo "Format complete!"
