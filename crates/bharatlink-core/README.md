# bharatlink-core

P2P file & text sharing engine powered by [iroh](https://iroh.computer) QUIC.

No servers, no accounts — pure peer-to-peer with mDNS local discovery, NAT hole punching, relay fallback, and BLAKE3-verified resumable file transfers.

## Features

- **QUIC + TLS 1.3** encrypted connections (iroh 0.95)
- **mDNS** local network peer discovery
- **BLAKE3** content-addressed, deduplicated file storage (iroh-blobs 0.97)
- **Resumable transfers** — interrupted downloads resume from where they left off
- **Chat-style messaging** — send text, files, clipboard, screenshots
- **Read receipts & typing indicators** — lightweight signal protocol
- **EventSink trait** — plug into any UI (Tauri, CLI, custom)

## Usage

```rust
use bharatlink_core::*;
use std::sync::Arc;

// Implement EventSink for your UI
struct MyEvents;
impl EventSink for MyEvents {
    fn emit(&self, event: BharatLinkEvent) {
        println!("{:?}", event);
    }
}

#[tokio::main]
async fn main() {
    let config_dir = dirs::config_dir().unwrap().join("myapp");
    let mut manager = BharatLinkManager::new(config_dir);

    let events: Arc<dyn EventSink> = Arc::new(MyEvents);
    manager.start(events).await.unwrap();

    // Send a file
    manager.send_file("peer_node_id".into(), "/path/to/file.txt".into()).await.unwrap();

    manager.stop().await.unwrap();
}
```

## CLI

Install the standalone CLI:

```bash
cargo install bharatlink
bharatlink start
bharatlink send <peer_id> ./file.txt
```

## License

MIT OR Apache-2.0
