use async_trait::async_trait;
use reqwest::Method;
use serde_json::Value;

use crate::{
    config::ProviderConfig,
    error::{AppError, AppResult},
    http_client::ProxyAwareHttpClient,
    models::{SearchType, UnifiedSearchRequest},
};

use super::{ProviderAdapter, ProviderResponse, execute_with_timing, normalize_result};

#[derive(Clone)]
pub struct CensusAdapter {
    config: ProviderConfig,
    client: ProxyAwareHttpClient,
}

impl CensusAdapter {
    pub fn new(config: ProviderConfig, client: ProxyAwareHttpClient) -> Self {
        Self { config, client }
    }
}

#[async_trait]
impl ProviderAdapter for CensusAdapter {
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
            let year = request
                .options
                .extra_string("year")
                .unwrap_or_else(|| "2022".to_string());
            let dataset = request
                .options
                .extra_string("dataset")
                .unwrap_or_else(|| "acs/acs5/profile".to_string());
            let fields = request
                .options
                .extra_string("get")
                .unwrap_or_else(|| "NAME".to_string());
            let geo = request
                .options
                .extra_string("for")
                .unwrap_or_else(|| "us:1".to_string());
            let url = format!(
                "{}/{year}/{dataset}",
                self.config
                    .base_url
                    .as_deref()
                    .unwrap_or_default()
                    .trim_end_matches('/')
            );
            let mut builder = self
                .client
                .request(Method::GET, &url)?
                .query(&[("get", fields.as_str()), ("for", geo.as_str())]);
            if let Some(key) = self.config.api_key.as_ref() {
                builder = builder.query(&[("key", key.expose_secret())]);
            }
            let response = builder.send().await?.error_for_status()?;
            let payload = response.json::<Value>().await?;
            let rows = payload
                .as_array()
                .ok_or_else(|| AppError::Provider("census response is not tabular".to_string()))?;

            if rows.len() <= 1 {
                return Ok(vec![]);
            }

            let headers = rows
                .first()
                .and_then(|row| row.as_array())
                .cloned()
                .unwrap_or_default();
            let mut results = Vec::new();
            for (index, row) in rows.iter().skip(1).enumerate() {
                let values = row.as_array().cloned().unwrap_or_default();
                let mut object = serde_json::Map::new();
                for (header, value) in headers.iter().zip(values.iter()) {
                    if let Some(header) = header.as_str() {
                        object.insert(header.to_string(), value.clone());
                    }
                }
                let name = object
                    .get("NAME")
                    .and_then(|value| value.as_str())
                    .unwrap_or("Census row")
                    .to_string();
                let metadata = Value::Object(object.clone());
                results.push(normalize_result(
                    self.id(),
                    format!("census-{index}"),
                    name,
                    None,
                    metadata.to_string(),
                    1.0_f64 - index as f64 * 0.01_f64,
                    metadata,
                ));
            }

            Ok(results)
        })
        .await
    }
}

use secrecy::ExposeSecret;
