use async_trait::async_trait;
use reqwest::Method;
use serde_json::Value;

use crate::{
    config::ProviderConfig,
    error::AppResult,
    http_client::ProxyAwareHttpClient,
    models::{SearchType, UnifiedSearchRequest},
};

use super::{
    ProviderAdapter, ProviderResponse, execute_with_timing, extract_results_list, fallback_text,
    normalize_result,
};

#[derive(Clone)]
pub struct OpenFdaAdapter {
    config: ProviderConfig,
    client: ProxyAwareHttpClient,
}

impl OpenFdaAdapter {
    pub fn new(config: ProviderConfig, client: ProxyAwareHttpClient) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl ProviderAdapter for OpenFdaAdapter {
    fn id(&self) -> &'static str {
        self.config.id
    }

    fn rpm(&self) -> f64 {
        self.config.rpm
    }

    fn supports(&self, search_type: &SearchType) -> bool {
        matches!(search_type, SearchType::Government | SearchType::Academic)
    }

    async fn execute(&self, request: &UnifiedSearchRequest) -> AppResult<ProviderResponse> {
        execute_with_timing(self.id(), || async {
            let dataset = request
                .options
                .extra_string("dataset")
                .unwrap_or_else(|| "drug/label".to_string());
            let url = format!(
                "{}/{dataset}.json",
                self.config
                    .base_url
                    .as_deref()
                    .unwrap_or_default()
                    .trim_end_matches('/')
            );
            let response = self
                .client
                .request(Method::GET, &url)?
                .query(&[
                    ("search", request.query.as_str()),
                    ("limit", &request.options.normalized_limit().to_string()),
                ])
                .send()
                .await?
                .error_for_status()?;
            let payload = response.json::<Value>().await?;
            Ok(extract_results_list(&payload)
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    normalize_result(
                        self.id(),
                        format!("openfda-{index}"),
                        fallback_text(&item, &["manufacturer_name", "purpose", "brand_name"]),
                        None,
                        fallback_text(&item, &["description", "indications_and_usage", "warnings"]),
                        1.0_f64 - index as f64 * 0.01_f64,
                        serde_json::json!(item),
                    )
                })
                .collect())
        })
        .await
    }
}
