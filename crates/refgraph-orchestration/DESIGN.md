# RefGraph Orchestration Layer - Design & Implementation

**Created:** May 16, 2026  
**Status:** ✅ RED/GREEN/REFACTOR complete (38 tests passing)  
**Version:** v0.1.0  

## Overview

The orchestration layer manages the complete RefGraph pipeline:
```
RefGraph Consolidated Output → Mimir Ingestion → Hit Rate Validation
```

### Core Components

#### 1. **PipelineOrchestrator**
Submits consolidated RefGraph output to Mimir.

```rust
let orchestrator = PipelineOrchestrator::new(
    "http://localhost:5000".to_string(),
    "asgard_insurance".to_string(),
);

let request = MimirIngestionRequest {
    tenant_id: "asgard_insurance".to_string(),
    domain: "insurance".to_string(),
    entity_count: 25,
    relationship_count: 15,
    average_confidence: 0.876,
    entities: vec![...],
    relationships: vec![...],
};

orchestrator.ingest(request).await?;
```

**Status Tracking:**
- `total_ingested`: Count of successful submissions
- `total_failed`: Count of failed submissions
- `last_ingestion_timestamp`: RFC3339 timestamp of last operation

#### 2. **HitRateValidator**
Executes test queries against Mimir and computes decision gates.

```rust
let validator = HitRateValidator::new(
    "http://localhost:5000".to_string(),
    "asgard_insurance".to_string(),
    vec![
        "Critical Illness insurance".to_string(),
        "ประกันสุขภาพ".to_string(), // Thai: health insurance
        // ... 10 standard queries total
    ],
);

let result = validator.validate().await?;

match result.decision {
    ValidationDecision::Pass => { /* GO to production */ }
    ValidationDecision::Warn => { /* Review results */ }
    ValidationDecision::Fail => { /* Activate Plan B: switch embedding model */ }
}
```

**Decision Thresholds:**
- **PASS**: Hit Rate@3 ≥ 75% → Proceed to production
- **WARN**: Hit Rate@3 50-74% → Review & escalate
- **FAIL**: Hit Rate@3 < 50% → Activate Plan B (switch BGE-M3 → Typhoon)

#### 3. **MimirClient**
HTTP client for Mimir interaction.

```rust
let client = MimirClient::new(
    "http://localhost:5000".to_string(),
    "asgard_insurance".to_string(),
);

// Ingest RefGraph output
client.ingest(&request).await?;

// Search with hit count
let result = client.search("insurance coverage", 3).await?;
println!("Hits: {}, Score: {}", result.hits, result.score);

// Health check
let healthy = client.health_check().await?;
```

### Data Structures

#### HitRateResult
```rust
pub struct HitRateResult {
    pub total_queries: usize,      // Total test queries executed
    pub hit_count: usize,          // Queries with ≥1 hit
    pub hit_rate: f32,             // hit_count / total_queries
    pub decision: ValidationDecision,
    pub query_results: Vec<QueryResult>,
}
```

#### QueryResult
```rust
pub struct QueryResult {
    pub query: String,             // The test query
    pub hits: usize,               // Number of hits in top-k
    pub score: f32,                // Average confidence of hits
    pub timestamp: String,         // RFC3339 timestamp
}
```

#### MimirIngestionRequest
```rust
pub struct MimirIngestionRequest {
    pub tenant_id: String,                     // "asgard_insurance"
    pub domain: String,                        // "insurance"
    pub entity_count: usize,                   // From RefGraph consolidation
    pub relationship_count: usize,             // From RefGraph consolidation
    pub average_confidence: f32,               // Mean entity confidence
    pub entities: Vec<serde_json::Value>,      // Raw entity array
    pub relationships: Vec<serde_json::Value>, // Raw relationship array
}
```

### Error Handling

All functions return `Result<T>` with rich error context:

```rust
pub enum OrchestrationError {
    PipelineError(String),
    MimirError(String),
    ValidationError(String),
    IoError(std::io::Error),
    SerializeError(serde_json::Error),
}
```

