use crate::types::{SearchError, SearchResult};
use reqwest::Client;
use serde_json::{json, Value};

/// Tavily 搜尋客戶端（深度內容提取）
pub struct TavilyClient {
    client: Client,
    api_key: String,
}

impl TavilyClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
        }
    }

    /// 執行搜尋
    pub async fn search(&self, query: &str, num_results: usize) -> Result<Vec<SearchResult>, SearchError> {
        let url = "https://api.tavily.com/search";

        let body = json!({
            "api_key": self.api_key,
            "query": query,
            "search_depth": "advanced",
            "max_results": num_results,
            "include_answer": true,
            "include_raw_content": true,
        });

        let response = self.client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| SearchError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SearchError::ApiError(format!("Tavily API 錯誤 {}: {}", status, error_text)));
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
                snippet: r["content"].as_str().map(|s| s.to_string()),
                content: r["raw_content"].as_str().map(|s| s.to_string()),
            })
            .collect();

        Ok(results)
    }

    /// 提取指定 URL 的內容
    pub async fn extract_content(&self, urls: &[&str]) -> Result<Vec<SearchResult>, SearchError> {
        let url = "https://api.tavily.com/extract";

        let body = json!({
            "api_key": self.api_key,
            "urls": urls,
        });

        let response = self.client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| SearchError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SearchError::ApiError(format!("Tavily Extract API 錯誤 {}: {}", status, error_text)));
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
                snippet: None,
                content: r["raw_content"].as_str().map(|s| s.to_string()),
            })
            .collect();

        Ok(results)
    }
}
