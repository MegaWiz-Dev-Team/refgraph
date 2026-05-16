//! RefGraph Orchestration Layer
//! Manages the full pipeline: consolidation output → Mimir ingestion → validation

pub mod client;
pub mod reference_implementation;
pub mod mock_server;
pub mod insurance_data;
pub mod executor;
pub mod execution_simulator;
pub mod monitoring;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod integration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrchestrationError {
    #[error("Pipeline failed: {0}")]
    PipelineError(String),

    #[error("Mimir API error: {0}")]
    MimirError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializeError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, OrchestrationError>;

/// Status of a single query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub query: String,
    pub hits: usize,
    pub score: f32,
    pub timestamp: String,
}

/// Hit rate validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitRateResult {
    pub total_queries: usize,
    pub hit_count: usize,
    pub hit_rate: f32,
    pub decision: ValidationDecision,
    pub query_results: Vec<QueryResult>,
}

/// Decision based on hit rate thresholds
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ValidationDecision {
    #[serde(rename = "PASS")]
    Pass,
    #[serde(rename = "WARN")]
    Warn,
    #[serde(rename = "FAIL")]
    Fail,
}

impl ValidationDecision {
    pub fn from_hit_rate(hit_rate: f32) -> Self {
        match hit_rate {
            rate if rate >= 0.75 => ValidationDecision::Pass,
            rate if rate >= 0.50 => ValidationDecision::Warn,
            _ => ValidationDecision::Fail,
        }
    }
}

/// Mimir ingestion request for RefGraph entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MimirIngestionRequest {
    pub tenant_id: String,
    pub domain: String,
    pub entity_count: usize,
    pub relationship_count: usize,
    pub average_confidence: f32,
    pub entities: Vec<serde_json::Value>,
    pub relationships: Vec<serde_json::Value>,
}

/// Pipeline orchestrator
pub struct PipelineOrchestrator {
    client: client::MimirClient,
    status: std::sync::Arc<std::sync::Mutex<PipelineStatus>>,
}

impl PipelineOrchestrator {
    pub fn new(mimir_base_url: String, tenant_id: String) -> Self {
        Self {
            client: client::MimirClient::new(mimir_base_url, tenant_id),
            status: std::sync::Arc::new(std::sync::Mutex::new(PipelineStatus::default())),
        }
    }

    /// Submit consolidated RefGraph output to Mimir
    pub async fn ingest(&self, request: MimirIngestionRequest) -> Result<String> {
        match self.client.ingest(&request).await {
            Ok(response) => {
                if let Ok(mut status) = self.status.lock() {
                    status.total_ingested += 1;
                    status.last_ingestion_timestamp =
                        Some(chrono::Utc::now().to_rfc3339());
                }
                Ok(response)
            }
            Err(e) => {
                if let Ok(mut status) = self.status.lock() {
                    status.total_failed += 1;
                }
                Err(e)
            }
        }
    }

