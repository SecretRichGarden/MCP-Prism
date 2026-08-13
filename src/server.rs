use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode},
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, post},
};
use futures::{StreamExt, stream};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::{RwLock, mpsc},
};
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    adapters::StdioMcpRegistry,
    auth::{RevocationRegistry, SecretCipher, WrappedApiKeyClaims, WrappedApiKeyService},
    config::AppConfig,
    error::{AppError, AppResult},
    mcp::{
        JsonRpcRequest, JsonRpcResponse, capability_summary_resource_list,
        capability_summary_resource_read, initialize_instructions, tool_definitions,
    },
    models::{HealthSnapshot, UnifiedSearchRequest},
    service::SearchService,
};

#[derive(Clone)]
pub struct AppRuntime {
    pub config: AppConfig,
    pub service: SearchService,
    pub wrapped_keys: WrappedApiKeyService,
    #[allow(dead_code)]
    pub secret_cipher: SecretCipher,
    /// stdio 型 MCP provider 聚合（透传工具），由 from_config 后 `init` 异步拉起。
    pub stdio: StdioMcpRegistry,
}

impl AppRuntime {
    pub fn from_config(config: AppConfig) -> AppResult<Self> {
        Ok(Self {
            service: SearchService::new(&config)?,
            wrapped_keys: WrappedApiKeyService::new(config.auth.hmac_secret.clone()),
            secret_cipher: SecretCipher::new(&config.auth.encryption_key),
            stdio: StdioMcpRegistry::empty(),
            config,
        })
    }
}

pub type SharedRuntime = Arc<RwLock<AppRuntime>>;

#[derive(Default)]
struct SseSessionHub {
    sessions: RwLock<HashMap<String, mpsc::Sender<JsonRpcResponse>>>,
}

impl SseSessionHub {
    async fn create_session(&self) -> (String, mpsc::Receiver<JsonRpcResponse>) {
        let id = uuid::Uuid::new_v4().to_string();
        let (sender, receiver) = mpsc::channel(16);
        self.sessions.write().await.insert(id.clone(), sender);
        (id, receiver)
    }

    async fn send(&self, session_id: &str, response: JsonRpcResponse) -> AppResult<()> {
        let sender = self
            .sessions
            .read()
            .await
            .get(session_id)
            .cloned()
            .ok_or_else(|| AppError::Validation(format!("unknown SSE session: {session_id}")))?;
        sender
            .send(response)
            .await
            .map_err(|_| AppError::Internal("failed to send SSE response".to_string()))
    }
}

#[derive(Clone)]
struct AppState {
    runtime: SharedRuntime,
    revocations: Arc<RevocationRegistry>,
    sse_hub: Arc<SseSessionHub>,
}

type SharedState = Arc<AppState>;

pub async fn serve(config: AppConfig) -> AppResult<()> {
    let addr = format!("{}:{}", config.server.host, config.server.port)
        .parse::<SocketAddr>()
        .map_err(|err| AppError::Config(err.to_string()))?;
    let app_runtime = AppRuntime::from_config(config.clone())?;
    app_runtime.stdio.init(&config).await; // 拉起 stdio 型 MCP provider（透传工具）
    let runtime: SharedRuntime = Arc::new(RwLock::new(app_runtime));
    let state = Arc::new(AppState {
        runtime,
        revocations: Arc::new(RevocationRegistry::new()),
        sse_hub: Arc::new(SseSessionHub::default()),
    });
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;

    tracing::info!(host = %config.server.host, port = config.server.port, "mcp-prism server listening");
    axum::serve(listener, app)
        .await
        .map_err(|err| AppError::Internal(err.to_string()))
}

pub async fn run_stdio(mut config: AppConfig) -> AppResult<()> {
    if config.auth.trust_stdio_transport {
        config.auth.require_client_key = false;
    }
    let runtime = AppRuntime::from_config(config.clone())?;
    runtime.stdio.init(&config).await; // 拉起 stdio 型 MCP provider（透传工具）
    let revocations = RevocationRegistry::new();
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines
        .next_line()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?
    {
        if line.trim().is_empty() {
            continue;
        }
        let request = match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => request,
            Err(error) => {
                let response = JsonRpcResponse::err(None, -32700, error.to_string());
                stdout
                    .write_all(format!("{}\n", serde_json::to_string(&response)?).as_bytes())
                    .await
                    .map_err(|err| AppError::Internal(err.to_string()))?;
                continue;
            }
        };

        if let Some(response) =
            process_jsonrpc_request(&runtime, &revocations, request, None).await?
        {
            stdout
                .write_all(format!("{}\n", serde_json::to_string(&response)?).as_bytes())
                .await
                .map_err(|err| AppError::Internal(err.to_string()))?;
            stdout
                .flush()
                .await
                .map_err(|err| AppError::Internal(err.to_string()))?;
        }
    }

    Ok(())
}

fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .route("/api/v1/providers", get(providers))
        .route(
            "/api/v1/providers/{provider_id}/search",
            post(provider_search),
        )
        .route("/api/v1/search", post(search))
        .route("/api/v1/admin/wrapped-keys", post(issue_wrapped_key))
        .route(
            "/api/v1/admin/wrapped-keys/revoke",
            post(revoke_wrapped_key),
        )
        .route(
            "/api/v1/admin/wrapped-keys/rotate",
            post(rotate_wrapped_key),
        )
        .route("/api/v1/admin/cache/prewarm", post(cache_prewarm))
        .route("/api/v1/admin/cache/invalidate", post(cache_invalidate))
        .route("/api/v1/admin/config/reload", post(reload_config))
        .route("/mcp", post(mcp_endpoint))
        .route("/mcp/ws", get(mcp_websocket))
        .route("/mcp/sse", get(mcp_sse))
        .route("/mcp/message/{session_id}", post(mcp_sse_message))
        .with_state(state)
}

async fn root(State(state): State<SharedState>) -> impl IntoResponse {
    let runtime = state.runtime.read().await;
    Json(runtime.config.redacted_snapshot())
}

async fn health(State(state): State<SharedState>) -> impl IntoResponse {
    let runtime = state.runtime.read().await;
    let snapshot = HealthSnapshot {
        status: "ok".to_string(),
        routing_mode: runtime.config.routing_mode_label(),
        proxy_mode: runtime.config.proxy.mode.to_string(),
        providers: runtime.service.provider_catalog(),
    };
    Json(snapshot)
}

async fn metrics(State(state): State<SharedState>) -> impl IntoResponse {
    let runtime = state.runtime.read().await;
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        runtime.service.metrics().render(),
    )
}

async fn providers(State(state): State<SharedState>) -> impl IntoResponse {
    let runtime = state.runtime.read().await;
    Json(runtime.service.provider_catalog())
}

async fn search(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(mut request): Json<UnifiedSearchRequest>,
) -> AppResult<Json<crate::models::UnifiedResponse>> {
    let claims = authorize(&headers, &state).await?;
    let runtime = state.runtime.read().await;
    if let Some(claims) = claims {
        request.client_id = Some(claims.client_id);
    }
    let response = runtime.service.search(request).await?;
    Ok(Json(response))
}

async fn provider_search(
    State(state): State<SharedState>,
    Path(provider_id): Path<String>,
    headers: HeaderMap,
    Json(mut request): Json<UnifiedSearchRequest>,
) -> AppResult<Json<crate::models::UnifiedResponse>> {
    let claims = authorize(&headers, &state).await?;
    let runtime = state.runtime.read().await;
    if let Some(claims) = claims {
        request.client_id = Some(claims.client_id);
    }
    request.options.provider_hints = vec![provider_id];
    let response = runtime.service.search(request).await?;
    Ok(Json(response))
}

async fn issue_wrapped_key(
    State(state): State<SharedState>,
    Json(input): Json<IssueWrappedKeyInput>,
) -> AppResult<Json<Value>> {
    let runtime = state.runtime.read().await;
    let token = runtime.wrapped_keys.issue(
        &input.client_id,
        Duration::from_secs(input.ttl_seconds.unwrap_or(3600)),
    )?;
    Ok(Json(json!({"token": token})))
}

async fn revoke_wrapped_key(
    State(state): State<SharedState>,
    Json(input): Json<RevokeWrappedKeyInput>,
) -> AppResult<Json<Value>> {
    if let Some(token) = input.token {
        let runtime = state.runtime.read().await;
        let claims = runtime.wrapped_keys.validate(&token)?;
        drop(runtime);
        state
            .revocations
            .revoke_token_signature(&claims.signature)
            .await;
        return Ok(Json(json!({"revoked_signature": claims.signature})));
    }
    if let Some(client_id) = input.client_id {
        state.revocations.revoke_client(&client_id).await;
        return Ok(Json(json!({"revoked_client_id": client_id})));
    }
    Err(AppError::Validation(
        "token or client_id is required to revoke a wrapped key".to_string(),
    ))
}

