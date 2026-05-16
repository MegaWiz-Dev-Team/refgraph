# RefGraph Orchestration CLI - Usage Guide

## Overview

The `orchestrate` CLI tool manages the complete RefGraph pipeline:
```
Consolidation Output → Mimir Ingestion → Hit Rate Validation
```

## Building

```bash
cargo build -p refgraph-orchestration --bin orchestrate --release
```

Binary location: `target/release/orchestrate`

## Global Options

All commands support these flags:

```
--mimir-url <URL>    Mimir API base URL [default: http://localhost:5000]
--tenant-id <ID>     Tenant identifier [default: asgard_insurance]
--verbose            Enable detailed output
```

## Commands

### 1. Ingest Consolidated Output

Submit RefGraph consolidation to Mimir API.

**Usage:**
```bash
orchestrate ingest <INPUT_FILE> [--output <OUTPUT_FILE>]
```

**Example:**
```bash
orchestrate ingest consolidation.json \
  --mimir-url http://localhost:5000 \
  --tenant-id asgard_insurance \
  --output ingestion-response.json
```

**Input Format (JSON):**
```json
{
  "tenant_id": "asgard_insurance",
  "domain": "insurance",
  "entity_count": 25,
  "relationship_count": 15,
  "average_confidence": 0.876,
  "entities": [
    {
      "id": "ent_001",
      "type": "PRODUCT",
      "text": "Critical Illness Insurance",
      "confidence": 0.92
    }
  ],
  "relationships": [
    {
      "source": "ent_001",
      "target": "ent_002",
      "type": "HAS_COVERAGE",
      "confidence": 0.85
    }
  ]
}
```

**Output:**
```
📥 Ingesting consolidated output from: consolidation.json
✅ Ingestion successful
   Entities: 25
   Relationships: 15
   Avg Confidence: 0.876
   Response saved to: ingestion-response.json
⏱️  Completed in 123.45ms
```

### 2. Validate Hit Rate

Execute test queries and compute validation decision.

**Usage:**
```bash
orchestrate validate [--queries <QUERIES>] [--k <K>]
```

**Parameters:**
- `--queries`: Comma-separated test queries (optional, uses 10 standard queries if not provided)
- `--k`: Top-k results per query [default: 3]

**Example - Standard 10 Queries:**
```bash
orchestrate validate \
  --mimir-url http://localhost:5000 \
  --tenant-id asgard_insurance
```

**Example - Custom Queries:**
```bash
orchestrate validate \
  --queries "Critical Illness,Heart Attack,Health Insurance" \
  --k 5
```

**Output:**
```
🔍 Running validation queries (k=3)
📋 10 queries to execute
✅ Validation complete
   Hit Rate: 80.0%
   Hits: 8/10
   Decision: Pass
🎉 PASS: Ready for production
⏱️  Completed in 456.78ms
```

**Standard Test Queries (10 total):**

| Tier | English | Thai | Query |
|------|---------|------|-------|
| Lookup | ✓ | | Critical Illness insurance |
| Lookup | ✓ | | Health Insurance plan |
| Reasoning | ✓ | | Accident coverage |
| Reasoning | ✓ | | Death benefit |
| Exclusion | ✓ | | Hospital admission |
| Exclusion | ✓ | | Exclusions list |
| Robustness | ✓ | | Premium payment |
| Robustness | ✓ | | Claim process |
| Thai | | ✓ | ประกันสุขภาพ (Health insurance) |
| Thai | | ✓ | ประกันชีวิต (Life insurance) |

**Decision Gates:**
- **PASS** (≥75%): Proceed to production ✅
- **WARN** (50-74%): Review required ⚠️
- **FAIL** (<50%): Activate Plan B ❌

### 3. End-to-End Pipeline

Execute complete flow: ingest → validate in one command.

**Usage:**
```bash
orchestrate pipeline <INPUT_FILE> [--queries <QUERIES>]
```

**Example:**
```bash
orchestrate pipeline consolidation.json \
  --mimir-url http://localhost:5000 \
  --tenant-id asgard_insurance
```

**Output:**
```
🔗 Running end-to-end pipeline

[1/2] Ingestion Phase
      ✅ Ingested 25 entities

[2/2] Validation Phase
      ✅ Validation complete
      Hit Rate: 80.0%
      Decision: Pass

✨ Pipeline complete in 456.78ms
```

### 4. Sample Data

Generate or display sample insurance documents for testing.

**Usage:**
```bash
orchestrate sample <COMMAND>
```

**Subcommands:**

#### Critical Illness Document
```bash
orchestrate sample critical-illness
```

Shows sample Critical Illness Insurance policy with:
- Coverage details (Heart Attack, Stroke, Cancer, etc.)
- Premium information
- Exclusions

#### Health Insurance Document
```bash
orchestrate sample health-insurance
```