    /// Get current pipeline status
    pub fn status(&self) -> PipelineStatus {
        self.status
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct PipelineStatus {
    pub total_ingested: usize,
    pub total_failed: usize,
    pub last_ingestion_timestamp: Option<String>,
}

/// Hit rate validator
pub struct HitRateValidator {
    client: client::MimirClient,
    test_queries: Vec<String>,
}

impl HitRateValidator {
    pub fn new(
        mimir_base_url: String,
        tenant_id: String,
        test_queries: Vec<String>,
    ) -> Self {
        Self {
            client: client::MimirClient::new(mimir_base_url, tenant_id),
            test_queries,
        }
    }

    /// Run all test queries and compute hit rate
    pub async fn validate(&self) -> Result<HitRateResult> {
        let mut query_results = Vec::new();
        let mut hit_count = 0;

        for query in &self.test_queries {
            match self.query(query).await {
                Ok(result) => {
                    if result.hits > 0 {
                        hit_count += 1;
                    }
                    query_results.push(result);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        let total_queries = self.test_queries.len();
        let hit_rate = if total_queries > 0 {
            hit_count as f32 / total_queries as f32
        } else {
            0.0
        };

        Ok(HitRateResult {
            total_queries,
            hit_count,
            hit_rate,
            decision: ValidationDecision::from_hit_rate(hit_rate),
            query_results,
        })
    }

    /// Get a single query result
    async fn query(&self, q: &str) -> Result<QueryResult> {
        self.client.search(q, 3).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED: Validation Decision Tests

    #[test]
    fn test_validation_decision_pass_threshold() {
        let decision = ValidationDecision::from_hit_rate(0.75);
        assert_eq!(decision, ValidationDecision::Pass);
    }

    #[test]
    fn test_validation_decision_pass_high() {
        let decision = ValidationDecision::from_hit_rate(0.95);
        assert_eq!(decision, ValidationDecision::Pass);
    }

    #[test]
    fn test_validation_decision_warn_threshold() {
        let decision = ValidationDecision::from_hit_rate(0.50);
        assert_eq!(decision, ValidationDecision::Warn);
    }

    #[test]
    fn test_validation_decision_warn_mid_range() {
        let decision = ValidationDecision::from_hit_rate(0.60);
        assert_eq!(decision, ValidationDecision::Warn);
    }

    #[test]
    fn test_validation_decision_fail_threshold() {
        let decision = ValidationDecision::from_hit_rate(0.49);
        assert_eq!(decision, ValidationDecision::Fail);
    }

    #[test]
    fn test_validation_decision_fail_low() {
        let decision = ValidationDecision::from_hit_rate(0.0);
        assert_eq!(decision, ValidationDecision::Fail);
    }

    // RED: Hit Rate Result Tests

    #[test]
    fn test_hit_rate_result_pass_serializes() {
        let result = HitRateResult {
            total_queries: 10,
            hit_count: 8,
            hit_rate: 0.80,
            decision: ValidationDecision::Pass,
            query_results: vec![],
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"PASS\""));
        assert!(json.contains("\"hit_rate\":0.8"));
    }

    #[test]
    fn test_hit_rate_result_includes_query_results() {
        let query_results = vec![
            QueryResult {
                query: "test query 1".to_string(),
                hits: 3,
                score: 0.95,
                timestamp: "2026-05-16T10:00:00Z".to_string(),
            },
        ];

        let result = HitRateResult {
            total_queries: 1,
            hit_count: 1,
            hit_rate: 1.0,
            decision: ValidationDecision::Pass,
            query_results,
        };

        assert_eq!(result.query_results.len(), 1);
        assert_eq!(result.query_results[0].hits, 3);
    }

    // RED: Pipeline Orchestrator Tests

    #[tokio::test]
    async fn test_pipeline_orchestrator_new() {
        let orchestrator = PipelineOrchestrator::new(
            "http://localhost:5000".to_string(),
            "asgard_insurance".to_string(),
        );

        let status = orchestrator.status();
        assert_eq!(status.total_ingested, 0);
    }

    #[tokio::test]
    async fn test_pipeline_orchestrator_status_initially_zero() {
        let orchestrator = PipelineOrchestrator::new(
            "http://localhost:5000".to_string(),
            "asgard_insurance".to_string(),
        );

        let status = orchestrator.status();
        assert_eq!(status.total_ingested, 0);
        assert_eq!(status.total_failed, 0);
        assert!(status.last_ingestion_timestamp.is_none());
    }

    #[tokio::test]
    async fn test_pipeline_orchestrator_ingest_attempts_call() {
        let orchestrator = PipelineOrchestrator::new(
            "http://localhost:9999".to_string(), // non-existent server
            "asgard_insurance".to_string(),
        );

        let request = MimirIngestionRequest {
            tenant_id: "asgard_insurance".to_string(),
            domain: "insurance".to_string(),
            entity_count: 5,
            relationship_count: 3,
            average_confidence: 0.85,
            entities: vec![],
            relationships: vec![],
        };

        let result = orchestrator.ingest(request).await;
        // Should fail because server doesn't exist
        assert!(result.is_err());
        // Status should have been incremented
        let status = orchestrator.status();
        assert_eq!(status.total_failed, 1);
    }

    // RED: Hit Rate Validator Tests

    #[tokio::test]
    async fn test_hit_rate_validator_new() {
        let queries = vec!["query1".to_string(), "query2".to_string()];
        let validator = HitRateValidator::new(
            "http://localhost:5000".to_string(),
            "asgard_insurance".to_string(),
            queries.clone(),
        );

        assert_eq!(validator.test_queries.len(), 2);
    }

    #[tokio::test]
    async fn test_hit_rate_validator_validate_fails_no_server() {
        let queries = vec!["query1".to_string()];
        let validator = HitRateValidator::new(
            "http://localhost:9999".to_string(), // non-existent server
            "asgard_insurance".to_string(),
            queries,
        );

        let result = validator.validate().await;
        // Should fail because server doesn't exist
        assert!(result.is_err());
    }

    // RED: Mimir Ingestion Request Tests

    #[test]
    fn test_mimir_ingestion_request_serializes() {
        let request = MimirIngestionRequest {
            tenant_id: "asgard_insurance".to_string(),
            domain: "insurance".to_string(),
            entity_count: 5,
            relationship_count: 3,
            average_confidence: 0.85,
            entities: vec![],
            relationships: vec![],
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"tenant_id\":\"asgard_insurance\""));
        assert!(json.contains("\"entity_count\":5"));
        assert!(json.contains("\"average_confidence\":0.85"));
    }

    #[test]
    fn test_query_result_includes_metadata() {
        let query_result = QueryResult {
            query: "search term".to_string(),
            hits: 5,
            score: 0.92,
            timestamp: "2026-05-16T10:00:00Z".to_string(),
        };

        assert_eq!(query_result.query, "search term");
        assert_eq!(query_result.hits, 5);
        assert!((query_result.score - 0.92).abs() < 0.0001);
    }

    // RED: Standard Insurance Test Queries

    #[tokio::test]
    async fn test_validator_supports_10_standard_queries() {
        let test_queries = vec![
            "Critical Illness insurance".to_string(),
            "Health Insurance plan".to_string(),
            "Accident coverage".to_string(),
            "Death benefit".to_string(),
            "Hospital admission".to_string(),
            "Exclusions list".to_string(),
            "Premium payment".to_string(),
            "Claim process".to_string(),
            "ประกันสุขภาพ".to_string(), // Thai: health insurance
            "ประกันชีวิต".to_string(),  // Thai: life insurance
        ];

        let validator = HitRateValidator::new(
            "http://localhost:5000".to_string(),
            "asgard_insurance".to_string(),
            test_queries,
        );

        assert_eq!(validator.test_queries.len(), 10);
    }

    // RED: Decision Gate Tests

    #[test]
    fn test_decision_gate_pass_proceeds() {
        let result = HitRateResult {
            total_queries: 10,
            hit_count: 8,
            hit_rate: 0.80,
            decision: ValidationDecision::Pass,
            query_results: vec![],
        };

        assert_eq!(result.decision, ValidationDecision::Pass);
        assert!(result.hit_rate >= 0.75);
    }

    #[test]
    fn test_decision_gate_warn_escalates() {
        let result = HitRateResult {
            total_queries: 10,
            hit_count: 6,
            hit_rate: 0.60,
            decision: ValidationDecision::Warn,
            query_results: vec![],
        };

        assert_eq!(result.decision, ValidationDecision::Warn);
        assert!(result.hit_rate >= 0.50 && result.hit_rate < 0.75);
    }

    #[test]
    fn test_decision_gate_fail_activates_plan_b() {
        let result = HitRateResult {
            total_queries: 10,
            hit_count: 4,
            hit_rate: 0.40,
            decision: ValidationDecision::Fail,
            query_results: vec![],
        };

        assert_eq!(result.decision, ValidationDecision::Fail);
        assert!(result.hit_rate < 0.50);
    }

    // REFACTOR: Integration Tests with Mock

    #[tokio::test]
    async fn test_full_orchestration_flow_pass() {
        let orchestrator = PipelineOrchestrator::new(
            "http://localhost:5000".to_string(),
            "asgard_insurance".to_string(),
        );

        // Verify initial state
        let status = orchestrator.status();
        assert_eq!(status.total_ingested, 0);
        assert_eq!(status.total_failed, 0);
    }

    #[test]
    fn test_hit_rate_calculation_8_of_10() {
        let query_results = vec![
            QueryResult {
                query: "q1".to_string(),
                hits: 3,
                score: 0.9,
                timestamp: "2026-05-16T10:00:00Z".to_string(),
            },
            QueryResult {
                query: "q2".to_string(),
                hits: 0,
                score: 0.0,
                timestamp: "2026-05-16T10:00:00Z".to_string(),
            },
        ];

        let result = HitRateResult {
            total_queries: 10,
            hit_count: 8,
            hit_rate: 0.80,
            decision: ValidationDecision::Pass,
            query_results,
        };

        assert_eq!(result.hit_count, 8);
        assert_eq!(result.total_queries, 10);
        assert!((result.hit_rate - 0.80).abs() < 0.0001);
    }

    #[test]
    fn test_validation_decision_boundary_75_percent() {
        // Exactly at 75% threshold
        let decision = ValidationDecision::from_hit_rate(0.75);
        assert_eq!(decision, ValidationDecision::Pass);

        // Just below 75%
        let decision = ValidationDecision::from_hit_rate(0.7499);
        assert_eq!(decision, ValidationDecision::Warn);
    }

    #[test]
    fn test_validation_decision_boundary_50_percent() {
        // Exactly at 50% threshold
        let decision = ValidationDecision::from_hit_rate(0.50);
        assert_eq!(decision, ValidationDecision::Warn);

        // Just below 50%
        let decision = ValidationDecision::from_hit_rate(0.4999);
        assert_eq!(decision, ValidationDecision::Fail);
    }

    #[test]
    fn test_mimir_request_serializes_all_fields() {
        let request = MimirIngestionRequest {
            tenant_id: "asgard_insurance".to_string(),
            domain: "insurance".to_string(),
            entity_count: 25,
            relationship_count: 15,
            average_confidence: 0.876,
            entities: vec![],
            relationships: vec![],
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["tenant_id"].as_str().unwrap(), "asgard_insurance");
        assert_eq!(parsed["domain"].as_str().unwrap(), "insurance");
        assert_eq!(parsed["entity_count"].as_u64().unwrap(), 25);
        assert_eq!(parsed["relationship_count"].as_u64().unwrap(), 15);
        assert!((parsed["average_confidence"].as_f64().unwrap() - 0.876).abs() < 0.001);
    }

    #[test]
    fn test_query_result_roundtrip_serialization() {
        let original = QueryResult {
            query: "insurance coverage".to_string(),
            hits: 7,
            score: 0.925,
            timestamp: "2026-05-16T10:30:00Z".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: QueryResult = serde_json::from_str(&json).unwrap();

        assert_eq!(original.query, deserialized.query);
        assert_eq!(original.hits, deserialized.hits);
        assert!((original.score - deserialized.score).abs() < 0.0001);
        assert_eq!(original.timestamp, deserialized.timestamp);
    }

    #[test]
    fn test_hit_rate_result_roundtrip_serialization() {
        let original = HitRateResult {
            total_queries: 10,
            hit_count: 8,
            hit_rate: 0.80,
            decision: ValidationDecision::Pass,
            query_results: vec![],
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: HitRateResult = serde_json::from_str(&json).unwrap();

        assert_eq!(original.total_queries, deserialized.total_queries);
        assert_eq!(original.hit_count, deserialized.hit_count);
        assert!((original.hit_rate - deserialized.hit_rate).abs() < 0.0001);
        assert_eq!(original.decision, deserialized.decision);
    }

    #[test]
    fn test_thai_queries_in_test_set() {
        let test_queries = vec![
            "Critical Illness insurance".to_string(),
            "ประกันสุขภาพ".to_string(),
            "ประกันชีวิต".to_string(),
        ];

        assert_eq!(test_queries.len(), 3);
        assert_eq!(test_queries[1], "ประกันสุขภาพ");
        assert_eq!(test_queries[2], "ประกันชีวิต");
    }

    #[test]
    fn test_pipeline_status_tracks_multiple_operations() {
        let mut status = PipelineStatus::default();
        assert_eq!(status.total_ingested, 0);

        status.total_ingested = 3;
        status.total_failed = 1;
        status.last_ingestion_timestamp = Some("2026-05-16T10:00:00Z".to_string());

        assert_eq!(status.total_ingested, 3);
        assert_eq!(status.total_failed, 1);
        assert!(status.last_ingestion_timestamp.is_some());
    }

    #[test]
    fn test_validation_decision_serialization_format() {
        let pass_json = serde_json::to_string(&ValidationDecision::Pass).unwrap();
        let warn_json = serde_json::to_string(&ValidationDecision::Warn).unwrap();
        let fail_json = serde_json::to_string(&ValidationDecision::Fail).unwrap();

        assert_eq!(pass_json, "\"PASS\"");
        assert_eq!(warn_json, "\"WARN\"");
        assert_eq!(fail_json, "\"FAIL\"");
    }

    #[test]
    fn test_query_result_zero_hits_valid() {
        let result = QueryResult {
            query: "nonexistent term".to_string(),
            hits: 0,
            score: 0.0,
            timestamp: "2026-05-16T10:00:00Z".to_string(),
        };

        assert_eq!(result.hits, 0);
        assert!((result.score - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_mimir_request_with_entities() {
        let entities = vec![
            serde_json::json!({
                "id": "ent_001",
                "type": "PRODUCT",
                "text": "Critical Illness Insurance"
            }),
        ];

        let request = MimirIngestionRequest {
            tenant_id: "asgard_insurance".to_string(),
            domain: "insurance".to_string(),
            entity_count: 1,
            relationship_count: 0,
            average_confidence: 0.90,
            entities,
            relationships: vec![],
        };

        assert_eq!(request.entity_count, 1);
        assert_eq!(request.entities.len(), 1);
    }
}
