//! Mock Mimir client for testing without a real server

use crate::{OrchestrationError, QueryResult, Result};
use std::collections::HashMap;

/// Mock Mimir client that simulates search results
pub struct MockMimirClient {
    search_results: HashMap<String, usize>,
}

impl MockMimirClient {
    pub fn new() -> Self {
        Self {
            search_results: HashMap::new(),
        }
    }

    /// Set expected result for a query
    pub fn with_result(mut self, query: String, hits: usize) -> Self {
        self.search_results.insert(query, hits);
        self
    }

    /// Simulate a search query
    pub async fn search(&self, query: &str, _k: usize) -> Result<QueryResult> {
        let hits = self
            .search_results
            .get(query)
            .copied()
            .unwrap_or(0);

        Ok(QueryResult {
            query: query.to_string(),
            hits,
            score: if hits > 0 { 0.85 } else { 0.0 },
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}

impl Default for MockMimirClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_client_returns_expected_hits() {
        let mock = MockMimirClient::new()
            .with_result("insurance".to_string(), 5)
            .with_result("health".to_string(), 0);

        let result = mock.search("insurance", 3).await.unwrap();
        assert_eq!(result.hits, 5);

        let result = mock.search("health", 3).await.unwrap();
        assert_eq!(result.hits, 0);
    }

    #[tokio::test]
    async fn test_mock_client_default_zero_hits() {
        let mock = MockMimirClient::new();

        let result = mock.search("unknown", 3).await.unwrap();
        assert_eq!(result.hits, 0);
    }

    #[tokio::test]
    async fn test_mock_client_maintains_query_text() {
        let mock = MockMimirClient::new()
            .with_result("test query".to_string(), 3);

        let result = mock.search("test query", 3).await.unwrap();
        assert_eq!(result.query, "test query");
    }
}
