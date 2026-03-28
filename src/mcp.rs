use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::models::{ProviderCatalogEntry, SearchType};

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