Shows sample Health Insurance plan with:
- Hospital admission coverage
- Death benefit
- Claim process steps
- Premium payment terms

#### Thai Insurance Document
```bash
orchestrate sample thai
```

Shows sample Thai insurance document with:
- ประกันสุขภาพ (Health insurance)
- ประกันชีวิต (Life insurance)
- Mixed English/Thai content

#### All Consolidations
```bash
orchestrate sample all-consolidations
```

Outputs JSON consolidation for both sample documents:
- 7 total entities
- 5 total relationships
- Ready to use with `ingest` command

**Example Use:**
```bash
# Generate consolidation
orchestrate sample all-consolidations > samples.json

# Ingest the sample
orchestrate ingest samples.json
```

### 5. Health Check

Display Mimir connection status and pipeline statistics.

**Usage:**
```bash
orchestrate health
```

**Output:**
```
🏥 Health Check
   Mimir URL: http://localhost:5000
   Tenant: asgard_insurance
   Total Ingested: 5
   Total Failed: 1
   Last Ingestion: 2026-05-16T10:30:45.123456Z
```

## Workflow Examples

### Scenario 1: Validate Existing Data

```bash
# Check if current Mimir data meets requirements
orchestrate validate \
  --mimir-url http://mimir.internal:5000 \
  --tenant-id asgard_insurance \
  --verbose

# Expected: Pass (80%+) for production, else Plan B
```

### Scenario 2: Process New Document

```bash
# RefGraph consolidates document → consolidation.json
# Then:

orchestrate pipeline consolidation.json \
  --mimir-url http://mimir.internal:5000 \
  --tenant-id asgard_insurance \
  --verbose

# Ingest + validate in one step
```

### Scenario 3: Test with Custom Queries

```bash
# Run validation with specific domain queries
orchestrate validate \
  --queries "Prudential coverage,AXA exclusions,Thai Health premium" \
  --k 3 \
  --mimir-url http://mimir.internal:5000

# Check hit rate for insurer-specific terms
```

### Scenario 4: Generate Sample Consolidations

```bash
# Create test data
orchestrate sample all-consolidations > test-data.json

# Ingest it
orchestrate ingest test-data.json \
  --output response.json \
  --verbose

# Validate
orchestrate validate --verbose
```

## Error Handling

### Connection Error
```
❌ Ingestion failed: Request failed: connection refused
```
**Solution:** Verify Mimir URL is correct and Mimir is running

### Invalid JSON
```
❌ Invalid JSON format: expected value at line 1 column 0
```
**Solution:** Ensure input file is valid JSON

### Unauthorized
```
❌ Ingestion failed: Unauthorized: check Mimir API credentials
```
**Solution:** Verify tenant_id and authentication

### Not Found
```
❌ Ingestion failed: Endpoint not found: http://localhost:5000/api/v1/tenants/...
```
**Solution:** Verify Mimir API version and endpoint paths

## Performance

Typical latencies:
- **Ingest**: 100-500ms (depends on network + Mimir processing)
- **Validate** (10 queries): 300-1000ms (Mimir search latency)
- **Pipeline**: 400-1500ms (ingest + validate)

## Integration with RefGraph

The orchestration tool expects input from RefGraph consolidation phase:

```
refgraph-core consolidate
    ↓
  JSON output
    ↓
orchestrate ingest
    ↓
  Mimir API
    ↓
orchestrate validate
    ↓
  Decision (PASS/WARN/FAIL)
```

## Advanced Usage

### Batch Processing

```bash
#!/bin/bash
for doc in documents/*.json; do
  echo "Processing $doc..."
  orchestrate pipeline "$doc" \
    --mimir-url http://mimir.internal:5000 \
    --verbose
done
```

### Hit Rate Monitoring

```bash
#!/bin/bash
# Run validation on schedule and log results
orchestrate validate \
  --mimir-url http://mimir.internal:5000 \
  --tenant-id asgard_insurance | \
  tee -a validation-$(date +%Y-%m-%d).log
```

### Multi-Tenant Support

```bash
# Ingest to different tenants
orchestrate ingest consolidation.json \
  --tenant-id asgard_insurance

orchestrate ingest consolidation.json \
  --tenant-id asgard_medical
```

## Exit Codes

- **0**: Success
- **1**: Error (invalid input, connection failure, etc.)
- **2**: Validation failure (FAIL decision triggered)

## Docker Usage

```dockerfile
FROM rust:latest
COPY . /app
WORKDIR /app
RUN cargo build -p refgraph-orchestration --release
ENTRYPOINT ["/app/target/release/orchestrate"]
```

```bash
docker build -t refgraph-orchestrate .
docker run refgraph-orchestrate health \
  --mimir-url http://mimir:5000
```

---

**Last Updated:** May 16, 2026  
**Version:** v0.1.0
