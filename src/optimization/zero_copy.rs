//! 零拷貝序列化 - 使用 rkyv 實現高效能資料處理

use rkyv::{Archive, Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// 可序列化的搜尋結果
#[derive(Archive, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[rkyv(
    // 啟用比較功能
    compare(PartialEq),
    // 啟用 Debug
    derive(Debug)
)]
pub struct CachedSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub content: Option<String>,
    pub timestamp: u64,
}

impl CachedSearchResult {
    /// 從 SearchResult 建立快取結果
    pub fn from_search_result(result: &crate::types::SearchResult) -> Self {
        Self {
            title: result.title.clone(),
            url: result.url.clone(),
            snippet: result.snippet.clone(),
            content: result.content.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// 轉換為 SearchResult
    pub fn to_search_result(&self) -> crate::types::SearchResult {
        crate::types::SearchResult {
            title: self.title.clone(),
            url: self.url.clone(),
            snippet: self.snippet.clone(),
            content: self.content.clone(),
        }
    }
}

/// 搜尋結果快取
pub struct SearchCache {
    cache: RwLock<HashMap<String, Vec<u8>>>,
    max_size: usize,
    ttl_seconds: u64,
}

impl SearchCache {
    /// 建立新的快取
    pub fn new(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_size,
            ttl_seconds,
        }
    }

    /// 使用預設配置建立快取
    /// 預設：最多 1000 個項目，TTL 1 小時
    pub fn with_defaults() -> Self {
        Self::new(1000, 3600)
    }

    /// 儲存搜尋結果（零拷貝序列化）
    pub fn store(&self, key: &str, results: &[CachedSearchResult]) -> Result<(), String> {
        // 轉換為 Vec 以便序列化
        let results_vec = results.to_vec();
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&results_vec)
            .map_err(|e| format!("序列化錯誤: {:?}", e))?;

        let mut cache = self.cache.write().unwrap();

        // 檢查快取大小，實作簡單的 LRU
        if cache.len() >= self.max_size {
            // 移除第一個項目（簡化版 LRU）
            if let Some(oldest_key) = cache.keys().next().cloned() {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(key.to_string(), bytes.to_vec());
        Ok(())
    }

    /// 讀取搜尋結果（零拷貝反序列化）
    pub fn get(&self, key: &str) -> Option<Vec<CachedSearchResult>> {
        let cache = self.cache.read().unwrap();

        cache.get(key).and_then(|bytes| {
            // 零拷貝存取
            let archived = unsafe {
                rkyv::access_unchecked::<rkyv::Archived<Vec<CachedSearchResult>>>(bytes)
            };

            // 檢查 TTL
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if let Some(first) = archived.first() {
                if now - first.timestamp > self.ttl_seconds {
                    return None;  // 已過期
                }
            }

            // 反序列化（如果需要修改）
            rkyv::deserialize::<Vec<CachedSearchResult>, rkyv::rancor::Error>(archived).ok()
        })
    }

    /// 檢查快取是否存在且未過期
    pub fn contains(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// 清除快取
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// 取得快取大小
    pub fn size(&self) -> usize {
        self.cache.read().unwrap().len()
    }

    /// 移除指定項目
    pub fn remove(&self, key: &str) -> bool {
        let mut cache = self.cache.write().unwrap();
        cache.remove(key).is_some()
    }

    /// 取得快取統計資訊
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read().unwrap();
        let total_bytes: usize = cache.values().map(|v| v.len()).sum();

        CacheStats {
            entries: cache.len(),
            total_bytes,
            max_size: self.max_size,
            ttl_seconds: self.ttl_seconds,
        }
    }
}

/// 快取統計資訊
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub total_bytes: usize,
    pub max_size: usize,
    pub ttl_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_results() -> Vec<CachedSearchResult> {
        vec![
            CachedSearchResult {
                title: "Test Result 1".to_string(),
                url: "https://example.com/1".to_string(),
                snippet: Some("Test snippet 1".to_string()),
                content: None,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
            CachedSearchResult {
                title: "Test Result 2".to_string(),
                url: "https://example.com/2".to_string(),
                snippet: Some("Test snippet 2".to_string()),
                content: Some("Full content here".to_string()),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        ]
    }

    #[test]
    fn test_cache_store_and_get() {
        let cache = SearchCache::new(100, 3600);
        let results = create_test_results();

        cache.store("test_query", &results).unwrap();

        let retrieved = cache.get("test_query").unwrap();
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].title, "Test Result 1");
        assert_eq!(retrieved[1].title, "Test Result 2");
    }

    #[test]
    fn test_cache_miss() {
        let cache = SearchCache::new(100, 3600);
        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn test_cache_contains() {
        let cache = SearchCache::new(100, 3600);
        let results = create_test_results();

        assert!(!cache.contains("test_query"));
        cache.store("test_query", &results).unwrap();
        assert!(cache.contains("test_query"));
    }

    #[test]
    fn test_cache_clear() {
        let cache = SearchCache::new(100, 3600);
        let results = create_test_results();

        cache.store("test1", &results).unwrap();
        cache.store("test2", &results).unwrap();
        assert_eq!(cache.size(), 2);

        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_remove() {
        let cache = SearchCache::new(100, 3600);
        let results = create_test_results();

        cache.store("test_query", &results).unwrap();
        assert!(cache.contains("test_query"));

        assert!(cache.remove("test_query"));
        assert!(!cache.contains("test_query"));
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = SearchCache::new(2, 3600);  // 最多 2 個項目
        let results = create_test_results();

        cache.store("key1", &results).unwrap();
        cache.store("key2", &results).unwrap();
        assert_eq!(cache.size(), 2);

        // 新增第三個項目應該觸發淘汰
        cache.store("key3", &results).unwrap();
        assert_eq!(cache.size(), 2);
    }

    #[test]
    fn test_cache_stats() {
        let cache = SearchCache::new(100, 3600);
        let results = create_test_results();

        cache.store("test", &results).unwrap();

        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert!(stats.total_bytes > 0);
        assert_eq!(stats.max_size, 100);
        assert_eq!(stats.ttl_seconds, 3600);
    }

    #[test]
    fn test_cached_result_conversion() {
        let search_result = crate::types::SearchResult {
            title: "Test".to_string(),
            url: "https://example.com".to_string(),
            snippet: Some("Snippet".to_string()),
            content: None,
        };

        let cached = CachedSearchResult::from_search_result(&search_result);
        assert_eq!(cached.title, "Test");
        assert_eq!(cached.url, "https://example.com");

        let converted = cached.to_search_result();
        assert_eq!(converted.title, search_result.title);
        assert_eq!(converted.url, search_result.url);
    }
}
