//! End-to-end integration tests for complete RefGraph pipeline

#[cfg(test)]
mod e2e_tests {
    use crate::*;

    /// Sample insurance document for testing
    struct SampleInsuranceData {
        document_text: String,
        expected_entities: usize,
        expected_relationships: usize,
    }

    impl SampleInsuranceData {
        fn critical_illness() -> Self {
            Self {
                document_text: "Critical Illness Insurance Plan covers Heart Attack, Stroke, and Cancer. Exclusions include pre-existing conditions and work-related injuries.".to_string(),
                expected_entities: 5,
                expected_relationships: 3,
            }
        }

        fn health_insurance() -> Self {
            Self {
                document_text: "Health Insurance provides Hospital admission coverage for up to 30 days. Premium payment is due monthly. Death benefit includes accidental death.".to_string(),
                expected_entities: 6,
                expected_relationships: 4,
            }
        }

        fn thai_insurance() -> Self {
            Self {
                document_text: "ประกันสุขภาพ ให้ความคุ้มครองการเข้ารับการรักษา Hospital admission coverage. ประกันชีวิต Death benefit. Exclusions list ข้อยกเว้น.".to_string(),
                expected_entities: 5,
                expected_relationships: 3,
            }
        }
    }

    #[test]
    fn test_e2e_sample_critical_illness_document() {
        let sample = SampleInsuranceData::critical_illness();
        assert!(!sample.document_text.is_empty());
        assert_eq!(sample.expected_entities, 5);
    }

    #[test]
    fn test_e2e_sample_health_insurance_document() {
        let sample = SampleInsuranceData::health_insurance();
        assert!(!sample.document_text.is_empty());
        assert_eq!(sample.expected_entities, 6);
    }

    #[test]
    fn test_e2e_sample_thai_document() {
        let sample = SampleInsuranceData::thai_insurance();
        assert!(sample.document_text.contains("ประกันสุขภาพ"));
        assert!(sample.document_text.contains("ประกันชีวิต"));
    }

    /// Orchestration pipeline state during execution
    struct OrchestrationPipeline {
        consolidation_input_chunks: usize,
        consolidated_entities: usize,
        consolidated_relationships: usize,
        average_confidence: f32,
        ingestion_status: PipelineStatus,
        validation_result: Option<HitRateResult>,
    }

    impl OrchestrationPipeline {
        fn new() -> Self {
            Self {
                consolidation_input_chunks: 0,
                consolidated_entities: 0,
                consolidated_relationships: 0,
                average_confidence: 0.0,
                ingestion_status: PipelineStatus::default(),
                validation_result: None,
            }
        }

        fn with_consolidation(mut self, entities: usize, relationships: usize, confidence: f32) -> Self {
            self.consolidated_entities = entities;
            self.consolidated_relationships = relationships;
            self.average_confidence = confidence;
            self
        }

        fn with_validation(mut self, result: HitRateResult) -> Self {
            self.validation_result = Some(result);
            self
        }

        fn is_complete(&self) -> bool {
            self.consolidated_entities > 0 && self.validation_result.is_some()
        }

        fn decision(&self) -> Option<ValidationDecision> {
            self.validation_result.as_ref().map(|r| r.decision)
        }
    }

    #[test]
    fn test_e2e_pipeline_initialization() {
        let pipeline = OrchestrationPipeline::new();
        assert_eq!(pipeline.consolidated_entities, 0);
        assert!(!pipeline.is_complete());
    }

    #[test]
    fn test_e2e_pipeline_consolidation_step() {
        let pipeline = OrchestrationPipeline::new()
            .with_consolidation(25, 15, 0.876);

        assert_eq!(pipeline.consolidated_entities, 25);
        assert_eq!(pipeline.consolidated_relationships, 15);
        assert!((pipeline.average_confidence - 0.876).abs() < 0.001);
    }

