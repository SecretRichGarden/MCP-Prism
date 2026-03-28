# MCP Prism MVP 架构说明

## 服务分层

```text
HTTP / MCP JSON-RPC / WebSocket / SSE / stdio
        |
        v
SearchService
        |
        +-- DualLineRouter
        |     +-- LLM line (OpenAI-compatible /chat/completions)
        |     +-- Heuristic fallback line
        |     +-- Weighted experiment line
        |
        +-- AdapterRegistry
              +-- web_search
              +-- academic
              +-- finance
              +-- maps
              +-- business
              +-- government
```

## 关键原则

- Provider 是可选加载的，不要求用户把所有 key 一次性填完。
- 路由必须可以在无模型情况下独立工作。
- 网络层必须兼容代理和直连两种部署现实。
- 所有外部响应都要归一成统一结果模型，避免 MCP 侧承受多协议碎片。

## 当前实现边界

- 已实现:
  - HTTP API
  - MCP JSON-RPC `initialize` / `tools/list` / `tools/call`
  - stdio / HTTP / WebSocket / SSE transport
  - Wrapped key issue/validate
  - Wrapped key rotate/revoke
  - 8 个 P0 provider 模块
  - GitHub / Data.gov / openFDA / Census 扩展模块
  - weighted experiment routing
  - rate limiting
  - memory + optional Redis cache
  - Prometheus metrics
- 暂未实现:
  - 外部商业/付费 provider 的真实联网验收
  - 完整的 OTLP / Jaeger exporter
