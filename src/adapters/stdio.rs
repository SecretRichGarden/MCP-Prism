//! stdio 型 MCP provider 的运行时聚合：
//! - `StdioMcpRegistry` 管理全部 stdio 子进程会话 + 工具**透传**（tools/list 聚合 / tools/call 路由）
//! - 工具命名空间 `{provider_id}__{tool_name}`，避免与 prism 原生工具冲突
//! - 子进程保活：调用时检测退出则自动重启（重新握手 + 重新索引工具）

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::config::{AppConfig, StdioProviderConfig};
use crate::error::{AppError, AppResult};
use crate::stdio_client::StdioMcpClient;

const TOOL_NS_SEP: &str = "__";

/// stdio provider 会话注册表 + 透传工具索引。
#[derive(Clone)]
pub struct StdioMcpRegistry {
    clients: Arc<RwLock<HashMap<String, Arc<StdioMcpClient>>>>,
    configs: Arc<RwLock<Vec<StdioProviderConfig>>>,
    tools: Arc<RwLock<Vec<Value>>>,
}

impl StdioMcpRegistry {
    /// 同步空构造（AppRuntime::from_config 阶段），随后由 `init` 异步 spawn。
    pub fn empty() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(Vec::new())),
            tools: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 异步初始化：spawn 全部 enabled stdio provider 并握手/索引工具。
    pub async fn init(&self, config: &AppConfig) {
        {
            let mut configs = self.configs.write().await;
            *configs = config
                .stdio_providers
                .iter()
                .filter(|provider| provider.is_available())
                .cloned()
                .collect::<Vec<_>>();
        }
        let snapshot = self.configs.read().await.clone();
        for provider in snapshot.iter() {
            match self.spawn_one(provider).await {
                Ok(()) => info!(id = %provider.id, "stdio MCP provider connected"),
                Err(error) => {
                    warn!(id = %provider.id, "stdio MCP provider init failed: {error}")
                }
            }
        }
        let count = snapshot.len();
        info!(count, "stdio MCP registry initialized");
    }

    /// spawn 一个 stdio provider：拉起子进程 → 握手 → 索引透传工具。
    async fn spawn_one(&self, config: &StdioProviderConfig) -> AppResult<()> {
        // 登记配置（保活重启用）；id 已存在则不重复登记
        {
            let mut configs = self.configs.write().await;
            if !configs.iter().any(|item| item.id == config.id) {
                configs.push(config.clone());
            }
        }
        let client = StdioMcpClient::spawn(config).await?;
        client.initialize().await?;
        client.initialized().await?;
        let tools = client.list_tools().await?;

        // 替换该 provider 的旧透传工具（重启时避免残留）
        let prefix = format!("{}{}", config.id, TOOL_NS_SEP);
        let mut all = self.tools.write().await;
        all.retain(|tool| {
            tool.get("name")
                .and_then(|name| name.as_str())
                .map_or(true, |name| !name.starts_with(&prefix))
        });
        for tool in tools {
            let name = match tool.get("name").and_then(|value| value.as_str()) {
                Some(name) if !name.is_empty() => name,
                _ => continue,
            };
            let mut definition = tool.clone();
            definition["name"] = json!(format!("{}{}{}", config.id, TOOL_NS_SEP, name));
            let description = definition
                .get("description")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            definition["description"] =
                json!(format!("[{}] {}", config.title, description));
            all.push(definition);
        }
        drop(all);

        self.clients
            .write()
            .await
            .insert(config.id.clone(), Arc::new(client));
        Ok(())
    }

    /// 全部透传工具定义（命名空间化），供 tools/list 合并。
    pub async fn tool_definitions(&self) -> Vec<Value> {
        self.tools.read().await.clone()
    }

    /// 该命名空间化工具名是否为已注册的 stdio 透传工具。
    pub async fn has_tool(&self, full_name: &str) -> bool {
        self.tools
            .read()
            .await
            .iter()
            .any(|tool| tool.get("name").and_then(|name| name.as_str()) == Some(full_name))
    }

    /// 透传工具调用：`{provider_id}__{tool_name}` → 路由到对应子进程。
    pub async fn call_tool(&self, full_name: &str, arguments: Value) -> AppResult<Value> {
        let (provider_id, tool_name) = full_name
            .split_once(TOOL_NS_SEP)
            .ok_or_else(|| AppError::Validation(format!("invalid stdio tool name: {full_name}")))?;
        let client = self
            .clients
            .read()
            .await
            .get(provider_id)
            .cloned()
            .ok_or_else(|| AppError::Validation(format!("unknown stdio provider: {provider_id}")))?;

        // 保活：子进程退出则自动重启（重新握手 + 重新索引工具）后重试一次
        if !client.is_alive().await {
            warn!(provider = %provider_id, "stdio MCP dead; restarting");
            self.clients.write().await.remove(provider_id);
            let config = self
                .configs
                .read()
                .await
                .iter()
                .find(|config| config.id == provider_id)
                .cloned()
                .ok_or_else(|| AppError::Internal(format!("stdio config missing: {provider_id}")))?;
            self.spawn_one(&config).await?;
            let client = self
                .clients
                .read()
                .await
                .get(provider_id)
                .cloned()
                .ok_or_else(|| AppError::Internal(format!("restart failed: {provider_id}")))?;
            return client.call_tool(tool_name, arguments).await;
        }

        client.call_tool(tool_name, arguments).await
    }

    /// 关闭全部子进程（服务退出时调用）。
    pub async fn shutdown(&self) {
        for (_, client) in self.clients.read().await.iter() {
            client.shutdown().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StdioProviderConfig;
    use std::collections::HashMap;

    fn provider_config(id: &str, command: &str, args: Vec<String>) -> StdioProviderConfig {
        StdioProviderConfig {
            id: id.to_string(),
            title: "Mock".to_string(),
            category: "search".to_string(),
            command: command.to_string(),
            args,
            env: HashMap::new(),
            cwd: None,
            enabled: true,
            search_capable: true,
            search_types: vec!["web".to_string()],
            rpm: 1.0,
        }
    }

    const MOCK_SERVER: &str = r#"
const readline = require('readline');
const rl = readline.createInterface({input: process.stdin, crlfDelay: Infinity});
rl.on('line', (line) => {
  const req = JSON.parse(line);
  if (req.method === 'initialize') {
    console.log(JSON.stringify({jsonrpc:'2.0',id:req.id,result:{protocolVersion:'2024-11-05',capabilities:{},serverInfo:{name:'mock',version:'1'}}}));
  } else if (req.method === 'tools/list') {
    console.log(JSON.stringify({jsonrpc:'2.0',id:req.id,result:{tools:[
      {name:'mock_echo',description:'echo a message',inputSchema:{type:'object',properties:{text:{type:'string'}}}},
      {name:'mock_add',description:'add two numbers',inputSchema:{type:'object',properties:{a:{type:'number'},b:{type:'number'}}}}
    ]}}));
  } else if (req.method === 'tools/call') {
    const p = req.params.arguments || {};
    if (req.params.name === 'mock_add') {
      console.log(JSON.stringify({jsonrpc:'2.0',id:req.id,result:{content:[{type:'text',text:String((p.a||0)+(p.b||0))}]}}));
    } else {
      console.log(JSON.stringify({jsonrpc:'2.0',id:req.id,result:{content:[{type:'text',text:'echo:'+String(p.text||'')}]}}));
    }
  }
});
"#;

    #[tokio::test]
    async fn registry_pass_through_tools_and_call() {
        let config = provider_config(
            "mockprov",
            "node",
            vec!["-e".to_string(), MOCK_SERVER.to_string()],
        );
        let registry = StdioMcpRegistry::empty();
        registry.spawn_one(&config).await.expect("spawn mock");

        // 透传工具索引（命名空间化）
        let tools = registry.tool_definitions().await;
        assert_eq!(tools.len(), 2);
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "mockprov__mock_echo"));
        assert!(tools
            .iter()
            .any(|tool| tool["name"] == "mockprov__mock_add"));
        assert!(registry.has_tool("mockprov__mock_add").await);
        assert!(!registry.has_tool("unknown_prov__x").await);

        // tools/call 透传路由
        let result = registry
            .call_tool("mockprov__mock_add", json!({"a": 2, "b": 3}))
            .await
            .expect("call mock_add");
        assert_eq!(result["content"][0]["text"], "5");

        // 未知 provider 报错
        let err = registry
            .call_tool("nope__mock_add", json!({}))
            .await
            .expect_err("unknown provider should fail");
        assert!(err.to_string().contains("unknown stdio provider"));

        registry.shutdown().await;
    }

    #[tokio::test]
    async fn registry_restarts_dead_process() {
        // mock server 处理完首个工具调用后异步退出（模拟崩溃，先回响应再退出）
        let script = r#"
const readline = require('readline');
const rl = readline.createInterface({input: process.stdin, crlfDelay: Infinity});
let calls = 0;
rl.on('line', (line) => {
  const req = JSON.parse(line);
  if (req.method === 'initialize') {
    console.log(JSON.stringify({jsonrpc:'2.0',id:req.id,result:{protocolVersion:'2024-11-05',capabilities:{},serverInfo:{name:'mock',version:'1'}}}));
  } else if (req.method === 'tools/list') {
    console.log(JSON.stringify({jsonrpc:'2.0',id:req.id,result:{tools:[{name:'mock_tool',description:'d',inputSchema:{type:'object',properties:{}}}]}}));
  } else if (req.method === 'tools/call') {
    calls++;
    console.log(JSON.stringify({jsonrpc:'2.0',id:req.id,result:{content:[{type:'text',text:'ok-'+calls}]}}));
    setTimeout(() => process.exit(0), 5); // 回完响应后崩溃退出
  }
});
"#;
        let config = provider_config(
            "crashprov",
            "node",
            vec!["-e".to_string(), script.to_string()],
        );
        let registry = StdioMcpRegistry::empty();
        registry.spawn_one(&config).await.expect("spawn");

        let first = registry
            .call_tool("crashprov__mock_tool", json!({}))
            .await
            .expect("first call");
        assert_eq!(first["content"][0]["text"], "ok-1");

        // 等子进程退出被 try_wait 捕获，再触发自动重启
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let second = registry
            .call_tool("crashprov__mock_tool", json!({}))
            .await
            .expect("auto-restart call");
        assert!(second["content"][0]["text"].as_str().unwrap().starts_with("ok-"));

        registry.shutdown().await;
    }
}
