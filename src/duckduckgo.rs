use crate::types::{SearchError, SearchResult};
use reqwest::Client;
use serde_json::Value;

/// DuckDuckGo 搜尋客戶端（完全免費，無需 API 金鑰）
pub struct DuckDuckGoClient {
    client: Client,
}

impl DuckDuckGoClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// 執行搜尋
    pub async fn search(&self, query: &str, num_results: usize) -> Result<Vec<SearchResult>, SearchError> {
        // DuckDuckGo Instant Answer API
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1",
            urlencoding::encode(query)
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| SearchError::NetworkError(e.to_string()))?;

        let json: Value = response
            .json()
            .await
            .map_err(|e| SearchError::ParseError(e.to_string()))?;

        let mut results = Vec::new();

        // 處理 Abstract（摘要）
        if let Some(abstract_text) = json["Abstract"].as_str() {
            if !abstract_text.is_empty() {
                results.push(SearchResult {
                    title: json["Heading"].as_str().unwrap_or("DuckDuckGo Result").to_string(),
                    url: json["AbstractURL"].as_str().unwrap_or("").to_string(),
                    snippet: Some(abstract_text.to_string()),
                    content: None,
                });
            }
        }

        // 處理 RelatedTopics（相關主題）
        if let Some(topics) = json["RelatedTopics"].as_array() {
            for topic in topics.iter().take(num_results.saturating_sub(results.len())) {
                if let Some(text) = topic["Text"].as_str() {
                    results.push(SearchResult {
                        title: text.split(" - ").next().unwrap_or(text).to_string(),
                        url: topic["FirstURL"].as_str().unwrap_or("").to_string(),
                        snippet: Some(text.to_string()),
                        content: None,
                    });
                }
            }
        }

        Ok(results)
    }
}

impl Default for DuckDuckGoClient {
    fn default() -> Self {
        Self::new()
    }
}