async fn rotate_wrapped_key(
    State(state): State<SharedState>,
    Json(input): Json<RotateWrappedKeyInput>,
) -> AppResult<Json<Value>> {
    let runtime = state.runtime.read().await;
    let claims = runtime.wrapped_keys.validate(&input.old_token)?;
    let token = runtime.wrapped_keys.issue(
        &input.client_id,
        Duration::from_secs(input.ttl_seconds.unwrap_or(3600)),
    )?;
    drop(runtime);
    state
        .revocations
        .revoke_token_signature(&claims.signature)
        .await;
    Ok(Json(json!({
        "token": token,
        "revoked_signature": claims.signature
    })))
}

async fn cache_prewarm(
    State(state): State<SharedState>,
    Json(input): Json<CachePrewarmInput>,
) -> AppResult<Json<Value>> {
    let runtime = state.runtime.read().await;
    let outcomes = runtime.service.prewarm(input.requests).await;
    let report = outcomes
        .into_iter()
        .map(|outcome| match outcome {
            Ok(key) => json!({"status": "ok", "key": key}),
            Err(error) => json!({"status": "error", "error": error.to_string()}),
        })
        .collect::<Vec<_>>();
    Ok(Json(json!({"results": report})))
}

async fn cache_invalidate(
    State(state): State<SharedState>,
    Json(input): Json<CacheInvalidateInput>,
) -> AppResult<Json<Value>> {
    let runtime = state.runtime.read().await;
    let key = runtime.service.invalidate(&input.request).await?;
    Ok(Json(json!({"invalidated_key": key})))
}

async fn reload_config(State(state): State<SharedState>) -> AppResult<Json<Value>> {
    let mut runtime = state.runtime.write().await;
    *runtime = AppRuntime::from_config(AppConfig::load()?)?;
    Ok(Json(json!({"status": "reloaded"})))
}

async fn mcp_endpoint(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    match authorize(&headers, &state).await {
        Ok(_claims) => match handle_mcp_request(&state, request, language_hint(&headers)).await {
            Ok(Some(response)) => (StatusCode::OK, Json(response)).into_response(),
            Ok(None) => StatusCode::NO_CONTENT.into_response(),
            Err(error) => (
                error.status_code(),
                Json(JsonRpcResponse::err(None, -32000, error.to_string())),
            )
                .into_response(),
        },
        Err(error) => (
            error.status_code(),
            Json(JsonRpcResponse::err(None, -32001, error.to_string())),
        )
            .into_response(),
    }
}

async fn mcp_websocket(
    State(state): State<SharedState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let _claims = authorize(&headers, &state).await?;
    Ok(ws
        .on_upgrade(move |socket| handle_websocket(socket, state))
        .into_response())
}

async fn mcp_sse(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let _claims = authorize(&headers, &state).await?;
    let runtime = state.runtime.read().await;
    let keepalive = runtime.config.server.sse_keepalive_seconds;
    drop(runtime);

    let (session_id, receiver) = state.sse_hub.create_session().await;
    let endpoint_event = Event::default()
        .event("endpoint")
        .data(format!("/mcp/message/{session_id}"));
    let stream = stream::once(async { Ok::<Event, Infallible>(endpoint_event) }).chain(
        ReceiverStream::new(receiver).map(|response| {
            Ok(Event::default()
                .event("message")
                .data(serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())))
        }),
    );

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(keepalive))
            .text("keepalive"),
    ))
}

async fn mcp_sse_message(
    State(state): State<SharedState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<JsonRpcRequest>,
) -> AppResult<StatusCode> {
    let _claims = authorize(&headers, &state).await?;
    if let Some(response) = handle_mcp_request(&state, request, language_hint(&headers)).await? {
        state.sse_hub.send(&session_id, response).await?;
    }
    Ok(StatusCode::ACCEPTED)
}

