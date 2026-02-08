//! 置信度計算 - 評估搜尋結果的品質

use crate::types::SearchResult;

/// 置信度計算器配置
#[derive(Debug, Clone)]
pub struct ConfidenceConfig {
    /// 結果數量權重
    pub result_count_weight: f32,
    /// 標題相關性權重
    pub title_relevance_weight: f32,
    /// URL 權威性權重
    pub url_authority_weight: f32,
    /// 內容品質權重
    pub content_quality_weight: f32,
    /// 語義密度權重
    pub semantic_density_weight: f32,
}

impl Default for ConfidenceConfig {
    fn default() -> Self {
        Self {
            result_count_weight: 0.15,
            title_relevance_weight: 0.30,
            url_authority_weight: 0.20,
            content_quality_weight: 0.20,
            semantic_density_weight: 0.15,
        }
    }
}

/// 置信度計算器
pub struct ConfidenceCalculator {
    config: ConfidenceConfig,
    authority_domains: Vec<String>,
}

impl ConfidenceCalculator {
    /// 建立新的置信度計算器
    pub fn new() -> Self {
        Self {
            config: ConfidenceConfig::default(),
            authority_domains: vec![
                "github.com".to_string(),
                "stackoverflow.com".to_string(),
                "docs.rs".to_string(),
                "rust-lang.org".to_string(),
                "arxiv.org".to_string(),
                "wikipedia.org".to_string(),
                "cve.mitre.org".to_string(),
                "nvd.nist.gov".to_string(),
            ],
        }
    }

    /// 計算搜尋結果的置信度
    pub fn calculate(&self, query: &str, results: &[SearchResult]) -> f32 {
        if results.is_empty() {
            return 0.0;
        }

        let result_count_score = self.score_result_count(results.len());
        let title_relevance_score = self.score_title_relevance(query, results);
        let url_authority_score = self.score_url_authority(results);
        let content_quality_score = self.score_content_quality(results);
        let semantic_density_score = self.score_semantic_density(query, results);

        // 加權總分
        let total = result_count_score * self.config.result_count_weight
            + title_relevance_score * self.config.title_relevance_weight
            + url_authority_score * self.config.url_authority_weight
            + content_quality_score * self.config.content_quality_weight
            + semantic_density_score * self.config.semantic_density_weight;

        total.clamp(0.0, 1.0)
    }

    /// 評分：結果數量
    fn score_result_count(&self, count: usize) -> f32 {
        match count {
            0 => 0.0,
            1..=2 => 0.3,
            3..=5 => 0.6,
            6..=10 => 0.9,
            _ => 1.0,
        }
    }

    /// 評分：標題相關性
    fn score_title_relevance(&self, query: &str, results: &[SearchResult]) -> f32 {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();

        if query_words.is_empty() {
            return 0.5;
        }

        let mut total_score = 0.0;
        for result in results {
            let title_lower = result.title.to_lowercase();
            let matches = query_words
                .iter()
                .filter(|w| title_lower.contains(*w))
                .count();
            total_score += matches as f32 / query_words.len() as f32;
        }

        (total_score / results.len() as f32).min(1.0)
    }

    /// 評分：URL 權威性
    fn score_url_authority(&self, results: &[SearchResult]) -> f32 {
        let authority_count = results
            .iter()
            .filter(|r| {
                self.authority_domains
                    .iter()
                    .any(|d| r.url.contains(d))
            })
            .count();

        (authority_count as f32 / results.len() as f32).min(1.0)
    }

    /// 評分：內容品質
    fn score_content_quality(&self, results: &[SearchResult]) -> f32 {
        let mut total_score = 0.0;

        for result in results {
            let mut score = 0.0;

            // 有 snippet 加分
            if result.snippet.is_some() {
                score += 0.3;
            }

            // 有 content 加分
            if let Some(ref content) = result.content {
                score += 0.3;
                // 內容長度加分
                if content.len() > 500 {
                    score += 0.2;
                }
                if content.len() > 1000 {
                    score += 0.2;
                }
            }

            total_score += score;
        }

        (total_score / results.len() as f32).min(1.0)
    }

    /// 評分：語義密度
    /// semantic_density = (relevant_keywords / total_words) * diversity_factor
    fn score_semantic_density(&self, query: &str, results: &[SearchResult]) -> f32 {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();

        if query_words.is_empty() {
            return 0.5;
        }

        let mut total_density = 0.0;

        for result in results {
            let text = format!(
                "{} {}",
                result.title,
                result.snippet.as_deref().unwrap_or("")
            ).to_lowercase();

            let words: Vec<&str> = text.split_whitespace().collect();
            if words.is_empty() {
                continue;
            }

            // 計算相關關鍵字數量
            let relevant_count = words
                .iter()
                .filter(|w| query_words.iter().any(|qw| w.contains(qw)))
                .count();

            // 計算多樣性因子
            let unique_words: std::collections::HashSet<&str> = words.iter().copied().collect();
            let diversity_factor = unique_words.len() as f32 / words.len() as f32;

            // 語義密度
            let density = (relevant_count as f32 / words.len() as f32) * diversity_factor;
            total_density += density;
        }

        (total_density / results.len() as f32 * 10.0).min(1.0)  // 放大係數
    }
}

impl Default for ConfidenceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_results() -> Vec<SearchResult> {
        vec![
            SearchResult {
                title: "Rust Security Best Practices".to_string(),
                url: "https://github.com/rust-lang/rust".to_string(),
                snippet: Some("Learn about Rust security features".to_string()),
                content: None,
            },
            SearchResult {
                title: "Rust Programming Language".to_string(),
                url: "https://rust-lang.org".to_string(),
                snippet: Some("A language empowering everyone".to_string()),
                content: Some("Rust is a systems programming language...".to_string()),
            },
        ]
    }

    #[test]
    fn test_confidence_calculation() {
        let calc = ConfidenceCalculator::new();
        let results = create_test_results();
        let confidence = calc.calculate("Rust security", &results);

        assert!(confidence > 0.0);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_empty_results() {
        let calc = ConfidenceCalculator::new();
        let confidence = calc.calculate("test", &[]);
        assert_eq!(confidence, 0.0);
    }

    #[test]
    fn test_result_count_scoring() {
        let calc = ConfidenceCalculator::new();
        assert_eq!(calc.score_result_count(0), 0.0);
        assert_eq!(calc.score_result_count(2), 0.3);
        assert_eq!(calc.score_result_count(5), 0.6);
        assert_eq!(calc.score_result_count(10), 0.9);
        assert_eq!(calc.score_result_count(20), 1.0);
    }
}
