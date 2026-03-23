# CLAUDE.md - Flux Terminal

## Project Overview

Flux Terminal is an AI-powered, GPU-accelerated terminal emulator with cyberpunk aesthetics. Built with **Tauri 2.0** (Rust backend) + **SolidJS** (TypeScript frontend) + **xterm.js** (terminal rendering).

**BharatLink** is the P2P file & text sharing engine, extracted as a standalone library (`bharatlink-core`) and CLI (`bharatlink`), both published on crates.io.

## Tech Stack

- **Frontend:** SolidJS, TypeScript, Vite, xterm.js
- **Backend:** Rust (edition 2021), Tauri 2.0, Tokio, portable-pty
- **P2P:** iroh 0.95 (QUIC + mDNS), iroh-blobs 0.97 (content-addressed transfers), futures-lite 2, tauri-plugin-notification
- **CLI:** clap 4, indicatif (progress bars), tracing
- **Package Manager:** npm
- **Build Tool:** Vite (port 1420)
- **Published Crates:** `bharatlink-core` (library), `bharatlink` (CLI binary)

## Commands

```bash
# ─── Flux Terminal (Desktop App) ───
npm install                # Install frontend dependencies
npm run dev                # Start Vite dev server (port 1420)
npm run tauri dev          # Start full Tauri app with hot reload
npm run build              # Build frontend to dist/
npm run tauri build        # Build distributable desktop app

# ─── BharatLink CLI ───
cargo run -p bharatlink -- start                    # Start interactive P2P node
cargo run -p bharatlink -- send file <id> ./file    # Send a file
cargo run -p bharatlink -- send text <id> "hello"   # Send a text message
cargo run -p bharatlink -- trust <id> "Name"        # Trust a peer
cargo run -p bharatlink -- receive                  # One-shot receive mode
cargo run -p bharatlink -- --help                   # Show all commands

# ─── Publishing ───
cargo publish -p bharatlink-core    # Publish library to crates.io
cargo publish -p bharatlink         # Publish CLI to crates.io (after core)
git tag cli-v0.x.x && git push --tags  # Trigger GitHub Actions release
```

There are no test or lint commands configured.

## Project Structure

