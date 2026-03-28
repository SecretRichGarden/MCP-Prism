mod amap;
mod brave;
mod census;
mod datagov;
mod finance_mcp;
mod github_graphql;
mod github_rest;
mod metaso;
mod openalex;
mod openfda;
mod pubmed;
mod tavily;
mod zhipu;

use std::{collections::HashMap, sync::Arc, time::Instant};

use async_trait::async_trait;
use chrono::Utc;
use secrecy::ExposeSecret;
use serde_json::json;

use crate::{
    config::{AppConfig, ProviderConfig},
    error::{AppError, AppResult},
    http_client::ProxyAwareHttpClient,
    models::{ProviderCatalogEntry, SearchType, UnifiedResult, UnifiedSearchRequest},
};

pub use amap::AmapAdapter;
pub use brave::BraveAdapter;
pub use census::CensusAdapter;
pub use datagov::DataGovAdapter;
pub use finance_mcp::FinanceMcpAdapter;
pub use github_graphql::GitHubGraphqlAdapter;
pub use github_rest::GitHubRestAdapter;
pub use metaso::MetasoAdapter;
pub use openalex::OpenAlexAdapter;
pub use openfda::OpenFdaAdapter;
pub use pubmed::PubMedAdapter;
pub use tavily::TavilyAdapter;
pub use zhipu::ZhipuAdapter;

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub provider_id: String,
    pub results: Vec<UnifiedResult>,
    pub latency_ms: u64,
}

#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn rpm(&self) -> f64;
    fn supports(&self, search_type: &SearchType) -> bool;
    async fn execute(&self, request: &UnifiedSearchRequest) -> AppResult<ProviderResponse>;
}

#[derive(Clone)]
pub struct AdapterRegistry {
    adapters: Vec<Arc<dyn ProviderAdapter>>,
    catalog: Vec<ProviderCatalogEntry>,
}

impl AdapterRegistry {
    pub fn from_config(config: &AppConfig) -> AppResult<Self> {
        let client = ProxyAwareHttpClient::new(config.server.request_timeout_ms, &config.proxy)?;
        let mut adapters: Vec<Arc<dyn ProviderAdapter>> = Vec::new();

        push_if_available(
            &mut adapters,
            &config.providers.brave,
            BraveAdapter::new(config.providers.brave.clone(), client.clone()),
        );
        push_if_available(
            &mut adapters,
            &config.providers.tavily,
            TavilyAdapter::new(config.providers.tavily.clone(), client.clone()),
        );
        push_if_available(
            &mut adapters,
            &config.providers.zhipu,
            ZhipuAdapter::new(config.providers.zhipu.clone(), client.clone()),
        );
        push_if_available(
            &mut adapters,
            &config.providers.metaso,
            MetasoAdapter::new(config.providers.metaso.clone(), client.clone()),
        );
        push_if_available(
            &mut adapters,
            &config.providers.pubmed,
            PubMedAdapter::new(config.providers.pubmed.clone(), client.clone()),
        );
        push_if_available(
            &mut adapters,
            &config.providers.openalex,
            OpenAlexAdapter::new(config.providers.openalex.clone(), client.clone()),
        );
        push_if_available(
            &mut adapters,
            &config.providers.amap,
            AmapAdapter::new(config.providers.amap.clone(), client.clone()),
        );
        push_if_available(
            &mut adapters,
            &config.providers.finance_mcp,
            FinanceMcpAdapter::new(config.providers.finance_mcp.clone(), client.clone()),
        );
        push_if_available(
            &mut adapters,
            &config.providers.github_rest,
            GitHubRestAdapter::new(config.providers.github_rest.clone(), client.clone()),
        );
        push_if_available(
            &mut adapters,
            &config.providers.github_graphql,
            GitHubGraphqlAdapter::new(config.providers.github_graphql.clone(), client.clone()),
        );
        push_if_available(
            &mut adapters,
            &config.providers.datagov,
            DataGovAdapter::new(config.providers.datagov.clone(), client.clone()),
        );
        push_if_available(
            &mut adapters,
            &config.providers.openfda,
            OpenFdaAdapter::new(config.providers.openfda.clone(), client.clone()),
        );
        push_if_available(
            &mut adapters,
            &config.providers.census,
            CensusAdapter::new(config.providers.census.clone(), client),
        );

        Ok(Self {
            adapters,
            catalog: config.provider_catalog(),
        })
    }

