use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// 速率限制器配置
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    pub requests_per_second: f64,
    pub burst_size: usize,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10.0,
            burst_size: 10,
        }
    }
}

/// Token Bucket 速率限制器
pub struct RateLimiter {
    tokens: Mutex<f64>,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Mutex<Instant>,
}

impl RateLimiter {
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            tokens: Mutex::new(config.burst_size as f64),
            max_tokens: config.burst_size as f64,
            refill_rate: config.requests_per_second,
            last_refill: Mutex::new(Instant::now()),
        }
    }

    pub async fn acquire(&self) {
        loop {
            self.refill();
            {
                let mut tokens = self.tokens.lock().unwrap();
                if *tokens >= 1.0 {
                    *tokens -= 1.0;
                    return;
                }
            }

            let wait_time = Duration::from_secs_f64(1.0 / self.refill_rate);
            sleep(wait_time).await;
        }
    }

    pub fn try_acquire(&self) -> bool {
        self.refill();
        let mut tokens = self.tokens.lock().unwrap();
        if *tokens >= 1.0 {
            *tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn refill(&self) {
        let now = Instant::now();
        let mut last_refill = self.last_refill.lock().unwrap();
        let elapsed = now.duration_since(*last_refill).as_secs_f64();

        if elapsed > 0.0 {
            let mut tokens = self.tokens.lock().unwrap();
            let new_tokens = elapsed * self.refill_rate;
            *tokens = (*tokens + new_tokens).min(self.max_tokens);
            *last_refill = now;
        }
    }

    pub fn available_tokens(&self) -> f64 {
        self.refill();
        *self.tokens.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_config_default() {
        let config = RateLimiterConfig::default();
        assert_eq!(config.requests_per_second, 10.0);
        assert_eq!(config.burst_size, 10);
    }

    #[test]
    fn test_rate_limiter_creation() {
        let config = RateLimiterConfig {
            requests_per_second: 5.0,
            burst_size: 10,
        };
        let limiter = RateLimiter::new(config);
        assert_eq!(limiter.available_tokens(), 10.0);
    }

    #[test]
    fn test_try_acquire_success() {
        let limiter = RateLimiter::new(RateLimiterConfig::default());
        assert!(limiter.try_acquire());
        assert!(limiter.available_tokens() < 10.0);
    }

    #[test]
    fn test_try_acquire_exhaustion() {
        let config = RateLimiterConfig {
            requests_per_second: 10.0,
            burst_size: 3,
        };
        let limiter = RateLimiter::new(config);

        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(!limiter.try_acquire());
    }

    #[tokio::test]
    async fn test_acquire_blocking() {
        let config = RateLimiterConfig {
            requests_per_second: 10.0,
            burst_size: 1,
        };
        let limiter = RateLimiter::new(config);

        limiter.acquire().await;

        let start = Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(80));
    }

    #[tokio::test]
    async fn test_token_refill() {
        let config = RateLimiterConfig {
            requests_per_second: 10.0,
            burst_size: 5,
        };
        let limiter = RateLimiter::new(config);

        limiter.try_acquire();
        limiter.try_acquire();

        let tokens_before = limiter.available_tokens();

        sleep(Duration::from_millis(200)).await;

        let tokens_after = limiter.available_tokens();
        assert!(tokens_after > tokens_before);
    }

    #[tokio::test]
    async fn test_max_tokens_limit() {
        let config = RateLimiterConfig {
            requests_per_second: 100.0,
            burst_size: 5,
        };
        let limiter = RateLimiter::new(config);

        sleep(Duration::from_secs(1)).await;

        let tokens = limiter.available_tokens();
        assert!(tokens <= 5.0);
    }
}