```
crates/
  bharatlink-core/            # P2P library (crates.io: bharatlink-core)
    src/
      lib.rs                  # Public API exports
      manager.rs              # BharatLinkManager — node lifecycle, send, connect
      protocols.rs            # MetaProtocolHandler, TextProtocolHandler, SignalProtocolHandler
      receive.rs              # FileReceiveHandler — blob download with streaming progress
      state.rs                # SharedState (Arc<TokioMutex<>>) for cross-handler access
      storage.rs              # JSON persistence (history, peers, settings, secret key)
      types.rs                # All shared types (TransferHistoryEntry, PeerInfo, etc.)
      events.rs               # EventSink trait — decouples from Tauri for CLI/library use
      util.rs                 # Helpers (epoch_ms, short_id, format_bytes)
  bharatlink-cli/             # CLI binary (crates.io: bharatlink)
    src/main.rs               # clap CLI: start, send, receive, trust, peers, history
src/                          # Frontend (SolidJS + TypeScript)
  components/                 # UI components (Terminal, AIBar, Settings, Sidebar, etc.)
    netops/                   # NETOPS Dashboard (network operations + security tools)
      index.ts                # Barrel export
      types.ts                # Interfaces, NetopsTool union (28 tools), NetopsStore type
      useNetopsData.ts        # Signals, runTool(), history management
      TopBar.tsx              # Top bar (logo, LIVE badge, UTC clock)
      ToolPanel.tsx           # Left panel (28 tool rows with icons)
      ResultPanel.tsx         # Center panel (input bar + result renderers per tool)
      InfoPanel.tsx           # Right panel (tool help + scan history)
      NetopsDashboard.tsx     # Orchestrator (keyboard, layout)
    bharatlink/               # BharatLink P2P file & text sharing
      index.ts                # Barrel export
      types.ts                # Interfaces (NodeInfo, PeerInfo, Transfer*, Settings, BharatLinkStore)
      useBharatLinkData.ts    # Signals, Tauri invoke/listen, event handlers
      TopBar.tsx              # Top bar (logo, node status, start/stop, UTC clock)
      PeerPanel.tsx           # Left panel (discovered + trusted peers, add peer)
      TransferPanel.tsx       # Center panel (chat-style UI, file/text send, progress, history)
      InfoPanel.tsx           # Right panel (node info, stats, settings)
      BharatLinkDashboard.tsx # Orchestrator (keyboard, layout)
  hooks/                      # SolidJS hooks (useTheme, useTerminal, useAI)
  effects/                    # Visual effects (CRT, Glow, MatrixRain, Particles, Hologram)
  themes/                     # Theme JSON files + ThemeEngine
  styles/                     # Global CSS (global.css, terminal.css, effects.css, netops.css, bharatlink.css)
src-tauri/                    # Rust backend (Tauri adapter)
  src/main.rs                 # Tauri app setup
  src/commands.rs             # Tauri IPC commands (~1500 lines)
  src/ai.rs                   # AI provider logic (Ollama, OpenAI, Anthropic)
  src/pty.rs                  # PTY session management
  src/terminal.rs             # Terminal session management
  src/config.rs               # Configuration management
  src/snippets.rs             # Snippet storage
  src/ssh.rs                  # SSH connections
  src/mcp.rs                  # Model Context Protocol
  src/netops.rs               # NETOPS commands (28 network/security tools)
  src/bharatlink.rs           # Thin adapter: Tauri commands → bharatlink-core (TauriEventSink bridges events)
  src/wifi_scan.swift         # CoreWLAN WiFi scanner (embedded via include_str!)
install.sh                    # One-liner installer (curl | sh) for pre-built CLI binaries
.github/workflows/
  build.yml                   # Flux Terminal desktop app CI
  release-cli.yml             # BharatLink CLI cross-platform release (triggered by cli-v* tags)
```

## Code Conventions

### TypeScript / SolidJS
- Components: PascalCase functional components with typed props interfaces
- Hooks: `use` prefix, return object with signals and actions
- State: SolidJS `createSignal()`, `createEffect()`, `createMemo()`
- Control flow: `<Show>`, `<For>` components (not ternaries/map)
- File naming: PascalCase for components (.tsx), camelCase for utilities (.ts)

### Rust
- Modules: snake_case filenames
- Tauri commands: `#[tauri::command]` attribute, return `Result<T, String>`
- State: `tauri::State<AppState>` with Mutex-protected fields
- Section comments: `// ─── Section Name ────`

### CSS
- Custom properties: `--variable-name`
- Classes: kebab-case
- Colors: hex format
- Transitions: 0.15s for interactive elements
- Dashboards use global theme variables (`--accent`, `--bg`, `--fg`, `--border`, `--accent-dim`, `--panel-bg`, `--glow-color`, `--selection`) so they adapt to the selected theme

## Architecture Notes

