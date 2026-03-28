# MCP Prism API 文档

> 最后更新: 2026-03-28 | 版本: v0.1.0

## 概览

MCP Prism 1.0 MVP 暴露一组统一 HTTP API 和一个 MCP JSON-RPC 入口。用户只需要配置自己实际拥有的 provider 凭据，服务会自动识别可用上游并用双线路由执行请求。

## Base URL

默认本地开发地址：`http://127.0.0.1:8787`

## 认证方式

- 默认可关闭：`MCP_PRISM_REQUIRE_CLIENT_KEY=false`
- 开启后使用 `Authorization: Bearer <wrapped-key>`
- wrapped key 可通过 `/api/v1/admin/wrapped-keys` 生成

## 端点列表

| 方法 | 路径 | 功能 | 认证 |
|------|------|------|------|
| `GET` | `/health` | 健康检查、路由模式、代理模式、provider 状态 | 否 |
| `GET` | `/metrics` | Prometheus 指标导出 | 否 |
| `GET` | `/api/v1/providers` | 返回当前环境下可用/不可用的 provider 目录 | 否 |
| `POST` | `/api/v1/providers/{provider_id}/search` | 强制命中指定 provider 的 API 包 | 可选 |
| `POST` | `/api/v1/search` | 统一搜索入口 | 可选 |
| `POST` | `/api/v1/admin/wrapped-keys` | 签发 wrapped client key | 否 |
| `POST` | `/api/v1/admin/wrapped-keys/revoke` | 撤销 wrapped key | 否 |
| `POST` | `/api/v1/admin/wrapped-keys/rotate` | 轮换 wrapped key | 否 |
| `POST` | `/api/v1/admin/cache/prewarm` | 缓存预热 | 否 |
| `POST` | `/api/v1/admin/cache/invalidate` | 缓存失效 | 否 |
| `POST` | `/api/v1/admin/config/reload` | 重读 `.env` / `.env.local` 和环境变量 | 否 |
| `POST` | `/mcp` | MCP JSON-RPC 入口，支持 `initialize`、`tools/list`、`tools/call` | 可选 |
| `GET` | `/mcp/ws` | MCP WebSocket transport | 可选 |
| `GET` | `/mcp/sse` | MCP SSE transport 会话入口 | 可选 |
| `POST` | `/mcp/message/{session_id}` | MCP SSE transport 消息提交 | 可选 |

## 详细接口

### 统一搜索

- **路径**: `POST /api/v1/search`
- **功能**: 通过一个入口调度当前可用的 provider 池
- **认证**: 取决于 `MCP_PRISM_REQUIRE_CLIENT_KEY`

#### 请求参数

```json
{
  "query": "阿里巴巴最新财务表现",
  "search_type": "finance",
  "options": {
    "limit": 5,
    "provider_hints": ["finance_mcp", "tavily"],
    "operation": "company_performance"
  }
}
```

#### 响应格式

成功：

```json
{
  "request_id": "uuid",
  "query": "阿里巴巴最新财务表现",
  "search_type": "finance",
  "decision": {
    "search_type": "finance",
    "primary_sources": ["finance_mcp", "tavily"],
    "secondary_sources": [],
    "strategy": "parallel",
    "line": "heuristic",
    "reasoning": "heuristic planner matched query class against currently available providers",
    "fallback_reason": null
  },
  "meta": {
    "total_results": 3,
    "sources_used": [],
    "fallback_used": null,
    "cached": false
  },
  "results": [],
  "timing": {
    "routing_ms": 4,
    "total_api_ms": 78,
    "normalization_ms": 1,
    "total_ms": 80
  }
}
```

失败：

```json
{
  "code": 502,
  "error": "provider unavailable: all selected providers failed during execution"
}
```

#### 常见错误

| HTTP 状态码 | 含义 | 处理建议 |
|-------------|------|----------|
| `400` | 请求体不合法或 `query` 为空 | 校验请求参数 |
| `401` | wrapped key 缺失或签名无效 | 重新签发并附带 `Authorization` |
| `502` | 当前选择的 provider 全部失败 | 检查 provider key、base URL、代理设置 |
| `500` | 配置或服务内部错误 | 查看服务日志 |

### Provider 目录

- **路径**: `GET /api/v1/providers`
- **功能**: 返回 provider 是否可用、需要何种凭据、对应功能域

### Wrapped Key 生成

- **路径**: `POST /api/v1/admin/wrapped-keys`
- **功能**: 为远端 MCP 客户端签发聚合 key

请求示例：

```json
{
  "client_id": "cursor-prod",
  "ttl_seconds": 86400
}
```

### MCP JSON-RPC

- **路径**: `POST /mcp`
- **支持方法**:
  - `initialize`
  - `ping`
  - `tools/list`
  - `tools/call`
  - `resources/list`
  - `resources/read`

`initialize` 会返回一个**短摘要**放在 `instructions` 字段中，用于在 Agent 挂载时快速告知：

- 优先使用 `unified_search`
- 当前有哪些能力域可用
- 每个能力域大概有多少上游 provider

这个摘要刻意保持简短，避免无意义挤占模型上下文。

摘要行为：

- 自动根据 `Accept-Language` 或初始化参数中的语言 hint 输出中英文
- 自动根据客户端类型和可选的 `preferredDomain` hint 调整重点能力域
- 仍然只返回短索引，不返回长说明

更完整的摘要通过资源读取：

- `mcp-prism://capability-summary/en`
- `mcp-prism://capability-summary/zh`
- `mcp-prism://provider-catalog`

可选 hint 示例：

```json
{
  "protocolVersion": "2025-11-25",
  "clientInfo": { "name": "Cursor", "version": "1.0" },
  "_meta": {
    "mcpPrism": {
      "language": "zh-CN",
      "preferredDomain": "coding"
    }
  }
}
```

当前工具：

- `unified_search`
- `provider_catalog`
- `issue_wrapped_key`
- `revoke_wrapped_key`
- `cache_prewarm`
- `cache_invalidate`
- `search_<provider_id>`
