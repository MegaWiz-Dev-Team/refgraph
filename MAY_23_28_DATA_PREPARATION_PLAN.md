# May 23-28 Data Preparation Plan

**Phase:** S1 Data Preparation  
**Duration:** 6 days (May 23-28, 2026)  
**Goal:** Load 9 real insurance documents and validate hit rate ≥75%  
**Status:** Ready to execute  

---

## 📋 Daily Breakdown

### **May 23 (Friday) - Prudential Documents**

**Task 1: Load Prudential Files**
- [ ] Source 3 Prudential insurance documents
- [ ] Verify file format (PDF/DOCX/TXT)
- [ ] Place in: `/tmp/prudential/` directory
- [ ] Document checksums for tracking

**Task 2: Extract & Consolidate**
```bash
./target/release/orchestrate ingest prudential-docs.json \
  --mimir-url=$MIMIR_URL \
  --tenant-id=asgard_insurance \
  --verbose
```
- [ ] Generate consolidated output
- [ ] Verify entity count: ~15-20 entities expected
- [ ] Verify relationship count: ~8-12 relationships expected
- [ ] Check average confidence: target >0.80

**Task 3: Validate**
```bash
./target/release/orchestrate validate \
  --mimir-url=$MIMIR_URL \
  --tenant-id=asgard_insurance \
  --k=3
```
- [ ] Run 5 test queries on Prudential data
- [ ] Target hit rate: 75-85%
- [ ] Log results

**Deliverable:**
- Consolidated JSON file: `prudential_consolidated_20260523.json`
- Hit rate report: `prudential_validation_20260523.md`

---

### **May 24 (Saturday) - AXA Documents**

**Task 1: Load AXA Files**
- [ ] Source 3 AXA insurance documents
- [ ] Verify file format
- [ ] Place in: `/tmp/axa/` directory
- [ ] Document checksums

**Task 2: Extract & Consolidate**
- [ ] Generate consolidated output
- [ ] Entity count: ~15-20 expected
- [ ] Relationship count: ~8-12 expected
- [ ] Avg confidence: >0.80

**Task 3: Validate**
- [ ] Run 5 test queries on AXA data
- [ ] Target hit rate: 75-85%
- [ ] Log results

**Deliverable:**
- Consolidated JSON file: `axa_consolidated_20260524.json`
- Hit rate report: `axa_validation_20260524.md`

---

### **May 25 (Sunday) - Thai Health Documents**

**Task 1: Load Thai Health Files**
- [ ] Source 3 Thai Health insurance documents
- [ ] Verify file format
- [ ] Place in: `/tmp/thai-health/` directory
- [ ] Document checksums

**Task 2: Extract & Consolidate**
- [ ] Generate consolidated output
- [ ] Entity count: ~15-20 expected
- [ ] Relationship count: ~8-12 expected
- [ ] Avg confidence: >0.80
- [ ] **Thai support verified** (Thai entities extracted)

**Task 3: Validate**
- [ ] Run 5 test queries on Thai Health data
- [ ] Include Thai language queries (ประกัน*)
- [ ] Target hit rate: 75-85%
- [ ] Log results

**Deliverable:**
- Consolidated JSON file: `thai_consolidated_20260525.json`
- Hit rate report: `thai_validation_20260525.md`

---

### **May 26-27 (Mon-Tue) - Combined Validation**

**Task 1: Merge All Consolidations**
```bash
# Combine all 9 documents into single request
./target/release/orchestrate pipeline \
  --input=all_consolidated.json \
  --mimir-url=$MIMIR_URL \
  --tenant-id=asgard_insurance
```
- [ ] Generate combined consolidated data
- [ ] Total entities expected: ~45-60
- [ ] Total relationships: ~24-36
- [ ] Overall avg confidence: >0.82

**Task 2: Full Hit Rate Validation**
- [ ] Run all 10 standard test queries
- [ ] Target combined hit rate: 75-87%
- [ ] Verify all query types work:
  - [ ] Lookup queries (e.g., "Critical Illness insurance")
  - [ ] Reasoning queries (e.g., "Health Insurance plan")
  - [ ] Exclusion queries (e.g., "Exclusions list")
  - [ ] Robustness queries (e.g., "Premium payment")
  - [ ] Thai queries (e.g., "ประกันสุขภาพ")

**Task 3: Quality Assessment**
- [ ] Check deduplication (Jaccard 0.95 threshold)
- [ ] Verify no data loss during consolidation
- [ ] Validate Neo4j graph relationships
- [ ] Check Thai language support

**Task 4: Documentation Review**
- [ ] Update CURRENT_STATUS.md
- [ ] Create combined validation report
- [ ] Document any issues or edge cases
- [ ] Prepare for June 2 execution

**Deliverable:**
- Combined consolidated JSON: `combined_consolidated_20260527.json`
- Full validation report: `combined_validation_report_20260527.md`

---

### **May 28 (Wednesday) - Final Preparation**

**Task 1: Production Readiness Check**
- [ ] Run full integration test with all 9 documents
- [ ] Verify hit rate ≥75%
- [ ] Check all CLI commands work
- [ ] Verify data format matches Mimir expectations

**Task 2: Documentation Completion**
- [ ] Create execution playbook for June 2-11
- [ ] Document any data quirks or special handling
- [ ] Create rollback procedures if needed
- [ ] Prepare team handoff notes

