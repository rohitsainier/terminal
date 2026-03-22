# bharatlink

P2P file & text sharing from your terminal. No servers, no accounts — just share.

Built on [iroh](https://iroh.computer) QUIC with mDNS discovery, BLAKE3 verification, and end-to-end encryption.

## Install

```bash
cargo install bharatlink
```

## Usage

```bash
# Start a node (foreground daemon)
bharatlink start

# Trust a peer
bharatlink trust <node_id> "Alice"

# Send a file
bharatlink send <node_id> ./photo.jpg

# Send a text message
bharatlink text <node_id> "hello!"

# List peers
bharatlink peers

# Show transfer history
bharatlink history

# Show settings
bharatlink settings
```

## How It Works

1. `bharatlink start` creates a QUIC endpoint with mDNS discovery
2. Peers on the same network are discovered automatically
3. Files are BLAKE3-hashed, chunked, and transferred directly peer-to-peer
4. All connections are encrypted with TLS 1.3

## License

MIT OR Apache-2.0
