use std::hash::{Hash, Hasher};

use reqwest::Method;
use secrecy::ExposeSecret;
use serde::Deserialize;
use serde_json::json;

use crate::{
    config::{AppConfig, ProviderConfig, RouterConfig, RoutingExperiment},
    error::{AppError, AppResult},
    http_client::ProxyAwareHttpClient,
    models::{DispatchStrategy, RouterLine, RoutingDecision, SearchType, UnifiedSearchRequest},
};

use crate::adapters::AdapterRegistry;

pub struct DualLineRouter {
    llm: Option<SmallModelRouter>,
    experiments: Vec<RoutingExperiment>,
}

impl DualLineRouter {
    pub fn new(config: &AppConfig) -> AppResult<Self> {
        let llm = SmallModelRouter::from_config(config)?;
        Ok(Self {
            llm,
            experiments: config.router.experiments.clone(),
        })
    }

    pub async fn plan(
        &self,
        request: &UnifiedSearchRequest,
        registry: &AdapterRegistry,
    ) -> AppResult<RoutingDecision> {
        if let Some(llm) = &self.llm {
            match llm.decide(request, registry).await {
                Ok(decision) => return Ok(decision),
                Err(error) => {
                    let mut decision = heuristic_plan(request, registry, &self.experiments);
                    decision.fallback_reason = Some(error.to_string());
                    return Ok(decision);
                }
            }
        }

        Ok(heuristic_plan(request, registry, &self.experiments))
    }
}

fn heuristic_plan(
    request: &UnifiedSearchRequest,
    registry: &AdapterRegistry,
    experiments: &[RoutingExperiment],
) -> RoutingDecision {
    let search_type = if request.search_type == SearchType::Auto {
        classify_query(&request.query)
    } else {
        request.search_type.clone()
    };

    let available = registry.available_provider_ids();
    let hinted: Vec<String> = request
        .options
        .provider_hints
        .iter()
        .filter(|hint| available.iter().any(|id| id == *hint))
        .cloned()
        .collect();

    if let Some(experiment_decision) =
        experiment_plan(request, registry, experiments, &search_type, &available)
    {
        return experiment_decision;
    }

    let ordered = if !hinted.is_empty() {
        hinted
    } else {
        default_provider_order(&search_type)
            .into_iter()
            .filter(|provider| available.iter().any(|id| id == provider))
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
    };

    let ordered = if ordered.is_empty() {
        available
    } else {
        ordered
    };
    let primary_sources = ordered.iter().take(2).cloned().collect::<Vec<_>>();
    let secondary_sources = ordered.iter().skip(2).take(2).cloned().collect::<Vec<_>>();
    let strategy = if primary_sources.len() > 1 {
        DispatchStrategy::Parallel
    } else {
        DispatchStrategy::Sequential
    };

    RoutingDecision {
        search_type,
        primary_sources,
        secondary_sources,
        strategy,
        line: RouterLine::Heuristic,
        reasoning: "heuristic planner matched query class against currently available providers"
            .to_string(),
        fallback_reason: None,
        experiment: None,
    }
}

fn experiment_plan(
    request: &UnifiedSearchRequest,
    registry: &AdapterRegistry,
    experiments: &[RoutingExperiment],
    search_type: &SearchType,
    available: &[String],
) -> Option<RoutingDecision> {
    let experiment = experiments.iter().find(|experiment| {
        experiment.search_types.is_empty()
            || experiment
                .search_types
                .iter()
                .any(|item| item == search_type)
    })?;

    let hash = stable_hash(&format!(
        "{}:{}:{}",
        request.client_id.as_deref().unwrap_or("anonymous"),
        request.query,
        experiment.name
    ));
    let total_weight = experiment
        .variants
        .iter()
        .map(|variant| u64::from(variant.weight))
        .sum::<u64>()
        .max(1);
    let mut cursor = hash % total_weight;
    let selected = experiment.variants.iter().find(|variant| {
        if cursor < u64::from(variant.weight) {
            true
        } else {
            cursor -= u64::from(variant.weight);
            false
        }
    })?;

    let primary_sources = selected
        .primary_sources
        .iter()
        .filter(|id| available.iter().any(|available_id| available_id == *id))
        .cloned()
        .collect::<Vec<_>>();
    let secondary_sources = selected
        .secondary_sources
        .iter()
        .filter(|id| available.iter().any(|available_id| available_id == *id))
        .cloned()
        .collect::<Vec<_>>();

    if primary_sources.is_empty() && secondary_sources.is_empty() {
        return None;
    }

    let _ = registry;
    Some(RoutingDecision {
        search_type: search_type.clone(),
        primary_sources,
        secondary_sources,
        strategy: selected.strategy.clone(),
        line: RouterLine::Heuristic,
        reasoning: format!(
            "experiment '{}' selected variant '{}'",
            experiment.name, selected.name
        ),
        fallback_reason: None,
        experiment: Some(format!("{}:{}", experiment.name, selected.name)),
    })
}

