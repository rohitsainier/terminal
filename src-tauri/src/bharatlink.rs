// ═══════════════════════════════════════════════════════════════════════════
//  FLUX BHARATLINK — P2P File & Text Sharing (iroh-powered)
//  Sovereign peer-to-peer sharing: no servers, no accounts, pure QUIC+mDNS
// ═══════════════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::Emitter;

use iroh::{
    discovery::mdns::MdnsDiscovery,
    endpoint::Incoming,
    protocol::Router,
    Endpoint, PublicKey, RelayMode, SecretKey,
};
use iroh_blobs::{store::fs::FsStore, BlobsProtocol};

// ═══ Constants ══════════════════════════════════════════════════════════════

const BHARATLINK_TEXT_ALPN: &[u8] = b"bharatlink/text/1";
const BHARATLINK_META_ALPN: &[u8] = b"bharatlink/meta/1";
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
}

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
        }
    }
}

// ═══ Internal transfer state ═════════════════════════════════════════════

#[allow(dead_code)]
struct TransferState {
    cancel_tx: tokio::sync::oneshot::Sender<()>,
    start_time: Instant,
}

// ═══ BharatLinkManager ══════════════════════════════════════════════════

pub struct BharatLinkManager {
    // iroh components
    endpoint: Option<Endpoint>,
    router: Option<Router>,
    #[allow(dead_code)]
    store: Option<FsStore>,

    // peer tracking
    peers: HashMap<String, PeerInfo>,
    trusted_peers: HashMap<String, String>, // node_id → nickname

    // transfer tracking
    active_transfers: HashMap<String, TransferState>,
    transfer_history: Vec<TransferHistoryEntry>,
    #[allow(dead_code)]
    pending_requests: HashMap<String, TransferRequest>,
    #[allow(dead_code)]
    accepted_transfers: std::collections::HashSet<String>,

    // config
    settings: BharatLinkSettings,
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

