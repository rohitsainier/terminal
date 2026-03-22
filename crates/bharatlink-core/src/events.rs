use crate::types::*;
use serde::{Deserialize, Serialize};

/// All events emitted by the BharatLink engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BharatLinkEvent {
    Error(BharatLinkError),
    TransferProgress(TransferProgress),
    TransferComplete(TransferHistoryEntry),
    IncomingRequest(TransferRequest),
    Signal(BharatLinkSignal),
    NodeStatus(NodeInfo),
    PeerDiscovered(PeerInfo),
    PeerReconnected { peer_id: String },
}

/// Trait for receiving BharatLink events. Implement for Tauri, CLI, etc.
pub trait EventSink: Send + Sync + 'static {
    fn emit(&self, event: BharatLinkEvent);

    /// Optional notification support. Default is no-op.
    fn notify(&self, _title: &str, _body: &str) {}
}

/// No-op event sink for headless/testing use.
pub struct NullEventSink;

impl EventSink for NullEventSink {
    fn emit(&self, _event: BharatLinkEvent) {}
}
