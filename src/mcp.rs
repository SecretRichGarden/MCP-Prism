use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::models::{ProviderCatalogEntry, SearchType};

pub const CAPABILITY_SUMMARY_URI: &str = "mcp-prism://capability-summary";
pub const CAPABILITY_SUMMARY_ZH_URI: &str = "mcp-prism://capability-summary/zh";
pub const CAPABILITY_SUMMARY_EN_URI: &str = "mcp-prism://capability-summary/en";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId {
    Number(i64),
    String(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: Option<String>,
    pub id: Option<JsonRpcId>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonRpcId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

impl JsonRpcResponse {
    pub fn ok(id: Option<JsonRpcId>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn err(id: Option<JsonRpcId>, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
            }),
        }
    }
}

pub fn tool_definitions(providers: &[ProviderCatalogEntry]) -> Value {
    let mut tools = vec![
        json!({
            "name": "unified_search",
            "description": "Search the currently configured provider pool through one MCP entrypoint.",
            "inputSchema": {
                "type": "object",
                "required": ["query"],
                "properties": {
                    "query": {"type": "string"},
                    "search_type": {"type": "string", "enum": search_type_values()},
                    "options": {
                        "type": "object",
                        "properties": {
                            "limit": {"type": "integer", "minimum": 1, "maximum": 50},
                            "operation": {"type": "string"},
                            "provider_hints": {"type": "array", "items": {"type": "string"}},
                            "language": {"type": "string"},
                            "region": {"type": "string"}
                        }
                    }
                }
            }
        }),
        json!({
            "name": "provider_catalog",
            "description": "Inspect which upstream adapters are available with the current env/config.",
            "inputSchema": {"type": "object", "properties": {}}
        }),
        json!({
            "name": "issue_wrapped_key",
            "description": "Issue an MCP Prism client key for remote consumers.",
            "inputSchema": {
                "type": "object",
                "required": ["client_id"],
                "properties": {
                    "client_id": {"type": "string"},
                    "ttl_seconds": {"type": "integer", "minimum": 60}
                }
            }
        }),
        json!({
            "name": "revoke_wrapped_key",
            "description": "Revoke a wrapped key by token or client id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "token": {"type": "string"},
                    "client_id": {"type": "string"}
                }
            }
        }),
        json!({
            "name": "cache_prewarm",
            "description": "Prewarm cache entries for a batch of unified search requests.",
            "inputSchema": {
                "type": "object",
                "required": ["requests"],
                "properties": {
                    "requests": {"type": "array", "items": {"type": "object"}}
                }
            }
        }),
        json!({
            "name": "cache_invalidate",
            "description": "Invalidate a unified search cache entry by request payload.",
            "inputSchema": {
                "type": "object",
                "required": ["request"],
                "properties": {
                    "request": {"type": "object"}
                }
            }
        }),
    ];

    for provider in providers.iter().filter(|provider| provider.available) {
        tools.push(json!({
            "name": format!("search_{}", provider.id),
            "description": format!("Run unified search constrained to provider '{}'.", provider.id),
            "inputSchema": {
                "type": "object",
                "required": ["query"],
                "properties": {
                    "query": {"type": "string"},
                    "search_type": {"type": "string", "enum": search_type_values()},
                    "options": {"type": "object"}
                }
            }
        }));
    }

    Value::Array(tools)
}

