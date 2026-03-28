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
pub struct FinanceMcpAdapter {
    config: ProviderConfig,
    client: ProxyAwareHttpClient,
}

impl FinanceMcpAdapter {
    pub fn new(config: ProviderConfig, client: ProxyAwareHttpClient) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl ProviderAdapter for FinanceMcpAdapter {
    fn id(&self) -> &'static str {
        self.config.id
    }

    fn rpm(&self) -> f64 {
        self.config.rpm
    }

    fn supports(&self, search_type: &SearchType) -> bool {
        *search_type == SearchType::Finance
    }

    async fn execute(&self, request: &UnifiedSearchRequest) -> AppResult<ProviderResponse> {
        execute_with_timing(self.id(), || async {
            let url = self
                .config
                .base_url
                .as_deref()
                .unwrap_or_default()
                .to_string();
            let tool = request
                .options
                .operation
                .clone()
                .unwrap_or_else(|| "company_performance".to_string());
            let body = json!({
                "tool": tool,
                "arguments": {
                    "query": request.query,
                    "limit": request.options.normalized_limit(),
                    "extras": request.options.extras
                }
            });

            let mut builder = self.client.request(Method::POST, &url)?.json(&body);
            if let Some(token) = self.config.api_key.as_ref() {
                builder = builder.bearer_auth(token.expose_secret());
            }
            let response = builder.send().await?.error_for_status()?;
            let payload = response.json::<serde_json::Value>().await?;

            let items = extract_results_list(&payload);
            Ok(if items.is_empty() {
                vec![normalize_result(
                    self.id(),
                    "finance-bridge",
                    request.query.clone(),
                    None,
                    payload.to_string(),
                    1.0,
                    payload,
                )]
            } else {
                items
                    .into_iter()
                    .enumerate()
                    .map(|(index, item)| {
                        normalize_result(
                            self.id(),
                            format!("finance-{index}"),
                            fallback_text(&item, &["title", "name", "symbol"]),
                            fallback_url(&item, &["url"]),
                            fallback_text(&item, &["snippet", "summary", "content", "description"]),
                            1.0_f64 - index as f64 * 0.01_f64,
                            serde_json::json!(item),
                        )
                    })
                    .collect()
            })
        })
        .await
    }
}

use secrecy::ExposeSecret;