- **Frontend-Backend IPC:** Tauri `invoke()` calls to `#[tauri::command]` Rust functions
- **State:** AppState holds pty_manager, ai_engine, config, snippet_manager, ssh_manager, session_manager, mcp_manager, bharatlink_manager (all Mutex-wrapped; bharatlink_manager uses `tokio::sync::Mutex` since iroh is fully async)
- **Terminal:** portable-pty for PTY sessions, xterm.js for rendering, event-based output streaming
- **AI Providers:** Ollama, OpenAI, Anthropic — configurable in Settings
- **Themes:** 6 JSON theme files loaded by ThemeEngine. `applyThemeToDOM()` sets CSS custom properties on `:root` (`--accent`, `--bg`, `--fg`, `--border`, `--accent-dim`, `--panel-bg`, `--glow-color`, `--selection`, `--cursor-color`, `--tab-active`, `--status-bg`). All dashboards (NETOPS, BharatLink) reference these global variables so they automatically match the selected theme.
- **Config:** Stored in OS config directory via `dirs` crate, hot-reloadable
- **NETOPS Dashboard:** 28 network/security tools (⌘⇧N). Architecture: `useNetopsData` hook → `NetopsStore` object → props. Rust backend uses `tokio::process::Command` for system tools (dig, whois, arp, traceroute, openssl, system_profiler) and `reqwest` for HTTP-based tools (ping, headers, geoip, threat intel, dir brute, web fingerprint, WAF detect, vuln scan). WiFi scanning via CoreWLAN Swift script (`wifi_scan.swift` embedded with `include_str!`). Caching for whois, geoip, & threat intel (1hr TTL). Port scan and service scan use concurrent `tokio::net::TcpStream::connect()` with banner grabbing. Subdomain enum uses batched `dig` lookups. Persistent JSON storage in `dirs::config_dir()/flux-terminal/` for WiFi baselines and incidents. Embedded wordlists for subdomain brute (~90 entries) and directory brute (~80 paths). WPA handshake analyzer uses `system_profiler SPAirPortDataType` for connection details + CoreWLAN scan for SSID matching (macOS Sequoia compatible). Log reports saved to `~/Downloads/` with detailed protocol explanations. Tools split into 3 categories:
  - **Network:** ping, port scan, DNS lookup, WHOIS, WiFi scan, WiFi auth monitor, HTTP headers, SSL inspect, IP geolocation, ARP table, subnet calc, reverse DNS, traceroute
  - **Security:** traffic anomaly detection, rogue AP detection, system log viewer, threat intelligence, security score, incident tracking, WPA handshake analyzer (with downloadable log reports)
  - **Offensive/Kali-style:** service scan (banner grab), subdomain enum, directory brute force, web fingerprint, WAF detection, web vuln scan (nikto-lite), hash identifier, cipher scan (TLS enum)
- **BharatLink (P2P Share):** Sovereign peer-to-peer file & text sharing. No servers, no accounts — pure QUIC + mDNS. **Extracted as a workspace crate** (`bharatlink-core`) so both Flux Terminal and the standalone CLI share the same engine. Architecture: `BharatLinkManager` in `bharatlink-core` handles all P2P logic; Flux Terminal's `src-tauri/bharatlink.rs` is a thin adapter that implements `EventSink` trait to bridge events to Tauri's `app_handle.emit()`. The CLI implements its own `EventSink` that prints to stdout. Custom ALPNs: `bharatlink/meta/1` (transfer request handshake), `bharatlink/text/1` (direct text sharing + clipboard sync), `bharatlink/signal/1` (read receipts + typing indicators). All connections encrypted via QUIC + TLS 1.3 (iroh default). Events: `bharatlink-peer-discovered`, `bharatlink-peer-lost`, `bharatlink-incoming-request`, `bharatlink-transfer-progress`, `bharatlink-transfer-complete`, `bharatlink-node-status`, `bharatlink-signal`, `bharatlink-error`, `bharatlink-peer-reconnected`. Features: chat-style UI (Flux Terminal), interactive daemon (CLI), drag & drop file send, multi-file/folder transfer, screenshot share, clipboard sync, QR code pairing, device names, peer online/offline status with active heartbeat probing, read receipts, typing indicator, image preview in chat, link preview cards, auto-accept from trusted peers, native OS notifications, error handling UX with inline chat error bubbles, transfer retry for failed downloads.
- **BharatLink CLI:** Standalone binary published as `bharatlink` on crates.io. Uses `bharatlink-core` directly (no Tauri dependency). Interactive daemon mode (`bharatlink start`) with inline `send file/text` commands. One-shot `bharatlink receive` waits for a single file and exits. Progress bars via `indicatif`. Cross-network transfers use `EndpointAddr` with relay URL hints for iroh to resolve peers via relay servers.
- **BharatLink Core Library:** Published as `bharatlink-core` on crates.io. Exports `BharatLinkManager`, all types, and the `EventSink` trait. Consumers implement `EventSink` to receive P2P events. No Tauri dependency — pure async Rust.
- **Cross-network connectivity:** `connect_with_retry()` builds an `EndpointAddr` with the local node's relay URL as a hint, so iroh can reach peers on different networks via relay servers. Without the relay hint, iroh only has the peer's `PublicKey` and can't resolve their address, causing "No addressing information available" errors.

