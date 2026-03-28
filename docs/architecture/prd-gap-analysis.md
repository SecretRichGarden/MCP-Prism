# MCP Prism PRD 对照与收敛结果

> 最后更新: 2026-03-28

## 结论

当前代码已经把原始 PRD 中相对 MVP 仍缺失的关键平台能力补齐到可运行版本：

- `FR-1`
  - JSON-RPC over `stdio`
  - JSON-RPC over HTTP
  - JSON-RPC over WebSocket
  - MCP SSE transport
  - tool discovery / invocation
- `FR-2`
  - 小模型优先路由
  - 启发式 fallback
  - provider 去重聚合
  - weighted experiment / canary 路由
- `FR-3`
  - Brave / Tavily / 智谱 / 秘塔 / FinanceMCP / PubMed / OpenAlex / 高德
- `FR-4`
  - `.env` / `.env.local`
  - AES-GCM secret cipher
  - reload 热更新
  - provider 可选加载
- `FR-5`
  - wrapped key issue / validate / rotate / revoke
  - per-client rate limiting
- `FR-6`
  - GitHub REST
  - GitHub GraphQL
- `FR-7`
  - Data.gov
  - openFDA
  - U.S. Census
- `FR-8`
  - memory + optional Redis cache
  - TTL 分层
  - prewarm / invalidate
- `FR-9`
  - Prometheus metrics
  - 结构化日志（JSON/pretty）
  - Prometheus alert rules 示例
- `FR-10`
  - weighted experiment 路由
  - canary / stable 双变体支持

## 仍需真实第三方凭据验证的部分

以下模块代码已接入，但最终效果仍依赖外部 key、额度或网络：

- Brave
- Tavily
- 智谱
- 高德
- GitHub REST / GraphQL
- FinanceMCP bridge

这些能力无法在无密钥环境里做真正端到端联网验收，但服务内逻辑、配置接口、路由与测试闭环已补齐。
