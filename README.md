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

## 快速开始（1.0 发布后）

> 以下命令将在首个可执行发行版就绪后与本仓库 `Cargo`/容器镜像对齐；当前阶段以 PRD 与适配器实现为准。

1. 在 VPS 上准备环境变量（聚合访问密钥、各下游 Key、可选 Redis）。
2. 启动 MCP Prism 服务进程（或容器），对外暴露 MCP 传输（如 stdio 由进程管理器托管，或 Streamable HTTP / WebSocket，以最终实现为准）。
3. 在 Agent 客户端中 **仅添加一个 MCP 服务器**，填入服务端颁发的 **一把** 客户端 Key。

具体安装包、配置项名与默认端口见后续 `docs/DEPLOYMENT.md`（随实现提交）。

## 品牌物料

- Logo 源文件：`materials/mcp-prism-logo.png`（棱镜折射光谱意象，适用于 README、演示与浅色/深色背景排版）。

## 许可与贡献

许可协议与贡献指南随 1.0 代码发布补充；在此之前欢迎通过 Issue 讨论适配器优先级与部署场景。

---

**MCP Prism** — *Refract many sources. Expose one door.*
