use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

// ─── Config Types ───────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MCPConfig {
    #[serde(rename = "mcpServers", default)]
    pub servers: HashMap<String, MCPServerConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MCPServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_true() -> bool {
    true
}

// ─── Protocol Types ─────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MCPTool {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "inputSchema", default)]
    pub input_schema: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MCPToolResult {
    pub content: Vec<MCPContent>,
    #[serde(rename = "isError", default)]
    pub is_error: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MCPContent {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MCPResource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MCPServerInfo {
    pub name: String,
    pub connected: bool,
    pub tools_count: usize,
    pub tools: Vec<MCPTool>,
    pub resources: Vec<MCPResource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub config: MCPServerConfig,
}

// ─── Connection ─────────────────────────────────

struct MCPConnection {
    child: Child,
    stdin: ChildStdin,
    responses: mpsc::Receiver<String>,
    next_id: u64,
    tools: Vec<MCPTool>,
    resources: Vec<MCPResource>,
    server_name: Option<String>,
    server_version: Option<String>,
}

// ─── Manager ────────────────────────────────────

pub struct MCPManager {
    config: MCPConfig,
    connections: HashMap<String, MCPConnection>,
    config_path: PathBuf,
}

impl MCPManager {
    pub fn new() -> Self {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("flux-terminal")
            .join("mcp.json");

        let config = Self::load_config_from_path(&config_path);

        Self {
            config,
            connections: HashMap::new(),
            config_path,
        }
    }

    fn load_config_from_path(path: &PathBuf) -> MCPConfig {
        if path.exists() {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    serde_json::from_str(&content).unwrap_or_else(|e| {
                        eprintln!("[MCP] Config parse error: {}", e);
                        MCPConfig {
                            servers: HashMap::new(),
                        }
                    })
                }
                Err(e) => {
                    eprintln!("[MCP] Config read error: {}", e);
                    MCPConfig {
                        servers: HashMap::new(),
                    }
                }
            }
        } else {
            let config = MCPConfig {
                servers: HashMap::new(),
            };
            // Create default config file
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let default_json = serde_json::to_string_pretty(&config).unwrap_or_default();
            let _ = std::fs::write(path, default_json);
            config
        }
    }

    // ── Config Operations ──

    pub fn get_config(&self) -> MCPConfig {
        self.config.clone()
    }

    pub fn save_config(&self) -> Result<(), String> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir: {}", e))?;
        }
        let json = serde_json::to_string_pretty(&self.config)
            .map_err(|e| format!("Serialize error: {}", e))?;
        std::fs::write(&self.config_path, json)
            .map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }

    pub fn reload_config(&mut self) {
        self.config = Self::load_config_from_path(&self.config_path);
    }

    pub fn set_config(&mut self, config: MCPConfig) {
        self.config = config;
    }

    pub fn add_server(&mut self, name: String, config: MCPServerConfig) -> Result<(), String> {
        self.config.servers.insert(name, config);
        self.save_config()
    }

    pub fn remove_server(&mut self, name: &str) -> Result<(), String> {
        // Stop if running
        let _ = self.stop_server(name);
        self.config.servers.remove(name);
        self.save_config()
    }

    pub fn update_server(
        &mut self,
        name: &str,
        config: MCPServerConfig,
    ) -> Result<(), String> {
        let was_running = self.connections.contains_key(name);
        if was_running {
            let _ = self.stop_server(name);
        }
        self.config.servers.insert(name.to_string(), config);
        self.save_config()?;
        if was_running {
            let _ = self.start_server(name);
        }
        Ok(())
    }

    // ── Server Lifecycle ──

    pub fn start_server(&mut self, name: &str) -> Result<(), String> {
        let config = self
            .config
            .servers
            .get(name)
            .ok_or_else(|| format!("Server '{}' not found in config", name))?
            .clone();

        if self.connections.contains_key(name) {
            return Err(format!("Server '{}' is already running", name));
        }

        if !config.enabled {
            return Err(format!("Server '{}' is disabled", name));
        }

        eprintln!("[MCP] Starting server '{}': {} {:?}", name, config.command, config.args);

        // Spawn the process
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // Inherit PATH so npx/uvx work
        if let Ok(path) = std::env::var("PATH") {
            cmd.env("PATH", path);
        }
        if let Ok(home) = std::env::var("HOME") {
            cmd.env("HOME", home);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start '{}': {}. Is '{}' installed?", name, e, config.command))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to capture stdin".to_string())?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout".to_string())?;

        // Capture stderr for logging
        if let Some(stderr) = child.stderr.take() {
            let server_name = name.to_string();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().flatten() {
                    eprintln!("[MCP:{}:stderr] {}", server_name, line);
                }
            });
        }

        // Reader thread for stdout
        let (tx, rx) = mpsc::channel();
        let server_name = name.to_string();

        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        let trimmed = l.trim().to_string();
                        if !trimmed.is_empty() && trimmed.starts_with('{') {
                            eprintln!("[MCP:{}:stdout] {}", server_name, truncate_str(&trimmed, 200));
                            if tx.send(trimmed).is_err() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[MCP:{}] Read error: {}", server_name, e);
                        break;
                    }
                }
            }
            eprintln!("[MCP:{}] Reader thread exited", server_name);
        });

        let mut conn = MCPConnection {
            child,
            stdin,
            responses: rx,
            next_id: 1,
            tools: Vec::new(),
            resources: Vec::new(),
            server_name: None,
            server_version: None,
        };

        // Initialize handshake
        let init_result = Self::send_request_on(
            &mut conn,
            "initialize",
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "roots": { "listChanged": true }
                },
                "clientInfo": {
                    "name": "flux-terminal",
                    "version": "0.1.0"
                }
            }),
        );

        match init_result {
            Ok(result) => {
                conn.server_name = result["serverInfo"]["name"]
                    .as_str()
                    .map(|s| s.to_string());
                conn.server_version = result["serverInfo"]["version"]
                    .as_str()
                    .map(|s| s.to_string());

                eprintln!(
                    "[MCP] Server '{}' initialized: {:?} v{:?}",
                    name, conn.server_name, conn.server_version
                );

                // Send initialized notification
                let _ = Self::send_notification_on(
                    &mut conn,
                    "notifications/initialized",
                    serde_json::json!({}),
                );

                // Discover tools
                match Self::send_request_on(&mut conn, "tools/list", serde_json::json!({})) {
                    Ok(tools_result) => {
                        if let Some(tools) = tools_result["tools"].as_array() {
                            conn.tools = tools
                                .iter()
                                .filter_map(|t| serde_json::from_value(t.clone()).ok())
                                .collect();
                            eprintln!(
                                "[MCP] Server '{}' has {} tools",
                                name,
                                conn.tools.len()
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("[MCP] Failed to list tools for '{}': {}", name, e);
                    }
                }

                // Discover resources (optional, some servers don't support this)
                match Self::send_request_on(
                    &mut conn,
                    "resources/list",
                    serde_json::json!({}),
                ) {
                    Ok(res_result) => {
                        if let Some(resources) = res_result["resources"].as_array() {
                            conn.resources = resources
                                .iter()
                                .filter_map(|r| serde_json::from_value(r.clone()).ok())
                                .collect();
                        }
                    }
                    Err(_) => {
                        // Not all servers support resources
                    }
                }
            }
            Err(e) => {
                let _ = conn.child.kill();
                return Err(format!("MCP handshake failed for '{}': {}", name, e));
            }
        }

        self.connections.insert(name.to_string(), conn);
        eprintln!("[MCP] Server '{}' is ready", name);
        Ok(())
    }

    pub fn stop_server(&mut self, name: &str) -> Result<(), String> {
        if let Some(mut conn) = self.connections.remove(name) {
            eprintln!("[MCP] Stopping server '{}'", name);
            let _ = conn.child.kill();
            let _ = conn.child.wait();
            Ok(())
        } else {
            Err(format!("Server '{}' is not running", name))
        }
    }

    pub fn restart_server(&mut self, name: &str) -> Result<(), String> {
        let _ = self.stop_server(name);
        // Small delay to let port/resources be freed
        thread::sleep(Duration::from_millis(500));
        self.start_server(name)
    }

    pub fn start_all_enabled(&mut self) {
        let enabled_servers: Vec<String> = self
            .config
            .servers
            .iter()
            .filter(|(_, c)| c.enabled)
            .map(|(name, _)| name.clone())
            .collect();

        for name in enabled_servers {
            match self.start_server(&name) {
                Ok(_) => eprintln!("[MCP] Auto-started '{}'", name),
                Err(e) => eprintln!("[MCP] Failed to auto-start '{}': {}", name, e),
            }
        }
    }

    pub fn stop_all(&mut self) {
        let names: Vec<String> = self.connections.keys().cloned().collect();
        for name in names {
            let _ = self.stop_server(&name);
        }
    }

    // ── Tool Operations ──

    pub fn list_all_tools(&self) -> Vec<(String, MCPTool)> {
        let mut all = Vec::new();
        for (name, conn) in &self.connections {
            for tool in &conn.tools {
                all.push((name.clone(), tool.clone()));
            }
        }
        all
    }

    pub fn list_server_tools(&self, name: &str) -> Result<Vec<MCPTool>, String> {
        let conn = self
            .connections
            .get(name)
            .ok_or_else(|| format!("Server '{}' is not running", name))?;
        Ok(conn.tools.clone())
    }

    pub fn call_tool(
        &mut self,
        server_name: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<MCPToolResult, String> {
        let conn = self
            .connections
            .get_mut(server_name)
            .ok_or_else(|| format!("Server '{}' is not running. Start it first.", server_name))?;

        // Verify tool exists
        if !conn.tools.iter().any(|t| t.name == tool_name) {
            let available: Vec<&str> = conn.tools.iter().map(|t| t.name.as_str()).collect();
            return Err(format!(
                "Tool '{}' not found on server '{}'. Available: {:?}",
                tool_name, server_name, available
            ));
        }

        eprintln!(
            "[MCP] Calling tool '{}/{}' with args: {}",
            server_name,
            tool_name,
            truncate_str(&arguments.to_string(), 200)
        );

        let result = Self::send_request_on(
            conn,
            "tools/call",
            serde_json::json!({
                "name": tool_name,
                "arguments": arguments
            }),
        )?;

        let tool_result: MCPToolResult = serde_json::from_value(result.clone()).map_err(|e| {
            format!(
                "Failed to parse tool result: {} | Raw: {}",
                e,
                truncate_str(&result.to_string(), 300)
            )
        })?;

        eprintln!(
            "[MCP] Tool call result: is_error={}, content_count={}",
            tool_result.is_error,
            tool_result.content.len()
        );

        Ok(tool_result)
    }

    // ── Server Status ──

    pub fn list_server_statuses(&self) -> Vec<MCPServerInfo> {
        self.config
            .servers
            .iter()
            .map(|(name, config)| {
                if let Some(conn) = self.connections.get(name) {
                    MCPServerInfo {
                        name: name.clone(),
                        connected: true,
                        tools_count: conn.tools.len(),
                        tools: conn.tools.clone(),
                        resources: conn.resources.clone(),
                        server_name: conn.server_name.clone(),
                        server_version: conn.server_version.clone(),
                        error: None,
                        config: config.clone(),
                    }
                } else {
                    MCPServerInfo {
                        name: name.clone(),
                        connected: false,
                        tools_count: 0,
                        tools: Vec::new(),
                        resources: Vec::new(),
                        server_name: None,
                        server_version: None,
                        error: None,
                        config: config.clone(),
                    }
                }
            })
            .collect()
    }

    pub fn get_tools_for_ai_context(&self) -> String {
        let all_tools = self.list_all_tools();
        if all_tools.is_empty() {
            return String::new();
        }

        let mut ctx = String::from("\n\nAvailable MCP Tools:\n");
        for (server, tool) in &all_tools {
            ctx.push_str(&format!(
                "- {}/{}: {}\n",
                server, tool.name, tool.description
            ));
            if !tool.input_schema.is_null() {
                if let Some(props) = tool.input_schema["properties"].as_object() {
                    let params: Vec<String> = props
                        .keys()
                        .map(|k| k.clone())
                        .collect();
                    ctx.push_str(&format!("  Parameters: {}\n", params.join(", ")));
                }
            }
        }
        ctx
    }

    // ── JSON-RPC Helpers ──

    fn send_request_on(
        conn: &mut MCPConnection,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let id = conn.next_id;
        conn.next_id += 1;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        let request_str = serde_json::to_string(&request)
            .map_err(|e| format!("Serialize error: {}", e))?;

        eprintln!("[MCP] → {}", truncate_str(&request_str, 200));

        conn.stdin
            .write_all(request_str.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        conn.stdin
            .write_all(b"\n")
            .map_err(|e| format!("Write newline error: {}", e))?;
        conn.stdin
            .flush()
            .map_err(|e| format!("Flush error: {}", e))?;

        // Wait for response with matching ID
        let timeout = Duration::from_secs(30);
        let deadline = Instant::now() + timeout;

        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or_else(|| {
                    format!(
                        "Timeout after {}s waiting for MCP response to '{}'",
                        timeout.as_secs(),
                        method
                    )
                })?;

            match conn.responses.recv_timeout(remaining) {
                Ok(line) => {
                    let response: serde_json::Value =
                        serde_json::from_str(&line).map_err(|e| {
                            format!("Invalid JSON from MCP: {} | Line: {}", e, truncate_str(&line, 200))
                        })?;

                    // Check if this is our response
                    let resp_id = response
                        .get("id")
                        .and_then(|v| v.as_u64());

                    if resp_id == Some(id) {
                        if let Some(error) = response.get("error") {
                            let msg = error["message"]
                                .as_str()
                                .unwrap_or("Unknown MCP error");
                            let code = error["code"].as_i64().unwrap_or(0);
                            return Err(format!("MCP error ({}): {}", code, msg));
                        }

                        return Ok(response["result"].clone());
                    }

                    // Not our response — it's a notification or different ID, skip
                    eprintln!(
                        "[MCP] Skipped message (id={:?}, waiting for {})",
                        resp_id, id
                    );
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    return Err(format!(
                        "Timeout waiting for MCP response to '{}'. Server may be unresponsive.",
                        method
                    ));
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(
                        "MCP server disconnected. It may have crashed.".to_string()
                    );
                }
            }
        }
    }

    fn send_notification_on(
        conn: &mut MCPConnection,
        method: &str,
        params: serde_json::Value,
    ) -> Result<(), String> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        let s = serde_json::to_string(&notification)
            .map_err(|e| format!("Serialize error: {}", e))?;

        conn.stdin
            .write_all(s.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        conn.stdin
            .write_all(b"\n")
            .map_err(|e| format!("Write error: {}", e))?;
        conn.stdin
            .flush()
            .map_err(|e| format!("Flush error: {}", e))?;

        Ok(())
    }
}

impl Drop for MCPManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}