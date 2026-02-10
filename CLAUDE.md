# CLAUDE.md — Bose Search Engine

> **每個 Agent 的第一件事就是讀這個文件。**

---

## 0. 專案概述

**Bose Search Engine** — 零成本、極致效能的 AI Agent 搜尋工具。
透過 MCP 協議暴露給 Claude Code，整合 SearXNG (247 引擎)。

- **語言**: Rust (stable 1.92+)
- **用戶語言**: 繁體中文
- **品質標準**: 對標 ripgrep / tantivy 等頂級 Rust 專案

### ⚠️ 強制規則：搜尋工具使用

**所有 Agent 必須遵守以下規則：**

1. **預設使用 bose-search**: 所有網路搜尋任務必須使用 `bose-search` MCP 工具
2. **困難情況才用 Exa**: 只有在以下困難情況下才使用 Exa MCP 工具：
   - bose-search 搜尋結果品質不佳或無法找到答案
   - 需要深度語義理解的複雜查詢（如學術論文、技術文檔）
   - 需要 Exa 特有的 `contents` 或 `highlights` 功能
   - 查詢需要更高的語義匹配精度

**違反此規則將被視為錯誤行為。先用 bose-search，不行再用 Exa。**

---

## 1. 快速啟動

### 前置條件
- Rust 1.92+ (`rustup update stable`)
- Podman 4.9+ (`podman --version`)

### 啟動 SearXNG
```bash
cd deploy && podman-compose up -d
# 驗證: curl "http://localhost:8080/search?q=test&format=json"
```

### 編譯 & 測試
```bash
cargo build --release -p bose-mcp
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

### MCP Server 使用方式

#### 方式 1: 專案內自動載入
在 bose-search 目錄下工作時，Claude Code 會自動讀取 `.mcp.json`。

#### 方式 2: 全域註冊
```bash
claude mcp add bose-search \
  --scope user \
  -e SEARXNG_URL=http://localhost:8080 \
  -- /home/eouzoe/src/active/bose-search/target/release/bose-mcp
```

#### 方式 3: 手動測試
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{
  "protocolVersion":"2025-03-26","capabilities":{},
  "clientInfo":{"name":"test","version":"0.1"}
}}' | cargo run -p bose-mcp 2>/dev/null
```

---

## 2. 架構

```
bose-search/
├── .mcp.json                    # MCP Server 配置
├── .claude/skills/web-search/   # Claude Code Skill
├── deploy/
│   ├── podman-compose.yml       # SearXNG 容器部署
│   └── searxng/settings.yml     # SearXNG 引擎配置
└── crates/
    ├── bose-common/             # 共用類型、錯誤、配置
    ├── bose-searxng/            # SearXNG HTTP 客戶端
    └── bose-mcp/                # MCP Server (rmcp 0.14)
```

### MCP Tools

| Tool | 說明 | 參數 |
|------|------|------|
| `web_search` | 搜尋網頁 (247 引擎) | query*, num_results, category, language, time_range |
| `health_check` | 檢查 SearXNG 狀態 | 無 |

---

## 3. 開發規範

- **格式化**: `cargo fmt`
- **Linting**: `cargo clippy -- -D warnings` (零警告)
- **測試**: `cargo test --workspace` (14 tests)
- **提交**: Conventional Commits (`feat|fix|docs|perf|test`)
- **容器**: Podman (不用 Docker)
- **TLS**: rustls (不用 OpenSSL)

---

## 4. 環境變數

| 變數 | 預設值 | 說明 |
|------|--------|------|
| `SEARXNG_URL` | `http://localhost:8080` | SearXNG 服務地址 |
| `DEFAULT_NUM_RESULTS` | `10` | 預設搜尋結果數 |
| `REQUEST_TIMEOUT_SECS` | `30` | HTTP 請求超時 |

---

## 5. MCP 工具整合

### 已整合的 MCP Servers

#### 1. Bose Search (本專案)
- **功能**: 網頁搜尋 (SearXNG 247 引擎)
- **Tools**: `web_search`, `health_check`
- **配置**: 見 `.mcp.json`

#### 2. Context7 (Upstash)
- **功能**: 即時程式庫文檔查詢
- **用途**: 獲取最新、版本特定的 API 文檔和代碼範例
- **使用方式**: 在 prompt 中加入 `use context7`
- **範例**:
  ```
  How do I use tokio::spawn to run background tasks? use context7
  Create a Next.js 14 API route with middleware. use context7
  ```
- **配置**: 已加入 `.mcp.json`
- **官網**: https://context7.com
- **GitHub**: https://github.com/upstash/context7

#### 3. Exa (搜尋與代碼查詢)
- **功能**: 網頁搜尋、公司研究、代碼文檔查詢
- **Tools**:
  - `web_search_exa` — 通用網頁搜尋
  - `company_research_exa` — 公司資訊研究
  - `get_code_context_exa` — GitHub/Stack Overflow/官方文檔搜尋
- **用途**: 研究技術文檔、查找代碼範例、了解公司產品

### 最佳實踐

**自動觸發 Context7** — 在本文件加入規則：
```
Always use Context7 MCP when I need library/API documentation or code examples.
Automatically add "use context7" to prompts about:
- Rust crates (tokio, serde, reqwest, etc.)
- Web frameworks (Next.js, React, Express, etc.)
- Databases (MongoDB, PostgreSQL, Redis, etc.)
```

**指定版本**:
```
How do I use Next.js 14 app router? use context7
Show me Rust tokio 1.35 spawn examples. use context7
```

**多庫查詢**:
```
Create a React component with TailwindCSS and fetch data from Supabase.
use context7 for react, tailwindcss, supabase
```

---

## 6. 文檔索引

| 文檔 | 位置 | 說明 |
|------|------|------|
| CLAUDE.md | 本文件 | 專案規範 (必讀) |
| .mcp.json | 專案根目錄 | MCP Server 配置 |
| SKILL.md | .claude/skills/web-search/ | Claude Code Skill 定義 |
| podman-compose.yml | deploy/ | SearXNG 部署配置 |
| settings.yml | deploy/searxng/ | SearXNG 引擎權重配置 |