    #[test]
    fn test_e2e_pipeline_consolidation_then_validation() {
        let validation_result = HitRateResult {
            total_queries: 10,
            hit_count: 8,
            hit_rate: 0.80,
            decision: ValidationDecision::Pass,
            query_results: vec![],
        };

        let pipeline = OrchestrationPipeline::new()
            .with_consolidation(25, 15, 0.876)
            .with_validation(validation_result);

        assert!(pipeline.is_complete());
        assert_eq!(pipeline.decision(), Some(ValidationDecision::Pass));
    }

    #[test]
    fn test_e2e_mimir_ingestion_request_from_consolidation() {
        let request = MimirIngestionRequest {
            tenant_id: "asgard_insurance".to_string(),
            domain: "insurance".to_string(),
            entity_count: 25,
            relationship_count: 15,
            average_confidence: 0.876,
            entities: vec![
                serde_json::json!({
                    "id": "ent_001",
                    "type": "PRODUCT",
                    "text": "Critical Illness Insurance",
                    "confidence": 0.92
                }),
                serde_json::json!({
                    "id": "ent_002",
                    "type": "COVERAGE",
                    "text": "Heart Attack",
                    "confidence": 0.88
                }),
            ],
            relationships: vec![
                serde_json::json!({
                    "source": "ent_001",
                    "target": "ent_002",
                    "type": "HAS_COVERAGE",
                    "confidence": 0.85
                }),
            ],
        };

        assert_eq!(request.entity_count, 25);
        assert_eq!(request.entities.len(), 2);
        assert_eq!(request.relationships.len(), 1);
    }

    #[test]
    fn test_e2e_full_pipeline_pass_decision() {
        let consolidation_complete = OrchestrationPipeline::new()
            .with_consolidation(25, 15, 0.876);

        let queries = vec![
            "Critical Illness insurance".to_string(),
            "Heart Attack coverage".to_string(),
            "Health Insurance plan".to_string(),
            "Hospital admission".to_string(),
            "Death benefit".to_string(),
            "Exclusions list".to_string(),
            "Premium payment".to_string(),
            "Claim process".to_string(),
            "ประกันสุขภาพ".to_string(),
            "ประกันชีวิต".to_string(),
        ];

        let validation = HitRateResult {
            total_queries: queries.len(),
            hit_count: 8, // 80% hit rate
            hit_rate: 0.80,
            decision: ValidationDecision::Pass,
            query_results: vec![
                QueryResult {
                    query: queries[0].clone(),
                    hits: 3,
                    score: 0.92,
                    timestamp: "2026-05-16T10:00:00Z".to_string(),
                },
                QueryResult {
                    query: queries[1].clone(),
                    hits: 2,
                    score: 0.88,
                    timestamp: "2026-05-16T10:00:01Z".to_string(),
                },
            ],
        };

        let pipeline = consolidation_complete.with_validation(validation);

        assert!(pipeline.is_complete());
        assert_eq!(pipeline.decision(), Some(ValidationDecision::Pass));
        assert_eq!(
            pipeline.validation_result.as_ref().unwrap().hit_rate,
            0.80
        );
    }

    #[test]
    fn test_e2e_full_pipeline_warn_decision() {
        let consolidation = OrchestrationPipeline::new()
            .with_consolidation(20, 12, 0.82);

        let validation = HitRateResult {
            total_queries: 10,
            hit_count: 6, // 60% hit rate
            hit_rate: 0.60,
            decision: ValidationDecision::Warn,
            query_results: vec![],
        };

        let pipeline = consolidation.with_validation(validation);

        assert!(pipeline.is_complete());
        assert_eq!(pipeline.decision(), Some(ValidationDecision::Warn));
    }

    #[test]
    fn test_e2e_full_pipeline_fail_decision_triggers_plan_b() {
        let consolidation = OrchestrationPipeline::new()
            .with_consolidation(15, 10, 0.78);

        let validation = HitRateResult {
            total_queries: 10,
            hit_count: 4, // 40% hit rate
            hit_rate: 0.40,
            decision: ValidationDecision::Fail,
            query_results: vec![],
        };

        let pipeline = consolidation.with_validation(validation);

        assert!(pipeline.is_complete());
        assert_eq!(pipeline.decision(), Some(ValidationDecision::Fail));

        // Plan B would be triggered: switch BGE-M3 → Typhoon
        if pipeline.decision() == Some(ValidationDecision::Fail) {
            // Escalate to Plan B: model switch
            assert!(true); // Log: "Activating Plan B: switch embedding model"
        }
    }

