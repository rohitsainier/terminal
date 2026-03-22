use crate::events::BharatLinkEvent;
use crate::state::SharedState;
use crate::types::*;
use crate::util::epoch_ms;

use futures_lite::StreamExt;
use iroh::Endpoint;
use iroh::PublicKey;
use iroh_blobs::api::remote::GetProgressItem;
use iroh_blobs::store::fs::FsStore;
use std::path::PathBuf;
use std::time::Instant;

/// Handles incoming file downloads via iroh-blobs (triggered after META request accepted)
#[derive(Debug, Clone)]
pub(crate) struct FileReceiveHandler {
    pub shared: SharedState,
    pub store: FsStore,
    pub endpoint: Endpoint,
}

impl FileReceiveHandler {
    /// Download a blob from a remote peer and save to disk, with progress events
    pub async fn download_blob(
        &self,
        hash_str: &str,
        filename: &str,
        file_size: u64,
        from_peer: &str,
        from_nickname: Option<String>,
        request_id: &str,
    ) -> Result<(), String> {
        tracing::info!("[BharatLink] Starting file download: hash={}, file={}, size={}, from={}",
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
        self.shared.events.emit(BharatLinkEvent::TransferProgress(
            TransferProgress {
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
        ));

        // Connect to the sender's blob protocol endpoint
        tracing::info!("[BharatLink] Connecting to sender {} for blob fetch...", from_peer);
        let conn = self.endpoint
            .connect(sender_key, iroh_blobs::ALPN)
            .await
            .map_err(|e| format!("Failed to connect for file download: {:?}", e))?;
        tracing::info!("[BharatLink] Connected to sender, starting blob fetch...");

        // Emit initial "transferring" progress
        self.shared.events.emit(BharatLinkEvent::TransferProgress(
            TransferProgress {
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
        ));

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

                        self.shared.events.emit(BharatLinkEvent::TransferProgress(
                            TransferProgress {
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
                        ));
                        last_progress_emit = Instant::now();
                    }
                }
                GetProgressItem::Done(_stats) => {
                    tracing::info!("[BharatLink] Download complete for {}", filename);
                    download_ok = true;
                    break;
                }
                GetProgressItem::Error(e) => {
                    let err_msg = format!("Download failed: {:?}", e);
                    tracing::error!("[BharatLink] {}", err_msg);
                    self.shared.events.emit(BharatLinkEvent::TransferProgress(
                        TransferProgress {
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
                    ));
                    return Err(err_msg);
                }
            }
        }

        if !download_ok {
            let err_msg = "Download stream ended unexpectedly".to_string();
            self.shared.events.emit(BharatLinkEvent::TransferProgress(
                TransferProgress {
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
            ));
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
        self.shared.events.emit(BharatLinkEvent::TransferComplete(entry));

        // Send notification for received file
        self.shared.notify("BharatLink: File Received",
            &format!("{} saved to Downloads", filename));

        tracing::info!("[BharatLink] File saved: {} ({} bytes, {}ms)",
            save_path.display(), bytes_written, duration);

        Ok(())
    }
}
