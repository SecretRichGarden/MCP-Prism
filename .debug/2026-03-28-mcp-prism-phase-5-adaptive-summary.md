# MCP Prism Phase 5 Adaptive Summary

## Goal

- 让 MCP 挂载摘要进一步压缩
- 提供中英文双版本
- 支持基于客户端和 domain hint 的重点能力域自适应

## Delivered

- `initialize.instructions`
  - 单行短摘要
  - 自动语言选择
  - 自动重点能力域选择
- `mcp-prism://capability-summary/en`
- `mcp-prism://capability-summary/zh`
- hint 支持：
  - `Accept-Language`
  - `_meta.mcpPrism.language`
  - `_meta.mcpPrism.preferredDomain`
  - `clientInfo.name`

## Guardrail

- 摘要必须继续保持很短
- 只展示当前真实可用能力域
- 详细 provider 说明继续留在 `resources/read`

## Checkfix

- `cargo fmt --check`
- `cargo check`
- `cargo test`
