//! 階梯式檢索 - 根據置信度自動升級搜尋引擎

use crate::duckduckgo::DuckDuckGoClient;
use crate::exa::ExaClient;
use crate::tavily::TavilyClient;
use crate::routing::confidence::ConfidenceCalculator;
use crate::types::{SearchResult, SearchError};

/// 階梯式檢索配置
#[derive(Debug, Clone)]
pub struct TieredConfig {
    /// L1 → L2 的置信度閾值
    pub l1_threshold: f32,
    /// L2 → L3 的置信度閾值
    pub l2_threshold: f32,
    /// 每層的最大結果數
    pub max_results_per_tier: usize,
}

impl Default for TieredConfig {
    fn default() -> Self {
        Self {
            l1_threshold: 0.80,  // DDG → Exa
            l2_threshold: 0.85,  // Exa → Tavily
            max_results_per_tier: 10,
        }
    }
}

/// 檢索層級
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalTier {
    L1,  // DuckDuckGo (免費)
    L2,  // Exa (付費，精準)
    L3,  // Tavily (付費，深度)
}

/// 階梯式檢索結果
#[derive(Debug)]
pub struct TieredResult {
    pub results: Vec<SearchResult>,
    pub tier_used: RetrievalTier,
    pub confidence: f32,
    pub cost_estimate: f32,
}

/// 階梯式檢索引擎
pub struct TieredRetrieval {
    duckduckgo: DuckDuckGoClient,
    exa: Option<ExaClient>,
    tavily: Option<TavilyClient>,
    confidence_calc: ConfidenceCalculator,
    config: TieredConfig,
}

impl TieredRetrieval {
    /// 建立新的階梯式檢索引擎
    pub fn new(config: TieredConfig) -> Self {
        Self {
            duckduckgo: DuckDuckGoClient::new(),
            exa: None,
            tavily: None,
            confidence_calc: ConfidenceCalculator::new(),
            config,
        }
    }

    /// 使用預設配置建立
    pub fn with_defaults() -> Self {
        Self::new(TieredConfig::default())
    }

    /// 設定 Exa 客戶端
    pub fn with_exa(mut self, api_key: &str) -> Self {
        self.exa = Some(ExaClient::new(api_key));
        self
    }

    /// 設定 Tavily 客戶端
    pub fn with_tavily(mut self, api_key: &str) -> Self {
        self.tavily = Some(TavilyClient::new(api_key));
        self
    }

    /// 執行階梯式檢索
    pub async fn search(&self, query: &str) -> Result<TieredResult, SearchError> {
        // L1: DuckDuckGo (免費)
        log::info!("🔍 L1: 使用 DuckDuckGo 搜尋...");
        let l1_results = self.duckduckgo
            .search(query, self.config.max_results_per_tier)
            .await?;

        let l1_confidence = self.confidence_calc.calculate(query, &l1_results);
        log::info!("📊 L1 置信度: {:.2}", l1_confidence);

        if l1_confidence >= self.config.l1_threshold {
            return Ok(TieredResult {
                results: l1_results,
                tier_used: RetrievalTier::L1,
                confidence: l1_confidence,
                cost_estimate: 0.0,  // 免費
            });
        }

        // L2: Exa (付費，精準語義搜尋)
        if let Some(ref exa) = self.exa {
            log::info!("🔍 L2: 使用 Exa 搜尋...");

            // 使用 L1 結果提取關鍵字優化查詢
            let refined_query = self.refine_query(query, &l1_results);
            let l2_results = exa
                .search(&refined_query, self.config.max_results_per_tier)
                .await?;

            let l2_confidence = self.confidence_calc.calculate(query, &l2_results);
            log::info!("📊 L2 置信度: {:.2}", l2_confidence);

            if l2_confidence >= self.config.l2_threshold {
                return Ok(TieredResult {
                    results: l2_results,
                    tier_used: RetrievalTier::L2,
                    confidence: l2_confidence,
                    cost_estimate: 0.005,  // ~$0.005/次
                });
            }

            // L3: Tavily (付費，深度內容提取)
            if let Some(ref tavily) = self.tavily {
                log::info!("🔍 L3: 使用 Tavily 深度提取...");

                // 只對最相關的 URL 進行深度提取
                let top_urls: Vec<&str> = l2_results
                    .iter()
                    .take(3)
                    .map(|r| r.url.as_str())
                    .collect();

                let l3_results = tavily
                    .extract_content(&top_urls)
                    .await?;

                let l3_confidence = self.confidence_calc.calculate(query, &l3_results);
                log::info!("📊 L3 置信度: {:.2}", l3_confidence);

                return Ok(TieredResult {
                    results: l3_results,
                    tier_used: RetrievalTier::L3,
                    confidence: l3_confidence,
                    cost_estimate: 0.015,  // ~$0.015/次
                });
            }

            // 沒有 Tavily，返回 L2 結果
            return Ok(TieredResult {
                results: l2_results,
                tier_used: RetrievalTier::L2,
                confidence: l2_confidence,
                cost_estimate: 0.005,
            });
        }

        // 沒有 Exa，返回 L1 結果
        Ok(TieredResult {
            results: l1_results,
            tier_used: RetrievalTier::L1,
            confidence: l1_confidence,
            cost_estimate: 0.0,
        })
    }

    /// 使用 L1 結果優化查詢
    fn refine_query(&self, original: &str, l1_results: &[SearchResult]) -> String {
        // 從 L1 結果提取關鍵字
        let keywords: Vec<&str> = l1_results
            .iter()
            .filter_map(|r| r.snippet.as_deref())
            .flat_map(|s| s.split_whitespace())
            .filter(|w| w.len() > 3)
            .take(5)
            .collect();

        if keywords.is_empty() {
            original.to_string()
        } else {
            format!("{} {}", original, keywords.join(" "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiered_config_default() {
        let config = TieredConfig::default();
        assert_eq!(config.l1_threshold, 0.80);
        assert_eq!(config.l2_threshold, 0.85);
        assert_eq!(config.max_results_per_tier, 10);
    }

    #[test]
    fn test_retrieval_tier_equality() {
        assert_eq!(RetrievalTier::L1, RetrievalTier::L1);
        assert_ne!(RetrievalTier::L1, RetrievalTier::L2);
    }

    #[test]
    fn test_refine_query_empty_results() {
        let retrieval = TieredRetrieval::with_defaults();
        let refined = retrieval.refine_query("test query", &[]);
        assert_eq!(refined, "test query");
    }

    #[test]
    fn test_refine_query_with_results() {
        let retrieval = TieredRetrieval::with_defaults();
        let results = vec![
            SearchResult {
                title: "Test".to_string(),
                url: "https://example.com".to_string(),
                snippet: Some("Rust programming language security".to_string()),
                content: None,
            },
        ];
        let refined = retrieval.refine_query("Rust", &results);
        assert!(refined.contains("Rust"));
        assert!(refined.len() > "Rust".len());
    }
}
