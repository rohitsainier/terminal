// ═══════════════════════════════════════════════════════════════════════════
//  FLUX BHARATLINK — P2P File & Text Sharing (iroh-powered)
//  Sovereign peer-to-peer sharing: no servers, no accounts, pure QUIC+mDNS
// ═══════════════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::Emitter;
use tokio::sync::Mutex as TokioMutex;

use iroh::{
    discovery::mdns::MdnsDiscovery,
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler, Router},
    Endpoint, PublicKey, SecretKey,
};
use iroh_blobs::{api::remote::GetProgressItem, store::fs::FsStore, BlobsProtocol};
use futures_lite::StreamExt;

// ═══ Constants ══════════════════════════════════════════════════════════════

const BHARATLINK_TEXT_ALPN: &[u8] = b"bharatlink/text/1";
const BHARATLINK_META_ALPN: &[u8] = b"bharatlink/meta/1";
const BHARATLINK_SIGNAL_ALPN: &[u8] = b"bharatlink/signal/1";
const MAX_HISTORY: usize = 500;
const MAX_TEXT_SIZE: usize = 10 * 1024 * 1024; // 10MB text limit

// ═══ Serializable Types (sent to frontend via Tauri IPC) ════════════════

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

/// Error event emitted to frontend for inline chat display
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

// ═══ Internal transfer state ═════════════════════════════════════════════

#[allow(dead_code)]
struct TransferState {
    cancel_tx: tokio::sync::oneshot::Sender<()>,
    start_time: Instant,
}

// ═══ Shared state for protocol handlers ═════════════════════════════════

/// Shared state accessible by protocol handlers (which run outside the manager's mutex)
#[derive(Debug, Clone)]
struct SharedState {
    app_handle: tauri::AppHandle,
    trusted_peers: Arc<TokioMutex<HashMap<String, String>>>,
    transfer_history: Arc<TokioMutex<Vec<TransferHistoryEntry>>>,
    pending_requests: Arc<TokioMutex<HashMap<String, TransferRequest>>>,
    download_dir: Arc<TokioMutex<String>>,
    settings: Arc<TokioMutex<BharatLinkSettings>>,
    #[allow(dead_code)]
    failed_transfers: Arc<TokioMutex<Vec<TransferHistoryEntry>>>,
    config_dir: PathBuf,
}

impl SharedState {
    fn io_err(msg: impl Into<String>) -> AcceptError {
        AcceptError::from_err(std::io::Error::new(std::io::ErrorKind::Other, msg.into()))
    }

    /// Persist history to disk
    async fn save_history(&self) {
        let history = self.transfer_history.lock().await;
        let recent: Vec<_> = history.iter().rev().take(MAX_HISTORY).rev().cloned().collect();
        if let Ok(data) = serde_json::to_string_pretty(&recent) {
            let _ = std::fs::create_dir_all(&self.config_dir);
            let _ = std::fs::write(self.config_dir.join("transfer_history.json"), data);
        }
    }
}

// ═══ Helpers ═════════════════════════════════════════════════════════════

/// Emit a BharatLinkError event to the frontend for inline chat display
fn emit_error(app: &tauri::AppHandle, error_type: &str, message: &str, peer_id: Option<&str>, transfer_id: Option<&str>) {
    let _ = app.emit("bharatlink-error", &BharatLinkError {
        error_type: error_type.to_string(),
        message: message.to_string(),
        peer_id: peer_id.map(String::from),
        transfer_id: transfer_id.map(String::from),
        timestamp: epoch_ms(),
    });
}

/// Send a native OS notification (gated by settings)
fn send_notification(app: &tauri::AppHandle, settings: &Arc<TokioMutex<BharatLinkSettings>>, title: &str, body: &str) {
    use tauri_plugin_notification::NotificationExt;
    // Check if notifications are enabled (non-blocking try_lock)
    let enabled = settings.try_lock().map(|s| s.notifications_enabled).unwrap_or(true);
    if enabled {
        let _ = app.notification().builder().title(title).body(body).show();
    }
}

// ═══ Protocol Handlers (registered with Router) ══════════════════════════

/// Handles incoming META ALPN connections (file transfer requests)
#[derive(Debug, Clone)]
struct MetaProtocolHandler {
    shared: SharedState,
    settings: Arc<TokioMutex<BharatLinkSettings>>,
    store: FsStore,
    endpoint_for_accept: Endpoint,
}

impl ProtocolHandler for MetaProtocolHandler {
    async fn accept(&self, conn: Connection) -> Result<(), AcceptError> {
        let (_send, mut recv) = conn
            .accept_bi()
            .await
            .map_err(|e| SharedState::io_err(format!("Stream error: {}", e)))?;

        // Read full message (may arrive in multiple chunks)
        let mut data = Vec::new();
        let mut buf = vec![0u8; 4096];
        loop {
            match recv.read(&mut buf).await {
                Ok(Some(n)) => {
                    data.extend_from_slice(&buf[..n]);
                    if data.len() > 256 * 1024 {
                        return Err(SharedState::io_err("Meta message too large"));
                    }
                }
                Ok(None) => break, // EOF
                Err(e) => {
                    return Err(SharedState::io_err(format!("Read error: {:?}", e)));
                }
            }
        }

        if !data.is_empty() {
            let msg: TransferRequest = serde_json::from_slice(&data)
                .map_err(|e| SharedState::io_err(format!("Parse error: {}", e)))?;

            eprintln!("[BharatLink] Incoming transfer request: type={}, id={}, from={}",
                msg.transfer_type, msg.id, msg.from_peer);

            // Check auto-accept settings
            let settings = self.settings.lock().await;
            let is_trusted = {
                let trusted = self.shared.trusted_peers.lock().await;
                trusted.contains_key(&msg.from_peer)
            };

            // Skip if accept_from_trusted_only and sender is not trusted
            if settings.accept_from_trusted_only && !is_trusted {
                eprintln!("[BharatLink] Rejecting request from untrusted peer: {}", msg.from_peer);
                return Ok(());
            }

            let should_auto_accept = settings.auto_accept_from_trusted && is_trusted;
            drop(settings);

            if should_auto_accept {
                eprintln!("[BharatLink] Auto-accepting transfer from trusted peer: {}", msg.from_peer);

                if msg.transfer_type == "file" {
                    if let (Some(hash), Some(filename)) = (&msg.blob_hash, &msg.filename) {
                        let receiver = FileReceiveHandler {
                            shared: self.shared.clone(),
                            store: self.store.clone(),
                            endpoint: self.endpoint_for_accept.clone(),
                        };
                        let hash = hash.clone();
                        let filename = filename.clone();
                        let file_size = msg.file_size.unwrap_or(0);
                        let from_peer = msg.from_peer.clone();
                        let from_nickname = msg.from_nickname.clone();
                        let request_id = msg.id.clone();
                        let app_handle = self.shared.app_handle.clone();

                        tokio::spawn(async move {
                            // Small delay to let META connection close cleanly
                            // before opening a new blob fetch connection
                            tokio::time::sleep(std::time::Duration::from_millis(300)).await;

                            if let Err(e) = receiver.download_blob(
                                &hash, &filename, file_size,
                                &from_peer, from_nickname.clone(), &request_id,
                            ).await {
                                eprintln!("[BharatLink] Auto-accept download failed: {}", e);
                                emit_error(&app_handle, "transfer", &format!("Auto-accept download failed: {}", e),
                                    Some(&from_peer), Some(&request_id));
                                let fail_entry = TransferHistoryEntry {
                                    id: request_id, direction: "receive".to_string(),
                                    peer_id: from_peer, peer_nickname: from_nickname,
                                    transfer_type: "file".to_string(),
                                    filename: Some(filename.clone()), file_size: None,
                                    text_content: None, status: "failed".to_string(),
                                    timestamp: epoch_ms(), duration_ms: None,
                                    save_path: Some(format!("Error: {}", e)),
                                    blob_hash: Some(hash),
                                };
                                // Persist to shared history so retry_transfer can find it
                                {
                                    let mut history = receiver.shared.transfer_history.lock().await;
                                    history.push(fail_entry.clone());
                                }
                                receiver.shared.save_history().await;
                                let _ = app_handle.emit("bharatlink-transfer-complete", &fail_entry);
                            }
                        });
                    }
                }
            } else {
                // Store pending request so accept_transfer can find it
                {
                    let mut pending = self.shared.pending_requests.lock().await;
                    pending.insert(msg.id.clone(), msg.clone());
                }
                let _ = self.shared.app_handle.emit("bharatlink-incoming-request", &msg);

                // Notification for incoming file request
                let notif_body = format!("{} wants to send you {}",
                    msg.from_nickname.as_deref().unwrap_or("Unknown peer"),
                    msg.filename.as_deref().unwrap_or("a file"));
                send_notification(&self.shared.app_handle, &self.shared.settings, "BharatLink: Incoming File", &notif_body);
            }
        }

        Ok(())
    }
}

