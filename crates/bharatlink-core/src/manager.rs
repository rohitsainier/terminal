use crate::events::{BharatLinkEvent, EventSink};
use crate::protocols::{MetaProtocolHandler, SignalProtocolHandler, TextProtocolHandler};
use crate::receive::FileReceiveHandler;
use crate::state::{SharedState, TransferState};
use crate::storage;
use crate::types::*;
use crate::util::{epoch_ms, short_id};

use iroh::discovery::mdns::MdnsDiscovery;
use iroh::protocol::Router;
use iroh::{Endpoint, PublicKey};
use iroh_blobs::store::fs::FsStore;
use iroh_blobs::BlobsProtocol;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

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

    // event sink for emitting events
    events: Option<Arc<dyn EventSink>>,
}

impl BharatLinkManager {
    pub fn new(config_dir: PathBuf) -> Self {
        let bl_config_dir = config_dir.join("bharatlink");

        // Blob store lives alongside config — each instance gets its own
        let data_dir = bl_config_dir.clone();

        let (settings, trusted_peers, history) = storage::load_state(&bl_config_dir);

        let mut mgr = Self {
            endpoint: None,
            router: None,
            store: None,
            peers: HashMap::new(),
            trusted_peers_shared: Arc::new(TokioMutex::new(trusted_peers)),
            active_transfers: HashMap::new(),
            transfer_history_shared: Arc::new(TokioMutex::new(history)),
            pending_requests_shared: Arc::new(TokioMutex::new(HashMap::new())),
            accepted_transfers: std::collections::HashSet::new(),
            file_receiver: None,
            failed_transfers_shared: Arc::new(TokioMutex::new(Vec::new())),
            settings,
            settings_shared: Arc::new(TokioMutex::new(BharatLinkSettings::default())),
            download_dir_shared: Arc::new(TokioMutex::new(String::new())),
            config_dir: bl_config_dir,
            data_dir,
            events: None,
        };

        // Sync shared state
        mgr.download_dir_shared = Arc::new(TokioMutex::new(mgr.settings.download_dir.clone()));
        mgr.settings_shared = Arc::new(TokioMutex::new(mgr.settings.clone()));

        // Auto-generate device name from hostname if not set
        if mgr.settings.device_name.is_none() {
            let hostname = gethostname::gethostname().to_string_lossy().to_string();
            mgr.settings.device_name = Some(hostname);
            let _ = storage::save_state(
                &mgr.config_dir, &mgr.settings,
                &mgr.trusted_peers_shared, &mgr.transfer_history_shared,
            );
        }

        mgr
    }

    // ── Persistence ─────────────────────────────────────────────────────

    fn save_state(&self) -> Result<(), String> {
        storage::save_state(
            &self.config_dir, &self.settings,
            &self.trusted_peers_shared, &self.transfer_history_shared,
        )
    }

    // ── Node lifecycle ──────────────────────────────────────────────────

