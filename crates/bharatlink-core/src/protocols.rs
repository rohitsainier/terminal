use crate::events::BharatLinkEvent;
use crate::receive::FileReceiveHandler;
use crate::state::SharedState;
use crate::types::*;
use crate::util::epoch_ms;

use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use iroh::Endpoint;
use iroh_blobs::store::fs::FsStore;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

// ═══ Protocol Handlers (registered with Router) ══════════════════════════

/// Handles incoming META ALPN connections (file transfer requests)
#[derive(Debug, Clone)]
pub(crate) struct MetaProtocolHandler {
    pub shared: SharedState,
    pub settings: Arc<TokioMutex<BharatLinkSettings>>,
    pub store: FsStore,
    pub endpoint_for_accept: Endpoint,
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

            tracing::info!("[BharatLink] Incoming transfer request: type={}, id={}, from={}",
                msg.transfer_type, msg.id, msg.from_peer);

            // Check auto-accept settings
            let settings = self.settings.lock().await;
            let is_trusted = {
                let trusted = self.shared.trusted_peers.lock().await;
                trusted.contains_key(&msg.from_peer)
            };

            // Skip if accept_from_trusted_only and sender is not trusted
            if settings.accept_from_trusted_only && !is_trusted {
                tracing::info!("[BharatLink] Rejecting request from untrusted peer: {}", msg.from_peer);
                return Ok(());
            }

            let should_auto_accept = settings.auto_accept_from_trusted && is_trusted;
            drop(settings);

            if should_auto_accept {
                tracing::info!("[BharatLink] Auto-accepting transfer from trusted peer: {}", msg.from_peer);

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

                        tokio::spawn(async move {
                            // Small delay to let META connection close cleanly
                            tokio::time::sleep(std::time::Duration::from_millis(300)).await;

                            if let Err(e) = receiver.download_blob(
                                &hash, &filename, file_size,
                                &from_peer, from_nickname.clone(), &request_id,
                            ).await {
                                tracing::error!("[BharatLink] Auto-accept download failed: {}", e);
                                receiver.shared.emit_error("transfer", &format!("Auto-accept download failed: {}", e),
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
                                receiver.shared.events.emit(BharatLinkEvent::TransferComplete(fail_entry));
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
                self.shared.events.emit(BharatLinkEvent::IncomingRequest(msg.clone()));

                // Notification for incoming file request
                let notif_body = format!("{} wants to send you {}",
                    msg.from_nickname.as_deref().unwrap_or("Unknown peer"),
                    msg.filename.as_deref().unwrap_or("a file"));
                self.shared.notify("BharatLink: Incoming File", &notif_body);
            }
        }

        Ok(())
    }
}

/// Handles incoming TEXT ALPN connections (direct text sharing)
#[derive(Debug, Clone)]
pub(crate) struct TextProtocolHandler {
    pub shared: SharedState,
    pub endpoint_for_signal: Endpoint,
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
        self.shared.events.emit(BharatLinkEvent::TransferComplete(entry.clone()));

        // Send notification for received text
        let preview = if text.len() > 80 { &text[..80] } else { &text };
        self.shared.notify("BharatLink: New Message", preview);

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
pub(crate) struct SignalProtocolHandler {
    pub shared: SharedState,
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
                self.shared.events.emit(BharatLinkEvent::Signal(signal));
            }
        }

        Ok(())
    }
}
