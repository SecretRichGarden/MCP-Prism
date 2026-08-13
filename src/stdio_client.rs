//! MCP stdio client — 以子进程方式运行外部 stdio MCP server，
//! 通过 stdin/stdout 走 JSON-RPC 行协议（对称于 `server::run_stdio` 的 server 端实现）。

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{Mutex, oneshot};
use tracing::{debug, warn};

use crate::config::StdioProviderConfig;
use crate::error::{AppError, AppResult};

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// 一个 stdio MCP server 子进程的 JSON-RPC 会话。
pub struct StdioMcpClient {
    provider_id: String,
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    next_id: Mutex<u64>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
}

impl StdioMcpClient {
    /// spawn 子进程并启动 stdout 读取循环。
    pub async fn spawn(config: &StdioProviderConfig) -> AppResult<Self> {
        let mut command = Command::new(&config.command);
        command
            .args(&config.args)
            .envs(&config.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()); // 子进程 stderr 透出到 prism 日志，便于排障
        if let Some(cwd) = &config.cwd {
            command.current_dir(cwd);
        }
        let mut child = command.spawn().map_err(|error| {
            AppError::Internal(format!(
                "spawn stdio MCP '{}' failed: {error}",
                config.id
            ))
        })?;
        let stdin = child.stdin.take().ok_or_else(|| {
            AppError::Internal(format!("stdio MCP '{}' stdin not available", config.id))
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            AppError::Internal(format!("stdio MCP '{}' stdout not available", config.id))
        })?;

        let client = Self {
            provider_id: config.id.clone(),
            child: Mutex::new(child),
            stdin: Mutex::new(stdin),
            next_id: Mutex::new(1),
            pending: Arc::new(Mutex::new(HashMap::new())),
        };
        client.spawn_reader(stdout);
        Ok(client)
    }

