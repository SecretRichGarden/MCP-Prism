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
    require_api_key,
};

#[derive(Clone)]
pub struct GitHubRestAdapter {
    config: ProviderConfig,
    client: ProxyAwareHttpClient,
}

impl GitHubRestAdapter {
    pub fn new(config: ProviderConfig, client: ProxyAwareHttpClient) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl ProviderAdapter for GitHubRestAdapter {
    fn id(&self) -> &'static str {
        self.config.id
    }

    fn rpm(&self) -> f64 {
        self.config.rpm
    }

    fn supports(&self, search_type: &SearchType) -> bool {
        matches!(search_type, SearchType::Web | SearchType::Company)
    }

    async fn execute(&self, request: &UnifiedSearchRequest) -> AppResult<ProviderResponse> {
        execute_with_timing(self.id(), || async {
            let operation = request
                .options
                .operation
                .as_deref()
                .unwrap_or("repositories");
            let endpoint = match operation {
                "issues" => "/search/issues",
                "users" => "/search/users",
                "code" => "/search/code",
                _ => "/search/repositories",
            };
            let url = format!(
                "{}{}",
                self.config
                    .base_url
                    .as_deref()
                    .unwrap_or_default()
                    .trim_end_matches('/'),
                endpoint
            );
            let response = self
                .client
                .request(Method::GET, &url)?
                .header("User-Agent", "mcp-prism")
                .bearer_auth(require_api_key(&self.config)?)
                .query(&[
                    ("q", request.query.as_str()),
                    ("per_page", &request.options.normalized_limit().to_string()),
                ])
                .send()
                .await?
                .error_for_status()?;

            let payload: GitHubSearchResponse = response.json().await?;
            Ok(payload
                .items
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    normalize_result(
                        self.id(),
                        item.id
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| format!("github-{index}")),
                        item.full_name
                            .clone()
                            .or(item.login.clone())
                            .unwrap_or(item.name.clone().unwrap_or_default()),
                        item.html_url.clone(),
                        item.description
                            .clone()
                            .or(item.body.clone())
                            .unwrap_or_default(),
                        1.0_f64 - index as f64 * 0.01_f64,
                        metadata_object(&[
                            (
                                "stars",
                                item.stargazers_count
                                    .map(serde_json::Value::from)
                                    .unwrap_or_default(),
                            ),
                            (
                                "language",
                                item.language
                                    .map(serde_json::Value::String)
                                    .unwrap_or_default(),
                            ),
                        ]),
                    )
                })
                .collect())
        })
        .await
    }
}

#[derive(Debug, Deserialize)]
struct GitHubSearchResponse {
    #[serde(default)]
    items: Vec<GitHubItem>,
}

#[derive(Debug, Deserialize)]
struct GitHubItem {
    #[serde(default)]
    id: Option<u64>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    full_name: Option<String>,
    #[serde(default)]
    login: Option<String>,
    #[serde(default)]
    html_url: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    stargazers_count: Option<u64>,
    #[serde(default)]
    language: Option<String>,
}
