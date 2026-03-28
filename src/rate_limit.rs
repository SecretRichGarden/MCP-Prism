use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use tokio::sync::Mutex;

use crate::{
    config::RateLimitConfig,
    error::{AppError, AppResult},
};

#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64,
    last_update: Instant,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_update: Instant::now(),
        }
    }

    fn try_acquire(&mut self, amount: f64) -> bool {
        let now = Instant::now();
        let elapsed = now
            .checked_duration_since(self.last_update)
            .unwrap_or(Duration::ZERO)
            .as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_update = now;

        if self.tokens >= amount {
            self.tokens -= amount;
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
pub struct RateLimitManager {
    clients: Mutex<HashMap<String, TokenBucket>>,
    providers: Mutex<HashMap<String, TokenBucket>>,
}

impl RateLimitManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn acquire_client(&self, client_id: &str, config: &RateLimitConfig) -> AppResult<()> {
        let mut clients = self.clients.lock().await;
        let bucket = clients.entry(client_id.to_string()).or_insert_with(|| {
            TokenBucket::new(config.client_capacity, config.client_refill_per_second)
        });
        if bucket.try_acquire(1.0) {
            Ok(())
        } else {
            Err(AppError::Auth(format!(
                "client rate limit exceeded for {client_id}"
            )))
        }
    }

    pub async fn acquire_provider(&self, provider_id: &str, rpm: f64) -> AppResult<()> {
        let mut providers = self.providers.lock().await;
        let refill = (rpm / 60.0).max(0.1);
        let bucket = providers
            .entry(provider_id.to_string())
            .or_insert_with(|| TokenBucket::new(rpm.max(1.0), refill));
        if bucket.try_acquire(1.0) {
            Ok(())
        } else {
            Err(AppError::Provider(format!(
                "provider rate limit exceeded for {provider_id}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RateLimitManager;
    use crate::config::RateLimitConfig;

    #[tokio::test]
    async fn client_limit_rejects_when_bucket_is_empty() {
        let limiter = RateLimitManager::new();
        let config = RateLimitConfig {
            client_capacity: 1.0,
            client_refill_per_second: 0.0,
            provider_capacity_default: 1.0,
            provider_refill_per_second_default: 0.0,
        };
        assert!(limiter.acquire_client("a", &config).await.is_ok());
        assert!(limiter.acquire_client("a", &config).await.is_err());
    }
}
