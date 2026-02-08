# CLAUDE.md

## Important Rules

### 安全與倫理
- **僅用於教育和防禦目的**：所有研究和工具僅用於安全研究和防禦性分析
- **禁止未授權使用**：不得用於未經授權的系統或網路
- **負責任披露**：遵守負責任的漏洞披露原則
- **API 金鑰保護**：絕不將 API 金鑰提交到版本控制系統

### 版本控制策略
- **語義化版本**：使用 `major.minor.patch` 格式
  - `patch`：Bug 修復、內部改進、文檔更新
  - `minor`：新功能、增強功能
  - `major`：破壞性變更、重大架構調整
- **變更記錄**：所有重要變更必須記錄在 CHANGELOG.md

### 開發原則
- **測試優先**：所有新功能必須包含測試
- **文檔同步**：代碼變更必須同步更新文檔
- **效能意識**：關注記憶體使用和執行效率
- **安全第一**：所有輸入必須驗證，避免注入攻擊

---

## 專案概述

### 目的

Bose 安全研究專案是一個基於 Rust 的高效能多引擎搜尋系統，專為安全研究和資訊收集設計。透過整合多個搜尋引擎（DuckDuckGo、Exa、Tavily）並採用智慧路由策略，實現成本優化和效能最大化。

### 目標

1. **成本優化**：透過階梯式檢索和智慧路由，降低 70-85% 的 API 成本
2. **效能提升**：使用零拷貝序列化、io_uring 等技術，降低 40-60% 延遲
3. **智慧路由**：根據查詢複雜度自動選擇最適合的搜尋引擎和 LLM 模型
4. **可擴展性**：模組化設計，易於新增更多搜尋引擎和功能

### 核心功能

- **多引擎支援**：整合 DuckDuckGo（免費）、Exa（AI 語義搜尋）、Tavily（深度內容提取）
- **階梯式檢索**：根據置信度自動升級搜尋引擎，平衡成本與品質
- **語義路由**：智慧分類查詢複雜度，選擇最適合的 LLM 模型
- **上下文優化**：自動裁剪和優化搜尋結果，減少 60-80% Token 消耗
- **零拷貝序列化**：使用 rkyv 實現高效能資料處理
- **併發控制**：連線池和信號量機制，支援 100+ QPS

---

## 快速開始

### 系統需求

- **作業系統**：Linux (推薦 5.1+，支援 io_uring)、macOS、Windows
- **Rust**：1.70+ (推薦使用 rustup 安裝)
- **記憶體**：最低 2GB，推薦 4GB+
- **網路**：穩定的網際網路連線

### 安裝

1. **克隆專案**：
```bash
git clone https://github.com/your-org/bose-security-research.git
cd bose-security-research
```

2. **安裝依賴**：
```bash
cargo build --release
```

3. **配置 API 金鑰**：

建立 `.env` 檔案：
```env
# Exa API 金鑰（可選，有 $10 免費額度）
EXA_API_KEY=your_exa_api_key_here

# Tavily API 金鑰（可選，用於深度內容提取）
TAVILY_API_KEY=your_tavily_api_key_here
```

### 基本使用

#### 使用 DuckDuckGo（完全免費）

```bash
cargo run --release -- --query "rust security vulnerabilities" --engine duckduckgo --num 10
```

#### 使用 Exa（AI 語義搜尋）

```bash
cargo run --release -- --query "Bose product security research" --engine exa --num 10
```

#### 使用簡短參數

```bash
cargo run --release -- -q "your query" -e duckduckgo -n 5
```

### 驗證安裝

執行測試確保一切正常：

```bash
# 執行所有測試
cargo test

# 執行特定測試
cargo test --test integration_tests
```

---

## 架構說明

### 系統架構

```
┌─────────────────────────────────────────────────────────────┐
│                     使用者查詢 (User Query)                    │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              語義路由層 (Semantic Router)                      │
│  - 任務分類 (簡單/複雜)                                         │
│  - 模型選擇 (Haiku 4.5 / Sonnet 4.5 / Opus 4.5)              │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│           階梯式檢索引擎 (Tiered Retrieval Engine)             │
│                                                               │
│  L1: DuckDuckGo (免費)                                        │
│      ├─ 置信度 ≥ 0.85 → 直接返回                              │
│      └─ 置信度 < 0.85 → 進入 L2                               │
│                                                               │
│  L2: Exa (付費，精準語義)                                      │
│      ├─ 使用 L1 關鍵字精準查詢                                 │
│      ├─ 置信度 ≥ 0.85 → 返回                                  │
│      └─ 置信度 < 0.85 → 進入 L3                               │
│                                                               │
│  L3: Tavily Extract (付費，深度提取)                           │
│      └─ 僅對最相關 URL 進行內容提取                            │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│            上下文處理層 (Context Processing)                   │
│  - HTML 雜訊移除                                              │
│  - 重複段落去除                                                │
│  - 語義塊提取                                                  │
│  - Token 預算分配                                             │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              LLM 推論層 (Inference Layer)                     │
│  - Haiku 4.5: 簡單摘要、事實查詢                              │
│  - Sonnet 4.5: 中等複雜度推理                                 │
│  - Opus 4.5: 複雜推理、多步驟分析                             │
└─────────────────────────────────────────────────────────────┘
```

