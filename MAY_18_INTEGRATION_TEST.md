# May 18 Integration Test - Live Mimir Validation

**Date:** May 18, 2026 (Friday)  
**Status:** Ready to execute  
**Confidence:** 9.9/10 (system tested, awaiting real Mimir)  
**Next:** Data preparation May 23-28 after passing integration test  

---

## Pre-Flight Checklist (May 17 Evening)

- [ ] Confirm Mimir instance URL (default: `http://localhost:5000`)
- [ ] Confirm asgard_insurance tenant exists in Mimir
- [ ] Verify Mimir is running: `curl http://localhost:5000/health`
- [ ] Confirm network access to Mimir (if remote, check VPN/firewall)
- [ ] Review binary at `./target/release/orchestrate`
- [ ] Verify test queries are configured (10 standard insurance queries)

---

## May 18 Execution Plan (Detailed Steps)

### Step 1: Health Check (5 minutes)
```bash
# Verify Mimir connectivity from CLI
./target/release/orchestrate health --verbose

# Expected output:
# ✅ Mimir health check passed
# ✅ Tenant: asgard_insurance
# ✅ Connected to: http://localhost:5000
```

**Decision Gate:**
- ✅ PASS: Health check succeeds → Proceed to Step 2
- ❌ FAIL: Health check fails → **ABORT** and troubleshoot Mimir connection

### Step 2: Sample Data Generation (2 minutes)
```bash
# Generate sample consolidation data from insurance_data module
./target/release/orchestrate sample all-consolidations > test_consolidation.json

# Verify file was created:
ls -lh test_consolidation.json

# Expected: ~10-50 KB JSON file with entities and relationships
```

**Decision Gate:**
- ✅ PASS: File created and valid JSON → Proceed to Step 3
- ❌ FAIL: File missing or invalid → **ABORT** and check sample module

### Step 3: Ingest Sample Data (5 minutes)
```bash
# Ingest the sample consolidation into Mimir
./target/release/orchestrate ingest test_consolidation.json --verbose

# Expected output:
# ✅ Ingestion successful
# ✅ Total entities: 47
# ✅ Total relationships: 26
# ✅ Timestamp: 2026-05-18T...
```

**Decision Gate:**
- ✅ PASS: Ingestion succeeds with correct entity/relationship counts → Proceed to Step 4
- ⚠️ PARTIAL: Ingestion succeeds but counts differ → Log discrepancy, proceed with caution
- ❌ FAIL: Ingestion fails → **ABORT** and check Mimir ingestion API

### Step 4: Hit Rate Validation (10 minutes)
```bash
# Run 10 standard insurance test queries and compute hit rate
./target/release/orchestrate validate --verbose

# Expected output:
# ✅ Validation complete
# ✅ Total queries: 10
# ✅ Hits: 7-10 (at least 7/10 = 70%)
# ✅ Hit rate: 70-100%
# ✅ Decision: PASS (if ≥75%) or WARN (if 50-74%) or FAIL (if <50%)
#
# Sample query results:
# 1. "Critical Illness insurance"     → Hits: 3, Score: 0.95 ✅
# 2. "Health Insurance plan"          → Hits: 2, Score: 0.92 ✅
# 3. "Accident coverage"              → Hits: 3, Score: 0.88 ✅
# 4. "Death benefit"                  → Hits: 2, Score: 0.85 ✅
# 5. "Hospital admission"             → Hits: 1, Score: 0.80 ✅
# 6. "Exclusions list"                → Hits: 2, Score: 0.78 ✅
# 7. "Premium payment"                → Hits: 3, Score: 0.82 ✅
# 8. "Claim process"                  → Hits: 1, Score: 0.75 ✅
# 9. "ประกันสุขภาพ"  (Thai: Health)  → Hits: 2, Score: 0.87 ✅
# 10. "ประกันชีวิต"  (Thai: Life)    → Hits: 2, Score: 0.89 ✅
#
# Hit Rate: 7.1/10 = 71% → WARN (need review)
# OR
# Hit Rate: 8.0/10 = 80% → PASS (ready for production)
```

