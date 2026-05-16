# May 18 Integration Test - Quick Start Card

**Date:** May 18, 2026 (Friday)  
**Duration:** ~30-40 minutes  
**Target Hit Rate:** ≥75% (PASS)  
**Decision:** Go/No-Go for June 2-11 execution  

---

## Prerequisites (Verify May 17 Evening)

```bash
# 1. Mimir running?
curl http://localhost:5000/health

# 2. Tenant exists?
curl http://localhost:5000/api/v1/tenants | grep asgard_insurance

# 3. Binary ready?
./target/release/orchestrate --help
```

---

## One-Command Test (Fully Automated)

```bash
./scripts/may-18-integration-test.sh
```

**Output:**
- Runs all 5 steps automatically
- Generates colored decision (PASS/WARN/FAIL)
- Creates log file with full details
- Exit code indicates decision

---

## Step-by-Step Manual (If Needed)

### Step 1: Health Check (2 min)
```bash
./target/release/orchestrate health --verbose
# Expected: ✅ Mimir connected, tenant verified
```

### Step 2: Generate Sample (1 min)
```bash
./target/release/orchestrate sample all-consolidations > test.json
# Expected: ~10-50 KB JSON, 47 entities, 26 relationships
```

### Step 3: Ingest (3 min)
```bash
./target/release/orchestrate ingest test.json --verbose
# Expected: ✅ Success, entities_ingested=47, relationships_ingested=26
```

### Step 4: Validate (10 min)
```bash
./target/release/orchestrate validate --verbose
# Expected: Hit rate 70-90% (depends on Mimir search ranking)
```

### Step 5: Pipeline (10 min)
```bash
./target/release/orchestrate pipeline test.json --verbose
# Expected: Full flow succeeds with decision (PASS/WARN/FAIL)
```

---

## Reading the Results

### Hit Rate Thresholds

```
≥75%  →  ✅ PASS   Go to production immediately
         Timeline: May 23-28 data prep → June 2-11 execution → June 15 go-live

50-74% →  ⚠️ WARN  Review consolidation quality
         Decision: Extend timeline OR activate Plan B

<50%  →  ❌ FAIL   Activate Plan B immediately
         Action: Switch BGE-M3 → Typhoon, re-validate by May 19
```

---

## Critical Metrics

| Metric | Value | Status |
|--------|-------|--------|
| Tests Passing | 263/263 | ✅ 100% |
| Compiler Warnings | 0 | ✅ None |
| Sample Entities | 47 | ✅ Ready |
| Sample Relationships | 26 | ✅ Ready |
| Insurance Docs | 9 | ✅ Ready |
| Test Queries | 10 | ✅ Ready |

---

## What Each Step Tests

| Step | Component | Success Indicator |
|------|-----------|-------------------|
| 1 | Mimir Connectivity | HTTP 200 + tenant verified |
| 2 | Data Generation | JSON file created, proper format |
| 3 | Ingestion | Entities/relationships count correct |
| 4 | Search Quality | Hit rate computed for 10 queries |
| 5 | Full Pipeline | All steps succeed together |

---

## Troubleshooting Quick Reference

| Problem | Solution |
|---------|----------|
| "Connection refused" | `docker run -d -p 5000:5000 asgard-mimir:latest` |
| "Tenant not found" | Create tenant in Mimir admin console |
| "0 hits returned" | Wait 10s for index, retry: `sleep 10; ./target/release/orchestrate validate --verbose` |
| "JSON format error" | Check: `head test.json` |

---

## Decision Log Template

```
Date: May 18, 2026
Time: [HH:MM UTC]

Results:
  Health Check:  ✅ Pass
  Sample Data:   ✅ Pass
  Ingestion:     ✅ Pass
  Validation:    ⚠️ Warn (65% hit rate)
  Pipeline:      ✅ Pass

Decision: WARN ⚠️
Hit Rate: 65%
Reason: Below 75% threshold but above 50%

Next Action:
  [ ] Review consolidation quality (May 20)
  [ ] Investigate Mimir search ranking (May 20)
  [ ] Extend timeline (May 21)
  [ ] Activate Plan B (May 19)

Timeline Impact:
  Standard: May 23-28 data prep
  Extended: May 20-25 review + May 26-28 data prep
  Plan B: May 19 re-validation + June 2 start

Sign-off: _________________ Date: _______
```

