use crate::types::{SearchEngine, SearchError, SearchResult};
use crate::duckduckgo::DuckDuckGoClient;
use crate::exa::ExaClient;

/// 統一的搜尋客戶端，支援多個搜尋引擎
pub struct MultiSearchClient {
    duckduckgo: DuckDuckGoClient,
    exa: Option<ExaClient>,
}

impl MultiSearchClient {
    /// 建立新的多引擎搜尋客戶端
    pub fn new() -> Self {
        Self {
            duckduckgo: DuckDuckGoClient::new(),
            exa: None,
        }
    }

    /// 設定 Exa API 金鑰
    pub fn with_exa(mut self, api_key: &str) -> Self {
        self.exa = Some(ExaClient::new(api_key));
        self
    }

    /// 執行搜尋
    pub async fn search(
        &self,
        query: &str,
        engine: SearchEngine,
        num_results: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        match engine {
            SearchEngine::DuckDuckGo => {
                println!("🦆 使用 DuckDuckGo 搜尋（完全免費）...");
                self.duckduckgo.search(query, num_results).await
            }
            SearchEngine::Exa => {
                println!("🔍 使用 Exa 搜尋（AI 語義搜尋）...");
                match &self.exa {
                    Some(client) => client.search(query, num_results).await,
                    None => Err(SearchError::ApiError(
                        "Exa 客戶端未初始化，請使用 with_exa() 設定 API 金鑰".to_string()
                    )),
                }
            }
            SearchEngine::Tavily => {
                Err(SearchError::ApiError("Tavily 尚未實作".to_string()))
            }
        }
    }
}

impl Default for MultiSearchClient {
    fn default() -> Self {
        Self::new()
    }
}
