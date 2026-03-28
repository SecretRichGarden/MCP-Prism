# MCP 灵犀 · MCP Prism

<p align="center">
  <img src="materials/mcp-prism-logo.png" alt="MCP Prism logo — 一束光经棱镜折射为多源信息谱" width="420">
</p>

<p align="center">
  <strong>Rust 构建的 MCP 信息池</strong> — 统一、安全、低上下文占用的智能化外部信息底座<br/>
  <em>与 LLM 协同，构成 Agent 高效执行任务的底层「信息引擎」</em>
</p>

---

## 这是什么

**MCP Prism（MCP 灵犀）** 面向 MCP 兼容的 Agent 运行时（如 Cursor、Claude Desktop、OpenClaw 等），在 **一台外网 VPS** 上提供：

- **单一 MCP 接入点**：Agent 侧只需配置 **一个 MCP 服务 + 一把聚合后的 API Key**，不再为几十个工具与代理牵丝攀藤。
- **高质量多源聚合**：将搜索、金融、学术、地图、政务与商业数据等能力，经适配层汇入同一套协议与响应规范（详见 [完整 API 清单](docs/PRD/2026-03-28-07-20-mcp-pool-rust-architecture-generated-complete-api-list.md)）。
- **上下文友好**：大量工具定义与路由细节留在服务端；Agent 侧 **embedding / 工具列表占用显著降低**，把 token 留给推理与任务本身。
- **凭据外置**：各数据源 Key 集中在 VPS 加密管理与轮换；**降低本地配置与代码泄露带来的凭据风险**（对比将密钥散落在本机多 MCP 配置中的模式）。

一句话：**一束请求进入棱镜，按需折射到正确下游；响应规范化后回到 Agent —— 像搜索引擎一样「快查」，又像堡垒一样「少暴露」。**

## 设计哲学

| 理念 | 含义 |
|------|------|
| **Prism（棱镜）** | 单一入口，多谱系输出 —— 统一协议下覆盖多类信息源。 |
| **灵犀** | 轻量智能路由（可选小模型）理解意图、选择适配器；**下游原始数据不二次灌入路由模型**，直接合并给上游 Agent。 |
| **Rust** | 高并发、低延迟、长期驻留外网服务的工程取向；内存安全与可预期性能利于 1.0 生产部署。 |
| **信息池** | 适配器池 + 缓存 + 配额 + 观测，形成可演进的数据面，而非一次性脚本集合。 |

## 核心能力（1.0 目标）

- **认证与租户**：聚合 API Key 校验、按客户端限流。
- **智能路由（可选）**：基于查询类型选择下游 API、并行/串行/降级策略；详见 [PRD §2.2.2](docs/PRD/PRD-mcp-pool-rust-architecture.md)。
- **适配器池**：搜索、金融、学术、地图、企业与政务等类别的 HTTP/gRPC 适配与 **响应规范化**。
- **缓存与韧性**：热点缓存、超时重试、健康检查与降级，缓解单点与配额抖动。
- **可观测性**：结构化日志、指标与追踪 ID，便于在 VPS 上排障。

更完整的功能编号、非功能指标与安全设计见 **[产品需求文档 (PRD)](docs/PRD/PRD-mcp-pool-rust-architecture.md)**。

## 架构一览

```text
Agent (MCP 客户端)
        │  MCP / HTTPS
        ▼
┌───────────────────────────────────┐
│  MCP Prism（Rust，部署于 VPS）      │
│  鉴权 · 限流 · 路由 · 适配器池 · 缓存 │
└───────────────────────────────────┘
        │  多路下游 API（各平台 Key 仅存服务端）
        ▼
外部信息源（搜索 / 金融 / 学术 / 政务 / 商业 …）
```

路由层若启用小模型，**仅参与「去哪查」**；**查到的正文/JSON 不再经该模型转发**，以降低延迟与成本，并避免无意义的信息压缩损失。

## 文档索引

| 文档 | 说明 |
|------|------|
| [PRD-mcp-pool-rust-architecture.md](docs/PRD/PRD-mcp-pool-rust-architecture.md) | 愿景、架构、功能/非功能需求、安全与里程碑 |
| [完整 API 清单](docs/PRD/2026-03-28-07-20-mcp-pool-rust-architecture-generated-complete-api-list.md) | 平台 · 官方文档链接 · 数据类型 · 认证与配额摘要（59 类来源调研底稿） |
| [部署指南](docs/DEPLOYMENT.md) | 环境变量、代理、缓存、容器化与 MCP 传输说明 |
| [统一 API 文档](docs/api/unified-search.md) | HTTP / MCP 端点、管理接口、工具列表 |

## 配置方式说明

很多 Node MCP 项目会把服务端配置写进 `config.json`。

**MCP Prism 不是这种模式。**

