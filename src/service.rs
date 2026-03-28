use std::{collections::HashSet, sync::Arc, time::Instant};

use uuid::Uuid;

use crate::{
    adapters::{AdapterRegistry, ProviderResponse},
    cache::SearchCache,
    config::{AppConfig, RateLimitConfig},
    error::{AppError, AppResult},
    models::{
        DispatchStrategy, ProviderCatalogEntry, ResponseMeta, ResponseTiming, RoutingDecision,
        SearchType, SourceInfo, UnifiedResponse, UnifiedSearchRequest,
    },
    observability::AppMetrics,
    rate_limit::RateLimitManager,
    router::DualLineRouter,
};

#[derive(Clone)]
pub struct SearchService {
    registry: AdapterRegistry,
    router: Arc<DualLineRouter>,
    cache: SearchCache,
    rate_limits: Arc<RateLimitManager>,
    metrics: AppMetrics,
    rate_limit_config: RateLimitConfig,
}

impl SearchService {
    pub fn new(config: &AppConfig) -> AppResult<Self> {
        Ok(Self {
            registry: AdapterRegistry::from_config(config)?,
            router: Arc::new(DualLineRouter::new(config)?),
            cache: SearchCache::new(config.cache.clone()),
            rate_limits: Arc::new(RateLimitManager::new()),
            metrics: AppMetrics::new(),
            rate_limit_config: config.rate_limit.clone(),
        })
    }

    pub fn provider_catalog(&self) -> Vec<ProviderCatalogEntry> {
        self.registry.catalog()
    }

    pub fn metrics(&self) -> AppMetrics {
        self.metrics.clone()
    }

    pub fn redis_enabled(&self) -> bool {
        self.cache.redis_enabled()
    }

    pub fn cache_key_for(request: &UnifiedSearchRequest) -> AppResult<String> {
        Ok(serde_json::to_string(request)?)
    }

    pub async fn prewarm(&self, requests: Vec<UnifiedSearchRequest>) -> Vec<AppResult<String>> {
        let mut outcomes = Vec::with_capacity(requests.len());
        for request in requests {
            let result = match Self::cache_key_for(&request) {
                Ok(key) => self.search(request).await.map(|_| key),
                Err(error) => Err(error),
            };
            outcomes.push(result);
        }
        outcomes
    }

    pub async fn invalidate(&self, request: &UnifiedSearchRequest) -> AppResult<String> {
        let key = Self::cache_key_for(request)?;
        self.cache.invalidate(&key).await;
        Ok(key)
    }

    pub async fn search(&self, request: UnifiedSearchRequest) -> AppResult<UnifiedResponse> {
        self.metrics.record_search_started();
        let started = Instant::now();
        let outcome = self.search_inner(request).await;
        match &outcome {
            Ok(response) => self
                .metrics
                .observe_search_latency_ms(response.timing.total_ms),
            Err(_) => {
                self.metrics.record_search_failed();
                self.metrics
                    .observe_search_latency_ms(started.elapsed().as_millis() as u64);
            }
        }
        outcome
    }

    async fn search_inner(&self, request: UnifiedSearchRequest) -> AppResult<UnifiedResponse> {
        request
            .validate()
            .map_err(|message| AppError::Validation(message.to_string()))?;

        let client_id = request.client_id.as_deref().unwrap_or("anonymous");
        self.rate_limits
            .acquire_client(client_id, &self.rate_limit_config)
            .await?;

        let cache_key = Self::cache_key_for(&request)?;
        if let Some(mut cached) = self.cache.get(&cache_key).await {
            cached.meta.cached = true;
            self.metrics.record_cache_hit();
            return Ok(cached);
        }

        let overall_started = Instant::now();
        let routing_started = Instant::now();
        let decision = self.router.plan(&request, &self.registry).await?;
        let routing_ms = routing_started.elapsed().as_millis() as u64;

        let responses = self.execute_plan(&decision, &request).await?;
        let api_ms = overall_started.elapsed().as_millis() as u64;
        let response = build_response(
            request,
            decision,
            responses,
            routing_ms,
            api_ms,
            overall_started,
        );

        self.cache
            .insert(cache_key, &response.search_type, response.clone())
            .await;
        self.metrics
            .observe_search_latency_ms(response.timing.total_ms);
        Ok(response)
    }

