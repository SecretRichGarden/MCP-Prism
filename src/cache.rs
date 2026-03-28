use std::{collections::HashMap, sync::Arc, time::Instant};

use redis::AsyncCommands;
use tokio::sync::RwLock;

use crate::{
    config::CacheConfig,
    models::{SearchType, UnifiedResponse},
};

#[derive(Clone)]
pub struct SearchCache {
    config: CacheConfig,
    memory: Arc<RwLock<HashMap<String, CachedResponse>>>,
    redis: Option<redis::Client>,
}

impl SearchCache {
    pub fn new(config: CacheConfig) -> Self {
        let redis = config
            .redis_url
            .as_ref()
            .and_then(|url| redis::Client::open(url.as_str()).ok());
        Self {
            config,
            memory: Arc::new(RwLock::new(HashMap::new())),
            redis,
        }
    }

    pub async fn get(&self, key: &str) -> Option<UnifiedResponse> {
        if let Some(value) = self.get_memory(key).await {
            return Some(value);
        }

        if let Some(redis) = &self.redis
            && let Ok(mut connection) = redis.get_multiplexed_async_connection().await
        {
            let redis_key = self.redis_key(key);
            if let Ok(Some(raw)) = connection.get::<_, Option<String>>(&redis_key).await
                && let Ok(value) = serde_json::from_str::<UnifiedResponse>(&raw)
            {
                let ttl = self.config.ttl_for(&value.search_type);
                self.insert_memory(key.to_string(), value.clone(), ttl)
                    .await;
                return Some(value);
            }
        }

        None
    }

    pub async fn insert(&self, key: String, search_type: &SearchType, value: UnifiedResponse) {
        let ttl = self.config.ttl_for(search_type);
        self.insert_memory(key.clone(), value.clone(), ttl).await;

        if let Some(redis) = &self.redis
            && let Ok(mut connection) = redis.get_multiplexed_async_connection().await
            && let Ok(serialized) = serde_json::to_string(&value)
        {
            let redis_key = self.redis_key(&key);
            let _ = connection
                .set_ex::<_, _, ()>(redis_key, serialized, ttl)
                .await;
        }
    }

    pub async fn invalidate(&self, key: &str) {
        self.memory.write().await.remove(key);
        if let Some(redis) = &self.redis
            && let Ok(mut connection) = redis.get_multiplexed_async_connection().await
        {
            let _ = connection.del::<_, ()>(self.redis_key(key)).await;
        }
    }

    pub fn redis_enabled(&self) -> bool {
        self.redis.is_some()
    }

    async fn get_memory(&self, key: &str) -> Option<UnifiedResponse> {
        let items = self.memory.read().await;
        items.get(key).and_then(|cached| {
            if cached.expires_at > Instant::now() {
                Some(cached.value.clone())
            } else {
                None
            }
        })
    }

    async fn insert_memory(&self, key: String, value: UnifiedResponse, ttl: u64) {
        self.memory.write().await.insert(
            key,
            CachedResponse {
                expires_at: Instant::now() + std::time::Duration::from_secs(ttl.max(1)),
                value,
            },
        );
    }

    fn redis_key(&self, key: &str) -> String {
        format!("{}:{key}", self.config.key_prefix)
    }
}

#[derive(Clone)]
struct CachedResponse {
    expires_at: Instant,
    value: UnifiedResponse,
}

#[cfg(test)]
mod tests {
    use super::SearchCache;
    use crate::{
        config::CacheConfig,
        models::{
            DispatchStrategy, ResponseMeta, ResponseTiming, RouterLine, RoutingDecision,
            SearchType, UnifiedResponse,
        },
    };

    #[tokio::test]
    async fn memory_cache_round_trip() {
        let cache = SearchCache::new(CacheConfig {
            ttl_seconds: 60,
            redis_url: None,
            key_prefix: "test".to_string(),
            realtime_ttl_seconds: 60,
            daily_ttl_seconds: 60,
            monthly_ttl_seconds: 60,
            yearly_ttl_seconds: 60,
        });
        let response = UnifiedResponse {
            request_id: "a".to_string(),
            query: "hello".to_string(),
            search_type: SearchType::Web,
            decision: RoutingDecision {
                search_type: SearchType::Web,
                primary_sources: vec!["brave".to_string()],
                secondary_sources: vec![],
                strategy: DispatchStrategy::Sequential,
                line: RouterLine::Heuristic,
                reasoning: "test".to_string(),
                fallback_reason: None,
                experiment: None,
            },
            meta: ResponseMeta {
                total_results: 0,
                sources_used: vec![],
                fallback_used: None,
                cached: false,
            },
            results: vec![],
            timing: ResponseTiming {
                routing_ms: 0,
                total_api_ms: 0,
                normalization_ms: 0,
                total_ms: 0,
            },
        };
        cache
            .insert("k".to_string(), &SearchType::Web, response.clone())
            .await;
        assert_eq!(cache.get("k").await.unwrap().request_id, "a");
    }
}
