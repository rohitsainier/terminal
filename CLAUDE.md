# CLAUDE.md - Flux Terminal

## Project Overview

Flux Terminal is an AI-powered, GPU-accelerated terminal emulator with cyberpunk aesthetics. Built with **Tauri 2.0** (Rust backend) + **SolidJS** (TypeScript frontend) + **xterm.js** (terminal rendering).

## Tech Stack

- **Frontend:** SolidJS, TypeScript, Vite, xterm.js
- **Backend:** Rust (edition 2021), Tauri 2.0, Tokio, portable-pty
- **P2P:** iroh 0.95 (QUIC + mDNS), iroh-blobs 0.97 (content-addressed transfers), futures-lite 2, tauri-plugin-notification
- **Package Manager:** npm
- **Build Tool:** Vite (port 1420)

## Commands

```bash
npm install                # Install frontend dependencies
npm run dev                # Start Vite dev server (port 1420)
npm run tauri dev          # Start full Tauri app with hot reload
npm run build              # Build frontend to dist/
npm run tauri build        # Build distributable desktop app
```

There are no test or lint commands configured.

## Project Structure

```
src/                        # Frontend (SolidJS + TypeScript)
  components/               # UI components (Terminal, AIBar, Settings, Sidebar, etc.)
    netops/                 # NETOPS Dashboard (network operations + security tools)
      index.ts              # Barrel export
      types.ts              # Interfaces, NetopsTool union (28 tools), NetopsStore type
      useNetopsData.ts      # Signals, runTool(), history management
      TopBar.tsx            # Top bar (logo, LIVE badge, UTC clock)
      ToolPanel.tsx         # Left panel (28 tool rows with icons)
      ResultPanel.tsx       # Center panel (input bar + result renderers per tool)
      InfoPanel.tsx         # Right panel (tool help + scan history)
      NetopsDashboard.tsx   # Orchestrator (keyboard, layout)
    bharatlink/             # BharatLink P2P file & text sharing
      index.ts              # Barrel export
      types.ts              # Interfaces (NodeInfo, PeerInfo, Transfer*, Settings, BharatLinkStore)
      useBharatLinkData.ts  # Signals, Tauri invoke/listen, event handlers
      TopBar.tsx            # Top bar (logo, node status, start/stop, UTC clock)
      PeerPanel.tsx         # Left panel (discovered + trusted peers, add peer)
      TransferPanel.tsx     # Center panel (chat-style UI, file/text send, progress, history)
      InfoPanel.tsx         # Right panel (node info, stats, settings)
      BharatLinkDashboard.tsx # Orchestrator (keyboard, layout)
  hooks/                    # SolidJS hooks (useTheme, useTerminal, useAI)
  effects/                  # Visual effects (CRT, Glow, MatrixRain, Particles, Hologram)
  themes/                   # Theme JSON files + ThemeEngine
  styles/                   # Global CSS (global.css, terminal.css, effects.css, netops.css, bharatlink.css)
src-tauri/                  # Rust backend
  src/main.rs               # Tauri app setup
  src/commands.rs            # Tauri IPC commands (~1500 lines)
  src/ai.rs                 # AI provider logic (Ollama, OpenAI, Anthropic)
  src/pty.rs                # PTY session management
  src/terminal.rs            # Terminal session management
  src/config.rs              # Configuration management
  src/snippets.rs            # Snippet storage
  src/ssh.rs                 # SSH connections
  src/mcp.rs                 # Model Context Protocol
  src/netops.rs              # NETOPS commands (28 network/security tools)
  src/bharatlink.rs          # BharatLink P2P engine (iroh QUIC + mDNS, 21 Tauri commands, active heartbeat, retry, notifications)
  src/wifi_scan.swift        # CoreWLAN WiFi scanner (embedded via include_str!)
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
- **BharatLink (P2P Share):** Sovereign peer-to-peer file & text sharing (⌘⇧B). No servers, no accounts — pure QUIC + mDNS. Architecture mirrors NETOPS: `useBharatLinkData` hook → `BharatLinkStore` object → props. Rust backend uses iroh 0.95 for QUIC endpoint with mDNS local discovery, NAT hole punching, and relay fallback. iroh-blobs 0.97 for chunked, BLAKE3-verified, resumable file transfers. Custom ALPNs: `bharatlink/meta/1` (transfer request handshake), `bharatlink/text/1` (direct text sharing + clipboard sync), `bharatlink/signal/1` (read receipts + typing indicators). 21 Tauri commands for node lifecycle, peer management (discover, add, trust/untrust), file/text transfer (send, accept, reject, cancel), multi-file/folder send, screenshot capture & send, clipboard share, signals (typing/delivered), history, and settings. Persistent storage in `dirs::config_dir()/flux-terminal/bharatlink/` (secret key, trusted peers, transfer history, settings as JSON). Blob store in `dirs::data_dir()/flux-terminal/bharatlink/blobs/`. All connections encrypted via QUIC + TLS 1.3 (iroh default). Events emitted to frontend: `bharatlink-peer-discovered`, `bharatlink-peer-lost`, `bharatlink-incoming-request`, `bharatlink-transfer-progress`, `bharatlink-transfer-complete`, `bharatlink-node-status`, `bharatlink-signal`, `bharatlink-error`, `bharatlink-peer-reconnected`. Features: chat-style UI, drag & drop file send, multi-file/folder transfer, screenshot share, clipboard sync, QR code pairing, device names, peer online/offline status with active heartbeat probing, read receipts (✓✓), typing indicator, image preview in chat, link preview cards, auto-accept from trusted peers, native OS notifications (via `tauri-plugin-notification`), error handling UX with inline chat error bubbles, transfer retry for failed downloads. QR modal uses `qrcode` npm package. Screenshot capture uses `screencapture` (macOS) / PowerShell (Windows).

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
- **QUIC stream lifecycle:** `send.finish()` signals EOF but dropping the connection immediately can lose data. Text sends use a 500ms `tokio::time::sleep` before dropping to ensure QUIC flushes all data.
- **Storage locations (macOS):**
  - Config: `~/Library/Application Support/flux-terminal/bharatlink/` (secret.key, settings.json, trusted_peers.json, transfer_history.json)
  - Blobs: `~/Library/Application Support/flux-terminal/bharatlink/blobs/` (content-addressed chunks)
- **Key imports:**
  ```rust
  use iroh::{
      discovery::mdns::MdnsDiscovery,
      endpoint::Connection,
      protocol::{AcceptError, ProtocolHandler, Router},
      Endpoint, PublicKey, SecretKey, Watcher,
  };
  use iroh_blobs::{api::remote::GetProgressItem, store::fs::FsStore, BlobsProtocol};
  use futures_lite::StreamExt;
  ```
- **Endpoint setup:** Uses `Endpoint::builder()` (N0 preset) which includes PkarrPublisher (publishes to dns.iroh.link), DnsDiscovery (resolves via DNS), and default relay servers — required for cross-network connectivity. `Endpoint::empty_builder()` only works for local-network discovery.
- **Router pattern:** All protocols (blobs, meta, text) are registered with a single `Router` — no separate accept loops. The router dispatches incoming connections by ALPN to the correct `ProtocolHandler`.
- **SecretKey generation:** `SecretKey::generate(&mut rand::rng())` (rand 0.9 API)
