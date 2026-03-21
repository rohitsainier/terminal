<div align="center">

# ⚡ Flux Terminal

### The AI-Powered Terminal of the Future

A blazing-fast, GPU-accelerated terminal emulator built with **Tauri + SolidJS + Rust** — featuring AI command translation, P2P file sharing, network security tools, holographic effects, and a cyberpunk soul.

[![License](https://img.shields.io/badge/license-MIT-00ff41?style=for-the-badge&labelColor=0a0e14)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?style=for-the-badge&logo=tauri&labelColor=0a0e14)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-🦀-orange?style=for-the-badge&labelColor=0a0e14)](https://www.rust-lang.org)
[![SolidJS](https://img.shields.io/badge/SolidJS-⚡-4f88c6?style=for-the-badge&labelColor=0a0e14)](https://www.solidjs.com)

<br/>

<!-- Hero Screenshot -->
<img width="1104" height="706" alt="Screenshot 2026-03-01 at 11 36 41 PM" src="https://github.com/user-attachments/assets/de854bf8-e20a-4281-935b-706b046d30b8" />

<br/>

**[Download](#-installation)** · **[Features](#-features)** · **[BharatLink](#-bharatlink--p2p-share)** · **[NETOPS](#-netops-dashboard)** · **[Themes](#-themes)** · **[AI Setup](#-ai-setup)** · **[Shortcuts](#-keyboard-shortcuts)** · **[Contributing](#-contributing)**

</div>

---

<br/>

## ✨ Features

<table>
<tr>
<td width="50%">

### 🤖 AI Command Bar
Press `⌘K` and describe what you want in plain English. Flux translates it to the right shell command instantly.

- Natural language → shell commands
- Danger detection & warnings
- One-click explain & execute
- Supports Ollama, OpenAI, Claude

</td>
<td width="50%">

<!-- AI Bar Screenshot -->
<img width="620" height="410" alt="Screenshot 2026-03-01 at 11 38 06 PM" src="https://github.com/user-attachments/assets/55ae75d7-b796-4138-98b8-f2b4cb5b038b" />

</td>
</tr>
<tr>
<td width="50%">

<!-- Themes Screenshot -->
<img width="608" height="610" alt="Screenshot 2026-03-01 at 11 38 20 PM" src="https://github.com/user-attachments/assets/f76380aa-f593-4b2e-af74-1750ff22363b" />

</td>
<td width="50%">

### 🎨 6 Stunning Themes
Hand-crafted color schemes designed for long coding sessions.

- **Hacker Green** — Classic terminal vibes
- **Cyberpunk** — Neon pink & cyan
- **Matrix** — Digital rain aesthetic
- **Ghost Protocol** — Stealth blue
- **Tron** — Light cycle glow
- **Midnight** — Deep purple calm

</td>
</tr>
<tr>
<td width="50%">

### 🔗 BharatLink — P2P Share
Sovereign peer-to-peer file & text sharing. No servers, no accounts, no cloud. India's AirDrop alternative.

- **Chat-style UI** — Send text & files in a familiar messaging interface
- **Zero-config discovery** — mDNS finds peers on your LAN automatically
- **Encrypted transfers** — QUIC + TLS 1.3, always on
- **Content-addressed storage** — BLAKE3-verified, deduplicated, resumable
- **Cross-network** — NAT hole punching + relay fallback
- **Multi-file & folder transfer** — Batch send with progress tracking
- **Drag & drop** — Drop files onto the chat window to send
- **Screenshot & clipboard sharing** — Instant screen capture & clipboard sync
- **QR code pairing** — Scan to connect, no manual ID copying
- **Works offline** — LAN transfers need no internet

</td>
<td width="50%">

### 🌐 NETOPS Dashboard
28 network & security tools in a unified terminal dashboard.

- **Network:** ping, port scan, DNS, WHOIS, WiFi scan, traceroute, SSL inspect
- **Security:** traffic anomalies, rogue AP detection, threat intel, security score
- **Offensive:** service scan, subdomain enum, dir brute, WAF detect, vuln scan
- WiFi auth monitor & WPA handshake analyzer (macOS)
- Incident tracking & downloadable log reports

</td>
</tr>
<tr>
<td width="50%">

### 📋 Snippet Library
Save, organize, and instantly run your most-used commands.

- Category-based organization
- Search & filter
- Import/export JSON
- One-click execution
- 10 built-in starter snippets

</td>
<td width="50%">

<!-- Snippets Screenshot -->
<img width="591" height="567" alt="Screenshot 2026-03-01 at 11 38 59 PM" src="https://github.com/user-attachments/assets/4e5bf556-694a-4219-aacb-b5d3c19d2620" />

</td>
</tr>
<tr>
<td width="50%">

<!-- Effects Screenshot -->
<img width="610" height="516" alt="Screenshot 2026-03-01 at 11 38 30 PM" src="https://github.com/user-attachments/assets/987b618f-66d9-4075-995b-10a6c705fca2" />

</td>
<td width="50%">

### ✨ Visual Effects Engine
Toggle cinematic effects on and off in real-time.

- **CRT Scanlines** — Retro monitor feel
- **Text Glow** — Neon text illumination
- **Matrix Rain** — Background digital rain
- **Keystroke Particles** — Sparks on every keypress
- **Hologram Sweep** — Futuristic scan lines

</td>
</tr>
</table>

<br/>

---

<br/>

## 🔗 BharatLink — P2P Share

BharatLink is a sovereign peer-to-peer file and text sharing system built into Flux Terminal. Think of it as **India's AirDrop** — but cross-platform, works across networks, and runs entirely without servers or accounts.

### How It Works

```
┌──────────────┐         QUIC + TLS 1.3          ┌──────────────┐
│   Your Mac   │ ◄──────────────────────────────► │ Friend's PC  │
│  (Endpoint)  │     mDNS (LAN) / Relay (WAN)     │  (Endpoint)  │
└──────────────┘                                   └──────────────┘
     │                                                    │
     ▼                                                    ▼
  Blob Store                                         Blob Store
  (BLAKE3 chunks)                                  (BLAKE3 chunks)
```

### Features

| Feature | Description |
|---------|-------------|
| **Chat UI** | Send messages and files in a familiar chat interface with message history |
| **File Transfer** | Send any file with real-time progress bar, speed, and percentage |
| **Multi-file / Folder** | Select multiple files or entire folders to send as a batch |
| **Text Messages** | Send text of any length — full content stored, no truncation |
| **Drag & Drop** | Drop files directly onto the chat window to send |
| **Screenshot Share** | Capture your screen and send instantly (`screencapture` on macOS) |
| **Clipboard Sync** | Share clipboard content between trusted devices |
| **QR Code Pairing** | Generate a QR code with your Endpoint ID for easy peer adding |
| **Device Names** | Set a custom name ("Rohit's MacBook") visible to all peers |
| **Trust System** | Approve peers before accepting transfers; nickname trusted peers |
| **Offline/LAN** | Works without internet via mDNS local discovery |
| **Cross-Network** | NAT hole punching + relay fallback for internet transfers |
| **Content Dedup** | Same file sent twice? Second transfer is instant (BLAKE3 hash match) |
| **Resumable** | Interrupted transfers resume from where they left off |

### Security

- **QUIC + TLS 1.3** — All connections encrypted by default (iroh's built-in security)
- **BLAKE3 verification** — Every chunk is hash-verified, corruption is impossible
- **No servers** — Data never touches any cloud; direct peer-to-peer only
- **Secret key identity** — Each node has a unique Ed25519 keypair stored locally
- **Approval required** — Incoming file transfers must be explicitly accepted

### Storage Locations (macOS)

```
~/Library/Application Support/flux-terminal/bharatlink/
├── blobs/                    # Content-addressed blob store (transferred files)
├── secret.key                # Node identity (Ed25519 private key)
├── settings.json             # BharatLink settings
├── transfer_history.json     # Chat & transfer history
└── trusted_peers.json        # Trusted peer list with nicknames
```

### Keyboard Shortcut

Open BharatLink: `⌘/Ctrl + Shift + B`

<br/>

---

<br/>

## 🌐 NETOPS Dashboard

28 network & security tools in a unified terminal dashboard. Open with `⌘/Ctrl + Shift + N`.

### Tool Categories

**Network (13 tools)**
| Tool | Description |
|------|-------------|
| Ping | ICMP ping with latency stats |
| Port Scan | Concurrent TCP port scanning |
| DNS Lookup | Forward DNS resolution via `dig` |
| WHOIS | Domain registration lookup (cached 1hr) |
| WiFi Scan | CoreWLAN scanner (macOS) |
| WiFi Auth Monitor | Monitors WiFi authentication events |
| HTTP Headers | Inspect response headers |
| SSL Inspect | Certificate chain analysis via `openssl` |
| IP Geolocation | IP-to-location lookup (cached 1hr) |
| ARP Table | Local ARP cache display |
| Subnet Calculator | CIDR math & host range |
| Reverse DNS | PTR record lookup |
| Traceroute | Network path tracing |

**Security (7 tools)**
| Tool | Description |
|------|-------------|
| Traffic Anomaly Detection | Detects unusual network patterns |
| Rogue AP Detection | Identifies unauthorized access points |
| System Log Viewer | macOS system log analysis |
| Threat Intelligence | IP/domain reputation check (cached 1hr) |
| Security Score | Overall network security assessment |
| Incident Tracking | Log and track security incidents |
| WPA Handshake Analyzer | WPA connection analysis with downloadable reports |

**Offensive / Kali-style (8 tools)**
| Tool | Description |
|------|-------------|
| Service Scan | Banner grabbing on open ports |
| Subdomain Enum | Subdomain discovery (~90 common entries) |
| Directory Brute Force | Web path discovery (~80 common paths) |
| Web Fingerprint | Technology detection (server, framework, CMS) |
| WAF Detection | Web Application Firewall identification |
| Web Vuln Scan | Nikto-lite vulnerability scanner |
| Hash Identifier | Identify hash types (MD5, SHA, bcrypt, etc.) |
| Cipher Scan | TLS cipher suite enumeration |

<br/>

---

<br/>

## 🚀 Installation

### Prerequisites

| Requirement | Version | Check |
|---|---|---|
| **Node.js** | ≥ 18 | `node --version` |
| **Rust** | ≥ 1.70 | `rustc --version` |
| **Tauri CLI** | ≥ 2.0 | `cargo tauri --version` |

### Build from Source

```bash
# Clone the repo
git clone https://github.com/rohitsainier/terminal.git
cd flux-terminal

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

<br/>

---

<br/>

## 🎨 Themes

All 6 themes are applied globally — every panel including NETOPS and BharatLink automatically matches your selected theme.

| Theme | Accent | Vibe |
|-------|--------|------|
| **Hacker Green** | `#00ff41` | Classic terminal |
| **Cyberpunk** | `#ff00ff` | Neon pink & cyan |
| **Matrix** | `#00ff00` | Digital rain |
| **Ghost Protocol** | Steel blue | Stealth ops |
| **Tron** | Electric blue | Light cycles |
| **Midnight** | Purple | Deep calm |

Switch themes via `⌘,` (Settings) or Command Palette (`⌘P`).

<br/>

---

<br/>

## 🤖 AI Setup

Flux supports three AI providers — configure in Settings (`⌘,`):

| Provider | Setup |
|----------|-------|
| **Ollama** (local) | Install [Ollama](https://ollama.ai), pull a model (`ollama pull llama3`), done |
| **OpenAI** | Add your API key in Settings |
| **Anthropic (Claude)** | Add your API key in Settings |

<br/>

---

<br/>

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `⌘/Ctrl + K` | AI Command Bar |
| `⌘/Ctrl + P` | Command Palette |
| `⌘/Ctrl + ,` | Settings |
| `⌘/Ctrl + B` | Toggle Sidebar |
| `⌘/Ctrl + T` | New Tab |
| `⌘/Ctrl + W` | Close Tab |
| `⌘/Ctrl + M` | MCP Servers Panel |
| `⌘/Ctrl + Shift + L` | Snippet Library |
| `⌘/Ctrl + Shift + C` | AI + MCP Chat |
| `⌘/Ctrl + Shift + N` | NETOPS Dashboard |
| `⌘/Ctrl + Shift + B` | BharatLink P2P Share |
| `Escape` | Close any overlay |
| `?` | Keyboard shortcuts help |

<br/>

---

<br/>

## 🏗️ Architecture

```
┌─────────────────────────────────────────────┐
│                 SolidJS Frontend             │
│  Terminal · AI Bar · NETOPS · BharatLink     │
│  Themes · Snippets · MCP Chat · Effects      │
└───────────────┬─────────────────────────────┘
                │ Tauri IPC (invoke/listen)
┌───────────────┴─────────────────────────────┐
│                 Rust Backend                  │
│  PTY · AI Providers · Config · SSH · MCP     │
│  NetOps (28 tools) · BharatLink (iroh P2P)   │
└──────────────────────────────────────────────┘
```

### BharatLink P2P Stack

```
┌──────────────────────────────┐
│  Chat UI (SolidJS + xterm)   │   ← TransferPanel.tsx
├──────────────────────────────┤
│  useBharatLinkData.ts        │   ← Signals, events, Tauri invoke
├──────────────────────────────┤
│  bharatlink.rs               │   ← BharatLinkManager + Handlers
├──────────────────────────────┤
│  iroh 0.95 (QUIC endpoint)   │   ← mDNS, NAT punch, relay
│  iroh-blobs 0.97 (storage)   │   ← BLAKE3, chunked, resumable
├──────────────────────────────┤
│  QUIC + TLS 1.3              │   ← Always encrypted
└──────────────────────────────┘
```

- **BharatLink** uses [iroh](https://iroh.computer) for QUIC-based P2P with mDNS discovery, NAT hole punching, relay fallback, and BLAKE3-verified resumable file transfers — zero servers required.
- **Content-addressed deduplication** — same file sent twice uses zero bandwidth on the second transfer (BLAKE3 hash match in local blob store).
- **Streaming progress** — real-time progress bar with bytes/speed/percentage during file receive (uses `GetProgress::stream()` from iroh-blobs).

<br/>

---

<br/>

## 🤝 Contributing

Contributions are welcome! Please open an issue or PR.

```bash
# Development
npm run tauri dev

# The app runs on http://localhost:1420 (Vite) with Tauri hot-reload
```

<br/>

---

<div align="center">
<br/>

Built with 🦀 Rust + ⚡ SolidJS + 💙 Tauri

**[Star this repo](https://github.com/rohitsainier/terminal)** if you like what you see!

</div>