### BharatLink Technical Details

- **Content-addressed deduplication:** iroh-blobs stores files as BLAKE3 chunks in the blob store. Same content = same hash. Re-sending the same file costs zero bandwidth — the receiver's local blob store already has the chunks.
- **Streaming progress:** File receive uses `GetProgress::stream()` from iroh-blobs instead of `.complete()`, yielding `GetProgressItem::Progress(bytes_received)` events throttled to ~10/sec. Frontend shows real-time progress bar with bytes/speed/percentage.
- **Active transfer rendering:** Active transfers are rendered separately from chat history in `TransferPanel.tsx` to avoid full list re-renders on progress ticks. The `blnk-chat-row-active` CSS class disables the `blnk-chatFadeIn` animation to prevent bubble flashing during progress updates.
- **Signal protocol:** `bharatlink/signal/1` ALPN handles lightweight signals (read receipts, typing indicators). `SignalProtocolHandler` accepts uni streams with JSON `BharatLinkSignal` payloads. Text handler auto-sends "delivered" signal back to sender after receiving a message. Frontend sends "typing"/"stop_typing" signals throttled to 1 per 2 seconds, with 4-second auto-clear timeout.
- **Auto-accept:** `MetaProtocolHandler` checks `settings_shared` (Arc<TokioMutex<BharatLinkSettings>>) on every incoming request. If `auto_accept_from_trusted` is enabled and sender is in trusted_peers, it spawns `FileReceiveHandler::download_blob()` directly without emitting to frontend. If `accept_from_trusted_only` is enabled and sender is untrusted, the request is silently dropped.
- **Error handling UX:** `BharatLinkError` struct emitted via `bharatlink-error` event for inline chat error bubbles. `emit_error()` helper categorizes errors (connection, transfer, timeout, system, reconnection). Frontend renders errors as red-tinted chat bubbles with contextual messages. Failed transfers persist `blob_hash` for retry support.
- **Transfer retry:** Failed file transfers store `blob_hash` in `TransferHistoryEntry`. `retry_transfer` command looks up the failed entry in shared history, re-initiates `download_blob()` with the original hash, and updates status. Both auto-accept and manual-accept fail paths now persist entries to shared history before emitting to frontend (fixes "Transfer not found" on retry).
- **Active peer heartbeat:** Peer online/offline detection uses active QUIC connection probes instead of stale `conn_type()` metadata. Every 3rd discovery cycle (~15s), trusted peers are probed via `endpoint.connect()` on the signal ALPN with a 4-second timeout. Successful probe = online, timeout/failure = offline. Prevents false "online" status for disconnected peers.
- **Reconnection detection:** `peer_discovery_loop` tracks `previous_online` state per peer. When a peer transitions from offline→online, emits `bharatlink-peer-reconnected` event. Frontend shows a green "✅ Peer is back online" system message in chat.
- **Native notifications:** `send_notification()` helper uses `tauri-plugin-notification` for OS-level notifications on incoming files/messages. Gated by `BharatLinkSettings.notifications_enabled` (default true). Non-blocking `try_lock` on settings mutex.
- **Sent file display:** Frontend only shows "Saved: filename" for received files (`!isSent` guard), not sent files — `save_path` on sent entries stores the source path for retry but isn't user-facing.
- **Image preview:** `TransferPanel.tsx` detects image extensions (png/jpg/gif/webp/etc.) in file transfer entries and renders inline `<img>` using `convertFileSrc()` from Tauri for local file access. Images show only for received files with a `save_path`.
- **Link preview:** URL regex extracts `https?://` URLs from text messages. Renders clickable links inline plus a compact preview card with domain name and truncated URL, styled with left-accent border.
- **SharedState pattern:** Protocol handlers (MetaProtocolHandler, TextProtocolHandler, SignalProtocolHandler) run in separate tasks outside the manager's mutex. Shared state (history, pending_requests, trusted_peers, download_dir, settings) uses `Arc<TokioMutex<>>` for cross-handler access.
- **EventSink trait:** Decouples bharatlink-core from Tauri. The trait defines `emit(event_name, payload)`. Flux Terminal implements `TauriEventSink` (wraps `app_handle.emit()`), CLI implements `CliEventSink` (prints to stdout with colored formatting). This allows the same P2P engine to power both GUI and CLI.
- **QUIC stream lifecycle:** `send.finish()` signals EOF but dropping the connection immediately can lose data. Text sends use a 500ms `tokio::time::sleep` before dropping to ensure QUIC flushes all data.
- **Cross-network relay hints:** `connect_with_retry()` builds `EndpointAddr::new(peer_key).with_relay_url(our_relay_url)` so iroh knows to use the relay server for peers on different networks. Without this, iroh only has the PublicKey and DNS/pkarr resolution may not find the peer in time.
- **Storage locations (macOS):**
  - Flux Terminal: `~/Library/Application Support/flux-terminal/bharatlink/` (config + blobs)
  - CLI: `~/Library/Application Support/bharatlink/` (separate config + blobs, no conflict)
