use crate::types::{SearchError, SearchResult};
use reqwest::Client;
use serde_json::{json, Value};

/// Exa 搜尋客戶端（$10 免費額度，AI 語義搜尋）
pub struct ExaClient {
    client: Client,
    api_key: String,
}

impl ExaClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
        }
    }

    /// 執行搜尋
    pub async fn search(&self, query: &str, num_results: usize) -> Result<Vec<SearchResult>, SearchError> {
        let url = "https://api.exa.ai/search";

        let body = json!({
            "query": query,
            "type": "auto",
            "numResults": num_results,
            "contents": {
                "text": {
                    "maxCharacters": 1000
                }
            }
        });

        let response = self.client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SearchError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SearchError::ApiError(format!("Exa API 錯誤 {}: {}", status, error_text)));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| SearchError::ParseError(e.to_string()))?;

        let results = json["results"]
            .as_array()
            .ok_or_else(|| SearchError::ParseError("無法解析結果".to_string()))?
            .iter()
            .map(|r| SearchResult {
                title: r["title"].as_str().unwrap_or("無標題").to_string(),
                url: r["url"].as_str().unwrap_or("").to_string(),
                snippet: r["snippet"].as_str().map(|s| s.to_string()),
                content: r["text"].as_str().map(|s| s.to_string()),
            })
            .collect();

        Ok(results)
    }
}