fn classify_query(query: &str) -> SearchType {
    let normalized = query.to_ascii_lowercase();
    let academic_keywords = [
        "paper", "论文", "study", "journal", "pubmed", "doi", "citation", "research",
    ];
    let finance_keywords = [
        "stock", "finance", "财务", "股票", "market", "earnings", "macro", "fund",
    ];
    let maps_keywords = [
        "route",
        "地图",
        "地址",
        "附近",
        "weather",
        "导航",
        "poi",
        "经纬度",
    ];
    let company_keywords = ["company", "企业", "公司", "工商", "股东"];
    let patent_keywords = ["patent", "专利", "ip", "知识产权"];
    let gov_keywords = ["data.gov", "政府", "census", "fda", "cdc", "开放数据"];
    let news_keywords = ["news", "新闻", "latest", "today", "breaking", "headlines"];

    if academic_keywords
        .iter()
        .any(|item| normalized.contains(item))
    {
        SearchType::Academic
    } else if finance_keywords
        .iter()
        .any(|item| normalized.contains(item))
    {
        SearchType::Finance
    } else if maps_keywords.iter().any(|item| normalized.contains(item)) {
        SearchType::Maps
    } else if company_keywords
        .iter()
        .any(|item| normalized.contains(item))
    {
        SearchType::Company
    } else if patent_keywords.iter().any(|item| normalized.contains(item)) {
        SearchType::Patent
    } else if gov_keywords.iter().any(|item| normalized.contains(item)) {
        SearchType::Government
    } else if news_keywords.iter().any(|item| normalized.contains(item)) {
        SearchType::News
    } else {
        SearchType::Web
    }
}

fn default_provider_order(search_type: &SearchType) -> Vec<&'static str> {
    match search_type {
        SearchType::News => vec!["brave", "tavily", "zhipu", "metaso"],
        SearchType::Academic => vec!["pubmed", "openalex", "tavily"],
        SearchType::Finance => vec!["finance_mcp", "tavily", "zhipu"],
        SearchType::Maps => vec!["amap", "brave"],
        SearchType::Government => vec!["datagov", "openfda", "census", "tavily"],
        SearchType::Company | SearchType::Patent => {
            vec!["github_rest", "github_graphql", "tavily", "zhipu", "brave"]
        }
        SearchType::Auto | SearchType::Web => {
            vec![
                "brave",
                "tavily",
                "github_rest",
                "github_graphql",
                "zhipu",
                "metaso",
            ]
        }
    }
}

struct SmallModelRouter {
    client: ProxyAwareHttpClient,
    base_url: String,
    endpoint: String,
    model: String,
    api_key: secrecy::SecretString,
}

impl SmallModelRouter {
    fn from_config(config: &AppConfig) -> AppResult<Option<Self>> {
        let RouterConfig {
            enable_llm,
            base_url,
            endpoint,
            model,
            api_key,
            experiments: _,
        } = &config.router;

        if !enable_llm {
            return Ok(None);
        }

        let Some(base_url) = base_url.clone() else {
            return Ok(None);
        };
        let Some(model) = model.clone() else {
            return Ok(None);
        };
        let Some(api_key) = api_key.clone() else {
            return Ok(None);
        };

        Ok(Some(Self {
            client: ProxyAwareHttpClient::new(config.server.request_timeout_ms, &config.proxy)?,
            base_url,
            endpoint: endpoint.clone(),
            model,
            api_key,
        }))
    }

