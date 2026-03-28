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
pub struct BraveAdapter {
    config: ProviderConfig,
    client: ProxyAwareHttpClient,
}

impl BraveAdapter {
    pub fn new(config: ProviderConfig, client: ProxyAwareHttpClient) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl ProviderAdapter for BraveAdapter {
    fn id(&self) -> &'static str {
        self.config.id
    }

    fn rpm(&self) -> f64 {
        self.config.rpm
    }

    fn supports(&self, search_type: &SearchType) -> bool {
        matches!(search_type, SearchType::Web | SearchType::News)
    }

    async fn execute(&self, request: &UnifiedSearchRequest) -> AppResult<ProviderResponse> {
        execute_with_timing(self.id(), || async {
            let endpoint = if request.search_type == SearchType::News {
                "/res/v1/news/search"
            } else {
                "/res/v1/web/search"
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
                .header("X-Subscription-Token", require_api_key(&self.config)?)
                .query(&[
                    ("q", request.query.as_str()),
                    ("count", &request.options.normalized_limit().to_string()),
                ])
                .send()
                .await?
                .error_for_status()?;

            let payload: BraveResponse = response.json().await?;
            let items = payload
                .web
                .map(|section| section.results)
                .or(payload.news.map(|section| section.results))
                .or(payload.results)
                .unwrap_or_default();

            Ok(items
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    normalize_result(
                        self.id(),
                        format!("brave-{index}"),
                        item.title,
                        Some(item.url),
                        item.description.unwrap_or_default(),
                        1.0_f64 - (index as f64 * 0.01_f64),
                        metadata_object(&[(
                            "age",
                            item.age.map(serde_json::Value::String).unwrap_or_default(),
                        )]),
                    )
                })
                .collect())
        })
        .await
    }
}

#[derive(Debug, Deserialize)]
struct BraveResponse {
    #[serde(default)]
    web: Option<BraveSection>,
    #[serde(default)]
    news: Option<BraveSection>,
    #[serde(default)]
    results: Option<Vec<BraveItem>>,
}

#[derive(Debug, Deserialize)]
struct BraveSection {
    #[serde(default)]
    results: Vec<BraveItem>,
}

#[derive(Debug, Deserialize)]
struct BraveItem {
    title: String,
    url: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    age: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::BraveResponse;

    #[test]
    fn parse_brave_web_payload() {
        let payload = serde_json::from_str::<BraveResponse>(
            r#"{"web":{"results":[{"title":"Example","url":"https://example.com","description":"Body"}]}}"#,
        )
        .unwrap();
        assert_eq!(payload.web.unwrap().results.len(), 1);
    }
}
