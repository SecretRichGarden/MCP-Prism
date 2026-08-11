use std::{env, fmt, str::FromStr};

use secrecy::SecretString;
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, AppResult},
    models::{DispatchStrategy, ProviderCatalogEntry, SearchType},
};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub router: RouterConfig,
    pub proxy: ProxyConfig,
    pub cache: CacheConfig,
    pub rate_limit: RateLimitConfig,
    pub observability: ObservabilityConfig,
    pub providers: ProviderConfigs,
    pub log_format: LogFormat,
}

impl AppConfig {
    pub fn load() -> AppResult<Self> {
        let _ = dotenvy::from_filename(".env");
        let _ = dotenvy::from_filename(".env.local");

        Ok(Self {
            server: ServerConfig::from_env(),
            auth: AuthConfig::from_env(),
            router: RouterConfig::from_env(),
            proxy: ProxyConfig::from_env(),
            cache: CacheConfig::from_env(),
            rate_limit: RateLimitConfig::from_env(),
            observability: ObservabilityConfig::from_env(),
            providers: ProviderConfigs::from_env(),
            log_format: LogFormat::from_env(),
        })
    }

    pub fn provider_catalog(&self) -> Vec<ProviderCatalogEntry> {
        self.providers.catalog()
    }

    pub fn routing_mode_label(&self) -> String {
        if self.router.enable_llm {
            "dual-line".to_string()
        } else {
            "heuristic-only".to_string()
        }
    }

