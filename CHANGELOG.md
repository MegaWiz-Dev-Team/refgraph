# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-16

### Added

- **Workspace Structure**: Split into 4 crates:
  - `refgraph-core`: Main consolidation engine (published)
  - `refgraph-neo4j`: Neo4j persistence layer (published)
  - `refgraph-domains`: Domain type definitions (published)
  - `refgraph-cli`: Command-line tool (unpublished)

- **Core Features**:
  - `RefGraph` pipeline coordinator with 4-step consolidation
  - Entity extraction with pattern matching
  - Jaccard similarity deduplication (0.95 threshold)
  - Semantic graph building with Neo4j relationship inference
  - JSON/JSONL output formatting

- **Types**:
  - `RawChunk`: Input data from web scraping
  - `Entity`: Extracted entities with confidence scores
  - `EntityType`: Domain-agnostic enum (insurance, medical, legal, finance)
  - `ConsolidatedEntity`: Deduplicated output with source tracking
  - `RefGraphOutput`: Final formatted output (JSON/JSONL)

- **Error Handling**: Comprehensive error types via `thiserror`

- **Domain Support**:
  - Insurance: Products, Coverage, Exclusions, Conditions, Organizations
  - Medical: Symptoms, Diagnoses, Drugs, Procedures, Conditions
  - Legal: Clauses, Parties, Obligations, Rights, Jurisdictions
  - Finance: Account Types, Fees, Rates, Products, Regulations

- **TOML Configuration**: Domain config files (insurance.toml, medical.toml, legal.toml, finance.toml)

- **Testing**: 24 unit tests in refgraph-core, 3 Cypher tests in refgraph-neo4j

- **CI/CD**: GitHub Actions workflows for testing and publishing to crates.io

- **Documentation**: Comprehensive README with quickstart, architecture, and roadmap

### Changed

- **Naming**: Decoupled from Asgard/Mimir internal naming
  - `MimirOutput` → `RefGraphOutput`
  - `mimir.rs` → `output.rs`
  - Author: "Asgard Team" → "RefGraph Contributors"

- **Logging**: Replaced `log`/`env_logger` with `tracing` for better observability

- **Types**: Made `EntityType` non-Copy to support `Custom(String)` variant

### Fixed

- Resolved Copy trait issue in `dedup.rs` by cloning `entity_type`

### Dependencies

- **Async**: tokio 1.49 (full features)
- **Serialization**: serde 1.0 + serde_json 1.0 + toml 1.1
- **Database**: neo4rs 0.8
- **Error Handling**: thiserror 2.0 + anyhow 1.0
- **Logging**: tracing 0.1 + tracing-subscriber 0.3
- **CLI**: clap 4.6 with derive macros
- **Algorithms**: itertools 0.14, hashbrown 0.15, rayon 1.10

### Removed

- Unused dependencies from library core (axum, hyper, reqwest removed from refgraph-core)

## [0.2.0] - Unreleased

### Planned

- [ ] Implement actual Neo4j service (connection, upserts)
- [ ] Add Embedder trait for semantic similarity
- [ ] Cosine similarity consolidation implementation
- [ ] Integration tests with Neo4j testcontainers
- [ ] TOML-based extraction patterns
- [ ] Fine-tuned NER models (medical, legal domains)

## [0.3.0] - Future

### Planned

- [ ] LLM-based entity extraction
- [ ] Streaming/chunked processing
- [ ] Parallel consolidation with rayon
- [ ] Web API service
- [ ] Performance benchmarking suite

---

**Made with ❤️ for data consolidation**
