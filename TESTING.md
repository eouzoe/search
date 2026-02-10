# Bose Search MCP Server — 測試指南

## ✅ 全域註冊完成

MCP Server 已成功註冊到 Claude Code 全域配置：

```json
{
  "mcpServers": {
    "bose-search": {
      "type": "stdio",
      "command": "/home/eouzoe/src/active/bose-search/target/release/bose-mcp",
      "args": [],
      "env": {
        "SEARXNG_URL": "http://localhost:8080"
      }
    }
  }
}
```

**配置文件**: `~/.claude.json`

---

## 🧪 如何測試

### 方式 1: 在對話中自然使用

直接在 Claude Code 對話中提出搜尋需求，MCP Server 會自動被調用：

```
請幫我搜尋 "Rust async programming best practices"
```

```
查詢最新的 Next.js 14 文檔
```

```
搜尋關於 io_uring 的技術文章
```

### 方式 2: 明確要求使用 MCP 工具

```
使用 web_search 工具搜尋 "tantivy full text search"
```

```
用 bose-search 查詢 "SIMD optimization techniques"
```

### 方式 3: 檢查 MCP Server 健康狀態

```
檢查 bose-search MCP server 的健康狀態
```

---

## 🔧 可用的 MCP Tools

### 1. `web_search`
搜尋網頁內容（247 個搜尋引擎）

**參數**:
- `query` (必填) — 搜尋關鍵字
- `num_results` (選填) — 結果數量，預設 10
- `category` (選填) — 分類過濾 (general, it, science, etc.)
- `language` (選填) — 語言代碼 (zh-TW, en, etc.)
- `time_range` (選填) — 時間範圍 (day, week, month, year)

**範例**:
```json
{
  "query": "rust tokio async",
  "num_results": 20,
  "category": "it",
  "language": "en"
}
```

### 2. `health_check`
檢查 SearXNG 服務狀態

**參數**: 無

**返回**: `true` (正常) 或 `false` (異常)

---

## 🚀 進階用法

### 結合 Context7 使用

```
搜尋 Rust tokio 的最新教學，然後用 context7 獲取官方文檔範例
```

### 多引擎結果比較

```
用 bose-search 搜尋 "WebAssembly performance"，
然後用 Exa 搜尋相同主題，比較結果差異
```

### 技術研究工作流

```
1. 用 bose-search 搜尋技術概述
2. 用 Exa get_code_context 查找代碼範例
3. 用 Context7 獲取最新 API 文檔
```

---

## 🐛 故障排除

### MCP Server 未響應

1. **檢查 SearXNG 容器**:
   ```bash
   podman ps | grep searxng
   curl "http://localhost:8080/search?q=test&format=json"
   ```

2. **檢查 binary 是否存在**:
   ```bash
   ls -lh /home/eouzoe/src/active/bose-search/target/release/bose-mcp
   ```

3. **手動測試 MCP Server**:
   ```bash
   echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{
     "protocolVersion":"2025-03-26","capabilities":{},
     "clientInfo":{"name":"test","version":"0.1"}
   }}' | /home/eouzoe/src/active/bose-search/target/release/bose-mcp
   ```

### 重新構建 binary

```bash
cd /home/eouzoe/src/active/bose-search
cargo build --release -p bose-mcp
```

### 重新註冊 MCP Server

```bash
claude mcp remove bose-search
claude mcp add bose-search \
  --scope user \
  -e SEARXNG_URL=http://localhost:8080 \
  -- /home/eouzoe/src/active/bose-search/target/release/bose-mcp
```

---

## 📊 預期行為

當你在對話中提出搜尋需求時，Claude Code 會：

1. 識別需要使用 `web_search` 工具
2. 調用 bose-search MCP Server
3. MCP Server 向 SearXNG 發送請求
4. SearXNG 查詢 4 個引擎 (Google, Brave, DuckDuckGo, Startpage)
5. 返回融合後的搜尋結果
6. Claude 分析結果並回答你的問題

**典型響應時間**: 1-3 秒

---

## 📝 日誌查看

MCP Server 使用 `tracing` 記錄日誌。要查看日誌，設置環境變數：

```bash
export RUST_LOG=bose_mcp=debug,bose_searxng=debug
```

然後重啟 Claude Code 或重新註冊 MCP Server。

---

**最後更新**: 2026-02-11
**版本**: Phase 1 MVP
**狀態**: ✅ 生產就緒