    pub fn redacted_snapshot(&self) -> ConfigSnapshot {
        ConfigSnapshot {
            host: self.server.host.clone(),
            port: self.server.port,
            require_client_key: self.auth.require_client_key,
            routing_mode: self.routing_mode_label(),
            proxy_mode: self.proxy.mode.to_string(),
            redis_enabled: self.cache.redis_url.is_some(),
            metrics_enabled: self.observability.metrics_enabled,
            providers: self.provider_catalog(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigSnapshot {
    pub host: String,
    pub port: u16,
    pub require_client_key: bool,
    pub routing_mode: String,
    pub proxy_mode: String,
    pub redis_enabled: bool,
    pub metrics_enabled: bool,
    pub providers: Vec<ProviderCatalogEntry>,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout_ms: u64,
    pub sse_keepalive_seconds: u64,
}

impl ServerConfig {
    fn from_env() -> Self {
        Self {
            host: env_first(&["MCP_PRISM_HOST"]).unwrap_or_else(|| "127.0.0.1".to_string()),
            port: env_parse(&["MCP_PRISM_PORT"]).unwrap_or(8787),
            request_timeout_ms: env_parse(&["MCP_PRISM_REQUEST_TIMEOUT_MS"]).unwrap_or(12_000),
            sse_keepalive_seconds: env_parse(&["MCP_PRISM_SSE_KEEPALIVE_SECONDS"]).unwrap_or(15),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub hmac_secret: SecretString,
    pub encryption_key: SecretString,
    pub require_client_key: bool,
    pub trust_stdio_transport: bool,
}

impl AuthConfig {
    fn from_env() -> Self {
        Self {
            hmac_secret: secret_from_aliases(&["MCP_PRISM_HMAC_SECRET"]).unwrap_or_else(|| {
                SecretString::new("local-dev-hmac-secret".to_string().into_boxed_str())
            }),
            encryption_key: secret_from_aliases(&["MCP_PRISM_ENCRYPTION_KEY"]).unwrap_or_else(
                || SecretString::new("local-dev-encryption-key".to_string().into_boxed_str()),
            ),
            require_client_key: env_bool(&["MCP_PRISM_REQUIRE_CLIENT_KEY"]).unwrap_or(false),
            trust_stdio_transport: env_bool(&["MCP_PRISM_TRUST_STDIO_TRANSPORT"]).unwrap_or(true),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyMode {
    System,
    Direct,
    Custom,
}

impl fmt::Display for ProxyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Direct => write!(f, "direct"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl FromStr for ProxyMode {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "system" => Ok(Self::System),
            "direct" => Ok(Self::Direct),
            "custom" => Ok(Self::Custom),
            other => Err(AppError::Config(format!("unknown proxy mode: {other}"))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub mode: ProxyMode,
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub all_proxy: Option<String>,
    pub no_proxy: Vec<String>,
}

impl ProxyConfig {
    fn from_env() -> Self {
        let http_proxy = env_first(&["MCP_PRISM_HTTP_PROXY", "HTTP_PROXY", "http_proxy"]);
        let https_proxy = env_first(&["MCP_PRISM_HTTPS_PROXY", "HTTPS_PROXY", "https_proxy"]);
        let all_proxy = env_first(&["MCP_PRISM_ALL_PROXY", "ALL_PROXY", "all_proxy"]);
        let mode = env_first(&["MCP_PRISM_PROXY_MODE"])
            .and_then(|raw| ProxyMode::from_str(&raw).ok())
            .unwrap_or_else(|| {
                if http_proxy.is_some() || https_proxy.is_some() || all_proxy.is_some() {
                    ProxyMode::Custom
                } else {
                    ProxyMode::System
                }
            });

        Self {
            mode,
            http_proxy,
            https_proxy,
            all_proxy,
            no_proxy: env_first(&["MCP_PRISM_NO_PROXY", "NO_PROXY", "no_proxy"])
                .map(|raw| {
                    raw.split(',')
                        .map(str::trim)
                        .filter(|item| !item.is_empty())
                        .map(ToOwned::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub enable_llm: bool,
    pub base_url: Option<String>,
    pub endpoint: String,
    pub model: Option<String>,
    pub api_key: Option<SecretString>,
    pub experiments: Vec<RoutingExperiment>,
}

impl RouterConfig {
    fn from_env() -> Self {
        Self {
            enable_llm: env_bool(&["MCP_PRISM_ROUTER_ENABLE_LLM"]).unwrap_or(false),
            base_url: env_first(&["MCP_PRISM_ROUTER_BASE_URL"]),
            endpoint: env_first(&["MCP_PRISM_ROUTER_ENDPOINT"])
                .unwrap_or_else(|| "/chat/completions".to_string()),
            model: env_first(&["MCP_PRISM_ROUTER_MODEL"]),
            api_key: secret_from_aliases(&["MCP_PRISM_ROUTER_API_KEY"]),
            experiments: env_first(&["MCP_PRISM_ROUTER_EXPERIMENTS"])
                .and_then(|raw| serde_json::from_str::<Vec<RoutingExperiment>>(&raw).ok())
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoutingExperiment {
    pub name: String,
    #[serde(default)]
    pub search_types: Vec<SearchType>,
    pub variants: Vec<RoutingVariant>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoutingVariant {
    pub name: String,
    pub weight: u16,
    #[serde(default)]
    pub primary_sources: Vec<String>,
    #[serde(default)]
    pub secondary_sources: Vec<String>,
    #[serde(default = "default_dispatch_strategy")]
    pub strategy: DispatchStrategy,
}

fn default_dispatch_strategy() -> DispatchStrategy {
    DispatchStrategy::Parallel
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub ttl_seconds: u64,
    pub redis_url: Option<String>,
    pub key_prefix: String,
    pub realtime_ttl_seconds: u64,
    pub daily_ttl_seconds: u64,
    pub monthly_ttl_seconds: u64,
    pub yearly_ttl_seconds: u64,
}

impl CacheConfig {
    fn from_env() -> Self {
        Self {
            ttl_seconds: env_parse(&["MCP_PRISM_CACHE_TTL_SECONDS"]).unwrap_or(60),
            redis_url: env_first(&["REDIS_URL", "MCP_PRISM_REDIS_URL"]),
            key_prefix: env_first(&["MCP_PRISM_CACHE_PREFIX"])
                .unwrap_or_else(|| "mcp-prism".to_string()),
            realtime_ttl_seconds: env_parse(&["MCP_PRISM_CACHE_TTL_REALTIME_SECONDS"])
                .unwrap_or(300),
            daily_ttl_seconds: env_parse(&["MCP_PRISM_CACHE_TTL_DAILY_SECONDS"]).unwrap_or(3600),
            monthly_ttl_seconds: env_parse(&["MCP_PRISM_CACHE_TTL_MONTHLY_SECONDS"])
                .unwrap_or(86_400),
            yearly_ttl_seconds: env_parse(&["MCP_PRISM_CACHE_TTL_YEARLY_SECONDS"])
                .unwrap_or(2_592_000),
        }
    }

    pub fn ttl_for(&self, search_type: &SearchType) -> u64 {
        match search_type {
            SearchType::Maps => self.realtime_ttl_seconds,
            SearchType::Finance => self.daily_ttl_seconds,
            SearchType::Government | SearchType::Academic => self.monthly_ttl_seconds,
            SearchType::Company | SearchType::Patent => self.yearly_ttl_seconds,
            SearchType::Auto | SearchType::Web | SearchType::News => self.ttl_seconds,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub client_capacity: f64,
    pub client_refill_per_second: f64,
    pub provider_capacity_default: f64,
    pub provider_refill_per_second_default: f64,
}

impl RateLimitConfig {
    fn from_env() -> Self {
        let client_rpm = env_parse::<f64>(&["MCP_PRISM_CLIENT_RPM"]).unwrap_or(120.0);
        let provider_rpm = env_parse::<f64>(&["MCP_PRISM_PROVIDER_DEFAULT_RPM"]).unwrap_or(60.0);
        Self {
            client_capacity: client_rpm.max(1.0),
            client_refill_per_second: (client_rpm / 60.0).max(0.1),
            provider_capacity_default: provider_rpm.max(1.0),
            provider_refill_per_second_default: (provider_rpm / 60.0).max(0.1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    pub metrics_enabled: bool,
}

impl ObservabilityConfig {
    fn from_env() -> Self {
        Self {
            metrics_enabled: env_bool(&["MCP_PRISM_ENABLE_METRICS"]).unwrap_or(true),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderConfigs {
    pub brave: ProviderConfig,
    pub tavily: ProviderConfig,
    pub zhipu: ProviderConfig,
    pub metaso: ProviderConfig,
    pub pubmed: ProviderConfig,
    pub openalex: ProviderConfig,
    pub amap: ProviderConfig,
    pub finance_mcp: ProviderConfig,
    pub github_rest: ProviderConfig,
    pub github_graphql: ProviderConfig,
    pub datagov: ProviderConfig,
    pub openfda: ProviderConfig,
    pub census: ProviderConfig,
}

impl ProviderConfigs {
    fn from_env() -> Self {
        Self {
            brave: ProviderConfig::new(
                "brave",
                "Brave Search",
                "search",
                true,
                vec![SearchType::Web, SearchType::News],
                env_first(&["BRAVE_BASE_URL"]).or(Some("https://api.search.brave.com".to_string())),
                secret_from_aliases(&["BRAVE_API_KEY", "BRAVE_SEARCH_API_KEY"]),
                Some(
                    "Official endpoint group: /res/v1/web/search, /res/v1/news/search".to_string(),
                ),
            ),
            tavily: ProviderConfig::new(
                "tavily",
                "Tavily",
                "search",
                true,
                vec![SearchType::Web, SearchType::News, SearchType::Finance],
                env_first(&["TAVILY_BASE_URL"]).or(Some("https://api.tavily.com".to_string())),
                secret_from_aliases(&["TAVILY_API_KEY"]),
                Some(
                    "Official endpoint group: /search, /extract, /crawl, /map, /research"
                        .to_string(),
                ),
            ),
            zhipu: ProviderConfig::new(
                "zhipu",
                "Zhipu Web Search",
                "search",
                true,
                vec![SearchType::Web, SearchType::News, SearchType::Finance],
                env_first(&["ZHIPU_BASE_URL"]).or(Some("https://open.bigmodel.cn".to_string())),
                secret_from_aliases(&["ZHIPU_API_KEY", "BIGMODEL_API_KEY"]),
                Some("Official endpoint: /api/paas/v4/web_search".to_string()),
            ),
            metaso: ProviderConfig::new(
                "metaso",
                "Metaso Search",
                "search",
                true,
                vec![SearchType::Web, SearchType::Academic],
                env_first(&["METASO_BASE_URL"]).or(Some("https://metaso.cn".to_string())),
                secret_from_aliases(&["METASO_API_KEY"]),
                Some(
                    "Experimental module; upstream documentation is sparse and fully configurable."
                        .to_string(),
                ),
            ),
            pubmed: ProviderConfig::new(
                "pubmed",
                "PubMed",
                "academic",
                false,
                vec![SearchType::Academic],
                env_first(&["PUBMED_BASE_URL", "NCBI_BASE_URL"]).or(Some(
                    "https://eutils.ncbi.nlm.nih.gov/entrez/eutils".to_string(),
                )),
                secret_from_aliases(&["NCBI_API_KEY", "PUBMED_API_KEY"]),
                Some("Official E-utilities flow: esearch.fcgi + esummary.fcgi".to_string()),
            ),
            openalex: ProviderConfig::new(
                "openalex",
                "OpenAlex",
                "academic",
                false,
                vec![SearchType::Academic],
                env_first(&["OPENALEX_BASE_URL"]).or(Some("https://api.openalex.org".to_string())),
                secret_from_aliases(&["OPENALEX_API_KEY"]),
                Some(
                    "Public API; OPENALEX_EMAIL can be supplied for polite pool access."
                        .to_string(),
                ),
            ),
            amap: ProviderConfig::new(
                "amap",
                "Amap Web Service",
                "maps",
                true,
                vec![SearchType::Maps],
                env_first(&["AMAP_BASE_URL"]).or(Some("https://restapi.amap.com/v3".to_string())),
                secret_from_aliases(&["AMAP_API_KEY", "GAODE_API_KEY"]),
                Some(
                    "Primary operations: place/text, geocode/geo, weather/weatherInfo, distance"
                        .to_string(),
                ),
            ),
            finance_mcp: ProviderConfig::new(
                "finance_mcp",
                "FinanceMCP Bridge",
                "finance",
                false,
                vec![SearchType::Finance],
                env_first(&["FINANCE_MCP_URL"]),
                secret_from_aliases(&["FINANCE_MCP_TOKEN", "TUSHARE_TOKEN"]),
                Some("Bridge module for upstream HTTP/MCP-compatible finance service.".to_string()),
            ),
            github_rest: ProviderConfig::new(
                "github_rest",
                "GitHub REST Search",
                "business",
                true,
                vec![SearchType::Web, SearchType::Company],
                env_first(&["GITHUB_BASE_URL"]).or(Some("https://api.github.com".to_string())),
                secret_from_aliases(&["GITHUB_TOKEN", "GITHUB_PAT"]),
                Some("REST search endpoints for repositories, code, issues and users.".to_string()),
            ),
            github_graphql: ProviderConfig::new(
                "github_graphql",
                "GitHub GraphQL Search",
                "business",
                true,
                vec![SearchType::Web, SearchType::Company],
                env_first(&["GITHUB_GRAPHQL_URL"])
                    .or(Some("https://api.github.com/graphql".to_string())),
                secret_from_aliases(&["GITHUB_TOKEN", "GITHUB_PAT"]),
                Some("GraphQL search endpoint for repository and user research.".to_string()),
            ),
            datagov: ProviderConfig::new(
                "datagov",
                "Data.gov CKAN",
                "government",
                false,
                vec![SearchType::Government],
                env_first(&["DATAGOV_BASE_URL"])
                    .or(Some("https://catalog.data.gov/api/3/action".to_string())),
                secret_from_aliases(&["DATAGOV_API_KEY"]),
                Some("CKAN metadata search via package_search.".to_string()),
            ),
            openfda: ProviderConfig::new(
                "openfda",
                "openFDA",
                "government",
                false,
                vec![SearchType::Government, SearchType::Academic],
                env_first(&["OPENFDA_BASE_URL"]).or(Some("https://api.fda.gov".to_string())),
                secret_from_aliases(&["OPENFDA_API_KEY"]),
                Some("Elasticsearch-style public datasets for drug, device and food.".to_string()),
            ),
            census: ProviderConfig::new(
                "census",
                "U.S. Census Bureau",
                "government",
                false,
                vec![SearchType::Government],
                env_first(&["CENSUS_BASE_URL"]).or(Some("https://api.census.gov/data".to_string())),
                secret_from_aliases(&["CENSUS_API_KEY"]),
                Some("Public tabular datasets via /{year}/{dataset} endpoints.".to_string()),
            ),
        }
    }

    pub fn catalog(&self) -> Vec<ProviderCatalogEntry> {
        [
            &self.brave,
            &self.tavily,
            &self.zhipu,
            &self.metaso,
            &self.pubmed,
            &self.openalex,
            &self.amap,
            &self.finance_mcp,
            &self.github_rest,
            &self.github_graphql,
            &self.datagov,
            &self.openfda,
            &self.census,
        ]
        .into_iter()
        .map(ProviderConfig::as_catalog_entry)
        .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub id: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub requires_api_key: bool,
    pub capabilities: Vec<SearchType>,
    pub base_url: Option<String>,
    pub api_key: Option<SecretString>,
    pub enabled: bool,
    pub notes: Option<String>,
    pub extra_identity: Option<String>,
    pub rpm: f64,
}

impl ProviderConfig {
    #[allow(clippy::too_many_arguments)]
    fn new(
        id: &'static str,
        title: &'static str,
        category: &'static str,
        requires_api_key: bool,
        capabilities: Vec<SearchType>,
        base_url: Option<String>,
        api_key: Option<SecretString>,
        notes: Option<String>,
    ) -> Self {
        let enabled = env_bool(&[&format!(
            "MCP_PRISM_PROVIDER_{}_ENABLED",
            id.to_ascii_uppercase()
        )])
        .unwrap_or(true);
        let extra_identity = match id {
            "openalex" => env_first(&["OPENALEX_EMAIL"]),
            "pubmed" => env_first(&["NCBI_EMAIL", "PUBMED_EMAIL"]),
            _ => None,
        };
        let rpm = env_parse(&[&format!(
            "MCP_PRISM_PROVIDER_{}_RPM",
            id.to_ascii_uppercase()
        )])
        .unwrap_or(60.0);

        Self {
            id,
            title,
            category,
            requires_api_key,
            capabilities,
            base_url,
            api_key,
            enabled,
            notes,
            extra_identity,
            rpm,
        }
    }

    pub fn is_available(&self) -> bool {
        if !self.enabled {
            return false;
        }

        let has_base = self.base_url.is_some();
        let has_auth = if self.requires_api_key {
            self.api_key.is_some()
        } else {
            self.base_url.is_some()
        };

        has_base && has_auth
    }

    pub fn as_catalog_entry(&self) -> ProviderCatalogEntry {
        ProviderCatalogEntry {
            id: self.id.to_string(),
            title: self.title.to_string(),
            category: self.category.to_string(),
            available: self.is_available(),
            base_url: self.base_url.clone(),
            capabilities: self.capabilities.clone(),
            requires_api_key: self.requires_api_key,
            notes: self.notes.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LogFormat {
    Json,
    Pretty,
}

impl LogFormat {
    fn from_env() -> Self {
        match env_first(&["MCP_PRISM_LOG_FORMAT"])
            .unwrap_or_else(|| "pretty".to_string())
            .as_str()
        {
            "json" => Self::Json,
            _ => Self::Pretty,
        }
    }
}

pub fn env_first(keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| env::var(key).ok())
        .filter(|value| !value.trim().is_empty())
}

pub fn secret_from_aliases(keys: &[&str]) -> Option<SecretString> {
    env_first(keys).map(|value| SecretString::new(value.into_boxed_str()))
}

pub fn env_bool(keys: &[&str]) -> Option<bool> {
    env_first(keys).map(|raw| matches!(raw.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
}

pub fn env_parse<T: FromStr>(keys: &[&str]) -> Option<T> {
    env_first(keys).and_then(|raw| raw.parse::<T>().ok())
}
