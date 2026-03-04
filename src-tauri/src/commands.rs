use crate::ai::{AIEngine, AIResponse};
use crate::config::AppConfig;
use crate::pty::PtyManager;
use crate::snippets::SnippetManager;
use crate::ssh::{SSHConnection, SSHManager};
use crate::terminal::{CommandHistoryEntry, SessionManager};
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub pty_manager: PtyManager,
    pub ai_engine: Mutex<Option<AIEngine>>,
    pub config: Mutex<AppConfig>,
    pub snippet_manager: Mutex<SnippetManager>,
    pub ssh_manager: Mutex<SSHManager>,
    pub session_manager: Mutex<SessionManager>,
    pub mcp_manager: Mutex<crate::mcp::MCPManager>,
}

// ─── Lock Helpers ────────────────────────────────

fn lock_ai_engine(state: &AppState) -> Result<AIEngine, String> {
    let guard = state
        .ai_engine
        .lock()
        .map_err(|_| "Failed to lock ai_engine".to_string())?;
    guard
        .clone()
        .ok_or_else(|| "AI not configured. Set up a provider in Settings.".to_string())
}

fn get_mcp_context(state: &AppState) -> Result<String, String> {
    let manager = state.mcp_manager.lock().map_err(|_| "Failed to lock mcp_manager")?;
    Ok(manager.get_tools_for_ai_context())
}

fn call_mcp_tool(
    state: &AppState,
    server: &str,
    tool: &str,
    arguments: serde_json::Value,
) -> Result<crate::mcp::MCPToolResult, String> {
    let mut manager = state.mcp_manager.lock().map_err(|_| "Failed to lock mcp_manager")?;
    manager.call_tool(server, tool, arguments)
}

fn format_tool_result_text(result: &crate::mcp::MCPToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.text.as_ref())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}

// ─── PTY Commands ────────────────────────────────

#[tauri::command]
pub fn create_session(
    state: State<AppState>,
    id: String,
    rows: u16,
    cols: u16,
    cwd: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    state
        .pty_manager
        .create_session(id.clone(), rows, cols, cwd.clone(), app_handle)?;

    // Register in session manager for tracking
    let sm = state.session_manager.lock().map_err(|_| "Failed to lock session_manager")?;
    let _ = sm.register_session(&id, None);
    if let Some(c) = cwd {
        let _ = sm.update_session(&id, None, Some(c), None);
    }
    Ok(())
}

#[tauri::command]
pub fn write_to_session(state: State<AppState>, id: String, data: String) -> Result<(), String> {
    state.pty_manager.write(&id, data.as_bytes())
}

#[tauri::command]
pub fn resize_session(state: State<AppState>, id: String, rows: u16, cols: u16) -> Result<(), String> {
    state.pty_manager.resize(&id, rows, cols)
}

#[tauri::command]
pub fn close_session(state: State<AppState>, id: String) -> Result<(), String> {
    state.pty_manager.close(&id)?;
    let sm = state.session_manager.lock().map_err(|_| "Failed to lock session_manager")?;
    let _ = sm.close_session(&id, None);
    let _ = sm.save_history();
    Ok(())
}

// ─── AI Commands ────────────────────────────────

#[tauri::command]
pub async fn ai_translate_command(
    state: State<'_, AppState>,
    query: String,
    cwd: String,
) -> Result<AIResponse, String> {
    let engine = lock_ai_engine(&state)?;
    let os = std::env::consts::OS;
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
    engine.translate_to_command(&query, os, &shell, &cwd).await
}

#[tauri::command]
pub async fn ai_explain_command(
    state: State<'_, AppState>,
    command: String,
) -> Result<String, String> {
    let engine = lock_ai_engine(&state)?;
    engine.explain_command(&command).await
}

#[tauri::command]
pub async fn ai_suggest_fix(
    state: State<'_, AppState>,
    command: String,
    error_output: String,
) -> Result<AIResponse, String> {
    let engine = lock_ai_engine(&state)?;
    engine.suggest_fix(&command, &error_output).await
}

#[tauri::command]
pub async fn list_ollama_models(base_url: Option<String>) -> Result<Vec<String>, String> {
    let url = base_url.unwrap_or_else(|| "http://localhost:11434".to_string());
    crate::ai::list_ollama_models(&url).await
}

#[tauri::command]
pub async fn list_openai_models(api_key: String) -> Result<Vec<String>, String> {
    crate::ai::list_openai_models(&api_key).await
}

// ─── Config Commands ────────────────────────────

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|_| "Failed to lock config")?;
    Ok(config.clone())
}

