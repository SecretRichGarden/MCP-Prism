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
pub struct GitHubGraphqlAdapter {
    config: ProviderConfig,
    client: ProxyAwareHttpClient,
}

impl GitHubGraphqlAdapter {
    pub fn new(config: ProviderConfig, client: ProxyAwareHttpClient) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl ProviderAdapter for GitHubGraphqlAdapter {
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
            let url = self
                .config
                .base_url
                .as_deref()
                .unwrap_or_default()
                .to_string();
            let body = GitHubGraphqlRequest {
                query: r#"query Search($query: String!, $first: Int!) {
                  search(query: $query, type: REPOSITORY, first: $first) {
                    nodes {
                      ... on Repository {
                        id
                        nameWithOwner
                        description
                        url
                        stargazerCount
                      }
                    }
                  }
                }"#
                .to_string(),
                variables: GitHubGraphqlVariables {
                    query: request.query.clone(),
                    first: request.options.normalized_limit(),
                },
            };

            let response = self
                .client
                .request(Method::POST, &url)?
                .header("User-Agent", "mcp-prism")
                .bearer_auth(require_api_key(&self.config)?)
                .json(&body)
                .send()
                .await?
                .error_for_status()?;
            let payload: GitHubGraphqlResponse = response.json().await?;
            let nodes = payload.data.search.nodes;

            Ok(nodes
                .into_iter()
                .enumerate()
                .map(|(index, node)| {
                    normalize_result(
                        self.id(),
                        node.id,
                        node.name_with_owner,
                        Some(node.url),
                        node.description.unwrap_or_default(),
                        1.0_f64 - index as f64 * 0.01_f64,
                        serde_json::json!({"stars": node.stargazer_count}),
                    )
                })
                .collect())
        })
        .await
    }
}

#[derive(Debug, Serialize)]
struct GitHubGraphqlRequest {
    query: String,
    variables: GitHubGraphqlVariables,
}

#[derive(Debug, Serialize)]
struct GitHubGraphqlVariables {
    query: String,
    first: usize,
}

#[derive(Debug, Deserialize)]
struct GitHubGraphqlResponse {
    data: GitHubGraphqlData,
}

#[derive(Debug, Deserialize)]
struct GitHubGraphqlData {
    search: GitHubGraphqlSearch,
}

#[derive(Debug, Deserialize)]
struct GitHubGraphqlSearch {
    #[serde(default)]
    nodes: Vec<GitHubGraphqlNode>,
}

#[derive(Debug, Deserialize)]
struct GitHubGraphqlNode {
    id: String,
    #[serde(rename = "nameWithOwner")]
    name_with_owner: String,
    #[serde(default)]
    description: Option<String>,
    url: String,
    #[serde(rename = "stargazerCount")]
    stargazer_count: u64,
}