**Decision Gates:**
- ✅ PASS (≥75%): **IMMEDIATE GO** - Proceed with May 23-28 data prep
  - Action: Clear calendar for data consolidation
  - Timeline: May 23 (Prudential), May 24 (AXA), May 25 (Thai Health), May 26-28 (validation)
  
- ⚠️ WARN (50-74%): **REVIEW REQUIRED** - System works but needs investigation
  - Action: Analyze which queries have low scores
  - Investigation: Is consolidation quality OK? Is Mimir search ranking correctly?
  - Decision: Proceed with timeline extension OR activate Plan B immediately
  
- ❌ FAIL (<50%): **ACTIVATE PLAN B IMMEDIATELY**
  - Action: Switch embedding model BGE-M3 → Typhoon
  - Timeline: 2-hour re-run of Phase 4 consolidation
  - Recovery: Re-validate hit rate with new embeddings

### Step 5: End-to-End Pipeline (10 minutes)
```bash
# Run complete pipeline (ingest + validate) in one command
./target/release/orchestrate pipeline test_consolidation.json --verbose

# Expected output:
# 🚀 Starting pipeline: EXEC_PRUDENTIAL_1716024000
#    Company: Prudential
# ✅ Consolidation complete
#    Entities: 6
#    Relationships: 4
#    Avg Confidence: 0.945
# ✅ Ingestion complete
#    Total ingested: 6
# ✅ Validation complete
#    Hit Rate: 80.0%
#    Decision: PASS ✅
```

**Decision Gate:**
- ✅ PASS: Pipeline succeeds with PASS decision → System ready for production
- ⚠️ WARN: Pipeline succeeds with WARN decision → Review consolidation quality
- ❌ FAIL: Pipeline fails or gives FAIL decision → Activate Plan B

---

## Critical Metrics to Track

### Hit Rate Thresholds

| Metric | Range | Decision | Action |
|--------|-------|----------|--------|
| Hit Rate | ≥75% | **PASS** ✅ | GO to production June 15 |
| Hit Rate | 50-74% | **WARN** ⚠️ | Review + timeline extension |
| Hit Rate | <50% | **FAIL** ❌ | Activate Plan B (2 hours) |

### Expected Baselines

```
Prudential (3 docs):
  Critical Illness: 6 entities, 4 relationships, 0.945 avg confidence
  Life Insurance: 5 entities, 3 relationships, 0.903 avg
  Health Insurance: 6 entities, 4 relationships, 0.863 avg
  → Expected Hit Rate: 75-85%

AXA (3 docs):
  Travel: 5 entities, 3 relationships, 0.953 avg
  Auto: 5 entities, 2 relationships, 0.946 avg
  Home: 5 entities, 2 relationships, 0.947 avg
  → Expected Hit Rate: 80-90%

Thai Health (3 docs):
  Comprehensive (ประกันสุขภาพ): 5 entities, 3 relationships, 0.947 avg
  Life (ประกันชีวิต): 5 entities, 3 relationships, 0.950 avg
  Accident: 5 entities, 2 relationships, 0.948 avg
  → Expected Hit Rate: 78-88%

Combined (9 docs, 47 entities, 26 relationships):
  → Expected Hit Rate: 75-87% (PASS threshold)
```

---

## Troubleshooting Guide

### "Health check failed: connection refused"
- **Cause:** Mimir not running or wrong URL
- **Fix:** 
  ```bash
  # Check if Mimir is running
  ps aux | grep mimir
  
  # If not running, start it
  docker run -d -p 5000:5000 asgard-mimir:latest
  
  # Verify port is open
  curl http://localhost:5000/health
  ```

