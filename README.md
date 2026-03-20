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

**[Download](#-installation)** · **[Features](#-features)** · **[Themes](#-themes)** · **[AI Setup](#-ai-setup)** · **[Shortcuts](#-keyboard-shortcuts)** · **[Contributing](#-contributing)**

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
Sovereign peer-to-peer file & text sharing. No servers, no accounts, no cloud.

- **Zero-config discovery** — mDNS finds peers on your LAN automatically
- **Encrypted transfers** — QUIC + TLS 1.3, always on
- **Resumable file transfers** — BLAKE3-verified, chunked via iroh-blobs
- **NAT traversal** — Hole punching + relay fallback for cross-network sharing
- **Trust management** — Nickname peers, approve incoming transfers
- **Text sharing** — Send snippets, commands, or messages instantly

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

- **BharatLink** uses [iroh](https://iroh.computer) for QUIC-based P2P with mDNS discovery, NAT hole punching, relay fallback, and BLAKE3-verified resumable file transfers — zero servers required.

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
