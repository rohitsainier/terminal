use bharatlink_core::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::sync::Mutex;

/// CLI event sink — renders BharatLink events to the terminal
pub struct CliEventSink {
    multi: MultiProgress,
    progress_bars: Mutex<HashMap<String, ProgressBar>>,
}

impl CliEventSink {
    pub fn new() -> Self {
        Self {
            multi: MultiProgress::new(),
            progress_bars: Mutex::new(HashMap::new()),
        }
    }

    fn get_or_create_pb(&self, transfer_id: &str, filename: &str, total: u64) -> ProgressBar {
        let mut bars = self.progress_bars.lock().unwrap();
        if let Some(pb) = bars.get(transfer_id) {
            return pb.clone();
        }

        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{bar:30.cyan/dim}] {bytes}/{total_bytes} ({eta}) {msg}"
            )
            .unwrap()
            .progress_chars("█▓░"),
        );
        pb.set_message(filename.to_string());
        bars.insert(transfer_id.to_string(), pb.clone());
        pb
    }
}

impl EventSink for CliEventSink {
    fn emit(&self, event: BharatLinkEvent) {
        match event {
            BharatLinkEvent::TransferProgress(p) => {
                let pb = self.get_or_create_pb(&p.transfer_id, &p.filename, p.total_bytes);
                match p.status.as_str() {
                    "connecting" => pb.set_message(format!("⏳ Connecting... {}", p.filename)),
                    "transferring" => {
                        pb.set_position(p.bytes_transferred);
                        pb.set_message(format!("{}", p.filename));
                    }
                    "error" => {
                        pb.abandon_with_message(format!("✗ {} — {}", p.filename,
                            p.error.as_deref().unwrap_or("unknown error")));
                        self.progress_bars.lock().unwrap().remove(&p.transfer_id);
                    }
                    _ => {}
                }
            }
            BharatLinkEvent::TransferComplete(e) => {
                if let Ok(mut bars) = self.progress_bars.lock() {
                    if let Some(pb) = bars.remove(&e.id) {
                        pb.finish_with_message(format!("✓ {} — {}",
                            e.filename.as_deref().unwrap_or("transfer"),
                            e.status));
                    }
                }
                let direction = if e.direction == "send" { "↑ Sent" } else { "↓ Received" };
                let name = e.filename.as_deref()
                    .or(e.text_content.as_deref().map(|t| if t.len() > 40 { &t[..40] } else { t }))
                    .unwrap_or("item");
                let size = e.file_size.map(|s| format_bytes(s)).unwrap_or_default();
                let peer = e.peer_nickname.as_deref().unwrap_or(&e.peer_id[..8.min(e.peer_id.len())]);
                if e.status == "complete" {
                    eprintln!("  {} {} {} ({})", direction, name, size, peer);
                } else {
                    eprintln!("  ✗ {} {} — {} ({})", direction, name, e.status, peer);
                }
            }
            BharatLinkEvent::IncomingRequest(r) => {
                let from = r.from_nickname.as_deref().unwrap_or(&r.from_peer[..8.min(r.from_peer.len())]);
                let what = match r.transfer_type.as_str() {
                    "file" => format!("{} ({})",
                        r.filename.as_deref().unwrap_or("file"),
                        r.file_size.map(format_bytes).unwrap_or_default()),
                    _ => "text message".to_string(),
                };
                eprintln!("\n📥 Incoming from {}: {}", from, what);
                eprintln!("   (auto-accept from trusted peers, or accept via API)");
            }
            BharatLinkEvent::PeerDiscovered(p) => {
                let status = if p.is_connected { "●" } else { "○" };
                let name = p.nickname.as_deref().unwrap_or(&p.node_id_short);
                eprintln!("  {} {} ({})", status, name, p.node_id_short);
            }
            BharatLinkEvent::PeerReconnected { peer_id } => {
                let short = if peer_id.len() >= 8 { &peer_id[..8] } else { &peer_id };
                eprintln!("  ✅ Peer {} is back online", short);
            }
            BharatLinkEvent::Error(e) => {
                let prefix = match e.error_type.as_str() {
                    "reconnection" => "ℹ️ ",
                    "connection" => "⚠️ ",
                    _ => "❌ ",
                };
                eprintln!("  {}{}", prefix, e.message);
            }
            BharatLinkEvent::NodeStatus(_) => {} // Handled by commands
            BharatLinkEvent::Signal(_) => {} // Signals are internal
        }
    }

    fn notify(&self, title: &str, body: &str) {
        eprintln!("🔔 {}: {}", title, body);
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
