//! HTTP client for Mimir interaction

use reqwest::{Client, StatusCode};
use serde_json::json;

use crate::{MimirIngestionRequest, OrchestrationError, QueryResult, Result};

pub struct MimirClient {
    client: Client,
    base_url: String,
    tenant_id: String,
    token: Option<String>,
}

impl MimirClient {
    pub fn new(base_url: String, tenant_id: String) -> Self {
        let token = std::env::var("MIMIR_TOKEN").ok();
        Self {
            client: Client::new(),
            base_url,
            tenant_id,
            token,
        }
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    /// Ingest RefGraph consolidated output into Mimir
    pub async fn ingest(&self, request: &MimirIngestionRequest) -> Result<String> {
        let url = format!(
            "{}/api/v1/tenants/{}/ingest/entities",
            self.base_url, self.tenant_id
        );

        let payload = json!({
            "domain": request.domain,
            "entity_count": request.entity_count,
            "relationship_count": request.relationship_count,
            "average_confidence": request.average_confidence,
            "entities": request.entities,
            "relationships": request.relationships,
        });

        let mut request = self.client.post(&url).json(&payload);

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| OrchestrationError::MimirError(format!("Request failed: {}", e)))?;

        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                response
                    .text()
                    .await
                    .map_err(|e| OrchestrationError::MimirError(format!("Response read failed: {}", e)))
            }
            StatusCode::UNAUTHORIZED => Err(OrchestrationError::MimirError(
                "Unauthorized: check Mimir API credentials".to_string(),
            )),
            StatusCode::NOT_FOUND => Err(OrchestrationError::MimirError(
                format!("Endpoint not found: {}", url),
            )),
            StatusCode::BAD_REQUEST => Err(OrchestrationError::MimirError(
                "Invalid request to Mimir".to_string(),
            )),
            _ => Err(OrchestrationError::MimirError(format!(
                "Unexpected status: {}",
                response.status()
            ))),
        }
    }

    /// Search Mimir and return top-k results
    pub async fn search(&self, query: &str, k: usize) -> Result<QueryResult> {
        let url = format!(
            "{}/api/v1/tenants/{}/search?q={}&k={}",
            self.base_url, self.tenant_id, query, k
        );

        let mut request = self.client.get(&url);

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| OrchestrationError::MimirError(format!("Search failed: {}", e)))?;

        match response.status() {
            StatusCode::OK => {
                let body: serde_json::Value = response
                    .json()
                    .await
                    .map_err(|e| OrchestrationError::MimirError(format!("Parse failed: {}", e)))?;

                let hits = body["results"]
                    .as_array()
                    .map(|a| a.len())
                    .unwrap_or(0);

                let score = body["score"].as_f64().unwrap_or(0.0) as f32;

                Ok(QueryResult {
                    query: query.to_string(),
                    hits,
                    score,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                })
            }
            StatusCode::NOT_FOUND => Ok(QueryResult {
                query: query.to_string(),
                hits: 0,
                score: 0.0,
                timestamp: chrono::Utc::now().to_rfc3339(),
            }),
            _ => Err(OrchestrationError::MimirError(format!(
                "Search failed with status {}",
                response.status()
            ))),
        }
    }

    /// Check if Mimir is healthy
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mimir_client_new() {
        let client = MimirClient::new(
            "http://localhost:5000".to_string(),
            "asgard_insurance".to_string(),
        );
        assert_eq!(client.base_url, "http://localhost:5000");
        assert_eq!(client.tenant_id, "asgard_insurance");
    }

    #[tokio::test]
    async fn test_mimir_client_health_check_no_server() {
        let client = MimirClient::new(
            "http://localhost:9999".to_string(), // non-existent
            "asgard_insurance".to_string(),
        );
        let health = client.health_check().await.unwrap();
        assert!(!health);
    }

    #[tokio::test]
    async fn test_mimir_client_constructs_correct_base_url() {
        let client = MimirClient::new(
            "http://localhost:5000".to_string(),
            "asgard_medical".to_string(),
        );

        assert_eq!(client.base_url, "http://localhost:5000");
    }

    #[tokio::test]
    async fn test_query_result_timestamp_format() {
        let client = MimirClient::new(
            "http://localhost:5000".to_string(),
            "asgard_insurance".to_string(),
        );

        let result = QueryResult {
            query: "test".to_string(),
            hits: 0,
            score: 0.0,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        assert!(!result.timestamp.is_empty());
        assert!(result.timestamp.contains('T')); // ISO format includes T
    }
}
