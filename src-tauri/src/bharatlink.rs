// ═══════════════════════════════════════════════════════════════════════════
//  FLUX BHARATLINK — Tauri Adapter Layer
//  Thin wrapper: TauriEventSink + #[tauri::command] functions
//  Core P2P logic lives in bharatlink-core crate
// ═══════════════════════════════════════════════════════════════════════════

use bharatlink_core::*;
use std::sync::Arc;
use tauri::Emitter;

// ═══ Re-exports for main.rs / commands.rs ═══════════════════════════════

pub use bharatlink_core::BharatLinkManager;

// ═══ Tauri Event Sink ═══════════════════════════════════════════════════

/// Routes BharatLinkEvents to Tauri's event system for frontend consumption
struct TauriEventSink {
    app_handle: tauri::AppHandle,
}

impl EventSink for TauriEventSink {
    fn emit(&self, event: BharatLinkEvent) {
        match &event {
            BharatLinkEvent::Error(e) => {
                let _ = self.app_handle.emit("bharatlink-error", e);
            }
            BharatLinkEvent::TransferProgress(p) => {
                let _ = self.app_handle.emit("bharatlink-transfer-progress", p);
            }
            BharatLinkEvent::TransferComplete(e) => {
                let _ = self.app_handle.emit("bharatlink-transfer-complete", e);
            }
            BharatLinkEvent::IncomingRequest(r) => {
                let _ = self.app_handle.emit("bharatlink-incoming-request", r);
            }
            BharatLinkEvent::Signal(s) => {
                let _ = self.app_handle.emit("bharatlink-signal", s);
            }
            BharatLinkEvent::NodeStatus(n) => {
                let _ = self.app_handle.emit("bharatlink-node-status", n);
            }
            BharatLinkEvent::PeerDiscovered(p) => {
                let _ = self.app_handle.emit("bharatlink-peer-discovered", p);
            }
            BharatLinkEvent::PeerReconnected { peer_id } => {
                let _ = self.app_handle.emit("bharatlink-peer-reconnected", peer_id.as_str());
            }
        }
    }

    fn notify(&self, title: &str, body: &str) {
        use tauri_plugin_notification::NotificationExt;
        let _ = self.app_handle.notification().builder().title(title).body(body).show();
    }
}

// ═══ Tauri Commands ═════════════════════════════════════════════════════

#[tauri::command]
pub async fn bharatlink_start(
    state: tauri::State<'_, crate::commands::AppState>,
    app_handle: tauri::AppHandle,
) -> Result<NodeInfo, String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    let events: Arc<dyn EventSink> = Arc::new(TauriEventSink { app_handle });
    mgr.start(events).await
}

#[tauri::command]
pub async fn bharatlink_stop(
    state: tauri::State<'_, crate::commands::AppState>,
) -> Result<(), String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.stop().await
}

#[tauri::command]
pub async fn bharatlink_node_info(
    state: tauri::State<'_, crate::commands::AppState>,
) -> Result<NodeInfo, String> {
    let mgr = state.bharatlink_manager.lock().await;
    mgr.get_node_info()
}

#[tauri::command]
pub async fn bharatlink_get_peers(
    state: tauri::State<'_, crate::commands::AppState>,
) -> Result<Vec<PeerInfo>, String> {
    let mgr = state.bharatlink_manager.lock().await;
    Ok(mgr.get_peers())
}

#[tauri::command]
pub async fn bharatlink_add_peer(
    state: tauri::State<'_, crate::commands::AppState>,
    node_id: String,
    nickname: Option<String>,
) -> Result<PeerInfo, String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.add_peer(node_id, nickname)
}

#[tauri::command]
pub async fn bharatlink_trust_peer(
    state: tauri::State<'_, crate::commands::AppState>,
    node_id: String,
    nickname: String,
) -> Result<(), String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.trust_peer(node_id, nickname).await
}

