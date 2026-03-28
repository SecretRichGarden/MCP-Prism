# 🦞 智能体决策系统 - 外部情报源完整清单

**生成时间**: 2026-03-27  
**整理人**: 小虾 🦞  
**用途**: 智能决策系统情报源规划与接入

---

## 📋 目录

- [第一部分：现有外部信息获取工具](#第一部分现有外部信息获取工具)
- [第二部分：建议补充的 MCP/API 清单](#第二部分建议补充的-mcpapi-清单)
- [第三部分：政府数据 API 接口情报](#第三部分政府数据-api-接口情报)
- [第四部分：综合优先级建议](#第四部分综合优先级建议)

---

# 第一部分：现有外部信息获取工具

## 🔍 搜索类（4个搜索引擎）

### 1. Brave Search (brave-search-dev)

**工具列表**:
- `brave_web_search` - 通用网页搜索
- `brave_local_search` - 本地商家搜索
- `brave_video_search` - 视频搜索
- `brave_image_search` - 图片搜索
- `brave_news_search` - 新闻搜索
- `brave_summarizer` - 搜索结果摘要（需 Pro 订阅）

**特色能力**:
- 支持 40+ 国家/地区搜索
- 支持 30+ 种语言
- 支持时间范围过滤（24小时/7天/31天/365天）
- 支持结果类型过滤（网页/新闻/视频/图片/本地/FAQ）
- Goggles 自定义重排序

---

### 2. Tavily Search (tavily-mcp-local)

**工具列表**:
- `tavily_search` - 深度搜索，支持多维度过滤
- `tavily_extract` - URL 内容提取（Markdown/文本）
- `tavily_crawl` - 网站爬虫，支持深度和广度配置
- `tavily_map` - 网站结构映射
- `tavily_research` - 综合研究（mini/pro/auto 模型）

**特色能力**:
- 搜索深度：basic/advanced/fast/ultra-fast
- 时间范围：day/week/month/year + 自定义起止日期
- 域名过滤：include_domains / exclude_domains
- 支持原始内容提取、图片、favicon
- 爬虫可配置最大深度、宽度、限制

---

### 3. 智谱搜索 (zhipu-web-search-sse)

**工具列表**:
- `webSearchPro` - 专业搜索（1-50 条结果，2500字摘要）
- `webSearchStd` - 标准搜索
- `webSearchSogou` - 搜狗引擎
- `webSearchQuark` - 夸克引擎

**特色能力**:
- 支持 1-50 条结果返回
- 支持域名白名单过滤
- 支持时间范围过滤（oneDay/oneWeek/oneMonth/oneYear/noLimit）
- 摘要字数可选：medium（400-600字）/ high（2500字）

---

### 4. 秘塔搜索 (metaso-search-mcp)

**工具列表**:
- `metaso_search` - 多维度搜索
- `metaso_reader` - 网页内容提取

**搜索范围**:
- webpage - 网页搜索（默认）
- document - 文库搜索
- paper - 学术论文搜索
- image - 图片搜索
- video - 视频搜索
- podcast - 播客/博客搜索

**特色能力**:
- includeSummary：通过网页摘要增强召回
- includeRawContent：抓取来源网页原文
- conciseSnippet：返回精简的原文匹配信息
- 支持 JSON 和 Markdown 输出

---

## 💰 金融数据类（1个超强）

### FinanceMCP (finance-mcp-local) 📈

**行情数据**:
- **市场覆盖**: A股/美股/港股/外汇/期货/基金/债券逆回购/可转债/期权/加密货币
- **分钟线**: 1MIN/5MIN/15MIN/30MIN/60MIN K线
- **指数数据**: CSI 指数成分权重与估值（PE TTM、PB、股息率、ROE等）

**财务数据**:
- `company_performance` - 上市公司综合表现
  - 业绩预告、业绩快报
  - 财务指标、分红送股
  - 主营业务构成
  - 股东变动、管理层信息
  - 资产负债表、现金流量表、利润表
  - 股权质押、回购

**宏观指标**:
- `macro_econ` - 宏观经济数据
  - Shibor/LPR/GDP/CPI/PPI/PMI/社融
  - 货币供应量、Libor、Hibor

**市场数据**:
- `money_flow` - 个股/大盘/板块资金流向
- `margin_trade` - 融资融券数据
- `block_trade` - 大宗交易数据
- `dragon_tiger_inst` - 龙虎榜机构成交明细
- `hot_news_7x24` - 7x24热点新闻

**基金数据**:
- `fund_data` - 公募基金全面数据
  - 基金列表、基金经理
  - 基金净值、分红、持仓

---

## 📚 学术文献类（2个权威）

### 1. PubMed (mcp-pubmed-llm-server)

**工具列表**:
- `pubmed_search` - 文献搜索（支持布尔逻辑和 MeSH）
- `pubmed_quick_search` - 快速搜索
- `pubmed_get_details` - 获取文献详情
- `pubmed_batch_query` - 批量查询（最多20篇）
- `pubmed_cross_reference` - 交叉引用相似/引用/文献
- `pubmed_extract_key_info` - 提取文献关键信息
- `pubmed_detect_fulltext` - 检测开放获取状态
- `pubmed_download_fulltext` - 下载全文 PDF
- `pubmed_batch_download` - 批量下载（最多10篇）
- `pubmed_get_fulltext_sections` - 提取论文章节

**特色能力**:
- 支持时间范围过滤（days_back）
- 支持排序：relevance/date/pubdate
- 响应格式：compact/standard/detailed
- 摘要长度：500-6000字符
- 缓存管理和 EndNote 导出

---

### 2. OpenAlex (openalex-mcp-server)

**工具列表**:
- `openalex_search` - 学术论文搜索
- `openalex_get_work` - 获取论文详情
- `openalex_batch_get_works` - 批量获取（最多50篇）
- `openalex_detect_fulltext` - 检测开放获取状态
- `openalex_download_fulltext` - 下载开放获取全文
- `openalex_get_fulltext_sections` - 提取论文章节
- `openalex_cache_stats` - 缓存统计

**特色能力**:
- 支持按 publication_year / is_oa / type 过滤
- 支持 cited_by_count:desc / publication_year:desc 排序
- 摘要模式：quick（缓存）/ deep（重新抓取）
- 章节：abstract/introduction/methods/results/discussion/conclusion/references

---

## 🗺️ 地图与地理类（1个）

### 高德地图 (amap-maps)

**工具列表**:
- `maps_geo` - 地址转经纬度
- `maps_regeocode` - 经纬度转地址
- `maps_text_search` - POI 关键词搜索
- `maps_around_search` - 周边搜索
- `maps_search_detail` - POI 详细信息查询
- `maps_direction_driving` - 驾车路径规划
- `maps_direction_walking` - 步行路径规划
- `maps_bicycling` - 骑行路径规划
- `maps_direction_transit_integrated` - 公交路径规划
- `maps_distance` - 距离测量（驾车/步行/球面）
- `maps_weather` - 城市天气查询
- `maps_ip_location` - IP 定位

**特色能力**:
- 支持地标性名胜景区、建筑物名称解析
- 骑行规划支持 500km 内
- 公交规划支持跨城
- 周边搜索可配置半径

---

## 🔮 命理民俗类（1个）

### 八字 MCP (Bazi)

**工具列表**:
- `getBaziDetail` - 获取八字详情（公历/农历）
- `getSolarTimes` - 八字转公历时间
- `getChineseCalendar` - 黄历信息

**特色能力**:
- 支持 ISO 时间格式和农历时间
- 支持性别区分（0女性/1男性）
- 支持早晚子时配置

---

# 第二部分：建议补充的 MCP/API 清单

## 🔥 第一批（核心决策能力）

### 1. 企业信息查询

**功能**: 企业工商信息、股权结构、经营风险、诉讼记录

**推荐 API**:
- 天眼查 API
- 企查查 API

**价值**: 金融尽调、商业合作前调研、风险控制

**应用场景**:
- 分析上市公司上下游产业链
- 调研潜在合作伙伴和供应商
- 风险评估和预警

---

### 2. 代码仓库与开源项目

**功能**: GitHub/GitLab API，搜索项目、代码片段、Issue、PR

**推荐 API**:
- GitHub REST API
- GitHub GraphQL

**价值**: 技术调研、代码审计、竞品分析

**应用场景**:
- 查找特定功能的实现方案
- 分析技术趋势和栈选择
- 开源项目选型和评估

---

### 3. 专利与知识产权

**功能**: 专利搜索、法律状态、专利族分析

**推荐 API**:
- Google Patents API
- USPTO API
- CNIPA API

**价值**: 技术情报、竞争分析、专利风险评估

**应用场景**:
- 技术创新调研
- 知识产权尽职调查
- 专利侵权风险评估

---

### 4. 产品与电商情报

**功能**: 产品信息、价格监控、销量数据、评论分析

**推荐 API**:
- 淘宝 API（需合作）
- 京东 API（需合作）
- 1688 API
- 亚马逊产品广告 API

**价值**: 市场调研、竞品分析、供应链洞察

**应用场景**:
- 行业趋势分析
- 价格策略制定
- 消费者需求洞察

---

## 🌟 第二批（能力增强）

### 5. 法律法规与政策

**功能**: 法律法规库、政策文件检索、裁判文书

**推荐 API**:
- 北大法宝 API
- 中国裁判文书网

**价值**: 合规审查、政策跟踪、法律咨询辅助

**应用场景**:
- 行业合规分析
- 新政策解读
- 法律风险预警

---

### 6. 舆情与社交媒体

**功能**: 微博/抖音/B站/小红书内容检索、热点分析

**推荐 API**:
- 微博开放平台 API
- 抖音开放平台 API（需企业认证）
- B站 API

**价值**: 品牌监控、舆情预警、用户洞察

**应用场景**:
- 品牌声誉监控
- 产品反馈收集
- 市场热度分析

---

### 7. 求职招聘与人才市场

**功能**: 职位数据、薪资范围、技能需求、行业趋势

**推荐 API**:
- Boss 直聘 API
- 拉勾 API
- LinkedIn API

**价值**: 职业规划、人才市场分析、薪资对标

**应用场景**:
- 行业人才需求分析
- 技能趋势追踪
- 职业发展建议

---

### 8. 票务与活动信息

**功能**: 演出票务、电影排片、展览活动

**推荐 API**:
- 大麦网 API
- 猫眼 API

**价值**: 文化娱乐推荐、市场热点分析

**应用场景**:
- 活动策划参考
- 文化趋势分析
- 娱乐市场洞察

---

## 💡 第三批（锦上添花）

### 9. 物流与快递

**功能**: 物流查询、时效预估、费用计算

**推荐 API**:
- 顺丰 API
- 圆通 API
- 中通 API

**应用场景**: 电商分析、供应链优化

---

### 10. 旅游与交通

**功能**: 航班信息、酒店价格、景点评价

**推荐 API**:
- TripAdvisor API
- 携程 API（需合作）
- 航班管家 API

**应用场景**: 差旅规划、旅游市场分析

---

### 11. 医疗健康

**功能**: 医院查询、医生信息、药品信息

**推荐 API**:
- 好大夫在线 API
- 药品查询 API

**应用场景**: 医疗资源查询（非医疗建议）

---

### 12. 教育课程

**功能**: 在线课程、MOOC 平台数据

**推荐 API**:
- Coursera API
- Udemy API
- 网易云课堂 API

**应用场景**: 学习资源推荐

---

# 第三部分：政府数据 API 接口情报

## 🇺🇸 美国政府 API 接口生态

### 核心平台：Data.gov

**基本信息**
- **官网**: https://data.gov
- **成立时间**: 2009年5月21日
- **运营机构**: 美国总务管理局（GSA）科技转型服务部门
- **数据规模**: 近30万个数据集，来自100+组织
- **月访问量**: 超过100万次页面浏览

**核心功能**
- ✅ **数据发布与共享**: 政府机构可将数据以标准化格式发布
- ✅ **API接口**: 提供标准化的API调用接口
- ✅ **免费下载**: 所有的数据集均可免费下载和使用
- ✅ **实时更新**: 70%以上数据集在一年内更新

**数据分类**
- 📊 **经济数据**: 财政、贸易、就业、物价
- 🌱 **环境数据**: 气候、能源、污染监测
- 🏥 **医疗数据**: 医院质量、疾病统计、医疗保险
- 🚗 **交通数据**: 道路安全、公共交通、航空数据
- 🏫 **教育数据**: 学校信息、学生成绩、教育统计
- 🌾 **农业数据**: 作物产量、农业补贴、食品安全

---

### API 管理服务：api.data.gov

**官网**: https://api.data.gov/about/

**核心能力**
- 🔄 **透明层**: 在现有API之上添加额外功能
- 🛡️ **自动化管理**: 处理API管理的重复性工作
- 🎛️ **完全控制**: 机构仍然拥有API的完整控制权
- ⚡ **零改动**: 不需要修改现有API即可接入

---

### 联邦机构专用 API

#### 环境保护署（EPA）
- **领域**: 环境数据、空气质量、水资源、化学物质
- **数据类型**:
  - 空气质量监测数据
  - 水质检测数据
  - 化学品毒性评估
  - PFAS 健康风险数据

#### 食品药品监督管理局（FDA）
- **领域**: 药品审批、食品安全、医疗器械
- **API 应用场景**:
  - 药品许可信息查询系统
  - 食品安全预警
  - 医疗器械注册信息

#### 国家卫生研究院（NIH）
- **领域**: 医学研究、公共卫生、临床试验
- **特色项目**:
  - NIH BRAIN Initiative 细胞普查
  - 与EPA、FDA合作的自动化化学物质筛选系统
  - 高通量化学毒性测试

#### 疾病控制与预防中心（CDC）
- **领域**: 公共卫生、疾病监测、疫情数据
- **数据类型**:
  - 传染病监测数据
  - 慢性病统计
  - 疫苗接种率

#### 美国人口普查局（Census Bureau）
- **领域**: 人口统计、经济数据、地理信息
- **核心数据集**:
  - 十年人口普查数据
  - 美国社区调查（ACS）
  - 经济普查数据

---

## 🇨🇳 中国政府数据开放平台生态

### 国家层面平台

#### 中国政府公开信息整合服务平台
- **官网**: http://govinfo.nlc.gov.cn/
- **定位**: 政府公报、法律法规等信息发布
- **现状**: 目前主要是信息发布，与真正意义上的"数据开放"仍有差距
- **政策目标**: 2018年底前建成国家政府数据统一开放平台（目标推进中）

#### 国家政府数据统一开放平台（建设中）
- **规划时间**: 2015年《促进大数据发展行动纲要》提出
- **目标时间**: 2018年底前建成
- **定位**: 
  - 政府统一管理开放数据的平台
  - 企业和个人访问政府数据的统一入口
  - 与政务信息公开既有联系又有区别
- **数据特征**: 可下载、可机读、非涉密政务数据

---

### 省市级开放平台汇总

#### 北京市
- **平台名称**: 北京市政务数据资源网
- **官网**: https://data.beijing.gov.cn
- **数据规模**: 
  - 开放部门: 103个
  - 数据集: 10,266个
- **特色**: 提供数据集浏览、下载、API接口服务

#### 上海市
- **平台名称**: 上海市政府数据服务网 / 上海市公共数据开放平台
- **官网**: https://data.sh.gov.cn/
- **数据规模**: 
  - 开放机构: 100个
  - 数据资源: 4,694项
  - 数据接口: 9657个
- **特色**: 2012年6月上线，中国第一个开放数据门户网站
- **API特点**: 接口服务较完善，但存在调用难度高、数据容量小、更新频率低等问题

#### 深圳市
- **平台名称**: 深圳市政府数据开放平台
- **官网**: http://opendata.sz.gov.cn/
- **数据规模**: 
  - 数据目录: 1,260个
  - 数据总量: 121,338,216条
  - 数据接口: 1,002个
  - 调用次数: 1,945,673次
- **特色**: 接口调用活跃，数据更新相对及时

#### 浙江省
- **平台名称**: 浙江省人民政府数据开放网站 / 浙江数据开放
- **官网**: https://data.zjzwfw.gov.cn/
- **数据规模**: 
  - 数据集: 18,306个
  - 数据接口: 9657个
- **特色**: 数据集数量全国领先，API接口丰富

#### 其他主要省市
| 地区 | 平台官网 | 数据规模 |
|------|---------|---------|
| 广州 | http://data.gz.gov.cn/ | 68部门，1,307数据集，1亿+条数据 |
| 天津 | https://data.tj.gov.cn/ | 52部门，1,095数据集，508接口 |
| 四川 | http://www.scdata.net.cn/ | 覆盖成都、达州、雅安等 |

---

### 其他重要数据接口

#### 中国气象局 API
- **平台**: 智慧天气应用编程接口开放平台
- **官网**: http://smart.weather.com.cn/wzfw/smart/weatherapi.shtml
- **API 接口**:
  - 实时天气: http://www.weather.com.cn/data/sk/{城市代码}.html
  - 城市信息: http://www.weather.com.cn/data/cityinfo/{城市代码}.html
  - 未来天气预报: http://m.weather.com.cn/data/{城市代码}.html
- **返回格式**: JSON
- **数据内容**: 温度、天气状况、风向风力、穿衣指数、紫外线指数等

#### 中国科技云大模型 API 开放平台
- **官网**: https://uni-api.cstcloud.cn/
- **英文版**: https://uni-api-global.cstcloud.cn/
- **平台特色**: 
  - 通过标准化API接口封装人工智能模型能力
  - 解决模型接口差异问题
  - 支持在线申请、审批等功能
  - API Key 认证体系
  - Token 计量与统计
- **已集成模型**:
  - DeepSeek-R1/V3（671B）
  - Qwen3（235B）
  - Qwen2.5-VL（72B）
  - bge-large-zh（嵌入模型）
  - bge-reranker-V2-m3（重排模型）
- **调用统计**: Token调用量已突破500亿（截至2025年8月）

---

## 中美政府数据开放对比

| 对比维度 | 美国 Data.gov | 中国各省市平台 |
|---------|--------------|--------------|
| **起步时间** | 2009年 | 2012年（上海、北京） |
| **统一平台** | ✅ 有（Data.gov） | ⚠️ 国家平台建设中，各省市分散 |
| **数据规模** | 30万数据集 | 不等，浙江1.8万、北京1万+ |
| **API标准化** | ✅ 统一REST API标准 | ⚠️ 各平台接口不统一 |
| **更新频率** | 70%以上数据集年内更新 | ⚠️ 普遍更新频率较低 |
| **法律保障** | ✅ 《信息自由法》、《开放政府法》 | ⚠️ 缺乏专门法律 |
| **数据格式** | ✅ JSON/CSV/XML统一 | ⚠️ 格式不统一 |
| **文档完善度** | ✅ 开发者友好的文档 | ⚠️ 文档参差不齐 |
| **调用便利性** | ✅ 统一认证、配额管理 | ⚠️ 调用难度较高 |

---

# 第四部分：综合优先级建议

## 🚀 第一批：立即接入（1-2个月）

### 1. 高价值商业情报
- ✅ **企业信息查询**（天眼查/企查查）
- ✅ **代码仓库**（GitHub API）
- ✅ **专利与知识产权**（Google Patents/USPTO）
- ✅ **产品与电商情报**（淘宝/京东/亚马逊）

### 2. 政府数据核心源
- ✅ **美国 Data.gov** - 经济、气候、公共卫生、交通
- ✅ **中国三大平台** - 北京（政策）、上海（开放最早）、深圳（数据量大）
- ✅ **美国联邦机构API** - FDA、EPA、CDC

### 3. 法律与舆情
- ✅ **法律法规与政策**（北大法宝）
- ✅ **舆情与社交媒体**（微博/抖音）

---

## 🌟 第二批：扩展覆盖（2-3个月）

### 4. 人才与市场
- ✅ **求职招聘与人才市场**（Boss 直聘/拉勾/LinkedIn）
- ✅ **票务与活动信息**（大麦/猫眼）

### 5. 其他政府平台
- ✅ 其他省市平台（浙江、广东、贵州、福建）
- ✅ 专用API（中国气象局、中国科技云大模型）

---

## 💡 第三批：补充资源（持续）

### 6. 生活服务类
- ✅ **物流与快递**（顺丰/圆通/中通）
- ✅ **旅游与交通**（TripAdvisor/携程）
- ✅ **医疗健康**（好大夫）
- ✅ **教育课程**（Coursera/Udemy）

### 7. 国际数据源
- ✅ 国外政府平台（英国 Gov.uk、新加坡）
- ✅ 国际组织（联合国、OECD、世界银行）

---

## 🛠️ 技术实施建议

### 数据采集架构

```
┌─────────────────────────────────────────────────────┐
│          统一数据采集层（Data Ingestion）          │
├─────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ 现有MCP  │  │ 新增API  │  │ 政府数据  │   │
│  │  适配器   │  │  适配器   │  │  适配器   │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       │             │             │             │
│       └─────────────┼─────────────┘             │
│                     ▼                           │
│          ┌─────────────────┐                   │
│          │  数据标准化层   │                   │
│          │  JSON/CSV/XML  │                   │
│          └────────┬────────┘                   │
│                   ▼                           │
│          ┌─────────────────┐                   │
│          │  数据存储与索引  │                   │
│          │  (Elasticsearch) │                   │
│          └────────┬────────┘                   │
│                   ▼                           │
│          ┌─────────────────┐                   │
│          │  API 服务层     │                   │
│          │  (统一查询接口)  │                   │
│          └─────────────────┘                   │
└─────────────────────────────────────────────────────┘
```

### 统一API抽象层示例

```python
class ExternalIntelAPI:
    """外部情报统一API抽象层"""
    
    def __init__(self):
        # 现有MCP工具
        self.existing_tools = {
            'finance': FinanceMCPClient(),
            'pubmed': PubMedClient(),
            'openalex': OpenAlexClient(),
            'amap': AmapClient(),
            'brave': BraveSearchClient(),
            'tavily': TavilyClient()
        }
        
        # 新增API工具
        self.new_tools = {
            'company': CompanyInfoClient(),  # 天眼查/企查查
            'github': GitHubClient(),
            'patent': PatentClient(),  # Google Patents/USPTO
            'ecommerce': EcommerceClient(),  # 淘宝/京东
            'legal': LegalClient(),  # 北大法宝
            'social': SocialMediaClient(),  # 微博/抖音
            'hr': JobMarketClient(),  # Boss直聘/拉勾
        }
        
        # 政府数据工具
        self.gov_tools = {
            'us_data_gov': USDataGovClient(),
            'us_fda': FDAClient(),
            'us_epa': EPAClient(),
            'cn_beijing': BeijingDataClient(),
            'cn_shanghai': ShanghaiDataClient(),
            'cn_shenzhen': ShenzhenDataClient()
        }
    
    def search_company(self, company_name, country='CN'):
        """企业信息查询"""
        if country == 'CN':
            return self.new_tools['company'].search(company_name)
        elif country == 'US':
            # 可能需要从政府数据或其他商业API获取
            return self.gov_tools['us_data_gov'].search_company(company_name)
    
    def search_code(self, query, language='javascript'):
        """代码仓库搜索"""
        return self.new_tools['github'].search(query, language=language)
    
    def search_patent(self, query, country='CN'):
        """专利查询"""
        return self.new_tools['patent'].search(query, country=country)
    
    def query_government_data(self, dataset_type, country='CN'):
        """政府数据查询"""
        if country == 'US':
            return self.gov_tools['us_data_gov'].query(dataset_type)
        elif country == 'CN':
            # 可能需要从多个省市平台聚合
            return self._aggregate_cn_data(dataset_type)
```

### 数据更新策略

| 数据类型 | 更新频率 | 数据源 | 缓存策略 |
|---------|---------|--------|---------|
| 实时数据 | 每小时/每分钟 | 天气API、实时交通 | Redis缓存，TTL=1小时 |
| 日度数据 | 每日 | 经济指标、股票行情 | Redis缓存，TTL=24小时 |
| 月度数据 | 每月 | 就业统计、CPI | 数据库存储，按需刷新 |
| 年度数据 | 每年 | 人口普查、GDP | 数据库存储，年度更新 |

---

## 总结与建议

### 核心观点
1. **现有工具基础扎实** - 搜索、金融、学术、地图、命理五大类工具已覆盖核心需求
2. **政府API生态差异大** - 美国统一成熟，中国分散建设中，但都具有重要价值
3. **建议分批接入** - 先高价值商业情报 + 政府核心数据，再逐步扩展
4. **技术架构先行** - 建立统一数据采集层和API抽象层，屏蔽底层差异

### 给主人的建议
- 🔥 **立即行动**: 开始接入天眼查/企查查、GitHub API、Data.gov和中国三大平台
- 🛠️ **技术准备**: 设计统一API抽象层和数据标准化方案
- 📊 **价值验证**: 先选择2-3个高价值数据源验证能力
- 🚀 **持续迭代**: 逐步扩展数据源覆盖范围

---

**文档结束**

*小虾整理 · 2026-03-27*  
*如需补充或修改，随时告诉主人～ 🦞✨*
