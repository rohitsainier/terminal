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
    Endpoint, PublicKey, SecretKey, Watcher,
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

// ═══ Shared state for protocol handlers ═════════════════════════════════

/// Shared state accessible by protocol handlers (which run outside the manager's mutex)
#[derive(Debug, Clone)]
struct SharedState {
    app_handle: tauri::AppHandle,
    trusted_peers: Arc<TokioMutex<HashMap<String, String>>>,
    transfer_history: Arc<TokioMutex<Vec<TransferHistoryEntry>>>,
    pending_requests: Arc<TokioMutex<HashMap<String, TransferRequest>>>,
    download_dir: Arc<TokioMutex<String>>,
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

// ═══ Protocol Handlers (registered with Router) ══════════════════════════

/// Handles incoming META ALPN connections (file transfer requests)
#[derive(Debug, Clone)]
struct MetaProtocolHandler {
    shared: SharedState,
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

            // Store pending request so accept_transfer can find it
            {
                let mut pending = self.shared.pending_requests.lock().await;
                pending.insert(msg.id.clone(), msg.clone());
            }

            let _ = self.shared.app_handle.emit("bharatlink-incoming-request", &msg);
        }

        Ok(())
    }
}

/// Handles incoming TEXT ALPN connections (direct text sharing)
#[derive(Debug, Clone)]
struct TextProtocolHandler {
    shared: SharedState,
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

        let text = String::from_utf8_lossy(&data).to_string();
        let trusted = self.shared.trusted_peers.lock().await;
        let nickname = trusted.get(&remote_str).cloned();
        drop(trusted);

        let text_preview = if text.len() > 200 {
            format!("{}...", &text[..200])
        } else {
            text.clone()
        };

