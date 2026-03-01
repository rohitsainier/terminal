<div align="center">

# ⚡ Flux Terminal

### The AI-Powered Terminal of the Future

A blazing-fast, GPU-accelerated terminal emulator built with **Tauri + SolidJS + Rust** — featuring AI command translation, holographic effects, and a cyberpunk soul.

[![License](https://img.shields.io/badge/license-MIT-00ff41?style=for-the-badge&labelColor=0a0e14)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?style=for-the-badge&logo=tauri&labelColor=0a0e14)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-🦀-orange?style=for-the-badge&labelColor=0a0e14)](https://www.rust-lang.org)
[![SolidJS](https://img.shields.io/badge/SolidJS-⚡-4f88c6?style=for-the-badge&labelColor=0a0e14)](https://www.solidjs.com)

<br/>

<!-- Hero Screenshot -->
<img width="1104" height="706" alt="Screenshot 2026-03-01 at 11 36 41 PM" src="https://github.com/user-attachments/assets/de854bf8-e20a-4281-935b-706b046d30b8" />

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
<img width="620" height="410" alt="Screenshot 2026-03-01 at 11 38 06 PM" src="https://github.com/user-attachments/assets/55ae75d7-b796-4138-98b8-f2b4cb5b038b" />

</td>
</tr>
<tr>
<td width="50%">

<!-- Themes Screenshot -->
<img width="608" height="610" alt="Screenshot 2026-03-01 at 11 38 20 PM" src="https://github.com/user-attachments/assets/f76380aa-f593-4b2e-af74-1750ff22363b" />

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
<img width="591" height="567" alt="Screenshot 2026-03-01 at 11 38 59 PM" src="https://github.com/user-attachments/assets/4e5bf556-694a-4219-aacb-b5d3c19d2620" />

</td>
</tr>
<tr>
<td width="50%">

<!-- Effects Screenshot -->
<img width="610" height="516" alt="Screenshot 2026-03-01 at 11 38 30 PM" src="https://github.com/user-attachments/assets/987b618f-66d9-4075-995b-10a6c705fca2" />

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
