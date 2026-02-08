mod types;
mod duckduckgo;
mod exa;
mod client;

pub use types::{SearchEngine, SearchError, SearchResult};
pub use client::MultiSearchClient;

use clap::{Parser, ValueEnum};
use dotenv::dotenv;
use std::env;

#[derive(Parser)]
#[command(name = "bose-search")]
#[command(about = "Bose 安全研究 - 多引擎搜尋工具", long_about = None)]
struct Cli {
    /// 搜尋查詢
    #[arg(short, long)]
    query: String,

    /// 搜尋引擎選擇
    #[arg(short, long, value_enum, default_value = "duckduckgo")]
    engine: EngineChoice,

    /// 結果數量
    #[arg(short, long, default_value = "10")]
    num: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum EngineChoice {
    /// DuckDuckGo（完全免費）
    Duckduckgo,
    /// Exa（$10 免費額度，AI 搜尋）
    Exa,
}

impl From<EngineChoice> for SearchEngine {
    fn from(choice: EngineChoice) -> Self {
        match choice {
            EngineChoice::Duckduckgo => SearchEngine::DuckDuckGo,
            EngineChoice::Exa => SearchEngine::Exa,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 載入 .env 檔案
    dotenv().ok();

    let cli = Cli::parse();

    // 建立搜尋客戶端
    let mut client = MultiSearchClient::new();

    // 如果有 Exa API 金鑰，則設定
    if let Ok(exa_key) = env::var("EXA_API_KEY") {
        client = client.with_exa(&exa_key);
    }

    // 執行搜尋
    println!("🔎 搜尋: \"{}\"", cli.query);
    println!("📊 引擎: {:?}", cli.engine);
    println!("📈 結果數: {}\n", cli.num);

    match client.search(&cli.query, cli.engine.into(), cli.num).await {
        Ok(results) => {
            if results.is_empty() {
                println!("❌ 沒有找到結果");
            } else {
                println!("✅ 找到 {} 個結果:\n", results.len());
                for (i, result) in results.iter().enumerate() {
                    println!("{}. {}", i + 1, result.title);
                    println!("   🔗 {}", result.url);
                    if let Some(snippet) = &result.snippet {
                        println!("   📝 {}", snippet);
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 搜尋失敗: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
