//! RefGraph Orchestration CLI
//! Complete pipeline: consolidation → ingestion → validation

use clap::{Parser, Subcommand};
use refgraph_orchestration::*;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "orchestrate")]
#[command(about = "RefGraph Insurance Document Orchestration Pipeline", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Mimir base URL
    #[arg(global = true, long, default_value = "http://localhost:5000")]
    mimir_url: String,

    /// Tenant ID (asgard_insurance, asgard_medical, etc.)
    #[arg(global = true, long, default_value = "asgard_insurance")]
    tenant_id: String,

    /// Enable verbose logging
    #[arg(global = true, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest consolidated RefGraph output to Mimir
    Ingest {
        /// JSON file with consolidation output
        #[arg(value_name = "FILE")]
        input: String,

        /// Output file for ingestion response
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Run validation queries and compute hit rate
    Validate {
        /// Optional: specific queries to run (comma-separated)
        #[arg(long)]
        queries: Option<String>,

        /// k-value for top-k results
        #[arg(long, default_value = "3")]
        k: usize,
    },

    /// End-to-end: ingest then validate
    Pipeline {
        /// JSON file with consolidation output
        #[arg(value_name = "FILE")]
        input: String,

        /// Optional queries (defaults to 10 standard insurance queries)
        #[arg(long)]
        queries: Option<String>,
    },

    /// Sample data operations
    Sample {
        #[command(subcommand)]
        sample_cmd: SampleCommands,
    },

    /// Health check and diagnostic info
    Health,
}

#[derive(Subcommand, Clone)]
enum SampleCommands {
    /// Show sample critical illness document
    CriticalIllness,

    /// Show sample health insurance document
    HealthInsurance,

    /// Show sample Thai insurance document
    Thai,

    /// Generate all sample consolidations
    AllConsolidations,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        println!("🔍 RefGraph Orchestration Pipeline");
        println!("   Mimir URL: {}", cli.mimir_url);
        println!("   Tenant: {}", cli.tenant_id);
    }

    match &cli.command {
        Commands::Ingest { input, output } => {
            cmd_ingest(&cli, input, output.clone()).await;
        }
        Commands::Validate { queries, k } => {
            cmd_validate(&cli, queries.clone(), *k).await;
        }
        Commands::Pipeline { input, queries } => {
            cmd_pipeline(&cli, input, queries.clone()).await;
        }
        Commands::Sample { sample_cmd } => {
            cmd_sample(sample_cmd.clone());
        }
        Commands::Health => {
            cmd_health(&cli).await;
        }
    }
}

async fn cmd_ingest(cli: &Cli, input_file: &str, output_file: Option<String>) {
    let start = Instant::now();
    println!("📥 Ingesting consolidated output from: {}", input_file);

    // Parse input file
    let content = match std::fs::read_to_string(input_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to read input: {}", e);
            return;
        }
    };

    let request: MimirIngestionRequest = match serde_json::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("❌ Invalid JSON format: {}", e);
            return;
        }
    };

    // Create orchestrator and ingest
    let orchestrator = PipelineOrchestrator::new(cli.mimir_url.clone(), cli.tenant_id.clone());

    match orchestrator.ingest(request.clone()).await {
        Ok(response) => {
            println!("✅ Ingestion successful");
            println!("   Entities: {}", request.entity_count);
            println!("   Relationships: {}", request.relationship_count);
            println!("   Avg Confidence: {:.3}", request.average_confidence);

            if let Some(output_path) = output_file {
                let _ = std::fs::write(&output_path, response);
                println!("   Response saved to: {}", output_path);
            }
        }
        Err(e) => {
            eprintln!("❌ Ingestion failed: {}", e);
        }
    }

    let elapsed = start.elapsed();
    println!("⏱️  Completed in {:.2}ms", elapsed.as_secs_f32() * 1000.0);
}