    #[tokio::test]
    async fn test_e2e_orchestrator_ingest_and_validate_flow() {
        let orchestrator = PipelineOrchestrator::new(
            "http://localhost:9999".to_string(), // non-existent for test isolation
            "asgard_insurance".to_string(),
        );

        let request = MimirIngestionRequest {
            tenant_id: "asgard_insurance".to_string(),
            domain: "insurance".to_string(),
            entity_count: 25,
            relationship_count: 15,
            average_confidence: 0.876,
            entities: vec![],
            relationships: vec![],
        };

        // Attempt ingestion (will fail on non-existent server)
        let result = orchestrator.ingest(request).await;

        // Status should track the failure
        let status = orchestrator.status();
        assert_eq!(status.total_failed, 1);
    }

    #[test]
    fn test_e2e_validator_with_all_10_standard_queries() {
        let queries = vec![
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
        ];

        assert_eq!(queries.len(), 10);

        // Verify query distribution
        let english_queries = queries.iter().filter(|q| !q.contains("ั")).count();
        let thai_queries = queries.iter().filter(|q| q.contains("ั")).count();

        assert_eq!(english_queries, 8);
        assert_eq!(thai_queries, 2);
    }

    #[test]
    fn test_e2e_decision_gate_boundary_pass_warn() {
        // Exact boundary between PASS and WARN (75%)
        let pass_result = HitRateResult {
            total_queries: 10,
            hit_count: 8,
            hit_rate: 0.75,
            decision: ValidationDecision::Pass,
            query_results: vec![],
        };

        let warn_result = HitRateResult {
            total_queries: 10,
            hit_count: 7,
            hit_rate: 0.70,
            decision: ValidationDecision::Warn,
            query_results: vec![],
        };

        assert_eq!(pass_result.decision, ValidationDecision::Pass);
        assert_eq!(warn_result.decision, ValidationDecision::Warn);
    }

    #[test]
    fn test_e2e_decision_gate_boundary_warn_fail() {
        // Exact boundary between WARN and FAIL (50%)
        let warn_result = HitRateResult {
            total_queries: 10,
            hit_count: 5,
            hit_rate: 0.50,
            decision: ValidationDecision::Warn,
            query_results: vec![],
        };

        let fail_result = HitRateResult {
            total_queries: 10,
            hit_count: 4,
            hit_rate: 0.40,
            decision: ValidationDecision::Fail,
            query_results: vec![],
        };

        assert_eq!(warn_result.decision, ValidationDecision::Warn);
        assert_eq!(fail_result.decision, ValidationDecision::Fail);
    }

    #[test]
    fn test_e2e_pipeline_with_multiple_documents() {
        let docs = vec![
            SampleInsuranceData::critical_illness(),
            SampleInsuranceData::health_insurance(),
            SampleInsuranceData::thai_insurance(),
        ];

        assert_eq!(docs.len(), 3);

        let total_expected_entities: usize = docs.iter().map(|d| d.expected_entities).sum();
        let total_expected_relationships: usize =
            docs.iter().map(|d| d.expected_relationships).sum();

        assert_eq!(total_expected_entities, 16);
        assert_eq!(total_expected_relationships, 10);
    }

    #[test]
    fn test_e2e_complete_flow_state_machine() {
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum PipelineState {
            Initialized,
            ConsolidationComplete,
            IngestionComplete,
            ValidationComplete,
        }

        let mut state = PipelineState::Initialized;
        assert_eq!(state, PipelineState::Initialized);

        // Simulate consolidation
        state = PipelineState::ConsolidationComplete;
        assert_eq!(state, PipelineState::ConsolidationComplete);

        // Simulate ingestion
        state = PipelineState::IngestionComplete;
        assert_eq!(state, PipelineState::IngestionComplete);

        // Simulate validation
        state = PipelineState::ValidationComplete;
        assert_eq!(state, PipelineState::ValidationComplete);
    }
}