### 模組說明

#### 1. 搜尋引擎模組 (`src/engines/`)

- **duckduckgo.rs**：DuckDuckGo 搜尋引擎實作，完全免費
- **exa.rs**：Exa AI 語義搜尋引擎，提供高品質結果
- **tavily.rs**：Tavily 深度內容提取引擎

#### 2. 路由模組 (`src/router/`)

- **semantic_router.rs**：語義路由器，分類查詢複雜度
- **tiered_retrieval.rs**：階梯式檢索系統，智慧選擇搜尋引擎

#### 3. 處理模組 (`src/processing/`)

- **context_pruner.rs**：上下文裁剪器，優化 Token 使用
- **confidence_calculator.rs**：置信度計算器，評估結果品質

#### 4. 客戶端模組 (`src/client.rs`)

- 統一的搜尋客戶端介面
- 連線池管理
- 併發控制

### 資料流程

1. **查詢接收**：使用者輸入查詢字串
2. **語義分析**：路由器分析查詢複雜度
3. **引擎選擇**：根據複雜度選擇起始搜尋引擎
4. **階梯式檢索**：
   - L1 (DuckDuckGo)：快速免費搜尋
   - L2 (Exa)：語義精準搜尋
   - L3 (Tavily)：深度內容提取
5. **置信度評估**：計算結果置信度，決定是否升級
6. **上下文處理**：裁剪和優化搜尋結果
7. **結果返回**：格式化並返回最終結果

---

## 開發規範

### 代碼風格

#### Rust 編碼規範

- **格式化**：使用 `rustfmt` 自動格式化
```bash
cargo fmt
```

- **Linting**：使用 `clippy` 檢查代碼品質
```bash
cargo clippy -- -D warnings
```

- **命名規範**：
  - 模組：`snake_case`
  - 結構體/枚舉：`PascalCase`
  - 函數/變數：`snake_case`
  - 常數：`SCREAMING_SNAKE_CASE`

#### 代碼組織

```rust
// 1. 外部 crate 導入
use std::collections::HashMap;
use tokio::sync::Semaphore;

// 2. 內部模組導入
use crate::engines::SearchEngine;
use crate::router::SemanticRouter;

// 3. 類型定義
pub struct SearchClient {
    // ...
}

// 4. 實作
impl SearchClient {
    // 公開方法在前
    pub fn new() -> Self {
        // ...
    }

    // 私有方法在後
    fn internal_method(&self) {
        // ...
    }
}
```

### 測試要求

#### 單元測試

每個模組必須包含單元測試：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_calculation() {
        let calculator = ConfidenceCalculator::new();
        let results = vec![/* ... */];
        let confidence = calculator.calculate(&results);
        assert!(confidence >= 0.0 && confidence <= 1.0);
    }
}
```

#### 整合測試

在 `tests/` 目錄下建立整合測試：

```rust
// tests/integration_tests.rs
use bose_security_research::SearchClient;

#[tokio::test]
async fn test_duckduckgo_search() {
    let client = SearchClient::new();
    let results = client.search("rust", "duckduckgo", 5).await;
    assert!(results.is_ok());
}
```

#### 測試覆蓋率目標

- **核心模組**：> 80%
- **公開 API**：> 90%
- **關鍵路徑**：100%

### 提交規範

#### Conventional Commits

使用標準化的提交訊息格式：

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

**類型 (type)**：
- `feat`：新功能
- `fix`：Bug 修復
- `docs`：文檔更新
- `style`：代碼格式（不影響功能）
- `refactor`：重構
- `perf`：效能優化
- `test`：測試相關
- `chore`：維護任務

**範例**：
```
feat(router): add semantic routing for query classification

Implemented semantic router that classifies queries into simple/medium/complex
categories and selects appropriate LLM models accordingly.

