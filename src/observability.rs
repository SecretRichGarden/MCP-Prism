use std::sync::{Arc, Mutex};

use prometheus_client::{
    encoding::{EncodeLabelSet, text::encode},
    metrics::{
        counter::Counter,
        family::Family,
        histogram::{Histogram, exponential_buckets},
    },
    registry::Registry,
};

#[derive(Clone)]
pub struct AppMetrics {
    registry: Arc<Mutex<Registry>>,
    search_requests: Counter,
    search_errors: Counter,
    cache_hits: Counter,
    provider_calls: Family<ProviderLabels, Counter>,
    provider_failures: Family<ProviderLabels, Counter>,
    search_latency: Histogram,
}

impl AppMetrics {
    pub fn new() -> Self {
        let registry = Arc::new(Mutex::new(Registry::default()));
        let search_requests = Counter::default();
        let search_errors = Counter::default();
        let cache_hits = Counter::default();
        let provider_calls = Family::<ProviderLabels, Counter>::default();
        let provider_failures = Family::<ProviderLabels, Counter>::default();
        let search_latency = Histogram::new(exponential_buckets(0.01, 2.0, 16));

        {
            let mut guard = registry.lock().expect("metrics registry poisoned");
            guard.register(
                "mcp_prism_search_requests_total",
                "Total unified search requests.",
                search_requests.clone(),
            );
            guard.register(
                "mcp_prism_search_errors_total",
                "Total unified search failures.",
                search_errors.clone(),
            );
            guard.register(
                "mcp_prism_cache_hits_total",
                "Total search cache hits.",
                cache_hits.clone(),
            );
            guard.register(
                "mcp_prism_provider_calls_total",
                "Provider call count by provider and outcome.",
                provider_calls.clone(),
            );
            guard.register(
                "mcp_prism_provider_failures_total",
                "Provider failure count by provider and outcome.",
                provider_failures.clone(),
            );
            guard.register(
                "mcp_prism_search_latency_seconds",
                "Unified search latency histogram.",
                search_latency.clone(),
            );
        }

        Self {
            registry,
            search_requests,
            search_errors,
            cache_hits,
            provider_calls,
            provider_failures,
            search_latency,
        }
    }

    pub fn record_search_started(&self) {
        self.search_requests.inc();
    }

    pub fn record_search_failed(&self) {
        self.search_errors.inc();
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.inc();
    }

    pub fn record_provider_call(&self, provider: &str, outcome: &str) {
        self.provider_calls
            .get_or_create(&ProviderLabels {
                provider: provider.to_string(),
                outcome: outcome.to_string(),
            })
            .inc();
        if outcome != "success" {
            self.provider_failures
                .get_or_create(&ProviderLabels {
                    provider: provider.to_string(),
                    outcome: outcome.to_string(),
                })
                .inc();
        }
    }

    pub fn observe_search_latency_ms(&self, latency_ms: u64) {
        self.search_latency.observe(latency_ms as f64 / 1000.0);
    }

    pub fn render(&self) -> String {
        let mut body = String::new();
        let guard = self.registry.lock().expect("metrics registry poisoned");
        let _ = encode(&mut body, &guard);
        body
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ProviderLabels {
    provider: String,
    outcome: String,
}