#[tauri::command]
pub fn set_config(state: State<AppState>, config: AppConfig) -> Result<(), String> {
    let mut current = state.config.lock().map_err(|_| "Failed to lock config")?;
    let ai = config.ai_provider.as_ref().map(|p| AIEngine::new(p.clone()));
    *state.ai_engine.lock().map_err(|_| "Failed to lock ai_engine")? = ai;
    *current = config;
    current.save()?;
    Ok(())
}

#[tauri::command]
pub fn list_themes() -> Result<Vec<String>, String> {
    Ok(vec![
        "hacker-green".into(),
        "cyberpunk".into(),
        "matrix".into(),
        "ghost-protocol".into(),
        "tron".into(),
        "midnight".into(),
    ])
}

#[tauri::command]
pub fn get_theme(name: String) -> Result<serde_json::Value, String> {
    crate::config::load_theme(&name)
}

// ─── Snippet Commands ───────────────────────────

#[tauri::command]
pub fn list_snippets(state: State<AppState>) -> Result<Vec<crate::snippets::Snippet>, String> {
    let manager = state.snippet_manager.lock().map_err(|_| "Failed to lock snippet_manager")?;
    Ok(manager.list())
}

#[tauri::command]
pub fn add_snippet(state: State<AppState>, snippet: crate::snippets::Snippet) -> Result<(), String> {
    let mut manager = state.snippet_manager.lock().map_err(|_| "Failed to lock snippet_manager")?;
    manager.add(snippet)
}

#[tauri::command]
pub fn delete_snippet(state: State<AppState>, id: String) -> Result<(), String> {
    let mut manager = state.snippet_manager.lock().map_err(|_| "Failed to lock snippet_manager")?;
    manager.delete(&id)
}

#[tauri::command]
pub fn run_snippet(state: State<AppState>, id: String, session_id: String) -> Result<(), String> {
    let manager = state.snippet_manager.lock().map_err(|_| "Failed to lock snippet_manager")?;
    let snippet = manager.get(&id).ok_or("Snippet not found")?;
    let command = format!("{}\n", snippet.command);
    state.pty_manager.write(&session_id, command.as_bytes())
}

#[tauri::command]
pub fn search_snippets(state: State<AppState>, query: String) -> Result<Vec<crate::snippets::Snippet>, String> {
    let manager = state.snippet_manager.lock().map_err(|_| "Failed to lock snippet_manager")?;
    Ok(manager.search(&query))
}

#[tauri::command]
pub fn get_snippet_categories(state: State<AppState>) -> Result<Vec<String>, String> {
    let manager = state.snippet_manager.lock().map_err(|_| "Failed to lock snippet_manager")?;
    Ok(manager.get_categories())
}

#[tauri::command]
pub fn import_snippets(state: State<AppState>, json_str: String) -> Result<usize, String> {
    let mut manager = state.snippet_manager.lock().map_err(|_| "Failed to lock snippet_manager")?;
    manager.import_from_json(&json_str)
}

#[tauri::command]
pub fn export_snippets(state: State<AppState>) -> Result<String, String> {
    let manager = state.snippet_manager.lock().map_err(|_| "Failed to lock snippet_manager")?;
    manager.export_to_json()
}

// ─── SSH Commands ───────────────────────────────

#[tauri::command]
pub fn list_ssh_connections(state: State<AppState>) -> Result<Vec<SSHConnection>, String> {
    let manager = state.ssh_manager.lock().map_err(|_| "Failed to lock ssh_manager")?;
    Ok(manager.list())
}

#[tauri::command]
pub fn add_ssh_connection(state: State<AppState>, connection: SSHConnection) -> Result<(), String> {
    let mut manager = state.ssh_manager.lock().map_err(|_| "Failed to lock ssh_manager")?;
    manager.add(connection)
}

#[tauri::command]
pub fn delete_ssh_connection(state: State<AppState>, id: String) -> Result<(), String> {
    let mut manager = state.ssh_manager.lock().map_err(|_| "Failed to lock ssh_manager")?;
    manager.delete(&id)
}

#[tauri::command]
pub fn connect_ssh(state: State<AppState>, id: String, session_id: String) -> Result<(), String> {
    let manager = state.ssh_manager.lock().map_err(|_| "Failed to lock ssh_manager")?;
    let ssh_command = manager.build_ssh_command(&id)?;
    let full_command = format!("{}\n", ssh_command);
    state.pty_manager.write(&session_id, full_command.as_bytes())
}

