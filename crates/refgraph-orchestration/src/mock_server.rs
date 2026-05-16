//! Sophisticated Mock Mimir Server for Integration Testing
//! Simulates realistic Mimir responses for complete pipeline testing

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock Mimir server state tracking ingestions and search results
#[derive(Debug, Clone)]
pub struct MockMimirServer {
    state: Arc<Mutex<ServerState>>,
}

#[derive(Debug, Clone)]
struct ServerState {
    ingested_data: Vec<IngestedDocument>,
    search_index: HashMap<String, Vec<SearchResult>>,
    health_status: bool,
}

#[derive(Debug, Clone)]
pub struct IngestedDocument {
    pub tenant_id: String,
    pub domain: String,
    pub entity_count: usize,
    pub relationship_count: usize,
    pub average_confidence: f32,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub entity_id: String,
    pub text: String,
    pub confidence: f32,
}

impl MockMimirServer {
    /// Create new mock server
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ServerState {
                ingested_data: Vec::new(),
                search_index: create_default_index(),
                health_status: true,
            })),
        }
    }

    /// Record an ingestion
    pub fn record_ingest(
        &self,
        tenant_id: String,
        domain: String,
        entity_count: usize,
        relationship_count: usize,
        average_confidence: f32,
    ) {
        if let Ok(mut state) = self.state.lock() {
            state.ingested_data.push(IngestedDocument {
                tenant_id,
                domain,
                entity_count,
                relationship_count,
                average_confidence,
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    /// Get total documents ingested
    pub fn total_ingested(&self) -> usize {
        self.state
            .lock()
            .map(|s| s.ingested_data.len())
            .unwrap_or(0)
    }

    /// Get total entities across all ingestions
    pub fn total_entities(&self) -> usize {
        self.state
            .lock()
            .map(|s| s.ingested_data.iter().map(|d| d.entity_count).sum())
            .unwrap_or(0)
    }

    /// Get total relationships across all ingestions
    pub fn total_relationships(&self) -> usize {
        self.state
            .lock()
            .map(|s| s.ingested_data.iter().map(|d| d.relationship_count).sum())
            .unwrap_or(0)
    }

    /// Search for query and return top-k results
    pub fn search(&self, query: &str, k: usize) -> Vec<SearchResult> {
        if let Ok(state) = self.state.lock() {
            state
                .search_index
                .get(query)
                .map(|results| results.iter().take(k).cloned().collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    /// Set custom search results for a query
    pub fn set_search_results(&self, query: String, results: Vec<SearchResult>) {
        if let Ok(mut state) = self.state.lock() {
            state.search_index.insert(query, results);
        }
    }

    /// Set health status
    pub fn set_health(&self, healthy: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.health_status = healthy;
        }
    }

    /// Get health status
    pub fn is_healthy(&self) -> bool {
        self.state
            .lock()
            .map(|s| s.health_status)
            .unwrap_or(false)
    }

    /// Get all ingested documents
    pub fn ingested_documents(&self) -> Vec<IngestedDocument> {
        self.state
            .lock()
            .map(|s| s.ingested_data.clone())
            .unwrap_or_default()
    }

    /// Hit rate for a set of queries
    pub fn compute_hit_rate(&self, queries: &[String]) -> f32 {
        let hits = queries
            .iter()
            .filter(|q| !self.search(q, 1).is_empty())
            .count();

        if queries.is_empty() {
            0.0
        } else {
            hits as f32 / queries.len() as f32
        }
    }

    /// Simulate poor hit rate scenario
    pub fn set_poor_performance(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.search_index.clear();
            // Only 40% of standard queries will have results
            state.search_index.insert(
                "Critical Illness insurance".to_string(),
                vec![SearchResult {
                    entity_id: "ent_001".to_string(),
                    text: "Critical Illness Insurance".to_string(),
                    confidence: 0.85,
                }],
            );
            state.search_index.insert(
                "Health Insurance plan".to_string(),
                vec![SearchResult {
                    entity_id: "ent_002".to_string(),
                    text: "Health Insurance Plan".to_string(),
                    confidence: 0.88,
                }],
            );
            state.search_index.insert(
                "Accident coverage".to_string(),
                vec![SearchResult {
                    entity_id: "ent_003".to_string(),
                    text: "Accident Coverage".to_string(),
                    confidence: 0.82,
                }],
            );
            state.search_index.insert(
                "Death benefit".to_string(),
                vec![SearchResult {
                    entity_id: "ent_004".to_string(),
                    text: "Death Benefit".to_string(),
                    confidence: 0.80,
                }],
            );
        }
    }

    /// Simulate excellent hit rate scenario
    pub fn set_excellent_performance(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.search_index = create_default_index();
        }
    }
}

impl Default for MockMimirServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Create default search index with standard insurance queries
fn create_default_index() -> HashMap<String, Vec<SearchResult>> {
    let mut index = HashMap::new();

    // English queries - all return good results
    index.insert(
        "Critical Illness insurance".to_string(),
        vec![
            SearchResult {
                entity_id: "ent_crit_001".to_string(),
                text: "Critical Illness Insurance".to_string(),
                confidence: 0.92,
            },
            SearchResult {
                entity_id: "ent_crit_002".to_string(),
                text: "Critical Illness Coverage".to_string(),
                confidence: 0.88,
            },
            SearchResult {
                entity_id: "ent_crit_003".to_string(),
                text: "Illness Protection Plan".to_string(),
                confidence: 0.85,
            },
        ],
    );

    index.insert(
        "Health Insurance plan".to_string(),
        vec![
            SearchResult {
                entity_id: "ent_health_001".to_string(),
                text: "Health Insurance Plan".to_string(),
                confidence: 0.94,
            },
            SearchResult {
                entity_id: "ent_health_002".to_string(),
                text: "Health Coverage".to_string(),
                confidence: 0.91,
            },
            SearchResult {
                entity_id: "ent_health_003".to_string(),
                text: "Medical Insurance".to_string(),
                confidence: 0.89,
            },
        ],
    );

    index.insert(
        "Accident coverage".to_string(),
        vec![
            SearchResult {
                entity_id: "ent_acc_001".to_string(),
                text: "Accident Coverage".to_string(),
                confidence: 0.90,
            },
            SearchResult {
                entity_id: "ent_acc_002".to_string(),
                text: "Accidental Injury Protection".to_string(),
                confidence: 0.87,
            },
            SearchResult {
                entity_id: "ent_acc_003".to_string(),
                text: "Accident Insurance".to_string(),
                confidence: 0.86,
            },
        ],
    );

    index.insert(
        "Death benefit".to_string(),
        vec![
            SearchResult {
                entity_id: "ent_death_001".to_string(),
                text: "Death Benefit".to_string(),
                confidence: 0.93,
            },
            SearchResult {
                entity_id: "ent_death_002".to_string(),
                text: "Mortality Benefit".to_string(),
                confidence: 0.89,
            },
            SearchResult {
                entity_id: "ent_death_003".to_string(),
                text: "Survivor Benefit".to_string(),
                confidence: 0.87,
            },
        ],
    );

    index.insert(
        "Hospital admission".to_string(),
        vec![
            SearchResult {
                entity_id: "ent_hosp_001".to_string(),
                text: "Hospital Admission".to_string(),
                confidence: 0.91,
            },
            SearchResult {
                entity_id: "ent_hosp_002".to_string(),
                text: "Hospitalization Coverage".to_string(),
                confidence: 0.88,
            },
            SearchResult {
                entity_id: "ent_hosp_003".to_string(),
                text: "Hospital Room Charges".to_string(),
                confidence: 0.85,
            },
        ],
    );

    index.insert(
        "Exclusions list".to_string(),
        vec![
            SearchResult {
                entity_id: "ent_excl_001".to_string(),
                text: "Exclusions List".to_string(),
                confidence: 0.89,
            },
            SearchResult {
                entity_id: "ent_excl_002".to_string(),
                text: "Policy Exclusions".to_string(),
                confidence: 0.86,
            },
            SearchResult {
                entity_id: "ent_excl_003".to_string(),
                text: "Excluded Conditions".to_string(),
                confidence: 0.84,
            },
        ],
    );

    index.insert(
        "Premium payment".to_string(),
        vec![
            SearchResult {
                entity_id: "ent_prem_001".to_string(),
                text: "Premium Payment".to_string(),
                confidence: 0.92,
            },
            SearchResult {
                entity_id: "ent_prem_002".to_string(),
                text: "Monthly Premium".to_string(),
                confidence: 0.90,
            },
            SearchResult {
                entity_id: "ent_prem_003".to_string(),
                text: "Premium Due".to_string(),
                confidence: 0.87,
            },
        ],
    );

    index.insert(
        "Claim process".to_string(),
        vec![
            SearchResult {
                entity_id: "ent_claim_001".to_string(),
                text: "Claim Process".to_string(),
                confidence: 0.88,
            },
            SearchResult {
                entity_id: "ent_claim_002".to_string(),
                text: "Filing a Claim".to_string(),
                confidence: 0.85,
            },
            SearchResult {
                entity_id: "ent_claim_003".to_string(),
                text: "Claims Procedure".to_string(),
                confidence: 0.83,
            },
        ],
    );

    // Thai queries - good results
    index.insert(
        "ประกันสุขภาพ".to_string(),
        vec![
            SearchResult {
                entity_id: "ent_thai_001".to_string(),
                text: "ประกันสุขภาพ (Health Insurance)".to_string(),
                confidence: 0.92,
            },
            SearchResult {
                entity_id: "ent_thai_002".to_string(),
                text: "การประกันสุขภาพ".to_string(),
                confidence: 0.89,
            },
            SearchResult {
                entity_id: "ent_thai_003".to_string(),
                text: "ประกันการรักษาพยาบาล".to_string(),
                confidence: 0.87,
            },
        ],
    );

    index.insert(
        "ประกันชีวิต".to_string(),
        vec![
            SearchResult {
                entity_id: "ent_thai_life_001".to_string(),
                text: "ประกันชีวิต (Life Insurance)".to_string(),
                confidence: 0.91,
            },
            SearchResult {
                entity_id: "ent_thai_life_002".to_string(),
                text: "ประกันชีวิตและสุขภาพ".to_string(),
                confidence: 0.88,
            },
            SearchResult {
                entity_id: "ent_thai_life_003".to_string(),
                text: "ประกันสำหรับครอบครัว".to_string(),
                confidence: 0.86,
            },
        ],
    );

    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_server_new() {
        let server = MockMimirServer::new();
        assert!(server.is_healthy());
        assert_eq!(server.total_ingested(), 0);
    }

    #[test]
    fn test_mock_server_record_ingest() {
        let server = MockMimirServer::new();
        server.record_ingest(
            "asgard_insurance".to_string(),
            "insurance".to_string(),
            25,
            15,
            0.876,
        );

        assert_eq!(server.total_ingested(), 1);
        assert_eq!(server.total_entities(), 25);
        assert_eq!(server.total_relationships(), 15);
    }

    #[test]
    fn test_mock_server_search_returns_results() {
        let server = MockMimirServer::new();
        let results = server.search("Critical Illness insurance", 3);

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].entity_id, "ent_crit_001");
        assert!((results[0].confidence - 0.92).abs() < 0.001);
    }

    #[test]
    fn test_mock_server_search_respects_k() {
        let server = MockMimirServer::new();
        let results_k1 = server.search("Health Insurance plan", 1);
        let results_k3 = server.search("Health Insurance plan", 3);

        assert_eq!(results_k1.len(), 1);
        assert_eq!(results_k3.len(), 3);
    }

    #[test]
    fn test_mock_server_compute_hit_rate_all_queries() {
        let server = MockMimirServer::new();
        let queries = vec![
            "Critical Illness insurance".to_string(),
            "Health Insurance plan".to_string(),
            "Death benefit".to_string(),
        ];

        let hit_rate = server.compute_hit_rate(&queries);
        assert_eq!(hit_rate, 1.0); // All 3 queries should have results
    }

    #[test]
    fn test_mock_server_set_poor_performance() {
        let server = MockMimirServer::new();
        server.set_poor_performance();

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

        let hit_rate = server.compute_hit_rate(&queries);
        assert!(hit_rate < 0.5); // Should be poor (around 40%)
    }

    #[test]
    fn test_mock_server_set_health_status() {
        let server = MockMimirServer::new();
        assert!(server.is_healthy());

        server.set_health(false);
        assert!(!server.is_healthy());

        server.set_health(true);
        assert!(server.is_healthy());
    }

    #[test]
    fn test_mock_server_multiple_ingestions() {
        let server = MockMimirServer::new();

        server.record_ingest(
            "asgard_insurance".to_string(),
            "insurance".to_string(),
            25,
            15,
            0.876,
        );
        server.record_ingest(
            "asgard_insurance".to_string(),
            "insurance".to_string(),
            30,
            20,
            0.890,
        );

        assert_eq!(server.total_ingested(), 2);
        assert_eq!(server.total_entities(), 55);
        assert_eq!(server.total_relationships(), 35);
    }

    #[test]
    fn test_mock_server_thai_queries() {
        let server = MockMimirServer::new();

        let thai_results = server.search("ประกันสุขภาพ", 3);
        assert_eq!(thai_results.len(), 3);

        let thai_life_results = server.search("ประกันชีวิต", 3);
        assert_eq!(thai_life_results.len(), 3);
    }

    #[test]
    fn test_mock_server_ingested_documents() {
        let server = MockMimirServer::new();
        server.record_ingest(
            "asgard_insurance".to_string(),
            "insurance".to_string(),
            25,
            15,
            0.876,
        );

        let docs = server.ingested_documents();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].entity_count, 25);
    }

    #[test]
    fn test_mock_server_custom_search_results() {
        let server = MockMimirServer::new();

        let custom_results = vec![SearchResult {
            entity_id: "custom_001".to_string(),
            text: "Custom Result".to_string(),
            confidence: 0.99,
        }];

        server.set_search_results("custom query".to_string(), custom_results);

        let results = server.search("custom query", 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity_id, "custom_001");
        assert!((results[0].confidence - 0.99).abs() < 0.001);
    }
}