    pub async fn start(&mut self, events: Arc<dyn EventSink>) -> Result<NodeInfo, String> {
        if self.endpoint.is_some() {
            return self.get_node_info();
        }

        self.events = Some(events.clone());

        let secret_key = storage::load_or_create_secret_key(&self.config_dir)?;

        // Create blob store on disk
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Failed to create data dir: {}", e))?;
        let store = FsStore::load(self.data_dir.join("blobs"))
            .await
            .map_err(|e| format!("Failed to create blob store: {}", e))?;

        // Build endpoint with full discovery stack
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

        // Add mDNS discovery for local network
        let mdns = MdnsDiscovery::builder()
            .build(endpoint.id())
            .map_err(|e| format!("Failed to start mDNS: {}", e))?;
        endpoint.discovery().add(mdns);

        // Update shared download dir
        *self.download_dir_shared.lock().await = self.settings.download_dir.clone();

        // Create shared state for protocol handlers
        let shared = SharedState {
            events: events.clone(),
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

        // Build router with ALL protocols
        let router = Router::builder(endpoint.clone())
            .accept(iroh_blobs::ALPN, blobs)
            .accept(BHARATLINK_META_ALPN, meta_handler)
            .accept(BHARATLINK_TEXT_ALPN, text_handler)
            .accept(BHARATLINK_SIGNAL_ALPN, signal_handler)
            .spawn();

        // Spawn peer discovery polling loop
        let ep_for_discovery = endpoint.clone();
        let events_for_discovery = events.clone();
        let trusted_for_discovery = self.trusted_peers_shared.clone();
        tokio::spawn(async move {
            Self::peer_discovery_loop(ep_for_discovery, events_for_discovery, trusted_for_discovery).await;
        });

        self.endpoint = Some(endpoint);
        self.router = Some(router);
        self.store = Some(store);

        self.get_node_info()
    }

    /// Periodically polls the endpoint for discovered remote nodes and emits events.
    async fn peer_discovery_loop(
        endpoint: Endpoint,
        events: Arc<dyn EventSink>,
        trusted_peers: Arc<TokioMutex<HashMap<String, String>>>,
    ) {
        let mut peer_state: HashMap<String, (u64, bool)> = HashMap::new();
        let mut cycle_count: u32 = 0;
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
            events.emit(BharatLinkEvent::NodeStatus(status));

            // Active probe: attempt a lightweight QUIC connect to each trusted peer
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
                                conn.close(0u32.into(), b"probe");
                                true
                            }
                            _ => false,
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
                    events.emit(BharatLinkEvent::Error(BharatLinkError {
                        error_type: "reconnection".to_string(),
                        message: format!("{} is back online", nickname),
                        peer_id: Some(peer_id.clone()),
                        transfer_id: None,
                        timestamp: now,
                    }));
                    events.emit(BharatLinkEvent::PeerReconnected { peer_id: peer_id.clone() });
                } else if status_changed && !is_connected && was_connected {
                    events.emit(BharatLinkEvent::Error(BharatLinkError {
                        error_type: "connection".to_string(),
                        message: format!("{} went offline", nickname),
                        peer_id: Some(peer_id.clone()),
                        transfer_id: None,
                        timestamp: now,
                    }));
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
                    events.emit(BharatLinkEvent::PeerDiscovered(peer_info));
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

        // Emit stopped status
        if let Some(ref events) = self.events {
            events.emit(BharatLinkEvent::NodeStatus(NodeInfo {
                node_id: String::new(),
                node_id_short: String::new(),
                is_running: false,
                relay_url: None,
                local_addrs: vec![],
                discovered_peers: 0,
            }));
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
        let endpoint = self.endpoint.as_ref().ok_or("Node not running")?.clone();
        let store = self.store.as_ref().ok_or("Blob store not initialized")?.clone();

        let path = PathBuf::from(&file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let transfer_id = uuid::Uuid::new_v4().to_string();
        let events = self.events.clone();

        // Emit initial progress
        if let Some(ref events) = events {
            events.emit(BharatLinkEvent::TransferProgress(
                TransferProgress {
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
            ));
        }

        // Add file to blob store
        let abs_path = std::fs::canonicalize(&path)
            .map_err(|e| format!("Cannot resolve path: {}", e))?;
        let tag_info = store.blobs().add_path(abs_path).await
            .map_err(|e| format!("Failed to add file to store: {}", e))?;
        let hash = tag_info.hash;

        // Parse peer public key
        let peer_key: PublicKey = peer_id.parse()
            .map_err(|e| format!("Invalid peer ID: {}", e))?;

        // Connect
        let conn = endpoint.connect(peer_key, BHARATLINK_META_ALPN).await
            .map_err(|e| {
                if let Some(ref ev) = events {
                    ev.emit(BharatLinkEvent::Error(BharatLinkError {
                        error_type: "connection".to_string(),
                        message: format!("Failed to send file: {}", e),
                        peer_id: Some(peer_id.clone()),
                        transfer_id: Some(transfer_id.clone()),
                        timestamp: epoch_ms(),
                    }));
                }
                format!("Failed to connect to peer: {}. Make sure the peer is online and both devices are on the same network.", e)
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

        let msg_bytes = serde_json::to_vec(&request).map_err(|e| format!("Serialize error: {}", e))?;
        let (mut send, _recv) = conn.open_bi().await.map_err(|e| format!("Stream error: {}", e))?;
        send.write_all(&msg_bytes).await.map_err(|e| format!("Write error: {}", e))?;
        send.finish().map_err(|e| format!("Finish error: {}", e))?;

        // Emit "transferring" progress
        if let Some(ref events) = events {
            events.emit(BharatLinkEvent::TransferProgress(
                TransferProgress {
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
            ));
        }

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

        if let Some(ref events) = events {
            events.emit(BharatLinkEvent::TransferComplete(entry));
        }

        Ok(transfer_id)
    }

    pub async fn send_text(
        &mut self,
        peer_id: String,
        text: String,
    ) -> Result<String, String> {
        let endpoint = self.endpoint.as_ref().ok_or("Node not running")?.clone();
        let transfer_id = uuid::Uuid::new_v4().to_string();
        let events = self.events.clone();

        let peer_key: PublicKey = peer_id.parse()
            .map_err(|e| format!("Invalid peer ID: {}", e))?;

        let conn = endpoint.connect(peer_key, BHARATLINK_TEXT_ALPN).await
            .map_err(|e| {
                if let Some(ref ev) = events {
                    ev.emit(BharatLinkEvent::Error(BharatLinkError {
                        error_type: "connection".to_string(),
                        message: format!("Failed to send text: {}", e),
                        peer_id: Some(peer_id.clone()),
                        transfer_id: Some(transfer_id.clone()),
                        timestamp: epoch_ms(),
                    }));
                }
                format!("Failed to connect to peer: {}. Make sure the peer is online.", e)
            })?;

        let mut send = conn.open_uni().await.map_err(|e| format!("Stream error: {}", e))?;
        let text_bytes = text.as_bytes();
        send.write_all(text_bytes).await.map_err(|e| format!("Write error: {}", e))?;
        send.finish().map_err(|e| format!("Finish error: {}", e))?;

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
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

        if let Some(ref events) = events {
            events.emit(BharatLinkEvent::TransferComplete(entry));
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
                        let events = self.events.clone();

                        tokio::spawn(async move {
                            if let Err(e) = receiver.download_blob(
                                &hash, &filename, file_size,
                                &from_peer, from_nickname.clone(), &request_id,
                            ).await {
                                tracing::error!("[BharatLink] File download failed: {}", e);
                                if let Some(ref ev) = events {
                                    ev.emit(BharatLinkEvent::Error(BharatLinkError {
                                        error_type: "transfer".to_string(),
                                        message: format!("Download failed: {}", e),
                                        peer_id: Some(from_peer.clone()),
                                        transfer_id: Some(request_id.clone()),
                                        timestamp: epoch_ms(),
                                    }));
                                }
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
                                {
                                    let mut history = receiver.shared.transfer_history.lock().await;
                                    history.push(fail_entry.clone());
                                }
                                receiver.shared.save_history().await;
                                if let Some(ref ev) = events {
                                    ev.emit(BharatLinkEvent::TransferComplete(fail_entry));
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
        let entry = {
            let history = self.transfer_history_shared.lock().await;
            history.iter().find(|e| e.id == transfer_id && e.status == "failed").cloned()
        };

        let entry = entry.ok_or("Transfer not found or not in failed state")?;

        if entry.direction == "receive" {
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
                let events = self.events.clone();

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
                        tracing::error!("[BharatLink] Retry download failed: {}", e);
                        if let Some(ref ev) = events {
                            ev.emit(BharatLinkEvent::Error(BharatLinkError {
                                error_type: "transfer".to_string(),
                                message: format!("Retry failed: {}", e),
                                peer_id: Some(from_peer.clone()),
                                transfer_id: Some(new_id_clone.clone()),
                                timestamp: epoch_ms(),
                            }));
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
                            ev.emit(BharatLinkEvent::TransferComplete(fail_entry));
                        }
                    }
                });

                Ok(new_id)
            } else {
                Err("File receiver not initialized — is the node running?".to_string())
            }
        } else if entry.direction == "send" {
            let file_path = entry.save_path.as_ref().ok_or("Cannot retry send: original file path not stored")?;
            self.send_file(entry.peer_id.clone(), file_path.clone()).await
        } else {
            Err("Cannot retry this transfer type".to_string())
        }
    }

    // ── History ─────────────────────────────────────────────────────────

    pub fn get_pending_requests(&self) -> Vec<TransferRequest> {
        self.pending_requests_shared.try_lock()
            .map(|p| p.values().cloned().collect())
            .unwrap_or_default()
    }

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
            tracing::info!("[BharatLink] Sending file {}/{} from batch {}: {}",
                i + 1, total, batch_id, file_path);
            match self.send_file(peer_id.clone(), file_path.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("[BharatLink] Batch file send error: {}", e);
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

        let result = self.send_file(peer_id, screenshot_path.to_string_lossy().to_string()).await;
        let _ = std::fs::remove_file(&screenshot_path);
        result
    }

    // ── Clipboard operations ────────────────────────────────────────────

    pub async fn send_clipboard_text(
        &mut self,
        peer_id: String,
        clipboard_text: String,
    ) -> Result<String, String> {
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

        let peer_key: PublicKey = peer_id.parse()
            .map_err(|e| format!("Invalid peer ID: {}", e))?;

        let signal = BharatLinkSignal {
            signal_type,
            message_id,
            from_peer: endpoint.id().to_string(),
            timestamp: epoch_ms(),
        };

        let data = serde_json::to_vec(&signal).map_err(|e| format!("Serialize error: {}", e))?;

        let conn = endpoint.connect(peer_key, BHARATLINK_SIGNAL_ALPN).await
            .map_err(|e| format!("Signal connect error: {}", e))?;

        let mut send = conn.open_uni().await.map_err(|e| format!("Stream error: {}", e))?;
        send.write_all(&data).await.map_err(|e| format!("Write error: {}", e))?;
        send.finish().map_err(|e| format!("Finish error: {}", e))?;

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        Ok(())
    }
}