#[tauri::command]
pub async fn check_ssh_reachable(host: String, port: u16) -> bool {
    tokio::task::spawn_blocking(move || crate::ssh::check_host_reachable(&host, port, 3000))
        .await
        .unwrap_or(false)
}

#[tauri::command]
pub fn get_known_hosts() -> Result<Vec<String>, String> {
    Ok(crate::ssh::get_known_hosts())
}

// ─── Session / History Commands ─────────────────

#[tauri::command]
pub fn list_sessions(
    state: State<AppState>,
) -> Result<Vec<crate::terminal::SessionInfo>, String> {
    let manager = state.session_manager.lock().map_err(|_| "Failed to lock session_manager")?;
    Ok(manager.list_sessions())
}

#[tauri::command]
pub fn save_history(state: State<AppState>) -> Result<(), String> {
    let manager = state.session_manager.lock().map_err(|_| "Failed to lock session_manager")?;
    manager.save_history()
}

#[tauri::command]
pub fn clear_history(state: State<AppState>) -> Result<(), String> {
    let manager = state.session_manager.lock().map_err(|_| "Failed to lock session_manager")?;
    manager.clear_history();
    manager.save_history()
}


#[tauri::command]
pub fn add_history_entry(state: State<AppState>, entry: CommandHistoryEntry) -> Result<(), String> {
    let sm = state.session_manager.lock().map_err(|_| "Failed to lock session_manager")?;
    sm.add_to_history(entry);
    Ok(())
}

