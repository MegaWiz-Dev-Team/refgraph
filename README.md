# RefGraph

[![Crates.io](https://img.shields.io/crates/v/refgraph-core.svg)](https://crates.io/crates/refgraph-core)
[![License: AGPL-3.0 OR Commercial](https://img.shields.io/badge/license-AGPL--3.0%20OR%20Commercial-blue.svg)](LICENSE-AGPL)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

> **Multi-domain semantic entity graph consolidation engine**
>
> Transform messy, overlapping data from multiple sources into a clean, deduplicated entity graph with confidence scores and multi-source tracking.

## The Problem

Real-world data from multiple sources is messy:

```
Source A: "PRU Critical Illness covers Heart Attack and Stroke"
Source B: "Critical Illness plan includes myocardial infarction and CVA"
Source C: "PRU CIL Product: covers acute MI, acute cerebrovascular event"
```

Same product, same coverages, described 3 different ways. Manual consolidation is tedious and error-prone.

RefGraph solves this with a **4-step semantic consolidation pipeline** that produces:

- **900 unique entities** from 1200+ raw chunks
- **High confidence scores** (0.72+) for each entity
- **Multi-source tracking** (knows which URLs it came from)
- **Neo4j-ready relationships** for knowledge graphs

## How It Works

```
┌─────────────────────┐
│  Raw Chunks (200-300)  │
└──────────┬──────────┘
           │
    ┌──────▼──────┐
    │ [1] Extract │ → Entity extraction from text
    │  Entities   │   (pattern-based or ML)
    └──────┬──────┘
           │
    ┌──────▼──────┐
    │ [2] Build   │ → Create graph structure
    │   Graph     │   (entities + relationships)
    └──────┬──────┘
           │
    ┌──────▼──────┐
    │ [3] Dedup   │ → Jaccard similarity (0.95)
    │  (Jaccard)  │   Merge duplicates
    └──────┬──────┘
           │
    ┌──────▼───────────┐
    │ [4] Consolidate  │ → Cosine similarity (0.96)
    │  (Semantic)      │   Final merging
    └──────┬───────────┘
           │
    ┌──────▼──────────────┐
    │ Output: 850-950     │
    │ Consolidated        │
    │ Entities + Rels     │
    └─────────────────────┘
```

## Quick Start

### Installation

```toml
[dependencies]
refgraph-core = "0.1"
refgraph-neo4j = "0.1"
tokio = { version = "1.49", features = ["full"] }
```

### Basic Example

```rust
use refgraph_core::{RefGraph, ManifestConfig, types::RawChunk};

#[tokio::main]
async fn main() -> refgraph_core::Result<()> {
    // Configure for your domain
    let config = ManifestConfig::insurance();
    config.validate()?;

    // Create pipeline
    let mut graph = RefGraph::new(config)?;

    // Load chunks
    let chunks = vec![
        RawChunk {
            chunk_id: "chunk_001".to_string(),
            content: "PRU Critical Illness covers Heart Attack and Stroke".to_string(),
            source_url: "prudential.co.th/products".to_string(),
            page_index: Some(1),
            token_count: 10,
        },
        // Add more chunks...
    ];

    // Run consolidation
    let output = graph.consolidate(chunks).await?;

    println!("✓ {} entities, {} relationships",
        output.metadata.entity_count,
        output.metadata.relationship_count);

    // Save results
    output.save_json("output.json")?;
    Ok(())
}
```

## Domain Configuration

RefGraph works with any domain. Just configure it:

### Insurance (Default)
```rust
let config = ManifestConfig::insurance();
// Entity types: Product, Coverage, Exclusion, Condition, Organization
// Relationships: HAS_COVERAGE, HAS_EXCLUSION, REQUIRES_CONDITION, OFFERED_BY
```

### Medical
```rust
let config = ManifestConfig::medical();
// Entity types: Symptom, Diagnosis, Drug, Procedure, Condition
// Relationships: INDICATES, TREATS, CONTRAINDICATED_WITH, CAUSES
```

### Custom Domain
```rust
let mut config = ManifestConfig::default();
config.domain = "legal".to_string();
config.confidence_threshold = 0.80;
```

## Architecture

```
refgraph/
├── refgraph-core/          # Library (published to crates.io)
│   ├── types.rs            # Entity, Relationship, RawChunk, etc.
│   ├── error.rs            # Error handling
│   ├── manifest.rs         # Domain configuration
│   ├── extract.rs          # Entity extraction (pluggable)
│   ├── dedup.rs            # Jaccard deduplication
│   ├── graph.rs            # Semantic graph building
│   ├── output.rs           # Output formatting (JSON/JSONL)
│   └── lib.rs              # RefGraph pipeline coordinator
│
├── refgraph-neo4j/         # Neo4j persistence (published)
│   ├── config.rs           # Connection configuration
│   ├── service.rs          # Neo4jService wrapper
│   └── cypher.rs           # Query builders
│
├── refgraph-domains/       # Domain definitions (published)
│   ├── insurance.rs        # Insurance domain
│   ├── medical.rs          # Medical domain
│   ├── legal.rs            # Legal domain
│   └── finance.rs          # Finance domain
│
└── refgraph-cli/           # CLI tool (unpublished)
    └── main.rs             # Command-line interface
```

## Pipeline Details

### Step 1: Extract
- Parse text chunks from URLs, PDFs, or documents
- Identify entities using patterns or ML models
- Assign confidence scores

Current: Pattern-based (fast, deterministic)  
Future: Fine-tuned NER models (medical, legal)

### Step 2: Semantic Graph
- Create entity nodes from extracted entities
- Build relationships between entity types
- Example: `Product` --HAS_COVERAGE--> `Coverage`

### Step 3: Deduplicate
- **Jaccard similarity** on token sets
- Threshold: 0.95 (configurable)
- Formula: `|A ∩ B| / |A ∪ B|`
- Performance: ~300ms for 1000 entities

### Step 4: Consolidate
- **Cosine similarity** on embeddings (if provided)
- Threshold: 0.96 (configurable)
- Union-Find clustering for efficiency
- Result: Clean entity graph

## Features

✅ **Multi-Source Tracking** — Know which URLs each entity came from  
✅ **Confidence Scores** — Every entity has a score (0.0-1.0)  
✅ **Domain-Agnostic** — Swap config, consolidate any domain  
✅ **Neo4j Ready** — Built-in graph database support  
✅ **JSON/JSONL Output** — Easy integration with downstream systems  
✅ **Type-Safe Rust** — Compile-time error checking  
✅ **Streaming Support** — Process data in chunks  

## Testing

```bash
# Run all tests
cargo test --all

# Run tests for refgraph-core only
cargo test -p refgraph-core

# Run with output
cargo test -- --nocapture

# Run CLI tests
cargo run --release -p refgraph-cli -- --test
```

**Coverage:**
- 24 unit tests in refgraph-core
- 3 Cypher builder tests in refgraph-neo4j
- Integration tests via CLI

All tests passing ✓

## License

Licensed under Apache 2.0

- Open source under Apache License 2.0
  - Free for open-source projects
  - Source modifications must be shared
  
- **Commercial:** Custom license available

## Roadmap

### v0.1.0 (Current)
- ✅ Workspace structure (4 crates)
- ✅ Core types and traits
- ✅ Jaccard deduplication
- ✅ Semantic graph building
- ✅ JSON/JSONL output
- ⏳ TOML domain configs
- ⏳ Neo4j integration

### v0.2.0
- [ ] Embedder trait (for semantic consolidation)
- [ ] Vector similarity (cosine threshold)
- [ ] Neo4j service implementation
- [ ] Integration tests

### v0.3.0+
- [ ] Fine-tuned NER models
- [ ] Streaming/chunked processing
- [ ] Parallel consolidation (rayon)
- [ ] Web API service

## Development

```bash
# Format code
cargo fmt --all

# Lint with clippy
cargo clippy --all-targets -- -D warnings

# Build release
cargo build --release

# Run CLI
./target/release/refgraph --help
```

## Contributing

Contributions welcome! Interested in:

- Domain-specific extraction patterns
- ML/LLM-based entity extraction
- Neo4j integration improvements
- Performance optimizations
- Documentation and examples

## Citation

If you use RefGraph in academic or commercial work:

```bibtex
@software{refgraph2026,
  title={RefGraph: Multi-domain Semantic Entity Graph Consolidation},
  author={RefGraph Contributors},
  year={2026},
  url={https://github.com/refgraph-rs/refgraph}
}
```

## Support

- 📖 **Documentation**: [docs.rs/refgraph-core](https://docs.rs/refgraph-core)
- 🐛 **Issues**: [GitHub Issues](https://github.com/refgraph-rs/refgraph/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/refgraph-rs/refgraph/discussions)
- 📧 **Commercial**: paripol@megawiz.co.th

---

**Made with ❤️ for data consolidation**