**Task 3: Final Decision Gate**
- [ ] Hit rate result: ✅ PASS (≥75%) or ⚠️ WARN (50-74%) or ❌ FAIL (<50%)
- [ ] If PASS: Proceed to June 2 execution
- [ ] If WARN: Execute remediation plan
- [ ] If FAIL: Activate contingency (unlikely given May 18 success)

**Deliverable:**
- Final status report: `MAY_28_FINAL_STATUS.md`
- Execution playbook: `JUNE_2_EXECUTION_PLAYBOOK.md`
- Go/No-Go decision: `MAY_28_DECISION.md`

---

## 📊 Success Criteria

### Minimum (Must Have)
- ✅ All 9 documents loaded and consolidated
- ✅ Hit rate ≥75% on combined data
- ✅ No data loss during consolidation
- ✅ All CLI commands functional

### Target (Should Have)
- ✅ Hit rate 75-87% (sweet spot)
- ✅ All 10 queries return results
- ✅ Thai queries work (ประกัน*)
- ✅ Deduplication working (no duplicates)
- ✅ Average confidence >0.82

### Stretch (Nice to Have)
- ✅ Hit rate >85%
- ✅ Perfect deduplication scores
- ✅ Zero false matches
- ✅ Optimized query times

---

## 🔧 Technical Requirements

### Environment Setup
```bash
# May 23 morning
export MIMIR_URL="http://localhost:8080"  # Or port-forward
export MIMIR_TOKEN="<oauth2-token>"        # If needed
export TENANT_ID="asgard_insurance"

cd /Users/mimir/Developer/refgraph-rs
```

### Key Commands
```bash
# Generate consolidated output from documents
./target/release/orchestrate ingest <json-file> \
  --mimir-url=$MIMIR_URL \
  --tenant-id=$TENANT_ID \
  --verbose

# Validate hit rate
./target/release/orchestrate validate \
  --mimir-url=$MIMIR_URL \
  --tenant-id=$TENANT_ID \
  --k=3

# Full pipeline
./target/release/orchestrate pipeline <json-file> \
  --mimir-url=$MIMIR_URL
```

---

## 📁 File Structure

```
/Users/mimir/Developer/refgraph-rs/
├── data/
│   ├── prudential/
│   │   ├── doc1.pdf
│   │   ├── doc2.pdf
│   │   └── doc3.pdf
│   ├── axa/
│   │   ├── doc1.pdf
│   │   ├── doc2.pdf
│   │   └── doc3.pdf
│   └── thai-health/
│       ├── doc1.pdf
│       ├── doc2.pdf
│       └── doc3.pdf
│
├── results/
│   ├── prudential_consolidated_20260523.json
│   ├── axa_consolidated_20260524.json
│   ├── thai_consolidated_20260525.json
│   ├── combined_consolidated_20260527.json
│   └── reports/
│       ├── prudential_validation_20260523.md
│       ├── axa_validation_20260524.md
│       ├── thai_validation_20260525.md
│       └── combined_validation_report_20260527.md
│
└── decisions/
    ├── MAY_28_FINAL_STATUS.md
    ├── JUNE_2_EXECUTION_PLAYBOOK.md
    └── MAY_28_DECISION.md
```

---

## ⚠️ Risk Mitigation

### If Hit Rate Drops Below 75%
1. **Root Cause Analysis**
   - Compare test results vs integration test (100% baseline)
   - Check document quality vs sample data
   - Verify deduplication is working

2. **Remediation Options**
   - Adjust Jaccard similarity threshold (0.95 → 0.90)
   - Fine-tune entity extraction patterns
   - Re-run consolidation with adjusted parameters
   - Escalate to Plan B if needed (May 19 approach)

### If Mimir Connectivity Issues
1. Verify OAuth2 token is valid
2. Check port-forward is active
3. Confirm Yggdrasil client is registered
4. Test health endpoint: `curl $MIMIR_URL/health`

### If Real Documents Are Missing
1. Use mock documents (already available from May 18)
2. Test pipeline with sample data
3. Proceed with mock validation
4. Re-run with real data when available

---

## 📅 Timeline Locks

```
✅ May 18:    Integration test PASS (100% hit rate)
⏳ May 23-28: Data preparation (THIS PLAN)
✅ June 2-11: Production execution (10-day window)
✅ June 12:   Final decision gate (Go-Live June 15+)
```

**Critical:** Stay on schedule. Any delays compress the June 2-11 execution window.

---

## 🚀 Expected Outcomes

By May 28 evening:
- 9 real documents consolidated
- Combined hit rate validated
- June 2-11 execution ready
- Team handoff complete
- Go-Live pathway confirmed

**Confidence:** 9.5/10 (based on May 18 100% success)

---

## 📞 Escalation

If anything blocks progress:
1. **Mimir Issues:** Check OAuth2 client registration, port-forward status
2. **Document Issues:** Use mock data, adjust timelines
3. **Hit Rate Issues:** Review consolidation quality, adjust parameters
4. **Timeline Pressure:** Escalate to leadership for go/no-go decision

---

**Status:** Ready to execute May 23  
**Owner:** Solo execution  
**Confidence:** 9.5/10  
**Last Updated:** May 17, 2026

