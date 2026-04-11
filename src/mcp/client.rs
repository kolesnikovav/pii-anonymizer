use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tracing::info;

/// Тип транспорта для MCP сервера
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum McpTransport {
    /// stdio — subprocess (локальный запуск)
    #[default]
    Stdio,
    /// HTTP/SSE — удалённое подключение
    Http,
}

/// Конфигурация внешнего MCP сервера
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExternalMcpConfig {
    /// Транспорт: stdio или http
    #[serde(default)]
    pub transport: McpTransport,

    // ── Для stdio ──
    /// Команда для запуска (только stdio)
    pub command: Option<String>,
    /// Аргументы команды (только stdio)
    #[serde(default)]
    pub args: Vec<String>,
    /// Переменные окружения (только stdio)
    #[serde(default)]
    pub env: HashMap<String, String>,

    // ── Для HTTP ──
    /// URL сервера (только http)
    pub url: Option<String>,
    /// Заголовки авторизации (только http)
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Включён ли сервер
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    // ── Анонимизация ──
    /// Какие поля каких инструментов анонимизировать.
    /// Формат: { "tool_name": ["field1", "field2"] }
    /// Если не указано — анонимизируются все строковые значения (обратная совместимость).
    /// Если указано пустой список `[]` — анонимизация отключена для этого сервера.
    #[serde(default)]
    pub anonymize_fields: HashMap<String, Vec<String>>,
}

fn default_enabled() -> bool {
    true
}

impl ExternalMcpConfig {
    pub fn stdio(command: String) -> Self {
        Self {
            transport: McpTransport::Stdio,
            command: Some(command),
            args: Vec::new(),
            env: HashMap::new(),
            url: None,
            headers: HashMap::new(),
            enabled: true,
            anonymize_fields: HashMap::new(),
        }
    }

    pub fn http(url: String) -> Self {
        Self {
            transport: McpTransport::Http,
            command: None,
            args: Vec::new(),
            env: HashMap::new(),
            url: Some(url),
            headers: HashMap::new(),
            enabled: true,
            anonymize_fields: HashMap::new(),
        }
    }

    pub fn http_with_auth(url: String, auth_header: &str, auth_value: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert(auth_header.to_string(), auth_value.to_string());
        Self {
            transport: McpTransport::Http,
            command: None,
            args: Vec::new(),
            env: HashMap::new(),
            url: Some(url),
            headers,
            enabled: true,
            anonymize_fields: HashMap::new(),
        }
    }
}

/// Конфигурация прокси менеджера
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct McpProxyConfig {
    #[serde(default)]
    pub upstream_servers: HashMap<String, ExternalMcpConfig>,
}

/// Инструмент от внешнего сервера
#[derive(Clone, Debug)]
pub struct ExternalTool {
    pub server_name: String,
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════
/// stdio подключение (JSON-RPC через subprocess)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
}

pub(crate) struct StdioConnection {
    _child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
    request_id: Arc<Mutex<u64>>,
}

impl StdioConnection {
    async fn send_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let id = {
            let mut id = self.request_id.lock().await;
            let c = *id;
            *id += 1;
            c
        };

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let json = serde_json::to_string(&request).map_err(|e| format!("Serialize: {}", e))?;
        let mut stdin = self.stdin.lock().await;
        stdin
            .write_all(format!("{}\n", json).as_bytes())
            .await
            .map_err(|e| format!("Write: {}", e))?;
        stdin.flush().await.map_err(|e| format!("Flush: {}", e))?;

        let mut stdout = self.stdout.lock().await;
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = stdout
                .read_line(&mut line)
                .await
                .map_err(|e| format!("Read: {}", e))?;
            if bytes == 0 {
                return Err("Connection closed".to_string());
            }

            if line.trim().is_empty() {
                continue;
            }