- **Key imports:**
  ```rust
  use iroh::{
      discovery::mdns::MdnsDiscovery,
      endpoint::Connection,
      protocol::{AcceptError, ProtocolHandler, Router},
      EndpointAddr, Endpoint, PublicKey, SecretKey, Watcher,
  };
  use iroh_blobs::{api::remote::GetProgressItem, store::fs::FsStore, BlobsProtocol};
  use futures_lite::StreamExt;
  ```
- **Endpoint setup:** Uses `Endpoint::builder()` (N0 preset) which includes PkarrPublisher (publishes to dns.iroh.link), DnsDiscovery (resolves via DNS), and default relay servers — required for cross-network connectivity. `Endpoint::empty_builder()` only works for local-network discovery.
- **Router pattern:** All protocols (blobs, meta, text, signal) are registered with a single `Router` — no separate accept loops. The router dispatches incoming connections by ALPN to the correct `ProtocolHandler`.
- **SecretKey generation:** `SecretKey::generate(&mut rand::rng())` (rand 0.9 API)
- **CLI interactive daemon:** `bharatlink start` runs an event loop that: (1) starts the node, (2) spawns a background task listening for P2P events via `EventSink`, (3) reads stdin for interactive commands (`send file/text`, `accept`, `reject`, `peers`, `history`, `quit`). One-shot `bharatlink receive` starts a node, waits for a single incoming file, saves it, and exits.
- **Publishing workflow:** `bharatlink-core` must be published first (it's a dependency). CLI (`bharatlink`) depends on it via crates.io version (not path) for publishing. GitHub Actions `release-cli.yml` builds cross-platform binaries on `cli-v*` tags: macOS ARM64/x86_64, Linux x86_64/ARM64, Windows x86_64.
- **Install script:** `install.sh` detects OS/arch, downloads the right binary from GitHub Releases, installs to `/usr/local/bin` (or `~/.local/bin` as fallback). Usage: `curl -fsSL https://rohitsainier.github.io/pages/bharatlink/install.sh | sh`
