use async_trait::async_trait;
use reqwest::Method;
use serde::Deserialize;

use crate::{
    config::ProviderConfig,
    error::{AppError, AppResult},
    http_client::ProxyAwareHttpClient,
    models::{SearchType, UnifiedSearchRequest},
};

use super::{ProviderAdapter, ProviderResponse, execute_with_timing, normalize_result};

#[derive(Clone)]
pub struct PubMedAdapter {
    config: ProviderConfig,
    client: ProxyAwareHttpClient,
}

impl PubMedAdapter {
    pub fn new(config: ProviderConfig, client: ProxyAwareHttpClient) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl ProviderAdapter for PubMedAdapter {
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
            let base_url = self
                .config
                .base_url
                .as_deref()
                .unwrap_or_default()
                .trim_end_matches('/');
            let search_url = format!("{base_url}/esearch.fcgi");
            let summary_url = format!("{base_url}/esummary.fcgi");

            let mut search_builder = self.client.request(Method::GET, &search_url)?.query(&[
                ("db", "pubmed"),
                ("term", request.query.as_str()),
                ("retmode", "json"),
                ("retmax", &request.options.normalized_limit().to_string()),
            ]);
            // NCBI E-utilities 最佳实践: tool + email 标识调用方 (email 配 key 后限流更稳)
            if let Some(email) = self.config.extra_identity.as_deref() {
                search_builder = search_builder.query(&[("email", email), ("tool", "kairos-mcp-hub")]);
            }
            if let Some(api_key) = self.config.api_key.as_ref() {
                search_builder = search_builder.query(&[("api_key", api_key.expose_secret())]);
            }
            let search_response = search_builder.send().await?.error_for_status()?;
            let search_payload: PubMedSearchResponse = search_response.json().await?;
            let ids = search_payload.esearchresult.idlist;
            if ids.is_empty() {
                return Ok(Vec::new());
            }

            let mut summary_builder = self.client.request(Method::GET, &summary_url)?.query(&[
                ("db", "pubmed"),
                ("retmode", "json"),
                ("id", &ids.join(",")),
            ]);
            if let Some(email) = self.config.extra_identity.as_deref() {
                summary_builder = summary_builder.query(&[("email", email), ("tool", "kairos-mcp-hub")]);
            }
            if let Some(api_key) = self.config.api_key.as_ref() {
                summary_builder = summary_builder.query(&[("api_key", api_key.expose_secret())]);
            }
            let summary_response = summary_builder.send().await?.error_for_status()?;
            let payload = summary_response.json::<serde_json::Value>().await?;
            let result = payload
                .get("result")
                .and_then(|value| value.as_object())
                .ok_or_else(|| {
                    AppError::Provider("pubmed summary missing result object".to_string())
                })?;

            Ok(ids
                .into_iter()
                .filter_map(|pmid| {
                    result.get(&pmid).map(|entry| {
                        let title = entry
                            .get("title")
                            .and_then(|value| value.as_str())
                            .unwrap_or_default();
                        let source = entry
                            .get("source")
                            .and_then(|value| value.as_str())
                            .unwrap_or_default();
                        let pubdate = entry
                            .get("pubdate")
                            .and_then(|value| value.as_str())
                            .unwrap_or_default();
                        normalize_result(
                            self.id(),
                            pmid.clone(),
                            title,
                            Some(format!("https://pubmed.ncbi.nlm.nih.gov/{pmid}/")),
                            format!("{source} {pubdate}"),
                            1.0,
                            entry.clone(),
                        )
                    })
                })
                .collect())
        })
        .await
    }
}

use secrecy::ExposeSecret;

#[derive(Debug, Deserialize)]
struct PubMedSearchResponse {
    esearchresult: PubMedSearchResult,
}

#[derive(Debug, Deserialize)]
struct PubMedSearchResult {
    #[serde(default)]
    idlist: Vec<String>,
}