Closes #123
```

---

## API 配置

### Exa API

#### 取得 API 金鑰

1. 訪問 [Exa.ai](https://exa.ai)
2. 註冊帳號（提供 $10 免費額度）
3. 在控制台取得 API 金鑰

#### 配置

在 `.env` 檔案中設定：
```env
EXA_API_KEY=your_exa_api_key_here
```

#### 使用範例

```rust
use crate::engines::exa::ExaClient;

let client = ExaClient::new()?;
let results = client.search("rust security", 10).await?;
```

#### 成本估算

- **搜尋請求**：$0.001 - $0.005 / 次
- **免費額度**：$10（約 2,000 - 10,000 次搜尋）
- **月度預算**：建議 $20 - $50

### Tavily API

#### 取得 API 金鑰

1. 訪問 [Tavily.com](https://tavily.com)
2. 註冊開發者帳號
3. 取得 API 金鑰

#### 配置

在 `.env` 檔案中設定：
```env
TAVILY_API_KEY=your_tavily_api_key_here
```

#### 使用範例

```rust
use crate::engines::tavily::TavilyClient;

let client = TavilyClient::new()?;
let content = client.extract_content(&urls).await?;
```

#### 成本估算

- **內容提取**：$0.005 - $0.01 / 次
- **建議使用場景**：僅用於高價值查詢（L3 層級）

### DuckDuckGo API

#### 配置

無需 API 金鑰，完全免費使用。

#### 使用範例

```rust
use crate::engines::duckduckgo::DuckDuckGoClient;

let client = DuckDuckGoClient::new();
let results = client.search("rust tutorial", 10).await?;
```

#### 限制

- **速率限制**：建議每秒不超過 1 次請求
- **結果品質**：適合一般查詢，複雜查詢建議使用 Exa

---

## 測試策略

### SDD (Specification-Driven Development)

#### Snapshot Testing

用於防止輸出格式意外變更：

```rust
#[test]
fn test_search_result_format() {
    let result = SearchResult {
        title: "Example".to_string(),
        url: "https://example.com".to_string(),
        snippet: Some("Description".to_string()),
    };

    let json = serde_json::to_string_pretty(&result).unwrap();
    insta::assert_snapshot!(json);
}
```

### TDD (Test-Driven Development)

#### 開發流程

1. **編寫測試**：先寫失敗的測試
2. **實作功能**：讓測試通過
3. **重構**：優化代碼品質

#### 範例

```rust
// 1. 編寫測試
#[test]
fn test_confidence_threshold() {
    let calculator = ConfidenceCalculator::new();
    assert_eq!(calculator.threshold(), 0.85);
}

// 2. 實作功能
impl ConfidenceCalculator {
    pub fn threshold(&self) -> f32 {
        0.85
    }
}
```

### BDD (Behavior-Driven Development)

#### 使用場景

- 端到端測試
- 使用者行為驗證
- 整合測試

#### 測試結構

```
tests/
├── integration/
│   ├── search_flow.rs
│   ├── routing_behavior.rs
│   └── cost_optimization.rs
└── fixtures/
    ├── sample_queries.json
    └── expected_results.json
```

### 效能測試

#### Benchmark 測試

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_search(c: &mut Criterion) {
    c.bench_function("duckduckgo_search", |b| {
        b.iter(|| {
            // 測試代碼
        });
    });
}

criterion_group!(benches, benchmark_search);
criterion_main!(benches);
```

#### 壓力測試

```bash
# 使用 wrk 進行壓力測試
wrk -t12 -c400 -d30s http://localhost:8080/search?q=rust
```

---

## 安全注意事項

### API 金鑰保護

#### 環境變數

**絕不**將 API 金鑰硬編碼在代碼中：

```rust
// ❌ 錯誤
let api_key = "sk-1234567890abcdef";

// ✅ 正確
let api_key = std::env::var("EXA_API_KEY")
    .expect("EXA_API_KEY must be set");
```

#### .gitignore 配置

確保 `.env` 檔案不被提交：

```gitignore
# API 金鑰
.env
.env.local
.env.*.local

# 敏感資料
secrets/
*.key
*.pem
```

### 輸入驗證

#### 查詢字串驗證

```rust
pub fn validate_query(query: &str) -> Result<(), ValidationError> {
    // 檢查長度
    if query.is_empty() || query.len() > 1000 {
        return Err(ValidationError::InvalidLength);
    }

    // 檢查特殊字符
    if query.contains(&['<', '>', '&', '"'][..]) {
        return Err(ValidationError::InvalidCharacters);
    }

    Ok(())
}
```

