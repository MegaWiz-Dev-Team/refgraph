# RefGraph: Consolidating Messy Multi-Source Data Into a Clean System

**Written:** May 16, 2026  
**Author:** RefGraph Team  
**Contact:** paripol@megawiz.co.th  

---

## ✨ The Problem RefGraph Solves

Imagine you need to consolidate Critical Illness (CI) insurance data from 3 different providers:

```
Prudential:  "PRU CI covers Heart Attack and Stroke"
AXA:         "AXA SIP includes coverage for myocardial infarction and CVA"
Thai Health: "Critical Care covers acute MI, cerebrovascular event"
```

**What's the problem?**
- 🔄 Same data described differently (Heart Attack vs myocardial infarction vs acute MI)
- 📊 You have 200-300 overlapping chunks from multiple sources
- ⏰ Manual consolidation = wasted time + errors
- 💾 You lose track of where data came from

---

## 🎯 How RefGraph Helps

RefGraph does this automatically:

1. **Extract entities** from text
   - Heart Attack = coverage (intent)
   - Pre-existing condition = exclusion (intent)

2. **Build a semantic graph** of relationships
   ```
   Critical Illness --[HAS_COVERAGE]--> Heart Attack
   Critical Illness --[HAS_EXCLUSION]--> Pre-existing
   ```

3. **Deduplicate** redundant data
   - "Heart Attack" = "myocardial infarction" = "acute MI" → 1 entity

4. **Consolidate** with confidence scores
   - confidence = 0.88 (how certain we are)
   - source = ["prudential.co.th", "axa.co.th"] (where it came from)

**Result:** 1200+ messy chunks → 900 clean, usable entities

---

## 🚀 Getting Started with RefGraph

### Step 1: Install

```toml
[dependencies]
refgraph-core = "0.1"
tokio = { version = "1.49", features = ["full"] }
```

### Step 2: Code Example

```rust
use refgraph_core::{RefGraph, ManifestConfig, types::RawChunk};

#[tokio::main]
async fn main() -> refgraph_core::Result<()> {
    // Choose domain (type: insurance, medical, legal, finance)
    let config = ManifestConfig::insurance();
    
    // Create pipeline
    let mut graph = RefGraph::new(config)?;
    
    // Load raw data from web sources
    let chunks = vec![
        RawChunk {
            chunk_id: "pru_001".to_string(),
            content: "PRU CI covers Heart Attack and Stroke".to_string(),
            source_url: "prudential.co.th/products".to_string(),
            page_index: Some(1),
            token_count: 10,
        },
        // Add more data...
    ];
    
    // Run consolidation
    let output = graph.consolidate(chunks).await?;
    
    // View results
    println!("✓ {} entities", output.metadata.entity_count);
    println!("✓ {} relationships", output.metadata.relationship_count);
    
    // Save as JSON
    output.save_json("output.json")?;
    
    Ok(())
}
```

### Step 3: Configure Your Domain

RefGraph supports 4 domains:

**1. Insurance (Default)**
```toml
[domain]
name = "insurance"

[entities.product]
patterns = ["Critical Illness", "Health Insurance", "Life Insurance"]

[entities.coverage]
patterns = ["Heart Attack", "Stroke", "Cancer"]

[entities.exclusion]
patterns = ["Pre-existing Condition", "War"]
```

**2. Medical**
```toml
[domain]
name = "medical"

[entities.symptom]
patterns = ["fever", "cough", "dyspnea"]

[entities.drug]
patterns = ["metformin", "aspirin", "atorvastatin"]
```

**3. Legal**
```toml
[domain]
name = "legal"

[entities.clause]
patterns = ["clause", "provision", "section"]

[entities.obligation]
patterns = ["shall", "must", "required"]
```

**4. Finance**
```toml
[domain]
name = "finance"

[entities.fee]
patterns = ["fee", "charge", "commission"]

[entities.rate]
patterns = ["interest rate", "APR", "APY"]
```

---

## 🔄 How The Pipeline Works