#[tauri::command]
pub async fn bharatlink_untrust_peer(
    state: tauri::State<'_, crate::commands::AppState>,
    node_id: String,
) -> Result<(), String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.untrust_peer(node_id).await
}

#[tauri::command]
pub async fn bharatlink_send_file(
    state: tauri::State<'_, crate::commands::AppState>,
    peer_id: String,
    file_path: String,
) -> Result<String, String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.send_file(peer_id, file_path).await
}

#[tauri::command]
pub async fn bharatlink_send_text(
    state: tauri::State<'_, crate::commands::AppState>,
    peer_id: String,
    text: String,
) -> Result<String, String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.send_text(peer_id, text).await
}

#[tauri::command]
pub async fn bharatlink_accept_transfer(
    state: tauri::State<'_, crate::commands::AppState>,
    request_id: String,
) -> Result<(), String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.accept_transfer(request_id).await
}

#[tauri::command]
pub async fn bharatlink_reject_transfer(
    state: tauri::State<'_, crate::commands::AppState>,
    request_id: String,
) -> Result<(), String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.reject_transfer(request_id).await
}

#[tauri::command]
pub async fn bharatlink_cancel_transfer(
    state: tauri::State<'_, crate::commands::AppState>,
    transfer_id: String,
) -> Result<(), String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.cancel_transfer(transfer_id)
}

#[tauri::command]
pub async fn bharatlink_retry_transfer(
    state: tauri::State<'_, crate::commands::AppState>,
    transfer_id: String,
) -> Result<String, String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.retry_transfer(transfer_id).await
}

#[tauri::command]
pub async fn bharatlink_get_history(
    state: tauri::State<'_, crate::commands::AppState>,
) -> Result<Vec<TransferHistoryEntry>, String> {
    let mgr = state.bharatlink_manager.lock().await;
    Ok(mgr.get_history())
}

#[tauri::command]
pub async fn bharatlink_clear_history(
    state: tauri::State<'_, crate::commands::AppState>,
) -> Result<(), String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.clear_history()
}

#[tauri::command]
pub async fn bharatlink_get_settings(
    state: tauri::State<'_, crate::commands::AppState>,
) -> Result<BharatLinkSettings, String> {
    let mgr = state.bharatlink_manager.lock().await;
    Ok(mgr.get_settings())
}

#[tauri::command]
pub async fn bharatlink_update_settings(
    state: tauri::State<'_, crate::commands::AppState>,
    settings: BharatLinkSettings,
) -> Result<(), String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.update_settings(settings).await
}

#[tauri::command]
pub async fn bharatlink_send_files(
    state: tauri::State<'_, crate::commands::AppState>,
    peer_id: String,
    file_paths: Vec<String>,
) -> Result<String, String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.send_files(peer_id, file_paths).await
}

#[tauri::command]
pub async fn bharatlink_list_dir_files(
    state: tauri::State<'_, crate::commands::AppState>,
    dir_path: String,
) -> Result<Vec<String>, String> {
    let mgr = state.bharatlink_manager.lock().await;
    mgr.list_dir_files(dir_path)
}

#[tauri::command]
pub async fn bharatlink_capture_screenshot(
    state: tauri::State<'_, crate::commands::AppState>,
    peer_id: String,
) -> Result<String, String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.capture_and_send_screenshot(peer_id).await
}

#[tauri::command]
pub async fn bharatlink_send_signal(
    state: tauri::State<'_, crate::commands::AppState>,
    peer_id: String,
    signal_type: String,
    message_id: Option<String>,
) -> Result<(), String> {
    let mgr = state.bharatlink_manager.lock().await;
    mgr.send_signal(peer_id, signal_type, message_id).await
}

#[tauri::command]
pub async fn bharatlink_send_clipboard(
    state: tauri::State<'_, crate::commands::AppState>,
    peer_id: String,
    clipboard_text: String,
) -> Result<String, String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.send_clipboard_text(peer_id, clipboard_text).await
}
