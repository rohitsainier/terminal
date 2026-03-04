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
  hooks/                    # SolidJS hooks (useTheme, useTerminal, useAI)
  effects/                  # Visual effects (CRT, Glow, MatrixRain, Particles, Hologram)
  themes/                   # Theme JSON files + ThemeEngine
  styles/                   # Global CSS (global.css, terminal.css, effects.css, monitor.css)
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
  src/monitor.rs             # System monitoring
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