/// Handles incoming TEXT ALPN connections (direct text sharing)
#[derive(Debug, Clone)]
struct TextProtocolHandler {
    shared: SharedState,
    endpoint_for_signal: Endpoint,
}

impl ProtocolHandler for TextProtocolHandler {
    async fn accept(&self, conn: Connection) -> Result<(), AcceptError> {
        let remote_id = conn.remote_id();
        let remote_str = remote_id.to_string();

        let mut recv = conn
            .accept_uni()
            .await
            .map_err(|e| SharedState::io_err(format!("Stream error: {}", e)))?;

        let mut data = Vec::new();
        let mut buf = vec![0u8; 4096];
        loop {
            match recv.read(&mut buf).await {
                Ok(Some(n)) => {
                    data.extend_from_slice(&buf[..n]);
                    if data.len() > MAX_TEXT_SIZE {
                        return Err(SharedState::io_err("Text too large"));
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    return Err(SharedState::io_err(format!("Read error: {:?}", e)));
                }
            }
        }

        let raw_text = String::from_utf8_lossy(&data).to_string();
        let trusted = self.shared.trusted_peers.lock().await;
        let nickname = trusted.get(&remote_str).cloned();
        drop(trusted);

        // Detect clipboard content (prefixed with [CLIPBOARD])
        let (text, transfer_type) = if raw_text.starts_with("[CLIPBOARD]") {
            (raw_text.trim_start_matches("[CLIPBOARD]").to_string(), "clipboard".to_string())
        } else {
            (raw_text, "text".to_string())
        };

        let entry = TransferHistoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            direction: "receive".to_string(),
            peer_id: remote_str,
            peer_nickname: nickname,
            transfer_type: transfer_type.clone(),
            filename: None,
            file_size: Some(data.len() as u64),
            text_content: Some(text.clone()),
            status: "complete".to_string(),
            timestamp: epoch_ms(),
            duration_ms: Some(0),
            save_path: None,
            blob_hash: None,
        };

        // Persist to shared history
        {
            let mut history = self.shared.transfer_history.lock().await;
            history.push(entry.clone());
        }
        self.shared.save_history().await;

        // Emit to frontend
        let _ = self.shared.app_handle.emit("bharatlink-transfer-complete", &entry);

        // Send notification for received text
        let preview = if text.len() > 80 { &text[..80] } else { &text };
        send_notification(&self.shared.app_handle, &self.shared.settings, "BharatLink: New Message", preview);

        // Send "delivered" signal back to sender (fire-and-forget)
        let ep = self.endpoint_for_signal.clone();
        let remote_key = remote_id;
        let msg_id = entry.id.clone();
        let our_id = ep.id().to_string();
        tokio::spawn(async move {
            if let Ok(conn) = ep.connect(remote_key, BHARATLINK_SIGNAL_ALPN).await {
                let signal = BharatLinkSignal {
                    signal_type: "delivered".to_string(),
                    message_id: Some(msg_id),
                    from_peer: our_id,
                    timestamp: epoch_ms(),
                };
                if let Ok(data) = serde_json::to_vec(&signal) {
                    if let Ok(mut send) = conn.open_uni().await {
                        let _ = send.write_all(&data).await;
                        let _ = send.finish();
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    }
                }
            }
        });

        Ok(())
    }
}

/// Handles incoming signal connections (read receipts, typing indicators)
#[derive(Debug, Clone)]
struct SignalProtocolHandler {
    shared: SharedState,
}

impl ProtocolHandler for SignalProtocolHandler {
    async fn accept(&self, conn: Connection) -> Result<(), AcceptError> {
        let mut recv = conn
            .accept_uni()
            .await
            .map_err(|e| SharedState::io_err(format!("Stream error: {}", e)))?;

        let mut data = Vec::new();
        let mut buf = vec![0u8; 1024];
        loop {
            match recv.read(&mut buf).await {
                Ok(Some(n)) => {
                    data.extend_from_slice(&buf[..n]);
                    if data.len() > 4096 {
                        return Err(SharedState::io_err("Signal too large"));
                    }
                }
                Ok(None) => break,
                Err(e) => return Err(SharedState::io_err(format!("Read error: {:?}", e))),
            }
        }

        if !data.is_empty() {
            if let Ok(signal) = serde_json::from_slice::<BharatLinkSignal>(&data) {
                let _ = self.shared.app_handle.emit("bharatlink-signal", &signal);
            }
        }

        Ok(())
    }
}

/// Handles incoming file downloads via iroh-blobs (triggered after META request accepted)
#[derive(Debug, Clone)]
struct FileReceiveHandler {
    shared: SharedState,
    store: FsStore,
    endpoint: Endpoint,
}

