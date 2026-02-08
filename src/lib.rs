pub mod types;
pub mod duckduckgo;
pub mod exa;
pub mod tavily;
pub mod client;
pub mod routing;
pub mod optimization;
pub mod processing;

pub use types::{SearchEngine, SearchError, SearchResult};
pub use client::MultiSearchClient;
pub use routing::{SemanticRouter, TaskComplexity, SearchStrategy};
pub use duckduckgo::DuckDuckGoClient;
pub use exa::ExaClient;
pub use tavily::TavilyClient;
pub use optimization::{SearchCache, CachedSearchResult};
pub use optimization::{PooledClient, PoolConfig};
pub use optimization::{RateLimiter, RateLimiterConfig};
pub use processing::{HtmlCleaner, ContextPruner};
