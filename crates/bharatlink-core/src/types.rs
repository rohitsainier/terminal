use serde::{Deserialize, Serialize};

// ═══ Constants ══════════════════════════════════════════════════════════════

pub const BHARATLINK_TEXT_ALPN: &[u8] = b"bharatlink/text/1";
pub const BHARATLINK_META_ALPN: &[u8] = b"bharatlink/meta/1";
pub const BHARATLINK_SIGNAL_ALPN: &[u8] = b"bharatlink/signal/1";
pub const MAX_HISTORY: usize = 500;
pub const MAX_TEXT_SIZE: usize = 10 * 1024 * 1024; // 10MB text limit

// ═══ Serializable Types ════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub node_id_short: String,
    pub is_running: bool,
    pub relay_url: Option<String>,
    pub local_addrs: Vec<String>,
    pub discovered_peers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: String,
    pub node_id_short: String,
    pub nickname: Option<String>,
    pub is_local: bool,
    pub last_seen: u64,
    pub is_connected: bool,
    pub is_trusted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub id: String,
    pub from_peer: String,
    pub from_nickname: Option<String>,
    pub transfer_type: String, // "file" | "text"
    pub filename: Option<String>,
    pub file_size: Option<u64>,
    pub text_preview: Option<String>,
    pub blob_hash: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    pub transfer_id: String,
    pub direction: String, // "send" | "receive"
    pub filename: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub percent: f64,
    pub speed_bps: u64,
    pub status: String, // "connecting" | "transferring" | "complete" | "failed" | "cancelled"
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferHistoryEntry {
    pub id: String,
    pub direction: String,
    pub peer_id: String,
    pub peer_nickname: Option<String>,
    pub transfer_type: String,
    pub filename: Option<String>,
    pub file_size: Option<u64>,
    pub text_content: Option<String>,
    pub status: String,
    pub timestamp: u64,
    pub duration_ms: Option<u64>,
    pub save_path: Option<String>,
    #[serde(default)]
    pub blob_hash: Option<String>,
}

/// Error event for inline display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BharatLinkError {
    pub error_type: String, // "connection" | "transfer" | "timeout" | "system" | "reconnection"
    pub message: String,
    pub peer_id: Option<String>,
    pub transfer_id: Option<String>,
    pub timestamp: u64,
}

/// Lightweight signals: read receipts, typing indicators, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BharatLinkSignal {
    pub signal_type: String, // "delivered" | "read" | "typing" | "stop_typing"
    pub message_id: Option<String>,
    pub from_peer: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BharatLinkSettings {
    pub auto_start: bool,
    pub accept_from_trusted_only: bool,
    pub auto_accept_text: bool,
    pub auto_accept_from_trusted: bool,
    pub download_dir: String,
    pub device_name: Option<String>,
    pub max_concurrent_transfers: usize,
    #[serde(default = "default_true")]
    pub notifications_enabled: bool,
}

fn default_true() -> bool { true }

impl Default for BharatLinkSettings {
    fn default() -> Self {
        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
            .to_string_lossy()
            .to_string();
        Self {
            auto_start: false,
            accept_from_trusted_only: false,
            auto_accept_text: false,
            auto_accept_from_trusted: false,
            download_dir,
            device_name: None,
            max_concurrent_transfers: 5,
            notifications_enabled: true,
        }
    }
}
