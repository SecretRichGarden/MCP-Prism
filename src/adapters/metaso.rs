use async_trait::async_trait;
use reqwest::Method;
use serde_json::json;

use crate::{
    config::ProviderConfig,
    error::AppResult,
    http_client::ProxyAwareHttpClient,
    models::{SearchType, UnifiedSearchRequest},
};

use super::{
    ProviderAdapter, ProviderResponse, execute_with_timing, extract_results_list, fallback_text,
    fallback_url, normalize_result,
};

#[derive(Clone)]
pub struct MetasoAdapter {
    config: ProviderConfig,
    client: ProxyAwareHttpClient,
}

impl MetasoAdapter {
    pub fn new(config: ProviderConfig, client: ProxyAwareHttpClient) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl ProviderAdapter for MetasoAdapter {
    fn id(&self) -> &'static str {
        self.config.id
    }

    fn rpm(&self) -> f64 {
        self.config.rpm
    }

    fn supports(&self, search_type: &SearchType) -> bool {
        matches!(search_type, SearchType::Web | SearchType::Academic)
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
                "/api/search"
            );
            let body = json!({
                "query": request.query,
                "limit": request.options.normalized_limit(),
                "scope": request.options.operation.clone().unwrap_or_else(|| "webpage".to_string())
            });

            let response = self
                .client
                .request(Method::POST, &url)?
                .json(&body)
                .send()
                .await?
                .error_for_status()?;

            let payload = response.json::<serde_json::Value>().await?;
            Ok(extract_results_list(&payload)
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    normalize_result(
                        self.id(),
                        format!("metaso-{index}"),
                        fallback_text(&item, &["title", "name"]),
                        fallback_url(&item, &["url", "link"]),
                        fallback_text(&item, &["snippet", "summary", "content"]),
                        1.0_f64 - index as f64 * 0.01_f64,
                        serde_json::json!(item),
                    )
                })
                .collect())
        })
        .await
    }
}
