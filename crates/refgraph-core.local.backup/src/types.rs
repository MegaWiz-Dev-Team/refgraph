//! RefGraph type definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Raw chunk from web extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawChunk {
    pub chunk_id: String,
    pub content: String,
    pub source_url: String,
    pub page_index: Option<usize>,
    pub token_count: usize,
}

/// Extracted entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_id: String,
    pub text: String,
    pub entity_type: EntityType,
    pub confidence: f32,
    pub sources: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Entity type classification (domain-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    // Insurance domain
    Product,
    Coverage,
    Exclusion,
    Condition,
    Organization,

    // Medical domain
    Symptom,
    Diagnosis,
    Drug,
    Procedure,

    // Legal domain
    Clause,
    Party,
    Obligation,
    Right,
    Jurisdiction,

    // Finance domain
    AccountType,
    Fee,
    Rate,
    Regulation,

    // Generic / custom
    Custom(String),
    Other,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Insurance
            Self::Product => write!(f, "product"),
            Self::Coverage => write!(f, "coverage"),
            Self::Exclusion => write!(f, "exclusion"),
            Self::Condition => write!(f, "condition"),
            Self::Organization => write!(f, "organization"),

            // Medical
            Self::Symptom => write!(f, "symptom"),
            Self::Diagnosis => write!(f, "diagnosis"),
            Self::Drug => write!(f, "drug"),
            Self::Procedure => write!(f, "procedure"),

            // Legal
            Self::Clause => write!(f, "clause"),
            Self::Party => write!(f, "party"),
            Self::Obligation => write!(f, "obligation"),
            Self::Right => write!(f, "right"),
            Self::Jurisdiction => write!(f, "jurisdiction"),

            // Finance
            Self::AccountType => write!(f, "account_type"),
            Self::Fee => write!(f, "fee"),
            Self::Rate => write!(f, "rate"),
            Self::Regulation => write!(f, "regulation"),

            // Custom/Other
            Self::Custom(name) => write!(f, "{}", name),
            Self::Other => write!(f, "other"),
        }
    }
}

impl EntityType {
    /// Parse entity type from string (case-insensitive)
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            // Insurance
            "product" => Self::Product,
            "coverage" => Self::Coverage,
            "exclusion" => Self::Exclusion,
            "condition" => Self::Condition,
            "organization" => Self::Organization,

            // Medical
            "symptom" => Self::Symptom,
            "diagnosis" => Self::Diagnosis,
            "drug" => Self::Drug,
            "procedure" => Self::Procedure,

            // Legal
            "clause" => Self::Clause,
            "party" => Self::Party,
            "obligation" => Self::Obligation,
            "right" => Self::Right,
            "jurisdiction" => Self::Jurisdiction,

            // Finance
            "account_type" | "accounttype" => Self::AccountType,
            "fee" => Self::Fee,
            "rate" => Self::Rate,
            "regulation" => Self::Regulation,

            // Other/Custom
            "other" => Self::Other,
            custom => Self::Custom(custom.to_string()),
        }
    }
}

/// Relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: String,
    pub confidence: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Compressed reference (pageIndex + position)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedRef {
    pub source_url: String,
    pub page_index: usize,
    pub token_position: usize,
}

/// Consolidation result for single entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedEntity {
    pub entity_id: String,
    pub text: String,
    pub entity_type: EntityType,
    pub confidence: f32,
    pub sources: Vec<String>,
    pub compressed_refs: Vec<CompressedRef>,
    pub merged_from: Vec<String>, // Entity IDs that were merged
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Neo4j relationship for graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRelationship {
    pub source_entity_id: String,
    pub target_entity_id: String,
    pub relationship_type: String,
    pub confidence: f32,
    pub properties: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_display() {
        assert_eq!(EntityType::Product.to_string(), "product");
        assert_eq!(EntityType::Coverage.to_string(), "coverage");
    }

    #[test]
    fn test_entity_creation() {
        let entity = Entity {
            entity_id: "ent_001".to_string(),
            text: "Critical Illness".to_string(),
            entity_type: EntityType::Product,
            confidence: 0.95,
            sources: vec!["prudential.co.th".to_string()],
            metadata: HashMap::new(),
        };
        assert_eq!(entity.entity_type, EntityType::Product);
    }
}
