mod ai;
mod commands;
mod config;
mod mcp;
mod pty;
mod snippets;
mod ssh;
mod terminal;
mod monitor;

use ai::AIEngine;
use commands::AppState;
use config::AppConfig;
use mcp::MCPManager;
use pty::PtyManager;
use snippets::SnippetManager;
use ssh::SSHManager;
use std::sync::Mutex;
use terminal::SessionManager;

fn main() {
    let config = AppConfig::load();

    let ai_engine = config
        .ai_provider
        .as_ref()
        .map(|p| AIEngine::new(p.clone()));

    let snippet_manager = SnippetManager::new();
    let ssh_manager = SSHManager::new();
    let session_manager = SessionManager::new();
    let _ = session_manager.load_history();

    let mut mcp_manager = MCPManager::new();
    // Auto-start enabled MCP servers
    mcp_manager.start_all_enabled();

    let state = AppState {
        pty_manager: PtyManager::new(),
        ai_engine: Mutex::new(ai_engine),
        config: Mutex::new(config),
        snippet_manager: Mutex::new(snippet_manager),
        ssh_manager: Mutex::new(ssh_manager),
        session_manager: Mutex::new(session_manager),
        mcp_manager: Mutex::new(mcp_manager),
    };

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            // PTY
            commands::create_session,
            commands::write_to_session,
            commands::resize_session,
            commands::close_session,
            // AI
            commands::ai_translate_command,
            commands::ai_explain_command,
            commands::ai_suggest_fix,
            commands::list_ollama_models,
            commands::list_openai_models,
            // Config & Themes
            commands::get_config,
            commands::set_config,
            commands::list_themes,
            commands::get_theme,
            // Snippets
            commands::list_snippets,
            commands::add_snippet,
            commands::delete_snippet,
            commands::run_snippet,
            commands::search_snippets,
            commands::export_snippets,
            commands::import_snippets,
            commands::get_snippet_categories,
            // SSH
            commands::list_ssh_connections,
            commands::add_ssh_connection,
            commands::delete_ssh_connection,
            commands::connect_ssh,
            commands::check_ssh_reachable,
            commands::get_known_hosts,
            // Sessions & History
            commands::list_sessions,
            commands::search_history,
            commands::recent_history,
            commands::add_history_entry,
            commands::unique_commands,
            commands::most_used_commands,
            commands::save_history,
            commands::clear_history,
            // System
            commands::list_available_shells,
            // MCP
            commands::mcp_get_config,
            commands::mcp_save_config,
            commands::mcp_add_server,
            commands::mcp_remove_server,
            commands::mcp_start_server,
            commands::mcp_stop_server,
            commands::mcp_restart_server,
            commands::mcp_list_servers,
            commands::mcp_list_tools,
            commands::mcp_call_tool,
            commands::mcp_get_ai_context,
            // MCP + AI Chat 
            commands::mcp_ai_chat,
            commands::mcp_ai_followup,
            commands::mcp_ai_step,

             // ═══ Monitor — all from monitor module ═══
            monitor::monitor_fetch_tle,
            monitor::monitor_flights,
            monitor::monitor_iss_position,
            monitor::monitor_news,
            monitor::monitor_system_stats,
            monitor::monitor_public_ip,
            monitor::monitor_activity,
        ])
        .run(tauri::generate_context!())
        .expect("error running flux terminal");
}