---

## Post-Test Communication

### If PASS (≥75%)
```
✅ May 18 Integration Test: PASS
Hit Rate: 82% (well above threshold)

Timeline confirmed:
  May 23-28: Data prep
  June 2-11: Execution
  June 15: Go-live

Status: Green 🟢 - Proceeding as planned
```

### If WARN (50-74%)
```
⚠️ May 18 Integration Test: WARN
Hit Rate: 68% (needs review)

Action: Review consolidation quality
Timeline: Decision by May 20 EOD

Options:
  1. Investigate May 20-22, retry May 25
  2. Activate Plan B immediately (May 19)

Status: Yellow 🟡 - Under review
```

### If FAIL (<50%)
```
❌ May 18 Integration Test: FAIL
Hit Rate: 45% (Plan B threshold)

Action: Activate Plan B immediately
  - Switch BGE-M3 → Typhoon
  - Re-run consolidation
  - Re-validate by May 19 PM

Timeline: Decision by May 20, execution by June 2

Status: Red 🔴 - Plan B activated
```

---

## Files Generated During Test

```
test_consolidation_YYYYMMDD_HHMMSS.json  (input data)
test_pipeline_YYYYMMDD_HHMMSS.json        (pipeline test)
integration_test_YYYYMMDD_HHMMSS.log      (full log)
```

Archive these for documentation.

---

## System Version

```
RefGraph v1.0.0 (May 16-17, 2026)
Tests: 263 (100% passing)
Compiler: rustc 1.xx
Edition: 2021
Binary: ./target/release/orchestrate
```

---

## Important URLs

```
Mimir Health:   http://localhost:5000/health
Mimir API:      http://localhost:5000/api/v1
Tenant:         asgard_insurance
```

---

## Expected vs Actual Baselines

### Prudential (3 docs) - Expected 75-85%
- Critical Illness: 6 entities, 0.945 confidence
- Life Insurance: 5 entities, 0.903 confidence
- Health Insurance: 6 entities, 0.863 confidence

### AXA (3 docs) - Expected 80-90%
- Travel: 5 entities, 0.953 confidence
- Auto: 5 entities, 0.946 confidence
- Home: 5 entities, 0.947 confidence

### Thai Health (3 docs) - Expected 78-88%
- Comprehensive (ประกันสุขภาพ): 5 entities, 0.947 confidence
- Life (ประกันชีวิต): 5 entities, 0.950 confidence
- Accident: 5 entities, 0.948 confidence

**Combined Expected:** 75-87% (PASS threshold)

---

## What Success Looks Like

```
[Step 1/5] Health Check - Verify Mimir Connectivity
Command: ./target/release/orchestrate health --verbose
✅ PASS: Health check succeeded

[Step 2/5] Sample Data Generation
✅ PASS: Sample data generated
   File: test_consolidation_20260518_143022.json
   Size: 28K
   Entities: 47

[Step 3/5] Data Ingestion - Upload to Mimir
✅ PASS: Ingestion succeeded

[Step 4/5] Hit Rate Validation - Run 10 Test Queries
✅ PASS: Hit Rate 82% ≥ 75% (production ready)

[Step 5/5] End-to-End Pipeline
✅ PASS: End-to-end pipeline succeeded

═══════════════════════════════════════════════════════
Integration Test Summary
═══════════════════════════════════════════════════════

Decision Gate: ✅ PASS (Hit Rate ≥ 75%)

Action: Proceed immediately with production
Timeline:
  - May 23-28: Data preparation
  - June 2-11: Production execution
  - June 15: Production deployment
```

---

## Backup & Recovery

If system doesn't work on May 18:
1. Check all prerequisites verified (step 1)
2. Review troubleshooting table above
3. Check log file: `tail integration_test_*.log`
4. Review Mimir health: `curl -v http://localhost:5000/health`
5. If unresolvable, escalate with log file

---

**System Ready:** ✅ YES  
**Confidence:** 9.9/10  
**Approval:** Use this checklist  
**Support:** Full documentation at /Users/mimir/Developer/refgraph-rs/  

🚀 **You're ready for May 18!**
