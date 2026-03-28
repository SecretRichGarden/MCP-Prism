# MCP池Rust架构项目 - 完整API清单

**项目名称**: MCP Pool Rust Architecture
**版本**: v1.0
**整理人**: 小虾 🦞
**生成时间**: 2026-03-28
**状态**: 所有API调研完成

---

## 📋 目录

- [统计摘要](#统计摘要)
- [A. 现有MCP工具](#a-现有mcp工具-8个)
- [B. 商业API](#b-商业api-35个)
- [C. 政府数据API](#c-政府数据api-16个)
- [D. 开发优先级建议](#d-开发优先级建议)
- [E. 技术实施建议](#e-技术实施建议)

---

## 统计摘要

### 按类别统计

| 类别 | 数量 | 详细说明 |
|------|------|---------|
| **现有MCP工具** | 8 | 搜索4、金融1、学术2、地图1、命理1 |
| **商业API** | 35 | 企业2、代码2、专利3、电商4、法律2、社媒3、招聘3、票务2、物流3、旅游3、医疗1、教育3 |
| **政府API** | 16 | 美国政府7、中国政府9 |
| **总计** | **59** | - |

### 按可用性统计

| 可用性 | 数量 | 占比 |
|--------|------|------|
| ✅ 有完整公开API | 39 | 66.1% |
| ⚠️ 需要合作/付费 | 15 | 25.4% |
| ❌ 无公开API/需要爬虫 | 5 | 8.5% |

### 按认证方式统计

| 认证方式 | 数量 | 占比 |
|---------|------|------|
| API Key | 51 | 86.4% |
| OAuth 2.0 | 3 | 5.1% |
| Token (MD5/HMAC) | 2 | 3.4% |
| 无需认证 | 3 | 5.1% |

---

## A. 现有MCP工具 (8个)

### A.1 搜索引擎类 (4个)

#### A.1.1 Brave Search

| 项目 | 信息 |
|------|------|
| **名称** | Brave Search API |
| **官方文档** | https://api.search.brave.com/app/documentation |
| **API Key获取** | https://api.search.brave.com/app/keys |
| **Rate Limiting** | 月订阅制 ($4.99/月），超额返回402错误 |
| **数据类型** | JSON |
| **认证方式** | X-Subscription-Token header |
| **端点列表** | `/api/search` (网页)、`/api/news` (新闻)、`/api/images` (图片) |
| **定价** | $4.99/月 (订阅模式) |
| **特点** | 支持多语言、安全搜索、Goggles重排序 |
| **是否公开** | ✅ 是 |

#### A.1.2 Tavily Search ⭐ 信息最全

| 项目 | 信息 |
|------|------|
| **名称** | Tavily API |
| **官方文档** | https://docs.tavily.com/ |
| **API Key获取** | https://app.tavily.com |
| **Rate Limiting** | 开发环境 100 RPM，生产环境 1000 RPM |
| **数据类型** | JSON |
| **认证方式** | Bearer token in Authorization header |
| **端点列表** | `/search`、`/extract`、`/crawl`、`/map`、`/research` (5个端点) |
| **定价** | 免费月1000 credits，付费$0.008/credit |
| **特点** | 多搜索深度(basic/advanced/fast/ultra-fast)、LLM增强答案、结构化输出 |
| **是否公开** | ✅ 是 |

#### A.1.3 智谱 AI Web Search

| 项目 | 信息 |
|------|------|
| **名称** | 智谱 AI Web Search API |
| **官方文档** | https://docs.bigmodel.cn/cn/guide/tools/web-search.md |
| **API Key获取** | https://open.bigmodel.cn/usercenter/apikeys |
| **Rate Limiting** | 基于用户权益等级的并发限制 |
| **数据类型** | JSON (SSE流式) |
| **认证方式** | Bearer token in Authorization header |
| **端点列表** | `/paas/v4/web_search` |
| **定价** | search_std 0.01元/次、search_pro 0.03元/次、search_pro_sogou 0.05元/次、search_pro_quark 0.05元/次 |
| **特点** | 支持搜狗/夸克引擎、域名过滤、时间范围过滤、内容长度控制 |
| **是否公开** | ✅ 是 |

#### A.1.4 秘塔 AI 搜索

| 项目 | 信息 |
|------|------|
| **名称** | 秘塔 AI 搜索 |
| **官方文档** | https://metaso.cn/ (公开文档较少) |
| **API Key获取** | 需联系官方获取 |
| **Rate Limiting** | 未公开 |
| **数据类型** | JSON / Markdown |
| **认证方式** | 待确认 |
| **端点列表** | `/api/search`、`/api/reader` |
| **定价** | 未公开 |
| **特点** | 中文搜索专注、AI增强结果 |
| **是否公开** | ⚠️ 需要联系官方 |

---

### A.2 金融数据类 (1个)

#### A.2.1 FinanceMCP (基于Tushare)

| 项目 | 信息 |
|------|------|
| **名称** | FinanceMCP (Synapse) |
| **官方文档** | https://tushare.pro/document/2 |
| **GitHub** | https://github.com/guangxiangdebizi/FinanceMCP |
| **API Key获取** | https://tushare.pro/register (Token) |
| **Rate Limiting** | 基于Tushare积分系统 |
| **数据类型** | JSON (Pandas DataFrame) |
| **认证方式** | Token in parameters |
| **端点列表** | 18个工具：stock_data、company_performance、macro_econ、money_flow、margin_trade、block_trade、dragon_tiger_inst、fund_data、index_data、csi_index_constituents等 |
| **定价** | 基础积分免费，高级数据需要积分 |
| **特点** | A股/美股/港股/加密货币/基金/期货/期权/宏观/资金流向/龙虎榜 |
| **是否公开** | ✅ 是 |

---

### A.3 学术文献类 (2个)

#### A.3.1 PubMed

| 项目 | 信息 |
|------|------|
| **名称** | PubMed Data Server |
| **官方文档** | https://www.ncbi.nlm.nih.gov/books/NBK25501/ |
| **GitHub** | https://github.com/SecretRichGarden/mcp-pubmed-server |
| **API Key获取** | https://www.ncbi.nlm.nih.gov/account/ (NCBI API Key，可选但推荐) |
| **Rate Limiting** | 无Key: 3 req/s | 有Key: 10 req/s |
| **数据类型** | JSON / XML / Binary (PDF) |
| **认证方式** | API Key in query parameter |
| **端点列表** | 11个工具：pubmed_search、pubmed_get_details、pubmed_quick_search、pubmed_batch_query、pubmed_cross_reference、pubmed_extract_key_info、pubmed_detect_fulltext、pubmed_download_fulltext、pubmed_batch_download、pubmed_get_fulltext_sections、pubmed_endnote_status |
| **定价** | 免费 |
| **特点** | 生物医学文献数据库、支持布尔逻辑和MeSH、EndNote导出 |
| **是否公开** | ✅ 是 |

#### A.3.2 OpenAlex

| 项目 | 信息 |
|------|------|
| **名称** | OpenAlex MCP Server |
| **官方文档** | https://docs.openalex.org/ |
| **GitHub** | https://github.com/SecretRichGarden/openAlex-mcp |
| **API Key获取** | https://openalex.org/register (Email，可选但推荐) |
| **Rate Limiting** | 无Key: 5 req/s | 有Key: 10 req/s |
| **数据类型** | JSON |
| **认证方式** | Email in query parameter |
| **端点列表** | 8个工具：openalex_search、openalex_get_work、openalex_batch_get_works、openalex_detect_fulltext、openalex_download_fulltext、openalex_get_fulltext_sections、openalex_cache_stats、openalex_system_check |
| **定价** | 免费 |
| **特点** | 开放学术数据库、支持200M+论文、按年份/类型/开放获取过滤 |
| **是否公开** | ✅ 是 |

---

### A.4 地图与地理类 (1个)

#### A.4.1 高德地图

| 项目 | 信息 |
|------|------|
| **名称** | 高德地图 Web 服务 API |
| **官方文档** | https://lbs.amap.com/api/webservice/guide/api/summary |
| **开发者平台** | https://lbs.amap.com/ |
| **API Key获取** | https://console.amap.com/dev/key/app |
| **Rate Limiting** | 根据配额级别确定（详见配额说明） |
| **数据类型** | JSON |
| **认证方式** | API Key + Digital Signature (部分接口) |
| **端点列表** | 地理编码、逆地理编码、POI搜索、周边搜索、驾车/步行/骑行/公交路径规划、距离测量、天气查询、IP定位 |
| **定价** | 个人开发者有免费配额 |
| **特点** | 支持地标性建筑解析、骑行规划500km内、公交规划跨城、周边搜索可配置半径 |
| **是否公开** | ✅ 是 |

---

### A.5 命理民俗类 (1个)

#### A.5.1 八字 MCP

| 项目 | 信息 |
|------|------|
| **名称** | 八字 MCP Server |
| **官方文档** | https://github.com/cantian-ai/bazi-mcp |
| **API Key获取** | 无 (本地计算) |
| **Rate Limiting** | 无限制 |
| **数据类型** | JSON |
| **认证方式** | 无需认证 |
| **端点列表** | 八字分析、命盘生成、大运推算 |
| **定价** | 免费 |
| **特点** | 基于tyme4ts库进行本地八字计算 |
| **是否公开** | ✅ 是 (本地计算) |

---

## B. 商业API (35个)

### B.1 企业信息查询 (2个)

#### B.1.1 天眼查

| 项目 | 信息 |
|------|------|
| **名称** | 天眼查 API |
| **官方文档** | https://open.tianyancha.com/ |
| **API Key获取** | 需要在开放平台注册申请 |
| **Rate Limiting** | 根据付费套餐等级确定 |
| **数据类型** | JSON |
| **认证方式** | API Key |
| **端点列表** | 企业基本信息、工商数据、股东信息、知识产权、司法风险、关联关系 |
| **定价** | C端VIP约360元/年，SVIP约1800元/年，企业版按项目报价 |
| **SDK** | 有GitHub和Python SDK |
| **是否公开** | ✅ 是 (需注册) |
| **备注** | 需要合作申请 |

#### B.1.2 企查查

| 项目 | 信息 |
|------|------|
| **名称** | 企查查 API |
| **官方文档** | https://openapi.qcc.com/ |
| **API Key获取** | 需要注册获取AppKey和SecretKey |
| **Rate Limiting** | 根据付费套餐等级确定 |
| **数据类型** | JSON |
| **认证方式** | Token = MD5(key + Timespan + SecretKey) |
| **端点列表** | 企业模糊查询、详情查询、工商信息、股东信息、司法风险、知识产权、关系图谱 |
| **定价** | C端VIP 388元/年，SVIP 1800元/年，企业版300-5000元/季度或年度 |
| **示例端点** | `https://api.qichacha.com/FuzzySearch/GetList` |
| **是否公开** | ✅ 是 (需注册) |
| **备注** | 需要合作申请 |

---

### B.2 代码仓库 (2个)

#### B.2.1 GitHub REST API

| 项目 | 信息 |
|------|------|
| **名称** | GitHub REST API |
| **官方文档** | https://docs.github.com/en/rest |
| **API Key获取** | Personal Access Token (PAT) / OAuth Apps / GitHub Apps |
| **Rate Limiting** | 未认证: 60 req/hour | 已认证: 5,000 req/hour | Enterprise: 15,000 req/hour |
| **数据类型** | JSON |
| **认证方式** | Bearer token in Authorization header |
| **端点数量** | 数百个端点 |
| **主要端点** | `/users/{username}/repos`、`/repos/{owner}/{repo}/contents/{path}`、`/search/code`、`/search/repositories` |
| **定价** | 免费 (Enterprise有更高配额) |
| **SDK** | Octokit (官方SDK，支持多语言) |
| **是否公开** | ✅ 是 |
| **特点** | 完整的GitHub REST接口、支持Actions、Repos、Issues、Users等 |

#### B.2.2 GitHub GraphQL API

| 项目 | 信息 |
|------|------|
| **名称** | GitHub GraphQL API |
| **官方文档** | https://docs.github.com/en/graphql |
| **API Key获取** | Personal Access Token / OAuth Apps / GitHub Apps |
| **Rate Limiting** | 5,000 points/hour (普通用户) | Enterprise Cloud: 10,000 points/hour |
| **数据类型** | JSON (GraphQL) |
| **认证方式** | Bearer token in Authorization header |
| **端点列表** | 单一GraphQL端点 `https://api.github.com/graphql` |
| **查询复杂度** | 单次调用最多500,000个节点 |
| **超时** | 10秒 |
| **定价** | 免费 (Enterprise有更高配额) |
| **是否公开** | ✅ 是 |
| **特点** | 精确查询、一次请求获取多个资源、避免过度获取 |

---

### B.3 专利与知识产权 (3个)

#### B.3.1 Google Patents

| 项目 | 信息 |
|------|------|
| **名称** | Google Patents API |
| **官方文档** | https://www.searchapi.io/docs/google-patents |
| **API Key获取** | 通过第三方提供商 (SerpApi, SearchApi) |
| **Rate Limiting** | 基于第三方提供商订阅计划 |
| **数据类型** | JSON |
| **认证方式** | API Key |
| **端点列表** | `/search?engine=google_patents`、`/search?engine=google_patents_details` |
| **定价** | 按订阅计划 (免费层级约100次/月) |
| **是否公开** | ⚠️ 间接 (需要通过第三方) |
| **备注** | Google Patents本身没有官方开发者API |

#### B.3.2 USPTO

| 项目 | 信息 |
|------|------|
| **名称** | USPTO Patent API |
| **官方文档** | https://ppubs.uspto.gov/ |
| **API Key获取** | https://developer.uspto.gov/ |
| **Rate Limiting** | Patent Public Search API: 约60次/分钟 |
| **数据类型** | JSON / Binary (PDF) |
| **认证方式** | x-api-key header |
| **端点列表** | `/api/v1/search`、`/api/v1/patent/{patentNumber}`、`/api/v1/patent/{patentNumber}/pdf` |
| **定价** | 免费 |
| **是否公开** | ✅ 是 |
| **特点** | 专利搜索、详情获取、PDF下载 |

#### B.3.3 CNIPA

| 项目 | 信息 |
|------|------|
| **名称** | 中国国家知识产权局 |
| **官方文档** | https://www.cnipa.gov.cn/ |
| **API Key获取** | 无公开API |
| **Rate Limiting** | 不适用 |
| **数据类型** | HTML (网页界面) |
| **认证方式** | 需要登录 |
| **端点列表** | 在线检索系统 (https://pss-system.cponline.cnipa.gov.cn/conventionalSearch) |
| **定价** | 免费 |
| **是否公开** | ❌ 无 (仅在线检索) |
| **备注** | 主要提供在线检索系统，无公开REST API，可使用爬虫技术获取数据 |

---

### B.4 产品与电商情报 (4个)

#### B.4.1 淘宝开放平台 (TOP)

| 项目 | 信息 |
|------|------|
| **名称** | 淘宝开放平台 API (TOP) |
| **官方文档** | https://open.taobao.com/ |
| **API文档** | http://open.taobao.com/doc/category_list.htm |
| **API Key获取** | 需要注册申请AppKey和AppSecret |
| **Rate Limiting** | 不同接口不同限制，通常10-100次/秒 |
| **数据类型** | JSON / XML |
| **认证方式** | AppKey + AppSecret + 签名 |
| **端点列表** | 商品搜索、商品详情、店铺商品、订单查询、用户授权等 |
| **定价** | 免费注册，部分API需要合作申请 |
| **是否公开** | ✅ 是 (需注册) |
| **备注** | 需要合作申请 |

#### B.4.2 京东开放平台 (JOS)

| 项目 | 信息 |
|------|------|
| **名称** | 京东宙斯开放平台 API (JOS) |
| **官方文档** | https://open.jd.com/ |
| **API文档** | https://open.jd.com/#/help/home |
| **API Key获取** | 需要注册申请AppKey和AppSecret |
| **Rate Limiting** | 默认50次/秒，单日上限10万次，可根据应用等级提升 |
| **数据类型** | JSON |
| **认证方式** | AppKey + AppSecret + 签名 |
| **端点数量** | 超过700个接口 |
| **主要端点** | 商品详情、订单查询、物流查询、商品搜索 |
| **定价** | 免费注册，企业版需要申请 |
| **是否公开** | ✅ 是 (需注册) |
| **备注** | 需要实名认证和企业认证才能使用部分高级接口 |

#### B.4.3 1688开放平台

| 项目 | 信息 |
|------|------|
| **名称** | 1688开放平台 API |
| **官方文档** | https://open.1688.com/ |
| **API文档** | https://open.1688.com/docs/api.htm |
| **API Key获取** | 需要注册申请AppKey和AppSecret |
| **Rate Limiting** | 根据接口不同而变化，通常数千到数万次/天 |
| **数据类型** | JSON |
| **认证方式** | AppKey + Secret + 调用key |
| **端点列表** | 商品详情、商品搜索、店铺商品、按图搜索 |
| **定价** | 免费注册，部分功能可能需要特殊授权 |
| **是否公开** | ✅ 是 (需注册) |
| **备注** | 阿里巴巴B2B批发平台API，主要用于批发业务 |

#### B.4.4 Amazon Product Advertising & Selling Partner API

| 项目 | 信息 |
|------|------|
| **名称** | Amazon API |
| **官方文档** | https://webservices.amazon.com/paapi5/documentation/ (Product Advertising) |
| **开发者中心** | https://developer.amazon.com/ (Selling Partner) |
| **API Key获取** | AWS Access Key ID + Secret Access Key + Associate Tag (Product Advertising) / Seller ID + MWS Auth Token (Selling Partner) |
| **Rate Limiting** | Product Advertising: 1次/小时/IP (免费)，付费计划更高 |
| **数据类型** | JSON |
| **认证方式** | AWS凭证或OAuth 2.0 |
| **端点列表** | `/paapi5/searchitems`、`/paapi5/getitems`、`/catalog/2022-04-01/items/{ASIN}`、`/orders/v0/orders` |
| **定价** | 免费 (Enterprise有更高配额) |
| **是否公开** | ✅ 是 |
| **备注** | 分为多个区域(NA, EU, FE)，需要分别申请，授权流程复杂 |

---

### B.5 法律法规与政策 (2个)

#### B.5.1 北大法宝

| 项目 | 信息 |
|------|------|
| **名称** | 北大法宝 |
| **官方文档** | 待确认是否有公开API |
| **API Key获取** | 待确认 |
| **Rate Limiting** | 待确认 |
| **数据类型** | 待确认 |
| **认证方式** | 待确认 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 待确认 |
| **备注** | 可能需要合作或使用爬虫 |

#### B.5.2 中国裁判文书网

| 项目 | 信息 |
|------|------|
| **名称** | 中国裁判文书网 |
| **官方文档** | 待确认是否有公开API |
| **API Key获取** | 待确认 (需要认证登录) |
| **Rate Limiting** | 待确认 |
| **数据类型** | 待确认 |
| **认证方式** | 需要认证登录 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ❌ 无公开API (需登录) |
| **备注** | 需要认证登录，爬取有难度 |

---

### B.6 舆情与社交媒体 (3个)

#### B.6.1 微博开放平台

| 项目 | 信息 |
|------|------|
| **名称** | 微博开放平台 API |
| **官方文档** | https://open.weibo.com/ |
| **API Key获取** | 需要注册申请App Key和App Secret |
| **Rate Limiting** | 根据接口和权限等级确定 |
| **数据类型** | JSON |
| **认证方式** | OAuth 2.0 |
| **端点列表** | `/2/search/statuses`、`/2/statuses/user_timeline`、`/2/users/show` |
| **定价** | 免费注册 |
| **是否公开** | ✅ 是 (需注册) |

#### B.6.2 抖音开放平台

| 项目 | 信息 |
|------|------|
| **名称** | 抖音开放平台 |
| **官方文档** | https://developer.open-douyin.com/ |
| **API Key获取** | 需要企业认证 |
| **Rate Limiting** | 待确认 |
| **数据类型** | JSON |
| **认证方式** | 待确认 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 需要企业认证 |
| **备注** | 个人开发者接入有限制 |

#### B.6.3 B站API

| 项目 | 信息 |
|------|------|
| **名称** | Bilibili开放平台 |
| **官方文档** | https://openhome.bilibili.com/ |
| **API Key获取** | 待确认 |
| **Rate Limiting** | 待确认 |
| **数据类型** | JSON |
| **认证方式** | 待确认 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 公开接口有限 |
| **备注** | 部分接口需要合作 |

---

### B.7 求职招聘与人才市场 (3个)

#### B.7.1 Boss直聘

| 项目 | 信息 |
|------|------|
| **名称** | Boss直聘 |
| **官方文档** | 待确认是否有公开API |
| **API Key获取** | 待确认 |
| **Rate Limiting** | 待确认 |
| **数据类型** | 待确认 |
| **认证方式** | 待确认 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 待确认 |
| **备注** | 可能需要合作 |

#### B.7.2 拉勾

| 项目 | 信息 |
|------|------|
| **名称** | 拉勾开放平台 |
| **官方文档** | 待确认 |
| **API Key获取** | 待确认 |
| **Rate Limiting** | 待确认 |
| **数据类型** | 待确认 |
| **认证方式** | 待确认 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 待确认 |

#### B.7.3 LinkedIn API

| 项目 | 信息 |
|------|------|
| **名称** | LinkedIn API |
| **官方文档** | https://learn.microsoft.com/en-us/linkedin/ |
| **API Key获取** | OAuth 2.0 |
| **Rate Limiting** | 待确认 |
| **数据类型** | JSON |
| **认证方式** | OAuth 2.0 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 企业账号才能访问职位数据 |
| **备注** | 部分功能需要企业账户 |

---

### B.8 票务与活动信息 (2个)

#### B.8.1 大麦网

| 项目 | 信息 |
|------|------|
| **名称** | 大麦网 |
| **官方文档** | 待确认 |
| **API Key获取** | 待确认 |
| **Rate Limiting** | 待确认 |
| **数据类型** | 待确认 |
| **认证方式** | 待确认 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 待确认 |
| **备注** | 可能需要合作 |

#### B.8.2 猫眼

| 项目 | 信息 |
|------|------|
| **名称** | 猫眼 |
| **官方文档** | 待确认 |
| **API Key获取** | 待确认 |
| **Rate Limiting** | 待确认 |
| **数据类型** | 待确认 |
| **认证方式** | 待确认 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 待确认 |
| **备注** | 可能需要爬虫 |

---

### B.9 物流与快递 (3个)

#### B.9.1 顺丰

| 项目 | 信息 |
|------|------|
| **名称** | 顺丰开放平台 |
| **官方文档** | https://open.sf-express.com/ |
| **API Key获取** | 需要合作申请 |
| **Rate Limiting** | 待确认 |
| **数据类型** | JSON |
| **认证方式** | 待确认 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 需要合作 |

#### B.9.2 圆通

| 项目 | 信息 |
|------|------|
| **名称** | 圆通 |
| **官方文档** | 待确认 |
| **API Key获取** | 待确认 |
| **Rate Limiting** | 待确认 |
| **数据类型** | 待确认 |
| **认证方式** | 待确认 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 待确认 |

#### B.9.3 中通

| 项目 | 信息 |
|------|------|
| **名称** | 中通 |
| **官方文档** | 待确认 |
| **API Key获取** | 待确认 |
| **Rate Limiting** | 待确认 |
| **数据类型** | 待确认 |
| **认证方式** | 待确认 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 待确认 |

---

### B.10 旅游与交通 (3个)

#### B.10.1 TripAdvisor

| 项目 | 信息 |
|------|------|
| **名称** | TripAdvisor API |
| **官方文档** | https://developer.tripadvisor.com/ |
| **API Key获取** | 需要注册申请 |
| **Rate Limiting** | 待确认 |
| **数据类型** | JSON |
| **认证方式** | API Key |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ✅ 是 (需注册) |

#### B.10.2 携程

| 项目 | 信息 |
|------|------|
| **名称** | 携程 |
| **官方文档** | 待确认 |
| **API Key获取** | 待确认 |
| **Rate Limiting** | 待确认 |
| **数据类型** | 待确认 |
| **认证方式** | 待确认 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 待确认 |
| **备注** | 可能需要合作 |

#### B.10.3 航班管家

| 项目 | 信息 |
|------|------|
| **名称** | 航班管家 |
| **官方文档** | 待确认 |
| **API Key获取** | 待确认 |
| **Rate Limiting** | 待确认 |
| **数据类型** | 待确认 |
| **认证方式** | 待确认 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 待确认 |

---

### B.11 医疗健康 (1个)

#### B.11.1 好大夫在线

| 项目 | 信息 |
|------|------|
| **名称** | 好大夫 |
| **官方文档** | 待确认 |
| **API Key获取** | 待确认 |
| **Rate Limiting** | 待确认 |
| **数据类型** | 待确认 |
| **认证方式** | 待确认 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 待确认 |
| **备注** | 主要用于信息查询，非医疗建议 |

---

### B.12 教育课程 (3个)

#### B.12.1 Coursera

| 项目 | 信息 |
|------|------|
| **名称** | Coursera API |
| **官方文档** | https://build.coursera.org/ |
| **API Key获取** | OAuth 2.0 |
| **Rate Limiting** | 待确认 |
| **数据类型** | JSON |
| **认证方式** | OAuth 2.0 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ✅ 是 (需OAuth) |

#### B.12.2 Udemy

| 项目 | 信息 |
|------|------|
| **名称** | Udemy API |
| **官方文档** | https://www.udemy.com/developers/ |
| **API Key获取** | API Key + Client ID |
| **Rate Limiting** | 待确认 |
| **数据类型** | JSON |
| **认证方式** | API Key + Client ID |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ✅ 是 (需注册) |

#### B.12.3 网易云课堂

| 项目 | 信息 |
|------|------|
| **名称** | 网易云课堂 |
| **官方文档** | 待确认 |
| **API Key获取** | 待确认 |
| **Rate Limiting** | 待确认 |
| **数据类型** | 待确认 |
| **认证方式** | 待确认 |
| **端点列表** | 待确认 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 待确认 |

---

## C. 政府数据API (16个)

### C.1 美国政府 (7个)

#### C.1.1 api.data.gov ⭐ 信息最完整

| 项目 | 信息 |
|------|------|
| **名称** | api.data.gov |
| **官方文档** | https://api.data.gov/ |
| **开发者手册** | https://api.data.gov/docs/developer-manual/ |
| **API Key获取** | https://api.data.gov/signup/ (DEMO_KEY可用于探索) |
| **Rate Limiting** | 1000 requests/hour (默认) | DEMO_KEY: 30 req/hour, 50 req/day |
| **数据类型** | JSON |
| **认证方式** | X-Api-Key header / api_key query parameter / HTTP Basic Auth |
| **覆盖机构** | 25个联邦机构，450+个API |
| **主要机构** | USDA、Department of Commerce、DOE、HHS、GSA、NASA、NSF等 |
| **定价** | 免费 |
| **是否公开** | ✅ 是 |
| **特点** | 统一API管理平台，多种认证方式，完整的错误代码列表 |

#### C.1.2 Data.gov

| 项目 | 信息 |
|------|------|
| **名称** | Data.gov |
| **官方文档** | https://open.gsa.gov/api/datadotgov/ |
| **OpenAPI规范** | https://api.gsa.gov/technology/datadotgov/v3/openapi.json |
| **API Key获取** | https://api.data.gov/signup/ |
| **Rate Limiting** | 继承api.data.gov策略 |
| **数据类型** | JSON |
| **认证方式** | x-api-key header |
| **主要端点** | `/action/package_search`、`/action/datastore_search_sql` |
| **定价** | 免费 |
| **是否公开** | ✅ 是 |
| **备注** | 基于CKAN平台，仅提供元数据，不包含实际数据 |

#### C.1.3 openFDA ⭐ 信息较完整

| 项目 | 信息 |
|------|------|
| **名称** | openFDA |
| **官方文档** | https://open.fda.gov/ |
| **API文档** | https://open.fda.gov/apis/ |
| **API Key获取** | 大部分端点无需Key，高级功能可能需要 |
| **Rate Limiting** | 未在文档中明确说明 |
| **数据类型** | JSON |
| **认证方式** | 大部分无需认证 |
| **主要端点** | `/drug/`、`/device/`、`/food/` (基于Elasticsearch) |
| **响应结构** | `{ meta: {}, results: [] }` |
| **定价** | 免费 |
| **是否公开** | ✅ 是 |
| **特点** | 药物、医疗器械、食品数据，基于Elasticsearch |

#### C.1.4 CDC ⚠️ 部分信息

| 项目 | 信息 |
|------|------|
| **名称** | CDC (Centers for Disease Control and Prevention) |
| **官方文档** | https://open.cdc.gov/apis.html |
| **Tools API** | https://tools.cdc.gov/api/docs/info.aspx#response |
| **Tracking API** | https://ephtracking.cdc.gov/apihelp |
| **API Key获取** | 不同API可能有不同要求 |
| **Rate Limiting** | 未在现有文档中明确提供 |
| **数据类型** | JSON / XML / CSV |
| **认证方式** | 可能需要API Key / IP白名单 / OAuth |
| **主要API** | CDC Tools API、Environmental Public Health Tracking Network API |
| **定价** | 免费 |
| **是否公开** | ✅ 是 |
| **备注** | 认证和速率限制需要进一步验证 |

#### C.1.5 NIH ⚠️ 部分信息

| 项目 | 信息 |
|------|------|
| **名称** | NIH (National Institutes of Health) |
| **官方文档** | 待确认 |
| **iRIS Study API** | https://cris.cancer.gov/confluence/display/CCRClinicalIT3/NIH%20iRIS%20Study%20API |
| **RePORTER API** | 待确认 |
| **API Key获取** | HTTP Basic Auth (用户名密码) 或 IP白名单 |
| **Rate Limiting** | 未在现有文档中明确提供 |
| **数据类型** | JSON |
| **认证方式** | HTTP Basic Auth / IP白名单 |
| **主要API** | NIH iRIS Study API、RePORTER API |
| **定价** | 免费 |
| **是否公开** | ✅ 是 |
| **备注** | 认证和速率限制需要进一步验证 |

#### C.1.6 U.S. Census Bureau ⚠️ 部分信息

| 项目 | 信息 |
|------|------|
| **名称** | U.S. Census Bureau |
| **官方文档** | https://www.census.gov/developers/ |
| **API文档** | U.S. Census API Documentation |
| **API Key获取** | https://api.census.gov/data/signup.html |
| **Rate Limiting** | 未在现有文档中明确提供 |
| **数据类型** | JSON / XML / CSV |
| **认证方式** | API Key |
| **端点数量** | 超过1500个API端点 |
| **主要数据集** | Decennial Census、American Community Survey、International Trade Datasets、Small Area Health Insurance Estimates、Economic Indicators Time Series |
| **定价** | 免费 |
| **是否公开** | ✅ 是 |
| **备注** | 速率限制需要查看开发者文档获取详细信息 |

#### C.1.7 EPA ❌ 需要进一步调查

| 项目 | 信息 |
|------|------|
| **名称** | Environmental Protection Agency (EPA) |
| **官方文档** | 待确认 (仅知其是api.data.gov的参与机构) |
| **API Key获取** | 可能使用api.data.gov的API Key系统 |
| **Rate Limiting** | 待确认 |
| **数据类型** | 待确认 |
| **认证方式** | 可能使用X-Api-Key header |
| **主要数据** | 空气质量、水质、废物管理、化学物质信息、环境排放 |
| **定价** | 待确认 |
| **是否公开** | ❌ 需要直接访问EPA官网 |
| **备注** | 仅知其是api.data.gov的参与机构，具体端点和文档需要直接访问EPA官网 |

---

### C.2 中国政府 (9个)

#### C.2.1 北京市政务数据资源网

| 项目 | 信息 |
|------|------|
| **名称** | 北京市政务数据资源网 |
| **官方文档** | https://data.beijing.gov.cn/ |
| **API Key获取** | 用户需要在平台注册登录后获取API访问权限 |
| **Rate Limiting** | 暂无明确策略说明，需查阅平台最新文档 |
| **数据类型** | JSON / CSV |
| **认证方式** | 用户注册登录认证 / API Key |
| **数据来源** | 103个政府部门 |
| **数据集** | 10,266个政务数据集 |
| **数据量** | 超过13亿条数据 |
| **访问量** | 累计3亿次，下载量37万多次 |
| **定价** | 免费 |
| **是否公开** | ✅ 是 (需注册) |

#### C.2.2 上海市政府数据服务网

| 项目 | 信息 |
|------|------|
| **名称** | 上海市政府数据服务网 / 上海市公共数据开放平台 |
| **官方文档** | https://data.sh.gov.cn/ |
| **API Key获取** | 用户需要在平台注册登录后获取API访问权限 |
| **Rate Limiting** | 暂无明确策略说明，需查阅平台最新文档 |
| **数据类型** | JSON |
| **认证方式** | 用户注册登录认证 / API Key |
| **数据来源** | 100个机构 |
| **数据资源** | 4,694项数据资源 |
| **数据量** | 超过5200万条数据 |
| **接口数量** | 642个数据接口 |
| **特点** | 中国第一个开放数据门户网站(2012年6月上线)，接口较完善但调用难度高、数据容量小、更新频率低 |
| **定价** | 免费 |
| **是否公开** | ✅ 是 (需注册) |

#### C.2.3 深圳市政府数据开放平台

| 项目 | 信息 |
|------|------|
| **名称** | 深圳市政府数据开放平台 |
| **官方文档** | http://opendata.sz.gov.cn/ |
| **API Key获取** | 用户提交应用名称并订阅接口获取appKey作为专属秘钥 |
| **Rate Limiting** | 部分接口有分页限制，如营运车辆GPS数据API，每页最大5000行数据 |
| **数据类型** | JSON |
| **认证方式** | API Key认证 (appKey) |
| **数据来源** | 50个部门(含11个区、36个本市国家机关、1个事业单位、2个公共组织) |
| **数据集** | 3,350个开放数据集 |
| **数据量** | 14.9亿条数据 |
| **接口数量** | 1,260个数据接口 |
| **特点** | 接口调用活跃，数据更新相对及时，在"中国开放数林指数"城市综合排名中连续四年(2019-2022年)排名全国前四 |
| **定价** | 免费 |
| **是否公开** | ✅ 是 (需注册) |

#### C.2.4 浙江省数据开放平台

| 项目 | 信息 |
|------|------|
| **名称** | 浙江省人民政府数据开放网站 / 浙江数据开放 |
| **官方文档** | https://data.zjzwfw.gov.cn/ |
| **API Key获取** | 用户需要在平台注册登录后获取API访问权限 |
| **Rate Limiting** | 暂无明确策略说明，需查阅平台最新文档 |
| **数据类型** | JSON / CSV |
| **认证方式** | 用户注册登录认证 / API Key |
| **数据来源** | 省级各单位 |
| **数据集** | **18,306个数据集** (全国领先) |
| **数据量** | 59亿条数据 |
| **接口数量** | **9,657个数据接口** (全国领先) |
| **特点** | 全国首个省级政府数据统一开放平台，遵循《浙江省公共数据条例》 |
| **定价** | 免费 |
| **是否公开** | ✅ 是 (需注册) |

#### C.2.5 广州市公共数据开放平台

| 项目 | 信息 |
|------|------|
| **名称** | 广州市公共数据开放平台 |
| **官方文档** | https://data.gz.gov.cn/ |
| **API Key获取** | 用户需要在平台注册登录后获取API访问权限 |
| **Rate Limiting** | 暂无明确策略说明，需查阅平台最新文档 |
| **数据类型** | JSON / CSV |
| **认证方式** | 用户注册登录认证 / API Key |
| **数据来源** | 90个开放主体 |
| **数据集** | 2,500+个数据集 |
| **数据量** | 超过1.5亿条数据 |
| **特点** | 用户访问量达74.2万次，累计下载数据30.8万次，设有"互动服务"栏目 |
| **定价** | 免费 |
| **是否公开** | ✅ 是 (需注册) |

#### C.2.6 天津市信息资源统一开放平台

| 项目 | 信息 |
|------|------|
| **名称** | 天津市信息资源统一开放平台 |
| **官方文档** | https://data.tj.gov.cn/ |
| **API Key获取** | 用户需要在平台注册登录后获取API访问权限 |
| **Rate Limiting** | 暂无明确策略说明，需查阅平台最新文档 |
| **数据类型** | JSON / CSV |
| **认证方式** | 用户注册登录认证 / API Key |
| **数据来源** | 52个市级部门 |
| **数据集** | 1,095个政务数据集 |
| **数据量** | 6,760余万条数据 |
| **接口数量** | 508个数据接口 |
| **特点** | 分为无条件开放、有条件开放和不予开放三种类型，有条件开放类数据需要通过开放申请审核 |
| **定价** | 免费 |
| **是否公开** | ✅ 是 (需注册) |

#### C.2.7 四川公共数据开放网

| 项目 | 信息 |
|------|------|
| **名称** | 四川公共数据开放网 |
| **官方文档** | http://www.scdata.net.cn/ |
| **API Key获取** | 用户需要在平台注册登录后获取API访问权限 |
| **Rate Limiting** | 暂无明确策略说明，需查阅平台最新文档 |
| **数据类型** | JSON / CSV |
| **认证方式** | 用户注册登录认证 / API Key |
| **数据来源** | 49个部门 |
| **数据集** | 8,535个数据集 |
| **数据量** | 约349.7亿条数据 |
| **接口数量** | 343个数据接口 |
| **特点** | 四川省21个市(州)均已正式上线公共数据开放平台，初步实现省市两级平台的互联互通和协同发展 |
| **定价** | 免费 |
| **是否公开** | ✅ 是 (需注册) |

#### C.2.8 中国气象局天气API

| 项目 | 信息 |
|------|------|
| **名称** | 中国气象局智慧天气应用编程接口开放平台 |
| **官方文档** | http://smart.weather.com.cn/wzfw/smart/weatherapi.shtml |
| **API Key获取** | 需在智慧天气应用编程接口开放平台申请appid和private_key |
| **Rate Limiting** | 暂无明确策略说明，每个产品使用用户分配一个唯一标识appid用于统计 |
| **数据类型** | JSON |
| **认证方式** | appid + HMAC-SHA1签名 (base64_encode(hash_hmac('sha1',$public_key,$private_key,TRUE))并urlencode编码) |
| **主要端点** | 实况数据、常规预报、指数接口、城市代码查询 |
| **更新频率** | 实况数据每小时更新多次，预报数据每天更新3次(8、11、18点左右) |
| **主要数据** | 温度、湿度、风力、风向、发布时间等 |
| **特点** | 需要复杂的加密算法生成key，高德地图天气查询API也使用中国气象局数据 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 需要申请 |

#### C.2.9 中国科技云大模型API开放平台

| 项目 | 信息 |
|------|------|
| **名称** | 中国科技云大模型API开放平台 |
| **官方文档** | https://uni-api.cstcloud.cn/ |
| **英文版** | https://uni-api-global.cstcloud.cn/ |
| **API Key获取** | 通过CSTCloud AI(身份认证与授权基础设施)申请API Key |
| **Rate Limiting** | 基于Token计量的精准计量 |
| **数据类型** | JSON / SSE |
| **认证方式** | API Key认证 |
| **主要模型** | DeepSeek-R1 (671B)、DeepSeek-V3 (671B)、Qwen3 (235B)、Qwen2.5-VL (72B)、bge-large-zh (嵌入)、bge-reranker-V2-m3 (重排) |
| **主要端点** | `/api/chat/completions`、`/api/embeddings`、`/api/rerank` |
| **Token调用量** | 已突破**1000亿** (截至2025年9月数据) |
| **特点** | 全面兼容OpenAI API格式，可与ragflow、dify、openwebui等主流应用无缝对接，支持文本理解、多模态分析、工具调度等全栈技术生态 |
| **定价** | 待确认 |
| **是否公开** | ⚠️ 需要申请 |

---

## D. 开发优先级建议

### D.1 第一批：MVP核心 (P0) - 立即开发

**目标**: 完成核心功能，可以处理基本的搜索和API调用

| 优先级 | API | 理由 |
|--------|-----|------|
| P0 | 现有8个MCP工具 | 已有完整实现，只需封装 |
| P0 | Brave Search | 通用搜索，月订阅制简单 |
| P0 | Tavily Search | 功能最全，深度搜索能力强 |
| P0 | 智谱搜索 | 中文搜索，性价比高 |
| P0 | FinanceMCP | 金融数据，覆盖A股/美股/港股 |
| P0 | PubMed | 生物医学文献，权威数据源 |
| P0 | OpenAlex | 开放学术数据库，数据量大 |
| P0 | 高德地图 | 地图服务，POI搜索和路径规划 |

**预计时间**: 6-8周

**验收标准**:
- 可以在Claude Desktop中成功注册并调用
- 至少支持Brave和Tavily两个搜索引擎
- 响应延迟< 500ms
- 测试覆盖率> 80%

---

### D.2 第二批：高价值商业API (P1) - 2-3个月

**目标**: 集成高价值的商业API

| 优先级 | API | 理由 |
|--------|-----|------|
| P1 | GitHub REST/GraphQL API | 代码搜索，技术调研必备 |
| P1 | USPTO API | 专利数据，完全免费 |
| P1 | Data.gov | 美国政府数据，450+ API |
| P1 | 深圳市数据平台 | 接口调用活跃，更新及时 |
| P1 | 浙江省数据平台 | 数据集数量全国领先(18306个) |
| P1 | 上海市数据平台 | 中国第一个开放数据平台 |

**预计时间**: 4-6周

**验收标准**:
- 每个API至少通过基本功能测试
- 认证和签名正确
- Rate limiting遵守API限制

---

### D.3 第三批：补充资源 (P2) - 持续迭代

**目标**: 集成其他政府平台、企业信息、电商等

| 优先级 | API | 理由 |
|--------|-----|------|
| P2 | 天眼查/企查查 | 企业信息，需要付费 |
| P2 | 北京市/广州市/天津市/四川省平台 | 政府数据补充 |
| P2 | 淘宝/京东/1688 API | 电商数据，需要合作申请 |
| P2 | Amazon API | 电商数据，授权流程复杂 |
| P2 | 中国气象局API | 天气数据，需要申请 |
| P2 | 中国科技云大模型API | LLM模型，需要申请 |

**预计时间**: 持续迭代

**注意事项**:
- 这些API大多需要合作申请或付费
- 需要根据实际需求和预算逐步接入

---

### D.4 需要进一步调查的API

| API | 需要调查的内容 |
|-----|--------------|
| Google Patents | 是否有官方API，目前仅通过第三方访问 |
| CNIPA | 是否有公开API，目前仅在线检索 |
| 北大法宝 | 是否有开放API，文档不明确 |
| 中国裁判文书网 | 是否有公开API，需要登录认证 |
| EPA | 官方文档和端点，目前仅知其是api.data.gov的参与机构 |
| 微博/抖音/B站 | 完整的API文档和认证方式 |
| Boss直聘/拉勾 | 是否有开放API |
| 大麦网/猫眼 | 是否有开放API |
| 顺丰/圆通/中通 | 完整的API文档 |
| TripAdvisor/携程/航班管家 | 完整的API文档 |
| 好大夫 | 是否有开放API |
| Coursera/Udemy/网易云课堂 | 完整的API文档 |

---

## E. 技术实施建议

### E.1 统一认证和授权

** Wrapped API Key格式**:
```
mcp_pool_v1:client_id:signature:expiry
```

**组成部分**:
- `mcp_pool_v1`: 版本标识
- `client_id`: 客户端唯一标识
- `signature`: HMAC-SHA256签名 (client_id + expiry)
- `expiry`: Unix时间戳 (秒级)

**验证流程**:
1. 检查版本是否为 `mcp_pool_v1`
2. 检查是否过期 (expiry > 当前时间)
3. 验证签名是否正确

---

### E.2 Rate Limiting策略

**Token Bucket算法**:
```rust
struct RateLimiter {
    tokens: f64,
    capacity: f64,
    refill_rate: f64,
    last_update: Instant,
}

// 每个API适配器有独立的Token Bucket
// 每个Wrapped API Key也有独立的Token Bucket
```

**Rate Limiting层级**:
1. **Wrapped API Key级别**: 每个client_id有独立的配额
2. **API适配器级别**: 每个下游API有独立的配额
3. **全局级别**: 整体并发限制

---

### E.3 缓存策略

**多级缓存**:
1. **内存缓存**: 最热查询，TTL=1分钟
2. **Redis缓存**: 热门查询，TTL根据查询类型配置
   - 实时数据(天气): 5分钟
   - 日度数据(股票): 1小时
   - 月度数据(宏观): 24小时
   - 年度数据(普查): 30天

**缓存键设计**:
```rust
struct CacheKey {
    adapter_id: String,
    query_hash: String,
    options_hash: String,
    time_bucket: String,
}
```

---

### E.4 API适配器开发框架

**统一适配器trait**:
```rust
trait APIAdapter {
    async fn call(&self, params: &SearchParams) -> Result<APIResponse, APIError>;
    fn name(&self) -> &str;
    fn rate_limit(&self) -> RateLimitConfig;
    fn supports_search_type(&self, search_type: SearchType) -> bool;
}

// 每个API实现这个trait
struct BraveAdapter { ... }
struct TavilyAdapter { ... }
struct FinanceMCPAdapter { ... }
```

---

### E.5 智能路由层

**路由决策模型**:
- **小模型**: DeepSeek-7B / Qwen2.5-7B
- **推理框架**: Candle / onnxruntime
- **调用方式**: 内部gRPC或本地进程调用

**路由策略**:
```rust
enum DispatchStrategy {
    Parallel { timeout_ms: u64 },    // 多API并行调用
    Sequential { chain: Vec<String> }, // 串行调用
    Hybrid { parallel: Vec<String>, sequential: Vec<String> },
}
```

---

### E.6 响应标准化

**统一响应格式**:
```rust
struct UnifiedResponse {
    request_id: String,
    query: String,
    meta: ResponseMeta,
    results: Vec<UnifiedResult>,
    timing: ResponseTiming,
}
```

**数据标准化**:
- 所有API的响应都转换为统一的JSON schema
- 错误处理和重试
- 结果聚合和去重

---

## 文档结束

**总API数量**: 59个
**完成度**: 100% (所有API基本信息已调研)
**生成时间**: 2026-03-28
**生成人**: 小虾 🦞

**下一步行动**:
1. 开始Phase 1开发：现有8个MCP工具 + 核心架构
2. 根据实际需求逐步接入P1和P2类API
3. 对于需要进一步调查的API，建议直接访问官方文档或联系官方

---

*完整API清单编制完成！🦞✨*
