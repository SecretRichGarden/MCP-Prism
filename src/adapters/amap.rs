use async_trait::async_trait;
use reqwest::Method;

use crate::{
    config::ProviderConfig,
    error::AppResult,
    http_client::ProxyAwareHttpClient,
    models::{SearchType, UnifiedSearchRequest},
};

use super::{
    ProviderAdapter, ProviderResponse, execute_with_timing, extract_results_list, fallback_text,
    normalize_result, require_api_key,
};

#[derive(Clone)]
pub struct AmapAdapter {
    config: ProviderConfig,
    client: ProxyAwareHttpClient,
}

impl AmapAdapter {
    pub fn new(config: ProviderConfig, client: ProxyAwareHttpClient) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl ProviderAdapter for AmapAdapter {
    fn id(&self) -> &'static str {
        self.config.id
    }

    fn rpm(&self) -> f64 {
        self.config.rpm
    }

    fn supports(&self, search_type: &SearchType) -> bool {
        *search_type == SearchType::Maps
    }

    async fn execute(&self, request: &UnifiedSearchRequest) -> AppResult<ProviderResponse> {
        execute_with_timing(self.id(), || async {
            let operation = request
                .options
                .operation
                .as_deref()
                .unwrap_or("text_search");
            let endpoint = match operation {
                "geocode" => "/geocode/geo",
                "weather" => "/weather/weatherInfo",
                "distance" => "/distance",
                _ => "/place/text",
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

            let mut query = vec![
                ("key", require_api_key(&self.config)?),
                ("keywords", request.query.clone()),
            ];
            if let Some(city) = request.options.extra_string("city") {
                query.push(("city", city));
            }
            if operation == "geocode" {
                query.retain(|(key, _)| *key != "keywords");
                query.push(("address", request.query.clone()));
            }
            if operation == "weather" {
                query.retain(|(key, _)| *key != "keywords");
                query.push(("city", request.query.clone()));
            }
            if operation == "distance" {
                query.retain(|(key, _)| *key != "keywords");
                if let Some(origin) = request.options.extra_string("origin") {
                    query.push(("origins", origin));
                }
                if let Some(destination) = request.options.extra_string("destination") {
                    query.push(("destination", destination));
                }
            }

            let response = self
                .client
                .request(Method::GET, &url)?
                .query(&query)
                .send()
                .await?
                .error_for_status()?;
            let payload = response.json::<serde_json::Value>().await?;
            let results = extract_results_list(&payload)
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    normalize_result(
                        self.id(),
                        format!("amap-{index}"),
                        fallback_text(&item, &["name", "formatted_address", "weather"]),
                        None,
                        fallback_text(
                            &item,
                            &["address", "location", "province", "city", "distance"],
                        ),
                        1.0_f64 - index as f64 * 0.01_f64,
                        serde_json::json!(item),
                    )
                })
                .collect::<Vec<_>>();

            Ok(if results.is_empty() {
                vec![normalize_result(
                    self.id(),
                    "amap-fallback",
                    request.query.clone(),
                    None,
                    payload.to_string(),
                    1.0,
                    payload,
                )]
            } else {
                results
            })
        })
        .await
    }
}