#[tauri::command]
pub fn search_history(
    state: State<AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<CommandHistoryEntry>, String> {
    let sm = state.session_manager.lock().map_err(|_| "Failed to lock session_manager")?;
    Ok(sm.search_history(&query, limit.unwrap_or(50)))
}

#[tauri::command]
pub fn recent_history(
    state: State<AppState>,
    limit: Option<usize>,
) -> Result<Vec<CommandHistoryEntry>, String> {
    let sm = state.session_manager.lock().map_err(|_| "Failed to lock session_manager")?;
    Ok(sm.recent_history(limit.unwrap_or(50)))
}

#[tauri::command]
pub fn unique_commands(
    state: State<AppState>,
    limit: Option<usize>,
) -> Result<Vec<String>, String> {
    let sm = state.session_manager.lock().map_err(|_| "Failed to lock session_manager")?;
    Ok(sm.unique_commands(limit.unwrap_or(50)))
}

#[derive(Serialize)]
pub struct CommandFrequency {
    pub command: String,
    pub count: usize,
}

#[tauri::command]
pub fn most_used_commands(
    state: State<AppState>,
    limit: Option<usize>,
) -> Result<Vec<CommandFrequency>, String> {
    let sm = state.session_manager.lock().map_err(|_| "Failed to lock session_manager")?;
    Ok(sm
        .most_used_commands(limit.unwrap_or(20))
        .into_iter()
        .map(|(command, count)| CommandFrequency { command, count })
        .collect())
}

// ─── Terminal Utility Commands ──────────────────

#[tauri::command]
pub fn list_available_shells() -> Result<Vec<String>, String> {
    Ok(crate::terminal::list_available_shells())
}

// ─── MCP Commands ───────────────────────────────

#[tauri::command]
pub fn mcp_get_config(state: State<AppState>) -> Result<crate::mcp::MCPConfig, String> {
    let manager = state.mcp_manager.lock().map_err(|_| "Failed to lock mcp_manager")?;
    Ok(manager.get_config())
}

#[tauri::command]
pub fn mcp_save_config(
    state: State<AppState>,
    config: crate::mcp::MCPConfig,
) -> Result<(), String> {
    let mut manager = state.mcp_manager.lock().map_err(|_| "Failed to lock mcp_manager")?;
    manager.set_config(config);
    manager.save_config()
}

#[tauri::command]
pub fn mcp_add_server(
    state: State<AppState>,
    name: String,
    config: crate::mcp::MCPServerConfig,
) -> Result<(), String> {
    let mut manager = state.mcp_manager.lock().map_err(|_| "Failed to lock mcp_manager")?;
    manager.add_server(name, config)
}

#[tauri::command]
pub fn mcp_remove_server(state: State<AppState>, name: String) -> Result<(), String> {
    let mut manager = state.mcp_manager.lock().map_err(|_| "Failed to lock mcp_manager")?;
    manager.remove_server(&name)
}

#[tauri::command]
pub fn mcp_start_server(state: State<AppState>, name: String) -> Result<(), String> {
    let mut manager = state.mcp_manager.lock().map_err(|_| "Failed to lock mcp_manager")?;
    manager.start_server(&name)
}

#[tauri::command]
pub fn mcp_stop_server(state: State<AppState>, name: String) -> Result<(), String> {
    let mut manager = state.mcp_manager.lock().map_err(|_| "Failed to lock mcp_manager")?;
    manager.stop_server(&name)
}

#[tauri::command]
pub fn mcp_restart_server(state: State<AppState>, name: String) -> Result<(), String> {
    let mut manager = state.mcp_manager.lock().map_err(|_| "Failed to lock mcp_manager")?;
    manager.restart_server(&name)
}

#[tauri::command]
pub fn mcp_list_servers(
    state: State<AppState>,
) -> Result<Vec<crate::mcp::MCPServerInfo>, String> {
    let manager = state.mcp_manager.lock().map_err(|_| "Failed to lock mcp_manager")?;
    Ok(manager.list_server_statuses())
}

#[tauri::command]
pub fn mcp_list_tools(
    state: State<AppState>,
    server: Option<String>,
) -> Result<Vec<(String, crate::mcp::MCPTool)>, String> {
    let manager = state.mcp_manager.lock().map_err(|_| "Failed to lock mcp_manager")?;

    if let Some(name) = server {
        let tools = manager.list_server_tools(&name)?;
        Ok(tools.into_iter().map(|t| (name.clone(), t)).collect())
    } else {
        Ok(manager.list_all_tools())
    }
}

#[tauri::command]
pub fn mcp_call_tool(
    state: State<AppState>,
    server: String,
    tool: String,
    arguments: serde_json::Value,
) -> Result<crate::mcp::MCPToolResult, String> {
    let mut manager = state.mcp_manager.lock().map_err(|_| "Failed to lock mcp_manager")?;
    manager.call_tool(&server, &tool, arguments)
}

#[tauri::command]
pub fn mcp_get_ai_context(state: State<AppState>) -> Result<String, String> {
    let manager = state.mcp_manager.lock().map_err(|_| "Failed to lock mcp_manager")?;
    Ok(manager.get_tools_for_ai_context())
}

// ─── MCP + AI Chat ──────────────────────────────

#[tauri::command]
pub async fn mcp_ai_chat(
    state: State<'_, AppState>,
    messages: Vec<crate::ai::ChatMessage>,
) -> Result<serde_json::Value, String> {
    let engine = lock_ai_engine(&state)?;

    let mcp_context = get_mcp_context(&state)?;

    let os = std::env::consts::OS;
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());

    let response = engine
        .chat_with_tools(&messages, &mcp_context, os, &shell, None)
        .await?;

    match response {
        crate::ai::ChatResponse::Message(msg) => {
            Ok(serde_json::json!({
                "type": "message",
                "content": msg
            }))
        }
        crate::ai::ChatResponse::ToolCall {
            server,
            tool,
            arguments,
        } => {
            // Try to execute the tool
            let tool_result = call_mcp_tool(&state, &server, &tool, arguments.clone());

            match tool_result {
                Ok(result) => {
                    let result_text = format_tool_result_text(&result);

                    Ok(serde_json::json!({
                        "type": "tool_call",
                        "server": server,
                        "tool": tool,
                        "arguments": arguments,
                        "result": result_text,
                        "is_error": result.is_error
                    }))
                }
                Err(e) => {
                    // ── AUTO-RETRY: Ask AI again with the error context ──
                    eprintln!("[MCP] Tool call failed: {}. Retrying with AI...", e);

                    // Build retry messages
                    let mut retry_messages = messages.clone();
                    retry_messages.push(crate::ai::ChatMessage {
                        role: "tool_result".to_string(),
                        content: format!(
                            "ERROR: {}.\n\n\
                             IMPORTANT: Use the EXACT tool name from the available list. \
                             Do NOT prefix with server name. \
                             For example, use \"create_frame\" not \"figma/create_frame\".\n\n\
                             Try again with the correct tool name, or if no suitable tool exists, \
                             respond with a message explaining what the user should do.",
                            e
                        ),
                    });

                    // Retry with AI
                    match engine
                        .chat_with_tools(&retry_messages, &mcp_context, os, &shell, None)
                        .await
                    {
                        Ok(retry_response) => match retry_response {
                            crate::ai::ChatResponse::ToolCall {
                                server: retry_server,
                                tool: retry_tool,
                                arguments: retry_args,
                            } => {
                                // Second attempt
                                let retry_result = call_mcp_tool(&state, &retry_server, &retry_tool, retry_args.clone());

                                match retry_result {
                                    Ok(result) => {
                                        let result_text = format_tool_result_text(&result);

                                        Ok(serde_json::json!({
                                            "type": "tool_call",
                                            "server": retry_server,
                                            "tool": retry_tool,
                                            "arguments": retry_args,
                                            "result": result_text,
                                            "is_error": result.is_error,
                                            "retried": true
                                        }))
                                    }
                                    Err(retry_err) => {
                                        // Both attempts failed — return error with available tools
                                        let available = get_mcp_context(&state)?;

                                        Ok(serde_json::json!({
                                            "type": "tool_error",
                                            "original_tool": tool,
                                            "retry_tool": retry_tool,
                                            "error": retry_err,
                                            "available_tools": available,
                                            "server": server
                                        }))
                                    }
                                }
                            }
                            crate::ai::ChatResponse::Message(msg) => {
                                // AI decided to respond with a message instead of retrying
                                Ok(serde_json::json!({
                                    "type": "message",
                                    "content": msg,
                                    "had_tool_error": true
                                }))
                            }
                        },
                        Err(retry_err) => {
                            // Retry AI call itself failed
                            Ok(serde_json::json!({
                                "type": "tool_error",
                                "original_tool": tool,
                                "error": format!("Original: {} | Retry: {}", e, retry_err),
                                "server": server
                            }))
                        }
                    }
                }
            }
        }
    }
}

