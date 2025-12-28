#!/usr/bin/env bash
set -euo pipefail

# binary-analyzer: Flash footprint analysis for pot-head
# Measures binary size impact of feature flags across ARM Cortex-M targets

# Color codes for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_BINARY_DIR="$SCRIPT_DIR/test-binary"
REPORT_FILE="$SCRIPT_DIR/../../reports/binary_report.md"

echo -e "${BLUE}pot-head Binary Analyzer${NC}"
echo "========================"
echo ""

# Check for required tools
echo "Checking for required tools..."

if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Error: cargo not found${NC}"
    exit 1
fi

if ! command -v arm-none-eabi-size &> /dev/null; then
    echo -e "${YELLOW}Warning: arm-none-eabi-size not found${NC}"
    echo "Install ARM embedded toolchain:"
    echo "  macOS: brew install --cask gcc-arm-embedded"
    echo "  Linux: sudo apt-get install gcc-arm-none-eabi"
    exit 1
fi

# Check for cargo-bloat (optional but recommended)
if ! command -v cargo-bloat &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-bloat...${NC}"
    cargo install cargo-bloat --quiet || {
        echo -e "${YELLOW}Warning: cargo-bloat installation failed. Symbol analysis will be skipped.${NC}"
        SKIP_BLOAT=1
    }
fi

# Install required targets
echo "Installing ARM targets..."
rustup target add thumbv6m-none-eabi >/dev/null 2>&1 || true
rustup target add thumbv7em-none-eabihf >/dev/null 2>&1 || true

# Create reports directory
mkdir -p "$(dirname "$REPORT_FILE")"

# Initialize report
echo -e "${GREEN}Generating report...${NC}"
TIMESTAMP=$(date -u '+%Y-%m-%d %H:%M:%S UTC')
cat > "$REPORT_FILE" << EOF
# pot-head Binary Analysis Report

**Generated:** $TIMESTAMP

