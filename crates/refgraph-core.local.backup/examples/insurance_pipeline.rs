//! Example: Insurance data consolidation pipeline
//!
//! This example demonstrates consolidating insurance product data
//! from multiple sources into a clean entity graph.
//!
//! Run with: cargo run -p refgraph-core --example insurance_pipeline

use refgraph_core::{types::RawChunk, ManifestConfig, RefGraph};

#[tokio::main]
async fn main() -> refgraph_core::Result<()> {
    println!("🏢 RefGraph: Insurance Data Consolidation Example\n");

    // Step 1: Configure domain
    println!("📋 Step 1: Configure insurance domain");
    let config = ManifestConfig::insurance();
    config.validate()?;
    println!("  ✓ Domain: {}", config.domain);
    println!("  ✓ Confidence threshold: {}", config.confidence_threshold);
    println!("  ✓ Dedup threshold: {}\n", config.dedup_threshold);

    // Step 2: Create pipeline
    println!("🔧 Step 2: Initialize RefGraph pipeline");
    let mut graph = RefGraph::new(config)?;
    println!("  ✓ Pipeline created\n");

    // Step 3: Prepare raw data chunks
    println!("📥 Step 3: Load raw chunks from sources");
    let chunks = vec![
        // Prudential: Product + Coverage + Exclusion
        RawChunk {
            chunk_id: "pru_001".to_string(),
            content: "PRU Critical Illness covers Heart Attack and Stroke with a waiting period of 30 days"
                .to_string(),
            source_url: "prudential.co.th/products/cil".to_string(),
            page_index: Some(1),
            token_count: 17,
        },
        RawChunk {
            chunk_id: "pru_002".to_string(),
            content: "Pre-existing conditions are excluded from PRU Critical Illness coverage".to_string(),
            source_url: "prudential.co.th/terms".to_string(),
            page_index: Some(2),
            token_count: 12,
        },

        // AXA: Similar product, different wording
        RawChunk {
            chunk_id: "axa_001".to_string(),
            content: "AXA Serious Illness Plan includes coverage for Heart Attack, Stroke, and Cancer"
                .to_string(),
            source_url: "axa.co.th/products".to_string(),
            page_index: Some(1),
            token_count: 14,
        },
        RawChunk {
            chunk_id: "axa_002".to_string(),
            content: "AXA excludes pre-existing medical conditions from SIP".to_string(),
            source_url: "axa.co.th/exclusions".to_string(),
            page_index: Some(3),
            token_count: 11,
        },

        // Thai Health: Same coverages, different phrasing
        RawChunk {
            chunk_id: "th_001".to_string(),
            content: "Thai Health Critical Care covers myocardial infarction and cerebrovascular event"
                .to_string(),
            source_url: "thaihealth.co.th/plans".to_string(),
            page_index: Some(2),
            token_count: 13,
        },
    ];
    println!("  ✓ Loaded {} chunks from 3 sources\n", chunks.len());

    // Step 4: Run consolidation
    println!("⚙️  Step 4: Run consolidation pipeline");
    println!("    [1] Extracting entities...");
    println!("    [2] Building semantic graph...");
    println!("    [3] Deduplicating (Jaccard similarity)...");
    println!("    [4] Consolidating (semantic merging)...\n");

    let output = graph.consolidate(chunks).await?;

    // Step 5: Display results
    println!("✅ Consolidation complete!\n");
    println!("📊 Results:");
    println!("  Entity count: {}", output.metadata.entity_count);
    println!(
        "  Relationship count: {}",
        output.metadata.relationship_count
    );
    println!(
        "  Average confidence: {:.2}",
        output.metadata.average_confidence
    );
    println!("  Domain: {}", output.metadata.domain);
    println!("  Timestamp: {}\n", output.metadata.timestamp);

    // Step 6: Display entities
    println!("🏷️  Consolidated Entities:");
    for (idx, entity) in output.entities.iter().enumerate() {
        println!("\n  {}. {}", idx + 1, entity.text);
        println!("     Type: {}", entity.entity_type);
        println!("     Confidence: {:.2}", entity.confidence);
        println!("     Sources: {}", entity.sources.join(", "));
        if !entity.merged_from.is_empty() {
            println!("     Merged from: {}", entity.merged_from.join(", "));
        }
    }

    // Step 7: Display relationships
    println!("\n\n🔗 Relationships:");
    for (idx, rel) in output.relationships.iter().enumerate() {
        println!(
            "\n  {}. {} --[{}]--> {}",
            idx + 1,
            rel.source_id,
            rel.relationship_type,
            rel.target_id
        );
        println!("     Confidence: {:.2}", rel.confidence);
    }

    // Step 8: Export results
    println!("\n\n💾 Export Options:");
    println!("  - JSON: output.save_json(\"output.json\")?");
    println!("  - JSONL: output.save_jsonl(\"output.jsonl\")?");

    println!("\n✨ Example complete!\n");

    Ok(())
}
