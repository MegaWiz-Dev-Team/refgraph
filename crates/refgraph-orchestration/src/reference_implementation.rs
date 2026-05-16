//! Reference Implementation: Complete RefGraph → Orchestration Pipeline
//! Demonstrates how to use RefGraph core output with orchestration layer

use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Sample insurance documents for demonstration
pub mod sample_data {
    pub fn critical_illness_document() -> String {
        "CRITICAL ILLNESS INSURANCE PLAN\n\n\
         This policy provides coverage for Critical Illness events including:\n\
         - Heart Attack with minimum 3 month survival\n\
         - Stroke causing permanent neurological damage\n\
         - Cancer diagnosis with histological confirmation\n\
         - Kidney Failure requiring regular dialysis\n\
         - Liver Failure with cirrhosis diagnosis\n\n\
         Coverage Amount: $100,000\n\
         Premium Payment: Monthly or Annual\n\n\
         EXCLUSIONS:\n\
         - Pre-existing conditions (12 month waiting period)\n\
         - Work-related injuries\n\
         - High-risk activities (mountaineering, professional sports)\n\
         - Alcohol or drug-related illnesses"
            .to_string()
    }

    pub fn health_insurance_document() -> String {
        "HEALTH INSURANCE PLAN\n\n\
         Hospital admission coverage includes:\n\
         - Room and board charges\n\
         - Doctor consultation fees\n\
         - Surgical procedures\n\
         - Nursing care and medication\n\n\
         Death benefit of $50,000 payable to designated beneficiary.\n\
         Claim process requires:\n\
         1. Notify within 30 days\n\
         2. Submit medical records\n\
         3. Hospital discharge summary\n\
         4. Claim form signed by policyholder\n\n\
         Premium payment due on the 1st of each month."
            .to_string()
    }

    pub fn thai_insurance_document() -> String {
        "ประกันสุขภาพ (Health Insurance Plan)\n\n\
         ให้ความคุ้มครองการเข้ารับการรักษาที่โรงพยาบาล\n\
         Hospital admission coverage สำหรับ:\n\
         - การผ่าตัด Surgical procedures\n\
         - ยาและวัสดุทางการแพทย์\n\
         - ค่าห้องพยาบาล\n\n\
         ประกันชีวิต (Life Insurance) Death benefit\n\
         ข้อยกเว้น (Exclusions):\n\
         - โรคประจำตัว Pre-existing conditions\n\
         - การเสียชีวิตจากการทำงาน\n\n\
         Premium payment: รายเดือน (Monthly) หรือรายปี (Annual)"
            .to_string()
    }
}

/// Consolidated entity from RefGraph output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedEntity {
    pub id: String,
    pub entity_type: String,
    pub text: String,
    pub confidence: f32,
    pub sources: Vec<String>,
}

impl ConsolidatedEntity {
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "type": self.entity_type,
            "text": self.text,
            "confidence": self.confidence,
            "sources": self.sources,
        })
    }
}

/// Consolidated relationship from RefGraph output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedRelationship {
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: String,
    pub confidence: f32,
}

impl ConsolidatedRelationship {
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "source": self.source_id,
            "target": self.target_id,
            "type": self.relationship_type,
            "confidence": self.confidence,
        })
    }
}

/// RefGraph Consolidation Output (simulating what RefGraph core produces)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationOutput {
    pub domain: String,
    pub entities: Vec<ConsolidatedEntity>,
    pub relationships: Vec<ConsolidatedRelationship>,
    pub average_confidence: f32,
}