pub fn initialize_instructions(
    providers: &[ProviderCatalogEntry],
    params: &Value,
    accept_language: Option<&str>,
) -> String {
    let grouped = grouped_available_providers(providers);
    let language = detect_summary_language(params, accept_language);
    let focus = detect_summary_focus(params, providers);
    if grouped.is_empty() {
        return match language {
            SummaryLanguage::Zh => "优先用 `unified_search`。当前没有可用上游。".to_string(),
            SummaryLanguage::En => {
                "Use `unified_search` first. No upstream providers are enabled.".to_string()
            }
        };
    }

    let groups = focus
        .iter()
        .filter_map(|category| {
            grouped
                .get(category)
                .map(|items| format_group(category, items.len(), language))
        })
        .take(3)
        .collect::<Vec<_>>()
        .join(match language {
            SummaryLanguage::Zh => "、",
            SummaryLanguage::En => ", ",
        });
    let groups = if groups.is_empty() {
        grouped
            .iter()
            .take(3)
            .map(|(category, items)| format_group(category, items.len(), language))
            .collect::<Vec<_>>()
            .join(match language {
                SummaryLanguage::Zh => "、",
                SummaryLanguage::En => ", ",
            })
    } else {
        groups
    };
    let details_uri = match language {
        SummaryLanguage::Zh => CAPABILITY_SUMMARY_ZH_URI,
        SummaryLanguage::En => CAPABILITY_SUMMARY_EN_URI,
    };

    match language {
        SummaryLanguage::Zh => {
            format!("优先用 `unified_search`。重点能力：{groups}。详情按需读 `{details_uri}`。")
        }
        SummaryLanguage::En => {
            format!("Use `unified_search` first. Focus: {groups}. Details: `{details_uri}`.")
        }
    }
}

pub fn capability_summary_resource_list(providers: &[ProviderCatalogEntry]) -> Value {
    json!({
        "resources": [
            {
                "uri": CAPABILITY_SUMMARY_URI,
                "name": "capability-summary",
                "title": "MCP Prism Capability Summary",
                "description": "Live summary of currently available MCP Prism providers and recommended usage.",
                "mimeType": "text/markdown"
            },
            {
                "uri": CAPABILITY_SUMMARY_ZH_URI,
                "name": "capability-summary-zh",
                "title": "MCP Prism Capability Summary (ZH)",
                "description": "Chinese capability summary.",
                "mimeType": "text/markdown"
            },
            {
                "uri": CAPABILITY_SUMMARY_EN_URI,
                "name": "capability-summary-en",
                "title": "MCP Prism Capability Summary (EN)",
                "description": "English capability summary.",
                "mimeType": "text/markdown"
            },
            {
                "uri": "mcp-prism://provider-catalog",
                "name": "provider-catalog",
                "title": "MCP Prism Provider Catalog",
                "description": "Machine-readable provider catalog snapshot.",
                "mimeType": "application/json"
            }
        ],
        "nextCursor": null,
        "_meta": {
            "availableProviderCount": providers.iter().filter(|provider| provider.available).count()
        }
    })
}

pub fn capability_summary_resource_read(
    uri: &str,
    providers: &[ProviderCatalogEntry],
) -> Option<Value> {
    match uri {
        CAPABILITY_SUMMARY_URI => Some(json!({
            "contents": [
                {
                    "uri": CAPABILITY_SUMMARY_URI,
                    "mimeType": "text/markdown",
                    "text": capability_summary_markdown(providers, SummaryLanguage::En)
                }
            ]
        })),
        CAPABILITY_SUMMARY_ZH_URI => Some(json!({
            "contents": [
                {
                    "uri": CAPABILITY_SUMMARY_ZH_URI,
                    "mimeType": "text/markdown",
                    "text": capability_summary_markdown(providers, SummaryLanguage::Zh)
                }
            ]
        })),
        CAPABILITY_SUMMARY_EN_URI => Some(json!({
            "contents": [
                {
                    "uri": CAPABILITY_SUMMARY_EN_URI,
                    "mimeType": "text/markdown",
                    "text": capability_summary_markdown(providers, SummaryLanguage::En)
                }
            ]
        })),
        "mcp-prism://provider-catalog" => Some(json!({
            "contents": [
                {
                    "uri": "mcp-prism://provider-catalog",
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(providers).unwrap_or_else(|_| "[]".to_string())
                }
            ]
        })),
        _ => None,
    }
}

