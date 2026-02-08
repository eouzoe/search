//! 語義路由器 - 根據查詢複雜度選擇最適合的處理策略

/// 任務複雜度分類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskComplexity {
    /// 簡單查詢：事實查詢、定義查詢
    /// 例如："Rust 是什麼？"、"Bose 產品價格"
    Simple,

    /// 中等查詢：比較分析、多步驟查詢
    /// 例如："比較 Rust 和 Go 的效能"
    Medium,

    /// 複雜查詢：深度分析、多步驟推理
    /// 例如："分析 Bose 藍牙協議的安全性並提出改進方案"
    Complex,
}

/// 語義路由器配置
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// 簡單查詢的最大長度（字元數）
    pub simple_max_length: usize,
    /// 複雜查詢的關鍵字
    pub complex_keywords: Vec<String>,
    /// 是否啟用語義分析
    pub enable_semantic_analysis: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            simple_max_length: 50,
            complex_keywords: vec![
                "分析".to_string(),
                "比較".to_string(),
                "為什麼".to_string(),
                "如何".to_string(),
                "evaluate".to_string(),
                "analyze".to_string(),
                "compare".to_string(),
            ],
            enable_semantic_analysis: true,
        }
    }
}

/// 語義路由器
pub struct SemanticRouter {
    config: RouterConfig,
}

impl SemanticRouter {
    /// 建立新的語義路由器
    pub fn new(config: RouterConfig) -> Self {
        Self { config }
    }

    /// 使用預設配置建立路由器
    pub fn with_defaults() -> Self {
        Self::new(RouterConfig::default())
    }

    /// 分類查詢複雜度
    pub fn classify(&self, query: &str) -> TaskComplexity {
        let query_lower = query.to_lowercase();
        let query_len = query.chars().count();

        // 檢查複雜查詢關鍵字
        let has_complex_keyword = self.config.complex_keywords
            .iter()
            .any(|kw| query_lower.contains(&kw.to_lowercase()));

        // 檢查多個子句（使用標點符號判斷）
        let clause_count = query.matches([',', '，', '、']).count() + 1;

        // 檢查問號數量（多個問題 = 複雜）
        let question_count = query.matches(['?', '？']).count();

        // 分類邏輯
        if question_count > 1 || clause_count > 2 ||
           (has_complex_keyword && query_len > 100) {
            TaskComplexity::Complex
        } else if has_complex_keyword || query_len > self.config.simple_max_length {
            TaskComplexity::Medium
        } else {
            TaskComplexity::Simple
        }
    }

    /// 根據複雜度選擇 LLM 模型
    pub fn select_model(&self, complexity: TaskComplexity) -> &'static str {
        match complexity {
            TaskComplexity::Simple => "claude-haiku-4-5",
            TaskComplexity::Medium => "claude-sonnet-4-5",
            TaskComplexity::Complex => "claude-opus-4-5",
        }
    }

    /// 根據複雜度選擇搜尋策略
    pub fn select_search_strategy(&self, complexity: TaskComplexity) -> SearchStrategy {
        match complexity {
            TaskComplexity::Simple => SearchStrategy::SingleEngine,
            TaskComplexity::Medium => SearchStrategy::TieredRetrieval,
            TaskComplexity::Complex => SearchStrategy::DeepResearch,
        }
    }
}

/// 搜尋策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchStrategy {
    /// 單引擎搜尋（DuckDuckGo）
    SingleEngine,
    /// 階梯式檢索（DDG → Exa → Tavily）
    TieredRetrieval,
    /// 深度研究（所有引擎 + 內容提取）
    DeepResearch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_query() {
        let router = SemanticRouter::with_defaults();
        assert_eq!(
            router.classify("Rust 是什麼？"),
            TaskComplexity::Simple
        );
    }

    #[test]
    fn test_medium_query() {
        let router = SemanticRouter::with_defaults();
        assert_eq!(
            router.classify("比較 Rust 和 Go 的效能差異"),
            TaskComplexity::Medium
        );
    }

    #[test]
    fn test_complex_query() {
        let router = SemanticRouter::with_defaults();
        assert_eq!(
            router.classify("分析 Bose 藍牙協議的安全性，找出潛在漏洞，並提出改進方案？"),
            TaskComplexity::Complex
        );
    }

    #[test]
    fn test_model_selection() {
        let router = SemanticRouter::with_defaults();
        assert_eq!(router.select_model(TaskComplexity::Simple), "claude-haiku-4-5");
        assert_eq!(router.select_model(TaskComplexity::Medium), "claude-sonnet-4-5");
        assert_eq!(router.select_model(TaskComplexity::Complex), "claude-opus-4-5");
    }

    #[test]
    fn test_search_strategy_selection() {
        let router = SemanticRouter::with_defaults();
        assert_eq!(
            router.select_search_strategy(TaskComplexity::Simple),
            SearchStrategy::SingleEngine
        );
        assert_eq!(
            router.select_search_strategy(TaskComplexity::Medium),
            SearchStrategy::TieredRetrieval
        );
        assert_eq!(
            router.select_search_strategy(TaskComplexity::Complex),
            SearchStrategy::DeepResearch
        );
    }
}