    async fn decide(
        &self,
        request: &UnifiedSearchRequest,
        registry: &AdapterRegistry,
    ) -> AppResult<RoutingDecision> {
        let url = format!(
            "{}{}",
            self.base_url.trim_end_matches('/'),
            ensure_leading_slash(&self.endpoint)
        );
        let available = registry
            .catalog()
            .into_iter()
            .filter(|provider| provider.available)
            .collect::<Vec<_>>();

        let prompt = json!({
            "query": request.query,
            "requested_search_type": request.search_type,
            "provider_hints": request.options.provider_hints,
            "providers": available,
            "instruction": "Choose search_type, primary_sources, secondary_sources, strategy (parallel|sequential|hybrid) and reasoning. Return JSON only.",
        });

        let body = json!({
            "model": self.model,
            "temperature": 0,
            "response_format": {"type": "json_object"},
            "messages": [
                {
                    "role": "system",
                    "content": "You are the routing line for MCP Prism. Prefer high signal, low cost provider combinations. Do not select providers not present in the providers list."
                },
                {
                    "role": "user",
                    "content": prompt.to_string()
                }
            ]
        });

        let response = self
            .client
            .request(Method::POST, &url)?
            .bearer_auth(self.api_key.expose_secret())
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let payload: LlmEnvelope = response.json().await?;
        let content = payload
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| AppError::Provider("routing model returned no content".to_string()))?;
        let decision: LlmDecision = serde_json::from_str(&content)
            .map_err(|err| AppError::Provider(format!("invalid routing JSON: {err}")))?;

        let search_type = decision
            .search_type
            .unwrap_or_else(|| classify_query(&request.query));
        let available_ids = registry.available_provider_ids();
        let primary_sources = decision
            .primary_sources
            .into_iter()
            .filter(|id| available_ids.iter().any(|item| item == id))
            .take(2)
            .collect::<Vec<_>>();
        let secondary_sources = decision
            .secondary_sources
            .into_iter()
            .filter(|id| available_ids.iter().any(|item| item == id))
            .take(2)
            .collect::<Vec<_>>();

        if primary_sources.is_empty() && secondary_sources.is_empty() {
            return Err(AppError::Provider(
                "routing model selected no available providers".to_string(),
            ));
        }

        Ok(RoutingDecision {
            search_type,
            primary_sources,
            secondary_sources,
            strategy: decision.strategy.unwrap_or(DispatchStrategy::Parallel),
            line: RouterLine::Llm,
            reasoning: decision
                .reasoning
                .unwrap_or_else(|| "model-based routing".to_string()),
            fallback_reason: None,
            experiment: None,
        })
    }
}

fn stable_hash(input: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

fn ensure_leading_slash(value: &str) -> String {
    if value.starts_with('/') {
        value.to_string()
    } else {
        format!("/{value}")
    }
}

#[derive(Debug, Deserialize)]
struct LlmEnvelope {
    choices: Vec<LlmChoice>,
}

#[derive(Debug, Deserialize)]
struct LlmChoice {
    message: LlmMessage,
}

#[derive(Debug, Deserialize)]
struct LlmMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LlmDecision {
    #[serde(default)]
    search_type: Option<SearchType>,
    #[serde(default)]
    primary_sources: Vec<String>,
    #[serde(default)]
    secondary_sources: Vec<String>,
    #[serde(default)]
    strategy: Option<DispatchStrategy>,
    #[serde(default)]
    reasoning: Option<String>,
}

#[allow(dead_code)]
fn _provider_enabled(provider: &ProviderConfig) -> bool {
    provider.is_available()
}

#[cfg(test)]
mod tests {
    use super::classify_query;
    use crate::models::SearchType;

    #[test]
    fn query_classifier_covers_main_domains() {
        assert_eq!(classify_query("latest ai news"), SearchType::News);
        assert_eq!(
            classify_query("find pubmed paper on diabetes"),
            SearchType::Academic
        );
        assert_eq!(classify_query("阿里巴巴 财务 表现"), SearchType::Finance);
        assert_eq!(classify_query("北京附近咖啡馆"), SearchType::Maps);
    }
}
