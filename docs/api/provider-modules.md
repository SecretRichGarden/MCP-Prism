# MCP Prism Provider 模块拆分

> 最后更新: 2026-03-28 | 范围: MVP / P0

## 模块总览

MCP Prism 按 provider 能力域拆成四个核心模块：

1. `web_search`
2. `academic`
3. `finance`
4. `maps`

路由层不会要求所有模块都配置完成。它只会根据当前环境下真正可用的 provider 进行调度。

## `web_search`

### Brave Search

- 认证: `X-Subscription-Token`
- 模块实现:
  - `BraveAdapter`
- 官方端点分组:
  - `GET /res/v1/web/search`
  - `GET /res/v1/news/search`
- 适用任务:
  - Web 搜索
  - 新闻检索

### Tavily

- 认证: Bearer
- 模块实现:
  - `TavilyAdapter`
- 官方端点分组:
  - `POST /search`
  - `POST /extract`
  - `POST /crawl`
  - `POST /map`
  - `POST /research`
- MVP 实际接入:
  - `/search`

### 智谱 Web Search

- 认证: Bearer
- 模块实现:
  - `ZhipuAdapter`
- 官方端点:
  - `POST /api/paas/v4/web_search`
- MVP 实际接入:
  - `search_std`
  - `search_pro_quark` 作为新闻优先分支

### 秘塔搜索

- 认证: 可配置，默认按无认证 HTTP JSON 实验模块处理
- 模块实现:
  - `MetasoAdapter`
- 调研端点:
  - `POST /api/search`
  - `POST /api/reader`
- MVP 实际接入:
  - `/api/search`
- 说明:
  - 因公开文档较少，保留为可配置实验 provider

## `academic`

### PubMed

- 认证: 可选 `api_key`
- 模块实现:
  - `PubMedAdapter`
- 官方调用链:
  - `GET /esearch.fcgi`
  - `GET /esummary.fcgi`
- 模块策略:
  - 先拿 PMID 列表，再批量拿摘要元数据

### OpenAlex

- 认证: 无 key 也可用，可选 `mailto`
- 模块实现:
  - `OpenAlexAdapter`
- 官方端点:
  - `GET /works`
- MVP 实际接入:
  - `search`
  - `per-page`
  - `mailto`

## `finance`

### FinanceMCP Bridge

- 认证: 可选 Bearer / token
- 模块实现:
  - `FinanceMcpAdapter`
- 上游形态:
  - 远端 HTTP/MCP bridge
- MVP 请求格式:
  - `POST <FINANCE_MCP_URL>`
  - body:
    - `tool`
    - `arguments.query`
    - `arguments.limit`
    - `arguments.extras`
- 说明:
  - 这样可以兼容本地已有 FinanceMCP 服务，避免在 Prism 内重写全部金融数据协议

## `business`

### GitHub REST Search

- 认证: PAT / Bearer
- 模块实现:
  - `GitHubRestAdapter`
- 端点:
  - `GET /search/repositories`
  - `GET /search/code`
  - `GET /search/issues`
  - `GET /search/users`

### GitHub GraphQL Search

- 认证: PAT / Bearer
- 模块实现:
  - `GitHubGraphqlAdapter`
- 端点:
  - `POST /graphql`

## `government`

### Data.gov

- 模块实现:
  - `DataGovAdapter`
- 端点:
  - `GET /package_search`

### openFDA

- 模块实现:
  - `OpenFdaAdapter`
- 端点:
  - `GET /drug/label.json`
  - 其他 dataset 可通过 `options.extras.dataset` 覆写

### U.S. Census

- 模块实现:
  - `CensusAdapter`
- 端点:
  - `GET /{year}/{dataset}`
- 说明:
  - 依赖 `get` / `for` / `year` / `dataset` 等参数

## `maps`

### 高德地图

- 认证: `key`
- 模块实现:
  - `AmapAdapter`
- 调研端点:
  - `GET /place/text`
  - `GET /geocode/geo`
  - `GET /weather/weatherInfo`
  - `GET /distance`
- MVP `operation` 映射:
  - `text_search`
  - `geocode`
  - `weather`
  - `distance`

## 双线路由

### Small-model line

- 输入:
  - query
  - search_type
  - provider_hints
  - 当前可用 provider 目录
- 输出:
  - `search_type`
  - `primary_sources`
  - `secondary_sources`
  - `strategy`
  - `reasoning`

### Linear fallback line

- 不依赖模型
- 用 query 关键词 + provider 能力矩阵直接决定调度顺序
- 覆盖:
  - web
  - news
  - academic
  - finance
  - maps
  - company / patent / government 的一般调研场景

### Experiment line

- 通过 `MCP_PRISM_ROUTER_EXPERIMENTS` 注入加权变体
- 用于灰度切流、A/B 对比和 provider 顺序试验
- 命中后会在响应 `decision.experiment` 中标记
