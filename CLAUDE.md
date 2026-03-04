# CLAUDE.md - Flux Terminal

## Project Overview

Flux Terminal is an AI-powered, GPU-accelerated terminal emulator with cyberpunk aesthetics. Built with **Tauri 2.0** (Rust backend) + **SolidJS** (TypeScript frontend) + **xterm.js** (terminal rendering).

## Tech Stack

- **Frontend:** SolidJS, TypeScript, Vite, xterm.js, Three.js
- **Backend:** Rust (edition 2021), Tauri 2.0, Tokio, portable-pty
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
    monitor/                # Monitor Dashboard (refactored into sub-modules)
      index.ts              # Barrel export
      types.ts              # Interfaces + MonitorStore type
      constants.ts          # MODE_CONFIG, cities, streams, webcams, sat groups
      utils.ts              # Pure helpers (TLE propagation, colors, threat gen)
      useMonitorData.ts     # Signals, fetch functions, derived helpers
      globeManager.ts       # Globe init/destroy, mode configs, markers, effects
      TopBar.tsx            # Top bar (logo, clocks, net throughput toggle, speedtest)
      LeftPanel.tsx         # Left panel (system info, mode-specific lists)
      RightPanel.tsx        # Right panel (mode summaries, news feed)
      GlobeOverlays.tsx     # HUD, mode menu, stream/webcam players
      BottomTicker.tsx      # Crypto ticker + scrolling news
      MonitorDashboard.tsx  # Orchestrator (timers, globe init, layout)
  hooks/                    # SolidJS hooks (useTheme, useTerminal, useAI)
  effects/                  # Visual effects (CRT, Glow, MatrixRain, Particles, Hologram)
  themes/                   # Theme JSON files + ThemeEngine
    netops/                 # NETOPS Dashboard (network operations + security tools)
      index.ts              # Barrel export
      types.ts              # Interfaces, NetopsTool union (28 tools), NetopsStore type
      useNetopsData.ts      # Signals, runTool(), history management
      TopBar.tsx            # Top bar (logo, LIVE badge, UTC clock)
      ToolPanel.tsx         # Left panel (28 tool rows with icons)
      ResultPanel.tsx       # Center panel (input bar + result renderers per tool)
      InfoPanel.tsx         # Right panel (tool help + scan history)
      NetopsDashboard.tsx   # Orchestrator (keyboard, layout)
  styles/                   # Global CSS (global.css, terminal.css, effects.css, monitor.css, netops.css)
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
  src/monitor.rs             # Monitor API commands (ISS, weather, quakes, crypto, flights, net throughput, speedtest)
  src/netops.rs              # NETOPS commands (28 tools: ping, port scan, DNS, whois, WiFi scan, WiFi auth, HTTP headers, SSL, geoIP, ARP, subnet calc, reverse DNS, traceroute, traffic anomalies, rogue AP, log viewer, threat intel, security score, incidents, service scan, subdomain enum, dir brute, web fingerprint, WAF detect, vuln scan, hash ID, cipher scan, WPA handshake analyzer)
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

## Architecture Notes

- **Frontend-Backend IPC:** Tauri `invoke()` calls to `#[tauri::command]` Rust functions
- **State:** AppState holds pty_manager, ai_engine, config, snippet_manager, ssh_manager, session_manager, mcp_manager (all Mutex-wrapped)
- **Terminal:** portable-pty for PTY sessions, xterm.js for rendering, event-based output streaming
- **AI Providers:** Ollama, OpenAI, Anthropic — configurable in Settings
- **Themes:** 6 JSON theme files loaded by ThemeEngine
- **Config:** Stored in OS config directory via `dirs` crate, hot-reloadable
- **Monitor Dashboard:** 7 modes (INTEL, CYBER, SAT, FLIGHTS, CAMS, WEATHER, QUAKE) with 3D globe (globe.gl). State managed via `useMonitorData` hook returning a `MonitorStore` object passed as props. Globe logic is imperative (not JSX) in `globeManager.ts`. Rust backend uses `OnceLock<Mutex<Option<CacheEntry<T>>>>` caching pattern with TTL. Free APIs: Open-Meteo (weather), USGS (earthquakes), CoinGecko (crypto), OpenSky (flights), CelesTrak (satellites). Features: Cloudflare CDN speedtest, real network throughput monitoring (toggle-based with `sysinfo` crate), fullscreen panel mode (double-click to expand any panel)
- **NETOPS Dashboard:** 28 network/security tools (⌘⇧N). Same architecture as Monitor: `useNetopsData` hook → `NetopsStore` object → props. Rust backend uses `tokio::process::Command` for system tools (dig, whois, arp, traceroute, openssl, system_profiler) and `reqwest` for HTTP-based tools (ping, headers, geoip, threat intel, dir brute, web fingerprint, WAF detect, vuln scan). WiFi scanning via CoreWLAN Swift script (`wifi_scan.swift` embedded with `include_str!`). Caching for whois, geoip, & threat intel (1hr TTL). Port scan and service scan use concurrent `tokio::net::TcpStream::connect()` with banner grabbing. Subdomain enum uses batched `dig` lookups. Persistent JSON storage in `dirs::config_dir()/flux-terminal/` for WiFi baselines and incidents. Embedded wordlists for subdomain brute (~90 entries) and directory brute (~80 paths). WPA handshake analyzer uses `system_profiler SPAirPortDataType` for connection details + CoreWLAN scan for SSID matching (macOS Sequoia compatible). Log reports saved to `~/Downloads/` with detailed protocol explanations. Tools split into 3 categories:
  - **Network:** ping, port scan, DNS lookup, WHOIS, WiFi scan, WiFi auth monitor, HTTP headers, SSL inspect, IP geolocation, ARP table, subnet calc, reverse DNS, traceroute
  - **Security:** traffic anomaly detection, rogue AP detection, system log viewer, threat intelligence, security score, incident tracking, WPA handshake analyzer (with downloadable log reports)
  - **Offensive/Kali-style:** service scan (banner grab), subdomain enum, directory brute force, web fingerprint, WAF detection, web vuln scan (nikto-lite), hash identifier, cipher scan (TLS enum)
- **Dashboard Color Scheme:** Both Monitor and NETOPS dashboards share a unified cyan `#00d4ff` color scheme. Monitor uses `--fcmd-*` CSS variables, NETOPS uses `--nops-*` CSS variables