            if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(line.trim()) {
                if response.id == Some(id) {
                    if let Some(err) = response.error {
                        return Err(format!("MCP error: {}", err));
                    }
                    return response.result.ok_or("Empty result".to_string());
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HTTP/SSE подключение (SSE endpoint + POST messages)
// ═══════════════════════════════════════════════════════════════════════════
pub(crate) struct HttpConnection {
    base_url: String,
    session_id: Arc<Mutex<Option<String>>>,
    client: reqwest::Client,
    headers: HashMap<String, String>,
    request_id: Arc<Mutex<u64>>,
}

impl HttpConnection {
    fn new(url: String, headers: HashMap<String, String>) -> Self {
        Self {
            base_url: url.trim_end_matches('/').to_string(),
            session_id: Arc::new(Mutex::new(None)),
            client: reqwest::Client::new(),
            headers,
            request_id: Arc::new(Mutex::new(1)),
        }
    }

    async fn get_session_id(&self) -> Result<String, String> {
        let mut session = self.session_id.lock().await;
        if session.is_some() {
            return Ok(session.as_ref().unwrap().clone());
        }

        info!("   🔄 SSE подключение к {}/sse...", self.base_url);
        let resp = self
            .client
            .get(format!("{}/sse", self.base_url))
            .send()
            .await
            .map_err(|e| format!("SSE: {}", e))?;
        let body = resp.text().await.map_err(|e| format!("Read SSE: {}", e))?;

        if let Some(line) = body.lines().find(|l| l.starts_with("data: ")) {
            let endpoint = line.trim_start_matches("data: ");
            if let Some(sid) = endpoint.split("sessionId=").nth(1) {
                info!("   ✅ Session: {}...", &sid[..20.min(sid.len())]);
                *session = Some(sid.to_string());
                return Ok(sid.to_string());
            }
        }
        Err("Failed to get session ID".to_string())
    }

    async fn send_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let id = {
            let mut id = self.request_id.lock().await;
            let c = *id;
            *id += 1;
            c
        };
        let session_id = self.get_session_id().await?;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let message_url = format!("{}/message?sessionId={}", self.base_url, session_id);
        info!("   📡 {} → {}", method, message_url);

        let mut req = self
            .client
            .post(&message_url)
            .header("Content-Type", "application/json");
        for (k, v) in &self.headers {
            req = req.header(k, v);
        }

        let resp = req
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP: {}", e))?;
        let body = resp.text().await.map_err(|e| format!("Read: {}", e))?;

        let json_str = body
            .lines()
            .find(|l| l.starts_with("data: "))
            .map(|l| l.trim_start_matches("data: "))
            .unwrap_or(&body);

        let result: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Parse: {} from: {}", e, json_str))?;

        if let Some(err) = result.get("error") {
            return Err(format!("MCP error: {}", err));
        }
        result
            .get("result")
            .cloned()
            .ok_or("Empty result".to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Универсальное подключение
// ═══════════════════════════════════════════════════════════════════════════
pub(crate) enum Transport {
    Stdio(StdioConnection),
    Http(HttpConnection),
}

impl Transport {
    async fn send_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match self {
            Transport::Stdio(conn) => conn.send_request(method, params).await,
            Transport::Http(conn) => conn.send_request(method, params).await,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Подключение к внешнему MCP серверу
// ═══════════════════════════════════════════════════════════════════════════
pub struct McpUpstreamConnection {
    pub name: String,
    pub(crate) transport: Arc<Mutex<Transport>>,
    pub tools: Vec<ExternalTool>,
}

impl McpUpstreamConnection {
    pub async fn connect(name: String, config: &ExternalMcpConfig) -> Result<Self, String> {
        info!(
            "🔌 Подключение к MCP серверу: {} ({:?})",
            name, config.transport
        );

        let transport = match config.transport {
            McpTransport::Stdio => Self::connect_stdio(&name, config).await?,
            McpTransport::Http => Self::connect_http(&name, config).await?,
        };

        let mut conn = Self {
            name: name.clone(),
            transport: Arc::new(Mutex::new(transport)),
            tools: Vec::new(),
        };

        // Initialize MCP
        conn.initialize().await?;
        conn.load_tools().await?;

        info!(
            "✅ Подключено к {}: {} инструментов",
            name,
            conn.tools.len()
        );
        for t in &conn.tools {
            info!("   └─ {}", t.name);
        }

        Ok(conn)
    }

    async fn connect_stdio(name: &str, config: &ExternalMcpConfig) -> Result<Transport, String> {
        let command = config
            .command
            .as_ref()
            .ok_or("Command required for stdio transport")?;

        info!("   Команда: {} {}", command, config.args.join(" "));

        let mut cmd = Command::new(command);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        for (k, v) in &config.env {
            cmd.env(k, v);
        }
        cmd.args(&config.args);

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Spawn {}: {}", command, e))?;
        let stdin = child.stdin.take().ok_or("No stdin")?;
        let stdout = child.stdout.take().ok_or("No stdout")?;
        let stderr = child.stderr.take().ok_or("No stderr")?;

        // Drain stderr in background to prevent blocking
        let stderr_name = name.to_string();
        tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut reader = BufReader::new(stderr);
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let msg = String::from_utf8_lossy(&buf[..n]);
                        for line in msg.lines() {
                            tracing::info!("   [{} stderr] {}", stderr_name, line);
                        }
                    }
                }
            }
        });

        // Give the subprocess time to initialize before sending requests
        // Some MCP servers (especially Docker-based) need a few seconds to start
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        Ok(Transport::Stdio(StdioConnection {
            _child: child,
            stdin: Arc::new(Mutex::new(stdin)),
            stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
            request_id: Arc::new(Mutex::new(1)),
        }))
    }

    async fn connect_http(_name: &str, config: &ExternalMcpConfig) -> Result<Transport, String> {
        let url = config
            .url
            .as_ref()
            .ok_or("URL required for http transport")?;

        info!("   URL: {}", url);

        Ok(Transport::Http(HttpConnection::new(
            url.clone(),
            config.headers.clone(),
        )))
    }

    async fn send_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let t = self.transport.lock().await;
        t.send_request(method, params).await
    }

    async fn initialize(&mut self) -> Result<(), String> {
        info!("   Initializing MCP protocol...");
        self.send_request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "pii-anonymizer", "version": "0.1.0"}
            }),
        )
        .await?;

        // Send initialized notification AFTER receiving initialize response
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        let json = serde_json::to_string(&notification).map_err(|e| format!("Serialize: {}", e))?;

        match &*self.transport.lock().await {
            Transport::Stdio(conn) => {
                let mut stdin = conn.stdin.lock().await;
                stdin
                    .write_all(format!("{}\n", json).as_bytes())
                    .await
                    .map_err(|e| format!("Write: {}", e))?;
                stdin.flush().await.map_err(|e| format!("Flush: {}", e))?;
            }
            Transport::Http(conn) => {
                let mut req = conn
                    .client
                    .post(&conn.base_url)
                    .header("Content-Type", "application/json");
                for (k, v) in &conn.headers {
                    req = req.header(k, v);
                }
                req.json(&notification)
                    .send()
                    .await
                    .map_err(|e| format!("HTTP notification: {}", e))?;
            }
        }

        info!("   ✅ MCP инициализирован, notification отправлен");
        Ok(())
    }

    async fn load_tools(&mut self) -> Result<(), String> {
        let result = self
            .send_request("tools/list", serde_json::json!({}))
            .await?;

        if let Some(tools_arr) = result.get("tools").and_then(|t| t.as_array()) {
            for tool in tools_arr {
                if let (Some(name), Some(schema)) = (
                    tool.get("name").and_then(|n| n.as_str()),
                    tool.get("inputSchema"),
                ) {
                    self.tools.push(ExternalTool {
                        server_name: self.name.clone(),
                        name: name.to_string(),
                        description: tool
                            .get("description")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string()),
                        input_schema: schema.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    pub async fn call_tool(
        &self,
        tool_name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        info!("📡 {}.{} → {:?}", self.name, tool_name, args);

        let result = self
            .send_request(
                "tools/call",
                serde_json::json!({
                    "name": tool_name,
                    "arguments": args
                }),
            )
            .await?;

        info!("✅ {}.{} выполнен", self.name, tool_name);
        Ok(result)
    }
}

impl Drop for McpUpstreamConnection {
    fn drop(&mut self) {
        info!("🔌 Отключено от {}", self.name);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Менеджер внешних MCP серверов
// ═══════════════════════════════════════════════════════════════════════════
pub struct McpProxyManager {
    connections: Vec<McpUpstreamConnection>,
}

impl McpProxyManager {
    pub fn new(connections: Vec<McpUpstreamConnection>) -> Self {
        info!(
            "🌐 McpProxyManager создан с {} подключениями",
            connections.len()
        );
        for conn in &connections {
            info!("   └─ {} ({} tools)", conn.name, conn.tools.len());
        }
        Self { connections }
    }

    pub fn find_connection(&self, tool_name: &str) -> Option<&McpUpstreamConnection> {
        for conn in &self.connections {
            let prefixed = format!("{}_", conn.name);
            if tool_name.starts_with(&prefixed) {
                return Some(conn);
            }
        }
        None
    }

    pub async fn call_tool(
        &self,
        tool_name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let conn = self
            .find_connection(tool_name)
            .ok_or_else(|| format!("Tool '{}' not found in any upstream", tool_name))?;

        let prefix = format!("{}_", conn.name);
        let actual_tool = tool_name.strip_prefix(&prefix).unwrap_or(tool_name);

        info!(
            "🔄 Проксирование {} → {}.{}",
            tool_name, conn.name, actual_tool
        );
        conn.call_tool(actual_tool, args).await
    }

    pub fn get_tools(&self) -> Vec<ExternalTool> {
        let mut tools = Vec::new();
        for conn in &self.connections {
            for tool in &conn.tools {
                let mut t = tool.clone();
                t.name = format!("{}_{}", conn.name, t.name);
                t.description = Some(format!(
                    "[{}] {}",
                    conn.name,
                    t.description.as_deref().unwrap_or("")
                ));
                tools.push(t);
            }
        }
        tools
    }

    pub fn server_names(&self) -> Vec<&str> {
        self.connections.iter().map(|c| c.name.as_str()).collect()
    }
}
