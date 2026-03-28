use async_trait::async_trait;
use reqwest::Method;
use serde::Deserialize;

use crate::{
    config::ProviderConfig,
    error::AppResult,
    http_client::ProxyAwareHttpClient,
    models::{SearchType, UnifiedSearchRequest},
};

use super::{
    ProviderAdapter, ProviderResponse, execute_with_timing, metadata_object, normalize_result,
};

#[derive(Clone)]
pub struct OpenAlexAdapter {
    config: ProviderConfig,
    client: ProxyAwareHttpClient,
}

impl OpenAlexAdapter {
    pub fn new(config: ProviderConfig, client: ProxyAwareHttpClient) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl ProviderAdapter for OpenAlexAdapter {
    fn id(&self) -> &'static str {
        self.config.id
    }

    fn rpm(&self) -> f64 {
        self.config.rpm
    }

    fn supports(&self, search_type: &SearchType) -> bool {
        *search_type == SearchType::Academic
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
                "/works"
            );
            let mut builder = self.client.request(Method::GET, &url)?.query(&[
                ("search", request.query.as_str()),
                ("per-page", &request.options.normalized_limit().to_string()),
            ]);
            if let Some(email) = self.config.extra_identity.as_deref() {
                builder = builder.query(&[("mailto", email)]);
            }
            let response = builder.send().await?.error_for_status()?;
            let payload: OpenAlexResponse = response.json().await?;

            Ok(payload
                .results
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    let publication_date = item.publication_date.clone().unwrap_or_default();
                    normalize_result(
                        self.id(),
                        item.id.clone(),
                        item.display_name,
                        item.primary_location
                            .and_then(|location| location.landing_page_url),
                        item.abstract_inverted_index
                            .map(|_| "abstract metadata available".to_string())
                            .unwrap_or_else(|| publication_date.clone()),
                        1.0_f64 - index as f64 * 0.01_f64,
                        metadata_object(&[(
                            "publication_date",
                            serde_json::Value::String(publication_date),
                        )]),
                    )
                })
                .collect())
        })
        .await
    }
}

#[derive(Debug, Deserialize)]
struct OpenAlexResponse {
    #[serde(default)]
    results: Vec<OpenAlexItem>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexItem {
    id: String,
    display_name: String,
    #[serde(default)]
    primary_location: Option<OpenAlexLocation>,
    #[serde(default)]
    publication_date: Option<String>,
    #[serde(default)]
    abstract_inverted_index: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexLocation {
    #[serde(default)]
    landing_page_url: Option<String>,
}
