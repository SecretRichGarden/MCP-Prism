use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SearchType {
    Auto,
    Web,
    News,
    Academic,
    Finance,
    Company,
    Patent,
    Government,
    Maps,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TimeRange {
    Day,
    Week,
    Month,
    Year,
    Custom { start: String, end: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchOptions {
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub time_range: Option<TimeRange>,
    #[serde(default)]
    pub include_raw_content: Option<bool>,
    #[serde(default)]
    pub operation: Option<String>,
    #[serde(default)]
    pub provider_hints: Vec<String>,
    #[serde(default)]
    pub extras: BTreeMap<String, Value>,
}

impl SearchOptions {
    pub fn normalized_limit(&self) -> usize {
        self.limit.unwrap_or(8).clamp(1, 50)
    }

    pub fn include_raw_content(&self) -> bool {
        self.include_raw_content.unwrap_or(false)
    }

    pub fn extra_string(&self, key: &str) -> Option<String> {
        self.extras
            .get(key)
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSearchRequest {
    pub query: String,
    #[serde(default = "default_search_type")]
    pub search_type: SearchType,
    #[serde(default)]
    pub options: SearchOptions,
    #[serde(default)]
    pub client_id: Option<String>,
}

fn default_search_type() -> SearchType {
    SearchType::Auto
}

impl UnifiedSearchRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.query.trim().is_empty() {
            return Err("query must not be empty".to_string());
        }

        if self.query.len() > 500 {
            return Err("query must not exceed 500 characters".to_string());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedResult {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    pub snippet: String,
    pub score: f64,
    pub source: String,
    #[serde(default)]
    pub metadata: Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub name: String,
    pub results_count: usize,
    pub latency_ms: u64,
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub total_results: usize,
    pub sources_used: Vec<SourceInfo>,
    #[serde(default)]
    pub fallback_used: Option<String>,
    pub cached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTiming {
    pub routing_ms: u64,
    pub total_api_ms: u64,
    pub normalization_ms: u64,
    pub total_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchStrategy {
    Parallel,
    Sequential,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouterLine {
    Llm,
    Heuristic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub search_type: SearchType,
    pub primary_sources: Vec<String>,
    pub secondary_sources: Vec<String>,
    pub strategy: DispatchStrategy,
    pub line: RouterLine,
    pub reasoning: String,
    #[serde(default)]
    pub fallback_reason: Option<String>,
    #[serde(default)]
    pub experiment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedResponse {
    pub request_id: String,
    pub query: String,
    pub search_type: SearchType,
    pub decision: RoutingDecision,
    pub meta: ResponseMeta,
    pub results: Vec<UnifiedResult>,
    pub timing: ResponseTiming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCatalogEntry {
    pub id: String,
    pub title: String,
    pub category: String,
    pub available: bool,
    pub base_url: Option<String>,
    pub capabilities: Vec<SearchType>,
    pub requires_api_key: bool,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub status: String,
    pub routing_mode: String,
    pub proxy_mode: String,
    pub providers: Vec<ProviderCatalogEntry>,
}