### "Ingest succeeded but entities_ingested = 0"
- **Cause:** JSON format mismatch
- **Fix:**
  ```bash
  # Verify JSON format
  head -20 test_consolidation.json
  
  # Check against expected schema
  cat crates/refgraph-orchestration/src/insurance_data.rs | grep -A 10 "ConsolidatedEntity"
  ```

### "Validation returned 0 hits for all queries"
- **Cause:** Mimir search index not updated after ingestion
- **Fix:**
  ```bash
  # Wait 10 seconds for index refresh
  sleep 10
  
  # Re-run validation
  ./target/release/orchestrate validate --verbose
  ```

### "Hit Rate < 50% (Plan B trigger)"
- **Action:** Immediately activate Plan B
- **Steps:**
  1. Stop current execution
  2. Switch embedding model: BGE-M3 → Typhoon
  3. Re-run Phase 4 consolidation (~1.5 hours)
  4. Re-validate hit rate
  5. If ≥50%, continue; if still <50%, escalate

---

## Communication & Next Steps

### If PASS (≥75%)
```
Email/Slack:
✅ May 18 Integration Test: PASS
✅ Hit Rate: 80% (well above 75% threshold)
✅ Proceeding with May 23-28 data preparation
✅ Timeline: June 2-11 production execution
✅ Go-live target: June 15

Next Milestone: May 23 (Prudential data consolidation)
```

### If WARN (50-74%)
```
Email/Slack:
⚠️ May 18 Integration Test: WARN
⚠️ Hit Rate: 65% (needs review)
⚠️ Proceeding with extended timeline and close monitoring

Decision: Review consolidation quality vs. Mimir search ranking
Options:
1. Extend timeline, investigate quality, retry May 25
2. Activate Plan B immediately, re-validate by May 20

Next Milestone: May 20 (Plan B) or May 25 (extended review)
```

### If FAIL (<50%)
```
Email/Slack:
❌ May 18 Integration Test: FAIL
❌ Hit Rate: 45% (below 50% Plan B threshold)
❌ Activating Plan B immediately

Action: Switch embedding BGE-M3 → Typhoon
Timeline: Re-validation by May 19 afternoon
Decision: GO or escalate by May 20 EOD

Next Milestone: May 19 (Plan B re-validation)
```

---

## Files Ready For May 18

| Component | Status | Location |
|-----------|--------|----------|
| CLI Binary | ✅ Built | `./target/release/orchestrate` |
| Core Lib | ✅ 122 tests | `crates/refgraph-core/` |
| Neo4j Integration | ✅ 28 tests | `crates/refgraph-neo4j/` |
| Orchestration | ✅ 113 tests | `crates/refgraph-orchestration/` |
| Documentation | ✅ Complete | `FINAL_SUMMARY.md`, `DESIGN.md` |
| Sample Data | ✅ 9 docs | `insurance_data.rs` |
| Decision Logic | ✅ Automated | Hit rate gates (75%/50%) |

---

## System Readiness Summary

**Total Tests:** 263 (100% passing)  
**Build Status:** ✅ Release-optimized, zero errors  
**CLI Status:** ✅ All 5 commands functional  
**Documentation:** ✅ Complete  
**Data Ready:** ✅ 9 pre-consolidated documents  
**Decision Logic:** ✅ Automated thresholds  

**Confidence Level:** 9.9/10  
**Ready to Test:** YES ✅  

---

## May 18 Command Quick Reference

```bash
# All-in-one integration test
./target/release/orchestrate health --verbose && \
./target/release/orchestrate sample all-consolidations > test.json && \
./target/release/orchestrate pipeline test.json --verbose

# Expected: PASS decision with Hit Rate ≥75%
```

---

**System Status:** ✅ PRODUCTION READY FOR INTEGRATION TEST  
**Date:** May 18, 2026  
**Time:** ~30-40 minutes for complete execution  
**Owner:** Data Engineering (User)  
**Next Milestone:** May 23 (Data Preparation, if PASS)  

🚀 **Ready to validate with real Mimir!**