impl FileReceiveHandler {
    /// Download a blob from a remote peer and save to disk, with progress events
    async fn download_blob(
        &self,
        hash_str: &str,
        filename: &str,
        file_size: u64,
        from_peer: &str,
        from_nickname: Option<String>,
        request_id: &str,
    ) -> Result<(), String> {
        eprintln!("[BharatLink] Starting file download: hash={}, file={}, size={}, from={}",
            hash_str, filename, file_size, from_peer);

        let download_dir = self.shared.download_dir.lock().await.clone();
        let save_dir = PathBuf::from(&download_dir);
        std::fs::create_dir_all(&save_dir)
            .map_err(|e| format!("Cannot create download dir: {}", e))?;

        let save_path = save_dir.join(filename);
        let start_time = Instant::now();

        // Parse the blob hash
        let hash: iroh_blobs::Hash = hash_str
            .parse()
            .map_err(|e| format!("Invalid blob hash: {}", e))?;

        // Parse sender's public key and connect via blobs ALPN
        let sender_key: PublicKey = from_peer
            .parse()
            .map_err(|e| format!("Invalid sender ID: {}", e))?;

        // Emit initial "connecting" progress
        let _ = self.shared.app_handle.emit(
            "bharatlink-transfer-progress",
            &TransferProgress {
                transfer_id: request_id.to_string(),
                direction: "receive".to_string(),
                filename: filename.to_string(),
                bytes_transferred: 0,
                total_bytes: file_size,
                percent: 0.0,
                speed_bps: 0,
                status: "connecting".to_string(),
                error: None,
            },
        );

        // Connect to the sender's blob protocol endpoint
        eprintln!("[BharatLink] Connecting to sender {} for blob fetch...", from_peer);
        let conn = self.endpoint
            .connect(sender_key, iroh_blobs::ALPN)
            .await
            .map_err(|e| format!("Failed to connect for file download: {:?}", e))?;
        eprintln!("[BharatLink] Connected to sender, starting blob fetch...");

        // Emit initial "transferring" progress
        let _ = self.shared.app_handle.emit(
            "bharatlink-transfer-progress",
            &TransferProgress {
                transfer_id: request_id.to_string(),
                direction: "receive".to_string(),
                filename: filename.to_string(),
                bytes_transferred: 0,
                total_bytes: file_size,
                percent: 0.0,
                speed_bps: 0,
                status: "transferring".to_string(),
                error: None,
            },
        );

        // Fetch the blob with streaming progress
        let hash_and_format = iroh_blobs::HashAndFormat::raw(hash);
        let get_progress = self.store.remote().fetch(conn, hash_and_format);
        let mut stream = get_progress.stream();
        let mut last_progress_emit = Instant::now();
        let mut download_ok = false;

        while let Some(item) = stream.next().await {
            match item {
                GetProgressItem::Progress(bytes_received) => {
                    // Throttle progress events to ~10/sec
                    if last_progress_emit.elapsed().as_millis() >= 100 {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let speed = if elapsed > 0.0 { (bytes_received as f64 / elapsed) as u64 } else { 0 };
                        let percent = if file_size > 0 {
                            (bytes_received as f64 / file_size as f64) * 100.0
                        } else {
                            0.0
                        };

                        let _ = self.shared.app_handle.emit(
                            "bharatlink-transfer-progress",
                            &TransferProgress {
                                transfer_id: request_id.to_string(),
                                direction: "receive".to_string(),
                                filename: filename.to_string(),
                                bytes_transferred: bytes_received,
                                total_bytes: file_size,
                                percent,
                                speed_bps: speed,
                                status: "transferring".to_string(),
                                error: None,
                            },
                        );
                        last_progress_emit = Instant::now();
                    }
                }
                GetProgressItem::Done(_stats) => {
                    eprintln!("[BharatLink] Download complete for {}", filename);
                    download_ok = true;
                    break;
                }
                GetProgressItem::Error(e) => {
                    let err_msg = format!("Download failed: {:?}", e);
                    eprintln!("[BharatLink] {}", err_msg);
                    let _ = self.shared.app_handle.emit(
                        "bharatlink-transfer-progress",
                        &TransferProgress {
                            transfer_id: request_id.to_string(),
                            direction: "receive".to_string(),
                            filename: filename.to_string(),
                            bytes_transferred: 0,
                            total_bytes: file_size,
                            percent: 0.0,
                            speed_bps: 0,
                            status: "error".to_string(),
                            error: Some(err_msg.clone()),
                        },
                    );
                    return Err(err_msg);
                }
            }
        }

        if !download_ok {
            let err_msg = "Download stream ended unexpectedly".to_string();
            let _ = self.shared.app_handle.emit(
                "bharatlink-transfer-progress",
                &TransferProgress {
                    transfer_id: request_id.to_string(),
                    direction: "receive".to_string(),
                    filename: filename.to_string(),
                    bytes_transferred: 0,
                    total_bytes: file_size,
                    percent: 0.0,
                    speed_bps: 0,
                    status: "error".to_string(),
                    error: Some(err_msg.clone()),
                },
            );
            return Err(err_msg);
        }

        // Stream the downloaded blob directly to disk (no full-file RAM buffer)
        let reader = self.store.blobs().reader(hash);
        let mut file = tokio::fs::File::create(&save_path)
            .await
            .map_err(|e| format!("Failed to create file: {}", e))?;
        let bytes_written = tokio::io::copy(&mut tokio::io::BufReader::new(reader), &mut file)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;

        let duration = start_time.elapsed().as_millis() as u64;

        let entry = TransferHistoryEntry {
            id: request_id.to_string(),
            direction: "receive".to_string(),
            peer_id: from_peer.to_string(),
            peer_nickname: from_nickname,
            transfer_type: "file".to_string(),
            filename: Some(filename.to_string()),
            file_size: Some(bytes_written),
            text_content: None,
            status: "complete".to_string(),
            timestamp: epoch_ms(),
            duration_ms: Some(duration),
            save_path: Some(save_path.to_string_lossy().to_string()),
            blob_hash: Some(hash_str.to_string()),
        };

        // Persist to shared history
        {
            let mut history = self.shared.transfer_history.lock().await;
            history.push(entry.clone());
        }
        self.shared.save_history().await;

        // Emit to frontend
        let _ = self.shared.app_handle.emit("bharatlink-transfer-complete", &entry);

        // Send notification for received file
        send_notification(&self.shared.app_handle, &self.shared.settings, "BharatLink: File Received",
            &format!("{} saved to Downloads", filename));

        eprintln!("[BharatLink] File saved: {} ({} bytes, {}ms)",
            save_path.display(), bytes_written, duration);

        Ok(())
    }
}

// ═══ BharatLinkManager ══════════════════════════════════════════════════

pub struct BharatLinkManager {
    // iroh components
    endpoint: Option<Endpoint>,
    router: Option<Router>,
    store: Option<FsStore>,

    // peer tracking
    peers: HashMap<String, PeerInfo>,
    trusted_peers_shared: Arc<TokioMutex<HashMap<String, String>>>,

    // transfer tracking (shared with protocol handlers)
    active_transfers: HashMap<String, TransferState>,
    transfer_history_shared: Arc<TokioMutex<Vec<TransferHistoryEntry>>>,
    pending_requests_shared: Arc<TokioMutex<HashMap<String, TransferRequest>>>,
    accepted_transfers: std::collections::HashSet<String>,

    // file receive handler (for accepting incoming file transfers)
    file_receiver: Option<FileReceiveHandler>,

    // failed transfers (for retry)
    failed_transfers_shared: Arc<TokioMutex<Vec<TransferHistoryEntry>>>,

    // config
    settings: BharatLinkSettings,
    settings_shared: Arc<TokioMutex<BharatLinkSettings>>,
    download_dir_shared: Arc<TokioMutex<String>>,
    config_dir: PathBuf,
    data_dir: PathBuf,

    // tauri handle for emitting events
    app_handle: Option<tauri::AppHandle>,
}

