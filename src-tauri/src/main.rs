mod ai;
mod commands;
mod config;
mod mcp;
mod pty;
mod snippets;
mod ssh;
mod terminal;
mod netops;
mod bharatlink;

use ai::AIEngine;
use commands::AppState;
use config::AppConfig;
use mcp::MCPManager;
use pty::PtyManager;
use snippets::SnippetManager;
use ssh::SSHManager;
use std::sync::{Arc, Mutex};
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

    // BharatLink P2P manager
    let bl_config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("flux-terminal");
    let bharatlink_manager = bharatlink::BharatLinkManager::new(bl_config_dir);

    let state = AppState {
        pty_manager: PtyManager::new(),
        ai_engine: Mutex::new(ai_engine),
        config: Mutex::new(config),
        snippet_manager: Mutex::new(snippet_manager),
        ssh_manager: Mutex::new(ssh_manager),
        session_manager: Mutex::new(session_manager),
        mcp_manager: Arc::new(Mutex::new(mcp_manager)),
        bharatlink_manager: Arc::new(tokio::sync::Mutex::new(bharatlink_manager)),
    };

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
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
            commands::mcp_ai_plan,
            commands::mcp_ai_step,

            // ═══ NetOps — all from netops module ═══
            netops::netops_ping,
            netops::netops_port_scan,
            netops::netops_dns_lookup,
            netops::netops_whois,
            netops::netops_wifi_scan,
            netops::netops_wifi_auth_monitor,
            netops::netops_http_headers,
            netops::netops_ssl_inspect,
            netops::netops_ip_geolocate,
            netops::netops_arp_table,
            netops::netops_subnet_calc,
            netops::netops_reverse_dns,
            netops::netops_traceroute,
            netops::netops_traffic_anomalies,
            netops::netops_rogue_ap_scan,
            netops::netops_rogue_ap_save_baseline,
            netops::netops_system_logs,
            netops::netops_threat_check,
            netops::netops_security_score,
            netops::netops_incident_list,
            netops::netops_incident_create,
            netops::netops_incident_update,
            // ═══ NetOps — Kali-style tools ═══
            netops::netops_service_scan,
            netops::netops_subdomain_enum,
            netops::netops_dir_bust,
            netops::netops_web_fingerprint,
            netops::netops_waf_detect,
            netops::netops_web_vuln_scan,
            netops::netops_hash_id,
            netops::netops_cipher_scan,
            netops::netops_handshake_analyze,
            netops::netops_save_handshake_log,
            netops::netops_pcap_analyze,
            netops::netops_psk_audit,

            // ═══ BharatLink — P2P file & text sharing ═══
            bharatlink::bharatlink_start,
            bharatlink::bharatlink_stop,
            bharatlink::bharatlink_node_info,
            bharatlink::bharatlink_get_peers,
            bharatlink::bharatlink_add_peer,
            bharatlink::bharatlink_trust_peer,
            bharatlink::bharatlink_untrust_peer,
            bharatlink::bharatlink_send_file,
            bharatlink::bharatlink_send_text,
            bharatlink::bharatlink_accept_transfer,
            bharatlink::bharatlink_reject_transfer,
            bharatlink::bharatlink_cancel_transfer,
            bharatlink::bharatlink_get_history,
            bharatlink::bharatlink_clear_history,
            bharatlink::bharatlink_get_settings,
            bharatlink::bharatlink_update_settings,
            bharatlink::bharatlink_send_files,
            bharatlink::bharatlink_list_dir_files,
            bharatlink::bharatlink_capture_screenshot,
            bharatlink::bharatlink_send_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("error running flux terminal");
}