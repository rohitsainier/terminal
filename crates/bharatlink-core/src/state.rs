use crate::events::{BharatLinkEvent, EventSink};
use crate::types::*;
use crate::util::epoch_ms;

use iroh::protocol::AcceptError;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex as TokioMutex;

// ═══ Internal transfer state ═════════════════════════════════════════════

#[allow(dead_code)]
pub(crate) struct TransferState {
    pub cancel_tx: tokio::sync::oneshot::Sender<()>,
    pub start_time: Instant,
}

// ═══ Shared state for protocol handlers ═════════════════════════════════

/// Shared state accessible by protocol handlers (which run outside the manager's mutex)
#[derive(Debug, Clone)]
pub(crate) struct SharedState {
    pub events: Arc<dyn EventSink>,
    pub trusted_peers: Arc<TokioMutex<HashMap<String, String>>>,
    pub transfer_history: Arc<TokioMutex<Vec<TransferHistoryEntry>>>,
    pub pending_requests: Arc<TokioMutex<HashMap<String, TransferRequest>>>,
    pub download_dir: Arc<TokioMutex<String>>,
    pub settings: Arc<TokioMutex<BharatLinkSettings>>,
    #[allow(dead_code)]
    pub failed_transfers: Arc<TokioMutex<Vec<TransferHistoryEntry>>>,
    pub config_dir: PathBuf,
}

impl std::fmt::Debug for dyn EventSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EventSink")
    }
}

impl SharedState {
    pub fn io_err(msg: impl Into<String>) -> AcceptError {
        AcceptError::from_err(std::io::Error::new(std::io::ErrorKind::Other, msg.into()))
    }

    /// Persist history to disk
    pub async fn save_history(&self) {
        let history = self.transfer_history.lock().await;
        let recent: Vec<_> = history.iter().rev().take(MAX_HISTORY).rev().cloned().collect();
        if let Ok(data) = serde_json::to_string_pretty(&recent) {
            let _ = std::fs::create_dir_all(&self.config_dir);
            let _ = std::fs::write(self.config_dir.join("transfer_history.json"), data);
        }
    }

    /// Emit an error event
    pub fn emit_error(&self, error_type: &str, message: &str, peer_id: Option<&str>, transfer_id: Option<&str>) {
        self.events.emit(BharatLinkEvent::Error(BharatLinkError {
            error_type: error_type.to_string(),
            message: message.to_string(),
            peer_id: peer_id.map(String::from),
            transfer_id: transfer_id.map(String::from),
            timestamp: epoch_ms(),
        }));
    }

    /// Send a native OS notification (gated by settings)
    pub fn notify(&self, title: &str, body: &str) {
        // Check if notifications are enabled (non-blocking try_lock)
        let enabled = self.settings.try_lock().map(|s| s.notifications_enabled).unwrap_or(true);
        if enabled {
            self.events.notify(title, body);
        }
    }
}
