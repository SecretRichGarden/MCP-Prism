# MCP池Rust架构项目 - 产品需求文档 (PRD)

**项目名称**: MCP Pool Rust Architecture
**版本**: v1.0
**撰写人**: 小虾 🦞
**撰写日期**: 2026-03-28
**项目周期**: 预计3-4个月

---

## 📋 目录

- [1. 项目概述](#1-项目概述)
- [2. 核心架构设计](#2-核心架构设计)
- [3. 功能需求](#3-功能需求)
- [4. 非功能需求](#4-非功能需求)
- [5. API端点清单](#5-api端点清单)
- [6. 数据模型设计](#6-数据模型设计)
- [7. 技术栈选型](#7-技术栈选型)
- [8. 安全设计](#8-安全设计)
- [9. 部署架构](#9-部署架构)
- [10. 开发里程碑](#10-开发里程碑)
- [11. 风险与缓解](#11-风险与缓解)
- [12. 附录：API平台详细清单](#12-附录api平台详细清单)

---

## 1. 项目概述

### 1.1 项目愿景

构建一个**高质量MCP池聚合wrapper服务**，将多个外部API统一封装为单一MCP接口，部署在外网VPS上，为各种AI agent提供高效、安全、零上下文开销的信息检索能力。

### 1.2 核心价值主张

| 维度 | 现状 | 本方案 |
|------|------|--------|
| **配置复杂度** | 每个agent需要配置几十个MCP工具 | 仅需1个MCP接口 + 1个API key |
| **上下文占用** | MCP tool definitions占大量token | 零占用，所有工具定义在wrapper |
| **开发效率** | 需要理解和配置每个MCP工具 | 统一接口，开箱即用 |
| **性能** | 取决于本地网络和MCP server | Rust并发，外网VPS直连 |
| **安全性** | API key分散在各个配置文件 | 集中管理，加密存储 |
| **可维护性** | 需要更新每个MCP工具配置 | 统一更新wrapper配置 |

### 1.3 目标用户

- **主要用户**: 使用OpenClaw、Claude Desktop、Cursor等MCP兼容框架的开发者
- **次要用户**: 需要信息检索能力的AI agent开发者

---

## 2. 核心架构设计

### 2.1 整体架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                        Agent Layer                             │
│  (Claude Desktop / OpenClaw / Cursor / Custom Agent)          │
└────────────────────────┬────────────────────────────────────────┘
                         │ MCP Protocol
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Unified MCP Interface                        │
│  ┌────────────────────────────────────────────────────────┐   │
│  │  Single MCP Endpoint                                  │   │
│  │  - mcp_call(tool, args)                              │   │
│  │  - Single wrapped API Key                             │   │
│  └────────────────────────────────────────────────────────┘   │
└────────────────────────┬────────────────────────────────────────┘
                         │ HTTPS / WebSocket
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│              MCP Pool Rust Service (VPS)                      │
│  ┌────────────────────────────────────────────────────────┐   │
│  │  1. Authentication & Authorization Layer              │   │
│  │     - API Key validation                            │   │
│  │     - Rate limiting per client                       │   │
│  └────────────────────────────────────────────────────────┘   │
│  ┌────────────────────────────────────────────────────────┐   │
│  │  2. Intelligent Request Router (LLM-powered)         │   │
│  │     - Query analysis and classification              │   │
│  │     - Multi-hop routing strategy                    │   │
│  │     - Parallel API dispatch                         │   │
│  └────────────────────────────────────────────────────────┘   │
│  ┌────────────────────────────────────────────────────────┐   │
│  │  3. MCP Adapter Pool                              │   │
│  │     ├── Search Engines (4 adapters)               │   │
│  │     ├── Finance Data (1 adapter)                   │   │
│  │     ├── Academic Databases (2 adapters)            │   │
│  │     ├── Maps & Location (1 adapter)                │   │
│  │     ├── Business APIs (35+ adapters)               │   │
│  │     ├── Government APIs (17+ adapters)              │   │
│  │     └── LLM Model APIs (5+ adapters)               │   │
│  └────────────────────────────────────────────────────────┘   │
│  ┌────────────────────────────────────────────────────────┐   │
│  │  4. Response Normalization Layer                   │   │
│  │     - JSON schema normalization                     │   │
│  │     - Error handling and retry                     │   │
│  │     - Caching layer (Redis)                       │   │
│  └────────────────────────────────────────────────────────┘   │
└────────────────────────┬────────────────────────────────────────┘
                         │ HTTP / gRPC / SSE
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    External API Layer                          │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐   │
│  │ Search   │ Finance  │ Academic │ Gov Data │ Business │   │
│  │ Engines  │ APIs     │ DBs      │ APIs     │ APIs     │   │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 关键组件说明

#### 2.2.1 统一MCP接口层

**职责**:
- 接收来自agent的MCP协议请求
- 验证wrapped API key
- 转换为内部调用格式
- 返回标准化的响应

**技术要点**:
- 实现MCP协议（JSON-RPC 2.0 over stdio/websocket）
- 支持MCP tool list discovery（动态返回可用工具）
- 支持MCP resource list discovery（如需要）
- 支持流式响应（SSE）

**暴露的工具列表**（示例）:
```json
{
  "tools": [
    {
      "name": "unified_search",
      "description": "Unified search across multiple engines and databases",
      "inputSchema": {
        "type": "object",
        "properties": {
          "query": {"type": "string"},
          "scope": {"type": "string", "enum": ["web", "news", "academic", "finance", "gov"]},
          "limit": {"type": "number", "default": 10}
        }
      }
    },
    {
      "name": "company_info",
      "description": "Query company information from multiple sources",
      "inputSchema": {
        "type": "object",
        "properties": {
          "company_name": {"type": "string"},
          "country": {"type": "string", "default": "CN"}
        }
      }
    },
    {
      "name": "patent_search",
      "description": "Search patents from global databases",
      "inputSchema": {
        "type": "object",
        "properties": {
          "query": {"type": "string"},
          "country": {"type": "string", "default": "global"}
        }
      }
    }
  ]
}
```

#### 2.2.2 智能请求路由层（LLM-powered）

**职责**:
- 分析agent请求的语义和意图
- 选择最合适的下游API
- 决定并行/串行调用策略
- 处理多跳查询（如：先搜索公司 → 再查询专利 → 再分析财务）

**技术选型**:
- **小模型**: 使用轻量级LLM（如 Qwen2.5-7B、DeepSeek-7B）
- **推理框架**: Candle / onnxruntime / llama.cpp
- **调用方式**: 内部gRPC或本地进程调用

**路由策略示例**:

```rust
struct RouterDecision {
    primary_source: APIAdapter,        // 主要数据源
    secondary_sources: Vec<APIAdapter>, // 辅助数据源（并行调用）
    strategy: DispatchStrategy,         // 并行/串行/混合
    fallback: Option<APIAdapter>,       // 降级方案
}

enum DispatchStrategy {
    Parallel,    // 多API并行调用，结果聚合
    Sequential,   // 串行调用，前一个结果作为后一个输入
    Hybrid,       // 混合策略
}
```

**提示词示例**:

```
You are an intelligent API router. Given the user query, determine:
1. Which API adapters to call
2. Whether to call them in parallel or sequence
3. Fallback options

User query: "查询阿里巴巴的专利情况和财务表现"

Your output (JSON format):
{
  "primary_sources": ["patent_search", "company_financials"],
  "secondary_sources": ["company_info", "news_search"],
  "strategy": "parallel",
  "reasoning": "The query requires both patent and financial data, which can be fetched independently."
}
```

#### 2.2.3 MCP适配器池

**职责**:
- 将下游API请求转换为标准化的内部调用
- 处理不同API的认证、签名、限流
- 实现重试、降级、缓存逻辑

**适配器结构**:

```rust
trait APIAdapter {
    async fn call(&self, params: &SearchParams) -> Result<APIResponse, APIError>;
    fn name(&self) -> &str;
    fn rate_limit(&self) -> RateLimitConfig;
    fn supports_search_type(&self, search_type: SearchType) -> bool;
}

struct SearchAdapter {
    name: String,
    api_client: HttpClient,
    api_key: Secret<String>,
    rate_limiter: RateLimiter,
    cache: Cache,
}

impl APIAdapter for SearchAdapter {
    async fn call(&self, params: &SearchParams) -> Result<APIResponse, APIError> {
        // 1. Check cache
        if let Some(cached) = self.cache.get(params).await {
            return Ok(cached);
        }

        // 2. Check rate limit
        self.rate_limiter.acquire().await?;

        // 3. Build request (API-specific)
        let request = self.build_request(params)?;

        // 4. Send request
        let response = self.api_client.send(request).await?;

        // 5. Parse and normalize response
        let normalized = self.normalize_response(response).await?;

        // 6. Cache result
        self.cache.set(params, &normalized).await;

        Ok(normalized)
    }
}
```

**适配器注册表**:

```rust
struct AdapterRegistry {
    search_engines: Vec<Box<dyn APIAdapter>>,
    finance_adapters: Vec<Box<dyn APIAdapter>>,
    academic_adapters: Vec<Box<dyn APIAdapter>>,
    business_adapters: Vec<Box<dyn APIAdapter>>,
    gov_adapters: Vec<Box<dyn APIAdapter>>,
}

impl AdapterRegistry {
    fn new() -> Self {
        Self {
            search_engines: vec![
                Box::new(BraveAdapter::new()),
                Box::new(TavilyAdapter::new()),
                Box::new(ZhipuAdapter::new()),
                Box::new(MetasoAdapter::new()),
            ],
            finance_adapters: vec![
                Box::new(FinanceMCPAdapter::new()),
            ],
            // ... 其他适配器
        }
    }

    fn get_adapters(&self, search_type: SearchType) -> Vec<&dyn APIAdapter> {
        match search_type {
            SearchType::Web => self.search_engines.iter().map(|a| a.as_ref()).collect(),
            SearchType::Finance => self.finance_adapters.iter().map(|a| a.as_ref()).collect(),
            // ...
        }
    }
}
```

#### 2.2.4 响应标准化层

**职责**:
- 将不同API的响应格式统一为标准JSON schema
- 处理错误和异常
- 实现智能降级和重试

**标准化响应格式**:

```rust
#[derive(Serialize, Deserialize)]
struct UnifiedResponse {
    meta: ResponseMeta,
    data: Vec<SearchResult>,
    sources: Vec<SourceInfo>,
    timing: TimingInfo,
}

#[derive(Serialize, Deserialize)]
struct SearchResult {
    title: String,
    url: String,
    snippet: String,
    score: f64,
    source: String,      // API来源
    timestamp: DateTime,
    metadata: HashMap<String, serde_json::Value>, // 额外字段
}

#[derive(Serialize, Deserialize)]
struct ResponseMeta {
    query: String,
    total_results: usize,
    page: usize,
    sources_used: Vec<String>,
    fallback_used: Option<String>,
}
```

---

## 3. 功能需求

### 3.1 核心功能（MVP）

#### FR-1: 统一MCP协议支持

**优先级**: P0
**描述**: 实现完整的MCP协议，支持与MCP兼容的agent框架无缝对接

**子需求**:
- FR-1.1: 支持JSON-RPC 2.0 over stdio
- FR-1.2: 支持JSON-RPC 2.0 over websocket
- FR-1.3: 支持tool list discovery
- FR-1.4: 支持tool invocation
- FR-1.5: 支持流式响应（SSE）

**验收标准**:
- 可以在Claude Desktop中成功注册为MCP server
- 可以在OpenClaw中成功配置并调用
- 响应延迟< 500ms（不含下游API）

#### FR-2: 智能请求路由

**优先级**: P0
**描述**: 使用小模型分析请求语义，自动选择最优API组合

**子需求**:
- FR-2.1: 查询意图分类（web搜索/学术/金融/法律/商业等）
- FR-2.2: 多源路由决策（单API/多API并行/多API串行）
- FR-2.3: 智能降级（主API失败时自动切换到备用API）
- FR-2.4: 结果聚合和去重

**验收标准**:
- 路由决策准确率> 85%（基于人工标注的测试集）
- 平均路由延迟< 200ms
- 主API失败时降级成功率> 90%

#### FR-3: 现有MCP工具集成

**优先级**: P0
**描述**: 集成文档中的8个现有MCP工具

**子需求**:
- FR-3.1: Brave Search集成
- FR-3.2: Tavily Search集成
- FR-3.3: 智谱搜索集成
- FR-3.4: 秘塔搜索集成
- FR-3.5: FinanceMCP集成
- FR-3.6: PubMed集成
- FR-3.7: OpenAlex集成
- FR-3.8: 高德地图集成

**验收标准**:
- 所有适配器通过单元测试
- 所有适配器通过集成测试
- 端到端调用成功率> 95%

#### FR-4: API凭据管理

**优先级**: P0
**描述**: 安全存储和管理所有下游API的凭据

**子需求**:
- FR-4.1: 加密存储API key（使用AES-256-GCM）
- FR-4.2: 支持环境变量和配置文件
- FR-4.3: 支持运行时热更新（不重启服务）
- FR-4.4: 支持API key轮换

**验收标准**:
- API key不以明文形式存储在磁盘
- 运行时更新API key无需重启服务
- 更新后10秒内生效

#### FR-5: 统一认证和授权

**优先级**: P0
**描述**: 为agent提供统一的认证机制

**子需求**:
- FR-5.1: 支持wrapped API key
- FR-5.2: 支持API key过期时间
- FR-5.3: 支持rate limiting per client
- FR-5.4: 支持API key撤销

**验收标准**:
- 支持至少1000个不同的wrapped API key
- Rate limiting精度误差< 5%
- 撤销后5秒内生效

### 3.2 扩展功能（v1.1）

#### FR-6: 商业API集成

**优先级**: P1
**描述**: 集成高价值的商业API

**子需求**:
- FR-6.1: 天眼查/企查查集成
- FR-6.2: GitHub API集成
- FR-6.3: 专利API集成（Google Patents/USPTO/CNIPA）
- FR-6.4: 北大法宝集成
- FR-6.5: 微博/抖音API集成

**验收标准**:
- 每个API至少通过基本功能测试
- 认证和签名正确
- Rate limiting遵守API限制

#### FR-7: 政府数据API集成

**优先级**: P1
**描述**: 集成中美两国政府开放数据API

**子需求**:
- FR-7.1: Data.gov集成
- FR-7.2: 北京/上海/深圳数据平台集成
- FR-7.3: 美国联邦机构API（EPA/FDA/NIH/CDC）集成

**验收标准**:
- 至少支持3个政府数据源
- 数据格式正确解析
- 支持缓存以减少API调用

#### FR-8: 缓存层

**优先级**: P1
**描述**: 实现多级缓存，减少API调用和延迟

**子需求**:
- FR-8.1: 支持Redis缓存
- FR-8.2: 支持TTL配置（按查询类型）
- FR-8.3: 支持缓存预热（热门查询）
- FR-8.4: 支持缓存失效策略

**验收标准**:
- 缓存命中率> 30%
- 缓存查询延迟< 10ms
- TTL配置灵活支持不同查询类型

### 3.3 高级功能（v1.2）

#### FR-9: 监控和可观测性

**优先级**: P2
**描述**: 提供完善的监控和日志系统

**子需求**:
- FR-9.1: Prometheus指标导出
- FR-9.2: 结构化日志（JSON格式）
- FR-9.3: 分布式追踪（Jaeger/Zipkin）
- FR-9.4: 告警规则配置

**验收标准**:
- 关键指标覆盖> 90%
- 日志包含请求ID用于追踪
- 告警准确率> 80%

#### FR-10: A/B测试和灰度发布

**优先级**: P2
**描述**: 支持对不同API版本或策略进行A/B测试

**子需求**:
- FR-10.1: 支持基于请求特征的流量分配
- FR-10.2: 支持实时切换API版本
- FR-10.3: 支持结果质量对比

**验收标准**:
- 流量分配精度误差< 2%
- 切换延迟< 5秒

---

## 4. 非功能需求

### 4.1 性能需求

| 指标 | 目标值 | 测量方法 |
|------|--------|----------|
| P99响应延迟 | < 2s | 不含下游API调用时间 |
| P50响应延迟 | < 500ms | 不含下游API调用时间 |
| 路由决策延迟 | < 200ms | 使用小模型进行路由 |
| 并发请求数 | > 1000 qps | 单实例 |
| 内存占用 | < 2GB | 空闲状态 |
| CPU占用 | < 30% | 平均负载（不含路由LLM） |

### 4.2 可用性需求

| 指标 | 目标值 | 说明 |
|------|--------|------|
| 系统可用性 | > 99.5% | 每月停机时间< 3.6小时 |
| 单点故障恢复时间 | < 30s | 主API故障时切换到备用API |
| 数据持久化 | 99.99% | 配置和日志数据 |

### 4.3 可扩展性需求

| 维度 | 目标值 | 说明 |
|------|--------|------|
| 水平扩展 | 支持多实例部署 | 使用共享Redis缓存 |
| 垂直扩展 | 支持动态增加API适配器 | 无需重启服务 |
| 配置扩展 | 支持100+ API配置 | 配置文件和环境变量 |

### 4.4 安全需求

| 指标 | 目标值 | 说明 |
|------|--------|------|
| API key加密 | AES-256-GCM | 所有API key加密存储 |
| 传输加密 | TLS 1.3 | 所有通信使用HTTPS |
| 认证强度 | API key + 可选IP白名单 | 防止未授权访问 |
| 审计日志 | 完整记录所有API调用 | 用于安全审计和追溯 |

### 4.5 兼容性需求

| 项目 | 目标值 | 说明 |
|------|--------|------|
| MCP协议版本 | 0.4+ | 兼容主流MCP框架 |
| Rust版本 | 1.70+ | 使用稳定版Rust |
| 部署环境 | Linux (Ubuntu 22.04+) | 支持主流Linux发行版 |
| 数据库 | Redis 6+ | 缓存层 |

---

## 5. API端点清单

### 5.1 现有MCP工具端点汇总

#### 5.1.1 Brave Search

| 工具名称 | 功能描述 | 数据类型 | 文档URL | 认证方式 |
|---------|---------|---------|---------|---------|
| brave_web_search | 通用网页搜索 | JSON (WebSearchResult) | https://brave.com/search/api | API Key |
| brave_local_search | 本地商家搜索 | JSON (LocalBusiness) | https://brave.com/search/api | API Key |
| brave_video_search | 视频搜索 | JSON (VideoResult) | https://brave.com/search/api | API Key |
| brave_image_search | 图片搜索 | JSON (ImageResult) | https://brave.com/search/api | API Key |
| brave_news_search | 新闻搜索 | JSON (NewsResult) | https://brave.com/search/api | API Key |
| brave_summarizer | 搜索结果摘要 | JSON (Summary) | https://brave.com/search/api | API Key (Pro) |

#### 5.1.2 Tavily Search

| 工具名称 | 功能描述 | 数据类型 | 文档URL | 认证方式 |
|---------|---------|---------|---------|---------|
| tavily_search | 深度搜索 | JSON (SearchResponse) | https://docs.tavily.com/docs/tavily-api | API Key |
| tavily_extract | URL内容提取 | JSON (ExtractResponse) | https://docs.tavily.com/docs/tavily-api | API Key |
| tavily_crawl | 网站爬虫 | JSON (CrawlResponse) | https://docs.tavily.com/docs/tavily-api | API Key |
| tavily_map | 网站结构映射 | JSON (MapResponse) | https://docs.tavily.com/docs/tavily-api | API Key |
| tavily_research | 综合研究 | JSON (ResearchResponse) | https://docs.tavily.com/docs/tavily-api | API Key |

#### 5.1.3 智谱搜索

| 工具名称 | 功能描述 | 数据类型 | 文档URL | 认证方式 |
|---------|---------|---------|---------|---------|
| webSearchPro | 专业搜索 | SSE (SearchStream) | https://open.bigmodel.cn/dev/api#search | API Key |
| webSearchStd | 标准搜索 | SSE (SearchStream) | https://open.bigmodel.cn/dev/api#search | API Key |
| webSearchSogou | 搜狗引擎 | SSE (SearchStream) | https://open.bigmodel.cn/dev/api#search | API Key |
| webSearchQuark | 夸克引擎 | SSE (SearchStream) | https://open.bigmodel.cn/dev/api#search | API Key |

#### 5.1.4 秘塔搜索

| 工具名称 | 功能描述 | 数据类型 | 文档URL | 认证方式 |
|---------|---------|---------|---------|---------|
| metaso_search | 多维度搜索 | JSON/Markdown | https://metaso.cn/docs | API Key |
| metaso_reader | 网页内容提取 | JSON/Markdown | https://metaso.cn/docs | API Key |

#### 5.1.5 FinanceMCP

| 工具名称 | 功能描述 | 数据类型 | 文档URL | 认证方式 |
|---------|---------|---------|---------|---------|
| stock_data | 股票行情 | JSON (StockKLine) | Tushare API | Token |
| company_performance | 公司综合表现 | JSON (CompanyInfo) | Tushare API | Token |
| macro_econ | 宏观经济数据 | JSON (MacroData) | Tushare API | Token |
| money_flow | 资金流向 | JSON (MoneyFlow) | Tushare API | Token |
| margin_trade | 融资融券 | JSON (MarginData) | Tushare API | Token |
| fund_data | 基金数据 | JSON (FundInfo) | Tushare API | Token |

#### 5.1.6 PubMed

| 工具名称 | 功能描述 | 数据类型 | 文档URL | 认证方式 |
|---------|---------|---------|---------|---------|
| pubmed_search | 文献搜索 | JSON (SearchResult) | https://www.ncbi.nlm.nih.gov/books/NBK25501/ | API Key (可选) |
| pubmed_get_details | 获取文献详情 | JSON (ArticleDetail) | https://www.ncbi.nlm.nih.gov/books/NBK25501/ | API Key (可选) |
| pubmed_batch_query | 批量查询 | JSON (BatchResult) | https://www.ncbi.nlm.nih.gov/books/NBK25501/ | API Key (可选) |
| pubmed_download_fulltext | 下载全文PDF | Binary (PDF) | https://www.ncbi.nlm.nih.gov/books/NBK25501/ | API Key (可选) |

#### 5.1.7 OpenAlex

| 工具名称 | 功能描述 | 数据类型 | 文档URL | 认证方式 |
|---------|---------|---------|---------|---------|
| openalex_search | 论文搜索 | JSON (Work) | https://docs.openalex.org/ | Email (可选) |
| openalex_get_work | 获取论文详情 | JSON (Work) | https://docs.openalex.org/ | Email (可选) |
| openalex_batch_get_works | 批量获取 | JSON (Works) | https://docs.openalex.org/ | Email (可选) |
| openalex_download_fulltext | 下载全文 | Binary (PDF) | https://docs.openalex.org/ | Email (可选) |

#### 5.1.8 高德地图

| 工具名称 | 功能描述 | 数据类型 | 文档URL | 认证方式 |
|---------|---------|---------|---------|---------|
| maps_geo | 地址转经纬度 | JSON (GeoCode) | https://lbs.amap.com/api/ | API Key + Digital Signature |
| maps_regeocode | 经纬度转地址 | JSON (ReGeoCode) | https://lbs.amap.com/api/ | API Key + Digital Signature |
| maps_text_search | POI搜索 | JSON (POIResult) | https://lbs.amap.com/api/ | API Key + Digital Signature |
| maps_direction_driving | 驾车路径规划 | JSON (Route) | https://lbs.amap.com/api/ | API Key + Digital Signature |
| maps_weather | 城市天气查询 | JSON (Weather) | https://lbs.amap.com/api/ | API Key + Digital Signature |

---

## 6. 数据模型设计

### 6.1 统一请求格式

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSearchRequest {
    pub query: String,
    pub search_type: SearchType,
    pub options: SearchOptions,
    pub client_id: Option<String>,  // Wrapped API key中的client标识
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchType {
    Web,
    News,
    Video,
    Image,
    Academic,
    Finance,
    Company,
    Patent,
    Government,
    Maps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub time_range: Option<TimeRange>,
    pub language: Option<String>,
    pub region: Option<String>,
    pub include_raw_content: Option<bool>,
    pub max_depth: Option<usize>,  // For crawl operations
}
```

### 6.2 统一响应格式

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedResponse {
    pub request_id: String,
    pub query: String,
    pub meta: ResponseMeta,
    pub results: Vec<UnifiedResult>,
    pub timing: ResponseTiming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub total_results: usize,
    pub sources_used: Vec<SourceInfo>,
    pub fallback_used: Option<String>,
    pub cached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub name: String,
    pub results_count: usize,
    pub latency_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedResult {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    pub snippet: String,
    pub score: f64,
    pub source: String,  // API来源
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTiming {
    pub routing_ms: u64,
    pub total_api_ms: u64,
    pub normalization_ms: u64,
    pub total_ms: u64,
}
```

### 6.3 路由决策格式

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub request_id: String,
    pub query: String,
    pub classification: QueryClassification,
    pub dispatch_plan: DispatchPlan,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryClassification {
    pub primary_intent: IntentType,
    pub secondary_intents: Vec<IntentType>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntentType {
    WebSearch,
    NewsSearch,
    AcademicSearch,
    FinanceSearch,
    CompanyInfo,
    PatentSearch,
    LegalSearch,
    GovernmentData,
    LocationQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchPlan {
    pub primary_sources: Vec<AdapterId>,
    pub secondary_sources: Vec<AdapterId>,
    pub strategy: DispatchStrategy,
    pub fallback: Option<AdapterId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DispatchStrategy {
    Parallel { timeout_ms: u64 },
    Sequential { chain: Vec<AdapterId> },
    Hybrid { parallel: Vec<AdapterId>, sequential: Vec<AdapterId> },
}
```

### 6.4 缓存键设计

```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CacheKey {
    pub adapter_id: String,
    pub query_hash: String,
    pub options_hash: String,
    pub time_bucket: String,  // For time-sensitive queries
}

impl CacheKey {
    pub fn new(adapter_id: &str, request: &UnifiedSearchRequest) -> Self {
        Self {
            adapter_id: adapter_id.to_string(),
            query_hash: Self::hash_string(&request.query),
            options_hash: Self::hash_string(&serde_json::to_string(&request.options).unwrap_or_default()),
            time_bucket: Self::get_time_bucket(&request.options.time_range),
        }
    }

    fn hash_string(s: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn get_time_bucket(time_range: &Option<TimeRange>) -> String {
        match time_range {
            Some(TimeRange::Day) => "1d".to_string(),
            Some(TimeRange::Week) => "1w".to_string(),
            Some(TimeRange::Month) => "1m".to_string(),
            Some(TimeRange::Year) => "1y".to_string(),
            None => "any".to_string(),
        }
    }
}
```

---

## 7. 技术栈选型

### 7.1 核心技术栈

| 组件 | 技术选型 | 版本 | 理由 |
|------|---------|------|------|
| **编程语言** | Rust | 1.70+ | 高性能、内存安全、并发能力强 |
| **异步运行时** | tokio | 1.35+ | 成熟稳定、生态丰富 |
| **HTTP客户端** | reqwest | 0.11+ | 异步、支持连接池、自动重试 |
| **Web框架** | axum | 0.7+ | 高性能、支持websocket |
| **序列化** | serde | 1.0+ | Rust生态标准 |
| **缓存** | Redis + redis-rs | 6+ | 高性能、支持TTL、发布订阅 |
| **LLM推理** | Candle / llama.cpp | latest | Rust本地推理、性能好 |
| **加密** | rust-openssl | 0.10+ | AES-256-GCM支持 |
| **日志** | tracing | 0.1+ | 结构化日志、支持分布式追踪 |
| **指标** | prometheus-client | 0.22+ | Prometheus指标导出 |

### 7.2 部署技术栈

| 组件 | 技术选型 | 版本 | 理由 |
|------|---------|------|------|
| **容器化** | Docker | 24+ | 跨平台、易于部署 |
| **编排** | Docker Compose | 2.20+ | 单机多容器编排 |
| **反向代理** | Nginx | 1.25+ | 支持TLS、负载均衡 |
| **进程管理** | systemd | latest | Linux标准 |
| **监控** | Prometheus + Grafana | latest | 指标采集和可视化 |

### 7.3 开发工具

| 工具 | 用途 |
|------|------|
| **cargo** | Rust包管理和构建 |
| **clippy** | Rust代码静态检查 |
| **rustfmt** | 代码格式化 |
| **cargo-test** | 单元测试和集成测试 |
| **cargo-tarpaulin** | 测试覆盖率 |

---

## 8. 安全设计

### 8.1 API Key加密存储

**加密方案**:
- 算法: AES-256-GCM
- 密钥派生: PBKDF2-HMAC-SHA256 (100,000 iterations)
- 密钥来源: 环境变量或硬件安全模块(HSM)

**实现示例**:

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

struct SecretStorage {
    master_key: Vec<u8>,
}

impl SecretStorage {
    fn encrypt_api_key(&self, api_key: &str) -> Result<String, CryptoError> {
        let key = Key::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(b"unique nonce"); // 实际应随机生成

        let ciphertext = cipher.encrypt(nonce, api_key.as_bytes())
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        Ok(hex::encode(ciphertext))
    }

    fn decrypt_api_key(&self, encrypted: &str) -> Result<String, CryptoError> {
        let key = Key::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(b"unique nonce");

        let ciphertext = hex::decode(encrypted)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

        Ok(String::from_utf8(plaintext)?)
    }
}
```

### 8.2 认证和授权

**Wrapped API Key格式**:

```
mcp_pool_v1:client_id:signature:expiry
```

**组成部分**:
- `mcp_pool_v1`: 版本标识
- `client_id`: 客户端唯一标识
- `signature`: HMAC-SHA256签名 (client_id + expiry)
- `expiry`: Unix时间戳 (秒级)

**验证流程**:

```rust
struct WrappedApiKey {
    version: String,
    client_id: String,
    signature: String,
    expiry: u64,
}

impl WrappedApiKey {
    fn validate(&self, secret: &[u8]) -> Result<(), AuthError> {
        // 1. 检查版本
        if self.version != "mcp_pool_v1" {
            return Err(AuthError::InvalidVersion);
        }

        // 2. 检查过期时间
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        if self.expiry < now {
            return Err(AuthError::TokenExpired);
        }

        // 3. 验证签名
        let data = format!("{}:{}", self.client_id, self.expiry);
        let expected_sig = hmac_sha256(secret, data.as_bytes());
        if self.signature != expected_sig {
            return Err(AuthError::InvalidSignature);
        }

        Ok(())
    }
}
```

### 8.3 Rate Limiting

**实现方案**: Token Bucket Algorithm

```rust
use std::time::Duration;
use std::collections::HashMap;
use tokio::sync::Mutex;

struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
}

struct TokenBucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64,
    last_update: Instant,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_update: Instant::now(),
        }
    }

    fn acquire(&mut self, tokens: f64) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();

        // Refill tokens
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_update = now;

        // Check if enough tokens
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }
}
```

### 8.4 审计日志

**日志格式** (JSON):

```json
{
  "timestamp": "2026-03-28T14:49:12.123Z",
  "level": "info",
  "request_id": "req_abc123",
  "client_id": "client_001",
  "event": "api_call",
  "adapter": "brave_search",
  "query": "搜索示例",
  "latency_ms": 234,
  "success": true,
  "cached": false
}
```

---

## 9. 部署架构

### 9.1 单实例部署

```
┌─────────────────────────────────────────────────────────────┐
│                    VPS (外网)                            │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │  Nginx     │  │   Redis     │  │   MCP Pool  │    │
│  │  (443/80)  │  │   (6379)   │  │   Service   │    │
│  └──────┬──────┘  └─────────────┘  └──────┬──────┘    │
│         │                                │             │
│         └───────────────┬────────────────┘             │
│                         ▼                              │
│              ┌─────────────────────┐                    │
│              │  Docker Compose   │                    │
│              └─────────────────────┘                    │
└─────────────────────────────────────────────────────────────┘
         │
         │ HTTPS
         ▼
┌─────────────────────────────────────────────────────────────┐
│               Agent (OpenClaw/Cursor)                    │
└─────────────────────────────────────────────────────────────┘
```

### 9.2 Docker Compose配置

```yaml
version: '3.8'

services:
  mcp-pool:
    build: .
    ports:
      - "127.0.0.1:8080:8080"
    environment:
      - RUST_LOG=info
      - REDIS_URL=redis://redis:6379
      - ENCRYPTION_KEY=${ENCRYPTION_KEY}
      - HMAC_SECRET=${HMAC_SECRET}
    depends_on:
      - redis
    volumes:
      - ./config:/app/config:ro
      - ./logs:/app/logs
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    ports:
      - "127.0.0.1:6379:6379"
    volumes:
      - redis-data:/data
    restart: unless-stopped

  nginx:
    image: nginx:1.25-alpine
    ports:
      - "443:443"
      - "80:80"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/ssl:/etc/nginx/ssl:ro
    depends_on:
      - mcp-pool
    restart: unless-stopped

volumes:
  redis-data:
```

### 9.3 Nginx配置

```nginx
events {
    worker_connections 1024;
}

http {
    upstream mcp_pool {
        server 127.0.0.1:8080;
    }

    # HTTP -> HTTPS redirect
    server {
        listen 80;
        server_name mcp-pool.example.com;
        return 301 https://$server_name$request_uri;
    }

    # HTTPS server
    server {
        listen 443 ssl http2;
        server_name mcp-pool.example.com;

        ssl_certificate /etc/nginx/ssl/fullchain.pem;
        ssl_certificate_key /etc/nginx/ssl/privkey.pem;
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers HIGH:!aNULL:!MD5;

        # MCP endpoint (WebSocket)
        location /mcp {
            proxy_pass http://mcp_pool;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_read_timeout 86400;
        }

        # Metrics endpoint
        location /metrics {
            proxy_pass http://mcp_pool/metrics;
            allow 127.0.0.1;
            deny all;
        }
    }
}
```

---

## 10. 开发里程碑

### Phase 1: MVP (6-8周)

**目标**: 完成核心功能，可以处理基本的搜索和API调用

| 任务 | 负责人 | 交付物 | 预计时间 |
|------|--------|--------|----------|
| 项目初始化和架构搭建 | - | Git仓库、目录结构、CI/CD | Week 1 |
| MCP协议实现 | - | JSON-RPC over stdio/websocket | Week 2-3 |
| API Key加密存储 | - | SecretStorage模块、配置加载 | Week 3 |
| 统一认证和授权 | - | Wrapped API Key验证 | Week 3-4 |
| 智能路由层 (简化版) | - | 基于规则的路由 (LLM后续) | Week 4-5 |
| 响应标准化层 | - | UnifiedResponse格式 | Week 5 |
| Brave Search适配器 | - | 完整适配器 | Week 5-6 |
| Tavily Search适配器 | - | 完整适配器 | Week 6 |
| 智谱搜索适配器 | - | 完整适配器 | Week 6-7 |
| 基础缓存层 | - | Redis集成 | Week 7 |
| 单元测试和集成测试 | - | 测试覆盖率> 80% | Week 8 |

**验收标准**:
- 可以在Claude Desktop中成功注册并调用
- 至少支持Brave和Tavily两个搜索引擎
- 响应延迟< 500ms
- 测试覆盖率> 80%

### Phase 2: 现有MCP工具集成 (4-6周)

**目标**: 集成文档中的所有现有MCP工具

| 任务 | 负责人 | 交付物 | 预计时间 |
|------|--------|--------|----------|
| 秘塔搜索适配器 | - | 完整适配器 | Week 9 |
| FinanceMCP适配器 | - | 完整适配器 | Week 9-10 |
| PubMed适配器 | - | 完整适配器 | Week 10-11 |
| OpenAlex适配器 | - | 完整适配器 | Week 11 |
| 高德地图适配器 | - | 完整适配器 | Week 12 |
| Rate Limiting增强 | - | Token Bucket算法 | Week 12 |
| 适配器注册表 | - | 动态加载机制 | Week 13 |
| 端到端测试 | - | 所有适配器测试 | Week 13-14 |

**验收标准**:
- 所有8个现有MCP工具适配器完成
- 端到端调用成功率> 95%
- 支持动态加载适配器（无需重启）

### Phase 3: LLM智能路由 (4-6周)

**目标**: 使用小模型实现智能路由决策

| 任务 | 负责人 | 交付物 | 预计时间 |
|------|--------|--------|----------|
| LLM模型选型和集成 | - | Candle/llama.cpp集成 | Week 15-16 |
| 路由提示词设计 | - | Prompt模板 | Week 16 |
| 路由决策模块 | - | RouterDecision模块 | Week 17 |
| 查询意图分类 | - | 意图分类器 | Week 18 |
| 多源路由策略 | - | 并行/串行/混合策略 | Week 19 |
| 路由效果评估 | - | 准确率测试 | Week 19-20 |

**验收标准**:
- 路由决策准确率> 85%
- 路由延迟< 200ms
- 支持并行和串行策略

### Phase 4: 扩展功能 (4-6周)

**目标**: 集成商业API和政府数据API

| 任务 | 负责人 | 交付物 | 预计时间 |
|------|--------|--------|----------|
| 天眼查/企查查适配器 | - | 完整适配器 | Week 21-22 |
| GitHub API适配器 | - | 完整适配器 | Week 22 |
| 专利API适配器 | - | 完整适配器 | Week 23 |
| 北大法宝适配器 | - | 完整适配器 | Week 23-24 |
| Data.gov适配器 | - | 完整适配器 | Week 24 |
| 中国政府数据适配器 | - | 完整适配器 | Week 25 |
| 缓存增强 | - | TTL配置、预热 | Week 25-26 |

**验收标准**:
- 至少集成5个新的API适配器
- 缓存命中率> 30%
- 支持TTL配置

### Phase 5: 生产化 (2-4周)

**目标**: 监控、日志、部署优化

| 任务 | 负责人 | 交付物 | 预计时间 |
|------|--------|--------|----------|
| Prometheus指标导出 | - | Metrics endpoint | Week 27 |
| 结构化日志 | - | tracing集成 | Week 27 |
| 分布式追踪 | - | Jaeger集成 | Week 28 |
| Docker镜像优化 | - | 多阶段构建 | Week 28-29 |
| 部署文档 | - | 完整部署指南 | Week 29 |
| 性能优化 | - | P99延迟优化 | Week 30 |

**验收标准**:
- 关键指标覆盖率> 90%
- P99响应延迟< 2s
- 完整的部署文档

---

## 11. 风险与缓解

### 11.1 技术风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| LLM路由延迟过高 | 中 | 高 | 1. 使用轻量级模型<br>2. 实现降级策略（规则路由）<br>3. 缓存路由决策 |
| API适配器开发复杂度高 | 高 | 中 | 1. 优先实现核心适配器<br>2. 复用代码框架<br>3. 充分测试 |
| 并发性能瓶颈 | 中 | 中 | 1. 使用tokio异步运行时<br>2. 连接池复用<br>3. 性能压测 |
| Rate Limiting冲突 | 中 | 中 | 1. 统一配额管理<br>2. 优先级队列<br>3. 降级策略 |

### 11.2 安全风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| API Key泄露 | 低 | 高 | 1. 加密存储<br>2. 定期轮换<br>3. 最小权限原则 |
| 未授权访问 | 中 | 高 | 1. 强认证<br>2. IP白名单（可选）<br>3. 审计日志 |
| DDoS攻击 | 中 | 中 | 1. Rate Limiting<br>2. Nginx限流<br>3. CDN防护 |

### 11.3 运维风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 单点故障 | 中 | 高 | 1. 健康检查<br>2. 自动重启<br>3. 多实例部署（后续） |
| 监控盲区 | 中 | 中 | 1. 完善指标<br>2. 告警规则<br>3. 日志审计 |
| 配置错误 | 低 | 高 | 1. 配置验证<br>2. 灰度发布<br>3. 回滚机制 |

---

## 12. 附录：API平台详细清单

### 12.1 完整的API平台-文档-数据类型清单

请参考下一章节的详细清单，包含所有60+个API的完整信息。

---

## 文档结束

**下一步行动**:
1. 等待并行agent完成API文档调研
2. 补充12章中的详细API清单
3. 生成开发任务清单
4. 开始Phase 1开发

---

*PRD撰写人: 小虾 🦞*
*最后更新: 2026-03-28*
*版本: v1.0*