    async fn execute_plan(
        &self,
        decision: &RoutingDecision,
        request: &UnifiedSearchRequest,
    ) -> AppResult<Vec<Result<ProviderResponse, AppError>>> {
        let mut ids = decision.primary_sources.clone();
        for id in &decision.secondary_sources {
            if !ids.contains(id) {
                ids.push(id.clone());
            }
        }

        let adapters = self.registry.resolve_many(&ids, &decision.search_type);
        if adapters.is_empty() {
            return Err(AppError::Provider(format!(
                "no available providers for {:?}",
                decision.search_type
            )));
        }

        let futures = adapters
            .into_iter()
            .map(|adapter| {
                let request = request.clone();
                let limiter = self.rate_limits.clone();
                let metrics = self.metrics.clone();
                async move {
                    if let Err(error) = limiter.acquire_provider(adapter.id(), adapter.rpm()).await
                    {
                        metrics.record_provider_call(adapter.id(), "rate_limited");
                        return Err(error);
                    }

                    let result = adapter.execute(&request).await;
                    metrics.record_provider_call(
                        adapter.id(),
                        if result.is_ok() { "success" } else { "error" },
                    );
                    result
                }
            })
            .collect::<Vec<_>>();

        let responses = match decision.strategy {
            DispatchStrategy::Sequential => {
                let mut outputs = Vec::new();
                for future in futures {
                    outputs.push(future.await);
                }
                outputs
            }
            DispatchStrategy::Parallel | DispatchStrategy::Hybrid => {
                futures::future::join_all(futures).await
            }
        };

        if responses.iter().all(Result::is_err) {
            return Err(AppError::Provider(
                "all selected providers failed during execution".to_string(),
            ));
        }

        Ok(responses)
    }
}

fn build_response(
    request: UnifiedSearchRequest,
    decision: RoutingDecision,
    responses: Vec<Result<ProviderResponse, AppError>>,
    routing_ms: u64,
    total_api_ms: u64,
    overall_started: Instant,
) -> UnifiedResponse {
    let mut sources = Vec::new();
    let mut results = Vec::new();
    let mut dedupe = HashSet::new();

    for response in responses {
        match response {
            Ok(provider) => {
                let count = provider.results.len();
                for result in provider.results {
                    let key = result
                        .url
                        .clone()
                        .unwrap_or_else(|| format!("{}:{}", result.source, result.title));
                    if dedupe.insert(key) {
                        results.push(result);
                    }
                }
                sources.push(SourceInfo {
                    name: provider.provider_id,
                    results_count: count,
                    latency_ms: provider.latency_ms,
                    success: true,
                    error: None,
                });
            }
            Err(error) => sources.push(SourceInfo {
                name: "unknown".to_string(),
                results_count: 0,
                latency_ms: 0,
                success: false,
                error: Some(error.to_string()),
            }),
        }
    }

    results.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    UnifiedResponse {
        request_id: Uuid::new_v4().to_string(),
        query: request.query,
        search_type: if request.search_type == SearchType::Auto {
            decision.search_type.clone()
        } else {
            request.search_type
        },
        decision: decision.clone(),
        meta: ResponseMeta {
            total_results: results.len(),
            sources_used: sources,
            fallback_used: decision.fallback_reason.clone(),
            cached: false,
        },
        results,
        timing: ResponseTiming {
            routing_ms,
            total_api_ms,
            normalization_ms: 0,
            total_ms: overall_started.elapsed().as_millis() as u64,
        },
    }
}