    pub fn available_provider_ids(&self) -> Vec<String> {
        self.catalog
            .iter()
            .filter(|provider| provider.available)
            .map(|provider| provider.id.clone())
            .collect()
    }

    pub fn resolve_many(
        &self,
        ids: &[String],
        search_type: &SearchType,
    ) -> Vec<Arc<dyn ProviderAdapter>> {
        let mut resolved = Vec::new();
        for id in ids {
            if let Some(adapter) = self
                .adapters
                .iter()
                .find(|adapter| adapter.id() == id && adapter.supports(search_type))
            {
                resolved.push(adapter.clone());
            }
        }
        resolved
    }

    pub fn catalog(&self) -> Vec<ProviderCatalogEntry> {
        self.catalog.clone()
    }
}

fn push_if_available<T>(
    items: &mut Vec<Arc<dyn ProviderAdapter>>,
    config: &ProviderConfig,
    adapter: T,
) where
    T: ProviderAdapter + 'static,
{
    if config.is_available() {
        items.push(Arc::new(adapter));
    }
}

pub fn normalize_result(
    provider_id: &str,
    id: impl Into<String>,
    title: impl Into<String>,
    url: Option<String>,
    snippet: impl Into<String>,
    score: f64,
    metadata: serde_json::Value,
) -> UnifiedResult {
    UnifiedResult {
        id: id.into(),
        title: title.into(),
        url,
        snippet: snippet.into(),
        score,
        source: provider_id.to_string(),
        metadata,
        timestamp: Utc::now(),
    }
}

pub async fn execute_with_timing<F, Fut>(
    provider_id: &'static str,
    func: F,
) -> AppResult<ProviderResponse>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = AppResult<Vec<UnifiedResult>>>,
{
    let started = Instant::now();
    let results = func().await?;
    Ok(ProviderResponse {
        provider_id: provider_id.to_string(),
        results,
        latency_ms: started.elapsed().as_millis() as u64,
    })
}

pub fn extract_results_list(
    payload: &serde_json::Value,
) -> Vec<HashMap<String, serde_json::Value>> {
    payload
        .get("results")
        .and_then(|value| value.as_array())
        .or_else(|| payload.get("data").and_then(|value| value.as_array()))
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.as_object().cloned())
        .map(|item| item.into_iter().collect::<HashMap<_, _>>())
        .collect()
}

pub fn fallback_text(item: &HashMap<String, serde_json::Value>, keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| item.get(*key).and_then(|value| value.as_str()))
        .unwrap_or_default()
        .to_string()
}

pub fn fallback_url(item: &HashMap<String, serde_json::Value>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| item.get(*key).and_then(|value| value.as_str()))
        .map(ToOwned::to_owned)
}

pub fn metadata_object(extra: &[(&str, serde_json::Value)]) -> serde_json::Value {
    let mut object = serde_json::Map::new();
    for (key, value) in extra {
        object.insert((*key).to_string(), value.clone());
    }
    json!(object)
}

fn missing_api_key(config: &ProviderConfig) -> AppError {
    AppError::Provider(format!("provider {} is missing api key", config.id))
}

pub(crate) fn require_api_key(config: &ProviderConfig) -> AppResult<String> {
    config
        .api_key
        .as_ref()
        .map(|value| value.expose_secret().to_string())
        .ok_or_else(|| missing_api_key(config))
}
