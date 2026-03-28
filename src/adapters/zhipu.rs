use async_trait::async_trait;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{
    config::ProviderConfig,
    error::AppResult,
    http_client::ProxyAwareHttpClient,
    models::{SearchType, UnifiedSearchRequest},
};

use super::{
    ProviderAdapter, ProviderResponse, execute_with_timing, normalize_result, require_api_key,
};

#[derive(Clone)]
pub struct ZhipuAdapter {
    config: ProviderConfig,
    client: ProxyAwareHttpClient,
}

impl ZhipuAdapter {
    pub fn new(config: ProviderConfig, client: ProxyAwareHttpClient) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl ProviderAdapter for ZhipuAdapter {
    fn id(&self) -> &'static str {
        self.config.id
    }

    fn rpm(&self) -> f64 {
        self.config.rpm
    }

    fn supports(&self, search_type: &SearchType) -> bool {
        matches!(
            search_type,
            SearchType::Web
                | SearchType::News
                | SearchType::Finance
                | SearchType::Company
                | SearchType::Patent
                | SearchType::Government
        )
    }

    async fn execute(&self, request: &UnifiedSearchRequest) -> AppResult<ProviderResponse> {
        execute_with_timing(self.id(), || async {
            let url = format!(
                "{}{}",
                self.config
                    .base_url
                    .as_deref()
                    .unwrap_or_default()
                    .trim_end_matches('/'),
                "/api/paas/v4/web_search"
            );
            let search_engine = match request.search_type {
                SearchType::News => "search_pro_quark",
                _ => "search_std",
            };
            let body = ZhipuRequest {
                search_engine: search_engine.to_string(),
                search_query: request.query.clone(),
                count: request.options.normalized_limit(),
            };
            let response = self
                .client
                .request(Method::POST, &url)?
                .bearer_auth(require_api_key(&self.config)?)
                .json(&body)
                .send()
                .await?
                .error_for_status()?;

            let payload: ZhipuResponse = response.json().await?;
            let items = payload
                .search_result
                .or(payload.results)
                .unwrap_or_default();

            Ok(items
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    normalize_result(
                        self.id(),
                        format!("zhipu-{index}"),
                        item.title,
                        Some(item.link),
                        item.content.unwrap_or_default(),
                        1.0_f64 - index as f64 * 0.01_f64,
                        serde_json::json!({"media": item.media}),
                    )
                })
                .collect())
        })
        .await
    }
}

#[derive(Debug, Serialize)]
struct ZhipuRequest {
    search_engine: String,
    search_query: String,
    count: usize,
}

#[derive(Debug, Deserialize)]
struct ZhipuResponse {
    #[serde(default)]
    search_result: Option<Vec<ZhipuItem>>,
    #[serde(default)]
    results: Option<Vec<ZhipuItem>>,
}

#[derive(Debug, Deserialize)]
struct ZhipuItem {
    title: String,
    link: String,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    media: Option<String>,
}