```
┌──────────────────┐
│  200-300 Chunks  │  Raw data from web
└─────────┬────────┘
          │
    ┌─────▼──────┐
    │ [1] EXTRACT │  Extract entities from text
    └─────┬──────┘
          │
    ┌─────▼──────┐
    │ [2] GRAPH  │  Build relationship graph
    └─────┬──────┘
          │
    ┌─────▼──────┐
    │ [3] DEDUP  │  Remove duplicates (Jaccard 0.95)
    └─────┬──────┘
          │
    ┌─────▼───────────┐
    │ [4] CONSOLIDATE │  Semantic merge (cosine 0.96)
    └─────┬───────────┘
          │
    ┌─────▼────────────────┐
    │ 850-950 Clean Entities│  Clean results + relationships
    └──────────────────────┘
```

---

## 💡 Real Example: Insurance

**Input:** 5 chunks from 3 providers

```json
[
  {"id": "pru_1", "text": "PRU Critical Illness covers Heart Attack and Stroke", "source": "prudential.co.th"},
  {"id": "pru_2", "text": "Pre-existing conditions excluded", "source": "prudential.co.th"},
  {"id": "axa_1", "text": "AXA SIP includes coverage for myocardial infarction and CVA", "source": "axa.co.th"},
  {"id": "axa_2", "text": "AXA excludes pre-existing medical conditions", "source": "axa.co.th"},
  {"id": "th_1", "text": "Thai Health Critical Care covers acute MI and cerebrovascular event", "source": "thaihealth.co.th"}
]
```

**Output:** Consolidated entities

```json
{
  "entities": [
    {
      "text": "Critical Illness",
      "type": "product",
      "confidence": 0.92,
      "sources": ["prudential.co.th", "axa.co.th"],
      "merged_from": ["prod_critical_illness"]
    },
    {
      "text": "Heart Attack",
      "type": "coverage",
      "confidence": 0.88,
      "sources": ["prudential.co.th", "axa.co.th", "thaihealth.co.th"],
      "merged_from": ["cov_heart_attack", "cov_myocardial_infarction", "cov_acute_mi"]
    },
    {
      "text": "Pre-existing Condition",
      "type": "exclusion",
      "confidence": 0.85,
      "sources": ["prudential.co.th", "axa.co.th"]
    }
  ],
  "relationships": [
    {
      "source": "prod_critical_illness",
      "target": "cov_heart_attack",
      "type": "HAS_COVERAGE",
      "confidence": 0.90
    }
  ]
}
```

**Benefits:**
- ✅ Heart Attack, MI, acute MI → merged into 1 entity
- ✅ Know that Critical Illness → HAS_COVERAGE → Heart Attack
- ✅ Track all sources (transparency)
- ✅ Confidence scores (certainty metric)

---

## 🎁 RefGraph Is Perfect For

### 1. **Insurance Companies**
- Compare competitor products
- Build product comparison tools
- Monitor coverage changes

### 2. **Research & Academia**
- Consolidate data from scientific papers
- Build concept graphs
- Find semantic relationships

### 3. **Legal Tech**
- Extract contract clauses
- Compare legal terms
- Detect contradictions

### 4. **FinTech**
- Consolidate bank fee structures
- Compare interest rates
- Build product comparison engines

---

## 📊 Performance Metrics

```
100 chunks:
  ✓ Extract: 50ms
  ✓ Dedup: 30ms
  ✓ Graph: 20ms
  = ~100ms total

1000 chunks:
  ✓ Extract: 500ms
  ✓ Dedup: 300ms
  ✓ Graph: 200ms
  = ~1 second total
```

---

## 🔐 License

RefGraph uses **Apache 2.0** - completely free!

```
✅ Commercial use
✅ Modify code
✅ Use in SaaS
✅ Redistribute
```

No need to open-source your own code.

---

## 🚀 Start Using RefGraph Now

```bash
# 1. Clone
git clone https://github.com/refgraph-rs/refgraph.git

# 2. Run example
cargo run --example insurance_pipeline

# 3. Add to project
cargo add refgraph-core
```

---

## 📧 Get In Touch

- **GitHub:** https://github.com/refgraph-rs/refgraph
- **Crates.io:** https://crates.io/crates/refgraph-core
- **Email:** paripol@megawiz.co.th

---

**RefGraph:** Turn messy data into clean, usable knowledge 🎯
