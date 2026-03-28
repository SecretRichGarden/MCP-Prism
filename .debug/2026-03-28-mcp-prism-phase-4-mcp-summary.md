# MCP Prism Phase 4 MCP Summary Optimization

## Goal

- 在 Agent 挂载 MCP Prism 时，主动反馈一个极短能力摘要。
- 细节不在初始化阶段展开，改为资源按需读取。

## Delivered

- `initialize.instructions`
  - 简短说明：优先用 `unified_search`
  - 仅返回能力域和数量
- `resources/list`
- `resources/read`
- `mcp-prism://capability-summary`

## Guardrail

- 初始化摘要必须足够短，不能反过来挤占 Agent 上下文。
- 详细 provider 描述只放在资源中，不在挂载时自动推送。

## Checkfix

- `cargo fmt`
- `cargo check`
- `cargo test`