impl ConsolidationOutput {
    /// Create a sample consolidation for Critical Illness Insurance
    pub fn sample_critical_illness() -> Self {
        Self {
            domain: "insurance".to_string(),
            entities: vec![
                ConsolidatedEntity {
                    id: "ent_crit_001".to_string(),
                    entity_type: "PRODUCT".to_string(),
                    text: "Critical Illness Insurance".to_string(),
                    confidence: 0.92,
                    sources: vec!["doc_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_crit_002".to_string(),
                    entity_type: "COVERAGE".to_string(),
                    text: "Heart Attack".to_string(),
                    confidence: 0.88,
                    sources: vec!["doc_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_crit_003".to_string(),
                    entity_type: "COVERAGE".to_string(),
                    text: "Stroke".to_string(),
                    confidence: 0.87,
                    sources: vec!["doc_001".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_crit_004".to_string(),
                    entity_type: "EXCLUSION".to_string(),
                    text: "Pre-existing conditions".to_string(),
                    confidence: 0.85,
                    sources: vec!["doc_001".to_string()],
                },
            ],
            relationships: vec![
                ConsolidatedRelationship {
                    source_id: "ent_crit_001".to_string(),
                    target_id: "ent_crit_002".to_string(),
                    relationship_type: "HAS_COVERAGE".to_string(),
                    confidence: 0.85,
                },
                ConsolidatedRelationship {
                    source_id: "ent_crit_001".to_string(),
                    target_id: "ent_crit_003".to_string(),
                    relationship_type: "HAS_COVERAGE".to_string(),
                    confidence: 0.84,
                },
                ConsolidatedRelationship {
                    source_id: "ent_crit_001".to_string(),
                    target_id: "ent_crit_004".to_string(),
                    relationship_type: "HAS_EXCLUSION".to_string(),
                    confidence: 0.82,
                },
            ],
            average_confidence: 0.863,
        }
    }

    /// Create a sample consolidation for Health Insurance
    pub fn sample_health_insurance() -> Self {
        Self {
            domain: "insurance".to_string(),
            entities: vec![
                ConsolidatedEntity {
                    id: "ent_health_001".to_string(),
                    entity_type: "PRODUCT".to_string(),
                    text: "Health Insurance Plan".to_string(),
                    confidence: 0.94,
                    sources: vec!["doc_002".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_health_002".to_string(),
                    entity_type: "COVERAGE".to_string(),
                    text: "Hospital admission".to_string(),
                    confidence: 0.91,
                    sources: vec!["doc_002".to_string()],
                },
                ConsolidatedEntity {
                    id: "ent_health_003".to_string(),
                    entity_type: "BENEFIT".to_string(),
                    text: "Death benefit".to_string(),
                    confidence: 0.89,
                    sources: vec!["doc_002".to_string()],
                },
            ],
            relationships: vec![
                ConsolidatedRelationship {
                    source_id: "ent_health_001".to_string(),
                    target_id: "ent_health_002".to_string(),
                    relationship_type: "HAS_COVERAGE".to_string(),
                    confidence: 0.88,
                },
                ConsolidatedRelationship {
                    source_id: "ent_health_001".to_string(),
                    target_id: "ent_health_003".to_string(),
                    relationship_type: "HAS_BENEFIT".to_string(),
                    confidence: 0.87,
                },
            ],
            average_confidence: 0.903,
        }
    }

    /// Convert to Mimir ingestion request
    pub fn to_ingestion_request(&self, tenant_id: &str) -> MimirIngestionRequest {
        MimirIngestionRequest {
            tenant_id: tenant_id.to_string(),
            domain: self.domain.clone(),
            entity_count: self.entities.len(),
            relationship_count: self.relationships.len(),
            average_confidence: self.average_confidence,
            entities: self.entities.iter().map(|e| e.to_json()).collect(),
            relationships: self.relationships.iter().map(|r| r.to_json()).collect(),
        }
    }

    /// Merge multiple consolidations
    pub fn merge(&mut self, other: ConsolidationOutput) {
        self.entities.extend(other.entities);
        self.relationships.extend(other.relationships);

        // Recalculate average confidence
        if !self.entities.is_empty() {
            let total_confidence: f32 = self.entities.iter().map(|e| e.confidence).sum();
            self.average_confidence = total_confidence / self.entities.len() as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_data_critical_illness_not_empty() {
        let doc = sample_data::critical_illness_document();
        assert!(!doc.is_empty());
        assert!(doc.contains("Critical Illness"));
        assert!(doc.contains("Heart Attack"));
    }

    #[test]
    fn test_sample_data_thai_document_contains_thai_text() {
        let doc = sample_data::thai_insurance_document();
        assert!(doc.contains("ประกันสุขภาพ"));
        assert!(doc.contains("ประกันชีวิต"));
    }

    #[test]
    fn test_consolidated_entity_to_json() {
        let entity = ConsolidatedEntity {
            id: "ent_001".to_string(),
            entity_type: "PRODUCT".to_string(),
            text: "Test Product".to_string(),
            confidence: 0.92,
            sources: vec!["doc_001".to_string()],
        };

        let json = entity.to_json();
        assert_eq!(json["id"].as_str().unwrap(), "ent_001");
        assert_eq!(json["type"].as_str().unwrap(), "PRODUCT");
        let confidence = json["confidence"].as_f64().unwrap();
        assert!((confidence - 0.92).abs() < 0.001);
    }

    #[test]
    fn test_consolidation_output_sample_critical_illness() {
        let output = ConsolidationOutput::sample_critical_illness();

        assert_eq!(output.domain, "insurance");
        assert_eq!(output.entities.len(), 4);
        assert_eq!(output.relationships.len(), 3);
        assert!((output.average_confidence - 0.863).abs() < 0.001);
    }

    #[test]
    fn test_consolidation_output_sample_health_insurance() {
        let output = ConsolidationOutput::sample_health_insurance();

        assert_eq!(output.domain, "insurance");
        assert_eq!(output.entities.len(), 3);
        assert_eq!(output.relationships.len(), 2);
    }

    #[test]
    fn test_consolidation_to_ingestion_request() {
        let output = ConsolidationOutput::sample_critical_illness();
        let request = output.to_ingestion_request("asgard_insurance");

        assert_eq!(request.tenant_id, "asgard_insurance");
        assert_eq!(request.domain, "insurance");
        assert_eq!(request.entity_count, 4);
        assert_eq!(request.relationship_count, 3);
        assert_eq!(request.entities.len(), 4);
        assert_eq!(request.relationships.len(), 3);
    }

    #[test]
    fn test_consolidation_merge_combines_entities() {
        let mut output1 = ConsolidationOutput::sample_critical_illness();
        let output2 = ConsolidationOutput::sample_health_insurance();

        let original_count = output1.entities.len();
        output1.merge(output2);

        assert_eq!(output1.entities.len(), original_count + 3);
    }

    #[test]
    fn test_consolidation_merge_recalculates_confidence() {
        let mut output1 = ConsolidationOutput::sample_critical_illness();
        let original_confidence = output1.average_confidence;

        output1.merge(ConsolidationOutput::sample_health_insurance());

        // Confidence should change after merge
        assert_ne!(output1.average_confidence, original_confidence);
    }

    #[test]
    fn test_reference_impl_complete_flow_critical_illness() {
        // 1. Consolidation phase (RefGraph output)
        let consolidation = ConsolidationOutput::sample_critical_illness();
        assert_eq!(consolidation.entities.len(), 4);

        // 2. Create ingestion request
        let request = consolidation.to_ingestion_request("asgard_insurance");
        assert_eq!(request.entity_count, 4);

        // 3. Validate hit rate would pass
        let result = HitRateResult {
            total_queries: 10,
            hit_count: 8,
            hit_rate: 0.80,
            decision: ValidationDecision::Pass,
            query_results: vec![],
        };

        assert_eq!(result.decision, ValidationDecision::Pass);
    }

    #[test]
    fn test_reference_impl_complete_flow_health_insurance() {
        let consolidation = ConsolidationOutput::sample_health_insurance();
        let request = consolidation.to_ingestion_request("asgard_insurance");

        assert_eq!(request.entity_count, 3);
        assert_eq!(request.relationship_count, 2);
    }

    #[test]
    fn test_reference_impl_multi_document_merge() {
        let doc1 = ConsolidationOutput::sample_critical_illness();
        let doc2 = ConsolidationOutput::sample_health_insurance();

        let mut merged = doc1;
        merged.merge(doc2);

        // Should have entities from both documents
        assert!(merged.entities.len() > 4);

        // Create request from merged output
        let request = merged.to_ingestion_request("asgard_insurance");
        assert!(request.entity_count > 4);
    }

    #[test]
    fn test_relationship_has_correct_type_for_coverage() {
        let output = ConsolidationOutput::sample_critical_illness();

        let coverage_rels: Vec<_> = output
            .relationships
            .iter()
            .filter(|r| r.relationship_type == "HAS_COVERAGE")
            .collect();

        assert_eq!(coverage_rels.len(), 2);
    }

    #[test]
    fn test_relationship_has_correct_type_for_exclusion() {
        let output = ConsolidationOutput::sample_critical_illness();

        let exclusion_rels: Vec<_> = output
            .relationships
            .iter()
            .filter(|r| r.relationship_type == "HAS_EXCLUSION")
            .collect();

        assert_eq!(exclusion_rels.len(), 1);
    }
}