### Testing Strategy

#### RED Phase (19 tests)
- Validation decision boundaries (75%, 50% thresholds)
- Hit rate result structure
- Pipeline orchestrator initialization
- Hit rate validator structure
- Standard test query support (10 queries, 2 Thai)
- Decision gate logic (PASS/WARN/FAIL)

#### GREEN Phase (38 tests)
- MimirClient HTTP operations
- Hit rate computation
- Status tracking
- Serialization/deserialization
- Error handling with non-existent server
- Mock client for isolated testing

#### REFACTOR Phase (38 tests)
- Integration tests with mock Mimir
- Roundtrip serialization verification
- Boundary condition testing (0.7499, 0.5001)
- Thai language query support
- Complete data structure coverage
- Multiple operation state tracking

### Standard Test Queries (10 total)

**English (Tier 1-4):**
1. `Critical Illness insurance` (Lookup)
2. `Health Insurance plan` (Lookup)
3. `Accident coverage` (Reasoning)
4. `Death benefit` (Reasoning)
5. `Hospital admission` (Exclusion)
6. `Exclusions list` (Exclusion)
7. `Premium payment` (Robustness)
8. `Claim process` (Robustness)

**Thai:**
9. `ประกันสุขภาพ` (Health insurance)
10. `ประกันชีวิต` (Life insurance)

### Integration with RefGraph

The orchestration layer consumes RefGraph consolidation output:

```
refgraph-core (extract.rs + dedup.rs + graph.rs + output.rs)
            ↓
         JSON/JSONL
            ↓
refgraph-orchestration (ingest to Mimir)
            ↓
      Hit Rate Validation
            ↓
    Decision Gate (PASS/WARN/FAIL)
```

### Future Enhancements

**Phase 4 (May 22-28):**
- [ ] End-to-end pipeline tests with real Mimir instance
- [ ] Performance benchmarking (consolidation → ingestion → validation)
- [ ] Retry logic with exponential backoff
- [ ] Batch ingestion for large entity sets
- [ ] Query result caching

**Phase 5 (June 2-11):**
- [ ] Plan B activation: automatic embedding model switch
- [ ] Multi-insurer consolidation tracking
- [ ] Detailed audit logs for compliance
- [ ] Prometheus metrics export

## Development Notes

### Async/Await

All I/O operations use Tokio for non-blocking execution:

```rust
pub async fn ingest(&self, request: MimirIngestionRequest) -> Result<String>
pub async fn validate(&self) -> Result<HitRateResult>
pub async fn search(&self, query: &str, k: usize) -> Result<QueryResult>
```

### Error Propagation

Idiomatic Rust error handling with `?` operator:

```rust
pub async fn ingest(&self, request: MimirIngestionRequest) -> Result<String> {
    let response = self.client.ingest(&request).await?;  // Early return on error
    // ... process response
}
```

### Serialization

All types derive `Serialize` + `Deserialize` for JSON roundtrips:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitRateResult { ... }
```

### Mock Testing

`MockMimirClient` allows tests without a running Mimir instance:

```rust
let mock = MockMimirClient::new()
    .with_result("query1".to_string(), 5)
    .with_result("query2".to_string(), 0);

let result = mock.search("query1", 3).await?;
assert_eq!(result.hits, 5);
```

## Deployment Checklist

- [x] Orchestration module created
- [x] All 38 tests passing
- [x] Error handling complete
- [x] Mock client for testing
- [x] Thai language support
- [x] Decision gate thresholds (75%/50%)
- [ ] Integration with real Mimir instance
- [ ] Performance benchmarking
- [ ] Production documentation

## Metrics

| Metric | Value |
|--------|-------|
| Total Tests | 38 |
| Test Pass Rate | 100% |
| Module Coverage | client.rs, lib.rs, mock.rs |
| Code Quality | No clippy warnings (aside from unused test utilities) |
| Performance | <1ms validation logic (async I/O dependent on Mimir latency) |

---

**Next Step:** Integration testing with real Mimir instance (May 18-19)
