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
    ProviderAdapter, ProviderResponse, execute_with_timing, metadata_object, normalize_result,
    require_api_key,
};

#[derive(Clone)]
pub struct TavilyAdapter {
    config: ProviderConfig,
    client: ProxyAwareHttpClient,
}

impl TavilyAdapter {
    pub fn new(config: ProviderConfig, client: ProxyAwareHttpClient) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl ProviderAdapter for TavilyAdapter {
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
                | SearchType::Academic
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
                "/search"
            );
            let topic = match request.search_type {
                SearchType::News => "news",
                _ => "general",
            };
            let body = TavilyRequest {
                query: request.query.clone(),
                topic: topic.to_string(),
                search_depth: "advanced".to_string(),
                max_results: request.options.normalized_limit(),
                include_raw_content: request.options.include_raw_content(),
            };
            let response = self
                .client
                .request(Method::POST, &url)?
                .bearer_auth(require_api_key(&self.config)?)
                .json(&body)
                .send()
                .await?
                .error_for_status()?;

            let payload: TavilyResponse = response.json().await?;
            Ok(payload
                .results
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    normalize_result(
                        self.id(),
                        format!("tavily-{index}"),
                        item.title,
                        Some(item.url),
                        item.content.unwrap_or_default(),
                        item.score.unwrap_or(1.0_f64 - index as f64 * 0.01_f64),
                        metadata_object(&[(
                            "published_date",
                            item.published_date
                                .map(serde_json::Value::String)
                                .unwrap_or_default(),
                        )]),
                    )
                })
                .collect())
        })
        .await
    }
}

#[derive(Debug, Serialize)]
struct TavilyRequest {
    query: String,
    topic: String,
    search_depth: String,
    max_results: usize,
    include_raw_content: bool,
}

#[derive(Debug, Deserialize)]
struct TavilyResponse {
    #[serde(default)]
    results: Vec<TavilyItem>,
}

#[derive(Debug, Deserialize)]
struct TavilyItem {
    title: String,
    url: String,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    score: Option<f64>,
    #[serde(default)]
    published_date: Option<String>,
}