fn capability_summary_markdown(
    providers: &[ProviderCatalogEntry],
    language: SummaryLanguage,
) -> String {
    let available = providers
        .iter()
        .filter(|provider| provider.available)
        .collect::<Vec<_>>();
    let mut lines = match language {
        SummaryLanguage::Zh => vec![
            "# MCP Prism 能力摘要".to_string(),
            "".to_string(),
            "大多数任务优先使用 `unified_search`。只有明确要求指定上游时，再使用 `search_<provider_id>`。".to_string(),
            "".to_string(),
        ],
        SummaryLanguage::En => vec![
            "# MCP Prism Capability Summary".to_string(),
            "".to_string(),
            "Use `unified_search` for most tasks. Use `search_<provider_id>` only when a specific upstream is required.".to_string(),
            "".to_string(),
        ],
    };

    if available.is_empty() {
        lines.push(match language {
            SummaryLanguage::Zh => "当前没有可用上游。".to_string(),
            SummaryLanguage::En => "No upstream providers are currently enabled.".to_string(),
        });
        return lines.join("\n");
    }

    let grouped = grouped_available_providers(providers);
    lines.push(match language {
        SummaryLanguage::Zh => "## 当前能力域".to_string(),
        SummaryLanguage::En => "## Available Groups".to_string(),
    });
    for (category, items) in &grouped {
        lines.push(format!(
            "- `{}`: {}",
            render_category_name(category, language),
            items.join(", ")
        ));
    }

    lines.push("".to_string());
    lines.push(match language {
        SummaryLanguage::Zh => "## Provider 说明".to_string(),
        SummaryLanguage::En => "## Provider Notes".to_string(),
    });
    for provider in available {
        let capabilities = provider
            .capabilities
            .iter()
            .map(search_type_name)
            .collect::<Vec<_>>()
            .join(match language {
                SummaryLanguage::Zh => "、",
                SummaryLanguage::En => ", ",
            });
        let note = provider.notes.clone().unwrap_or_else(|| match language {
            SummaryLanguage::Zh => "无额外说明。".to_string(),
            SummaryLanguage::En => "No extra note.".to_string(),
        });
        lines.push(format!(
            "- `{}`: covers {}. {}",
            provider.id, capabilities, note
        ));
    }

    lines.push("".to_string());
    lines.push(match language {
        SummaryLanguage::Zh => "## 运维辅助".to_string(),
        SummaryLanguage::En => "## Operational Helpers".to_string(),
    });
    lines.push(match language {
        SummaryLanguage::Zh => "- `provider_catalog`: 查看当前实时可用 provider。".to_string(),
        SummaryLanguage::En => {
            "- `provider_catalog`: inspect the current live availability map.".to_string()
        }
    });
    lines.push(match language {
        SummaryLanguage::Zh => "- `cache_prewarm` / `cache_invalidate`: 管理缓存行为。".to_string(),
        SummaryLanguage::En => {
            "- `cache_prewarm` / `cache_invalidate`: manage cache behavior.".to_string()
        }
    });
    lines.push(match language {
        SummaryLanguage::Zh => {
            "- `issue_wrapped_key` / `revoke_wrapped_key`: 管理远端客户端访问。".to_string()
        }
        SummaryLanguage::En => {
            "- `issue_wrapped_key` / `revoke_wrapped_key`: manage remote client access.".to_string()
        }
    });

    lines.join("\n")
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SummaryLanguage {
    Zh,
    En,
}

fn detect_summary_language(params: &Value, accept_language: Option<&str>) -> SummaryLanguage {
    if let Some(language) = params
        .get("language")
        .and_then(|value| value.as_str())
        .or_else(|| params.get("locale").and_then(|value| value.as_str()))
        .or_else(|| {
            params
                .get("_meta")
                .and_then(|value| value.get("mcpPrism"))
                .and_then(|value| value.get("language"))
                .and_then(|value| value.as_str())
        })
        .or(accept_language)
    {
        let language = language.to_ascii_lowercase();
        if language.contains("zh") || language.contains("cn") {
            return SummaryLanguage::Zh;
        }
    }
    SummaryLanguage::En
}

fn detect_summary_focus(params: &Value, providers: &[ProviderCatalogEntry]) -> Vec<String> {
    if let Some(domain) = params
        .get("preferredDomain")
        .and_then(|value| value.as_str())
        .or_else(|| {
            params
                .get("_meta")
                .and_then(|value| value.get("mcpPrism"))
                .and_then(|value| value.get("preferredDomain"))
                .and_then(|value| value.as_str())
        })
    {
        return preferred_categories_for_domain(domain);
    }

    let client_name = params
        .get("clientInfo")
        .and_then(|value| value.get("name"))
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if ["cursor", "codex", "cline", "windsurf", "roo"]
        .iter()
        .any(|item| client_name.contains(item))
    {
        return vec![
            "business".to_string(),
            "search".to_string(),
            "academic".to_string(),
        ];
    }

    if ["claude", "chatgpt", "openclaw"]
        .iter()
        .any(|item| client_name.contains(item))
    {
        return vec![
            "search".to_string(),
            "academic".to_string(),
            "finance".to_string(),
        ];
    }

    grouped_available_providers(providers)
        .keys()
        .take(3)
        .cloned()
        .collect()
}

fn preferred_categories_for_domain(domain: &str) -> Vec<String> {
    match domain.to_ascii_lowercase().as_str() {
        "code" | "coding" | "developer" => vec![
            "business".to_string(),
            "search".to_string(),
            "academic".to_string(),
        ],
        "research" | "academic" | "paper" => vec![
            "academic".to_string(),
            "search".to_string(),
            "government".to_string(),
        ],
        "finance" | "market" => vec![
            "finance".to_string(),
            "search".to_string(),
            "government".to_string(),
        ],
        "maps" | "geo" | "travel" => vec![
            "maps".to_string(),
            "search".to_string(),
            "government".to_string(),
        ],
        "government" | "policy" | "public-data" => vec![
            "government".to_string(),
            "search".to_string(),
            "academic".to_string(),
        ],
        _ => vec![
            "search".to_string(),
            "academic".to_string(),
            "finance".to_string(),
        ],
    }
}

fn format_group(category: &str, count: usize, language: SummaryLanguage) -> String {
    match language {
        SummaryLanguage::Zh => format!("{}{}", render_category_name(category, language), count),
        SummaryLanguage::En => format!("{}({count})", render_category_name(category, language)),
    }
}

fn grouped_available_providers(
    providers: &[ProviderCatalogEntry],
) -> BTreeMap<String, Vec<String>> {
    let mut grouped = BTreeMap::new();
    for provider in providers.iter().filter(|provider| provider.available) {
        grouped
            .entry(provider.category.clone())
            .or_insert_with(Vec::new)
            .push(provider.id.clone());
    }
    grouped
}

fn render_category_name(category: &str, language: SummaryLanguage) -> String {
    match (language, category) {
        (SummaryLanguage::Zh, "search") => "搜索".to_string(),
        (SummaryLanguage::Zh, "academic") => "学术".to_string(),
        (SummaryLanguage::Zh, "finance") => "金融".to_string(),
        (SummaryLanguage::Zh, "maps") => "地图".to_string(),
        (SummaryLanguage::Zh, "government") => "政务".to_string(),
        (SummaryLanguage::Zh, "business") => "企业/代码".to_string(),
        (_, other) => other.to_string(),
    }
}

fn search_type_name(search_type: &SearchType) -> String {
    match search_type {
        SearchType::Auto => "auto",
        SearchType::Web => "web",
        SearchType::News => "news",
        SearchType::Academic => "academic",
        SearchType::Finance => "finance",
        SearchType::Company => "company",
        SearchType::Patent => "patent",
        SearchType::Government => "government",
        SearchType::Maps => "maps",
    }
    .to_string()
}

fn search_type_values() -> Vec<String> {
    [
        SearchType::Auto,
        SearchType::Web,
        SearchType::News,
        SearchType::Academic,
        SearchType::Finance,
        SearchType::Company,
        SearchType::Patent,
        SearchType::Government,
        SearchType::Maps,
    ]
    .into_iter()
    .map(|value| match value {
        SearchType::Auto => "auto",
        SearchType::Web => "web",
        SearchType::News => "news",
        SearchType::Academic => "academic",
        SearchType::Finance => "finance",
        SearchType::Company => "company",
        SearchType::Patent => "patent",
        SearchType::Government => "government",
        SearchType::Maps => "maps",
    })
    .map(ToOwned::to_owned)
    .collect()
}
