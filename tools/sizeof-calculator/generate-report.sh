#!/usr/bin/env bash
set -euo pipefail

# Simple script to generate sizeof report for ARM Cortex-M target

TARGET=${1:-thumbv7em-none-eabihf}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_FILE="${SCRIPT_DIR}/../../reports/sizeof_report.md"

echo "Building for $TARGET..."
cd "$SCRIPT_DIR"

# Ensure target is installed
rustup target add "$TARGET" >/dev/null 2>&1 || true

# Build the embedded binary
cargo build --release --target "$TARGET" --bin sizeof-embedded

# Use objdump to dump .rodata and parse values
BINARY="target/${TARGET}/release/sizeof-embedded"

# Extract the hex dump lines
RODATA=$(arm-none-eabi-objdump -s -j .rodata "$BINARY" | grep "^ ")

# Parse little-endian 32-bit values from first two lines
# Line 1: 100e4 2c000000 28000000 28000000 e4000000
# Line 2: 100f4 e0000000 e0000000 b8000000
parse_le32() {
    local hex=$1
    printf "%d" "0x${hex:6:2}${hex:4:2}${hex:2:2}${hex:0:2}"
}

# Get hex values from the dump
VALUES=$(echo "$RODATA" | awk '{for(i=2;i<=5;i++) if($i!="") print $i}')

# Convert to array
IFS=$'\n' read -d '' -r -a HEX_ARRAY <<< "$VALUES" || true

SIZE_CONFIG_U16_F32=$(parse_le32 "${HEX_ARRAY[0]}")
SIZE_CONFIG_U16_U16=$(parse_le32 "${HEX_ARRAY[1]}")
SIZE_CONFIG_U8_F32=$(parse_le32 "${HEX_ARRAY[2]}")
SIZE_POTHEAD_U16_F32=$(parse_le32 "${HEX_ARRAY[3]}")
SIZE_POTHEAD_U16_U16=$(parse_le32 "${HEX_ARRAY[4]}")
SIZE_POTHEAD_U8_F32=$(parse_le32 "${HEX_ARRAY[5]}")
SIZE_STATE_F32=$(parse_le32 "${HEX_ARRAY[6]}")

# Map target to description
case "$TARGET" in
    thumbv6m-none-eabi)
        TARGET_DESC="ARM Cortex-M0/M0+"
        ;;
    thumbv7m-none-eabi)
        TARGET_DESC="ARM Cortex-M3"
        ;;
    thumbv7em-none-eabi)
        TARGET_DESC="ARM Cortex-M4/M7 (no FPU)"
        ;;
    thumbv7em-none-eabihf)
        TARGET_DESC="ARM Cortex-M4F/M7 (with FPU)"
        ;;
    *)
        TARGET_DESC="$TARGET"
        ;;
esac

# Generate report
cat > "$REPORT_FILE" << EOF
# pot-head sizeof Report

**Generated:** $(date -u '+%Y-%m-%d %H:%M:%S UTC')

**Target:** $TARGET_DESC (\`$TARGET\`)

## Component Sizes

| Type | Size (bytes) |
|------|--------------|
| Config<u16, f32> | $SIZE_CONFIG_U16_F32 |
| Config<u16, u16> | $SIZE_CONFIG_U16_U16 |
| Config<u8, f32> | $SIZE_CONFIG_U8_F32 |
| State<f32> | $SIZE_STATE_F32 |
| PotHead<u16, f32> | $SIZE_POTHEAD_U16_F32 |
| PotHead<u16, u16> | $SIZE_POTHEAD_U16_U16 |
| PotHead<u8, f32> | $SIZE_POTHEAD_U8_F32 |

## Common Configurations

Real-world embedded usage scenarios:

| Use Case | Type Combo | Est. Total RAM |
|----------|------------|----------------|
| Simple volume knob | u16→f32 | ~$SIZE_POTHEAD_U16_F32 bytes |
| Integer scaler | u16→u16 | ~$SIZE_POTHEAD_U16_U16 bytes |
| Compact config | u8→f32 | ~$SIZE_POTHEAD_U8_F32 bytes |

## Summary

- **Config storage** (flash): $(($SIZE_CONFIG_U16_F32))-$(($SIZE_CONFIG_U16_F32)) bytes (depends on type parameters)
- **Runtime state** (RAM): $SIZE_STATE_F32 bytes per instance
- **Typical usage**: ~$SIZE_POTHEAD_U16_F32 bytes total per potentiometer

These measurements are for **$TARGET_DESC** and represent actual embedded target sizes.

---

*Run \`tools/sizeof-calculator/generate-report.sh\` to regenerate this report.*
*Optionally specify a target: \`generate-report.sh thumbv6m-none-eabi\`*
EOF

echo "Report generated at: $REPORT_FILE"
echo ""
cat "$REPORT_FILE"