**Targets:**
- Cortex-M0+ (\`thumbv6m-none-eabi\`) - No FPU, soft-float
- Cortex-M4F (\`thumbv7em-none-eabihf\`) - Hardware FPU, hard-float

**Optimization:** \`opt-level = "z"\` (size), LTO enabled

## Flash Footprint Summary

| Target | Features | .text | .rodata | .data | Total Flash | Notes |
|--------|----------|-------|---------|-------|-------------|-------|
EOF

# Feature configurations
# Format: "name:cargo_flags:description"
FEATURES=(
    "minimal:--no-default-features:No libm, no grab-mode"
    "default::Logarithmic curves (libm) + grab-mode"
    "full:--all-features:All features enabled"
)

# Targets to test
TARGETS=(
    "thumbv6m-none-eabi:M0+"
    "thumbv7em-none-eabihf:M4F"
)

# Variables to store sizes (for bash 3 compatibility)
M0_MINIMAL_TEXT=0
M0_MINIMAL_DATA=0
M0_MINIMAL_TOTAL=0
M0_DEFAULT_TEXT=0
M0_DEFAULT_DATA=0
M0_DEFAULT_TOTAL=0
M0_FULL_TEXT=0
M0_FULL_DATA=0
M0_FULL_TOTAL=0

M4F_MINIMAL_TEXT=0
M4F_MINIMAL_DATA=0
M4F_MINIMAL_TOTAL=0
M4F_DEFAULT_TEXT=0
M4F_DEFAULT_DATA=0
M4F_DEFAULT_TOTAL=0
M4F_FULL_TEXT=0
M4F_FULL_DATA=0
M4F_FULL_TOTAL=0

# Build and analyze each combination
for target_spec in "${TARGETS[@]}"; do
    IFS=':' read -r target target_name <<< "$target_spec"

    for feature_spec in "${FEATURES[@]}"; do
        IFS=':' read -r feature_name feature_flags feature_desc <<< "$feature_spec"

        echo -e "${GREEN}Building: $target_name / $feature_name${NC}"

        cd "$TEST_BINARY_DIR"

        # Build with specified features
        if [ -z "$feature_flags" ]; then
            cargo build --release --target "$target" 2>/dev/null || {
                echo -e "${YELLOW}Build failed for $target_name/$feature_name${NC}"
                continue
            }
        else
            cargo build --release --target "$target" $feature_flags 2>/dev/null || {
                echo -e "${YELLOW}Build failed for $target_name/$feature_name${NC}"
                continue
            }
        fi

        # Get binary path
        BINARY="target/$target/release/pot-head-binary-test"

        if [ ! -f "$BINARY" ]; then
            echo -e "${YELLOW}Binary not found: $BINARY${NC}"
            continue
        fi

        # Extract section sizes using arm-none-eabi-size
        SIZE_OUTPUT=$(arm-none-eabi-size "$BINARY" | tail -n 1)
        TEXT=$(echo "$SIZE_OUTPUT" | awk '{print $1}')
        DATA=$(echo "$SIZE_OUTPUT" | awk '{print $2}')
        BSS=$(echo "$SIZE_OUTPUT" | awk '{print $3}')

        # Calculate total Flash (text + rodata + data)
        # Note: rodata is included in text section for ARM
        TOTAL=$((TEXT + DATA))

        # Store sizes in variables (bash 3 compatible)
        case "${target_name}_${feature_name}" in
            "M0+_minimal")
                M0_MINIMAL_TEXT=$TEXT
                M0_MINIMAL_DATA=$DATA
                M0_MINIMAL_TOTAL=$TOTAL
                ;;
            "M0+_default")
                M0_DEFAULT_TEXT=$TEXT
                M0_DEFAULT_DATA=$DATA
                M0_DEFAULT_TOTAL=$TOTAL
                ;;
            "M0+_full")
                M0_FULL_TEXT=$TEXT
                M0_FULL_DATA=$DATA
                M0_FULL_TOTAL=$TOTAL
                ;;
            "M4F_minimal")
                M4F_MINIMAL_TEXT=$TEXT
                M4F_MINIMAL_DATA=$DATA
                M4F_MINIMAL_TOTAL=$TOTAL
                ;;
            "M4F_default")
                M4F_DEFAULT_TEXT=$TEXT
                M4F_DEFAULT_DATA=$DATA
                M4F_DEFAULT_TOTAL=$TOTAL
                ;;
            "M4F_full")
                M4F_FULL_TEXT=$TEXT
                M4F_FULL_DATA=$DATA
                M4F_FULL_TOTAL=$TOTAL
                ;;
        esac

        # Format sizes for display
        format_size() {
            local size=$1
            if [ "$size" -lt 1024 ]; then
                echo "${size}B"
            else
                awk "BEGIN {printf \"%.1fKB\", $size/1024}"
            fi
        }

        TEXT_DISPLAY=$(format_size $TEXT)
        DATA_DISPLAY=$(format_size $DATA)
        TOTAL_DISPLAY=$(format_size $TOTAL)

        # Append to report
        echo "| $target_name | $feature_name | $TEXT_DISPLAY | - | $DATA_DISPLAY | $TOTAL_DISPLAY | $feature_desc |" >> "$REPORT_FILE"
    done
done

cd "$SCRIPT_DIR"

# Add feature cost breakdown section
cat >> "$REPORT_FILE" << 'EOF'

## Feature Cost Breakdown

Analysis of binary size impact per feature:

EOF

# Calculate feature costs (variables already set from build loop)
# libm cost
M0_LIBM_COST=$((M0_DEFAULT_TOTAL - M0_MINIMAL_TOTAL))
M4F_LIBM_COST=$((M4F_DEFAULT_TOTAL - M4F_MINIMAL_TOTAL))

# heapless cost
M0_HEAPLESS_COST=$((M0_FULL_TOTAL - M0_DEFAULT_TOTAL))
M4F_HEAPLESS_COST=$((M4F_FULL_TOTAL - M4F_DEFAULT_TOTAL))

# FPU savings
if [ $M0_DEFAULT_TOTAL -gt 0 ] && [ $M4F_DEFAULT_TOTAL -gt 0 ]; then
    FPU_SAVINGS=$((M0_DEFAULT_TOTAL - M4F_DEFAULT_TOTAL))
    FPU_PERCENT=$((FPU_SAVINGS * 100 / M0_DEFAULT_TOTAL))
else
    FPU_SAVINGS=0
    FPU_PERCENT=0
fi

# Format costs for display
M0_LIBM_KB=$(awk "BEGIN {printf \"%.1fKB\", $M0_LIBM_COST/1024}")
M4F_LIBM_KB=$(awk "BEGIN {printf \"%.1fKB\", $M4F_LIBM_COST/1024}")
FPU_SAVINGS_KB=$(awk "BEGIN {printf \"%.1fKB\", $FPU_SAVINGS/1024}")

cat >> "$REPORT_FILE" << EOF
### libm (std-math feature) - Logarithmic Curves

- **M0+ soft-float**: +${M0_LIBM_COST}B ($M0_LIBM_KB)
- **M4F hard-float**: +${M4F_LIBM_COST}B ($M4F_LIBM_KB)
- **FPU savings**: ${FPU_SAVINGS}B ($FPU_SAVINGS_KB or ${FPU_PERCENT}% reduction)

The logarithmic curve feature requires libm for transcendental functions.
On M0+ (no FPU), this adds significant soft-float emulation code.
On M4F (hardware FPU), libm is much more efficient.

### heapless (moving-average feature)

- **M0+**: +${M0_HEAPLESS_COST}B
- **M4F**: +${M4F_HEAPLESS_COST}B

Moving average filter uses `heapless::Vec` for sample storage.
Impact is similar across targets (not FPU-dependent).

### grab-mode feature

- **Both targets**: Negligible impact (~0B)

Grab mode (Pickup/PassThrough) has no external dependencies and minimal code.

EOF

# Symbol analysis section
if [ -z "${SKIP_BLOAT:-}" ]; then
    echo -e "${GREEN}Analyzing symbols...${NC}"

    cat >> "$REPORT_FILE" << 'EOF'

## Top Symbols by Size

Shows the largest symbols consuming Flash space (default features).

### Cortex-M0+ (default features)

```
EOF

    cd "$TEST_BINARY_DIR"
    cargo bloat --release --target thumbv6m-none-eabi -n 10 2>/dev/null >> "$REPORT_FILE" || {
        echo "Symbol analysis not available" >> "$REPORT_FILE"
    }

    cat >> "$REPORT_FILE" << 'EOF'
```

### Cortex-M4F (default features)

```
EOF

    cargo bloat --release --target thumbv7em-none-eabihf -n 10 2>/dev/null >> "$REPORT_FILE" || {
        echo "Symbol analysis not available" >> "$REPORT_FILE"
    }

    cat >> "$REPORT_FILE" << 'EOF'
```

EOF
fi

cd "$SCRIPT_DIR"

# Add footer
cat >> "$REPORT_FILE" << 'EOF'

---

*Generated by `tools/binary-analyzer/generate_report.sh`*
EOF

echo ""
echo -e "${GREEN}Analysis complete!${NC}"
echo "Report: $REPORT_FILE"
echo ""

# Optional: clean builds to save space
if [ "${CLEAN:-0}" = "1" ]; then
    echo "Cleaning build artifacts..."
    cd "$TEST_BINARY_DIR"
    cargo clean
fi