impl BharatLinkManager {
    pub fn new(config_dir: PathBuf) -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| config_dir.clone())
            .join("flux-terminal")
            .join("bharatlink");

        let bl_config_dir = config_dir.join("bharatlink");

        let default_settings = BharatLinkSettings::default();
        let mut mgr = Self {
            endpoint: None,
            router: None,
            store: None,
            peers: HashMap::new(),
            trusted_peers_shared: Arc::new(TokioMutex::new(HashMap::new())),
            active_transfers: HashMap::new(),
            transfer_history_shared: Arc::new(TokioMutex::new(Vec::new())),
            pending_requests_shared: Arc::new(TokioMutex::new(HashMap::new())),
            accepted_transfers: std::collections::HashSet::new(),
            file_receiver: None,
            failed_transfers_shared: Arc::new(TokioMutex::new(Vec::new())),
            settings: default_settings.clone(),
            settings_shared: Arc::new(TokioMutex::new(default_settings)),
            download_dir_shared: Arc::new(TokioMutex::new(String::new())),
            config_dir: bl_config_dir,
            data_dir,
            app_handle: None,
        };
        mgr.load_state();

        // Auto-generate device name from hostname if not set
        if mgr.settings.device_name.is_none() {
            let hostname = sysinfo::System::host_name().unwrap_or_else(|| "Unknown".to_string());
            mgr.settings.device_name = Some(hostname);
            let _ = mgr.save_state();
        }

        mgr
    }

    // ── Persistence ─────────────────────────────────────────────────────

    fn load_state(&mut self) {
        let settings_path = self.config_dir.join("settings.json");
        if let Ok(data) = std::fs::read_to_string(&settings_path) {
            if let Ok(s) = serde_json::from_str(&data) {
                self.settings = s;
            }
        }

        let peers_path = self.config_dir.join("trusted_peers.json");
        if let Ok(data) = std::fs::read_to_string(&peers_path) {
            if let Ok(p) = serde_json::from_str::<HashMap<String, String>>(&data) {
                self.trusted_peers_shared = Arc::new(TokioMutex::new(p));
            }
        }

        let history_path = self.config_dir.join("transfer_history.json");
        if let Ok(data) = std::fs::read_to_string(&history_path) {
            if let Ok(h) = serde_json::from_str::<Vec<TransferHistoryEntry>>(&data) {
                self.transfer_history_shared = Arc::new(TokioMutex::new(h));
            }
        }

        // Initialize shared state
        self.download_dir_shared = Arc::new(TokioMutex::new(self.settings.download_dir.clone()));
        self.settings_shared = Arc::new(TokioMutex::new(self.settings.clone()));
    }

    fn save_state(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.config_dir)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;

        let data =
            serde_json::to_string_pretty(&self.settings).map_err(|e| format!("{}", e))?;
        std::fs::write(self.config_dir.join("settings.json"), data)
            .map_err(|e| format!("Failed to save settings: {}", e))?;

        // Save trusted peers from shared state — use try_lock to avoid async in sync context
        if let Ok(trusted) = self.trusted_peers_shared.try_lock() {
            let data =
                serde_json::to_string_pretty(&*trusted).map_err(|e| format!("{}", e))?;
            std::fs::write(self.config_dir.join("trusted_peers.json"), data)
                .map_err(|e| format!("Failed to save trusted peers: {}", e))?;
        }

        // Save history from shared state
        let history: Vec<_> = if let Ok(h) = self.transfer_history_shared.try_lock() {
            h.iter().rev().take(MAX_HISTORY).rev().cloned().collect()
        } else {
            Vec::new()
        };
        let data = serde_json::to_string_pretty(&history).map_err(|e| format!("{}", e))?;
        std::fs::write(self.config_dir.join("transfer_history.json"), data)
            .map_err(|e| format!("Failed to save history: {}", e))?;

        Ok(())
    }

    // ── Secret key management ───────────────────────────────────────────

    fn load_or_create_secret_key(&self) -> Result<SecretKey, String> {
        std::fs::create_dir_all(&self.config_dir)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;

        let key_path = self.config_dir.join("secret.key");
        if key_path.exists() {
            let bytes = std::fs::read(&key_path)
                .map_err(|e| format!("Failed to read secret key: {}", e))?;
            if bytes.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                return Ok(SecretKey::from_bytes(&arr));
            }
        }

        let key = SecretKey::generate(&mut rand::rng());
        std::fs::write(&key_path, key.to_bytes())
            .map_err(|e| format!("Failed to save secret key: {}", e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            let _ = std::fs::set_permissions(&key_path, perms);
        }

        Ok(key)
    }

    // ── Node lifecycle ──────────────────────────────────────────────────

    pub async fn start(&mut self, app_handle: tauri::AppHandle) -> Result<NodeInfo, String> {
        if self.endpoint.is_some() {
            return self.get_node_info();
        }

        self.app_handle = Some(app_handle.clone());

        let secret_key = self.load_or_create_secret_key()?;

        // Create blob store on disk
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Failed to create data dir: {}", e))?;
        let store = FsStore::load(self.data_dir.join("blobs"))
            .await
            .map_err(|e| format!("Failed to create blob store: {}", e))?;

        // Build endpoint with full discovery stack:
        //   - N0 preset: PkarrPublisher (publishes to dns.iroh.link) + DnsDiscovery (resolves via DNS)
        //   - Relay servers for NAT traversal
        //   - mDNS for local network discovery (added after bind)
        let endpoint = Endpoint::builder()
            .secret_key(secret_key)
            .alpns(vec![
                BHARATLINK_META_ALPN.to_vec(),
                BHARATLINK_TEXT_ALPN.to_vec(),
                BHARATLINK_SIGNAL_ALPN.to_vec(),
            ])
            .bind()
            .await
            .map_err(|e| format!("Failed to bind endpoint: {}", e))?;

        // Add mDNS discovery for local network (on top of DNS/pkarr for remote)
        let mdns = MdnsDiscovery::builder()
            .build(endpoint.id())
            .map_err(|e| format!("Failed to start mDNS: {}", e))?;
        endpoint.discovery().add(mdns);

        // Update shared download dir
        *self.download_dir_shared.lock().await = self.settings.download_dir.clone();

        // Create shared state for protocol handlers
        let shared = SharedState {
            app_handle: app_handle.clone(),
            trusted_peers: self.trusted_peers_shared.clone(),
            transfer_history: self.transfer_history_shared.clone(),
            pending_requests: self.pending_requests_shared.clone(),
            download_dir: self.download_dir_shared.clone(),
            settings: self.settings_shared.clone(),
            failed_transfers: self.failed_transfers_shared.clone(),
            config_dir: self.config_dir.clone(),
        };

        // Create protocol handlers
        let blobs = BlobsProtocol::new(&store, None);
        let meta_handler = MetaProtocolHandler {
            shared: shared.clone(),
            settings: self.settings_shared.clone(),
            store: store.clone(),
            endpoint_for_accept: endpoint.clone(),
        };
        let text_handler = TextProtocolHandler {
            shared: shared.clone(),
            endpoint_for_signal: endpoint.clone(),
        };
        let signal_handler = SignalProtocolHandler {
            shared: shared.clone(),
        };

        // Store file receive handler for accepting incoming transfers
        self.file_receiver = Some(FileReceiveHandler {
            shared,
            store: store.clone(),
            endpoint: endpoint.clone(),
        });

        // Build router with ALL protocols — blobs + meta + text + signal
        let router = Router::builder(endpoint.clone())
            .accept(iroh_blobs::ALPN, blobs)
            .accept(BHARATLINK_META_ALPN, meta_handler)
            .accept(BHARATLINK_TEXT_ALPN, text_handler)
            .accept(BHARATLINK_SIGNAL_ALPN, signal_handler)
            .spawn();

        // Spawn peer discovery polling loop
        let ep_for_discovery = endpoint.clone();
        let app_for_discovery = app_handle.clone();
        let trusted_for_discovery = self.trusted_peers_shared.clone();
        tokio::spawn(async move {
            Self::peer_discovery_loop(ep_for_discovery, app_for_discovery, trusted_for_discovery).await;
        });

        self.endpoint = Some(endpoint);
        self.router = Some(router);
        self.store = Some(store);

        self.get_node_info()
    }

    /// Periodically polls the endpoint for discovered remote nodes and emits events.
    /// Uses active QUIC probes to determine peer liveness — iroh's latency()
    /// and conn_type() return stale cached data and cannot be trusted.
    async fn peer_discovery_loop(
        endpoint: Endpoint,
        app_handle: tauri::AppHandle,
        trusted_peers: Arc<TokioMutex<HashMap<String, String>>>,
    ) {
        // Track peer state: node_id -> (last_seen_ms, is_connected)
        let mut peer_state: HashMap<String, (u64, bool)> = HashMap::new();
        let mut cycle_count: u32 = 0;

        // How long before a peer is considered offline if probe hasn't run yet
        const PROBE_TIMEOUT_SECS: u64 = 5;

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            cycle_count += 1;

            if endpoint.is_closed() {
                break;
            }

            let our_addr = endpoint.addr();
            let our_id = endpoint.id().to_string();

            let trusted = trusted_peers.lock().await;
            let trusted_clone = trusted.clone();
            drop(trusted);

            // Emit node status
            let relay_url = our_addr.relay_urls().next().map(|u| u.to_string());
            let local_addrs: Vec<String> = our_addr.ip_addrs().map(|a| a.to_string()).collect();

            let status = NodeInfo {
                node_id: our_id.clone(),
                node_id_short: short_id(&our_id),
                is_running: true,
                relay_url,
                local_addrs,
                discovered_peers: peer_state.len(),
            };
            let _ = app_handle.emit("bharatlink-node-status", &status);

            // Active probe: attempt a lightweight QUIC connect to each trusted peer
            // with a short timeout to determine actual reachability.
            // Probes run concurrently for all peers.
            let mut probe_handles = Vec::new();

            for (peer_id, _nickname) in &trusted_clone {
                if peer_id == &our_id {
                    continue;
                }

                if let Ok(peer_key) = peer_id.parse::<PublicKey>() {
                    let ep = endpoint.clone();
                    let pid = peer_id.clone();
                    probe_handles.push(tokio::spawn(async move {
                        let result = tokio::time::timeout(
                            std::time::Duration::from_secs(PROBE_TIMEOUT_SECS),
                            ep.connect(peer_key, BHARATLINK_SIGNAL_ALPN),
                        ).await;
                        let online = match result {
                            Ok(Ok(conn)) => {
                                // Connection succeeded — peer is online. Drop it immediately.
                                conn.close(0u32.into(), b"probe");
                                true
                            }
                            _ => false, // Timeout or connection error = offline
                        };
                        (pid, online)
                    }));
                }
            }

            // Collect probe results
            let mut probe_results: HashMap<String, bool> = HashMap::new();
            for handle in probe_handles {
                if let Ok((pid, online)) = handle.await {
                    probe_results.insert(pid, online);
                }
            }

            // Process results and emit events
            let now = epoch_ms();
            for (peer_id, nickname) in &trusted_clone {
                if peer_id == &our_id {
                    continue;
                }

                let is_connected = probe_results.get(peer_id).copied().unwrap_or(false);

                let prev_state = peer_state.get(peer_id).copied();
                let is_new = prev_state.is_none();
                let was_connected = prev_state.map(|(_, wc)| wc).unwrap_or(false);
                let status_changed = prev_state
                    .map(|(_, prev_conn)| prev_conn != is_connected)
                    .unwrap_or(false);

                let last_seen = if is_connected { now } else {
                    prev_state.map(|(ls, _)| ls).unwrap_or(0)
                };
                peer_state.insert(peer_id.clone(), (last_seen, is_connected));

                // Detect state transitions
                if status_changed && is_connected && !was_connected && !is_new {
                    emit_error(&app_handle, "reconnection",
                        &format!("{} is back online", nickname),
                        Some(peer_id), None);
                    let _ = app_handle.emit("bharatlink-peer-reconnected", peer_id.as_str());
                } else if status_changed && !is_connected && was_connected {
                    emit_error(&app_handle, "connection",
                        &format!("{} went offline", nickname),
                        Some(peer_id), None);
                }

                // Re-emit all peer statuses every 3rd cycle (~15s) as heartbeat
                let is_heartbeat = cycle_count % 3 == 0;

                if is_new || status_changed || is_heartbeat {
                    let peer_info = PeerInfo {
                        node_id_short: short_id(peer_id),
                        node_id: peer_id.clone(),
                        nickname: Some(nickname.clone()),
                        is_local: false,
                        last_seen,
                        is_connected,
                        is_trusted: true,
                    };
                    let _ = app_handle.emit("bharatlink-peer-discovered", &peer_info);
                }
            }
        }
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        if let Some(router) = self.router.take() {
            router
                .shutdown()
                .await
                .map_err(|e| format!("Router shutdown error: {}", e))?;
        }
        if let Some(endpoint) = self.endpoint.take() {
            endpoint.close().await;
        }
        self.store = None;
        self.save_state()?;

        // Emit stopped status so the status bar updates
        if let Some(ref app) = self.app_handle {
            let _ = app.emit("bharatlink-node-status", &NodeInfo {
                node_id: String::new(),
                node_id_short: String::new(),
                is_running: false,
                relay_url: None,
                local_addrs: vec![],
                discovered_peers: 0,
            });
        }

        Ok(())
    }

    pub fn get_node_info(&self) -> Result<NodeInfo, String> {
        match &self.endpoint {
            Some(ep) => {
                let node_id = ep.id().to_string();
                let node_id_short = if node_id.len() >= 8 {
                    node_id[..8].to_string()
                } else {
                    node_id.clone()
                };

                let addr = ep.addr();
                let relay_url = addr.relay_urls().next().map(|u| u.to_string());
                let local_addrs: Vec<String> = addr.ip_addrs().map(|a| a.to_string()).collect();

                Ok(NodeInfo {
                    node_id,
                    node_id_short,
                    is_running: true,
                    relay_url,
                    local_addrs,
                    discovered_peers: self.peers.len(),
                })
            }
            None => Ok(NodeInfo {
                node_id: String::new(),
                node_id_short: String::new(),
                is_running: false,
                relay_url: None,
                local_addrs: Vec::new(),
                discovered_peers: 0,
            }),
        }
    }

    // ── Peer management ─────────────────────────────────────────────────

    pub fn get_peers(&self) -> Vec<PeerInfo> {
        let mut result: Vec<PeerInfo> = self.peers.values().cloned().collect();

        // Also include trusted peers not yet discovered
        if let Ok(trusted) = self.trusted_peers_shared.try_lock() {
            for (id, nickname) in trusted.iter() {
                if !self.peers.contains_key(id) {
                    result.push(PeerInfo {
                        node_id_short: short_id(id),
                        node_id: id.clone(),
                        nickname: Some(nickname.clone()),
                        is_local: false,
                        last_seen: 0,
                        is_connected: false,
                        is_trusted: true,
                    });
                }
            }
        }

        result
    }

    pub fn add_peer(
        &mut self,
        node_id: String,
        nickname: Option<String>,
    ) -> Result<PeerInfo, String> {
        if node_id.len() < 8 {
            return Err("Invalid node ID: too short".to_string());
        }

        let is_trusted = self.trusted_peers_shared.try_lock()
            .map(|t| t.contains_key(&node_id))
            .unwrap_or(false);

        let peer = PeerInfo {
            node_id_short: short_id(&node_id),
            node_id: node_id.clone(),
            nickname: nickname.clone(),
            is_local: false,
            last_seen: epoch_ms(),
            is_connected: false,
            is_trusted,
        };

        self.peers.insert(node_id.clone(), peer.clone());

        // Also add to trusted peers if a nickname is provided
        if let Some(name) = nickname {
            if let Ok(mut trusted) = self.trusted_peers_shared.try_lock() {
                trusted.insert(node_id, name);
            }
            let _ = self.save_state();
        }

        Ok(peer)
    }

    pub async fn trust_peer(&mut self, node_id: String, nickname: String) -> Result<(), String> {
        {
            let mut trusted = self.trusted_peers_shared.lock().await;
            trusted.insert(node_id.clone(), nickname.clone());
        }

        if let Some(peer) = self.peers.get_mut(&node_id) {
            peer.is_trusted = true;
            peer.nickname = Some(nickname);
        }

        self.save_state()
    }

    pub async fn untrust_peer(&mut self, node_id: String) -> Result<(), String> {
        {
            let mut trusted = self.trusted_peers_shared.lock().await;
            trusted.remove(&node_id);
        }

        if let Some(peer) = self.peers.get_mut(&node_id) {
            peer.is_trusted = false;
        }

        self.save_state()
    }

    // ── File transfer ───────────────────────────────────────────────────

    pub async fn send_file(
        &mut self,
        peer_id: String,
        file_path: String,
    ) -> Result<String, String> {
        let endpoint = self
            .endpoint
            .as_ref()
            .ok_or("Node not running")?
            .clone();
        let store = self
            .store
            .as_ref()
            .ok_or("Blob store not initialized")?
            .clone();

        let path = PathBuf::from(&file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        let transfer_id = uuid::Uuid::new_v4().to_string();
        let app_handle = self.app_handle.clone();

        // Emit initial progress
        if let Some(ref handle) = app_handle {
            let _ = handle.emit(
                "bharatlink-transfer-progress",
                &TransferProgress {
                    transfer_id: transfer_id.clone(),
                    direction: "send".to_string(),
                    filename: file_name.clone(),
                    bytes_transferred: 0,
                    total_bytes: file_size,
                    percent: 0.0,
                    speed_bps: 0,
                    status: "connecting".to_string(),
                    error: None,
                },
            );
        }

        // Add file to blob store
        let abs_path = std::fs::canonicalize(&path)
            .map_err(|e| format!("Cannot resolve path: {}", e))?;

        let tag_info = store
            .blobs()
            .add_path(abs_path)
            .await
            .map_err(|e| format!("Failed to add file to store: {}", e))?;

        let hash = tag_info.hash;

        // Parse peer public key
        let peer_key: PublicKey = peer_id
            .parse()
            .map_err(|e| format!("Invalid peer ID: {}", e))?;

        // Connect using EndpointAddr — iroh will use mDNS/relay to resolve
        let conn = endpoint
            .connect(peer_key, BHARATLINK_META_ALPN)
            .await
            .map_err(|e| {
                if let Some(ref h) = app_handle {
                    emit_error(h, "connection", &format!("Failed to send file: {}", e), Some(&peer_id), Some(&transfer_id));
                }
                format!("Failed to connect to peer: {}. Make sure the peer is online and both devices are on the same network, or try adding the peer's full endpoint address.", e)
            })?;

        let request = TransferRequest {
            id: transfer_id.clone(),
            from_peer: endpoint.id().to_string(),
            from_nickname: self.settings.device_name.clone(),
            transfer_type: "file".to_string(),
            filename: Some(file_name.clone()),
            file_size: Some(file_size),
            text_preview: None,
            blob_hash: Some(hash.to_string()),
            timestamp: epoch_ms(),
        };

        let msg_bytes =
            serde_json::to_vec(&request).map_err(|e| format!("Serialize error: {}", e))?;

        let (mut send, _recv) = conn
            .open_bi()
            .await
            .map_err(|e| format!("Stream error: {}", e))?;

        send.write_all(&msg_bytes)
            .await
            .map_err(|e| format!("Write error: {}", e))?;
        send.finish()
            .map_err(|e| format!("Finish error: {}", e))?;

        // Emit "transferring" progress — file is ready for receiver to fetch
        if let Some(ref handle) = app_handle {
            let _ = handle.emit(
                "bharatlink-transfer-progress",
                &TransferProgress {
                    transfer_id: transfer_id.clone(),
                    direction: "send".to_string(),
                    filename: file_name.clone(),
                    bytes_transferred: file_size,
                    total_bytes: file_size,
                    percent: 100.0,
                    speed_bps: 0,
                    status: "transferring".to_string(),
                    error: None,
                },
            );
        }

        // Give time for the receiver to read the META request before dropping connection
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        drop(send);
        drop(conn);

        // Record in history
        let nickname = {
            let trusted = self.trusted_peers_shared.lock().await;
            trusted.get(&peer_id).cloned()
        };

        let entry = TransferHistoryEntry {
            id: transfer_id.clone(),
            direction: "send".to_string(),
            peer_id: peer_id.clone(),
            peer_nickname: nickname,
            transfer_type: "file".to_string(),
            filename: Some(file_name.clone()),
            file_size: Some(file_size),
            text_content: None,
            status: "complete".to_string(),
            timestamp: epoch_ms(),
            duration_ms: None,
            save_path: Some(file_path.clone()),
            blob_hash: Some(hash.to_string()),
        };
        if let Ok(mut h) = self.transfer_history_shared.try_lock() {
            h.push(entry.clone());
        }
        let _ = self.save_state();

        if let Some(ref handle) = app_handle {
            let _ = handle.emit("bharatlink-transfer-complete", &entry);
        }

        Ok(transfer_id)
    }

    pub async fn send_text(
        &mut self,
        peer_id: String,
        text: String,
    ) -> Result<String, String> {
        let endpoint = self
            .endpoint
            .as_ref()
            .ok_or("Node not running")?
            .clone();

        let transfer_id = uuid::Uuid::new_v4().to_string();
        let app_handle = self.app_handle.clone();

        let peer_key: PublicKey = peer_id
            .parse()
            .map_err(|e| format!("Invalid peer ID: {}", e))?;

        let conn = endpoint
            .connect(peer_key, BHARATLINK_TEXT_ALPN)
            .await
            .map_err(|e| {
                if let Some(ref h) = app_handle {
                    emit_error(h, "connection", &format!("Failed to send text: {}", e), Some(&peer_id), Some(&transfer_id));
                }
                format!("Failed to connect to peer: {}. Make sure the peer is online and both devices are on the same network.", e)
            })?;

        let mut send = conn
            .open_uni()
            .await
            .map_err(|e| format!("Stream error: {}", e))?;

        let text_bytes = text.as_bytes();
        send.write_all(text_bytes)
            .await
            .map_err(|e| format!("Write error: {}", e))?;
        send.finish()
            .map_err(|e| format!("Finish error: {}", e))?;

        // Give the receiver time to read the data before dropping the connection.
        // finish() signals EOF but dropping conn immediately can close the QUIC
        // connection before all data is flushed. A small delay ensures delivery.
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Drop connection gracefully (QUIC will flush remaining data)
        drop(send);
        drop(conn);

        let nickname = {
            let trusted = self.trusted_peers_shared.lock().await;
            trusted.get(&peer_id).cloned()
        };

        let entry = TransferHistoryEntry {
            id: transfer_id.clone(),
            direction: "send".to_string(),
            peer_id: peer_id.clone(),
            peer_nickname: nickname,
            transfer_type: "text".to_string(),
            filename: None,
            file_size: Some(text_bytes.len() as u64),
            text_content: Some(text.clone()),
            status: "complete".to_string(),
            timestamp: epoch_ms(),
            duration_ms: None,
            save_path: None,
            blob_hash: None,
        };
        if let Ok(mut h) = self.transfer_history_shared.try_lock() {
            h.push(entry.clone());
        }
        let _ = self.save_state();

        if let Some(ref handle) = app_handle {
            let _ = handle.emit("bharatlink-transfer-complete", &entry);
        }

        Ok(transfer_id)
    }

    // ── Transfer management ─────────────────────────────────────────────

    pub async fn accept_transfer(&mut self, request_id: String) -> Result<(), String> {
        let request = {
            let mut pending = self.pending_requests_shared.lock().await;
            pending.remove(&request_id)
        };
        self.accepted_transfers.insert(request_id.clone());

        // If it's a file transfer, trigger the download
        if let Some(req) = request {
            if req.transfer_type == "file" {
                if let (Some(hash), Some(filename)) = (&req.blob_hash, &req.filename) {
                    if let Some(ref receiver) = self.file_receiver {
                        let receiver = receiver.clone();
                        let hash = hash.clone();
                        let filename = filename.clone();
                        let file_size = req.file_size.unwrap_or(0);
                        let from_peer = req.from_peer.clone();
                        let from_nickname = req.from_nickname.clone();
                        let request_id = req.id.clone();

                        let app_handle_for_err = self.app_handle.clone();

                        // Spawn download in background
                        tokio::spawn(async move {
                            if let Err(e) = receiver.download_blob(
                                &hash,
                                &filename,
                                file_size,
                                &from_peer,
                                from_nickname.clone(),
                                &request_id,
                            ).await {
                                eprintln!("[BharatLink] File download failed: {}", e);
                                // Emit error event for inline chat display
                                if let Some(ref handle) = app_handle_for_err {
                                    emit_error(handle, "transfer", &format!("Download failed: {}", e),
                                        Some(&from_peer), Some(&request_id));
                                }
                                // Emit failed entry so frontend knows
                                let fail_entry = TransferHistoryEntry {
                                    id: request_id.clone(),
                                    direction: "receive".to_string(),
                                    peer_id: from_peer.clone(),
                                    peer_nickname: from_nickname,
                                    transfer_type: "file".to_string(),
                                    filename: Some(filename.clone()),
                                    file_size: None,
                                    text_content: None,
                                    status: "failed".to_string(),
                                    timestamp: epoch_ms(),
                                    duration_ms: None,
                                    save_path: Some(format!("Error: {}", e)),
                                    blob_hash: Some(hash),
                                };
                                // Persist to shared history so retry_transfer can find it
                                {
                                    let mut history = receiver.shared.transfer_history.lock().await;
                                    history.push(fail_entry.clone());
                                }
                                receiver.shared.save_history().await;
                                if let Some(ref handle) = app_handle_for_err {
                                    let _ = handle.emit("bharatlink-transfer-complete", &fail_entry);
                                }
                            }
                        });
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn reject_transfer(&mut self, request_id: String) -> Result<(), String> {
        let mut pending = self.pending_requests_shared.lock().await;
        pending.remove(&request_id);
        Ok(())
    }

    pub fn cancel_transfer(&mut self, transfer_id: String) -> Result<(), String> {
        if let Some(state) = self.active_transfers.remove(&transfer_id) {
            let _ = state.cancel_tx.send(());
        }
        Ok(())
    }

    /// Retry a failed transfer by looking up its metadata in history
    pub async fn retry_transfer(&mut self, transfer_id: String) -> Result<String, String> {
        // Find the failed entry in history
        let entry = {
            let history = self.transfer_history_shared.lock().await;
            history.iter().find(|e| e.id == transfer_id && e.status == "failed").cloned()
        };

        let entry = entry.ok_or("Transfer not found or not in failed state")?;

        if entry.direction == "receive" {
            // Retry receiving a file
            let blob_hash = entry.blob_hash.as_ref().ok_or("Cannot retry: no blob hash stored")?;
            let filename = entry.filename.as_ref().ok_or("Cannot retry: no filename stored")?;
            let file_size = entry.file_size.unwrap_or(0);

            if let Some(ref receiver) = self.file_receiver {
                let receiver = receiver.clone();
                let hash = blob_hash.clone();
                let filename = filename.clone();
                let from_peer = entry.peer_id.clone();
                let from_nickname = entry.peer_nickname.clone();
                let new_id = uuid::Uuid::new_v4().to_string();
                let app_handle_for_err = self.app_handle.clone();

                // Remove the old failed entry from history
                {
                    let mut history = self.transfer_history_shared.lock().await;
                    history.retain(|e| e.id != transfer_id);
                }

                let new_id_clone = new_id.clone();
                tokio::spawn(async move {
                    if let Err(e) = receiver.download_blob(
                        &hash, &filename, file_size,
                        &from_peer, from_nickname.clone(), &new_id_clone,
                    ).await {
                        eprintln!("[BharatLink] Retry download failed: {}", e);
                        if let Some(ref handle) = app_handle_for_err {
                            emit_error(handle, "transfer", &format!("Retry failed: {}", e),
                                Some(&from_peer), Some(&new_id_clone));
                            let fail_entry = TransferHistoryEntry {
                                id: new_id_clone, direction: "receive".to_string(),
                                peer_id: from_peer, peer_nickname: from_nickname,
                                transfer_type: "file".to_string(),
                                filename: Some(filename.clone()), file_size: None,
                                text_content: None, status: "failed".to_string(),
                                timestamp: epoch_ms(), duration_ms: None,
                                save_path: Some(format!("Error: {}", e)),
                                blob_hash: Some(hash),
                            };
                            let _ = handle.emit("bharatlink-transfer-complete", &fail_entry);
                        }
                    }
                });

                Ok(new_id)
            } else {
                Err("File receiver not initialized — is the node running?".to_string())
            }
        } else if entry.direction == "send" {
            // Retry sending a file
            let file_path = entry.save_path.as_ref().ok_or("Cannot retry send: original file path not stored")?;
            self.send_file(entry.peer_id.clone(), file_path.clone()).await
        } else {
            Err("Cannot retry this transfer type".to_string())
        }
    }

    // ── History ─────────────────────────────────────────────────────────

    pub fn get_history(&self) -> Vec<TransferHistoryEntry> {
        self.transfer_history_shared.try_lock()
            .map(|h| h.clone())
            .unwrap_or_default()
    }

    pub fn clear_history(&mut self) -> Result<(), String> {
        if let Ok(mut h) = self.transfer_history_shared.try_lock() {
            h.clear();
        }
        self.save_state()
    }

    // ── Settings ────────────────────────────────────────────────────────

    pub fn get_settings(&self) -> BharatLinkSettings {
        self.settings.clone()
    }

    pub async fn update_settings(&mut self, settings: BharatLinkSettings) -> Result<(), String> {
        // Update shared download dir if it changed
        if settings.download_dir != self.settings.download_dir {
            *self.download_dir_shared.lock().await = settings.download_dir.clone();
        }
        self.settings = settings.clone();
        *self.settings_shared.lock().await = settings;
        self.save_state()
    }

    // ── Multi-file transfer ─────────────────────────────────────────────

    pub async fn send_files(
        &mut self,
        peer_id: String,
        file_paths: Vec<String>,
    ) -> Result<String, String> {
        let batch_id = uuid::Uuid::new_v4().to_string();
        let total = file_paths.len();

        for (i, file_path) in file_paths.iter().enumerate() {
            eprintln!(
                "[BharatLink] Sending file {}/{} from batch {}: {}",
                i + 1, total, batch_id, file_path
            );
            match self.send_file(peer_id.clone(), file_path.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("[BharatLink] Batch file send error: {}", e);
                    // Continue with remaining files instead of aborting
                }
            }
        }

        Ok(batch_id)
    }

    pub fn list_dir_files(&self, dir_path: String) -> Result<Vec<String>, String> {
        let path = PathBuf::from(&dir_path);
        if !path.is_dir() {
            return Err(format!("Not a directory: {}", dir_path));
        }

        let mut files = Vec::new();
        Self::collect_files(&path, &mut files)
            .map_err(|e| format!("Failed to list files: {}", e))?;
        Ok(files)
    }

    fn collect_files(dir: &std::path::Path, files: &mut Vec<String>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                Self::collect_files(&path, files)?;
            } else if path.is_file() {
                if let Some(name) = path.file_name() {
                    // Skip hidden files
                    if !name.to_string_lossy().starts_with('.') {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
        Ok(())
    }

    // ── Screenshot capture ──────────────────────────────────────────────

    pub async fn capture_and_send_screenshot(&mut self, peer_id: String) -> Result<String, String> {
        let screenshot_path = std::env::temp_dir()
            .join(format!("bharatlink_screenshot_{}.png", uuid::Uuid::new_v4()));

        #[cfg(target_os = "macos")]
        {
            let output = tokio::process::Command::new("screencapture")
                .args(["-x", &screenshot_path.to_string_lossy()])
                .output()
                .await
                .map_err(|e| format!("Screenshot capture failed: {}", e))?;

            if !output.status.success() {
                return Err("Screenshot capture was cancelled or failed".to_string());
            }
        }

        #[cfg(target_os = "windows")]
        {
            let ps_script = format!(
                r#"
                Add-Type -AssemblyName System.Windows.Forms
                $screen = [System.Windows.Forms.Screen]::PrimaryScreen
                $bitmap = New-Object System.Drawing.Bitmap($screen.Bounds.Width, $screen.Bounds.Height)
                $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
                $graphics.CopyFromScreen($screen.Bounds.Location, [System.Drawing.Point]::Empty, $screen.Bounds.Size)
                $bitmap.Save('{}')
                $graphics.Dispose()
                $bitmap.Dispose()
                "#,
                screenshot_path.to_string_lossy().replace('\\', "\\\\")
            );

            let output = tokio::process::Command::new("powershell")
                .args(["-Command", &ps_script])
                .output()
                .await
                .map_err(|e| format!("Screenshot capture failed: {}", e))?;

            if !output.status.success() {
                return Err("Screenshot capture failed".to_string());
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            return Err("Screenshot not supported on this platform".to_string());
        }

        if !screenshot_path.exists() {
            return Err("Screenshot file was not created".to_string());
        }

        let result = self
            .send_file(peer_id, screenshot_path.to_string_lossy().to_string())
            .await;

        // Clean up temp file
        let _ = std::fs::remove_file(&screenshot_path);

        result
    }

    // ── Clipboard operations ────────────────────────────────────────────

    pub async fn send_clipboard_text(
        &mut self,
        peer_id: String,
        clipboard_text: String,
    ) -> Result<String, String> {
        // Prefix with [CLIPBOARD] marker so receiver knows it's clipboard content
        let text = format!("[CLIPBOARD]{}", clipboard_text);
        self.send_text(peer_id, text).await
    }

    // ── Signals (typing indicator, read receipts) ──────────────────────

    pub async fn send_signal(
        &self,
        peer_id: String,
        signal_type: String,
        message_id: Option<String>,
    ) -> Result<(), String> {
        let endpoint = self.endpoint.as_ref().ok_or("Node not running")?.clone();

        let peer_key: PublicKey = peer_id
            .parse()
            .map_err(|e| format!("Invalid peer ID: {}", e))?;

        let signal = BharatLinkSignal {
            signal_type,
            message_id,
            from_peer: endpoint.id().to_string(),
            timestamp: epoch_ms(),
        };

        let data = serde_json::to_vec(&signal).map_err(|e| format!("Serialize error: {}", e))?;

        let conn = endpoint
            .connect(peer_key, BHARATLINK_SIGNAL_ALPN)
            .await
            .map_err(|e| format!("Signal connect error: {}", e))?;

        let mut send = conn.open_uni().await.map_err(|e| format!("Stream error: {}", e))?;
        send.write_all(&data).await.map_err(|e| format!("Write error: {}", e))?;
        send.finish().map_err(|e| format!("Finish error: {}", e))?;

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        Ok(())
    }
}

// ═══ Utility ════════════════════════════════════════════════════════════

fn epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn short_id(id: &str) -> String {
    if id.len() >= 8 {
        id[..8].to_string()
    } else {
        id.to_string()
    }
}

// ═══ Tauri Commands ═════════════════════════════════════════════════════

#[tauri::command]
pub async fn bharatlink_start(
    state: tauri::State<'_, crate::commands::AppState>,
    app_handle: tauri::AppHandle,
) -> Result<NodeInfo, String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.start(app_handle).await
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

// ═══ New commands — Multi-file, Screenshot, Clipboard ═══

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
