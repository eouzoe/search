use serde::{Deserialize, Serialize};

/// 搜尋結果的統一格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub content: Option<String>,
}

/// 搜尋引擎類型
#[derive(Debug, Clone, Copy)]
pub enum SearchEngine {
    DuckDuckGo,  // 完全免費
    Tavily,      // 1000次/月免費
    Exa,         // $10 免費額度
}

/// 搜尋錯誤類型
#[derive(Debug)]
pub enum SearchError {
    NetworkError(String),
    ApiError(String),
    ParseError(String),
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::NetworkError(msg) => write!(f, "網路錯誤: {}", msg),
            SearchError::ApiError(msg) => write!(f, "API 錯誤: {}", msg),
            SearchError::ParseError(msg) => write!(f, "解析錯誤: {}", msg),
        }
    }
}

impl std::error::Error for SearchError {}
