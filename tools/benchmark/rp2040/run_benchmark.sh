#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "Running RP2040 benchmark..."
echo "Make sure RP2040 hardware is connected and probe-rs is configured."
echo ""

OUTPUT_FILE="$SCRIPT_DIR/../../../reports/rp2040_benchmarks.md"
TEMP_FILE=$(mktemp)
trap "rm -f $TEMP_FILE" EXIT

# Create reports directory if it doesn't exist
mkdir -p "$(dirname "$OUTPUT_FILE")"

# Run cargo in background, capture to temp file
cargo run --release > "$TEMP_FILE" 2>&1 &
CARGO_PID=$!

# Wait for benchmark to complete
sleep 8

# Kill the process
kill $CARGO_PID 2>/dev/null || true
wait $CARGO_PID 2>/dev/null || true

# Generate output file, filtering out termination messages
{
    echo "# RP2040 Benchmark Results"
    echo ""
    echo "> **Note:** These are reference results from specific hardware. Performance may vary"
    echo "> depending on your hardware revision, toolchain version, and environmental factors."
    echo "> Run the benchmark yourself for accurate results on your setup."
    echo ""
    echo "**Last Updated:** $(date '+%Y-%m-%d %H:%M:%S')  "
    echo "**Toolchain:** $(rustc --version)  "
    echo "**Hardware:** Raspberry Pi Pico (RP2040)  "
    echo "**Target:** thumbv6m-none-eabi (Cortex-M0+, no FPU)  "
    echo "**Optimization:** --release (LTO enabled)"
    echo ""
    echo "## Results"
    echo ""
    echo '```'

    grep -v "Received SIGTERM" "$TEMP_FILE" | grep -v "Exited by user request" | grep -v "Benchmark complete"

    echo '```'
} > "$OUTPUT_FILE"

echo ""
echo "Results saved to $OUTPUT_FILE"
