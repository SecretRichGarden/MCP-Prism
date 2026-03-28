# MCP Prism 部署指南

> 最后更新: 2026-03-28 | 范围: MVP

## 1. 前提

- Rust 1.92+
- 可访问公网的 Linux 主机或本地开发环境
- 至少一组可用 provider 凭据，或只用无需 key 的 OpenAlex / PubMed

## 2. 环境变量

最小可运行配置：

```bash
cp .env.example .env
```

建议至少设置：

```bash
MCP_PRISM_HMAC_SECRET=replace-me
MCP_PRISM_ENCRYPTION_KEY=replace-me
BRAVE_API_KEY=...
TAVILY_API_KEY=...
```

如果你没有这些 key，也可以先只依赖公开 provider：

```bash
OPENALEX_EMAIL=you@example.com
```

服务不会因为其他 provider 缺失而启动失败。

## 3. 代理策略

MCP Prism 明确支持三种网络模式。

### 模式 A: 继承当前系统代理

适合本地开发环境已经有 `http_proxy` / `https_proxy` / `all_proxy` 的情况。

```bash
MCP_PRISM_PROXY_MODE=system
```

### 模式 B: 完全直连

适合部署在境外 VPS 或无需代理的内网出口。

```bash
MCP_PRISM_PROXY_MODE=direct
```

### 模式 C: 服务级自定义代理

适合服务运行环境本身没有代理变量，但你希望只让 Prism 走代理。

```bash
MCP_PRISM_PROXY_MODE=custom
MCP_PRISM_HTTP_PROXY=http://127.0.0.1:52380
MCP_PRISM_HTTPS_PROXY=http://127.0.0.1:52380
MCP_PRISM_ALL_PROXY=socks5://127.0.0.1:1080
MCP_PRISM_NO_PROXY=127.0.0.1,localhost,.internal
```

`custom` 模式下，服务内部会自己处理 `no_proxy` 旁路。

## 4. 运行

```bash
cargo run -- serve
```

默认监听：

- `http://127.0.0.1:8787`

检查服务：

```bash
curl http://127.0.0.1:8787/health
curl http://127.0.0.1:8787/api/v1/providers
curl http://127.0.0.1:8787/metrics
```

## 5. 签发客户端 key

```bash
curl -X POST http://127.0.0.1:8787/api/v1/admin/wrapped-keys \
  -H 'content-type: application/json' \
  -d '{"client_id":"cursor-prod","ttl_seconds":86400}'
```

如果启用了 `MCP_PRISM_REQUIRE_CLIENT_KEY=true`，后续搜索请求需要带上：

```bash
Authorization: Bearer <wrapped-key>
```

## 6. 调用搜索 API

```bash
curl -X POST http://127.0.0.1:8787/api/v1/search \
  -H 'content-type: application/json' \
  -d '{
    "query": "Rust async runtime comparison",
    "search_type": "web",
    "options": { "limit": 5 }
  }'
```

## 7. 调用 MCP JSON-RPC

```bash
curl -X POST http://127.0.0.1:8787/mcp \
  -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

## 8. MCP 传输选项

- HTTP JSON-RPC:
  - `POST /mcp`
- WebSocket:
  - `GET /mcp/ws`
- SSE:
  - `GET /mcp/sse`
  - 服务会下发一个 `/mcp/message/{session_id}` 供后续 POST
- stdio:

```bash
cargo run -- stdio
```

## 9. 热重载配置

当你更新 `.env` 或 `.env.local` 后，可以重载：

```bash
curl -X POST http://127.0.0.1:8787/api/v1/admin/config/reload
```

## 10. 缓存与 key 管理

缓存预热：

```bash
curl -X POST http://127.0.0.1:8787/api/v1/admin/cache/prewarm \
  -H 'content-type: application/json' \
  -d '{"requests":[{"query":"Rust async runtime","search_type":"web","options":{"limit":3}}]}'
```

撤销 wrapped key：

```bash
curl -X POST http://127.0.0.1:8787/api/v1/admin/wrapped-keys/revoke \
  -H 'content-type: application/json' \
  -d '{"token":"<wrapped-key>"}'
```

## 11. 容器部署

```bash
docker compose up --build
```

相关文件：

- `Dockerfile`
- `docker-compose.yml`
- `nginx/nginx.conf`
- `prometheus/prometheus.yml`
- `prometheus/alerts.yml`

## 12. Checkfix

部署前至少执行：

```bash
cargo fmt --check
cargo check
cargo test
```

## 13. 故障排查

- provider 全部不可用：
  - 检查 `/api/v1/providers`
  - 检查对应 key 是否存在
  - 检查代理模式是否正确
- 路由模型不可用：
  - 检查 `MCP_PRISM_ROUTER_ENABLE_LLM`
  - 检查 `MCP_PRISM_ROUTER_BASE_URL` / `MCP_PRISM_ROUTER_API_KEY`
  - 即使失败，也应自动退回 heuristic line
- 远端 VPS 不走代理：
  - 设置 `MCP_PRISM_PROXY_MODE=direct`