async fn cmd_validate(cli: &Cli, queries: Option<String>, k: usize) {
    let start = Instant::now();
    println!("🔍 Running validation queries (k={})", k);

    // Parse queries or use defaults
    let test_queries = if let Some(q) = queries {
        q.split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>()
    } else {
        default_test_queries()
    };

    println!("📋 {} queries to execute", test_queries.len());

    // Create validator
    let validator =
        HitRateValidator::new(cli.mimir_url.clone(), cli.tenant_id.clone(), test_queries);

    match validator.validate().await {
        Ok(result) => {
            println!("✅ Validation complete");
            println!("   Hit Rate: {:.1}%", result.hit_rate * 100.0);
            println!("   Hits: {}/{}", result.hit_count, result.total_queries);
            println!("   Decision: {:?}", result.decision);

            match result.decision {
                ValidationDecision::Pass => {
                    println!("🎉 PASS: Ready for production");
                }
                ValidationDecision::Warn => {
                    println!("⚠️  WARN: Review required before production");
                }
                ValidationDecision::Fail => {
                    println!("❌ FAIL: Activate Plan B (model switch BGE-M3 → Typhoon)");
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Validation failed: {}", e);
        }
    }

    let elapsed = start.elapsed();
    println!("⏱️  Completed in {:.2}ms", elapsed.as_secs_f32() * 1000.0);
}

async fn cmd_pipeline(cli: &Cli, input_file: &str, queries: Option<String>) {
    let start = Instant::now();
    println!("🔗 Running end-to-end pipeline");

    // Step 1: Ingest
    println!("\n[1/2] Ingestion Phase");
    let content = match std::fs::read_to_string(input_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to read input: {}", e);
            return;
        }
    };

    let request: MimirIngestionRequest = match serde_json::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("❌ Invalid JSON: {}", e);
            return;
        }
    };

    let orchestrator = PipelineOrchestrator::new(cli.mimir_url.clone(), cli.tenant_id.clone());

    match orchestrator.ingest(request.clone()).await {
        Ok(_) => {
            println!("      ✅ Ingested {} entities", request.entity_count);
        }
        Err(e) => {
            eprintln!("      ❌ Ingestion failed: {}", e);
            return;
        }
    }

    // Step 2: Validate
    println!("\n[2/2] Validation Phase");
    let test_queries = if let Some(q) = queries {
        q.split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>()
    } else {
        default_test_queries()
    };

    let validator =
        HitRateValidator::new(cli.mimir_url.clone(), cli.tenant_id.clone(), test_queries);

    match validator.validate().await {
        Ok(result) => {
            println!("      ✅ Validation complete");
            println!("      Hit Rate: {:.1}%", result.hit_rate * 100.0);
            println!("      Decision: {:?}", result.decision);
        }
        Err(e) => {
            eprintln!("      ❌ Validation failed: {}", e);
            return;
        }
    }

    let elapsed = start.elapsed();
    println!(
        "\n✨ Pipeline complete in {:.2}ms",
        elapsed.as_secs_f32() * 1000.0
    );
}

fn cmd_sample(cmd: SampleCommands) {
    match cmd {
        SampleCommands::CriticalIllness => {
            println!(
                "{}",
                reference_implementation::sample_data::critical_illness_document()
            );
        }
        SampleCommands::HealthInsurance => {
            println!(
                "{}",
                reference_implementation::sample_data::health_insurance_document()
            );
        }
        SampleCommands::Thai => {
            println!(
                "{}",
                reference_implementation::sample_data::thai_insurance_document()
            );
        }
        SampleCommands::AllConsolidations => {
            println!("=== Critical Illness Consolidation ===");
            let crit = reference_implementation::ConsolidationOutput::sample_critical_illness();
            println!("{}", serde_json::to_string_pretty(&crit).unwrap_or_default());

            println!("\n=== Health Insurance Consolidation ===");
            let health = reference_implementation::ConsolidationOutput::sample_health_insurance();
            println!("{}", serde_json::to_string_pretty(&health).unwrap_or_default());
        }
    }
}

async fn cmd_health(cli: &Cli) {
    println!("🏥 Health Check");
    let orchestrator = PipelineOrchestrator::new(cli.mimir_url.clone(), cli.tenant_id.clone());

    let status = orchestrator.status();
    println!("   Mimir URL: {}", cli.mimir_url);
    println!("   Tenant: {}", cli.tenant_id);
    println!("   Total Ingested: {}", status.total_ingested);
    println!("   Total Failed: {}", status.total_failed);

    if let Some(ts) = status.last_ingestion_timestamp {
        println!("   Last Ingestion: {}", ts);
    }
}

fn default_test_queries() -> Vec<String> {
    vec![
        "Critical Illness insurance".to_string(),
        "Health Insurance plan".to_string(),
        "Accident coverage".to_string(),
        "Death benefit".to_string(),
        "Hospital admission".to_string(),
        "Exclusions list".to_string(),
        "Premium payment".to_string(),
        "Claim process".to_string(),
        "ประกันสุขภาพ".to_string(),
        "ประกันชีวิต".to_string(),
    ]
}
