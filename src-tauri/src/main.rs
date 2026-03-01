mod ai;
mod commands;
mod config;
mod pty;
mod snippets;
mod ssh;
mod terminal;

use ai::AIEngine;
use commands::AppState;
use config::AppConfig;
use pty::PtyManager;
use snippets::SnippetManager;
use ssh::SSHManager;
use terminal::SessionManager;
use std::sync::Mutex;

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

    let state = AppState {
        pty_manager: PtyManager::new(),
        ai_engine: Mutex::new(ai_engine),
        config: Mutex::new(config),
        snippet_manager: Mutex::new(snippet_manager),
        ssh_manager: Mutex::new(ssh_manager),
        session_manager: Mutex::new(session_manager),
    };

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            commands::create_session,
            commands::write_to_session,
            commands::resize_session,
            commands::close_session,
            commands::ai_translate_command,
            commands::ai_explain_command,
            commands::ai_suggest_fix,
            commands::get_config,
            commands::set_config,
            commands::list_themes,
            commands::get_theme,
            commands::list_snippets,
            commands::add_snippet,
            commands::delete_snippet,
            commands::run_snippet,
            commands::list_ssh_connections,
            commands::add_ssh_connection,
            commands::delete_ssh_connection,
            commands::connect_ssh,
            commands::search_history,
            commands::recent_history,
            commands::list_ollama_models, 
        ])
        .run(tauri::generate_context!())
        .expect("error running flux terminal");
}