#[tauri::command]
pub async fn mcp_ai_followup(
    state: State<'_, AppState>,
    messages: Vec<crate::ai::ChatMessage>,
    tool_result: String,
) -> Result<String, String> {
    let engine = lock_ai_engine(&state)?;

    let mcp_context = get_mcp_context(&state)?;

    // Add tool result to conversation and ask AI to summarize
    let mut all_messages = messages;
    all_messages.push(crate::ai::ChatMessage {
        role: "tool_result".to_string(),
        content: tool_result,
    });
    all_messages.push(crate::ai::ChatMessage {
        role: "user".to_string(),
        content: "Based on the tool result above, provide a clear summary. Respond with: {\"message\": \"your summary\"}".to_string(),
    });

    let os = std::env::consts::OS;
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());

    let response = engine
        .chat_with_tools(&all_messages, &mcp_context, os, &shell, None)
        .await?;

    match response {
        crate::ai::ChatResponse::Message(msg) => Ok(msg),
        crate::ai::ChatResponse::ToolCall { .. } => {
            // If AI wants another tool call, just return the raw result
            Ok("Tool call completed. See the result above.".to_string())
        }
    }
}

/// Generate a task execution plan from user's request.
/// Returns structured plan with title and steps.
#[tauri::command]
pub async fn mcp_ai_plan(
    state: State<'_, AppState>,
    messages: Vec<crate::ai::ChatMessage>,
) -> Result<serde_json::Value, String> {
    let engine = lock_ai_engine(&state)?;
    let mcp_context = get_mcp_context(&state)?;
    let os = std::env::consts::OS;
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());

    engine.plan_task(&messages, &mcp_context, os, &shell).await
}

/// One step of the agentic loop: AI either responds or calls a tool.
/// If tool_call, backend auto-executes it and returns the result.
/// Frontend loops until it gets type="message".
/// Optional plan_step provides context about which plan step is being executed.
#[tauri::command]
pub async fn mcp_ai_step(
    state: State<'_, AppState>,
    messages: Vec<crate::ai::ChatMessage>,
    plan_step: Option<String>,
) -> Result<serde_json::Value, String> {
    let engine = lock_ai_engine(&state)?;

    let mcp_context = get_mcp_context(&state)?;

    let os = std::env::consts::OS;
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());

    let response = engine
        .chat_with_tools(&messages, &mcp_context, os, &shell, plan_step.as_deref())
        .await?;

    match response {
        crate::ai::ChatResponse::Message(msg) => Ok(serde_json::json!({
            "type": "message",
            "content": msg
        })),
        crate::ai::ChatResponse::ToolCall { server, tool, arguments } => {
            let tool_result = call_mcp_tool(&state, &server, &tool, arguments.clone());

            match tool_result {
                Ok(result) => {
                    let text = format_tool_result_text(&result);

                    Ok(serde_json::json!({
                        "type": "tool_call",
                        "server": server,
                        "tool": tool,
                        "arguments": arguments,
                        "result": text,
                        "is_error": result.is_error
                    }))
                }
                Err(e) => Ok(serde_json::json!({
                    "type": "tool_error",
                    "server": server,
                    "tool": tool,
                    "arguments": arguments,
                    "error": e
                })),
            }
        }
    }
}