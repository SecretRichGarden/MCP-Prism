use async_trait::async_trait;
use reqwest::Method;
use serde::Deserialize;

use crate::{
    config::ProviderConfig,
    error::AppResult,
    http_client::ProxyAwareHttpClient,
    models::{SearchType, UnifiedSearchRequest},
};

use super::{ProviderAdapter, ProviderResponse, execute_with_timing, normalize_result};

#[derive(Clone)]
pub struct DataGovAdapter {
    config: ProviderConfig,
    client: ProxyAwareHttpClient,
}

impl DataGovAdapter {
    pub fn new(config: ProviderConfig, client: ProxyAwareHttpClient) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl ProviderAdapter for DataGovAdapter {
    fn id(&self) -> &'static str {
        self.config.id
    }

    fn rpm(&self) -> f64 {
        self.config.rpm
    }

    fn supports(&self, search_type: &SearchType) -> bool {
        *search_type == SearchType::Government
    }

    async fn execute(&self, request: &UnifiedSearchRequest) -> AppResult<ProviderResponse> {
        execute_with_timing(self.id(), || async {
            let url = format!(
                "{}{}",
                self.config.base_url.as_deref().unwrap_or_default().trim_end_matches('/'),
                "/package_search"
            );
            let response = self
                .client
                .request(Method::GET, &url)?
                .query(&[
                    ("q", request.query.as_str()),
                    ("rows", &request.options.normalized_limit().to_string()),
                ])
                .send()
                .await?
                .error_for_status()?;
            let payload: DataGovResponse = response.json().await?;
            Ok(payload
                .result
                .results
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    normalize_result(
                        self.id(),
                        item.id,
                        item.title,
                        Some(format!("https://catalog.data.gov/dataset/{}", item.name)),
                        item.notes.unwrap_or_default(),
                        1.0_f64 - index as f64 * 0.01_f64,
                        serde_json::json!({"organization": item.organization.and_then(|org| org.title)}),
                    )
                })
                .collect())
        })
        .await
    }
}

#[derive(Debug, Deserialize)]
struct DataGovResponse {
    result: DataGovResult,
}

#[derive(Debug, Deserialize)]
struct DataGovResult {
    #[serde(default)]
    results: Vec<DataGovItem>,
}

#[derive(Debug, Deserialize)]
struct DataGovItem {
    id: String,
    name: String,
    title: String,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    organization: Option<DataGovOrganization>,
}

#[derive(Debug, Deserialize)]
struct DataGovOrganization {
    #[serde(default)]
    title: Option<String>,
}
