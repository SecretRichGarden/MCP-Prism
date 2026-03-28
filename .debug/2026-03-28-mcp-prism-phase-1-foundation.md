# MCP Prism Phase 1 Foundation

## Scope

- 将现有 PRD 转换为 Ralph 可执行拆分。
- 建立 Rust 工程、统一配置层、双线路由和 P0 provider 模块骨架。
- 同步 `.env` 体验设计与代理兼容策略。

## Architecture Decisions

- 远端入口优先做成 HTTP + MCP JSON-RPC，适合 VPS 场景；stdio 暂不作为 1.0 首实现。
- provider 采用可选加载，不要求用户一次性填完所有 Key。
- 路由分为两条线：
  - small-model line: 兼容 OpenAI 风格 `/chat/completions`；
  - linear fallback line: 按 query 类型、provider 能力和可用性直接调度。
- 代理处理做成 `system` / `direct` / `custom` 三种模式，并在 custom 模式自行处理 `no_proxy`。

## External Docs Read Summary

- Brave Search: 官方文档确认 `web`、`news`、`images` 等资源和 `X-Subscription-Token` 认证模式。
- Tavily: 官方文档确认 `/search`、`/extract`、`/crawl`、`/map`、`/research` 五个主端点，使用 Bearer 认证。
- 智谱 Web Search: 官方 API 文档确认 `POST /api/paas/v4/web_search`，支持 `search_std/search_pro/search_pro_sogou/search_pro_quark`。
- OpenAlex: 官方文档确认 `/works` 的 `search/filter/sort/select` 主查询模型。
- PubMed: NCBI 官方文档确认 `esearch` + `esummary` 双阶段调用模式。
- 其余模块以仓库 PRD/API 调研底稿为设计输入，保持可替换实现。

## Phase Status

- Completed

## Checkfix

- `cargo fmt --check`
- `cargo check`
- `cargo test`

## Outcome

- 项目从纯 PRD 状态进入可运行 MVP 状态。
- P0 provider 模块、双线路由、HTTP/MCP API 和部署文档均已落地。