    /// 后台任务：逐行读子进程 stdout，按 id 投递响应。
    fn spawn_reader(&self, stdout: tokio::process::ChildStdout) {
        let pending = self.pending.clone();
        let provider_id = self.provider_id.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                let value: Value = match serde_json::from_str(&line) {
                    Ok(value) => value,
                    Err(error) => {
                        warn!(provider=%provider_id, "non-JSON line from stdio MCP: {error}");
                        continue;
                    }
                };
                let id_value = match value.get("id") {
                    Some(id) => id.clone(),
                    None => continue, // 无 id 的通知，忽略
                };
                let id = id_value.as_i64().unwrap_or(-1);
                if id < 0 {
                    continue;
                }
                let response = if let Some(error) = value.get("error") {
                    json!({"error": error})
                } else {
                    value.get("result").cloned().unwrap_or(Value::Null)
                };
                if let Some(sender) = pending.lock().await.remove(&(id as u64)) {
                    let _ = sender.send(response);
                }
            }
            // stdout EOF → 通知所有等待中的请求"连接关闭"（发送 Null 占位）
            let mut guard = pending.lock().await;
            let drained = std::mem::take(&mut *guard);
            drop(guard);
            for (_, sender) in drained {
                let _ = sender.send(Value::Null);
            }
            debug!(provider=%provider_id, "stdio MCP stdout closed");
        });
    }

    /// 发一个 JSON-RPC 请求并等待响应（带超时）。
    async fn request(&self, method: &str, params: Value) -> AppResult<Value> {
        let id = {
            let mut next = self.next_id.lock().await;
            let id = *next;
            *next += 1;
            id
        };
        let (sender, receiver) = oneshot::channel();
        self.pending.lock().await.insert(id, sender);

        let payload = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        let mut line = serde_json::to_string(&payload)?;
        line.push('\n');
        {
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(line.as_bytes()).await.map_err(|error| {
                AppError::Internal(format!(
                    "write to stdio MCP '{}' failed: {error}",
                    self.provider_id
                ))
            })?;
            stdin.flush().await.map_err(|error| {
                AppError::Internal(format!(
                    "flush stdin of '{}' failed: {error}",
                    self.provider_id
                ))
            })?;
        }

        let result = match tokio::time::timeout(DEFAULT_REQUEST_TIMEOUT, receiver).await {
            Ok(Ok(value)) => value,
            Ok(Err(_)) => {
                return Err(AppError::Internal(format!(
                    "stdio MCP '{}' response channel dropped",
                    self.provider_id
                )));
            }
            Err(_) => {
                self.pending.lock().await.remove(&id);
                return Err(AppError::Internal(format!(
                    "stdio MCP '{}' request {method} timed out after {}s",
                    self.provider_id,
                    DEFAULT_REQUEST_TIMEOUT.as_secs()
                )));
            }
        };

        if result == Value::Null {
            return Err(AppError::Internal(format!(
                "stdio MCP '{}' connection closed (process exited?)",
                self.provider_id
            )));
        }
        if let Some(error) = result.get("error") {
            return Err(AppError::Provider(format!(
                "stdio MCP '{}' returned error: {}",
                self.provider_id, error
            )));
        }
        Ok(result)
    }

    /// MCP initialize 握手。
    pub async fn initialize(&self) -> AppResult<Value> {
        self.request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "mcp-prism", "version": env!("CARGO_PKG_VERSION")}
            }),
        )
        .await
    }

    /// 发送 notifications/initialized（无响应）。
    pub async fn initialized(&self) -> AppResult<()> {
        let mut line = serde_json::to_string(&json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        }))?;
        line.push('\n');
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(line.as_bytes()).await.map_err(|error| {
            AppError::Internal(format!(
                "write initialized to '{}' failed: {error}",
                self.provider_id
            ))
        })?;
        stdin.flush().await.map_err(|error| {
            AppError::Internal(format!("flush '{}' failed: {error}", self.provider_id))
        })
    }

    /// 获取子进程暴露的工具列表（MCP tools/list）。
    pub async fn list_tools(&self) -> AppResult<Vec<Value>> {
        let result = self.request("tools/list", json!({})).await?;
        Ok(result
            .get("tools")
            .and_then(|tools| tools.as_array())
            .cloned()
            .unwrap_or_default())
    }

    /// 调用子进程工具（MCP tools/call）。
    pub async fn call_tool(&self, name: &str, arguments: Value) -> AppResult<Value> {
        self.request(
            "tools/call",
            json!({
                "name": name,
                "arguments": arguments
            }),
        )
        .await
    }

    /// 子进程是否存活（try_wait 探测，用于保活检测）。
    pub async fn is_alive(&self) -> bool {
        let mut child = self.child.lock().await;
        match child.try_wait() {
            Ok(Some(status)) => {
                warn!(provider=%self.provider_id, "stdio MCP exited with status {status}");
                false
            }
            Ok(None) => true,
            Err(error) => {
                warn!(provider=%self.provider_id, "try_wait error: {error}");
                false
            }
        }
    }

    /// 关闭子进程并回收。
    pub async fn shutdown(&self) {
        let mut child = self.child.lock().await;
        let _ = child.kill().await;
        let _ = child.wait().await;
    }

    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(command: &str, args: Vec<String>) -> StdioProviderConfig {
        StdioProviderConfig {
            id: "mock".to_string(),
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

    #[tokio::test]
    async fn initialize_and_list_tools() {
        // 用 `node` 跑一段内联脚本模拟 stdio MCP server：initialize 回显 + tools/list 返回一个工具。
        let script = r#"
const readline = require('readline');
const rl = readline.createInterface({input: process.stdin, crlfDelay: Infinity});
rl.on('line', (line) => {
  const req = JSON.parse(line);
  if (req.method === 'initialize') {
    console.log(JSON.stringify({jsonrpc:'2.0',id:req.id,result:{protocolVersion:'2024-11-05',capabilities:{},serverInfo:{name:'mock',version:'1'}}}));
  } else if (req.method === 'tools/list') {
    console.log(JSON.stringify({jsonrpc:'2.0',id:req.id,result:{tools:[{name:'mock_echo',description:'echo',inputSchema:{type:'object',properties:{}}} ]}}));
  } else if (req.method === 'tools/call') {
    const name = req.params.name;
    console.log(JSON.stringify({jsonrpc:'2.0',id:req.id,result:{content:[{type:'text',text:'echo:'+name}]}}));
  }
});
"#;
        let client = StdioMcpClient::spawn(&test_config(
            "node",
            vec!["-e".to_string(), script.to_string()],
        ))
        .await
        .expect("spawn node mock");
        let init = client.initialize().await.expect("initialize");
        assert_eq!(init["serverInfo"]["name"], "mock");
        client.initialized().await.expect("initialized");
        let tools = client.list_tools().await.expect("list_tools");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "mock_echo");
        let result = client
            .call_tool("mock_echo", json!({}))
            .await
            .expect("call_tool");
        assert_eq!(result["content"][0]["text"], "echo:mock_echo");
        client.shutdown().await;
    }

    #[tokio::test]
    async fn missing_command_errors() {
        let config = test_config("definitely-not-a-real-binary-xyz", vec![]);
        assert!(StdioMcpClient::spawn(&config).await.is_err());
    }
}