        let mut mgr = Self {
            endpoint: None,
            router: None,
            store: None,
            peers: HashMap::new(),
            trusted_peers: HashMap::new(),
            active_transfers: HashMap::new(),
            transfer_history: Vec::new(),
            pending_requests: HashMap::new(),
            accepted_transfers: std::collections::HashSet::new(),
            settings: BharatLinkSettings::default(),
            config_dir: bl_config_dir,
            data_dir,
            app_handle: None,
        };
        mgr.load_state();
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
            if let Ok(p) = serde_json::from_str(&data) {
                self.trusted_peers = p;
            }
        }

        let history_path = self.config_dir.join("transfer_history.json");
        if let Ok(data) = std::fs::read_to_string(&history_path) {
            if let Ok(h) = serde_json::from_str(&data) {
                self.transfer_history = h;
            }
        }
    }

    fn save_state(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.config_dir)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;

        let data =
            serde_json::to_string_pretty(&self.settings).map_err(|e| format!("{}", e))?;
        std::fs::write(self.config_dir.join("settings.json"), data)
            .map_err(|e| format!("Failed to save settings: {}", e))?;

        let data =
            serde_json::to_string_pretty(&self.trusted_peers).map_err(|e| format!("{}", e))?;
        std::fs::write(self.config_dir.join("trusted_peers.json"), data)
            .map_err(|e| format!("Failed to save trusted peers: {}", e))?;

        let history: Vec<_> = self
            .transfer_history
            .iter()
            .rev()
            .take(MAX_HISTORY)
            .rev()
            .cloned()
            .collect();
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
        let bytes = key.to_bytes();
        std::fs::write(&key_path, bytes)
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

        // Build endpoint with mDNS discovery
        let endpoint = Endpoint::empty_builder(RelayMode::Default)
            .secret_key(secret_key)
            .alpns(vec![
                BHARATLINK_META_ALPN.to_vec(),
                BHARATLINK_TEXT_ALPN.to_vec(),
            ])
            .bind()
            .await
            .map_err(|e| format!("Failed to bind endpoint: {}", e))?;

        // Register mDNS discovery
        let mdns = MdnsDiscovery::builder()
            .build(endpoint.id())
            .map_err(|e| format!("Failed to start mDNS: {}", e))?;
        endpoint.discovery().add(mdns);

        // Set up blob protocol handler
        let blobs = BlobsProtocol::new(&store, None);

        // Build router for blobs protocol
        let router = Router::builder(endpoint.clone())
            .accept(iroh_blobs::ALPN, blobs)
            .spawn();

        self.endpoint = Some(endpoint.clone());
        self.router = Some(router);
        self.store = Some(store);

        // Spawn accept loop for custom protocols (meta + text)
        let ep_clone = endpoint.clone();
        let handle_clone = app_handle;
        let trusted = self.trusted_peers.clone();

        tokio::spawn(async move {
            Self::accept_loop(ep_clone, handle_clone, trusted).await;
        });

        self.get_node_info()
    }

    async fn accept_loop(
        endpoint: Endpoint,
        app_handle: tauri::AppHandle,
        trusted_peers: HashMap<String, String>,
    ) {
        loop {
            let incoming = match endpoint.accept().await {
                Some(incoming) => incoming,
                None => break,
            };

            let handle = app_handle.clone();
            let trusted = trusted_peers.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_incoming(incoming, handle, trusted).await {
                    eprintln!("[BharatLink] Accept error: {}", e);
                }
            });
        }
    }

    async fn handle_incoming(
        incoming: Incoming,
        app_handle: tauri::AppHandle,
        trusted_peers: HashMap<String, String>,
    ) -> Result<(), String> {
        let mut connecting = incoming
            .accept()
            .map_err(|e| format!("Accept error: {}", e))?;

        let alpn = connecting
            .alpn()
            .await
            .map_err(|e| format!("ALPN error: {}", e))?;

        let conn = connecting
            .await
            .map_err(|e| format!("Connection error: {}", e))?;

        let remote_id = conn.remote_id();
        let remote_str = remote_id.to_string();

        if alpn == BHARATLINK_META_ALPN {
            let (_send, mut recv) = conn
                .accept_bi()
                .await
                .map_err(|e| format!("Stream error: {}", e))?;

            let mut buf = vec![0u8; 64 * 1024];
            let n = recv
                .read(&mut buf)
                .await
                .map_err(|e| format!("Read error: {:?}", e))?;
            if let Some(n) = n {
                let msg: TransferRequest = serde_json::from_slice(&buf[..n])
                    .map_err(|e| format!("Parse error: {}", e))?;

                let _ = app_handle.emit("bharatlink-incoming-request", &msg);
            }
        } else if alpn == BHARATLINK_TEXT_ALPN {
            let mut recv = conn
                .accept_uni()
                .await
                .map_err(|e| format!("Stream error: {}", e))?;

            let mut data = Vec::new();
            let mut buf = vec![0u8; 4096];
            loop {
                match recv.read(&mut buf).await {
                    Ok(Some(n)) => {
                        data.extend_from_slice(&buf[..n]);
                        if data.len() > MAX_TEXT_SIZE {
                            return Err("Text too large".to_string());
                        }
                    }
                    Ok(None) => break,
                    Err(e) => return Err(format!("Read error: {:?}", e)),
                }
            }

            let text = String::from_utf8_lossy(&data).to_string();
            let nickname = trusted_peers.get(&remote_str).cloned();

            let entry = TransferHistoryEntry {
                id: uuid::Uuid::new_v4().to_string(),
                direction: "receive".to_string(),
                peer_id: remote_str,
                peer_nickname: nickname,
                transfer_type: "text".to_string(),
                filename: None,
                file_size: Some(data.len() as u64),
                text_content: Some(text),
                status: "complete".to_string(),
                timestamp: epoch_ms(),
                duration_ms: Some(0),
                save_path: None,
            };

            let _ = app_handle.emit("bharatlink-transfer-complete", &entry);
        }

        Ok(())
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

                Ok(NodeInfo {
                    node_id,
                    node_id_short,
                    is_running: true,
                    relay_url: None,
                    local_addrs: Vec::new(),
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

        for (id, nickname) in &self.trusted_peers {
            if !self.peers.contains_key(id) {
                result.push(PeerInfo {
                    node_id: id.clone(),
                    nickname: Some(nickname.clone()),
                    is_local: false,
                    last_seen: 0,
                    is_connected: false,
                    is_trusted: true,
                });
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

        let is_trusted = self.trusted_peers.contains_key(&node_id);
        let peer = PeerInfo {
            node_id: node_id.clone(),
            nickname: nickname.or_else(|| self.trusted_peers.get(&node_id).cloned()),
            is_local: false,
            last_seen: epoch_ms(),
            is_connected: false,
            is_trusted,
        };

        self.peers.insert(node_id, peer.clone());
        Ok(peer)
    }

    pub fn trust_peer(&mut self, node_id: String, nickname: String) -> Result<(), String> {
        self.trusted_peers.insert(node_id.clone(), nickname.clone());

        if let Some(peer) = self.peers.get_mut(&node_id) {
            peer.is_trusted = true;
            peer.nickname = Some(nickname);
        }

        self.save_state()
    }

    pub fn untrust_peer(&mut self, node_id: String) -> Result<(), String> {
        self.trusted_peers.remove(&node_id);

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

        // Send transfer request via META ALPN
        let conn = endpoint
            .connect(peer_key, BHARATLINK_META_ALPN)
            .await
            .map_err(|e| format!("Failed to connect to peer: {}", e))?;

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

        // Emit completion
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
                    status: "complete".to_string(),
                    error: None,
                },
            );
        }

        let nickname = self.trusted_peers.get(&peer_id).cloned();
        let entry = TransferHistoryEntry {
            id: transfer_id.clone(),
            direction: "send".to_string(),
            peer_id: peer_id.clone(),
            peer_nickname: nickname,
            transfer_type: "file".to_string(),
            filename: Some(file_name),
            file_size: Some(file_size),
            text_content: None,
            status: "complete".to_string(),
            timestamp: epoch_ms(),
            duration_ms: None,
            save_path: Some(file_path),
        };
        self.transfer_history.push(entry.clone());
        let _ = self.save_state();

        if let Some(ref handle) = app_handle {
            let _ = handle.emit("bharatlink-transfer-complete", &entry);
        }

        Ok(transfer_id)
    }

    // ── Text transfer ───────────────────────────────────────────────────

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
            .map_err(|e| format!("Failed to connect to peer: {}", e))?;

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

        conn.close(0u8.into(), b"done");

        let text_preview = if text.len() > 100 {
            format!("{}...", &text[..100])
        } else {
            text.clone()
        };

        let nickname = self.trusted_peers.get(&peer_id).cloned();
        let entry = TransferHistoryEntry {
            id: transfer_id.clone(),
            direction: "send".to_string(),
            peer_id: peer_id.clone(),
            peer_nickname: nickname,
            transfer_type: "text".to_string(),
            filename: None,
            file_size: Some(text_bytes.len() as u64),
            text_content: Some(text_preview),
            status: "complete".to_string(),
            timestamp: epoch_ms(),
            duration_ms: None,
            save_path: None,
        };
        self.transfer_history.push(entry.clone());
        let _ = self.save_state();

        if let Some(ref handle) = app_handle {
            let _ = handle.emit("bharatlink-transfer-complete", &entry);
        }

        Ok(transfer_id)
    }

    // ── Transfer management ─────────────────────────────────────────────

    pub fn accept_transfer(&mut self, request_id: String) -> Result<(), String> {
        self.accepted_transfers.insert(request_id.clone());
        self.pending_requests.remove(&request_id);
        Ok(())
    }

    pub fn reject_transfer(&mut self, request_id: String) -> Result<(), String> {
        self.pending_requests.remove(&request_id);
        Ok(())
    }

    pub fn cancel_transfer(&mut self, transfer_id: String) -> Result<(), String> {
        if let Some(state) = self.active_transfers.remove(&transfer_id) {
            let _ = state.cancel_tx.send(());
        }
        Ok(())
    }

    // ── History ─────────────────────────────────────────────────────────

    pub fn get_history(&self) -> Vec<TransferHistoryEntry> {
        self.transfer_history.clone()
    }

    pub fn clear_history(&mut self) -> Result<(), String> {
        self.transfer_history.clear();
        self.save_state()
    }

    // ── Settings ────────────────────────────────────────────────────────

    pub fn get_settings(&self) -> BharatLinkSettings {
        self.settings.clone()
    }

    pub fn update_settings(&mut self, settings: BharatLinkSettings) -> Result<(), String> {
        self.settings = settings;
        self.save_state()
    }
}

// ═══ Utility ════════════════════════════════════════════════════════════

fn epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
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
    mgr.trust_peer(node_id, nickname)
}

#[tauri::command]
pub async fn bharatlink_untrust_peer(
    state: tauri::State<'_, crate::commands::AppState>,
    node_id: String,
) -> Result<(), String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.untrust_peer(node_id)
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
    mgr.accept_transfer(request_id)
}

#[tauri::command]
pub async fn bharatlink_reject_transfer(
    state: tauri::State<'_, crate::commands::AppState>,
    request_id: String,
) -> Result<(), String> {
    let mut mgr = state.bharatlink_manager.lock().await;
    mgr.reject_transfer(request_id)
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
    mgr.update_settings(settings)
}