async fn handle_websocket(mut socket: WebSocket, state: SharedState) {
    while let Some(message) = socket.recv().await {
        let Ok(message) = message else {
            break;
        };
        match message {
            Message::Text(text) => {
                let response = match serde_json::from_str::<JsonRpcRequest>(&text) {
                    Ok(request) => handle_mcp_request(&state, request, None)
                        .await
                        .map_err(|error| JsonRpcResponse::err(None, -32000, error.to_string())),
                    Err(error) => Ok(Some(JsonRpcResponse::err(None, -32700, error.to_string()))),
                };

                match response {
                    Ok(Some(response)) => {
                        if socket
                            .send(Message::Text(
                                serde_json::to_string(&response)
                                    .unwrap_or_else(|_| "{}".to_string())
                                    .into(),
                            ))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Ok(None) => {}
                    Err(response) => {
                        let _ = socket
                            .send(Message::Text(
                                serde_json::to_string(&response)
                                    .unwrap_or_else(|_| "{}".to_string())
                                    .into(),
                            ))
                            .await;
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

async fn handle_mcp_request(
    state: &SharedState,
    request: JsonRpcRequest,
    accept_language: Option<String>,
) -> AppResult<Option<JsonRpcResponse>> {
    let runtime = state.runtime.read().await;
    process_jsonrpc_request(
        &runtime,
        &state.revocations,
        request,
        accept_language.as_deref(),
    )
    .await
}

async fn process_jsonrpc_request(
    runtime: &AppRuntime,
    revocations: &RevocationRegistry,
    request: JsonRpcRequest,
    accept_language: Option<&str>,
) -> AppResult<Option<JsonRpcResponse>> {
    let id = request.id.clone();

    let response = match request.method.as_str() {
        "initialize" => {
            let providers = runtime.service.provider_catalog();
            JsonRpcResponse::ok(
                id,
                json!({
                    "protocolVersion": request.params.get("protocolVersion").and_then(|value| value.as_str()).unwrap_or("2025-11-25"),
                    "capabilities": {
                        "tools": {"listChanged": true},
                        "streaming": {},
                        "resources": {"listChanged": false}
                    },
                    "serverInfo": {"name": "mcp-prism", "version": env!("CARGO_PKG_VERSION")},
                    "instructions": initialize_instructions(&providers, &request.params, accept_language)
                }),
            )
        }
        "notifications/initialized" => return Ok(None),
        "ping" => JsonRpcResponse::ok(id, json!({})),
        "tools/list" => {
            let mut tools = tool_definitions(&runtime.service.provider_catalog());
            if let Some(array) = tools.as_array_mut() {
                array.extend(runtime.stdio.tool_definitions().await);
            }
            JsonRpcResponse::ok(id, json!({"tools": tools}))
        }
        "resources/list" => JsonRpcResponse::ok(
            id,
            capability_summary_resource_list(&runtime.service.provider_catalog()),
        ),
        "resources/read" => {
            let uri = request
                .params
                .get("uri")
                .and_then(|value| value.as_str())
                .ok_or_else(|| AppError::Validation("resource uri is required".to_string()))?;
            let result = capability_summary_resource_read(uri, &runtime.service.provider_catalog())
                .ok_or_else(|| AppError::Validation(format!("unknown resource: {uri}")))?;
            JsonRpcResponse::ok(id, result)
        }
        "tools/call" => {
            let name = request
                .params
                .get("name")
                .and_then(|value| value.as_str())
                .ok_or_else(|| AppError::Validation("tool name is required".to_string()))?;
            let arguments = request
                .params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));

            let tool_result = process_tool_call(runtime, revocations, name, arguments).await?;
            JsonRpcResponse::ok(
                id,
                json!({
                    "content": [{"type": "text", "text": serde_json::to_string_pretty(&tool_result)?}],
                    "structuredContent": tool_result,
                    "isError": false
                }),
            )
        }
        method => JsonRpcResponse::err(id, -32601, format!("unknown method: {method}")),
    };

    Ok(Some(response))
}

async fn process_tool_call(
    runtime: &AppRuntime,
    revocations: &RevocationRegistry,
    name: &str,
    arguments: Value,
) -> AppResult<Value> {
    match name {
        "unified_search" => {
            let search_request = serde_json::from_value::<UnifiedSearchRequest>(arguments)?;
            Ok(serde_json::to_value(
                runtime.service.search(search_request).await?,
            )?)
        }
        "provider_catalog" => Ok(serde_json::to_value(runtime.service.provider_catalog())?),
        "issue_wrapped_key" => {
            let payload = serde_json::from_value::<IssueWrappedKeyInput>(arguments)?;
            let token = runtime.wrapped_keys.issue(
                &payload.client_id,
                Duration::from_secs(payload.ttl_seconds.unwrap_or(3600)),
            )?;
            Ok(json!({"token": token}))
        }
        "revoke_wrapped_key" => {
            let payload = serde_json::from_value::<RevokeWrappedKeyInput>(arguments)?;
            if let Some(token) = payload.token {
                let claims = runtime.wrapped_keys.validate(&token)?;
                revocations.revoke_token_signature(&claims.signature).await;
                Ok(json!({"revoked_signature": claims.signature}))
            } else if let Some(client_id) = payload.client_id {
                revocations.revoke_client(&client_id).await;
                Ok(json!({"revoked_client_id": client_id}))
            } else {
                Err(AppError::Validation(
                    "token or client_id is required".to_string(),
                ))
            }
        }
        "cache_prewarm" => {
            let payload = serde_json::from_value::<CachePrewarmInput>(arguments)?;
            let results = runtime.service.prewarm(payload.requests).await;
            Ok(json!({
                "results": results.into_iter().map(|item| match item {
                    Ok(key) => json!({"status": "ok", "key": key}),
                    Err(error) => json!({"status": "error", "error": error.to_string()}),
                }).collect::<Vec<_>>()
            }))
        }
        "cache_invalidate" => {
            let payload = serde_json::from_value::<CacheInvalidateInput>(arguments)?;
            let key = runtime.service.invalidate(&payload.request).await?;
            Ok(json!({"invalidated_key": key}))
        }
        _ if name.starts_with("search_") => {
            let provider_id = name.trim_start_matches("search_").to_string();
            let mut search_request = serde_json::from_value::<UnifiedSearchRequest>(arguments)?;
            search_request.options.provider_hints = vec![provider_id];
            Ok(serde_json::to_value(
                runtime.service.search(search_request).await?,
            )?)
        }
        _ if runtime.stdio.has_tool(name).await => {
            // stdio 型 MCP 工具透传：{provider_id}__{tool_name} → 子进程
            runtime.stdio.call_tool(name, arguments).await
        }
        _ => Err(AppError::Validation(format!("unknown tool: {name}"))),
    }
}

fn language_hint(headers: &HeaderMap) -> Option<String> {
    headers
        .get("accept-language")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

async fn authorize(
    headers: &HeaderMap,
    state: &SharedState,
) -> AppResult<Option<WrappedApiKeyClaims>> {
    let runtime = state.runtime.read().await;
    if !runtime.config.auth.require_client_key {
        return Ok(None);
    }

    let header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::Auth("missing Authorization header".to_string()))?;

    let token = header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .ok_or_else(|| AppError::Auth("expected Bearer token".to_string()))?;

    let claims = runtime.wrapped_keys.validate(token)?;
    drop(runtime);
    if state.revocations.is_revoked(&claims).await {
        return Err(AppError::Auth("wrapped key has been revoked".to_string()));
    }
    Ok(Some(claims))
}

#[derive(Debug, Deserialize)]
struct IssueWrappedKeyInput {
    client_id: String,
    #[serde(default)]
    ttl_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RevokeWrappedKeyInput {
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    client_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RotateWrappedKeyInput {
    client_id: String,
    old_token: String,
    #[serde(default)]
    ttl_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CachePrewarmInput {
    requests: Vec<UnifiedSearchRequest>,
}

#[derive(Debug, Deserialize)]
struct CacheInvalidateInput {
    request: UnifiedSearchRequest,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn test_router() -> Router {
        let mut config = AppConfig::load().unwrap();
        config.auth.require_client_key = false;
        let state: SharedState = Arc::new(AppState {
            runtime: Arc::new(RwLock::new(AppRuntime::from_config(config).unwrap())),
            revocations: Arc::new(RevocationRegistry::new()),
            sse_hub: Arc::new(SseSessionHub::default()),
        });
        build_router(state)
    }

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let app = test_router().await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn metrics_endpoint_is_available() {
        let app = test_router().await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn mcp_tools_list_is_available() {
        let app = test_router().await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["result"]["tools"].as_array().unwrap().len() >= 6);
    }

    #[tokio::test]
    async fn initialize_returns_short_instructions() {
        let app = test_router().await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let instructions = json["result"]["instructions"].as_str().unwrap();
        assert!(instructions.contains("unified_search"));
        assert!(instructions.len() < 160);
    }

    #[tokio::test]
    async fn initialize_supports_zh_and_domain_focus() {
        let app = test_router().await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header("content-type", "application/json")
                    .header("accept-language", "zh-CN")
                    .body(Body::from(
                        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","_meta":{"mcpPrism":{"preferredDomain":"coding"}},"clientInfo":{"name":"Cursor","version":"1.0"}}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let instructions = json["result"]["instructions"].as_str().unwrap();
        assert!(instructions.contains("重点能力"));
        assert!(instructions.contains(crate::mcp::CAPABILITY_SUMMARY_ZH_URI));
        assert!(instructions.len() < 140);
    }

    #[tokio::test]
    async fn capability_summary_resource_is_readable() {
        let app = test_router().await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"jsonrpc":"2.0","id":1,"method":"resources/read","params":{"uri":"mcp-prism://capability-summary"}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let text = json["result"]["contents"][0]["text"].as_str().unwrap();
        assert!(text.contains("MCP Prism Capability Summary"));
    }
}