- **Rust 服务端配置**：使用 `.env` / `.env.local` / 系统环境变量
- **Agent 客户端配置**：仍然用你所使用的 MCP 客户端自己的配置文件

也就是说：

- `mcp-prism` 这个 Rust 服务本身，**不需要**单独的 `config.json`
- 如果你的 Cursor / Claude Desktop / OpenClaw / 自研 Agent 需要 JSON 配置，那份 JSON 是**客户端自己的 MCP 配置**，作用是告诉客户端去连哪一个 `mcp-prism` 服务

## 快速开始

### 1. 配置服务端

```bash
cp .env.example .env
```

最少需要改这些：

```bash
MCP_PRISM_HMAC_SECRET=replace-me
MCP_PRISM_ENCRYPTION_KEY=replace-me
MCP_PRISM_PROXY_MODE=direct
BRAVE_API_KEY=...
TAVILY_API_KEY=...
```

说明：

- 你**不需要**把所有 provider 的 key 都填满
- 只填你手里真正有的 key 即可
- 没填的 provider 会自动变成 unavailable，但服务仍然能启动

### 2. 启动服务

本地直接跑：

```bash
cargo run -- serve
```

或者容器方式：

```bash
docker compose up --build
```

默认监听：

- `http://127.0.0.1:8787`

先确认服务活着：

```bash
curl http://127.0.0.1:8787/health
curl http://127.0.0.1:8787/api/v1/providers
```

### 3. 签发给 Agent 的访问 key

如果你希望另一个 Agent 远程访问你的 MCP Prism，先签发一个 wrapped key：

```bash
curl -X POST http://127.0.0.1:8787/api/v1/admin/wrapped-keys \
  -H 'content-type: application/json' \
  -d '{"client_id":"cursor-prod","ttl_seconds":86400}'
```

返回结果里的 `token` 就是给 Agent 配的那把聚合 key。

## Agent 接入方式

MCP Prism 当前支持四种传输：

- HTTP JSON-RPC：`POST /mcp`
- WebSocket：`GET /mcp/ws`
- SSE：`GET /mcp/sse`
- stdio：`mcp-prism stdio`

### 场景 A：服务和 Agent 在同一台机器

这种情况下最简单，直接让客户端用 `stdio` 启动：

```bash
mcp-prism stdio
```

如果你的 MCP 客户端是 JSON 配置风格，思路通常是：

```json
{
  "mcpServers": {
    "mcp-prism": {
      "command": "mcp-prism",
      "args": ["stdio"]
    }
  }
}
```

这类配置适合同机部署。

### 场景 B：MCP Prism 部署在 VPS，另一个 Agent 远程访问

这种情况下，Agent 不直接启动 Rust 进程，而是配置一个**远程 MCP 地址**。

你需要把客户端指向以下任一入口：

- `https://your-domain.com/mcp`
- `wss://your-domain.com/mcp/ws`
- `https://your-domain.com/mcp/sse`

并在请求头里带上：

```text
Authorization: Bearer <wrapped-key>
```

如果你的客户端是 JSON 配置风格，通常会长成下面这种形式：

```json
{
  "mcpServers": {
    "mcp-prism-remote": {
      "transport": "http",
      "url": "https://your-domain.com/mcp",
      "headers": {
        "Authorization": "Bearer <wrapped-key>"
      }
    }
  }
}
```

如果客户端支持 WebSocket，也可以改成：

```json
{
  "mcpServers": {
    "mcp-prism-remote": {
      "transport": "websocket",
      "url": "wss://your-domain.com/mcp/ws",
      "headers": {
        "Authorization": "Bearer <wrapped-key>"
      }
    }
  }
}
```

注意：

- 不同客户端字段名可能不完全一样
- 但核心信息就三样：
  - 远程 MCP URL
  - 传输方式
  - `Authorization: Bearer <wrapped-key>`

## 什么时候用哪种方式

| 场景 | 推荐方式 |
|------|----------|
| 本机开发、同机 Agent | `stdio` |
| 远程 VPS 给多个 Agent 共用 | HTTP / WebSocket |
| 客户端只支持流式 HTTP | SSE |

## 一句话理解

- **Rust 服务怎么配**：看 `.env`
- **Agent 怎么连它**：看你的 MCP 客户端配置文件，把地址指向 `/mcp`、`/mcp/ws` 或 `/mcp/sse`
- **Node 常见的 `config.json` 替代物**：在这个项目里不是服务端配置，而是 Agent 客户端自己的 MCP 连接配置

## 品牌物料

- Logo 源文件：`materials/mcp-prism-logo.png`（棱镜折射光谱意象，适用于 README、演示与浅色/深色背景排版）。

## 许可与贡献

许可协议与贡献指南随 1.0 代码发布补充；在此之前欢迎通过 Issue 讨论适配器优先级与部署场景。

---

**MCP Prism** — *Refract many sources. Expose one door.*
