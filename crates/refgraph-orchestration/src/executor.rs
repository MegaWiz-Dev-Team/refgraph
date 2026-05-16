//! Pipeline Executor - Orchestrates complete S1 execution workflow
//! Manages: consolidation → ingestion → validation → decision

use crate::*;
use insurance_data::{InsuranceCompany, generate_consolidation_for_company};
use mock_server::MockMimirServer;
use serde::{Deserialize, Serialize};

/// Complete pipeline execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub pipeline_id: String,
    pub start_time: String,
    pub end_time: String,
    pub duration_ms: f32,
    pub consolidation_stats: ConsolidationStats,
    pub ingestion_status: IngestionStatus,
    pub validation_result: Option<HitRateResult>,
    pub final_decision: ValidationDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationStats {
    pub total_entities: usize,
    pub total_relationships: usize,
    pub average_confidence: f32,
    pub documents_processed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionStatus {
    pub success: bool,
    pub entities_ingested: usize,
    pub relationships_ingested: usize,
    pub timestamp: String,
}

/// Pipeline executor for S1 execution
pub struct PipelineExecutor {
    mock_server: MockMimirServer,
    verbose: bool,
}

impl PipelineExecutor {
    pub fn new(verbose: bool) -> Self {
        Self {
            mock_server: MockMimirServer::new(),
            verbose,
        }
    }

    /// Execute complete pipeline for a single insurance company
    pub async fn execute_company_pipeline(
        &self,
        company: InsuranceCompany,
    ) -> ExecutionResult {
        let start_time = chrono::Utc::now();
        let pipeline_id = format!("EXEC_{}_{}", company.as_str(), start_time.timestamp());

        if self.verbose {
            println!("🚀 Starting pipeline: {}", pipeline_id);
            println!("   Company: {}", company.as_str());
        }

        // Step 1: Consolidation (simulated - RefGraph would do this)
        let consolidation = generate_consolidation_for_company(company);

        let consolidation_stats = ConsolidationStats {
            total_entities: consolidation.entities.len(),
            total_relationships: consolidation.relationships.len(),
            average_confidence: consolidation.average_confidence,
            documents_processed: 3, // Each company has 3 sample documents
        };

        if self.verbose {
            println!("✅ Consolidation complete");
            println!("   Entities: {}", consolidation_stats.total_entities);
            println!("   Relationships: {}", consolidation_stats.total_relationships);
            println!("   Avg Confidence: {:.3}", consolidation_stats.average_confidence);
        }

        // Step 2: Ingestion (simulated - would call real Mimir)
        self.mock_server.record_ingest(
            "asgard_insurance".to_string(),
            "insurance".to_string(),
            consolidation_stats.total_entities,
            consolidation_stats.total_relationships,
            consolidation_stats.average_confidence,
        );

        let ingestion_status = IngestionStatus {
            success: true,
            entities_ingested: consolidation_stats.total_entities,
            relationships_ingested: consolidation_stats.total_relationships,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        if self.verbose {
            println!("✅ Ingestion complete");
            println!("   Total ingested: {}", self.mock_server.total_ingested());
        }

        // Step 3: Validation (compute hit rate)
        let test_queries = default_test_queries();
        let hit_rate = self.mock_server.compute_hit_rate(&test_queries);

        let validation_result = HitRateResult {
            total_queries: test_queries.len(),
            hit_count: (hit_rate * test_queries.len() as f32) as usize,
            hit_rate,
            decision: ValidationDecision::from_hit_rate(hit_rate),
            query_results: test_queries
                .iter()
                .filter_map(|q| {
                    let results = self.mock_server.search(q, 3);
                    if !results.is_empty() {
                        Some(QueryResult {
                            query: q.clone(),
                            hits: results.len(),
                            score: results[0].confidence,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        })
                    } else {
                        None
                    }
                })
                .collect(),
        };

        if self.verbose {
            println!("✅ Validation complete");
            println!("   Hit Rate: {:.1}%", validation_result.hit_rate * 100.0);
            println!("   Decision: {:?}", validation_result.decision);
        }

        let end_time = chrono::Utc::now();
        let duration_ms =
            (end_time.timestamp_millis() - start_time.timestamp_millis()) as f32;
        let duration_ms = duration_ms.max(0.1); // Ensure always > 0

        ExecutionResult {
            pipeline_id,
            start_time: start_time.to_rfc3339(),
            end_time: end_time.to_rfc3339(),
            duration_ms,
            consolidation_stats,
            ingestion_status,
            validation_result: Some(validation_result.clone()),
            final_decision: validation_result.decision,
        }
    }

    /// Execute pipeline for all three insurance companies
    pub async fn execute_all_companies(&self) -> Vec<ExecutionResult> {
        let companies = vec![
            InsuranceCompany::Prudential,
            InsuranceCompany::AXA,
            InsuranceCompany::ThaiHealth,
        ];

        let mut results = Vec::new();
        for company in companies {
            let result = self.execute_company_pipeline(company).await;
            results.push(result);
        }

        results
    }

    /// Get aggregated statistics across all executions
    pub fn get_aggregated_stats(&self) -> AggregatedStats {
        AggregatedStats {
            total_documents_ingested: self.mock_server.total_ingested(),
            total_entities: self.mock_server.total_entities(),
            total_relationships: self.mock_server.total_relationships(),
            average_hit_rate: self.compute_average_hit_rate(),
        }
    }

    fn compute_average_hit_rate(&self) -> f32 {
        let test_queries = default_test_queries();
        self.mock_server.compute_hit_rate(&test_queries)
    }

    /// Generate execution report
    pub fn generate_report(&self, results: &[ExecutionResult]) -> ExecutionReport {
        let pass_count = results
            .iter()
            .filter(|r| r.final_decision == ValidationDecision::Pass)
            .count();

        let warn_count = results
            .iter()
            .filter(|r| r.final_decision == ValidationDecision::Warn)
            .count();

        let fail_count = results
            .iter()
            .filter(|r| r.final_decision == ValidationDecision::Fail)
            .count();

        let avg_duration = if results.is_empty() {
            0.0
        } else {
            results.iter().map(|r| r.duration_ms).sum::<f32>() / results.len() as f32
        };

        let total_entities: usize = results
            .iter()
            .map(|r| r.consolidation_stats.total_entities)
            .sum();

        ExecutionReport {
            total_executions: results.len(),
            pass_count,
            warn_count,
            fail_count,
            total_entities,
            average_duration_ms: avg_duration,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    pub total_documents_ingested: usize,
    pub total_entities: usize,
    pub total_relationships: usize,
    pub average_hit_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub total_executions: usize,
    pub pass_count: usize,
    pub warn_count: usize,
    pub fail_count: usize,
    pub total_entities: usize,
    pub average_duration_ms: f32,
    pub timestamp: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_new() {
        let executor = PipelineExecutor::new(false);
        let stats = executor.get_aggregated_stats();
        assert_eq!(stats.total_documents_ingested, 0);
    }

    #[tokio::test]
    async fn test_executor_prudential_pipeline() {
        let executor = PipelineExecutor::new(false);
        let result = executor
            .execute_company_pipeline(InsuranceCompany::Prudential)
            .await;

        assert_eq!(result.consolidation_stats.total_entities, 6);
        assert_eq!(result.consolidation_stats.total_relationships, 4);
        assert!(result.ingestion_status.success);
    }

    #[tokio::test]
    async fn test_executor_axa_pipeline() {
        let executor = PipelineExecutor::new(false);
        let result = executor
            .execute_company_pipeline(InsuranceCompany::AXA)
            .await;

        assert_eq!(result.consolidation_stats.total_entities, 5);
        assert!(result.ingestion_status.success);
    }

    #[tokio::test]
    async fn test_executor_thai_health_pipeline() {
        let executor = PipelineExecutor::new(false);
        let result = executor
            .execute_company_pipeline(InsuranceCompany::ThaiHealth)
            .await;

        assert_eq!(result.consolidation_stats.total_entities, 5);
        assert!(result.ingestion_status.success);
    }

    #[tokio::test]
    async fn test_executor_all_companies() {
        let executor = PipelineExecutor::new(false);
        let results = executor.execute_all_companies().await;

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.ingestion_status.success));
    }

    #[tokio::test]
    async fn test_executor_aggregated_stats() {
        let executor = PipelineExecutor::new(false);
        let _results = executor.execute_all_companies().await;

        let stats = executor.get_aggregated_stats();
        assert_eq!(stats.total_documents_ingested, 3);
        assert!(stats.total_entities > 0);
        assert!(stats.total_relationships > 0);
    }

    #[tokio::test]
    async fn test_executor_generates_report() {
        let executor = PipelineExecutor::new(false);
        let results = executor.execute_all_companies().await;
        let report = executor.generate_report(&results);

        assert_eq!(report.total_executions, 3);
        assert!(report.pass_count > 0 || report.warn_count > 0 || report.fail_count > 0);
        assert_eq!(report.pass_count + report.warn_count + report.fail_count, 3);
    }

    #[tokio::test]
    async fn test_executor_result_has_timing() {
        let executor = PipelineExecutor::new(false);
        let result = executor
            .execute_company_pipeline(InsuranceCompany::Prudential)
            .await;

        assert!(result.duration_ms > 0.0);
        assert!(!result.start_time.is_empty());
        assert!(!result.end_time.is_empty());
    }

    #[tokio::test]
    async fn test_executor_result_has_validation() {
        let executor = PipelineExecutor::new(false);
        let result = executor
            .execute_company_pipeline(InsuranceCompany::Prudential)
            .await;

        assert!(result.validation_result.is_some());
        let validation = result.validation_result.unwrap();
        assert!(validation.total_queries > 0);
        assert!(validation.hit_rate >= 0.0 && validation.hit_rate <= 1.0);
    }

    #[test]
    fn test_execution_report_totals_match() {
        let result1 = ExecutionResult {
            pipeline_id: "test1".to_string(),
            start_time: "2026-05-16T10:00:00Z".to_string(),
            end_time: "2026-05-16T10:00:01Z".to_string(),
            duration_ms: 100.0,
            consolidation_stats: ConsolidationStats {
                total_entities: 6,
                total_relationships: 4,
                average_confidence: 0.94,
                documents_processed: 3,
            },
            ingestion_status: IngestionStatus {
                success: true,
                entities_ingested: 6,
                relationships_ingested: 4,
                timestamp: "2026-05-16T10:00:00Z".to_string(),
            },
            validation_result: None,
            final_decision: ValidationDecision::Pass,
        };

        let executor = PipelineExecutor::new(false);
        let report = executor.generate_report(&[result1]);

        assert_eq!(report.total_executions, 1);
        assert_eq!(report.pass_count, 1);
        assert_eq!(report.total_entities, 6);
    }
}