        let entry = TransferHistoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            direction: "receive".to_string(),
            peer_id: remote_str,
            peer_nickname: nickname,
            transfer_type: "text".to_string(),
            filename: None,
            file_size: Some(data.len() as u64),
            text_content: Some(text_preview),
            status: "complete".to_string(),
            timestamp: epoch_ms(),
            duration_ms: Some(0),
            save_path: None,
        };

        // Persist to shared history
        {
            let mut history = self.shared.transfer_history.lock().await;
            history.push(entry.clone());
        }
        self.shared.save_history().await;

        // Emit to frontend
        let _ = self.shared.app_handle.emit("bharatlink-transfer-complete", &entry);

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
    /// Download a blob from a remote peer and save to disk
    async fn download_blob(
        &self,
        hash_str: &str,
        filename: &str,
        from_peer: &str,
        from_nickname: Option<String>,
        request_id: &str,
    ) -> Result<(), String> {
        eprintln!("[BharatLink] Starting file download: hash={}, file={}, from={}",
            hash_str, filename, from_peer);

        let download_dir = self.shared.download_dir.lock().await.clone();
        let save_dir = PathBuf::from(&download_dir);
        std::fs::create_dir_all(&save_dir)
            .map_err(|e| format!("Cannot create download dir: {}", e))?;

        let save_path = save_dir.join(filename);

        // Parse the blob hash
        let hash: iroh_blobs::Hash = hash_str
            .parse()
            .map_err(|e| format!("Invalid blob hash: {}", e))?;

        // Parse sender's public key and connect via blobs ALPN
        let sender_key: PublicKey = from_peer
            .parse()
            .map_err(|e| format!("Invalid sender ID: {}", e))?;

        // Connect to the sender's blob protocol endpoint
        let conn = self.endpoint
            .connect(sender_key, iroh_blobs::ALPN)
            .await
            .map_err(|e| format!("Failed to connect for file download: {}", e))?;

        // Fetch the blob using the remote API
        let hash_and_format = iroh_blobs::HashAndFormat::raw(hash);
        self.store
            .remote()
            .fetch(conn, hash_and_format)
            .complete()
            .await
            .map_err(|e| format!("Download failed: {}", e))?;

        // Read the downloaded blob and write to file
        use tokio::io::AsyncReadExt;
        let mut reader = self.store.blobs().reader(hash);
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|e| format!("Read blob error: {}", e))?;

        std::fs::write(&save_path, &bytes)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        let entry = TransferHistoryEntry {
            id: request_id.to_string(),
            direction: "receive".to_string(),
            peer_id: from_peer.to_string(),
            peer_nickname: from_nickname,
            transfer_type: "file".to_string(),
            filename: Some(filename.to_string()),
            file_size: Some(bytes.len() as u64),
            text_content: None,
            status: "complete".to_string(),
            timestamp: epoch_ms(),
            duration_ms: None,
            save_path: Some(save_path.to_string_lossy().to_string()),
        };

        // Persist to shared history
        {
            let mut history = self.shared.transfer_history.lock().await;
            history.push(entry.clone());
        }
        self.shared.save_history().await;

        // Emit to frontend
        let _ = self.shared.app_handle.emit("bharatlink-transfer-complete", &entry);

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

    // config
    settings: BharatLinkSettings,
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
            settings: BharatLinkSettings::default(),
            download_dir_shared: Arc::new(TokioMutex::new(String::new())),
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

        // Initialize shared download dir
        self.download_dir_shared = Arc::new(TokioMutex::new(self.settings.download_dir.clone()));
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
            config_dir: self.config_dir.clone(),
        };

        // Create protocol handlers
        let blobs = BlobsProtocol::new(&store, None);
        let meta_handler = MetaProtocolHandler {
            shared: shared.clone(),
        };
        let text_handler = TextProtocolHandler {
            shared: shared.clone(),
        };

        // Store file receive handler for accepting incoming transfers
        self.file_receiver = Some(FileReceiveHandler {
            shared,
            store: store.clone(),
            endpoint: endpoint.clone(),
        });

        // Build router with ALL protocols — blobs + meta + text
        let router = Router::builder(endpoint.clone())
            .accept(iroh_blobs::ALPN, blobs)
            .accept(BHARATLINK_META_ALPN, meta_handler)
            .accept(BHARATLINK_TEXT_ALPN, text_handler)
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

    /// Periodically polls the endpoint for discovered remote nodes and emits events
    async fn peer_discovery_loop(
        endpoint: Endpoint,
        app_handle: tauri::AppHandle,
        trusted_peers: Arc<TokioMutex<HashMap<String, String>>>,
    ) {
        // Track peer state: node_id -> (last_seen_ms, was_connected)
        let mut peer_state: HashMap<String, (u64, bool)> = HashMap::new();

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            // Check if endpoint is still alive
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

            // Check each trusted peer's connectivity
            for (peer_id, nickname) in &trusted_clone {
                if peer_id == &our_id {
                    continue;
                }

                let now = epoch_ms();

                // Check if iroh has addressing info for this peer
                let is_connected = if let Ok(peer_key) = peer_id.parse::<PublicKey>() {
                    // Check latency — returns Some if iroh has seen this peer
                    let has_latency = endpoint.latency(peer_key).is_some();

                    // Check conn_type for active connection pathway
                    let has_path = if let Some(mut conn_type) = endpoint.conn_type(peer_key) {
                        let ct = conn_type.get();
                        !matches!(ct, iroh::endpoint::ConnectionType::None)
                    } else {
                        false
                    };

                    has_latency || has_path
                } else {
                    false
                };

                let prev_state = peer_state.get(peer_id);
                let is_new = prev_state.is_none();
                let status_changed = prev_state
                    .map(|(_, was_connected)| *was_connected != is_connected)
                    .unwrap_or(false);

                // Update tracked state
                let last_seen = if is_connected { now } else {
                    prev_state.map(|(ls, _)| *ls).unwrap_or(0)
                };
                peer_state.insert(peer_id.clone(), (last_seen, is_connected));

                // Emit on first discovery or status change
                if is_new || status_changed {
                    let peer_info = PeerInfo {
                        node_id_short: short_id(peer_id),
                        node_id: peer_id.clone(),
                        nickname: Some(nickname.clone()),
                        is_local: false,
                        last_seen,
                        is_connected,
                        is_trusted: true,
                    };

                    if is_connected || is_new {
                        let _ = app_handle.emit("bharatlink-peer-discovered", &peer_info);
                    }

                    // If peer went offline, emit peer-lost so UI can update
                    if status_changed && !is_connected {
                        let _ = app_handle.emit("bharatlink-peer-discovered", &peer_info);
                    }
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
            .map_err(|e| format!("Failed to connect to peer: {}. Make sure the peer is online and both devices are on the same network, or try adding the peer's full endpoint address.", e))?;

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
            save_path: None,
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
            .map_err(|e| format!("Failed to connect to peer: {}. Make sure the peer is online and both devices are on the same network.", e))?;

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

        let text_preview = if text.len() > 100 {
            format!("{}...", &text[..100])
        } else {
            text.clone()
        };

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
            text_content: Some(text_preview),
            status: "complete".to_string(),
            timestamp: epoch_ms(),
            duration_ms: None,
            save_path: None,
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
                        let from_peer = req.from_peer.clone();
                        let from_nickname = req.from_nickname.clone();
                        let request_id = req.id.clone();

                        let app_handle_for_err = self.app_handle.clone();

                        // Spawn download in background
                        tokio::spawn(async move {
                            if let Err(e) = receiver.download_blob(
                                &hash,
                                &filename,
                                &from_peer,
                                from_nickname.clone(),
                                &request_id,
                            ).await {
                                eprintln!("[BharatLink] File download failed: {}", e);
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
                                };
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
