#!/bin/bash
# May 18 Integration Test - Automated Execution
# Purpose: Validate RefGraph system with real Mimir instance
# Time: ~30-40 minutes for complete flow

set -e

BINARY="./target/release/orchestrate"
MIMIR_URL="${MIMIR_URL:-http://localhost:5000}"
TENANT_ID="${TENANT_ID:-asgard_insurance}"
OUTPUT_FILE="test_consolidation_$(date +%Y%m%d_%H%M%S).json"
LOG_FILE="integration_test_$(date +%Y%m%d_%H%M%S).log"

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}May 18 Integration Test - RefGraph S1 System${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""
echo "Date: $(date)"
echo "Mimir URL: $MIMIR_URL"
echo "Tenant ID: $TENANT_ID"
echo "Log file: $LOG_FILE"
echo ""

# Create log file
touch "$LOG_FILE"

# ============================================================================
# Step 1: Health Check
# ============================================================================
echo -e "${BLUE}[Step 1/5]${NC} Health Check - Verify Mimir Connectivity"
echo "Command: $BINARY health --verbose --mimir-url=$MIMIR_URL"
echo ""

if $BINARY health --verbose --mimir-url="$MIMIR_URL" 2>&1 | tee -a "$LOG_FILE"; then
    echo -e "${GREEN}✅ PASS${NC}: Health check succeeded"
    echo ""
else
    echo -e "${RED}❌ FAIL${NC}: Health check failed - Mimir not accessible"
    echo "Troubleshooting:"
    echo "  1. Verify Mimir is running: docker ps | grep mimir"
    echo "  2. Check URL: $MIMIR_URL"
    echo "  3. Test manually: curl $MIMIR_URL/health"
    exit 1
fi

# ============================================================================
# Step 2: Sample Data Generation
# ============================================================================
echo -e "${BLUE}[Step 2/5]${NC} Sample Data Generation"
echo "Generating sample consolidation data from 9 insurance documents..."
echo ""

if $BINARY sample all-consolidations > "$OUTPUT_FILE" 2>/dev/null; then
    FILE_SIZE=$(ls -lh "$OUTPUT_FILE" | awk '{print $5}')
    ENTITY_COUNT=$(grep -c '"entity_id"' "$OUTPUT_FILE" || echo "unknown")
    echo -e "${GREEN}✅ PASS${NC}: Sample data generated"
    echo "   File: $OUTPUT_FILE"
    echo "   Size: $FILE_SIZE"
    echo "   Entities: $ENTITY_COUNT"
    echo ""
else
    echo -e "${RED}❌ FAIL${NC}: Sample data generation failed"
    exit 1
fi

# ============================================================================
# Step 3: Ingestion Test
# ============================================================================
echo -e "${BLUE}[Step 3/5]${NC} Data Ingestion - Upload to Mimir"
echo "Command: $BINARY ingest $OUTPUT_FILE --verbose --mimir-url=$MIMIR_URL --tenant-id=$TENANT_ID"
echo ""

if $BINARY ingest "$OUTPUT_FILE" --verbose --mimir-url="$MIMIR_URL" --tenant-id="$TENANT_ID" 2>&1 | tee -a "$LOG_FILE"; then
    echo -e "${GREEN}✅ PASS${NC}: Ingestion succeeded"
    echo ""
else
    echo -e "${RED}❌ FAIL${NC}: Ingestion failed"
    echo "Check log for details: cat $LOG_FILE"
    exit 1
fi

# ============================================================================
# Step 4: Hit Rate Validation
# ============================================================================
echo -e "${BLUE}[Step 4/5]${NC} Hit Rate Validation - Run 10 Test Queries"
echo "Running validation with standard insurance test queries..."
echo ""

VALIDATION_OUTPUT=$($BINARY validate --verbose --mimir-url="$MIMIR_URL" --tenant-id="$TENANT_ID" 2>&1)
echo "$VALIDATION_OUTPUT" | tee -a "$LOG_FILE"

# Extract hit rate from output (macOS compatible)
HIT_RATE=$(echo "$VALIDATION_OUTPUT" | grep "Hit Rate:" | sed -E 's/.*Hit.?Rate:? *([0-9.]+)%.*/\1/' | head -1)
if [[ -z "$HIT_RATE" ]]; then
  HIT_RATE=$(echo "$VALIDATION_OUTPUT" | grep -oE "Hit Rate: [0-9.]+" | grep -oE "[0-9.]+" | head -1)
fi
# Add % sign if not present
[[ "$HIT_RATE" != *"%" ]] && HIT_RATE="${HIT_RATE}%"

if [[ -z "$HIT_RATE" ]]; then
    echo -e "${YELLOW}⚠️  WARNING${NC}: Could not extract hit rate from output"
    echo "Manual review needed - check log: cat $LOG_FILE"
else
    HIT_RATE_NUM=$(echo "$HIT_RATE" | sed 's/%//g')

    if (( $(echo "$HIT_RATE_NUM >= 75" | bc -l) )); then
        echo -e "${GREEN}✅ PASS${NC}: Hit Rate $HIT_RATE ≥ 75% (production ready)"
        DECISION="PASS"
    elif (( $(echo "$HIT_RATE_NUM >= 50" | bc -l) )); then
        echo -e "${YELLOW}⚠️  WARN${NC}: Hit Rate $HIT_RATE in 50-74% range (review needed)"
        DECISION="WARN"
    else
        echo -e "${RED}❌ FAIL${NC}: Hit Rate $HIT_RATE < 50% (Plan B activation)"
        DECISION="FAIL"
    fi
fi

echo ""

# ============================================================================
# Step 5: End-to-End Pipeline
# ============================================================================
echo -e "${BLUE}[Step 5/5]${NC} End-to-End Pipeline - Complete Flow"
echo "Running complete pipeline (ingest + validate in sequence)..."
echo ""

PIPELINE_FILE="test_pipeline_$(date +%Y%m%d_%H%M%S).json"
cp "$OUTPUT_FILE" "$PIPELINE_FILE"

if $BINARY pipeline "$PIPELINE_FILE" --verbose --mimir-url="$MIMIR_URL" --tenant-id="$TENANT_ID" 2>&1 | tee -a "$LOG_FILE"; then
    echo -e "${GREEN}✅ PASS${NC}: End-to-end pipeline succeeded"
    echo ""
else
    echo -e "${RED}❌ FAIL${NC}: End-to-end pipeline failed"
    exit 1
fi

# ============================================================================
# Summary Report
# ============================================================================
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Integration Test Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""
echo "Test Execution: $(date)"
echo "Duration: $(date -d @$(($(date +%s) - $(stat -c %Y integration_test_*.log 2>/dev/null | head -1 || date +%s))) +'%M minutes')"
echo ""
echo "Results:"
echo "  [1/5] Health Check:        ✅ PASS"
echo "  [2/5] Sample Data:         ✅ PASS"
echo "  [3/5] Ingestion:           ✅ PASS"
echo "  [4/5] Validation:          ✅ ${DECISION:-MANUAL_REVIEW}"
if [[ "$DECISION" == "PASS" ]]; then
    echo "  [5/5] Pipeline:            ✅ PASS"
elif [[ "$DECISION" == "WARN" ]]; then
    echo "  [5/5] Pipeline:            ⚠️  WARN (review consolidation)"
else
    echo "  [5/5] Pipeline:            ❌ FAIL (Plan B activation)"
fi
echo ""

# Decision logic
echo -e "${BLUE}Decision Gate:${NC}"
if [[ "$DECISION" == "PASS" ]]; then
    echo -e "${GREEN}✅ PASS (Hit Rate ≥ 75%)${NC}"
    echo ""
    echo "Action: Proceed immediately with production"
    echo "Timeline:"
    echo "  - May 23-28: Data preparation (load Prudential/AXA/Thai Health)"
    echo "  - June 2-11: Production execution (daily runs)"
    echo "  - June 12: Final decision (Go-Live June 15)"
    echo ""
    DECISION_CODE=0
elif [[ "$DECISION" == "WARN" ]]; then
    echo -e "${YELLOW}⚠️  WARN (Hit Rate 50-74%)${NC}"
    echo ""
    echo "Action: Review consolidation quality"
    echo "Options:"
    echo "  1. Extend timeline, investigate May 20-22"
    echo "  2. Activate Plan B immediately (2-hour re-run)"
    echo ""
    DECISION_CODE=1
else
    echo -e "${RED}❌ FAIL (Hit Rate < 50%)${NC}"
    echo ""
    echo "Action: Activate Plan B immediately"
    echo "Steps:"
    echo "  1. Switch embedding: BGE-M3 → Typhoon"
    echo "  2. Re-run Phase 4 consolidation (~2 hours)"
    echo "  3. Re-validate hit rate"
    echo ""
    DECISION_CODE=2
fi

echo ""
echo "Files generated:"
echo "  - $OUTPUT_FILE (sample consolidation)"
echo "  - $PIPELINE_FILE (pipeline test data)"
echo "  - $LOG_FILE (complete execution log)"
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

exit $DECISION_CODE