#### URL 驗證

```rust
use url::Url;

pub fn validate_url(url_str: &str) -> Result<Url, ValidationError> {
    let url = Url::parse(url_str)
        .map_err(|_| ValidationError::InvalidUrl)?;

    // 只允許 HTTP/HTTPS
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(ValidationError::UnsupportedScheme);
    }

    Ok(url)
}
```

### 錯誤處理

#### 不洩漏敏感資訊

```rust
// ❌ 錯誤：洩漏 API 金鑰
eprintln!("API request failed: {}", api_key);

// ✅ 正確：隱藏敏感資訊
eprintln!("API request failed: authentication error");
```

#### 優雅降級

```rust
pub async fn search_with_fallback(&self, query: &str) -> Result<Vec<SearchResult>> {
    // 嘗試 Exa
    match self.exa.search(query).await {
        Ok(results) => return Ok(results),
        Err(e) => {
            warn!("Exa search failed: {}, falling back to DuckDuckGo", e);
        }
    }

    // 降級到 DuckDuckGo
    self.duckduckgo.search(query).await
}
```

### 速率限制

#### 實作速率限制器

```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

pub struct RateLimitedClient {
    limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl RateLimitedClient {
    pub fn new(requests_per_second: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap());
        Self {
            limiter: RateLimiter::direct(quota),
        }
    }

    pub async fn request(&self) -> Result<()> {
        self.limiter.until_ready().await;
        // 執行請求
        Ok(())
    }
}
```

---

## 常見問題

### Q1: 如何選擇搜尋引擎？

**A**: 根據需求選擇：
- **DuckDuckGo**：一般查詢、無預算限制、快速測試
- **Exa**：需要高品質結果、語義搜尋、技術查詢
- **Tavily**：需要深度內容、複雜分析、高價值查詢

### Q2: 置信度閾值如何調整？

**A**: 根據研究發現，建議設定：
- **L1 → L2**：0.85（平衡成本與品質）
- **L2 → L3**：0.85（僅在必要時使用 Tavily）

可在 `config/thresholds.toml` 中調整：
```toml
[confidence]
l1_threshold = 0.85
l2_threshold = 0.85
```

### Q3: 如何優化成本？

**A**: 採用以下策略：
1. **優先使用免費引擎**：80% 查詢使用 DuckDuckGo
2. **智慧路由**：根據查詢複雜度選擇模型
3. **上下文裁剪**：減少 60-80% Token 消耗
4. **結果快取**：避免重複查詢

### Q4: 如何處理 API 速率限制？

**A**: 實作速率限制器和重試機制：
```rust
use backoff::ExponentialBackoff;

let backoff = ExponentialBackoff::default();
backoff::retry(backoff, || {
    self.api_request()
}).await?;
```

### Q5: 如何監控系統效能？

**A**: 使用日誌和指標：
```rust
use tracing::{info, warn, error};

info!("Search completed in {}ms", elapsed);
warn!("High latency detected: {}ms", latency);
error!("API request failed: {}", error);
```

### Q6: 如何貢獻代碼？

**A**: 遵循以下流程：
1. Fork 專案
2. 建立功能分支：`git checkout -b feature/new-engine`
3. 編寫代碼和測試
4. 提交 PR，包含詳細說明
5. 等待審查和合併

### Q7: 如何報告安全漏洞？

**A**: 請私下聯繫維護者，不要公開披露：
- Email: security@example.com
- 遵守 90 天披露期限
- 提供詳細的 PoC 和影響分析

---

## 參考資源

### 官方文檔
- [Rust 官方文檔](https://doc.rust-lang.org/)
- [Tokio 異步運行時](https://tokio.rs/)
- [Exa API 文檔](https://docs.exa.ai)
- [Tavily API 文檔](https://docs.tavily.com)

### 技術文章
- [rkyv 零拷貝序列化](https://rkyv.org/)
- [io_uring 效能優化](https://github.com/tokio-rs/tokio-uring)
- [語義路由最佳實踐](https://vllm-semantic-router.com/)

### 相關專案
- [Tantivy 搜尋引擎](https://github.com/quickwit-oss/tantivy)
- [Meilisearch](https://github.com/meilisearch/meilisearch)
- [RouteLLM](https://github.com/lm-sys/RouteLLM)

---

**文檔版本**: 1.0.0
**最後更新**: 2026-02-08
**維護者**: Bose Security Research Team
