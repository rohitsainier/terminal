use crate::ai::{AIEngine, AIResponse};
use crate::config::AppConfig;
use crate::pty::PtyManager;
use crate::snippets::SnippetManager;
use crate::ssh::{SSHConnection, SSHManager};
use crate::terminal::{CommandHistoryEntry, SessionManager};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub pty_manager: PtyManager,
    pub ai_engine: Mutex<Option<AIEngine>>,
    pub config: Mutex<AppConfig>,
    pub snippet_manager: Mutex<SnippetManager>,
    pub ssh_manager: Mutex<SSHManager>,
    pub session_manager: Mutex<SessionManager>,
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
        .create_session(id, rows, cols, cwd, app_handle)
}

#[tauri::command]
pub fn write_to_session(
    state: State<AppState>,
    id: String,
    data: String,
) -> Result<(), String> {
    state.pty_manager.write(&id, data.as_bytes())
}

#[tauri::command]
pub fn resize_session(
    state: State<AppState>,
    id: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    state.pty_manager.resize(&id, rows, cols)
}

#[tauri::command]
pub fn close_session(state: State<AppState>, id: String) -> Result<(), String> {
    state.pty_manager.close(&id)
}

// ─── AI Commands ────────────────────────────────

#[tauri::command]
pub async fn ai_translate_command(
    state: State<'_, AppState>,
    query: String,
    cwd: String,
) -> Result<AIResponse, String> {
    let engine = {
        let guard = state.ai_engine.lock().map_err(|_| "Lock error")?;
        guard.clone().ok_or("AI not configured. Set up Ollama or add API key in Settings.")?
    };

    let os = std::env::consts::OS;
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());

    engine
        .translate_to_command(&query, os, &shell, &cwd)
        .await
}

#[tauri::command]
pub async fn ai_explain_command(
    state: State<'_, AppState>,
    command: String,
) -> Result<String, String> {
    let engine = {
        let guard = state.ai_engine.lock().map_err(|_| "Lock error")?;
        guard.clone().ok_or("AI not configured")?
    };

    engine.explain_command(&command).await
}

#[tauri::command]
pub async fn ai_suggest_fix(
    state: State<'_, AppState>,
    command: String,
    error_output: String,
) -> Result<AIResponse, String> {
    let engine = {
        let guard = state.ai_engine.lock().map_err(|_| "Lock error")?;
        guard.clone().ok_or("AI not configured")?
    };

    engine.suggest_fix(&command, &error_output).await
}

// ─── Config Commands ────────────────────────────

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|_| "Lock error")?;
    Ok(config.clone())
}

#[tauri::command]
pub fn set_config(
    state: State<AppState>,
    config: AppConfig,
) -> Result<(), String> {
    let mut current = state.config.lock().map_err(|_| "Lock error")?;

    let ai = match &config.ai_provider {
        Some(provider) => Some(AIEngine::new(provider.clone())),
        None => None,
    };
    *state.ai_engine.lock().map_err(|_| "Lock error")? = ai;

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

#[derive(Serialize, Deserialize, Clone)]
pub struct SnippetInput {
    pub id: String,
    pub name: String,
    pub command: String,
    pub icon: String,
    pub tags: Vec<String>,
}

#[tauri::command]
pub fn list_snippets(state: State<AppState>) -> Result<Vec<crate::snippets::Snippet>, String> {
    let manager = state.snippet_manager.lock().map_err(|_| "Lock error")?;
    Ok(manager.list())
}

#[tauri::command]
pub fn add_snippet(state: State<AppState>, snippet: crate::snippets::Snippet) -> Result<(), String> {
    let mut manager = state.snippet_manager.lock().map_err(|_| "Lock error")?;
    manager.add(snippet)
}

#[tauri::command]
pub fn delete_snippet(state: State<AppState>, id: String) -> Result<(), String> {
    let mut manager = state.snippet_manager.lock().map_err(|_| "Lock error")?;
    manager.delete(&id)
}

#[tauri::command]
pub fn run_snippet(
    state: State<AppState>,
    id: String,
    session_id: String,
) -> Result<(), String> {
    let manager = state.snippet_manager.lock().map_err(|_| "Lock error")?;
    let snippet = manager
        .get(&id)
        .ok_or("Snippet not found")?;

    let command = format!("{}\n", snippet.command);
    state.pty_manager.write(&session_id, command.as_bytes())
}

// ─── SSH Commands ───────────────────────────────

#[tauri::command]
pub fn list_ssh_connections(state: State<AppState>) -> Result<Vec<SSHConnection>, String> {
    let manager = state.ssh_manager.lock().map_err(|_| "Lock error")?;
    Ok(manager.list())
}

#[tauri::command]
pub fn add_ssh_connection(
    state: State<AppState>,
    connection: SSHConnection,
) -> Result<(), String> {
    let mut manager = state.ssh_manager.lock().map_err(|_| "Lock error")?;
    manager.add(connection)
}

#[tauri::command]
pub fn delete_ssh_connection(
    state: State<AppState>,
    id: String,
) -> Result<(), String> {
    let mut manager = state.ssh_manager.lock().map_err(|_| "Lock error")?;
    manager.delete(&id)
}

#[tauri::command]
pub fn connect_ssh(
    state: State<AppState>,
    id: String,
    session_id: String,
) -> Result<(), String> {
    let manager = state.ssh_manager.lock().map_err(|_| "Lock error")?;
    let ssh_command = manager.build_ssh_command(&id)?;

    let full_command = format!("{}\n", ssh_command);
    state
        .pty_manager
        .write(&session_id, full_command.as_bytes())
}

// ─── History Commands ───────────────────────────

#[tauri::command]
pub fn search_history(
    state: State<AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<CommandHistoryEntry>, String> {
    let manager = state.session_manager.lock().map_err(|_| "Lock error")?;
    Ok(manager.search_history(&query, limit.unwrap_or(50)))
}

#[tauri::command]
pub fn recent_history(
    state: State<AppState>,
    limit: Option<usize>,
) -> Result<Vec<CommandHistoryEntry>, String> {
    let manager = state.session_manager.lock().map_err(|_| "Lock error")?;
    Ok(manager.recent_history(limit.unwrap_or(50)))
}

#[tauri::command]
pub async fn list_ollama_models(
    base_url: Option<String>,
) -> Result<Vec<String>, String> {
    let url = base_url.unwrap_or_else(|| "http://localhost:11434".to_string());
    crate::ai::list_ollama_models(&url).await
}