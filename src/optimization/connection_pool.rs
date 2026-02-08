use reqwest::Client;
use tokio::sync::Semaphore;
use std::sync::Arc;
use std::time::Duration;

/// 連線池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_concurrent: usize,
    pub pool_idle_per_host: usize,
    pub pool_idle_timeout: Duration,
    pub request_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            pool_idle_per_host: 20,
            pool_idle_timeout: Duration::from_secs(90),
            request_timeout: Duration::from_secs(30),
        }
    }
}

/// 帶連線池的 HTTP 客戶端
pub struct PooledClient {
    client: Client,
    pub(crate) semaphore: Arc<Semaphore>,
    config: PoolConfig,
}

impl PooledClient {
    pub fn new(config: PoolConfig) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .pool_idle_timeout(config.pool_idle_timeout)
            .pool_max_idle_per_host(config.pool_idle_per_host)
            .timeout(config.request_timeout)
            .build()?;

        Ok(Self {
            client,
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            config,
        })
    }

    pub fn with_defaults() -> Result<Self, reqwest::Error> {
        Self::new(PoolConfig::default())
    }

    pub async fn get(
        &self,
        url: &str,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        let _permit = self.semaphore.acquire().await?;
        let response = self.client.get(url).send().await?;
        Ok(response)
    }

    pub async fn post_json<T: serde::Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        let _permit = self.semaphore.acquire().await?;
        let response = self.client.post(url).json(body).send().await?;
        Ok(response)
    }

    pub fn active_permits(&self) -> usize {
        self.config.max_concurrent - self.semaphore.available_permits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_concurrent, 10);
        assert_eq!(config.pool_idle_per_host, 20);
        assert_eq!(config.pool_idle_timeout, Duration::from_secs(90));
        assert_eq!(config.request_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_pool_config_custom() {
        let config = PoolConfig {
            max_concurrent: 5,
            pool_idle_per_host: 10,
            pool_idle_timeout: Duration::from_secs(60),
            request_timeout: Duration::from_secs(15),
        };
        assert_eq!(config.max_concurrent, 5);
    }

    #[tokio::test]
    async fn test_pooled_client_creation() {
        let client = PooledClient::with_defaults();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_active_permits_initial() {
        let client = PooledClient::with_defaults().unwrap();
        assert_eq!(client.active_permits(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_limit() {
        let config = PoolConfig {
            max_concurrent: 2,
            ..Default::default()
        };
        let client = Arc::new(PooledClient::new(config).unwrap());

        let client1 = client.clone();
        let client2 = client.clone();
        let client3 = client.clone();

        let handle1 = tokio::spawn(async move {
            let _permit = client1.semaphore.acquire().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        });

        let handle2 = tokio::spawn(async move {
            let _permit = client2.semaphore.acquire().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        });

        tokio::time::sleep(Duration::from_millis(10)).await;

        let active = client.active_permits();
        assert_eq!(active, 2);

        let handle3 = tokio::spawn(async move {
            let _permit = client3.semaphore.acquire().await;
        });

        let _ = tokio::join!(handle1, handle2, handle3);
    }
}