use crate::ai::AIProvider;
use crate::snippets::Snippet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub theme: String,
    pub font_family: String,
    pub font_size: f32,
    pub cursor_style: String,         // block, beam, underline
    pub cursor_blink: bool,
    pub opacity: f32,                  // window opacity 0.0-1.0
    pub blur: bool,                    // background blur
    pub effects: EffectsConfig,
    pub ai_provider: Option<AIProvider>,
    pub default_shell: Option<String>,
    pub default_cwd: Option<String>,
    pub snippets: Vec<Snippet>,
    pub keybindings: KeyBindings,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EffectsConfig {
    pub crt_scanlines: bool,
    pub crt_curvature: bool,
    pub glow: bool,
    pub glow_intensity: f32,
    pub matrix_rain: bool,
    pub matrix_rain_opacity: f32,
    pub particles_on_keystroke: bool,
    pub screen_flicker: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KeyBindings {
    pub ai_bar: String,               // default: Ctrl+K / Cmd+K
    pub command_palette: String,       // default: Ctrl+P / Cmd+P
    pub new_tab: String,
    pub close_tab: String,
    pub split_vertical: String,
    pub split_horizontal: String,
    pub snippet_library: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "hacker-green".into(),
            font_family: "JetBrains Mono".into(),
            font_size: 14.0,
            cursor_style: "block".into(),
            cursor_blink: true,
            opacity: 0.95,
            blur: true,
            effects: EffectsConfig {
                crt_scanlines: false,
                crt_curvature: false,
                glow: true,
                glow_intensity: 0.3,
                matrix_rain: false,
                matrix_rain_opacity: 0.05,
                particles_on_keystroke: false,
                screen_flicker: false,
            },
            ai_provider: Some(AIProvider::Ollama {
                model: "llama3.2".into(),
                base_url: "http://localhost:11434".into(),
            }),
            default_shell: None,
            default_cwd: None,
            snippets: default_snippets(),
            keybindings: KeyBindings {
                ai_bar: "CommandOrControl+K".into(),
                command_palette: "CommandOrControl+P".into(),
                new_tab: "CommandOrControl+T".into(),
                close_tab: "CommandOrControl+W".into(),
                split_vertical: "CommandOrControl+D".into(),
                split_horizontal: "CommandOrControl+Shift+D".into(),
                snippet_library: "CommandOrControl+Shift+L".into(),
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = config_file_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            let config = Self::default();
            let _ = config.save();
            config
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = config_file_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| format!("Serialize error: {}", e))?;
        std::fs::write(&path, content).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }
}

fn config_file_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("flux-terminal")
        .join("config.toml")
}

fn default_snippets() -> Vec<Snippet> {
    vec![
        Snippet {
            id: "1".into(),
            name: "Docker Cleanup".into(),
            command: "docker system prune -af --volumes".into(),
            icon: "🐳".into(),
            tags: vec!["docker".into(), "cleanup".into()],
            category: Some("Docker".into()),
            created_at: None,
            description: Some("Remove all unused containers, images, networks and volumes".into()),
        },
        Snippet {
            id: "2".into(),
            name: "Kill Port".into(),
            command: "lsof -ti:${PORT} | xargs kill -9".into(),
            icon: "💀".into(),
            tags: vec!["network".into(), "port".into()],
            category: Some("Network".into()),
            created_at: None,
            description: Some("Kill process running on a specific port".into()),
        },
        Snippet {
            id: "3".into(),
            name: "Git Undo Last Commit".into(),
            command: "git reset --soft HEAD~1".into(),
            icon: "⏪".into(),
            tags: vec!["git".into()],
            category: Some("Git".into()),
            created_at: None,
            description: Some("Undo last commit but keep changes staged".into()),
        },
        Snippet {
            id: "4".into(),
            name: "Find Large Files".into(),
            command: "find . -size +100M -exec ls -lh {} \\;".into(),
            icon: "🔍".into(),
            tags: vec!["files".into(), "disk".into()],
            category: Some("Files".into()),
            created_at: None,
            description: Some("Find all files larger than 100MB".into()),
        },
        Snippet {
            id: "5".into(),
            name: "System Info".into(),
            command: "uname -a && echo '---' && df -h".into(),
            icon: "💻".into(),
            tags: vec!["system".into()],
            category: Some("System".into()),
            created_at: None,
            description: Some("Show OS and disk info".into()),
        },
    ]
}

pub fn load_theme(name: &str) -> Result<serde_json::Value, String> {
    // Built-in themes
    let theme = match name {
        "hacker-green" => serde_json::json!({
            "name": "Hacker Green",
            "background": "#0a0e14",
            "foreground": "#00ff41",
            "cursor": "#00ff41",
            "cursorAccent": "#0a0e14",
            "selection": "#00ff4133",
            "border": "#00ff4122",
            "accent": "#00ff41",
            "accentDim": "#00ff4166",
            "panelBackground": "#0d1117",
            "tabActive": "#00ff4120",
            "statusBar": "#050808",
            "ansi": {
                "black": "#0a0e14",
                "red": "#ff3333",
                "green": "#00ff41",
                "yellow": "#ffff00",
                "blue": "#00d4ff",
                "magenta": "#ff00ff",
                "cyan": "#00ffff",
                "white": "#b3b3b3",
                "brightBlack": "#555555",
                "brightRed": "#ff6666",
                "brightGreen": "#66ff66",
                "brightYellow": "#ffff66",
                "brightBlue": "#66d4ff",
                "brightMagenta": "#ff66ff",
                "brightCyan": "#66ffff",
                "brightWhite": "#ffffff"
            },
            "effects": {
                "glowColor": "#00ff41",
                "scanlineColor": "#00ff4108",
                "particleColor": "#00ff41"
            }
        }),
        "cyberpunk" => serde_json::json!({
            "name": "Cyberpunk",
            "background": "#0d0221",
            "foreground": "#ff00ff",
            "cursor": "#00fff2",
            "cursorAccent": "#0d0221",
            "selection": "#ff00ff33",
            "border": "#ff00ff22",
            "accent": "#00fff2",
            "accentDim": "#00fff266",
            "panelBackground": "#120330",
            "tabActive": "#ff00ff20",
            "statusBar": "#080115",
            "ansi": {
                "black": "#0d0221",
                "red": "#ff003c",
                "green": "#00ff9f",
                "yellow": "#fff700",
                "blue": "#00fff2",
                "magenta": "#ff00ff",
                "cyan": "#00d4ff",
                "white": "#c0c0c0",
                "brightBlack": "#444444",
                "brightRed": "#ff3366",
                "brightGreen": "#33ffaa",
                "brightYellow": "#ffff33",
                "brightBlue": "#33fff2",
                "brightMagenta": "#ff33ff",
                "brightCyan": "#33d4ff",
                "brightWhite": "#ffffff"
            },
            "effects": {
                "glowColor": "#ff00ff",
                "scanlineColor": "#ff00ff06",
                "particleColor": "#00fff2"
            }
        }),
        "matrix" => serde_json::json!({
            "name": "Matrix",
            "background": "#000000",
            "foreground": "#00ff00",
            "cursor": "#00ff00",
            "cursorAccent": "#000000",
            "selection": "#00ff0033",
            "border": "#00ff0015",
            "accent": "#00ff00",
            "accentDim": "#00ff0044",
            "panelBackground": "#0a0a0a",
            "tabActive": "#00ff0015",
            "statusBar": "#000000",
            "ansi": {
                "black": "#000000",
                "red": "#ff0000",
                "green": "#00ff00",
                "yellow": "#ffff00",
                "blue": "#0000ff",
                "magenta": "#ff00ff",
                "cyan": "#00ffff",
                "white": "#aaaaaa",
                "brightBlack": "#333333",
                "brightRed": "#ff3333",
                "brightGreen": "#33ff33",
                "brightYellow": "#ffff33",
                "brightBlue": "#3333ff",
                "brightMagenta": "#ff33ff",
                "brightCyan": "#33ffff",
                "brightWhite": "#ffffff"
            },
            "effects": {
                "glowColor": "#00ff00",
                "scanlineColor": "#00ff0008",
                "particleColor": "#00ff00"
            }
        }),
        "ghost-protocol" => serde_json::json!({
            "name": "Ghost Protocol",
            "background": "#0b0e14",
            "foreground": "#c5c8d4",
            "cursor": "#39bae6",
            "cursorAccent": "#0b0e14",
            "selection": "#39bae633",
            "border": "#39bae622",
            "accent": "#39bae6",
            "accentDim": "#39bae644",
            "panelBackground": "#0f1219",
            "tabActive": "#39bae620",
            "statusBar": "#080a0f",
            "ansi": {
                "black": "#0b0e14",
                "red": "#f07178",
                "green": "#aad94c",
                "yellow": "#e6b450",
                "blue": "#39bae6",
                "magenta": "#d2a6ff",
                "cyan": "#95e6cb",
                "white": "#c5c8d4",
                "brightBlack": "#5c6773",
                "brightRed": "#ff8f9d",
                "brightGreen": "#c2ee74",
                "brightYellow": "#ffd580",
                "brightBlue": "#73d0ff",
                "brightMagenta": "#dfbfff",
                "brightCyan": "#b8ffea",
                "brightWhite": "#ffffff"
            },
            "effects": {
                "glowColor": "#39bae6",
                "scanlineColor": "#39bae608",
                "particleColor": "#39bae6"
            }
        }),
        "tron" => serde_json::json!({
            "name": "Tron",
            "background": "#000000",
            "foreground": "#6fc3df",
            "cursor": "#ffffff",
            "cursorAccent": "#000000",
            "selection": "#6fc3df33",
            "border": "#6fc3df22",
            "accent": "#6fc3df",
            "accentDim": "#6fc3df44",
            "panelBackground": "#050505",
            "tabActive": "#6fc3df15",
            "statusBar": "#000000",
            "ansi": {
                "black": "#000000",
                "red": "#ff3a3a",
                "green": "#6fc3df",
                "yellow": "#ffd242",
                "blue": "#6fc3df",
                "magenta": "#df740c",
                "cyan": "#6fc3df",
                "white": "#b0b0b0",
                "brightBlack": "#444444",
                "brightRed": "#ff6666",
                "brightGreen": "#8fd8ef",
                "brightYellow": "#ffe066",
                "brightBlue": "#8fd8ef",
                "brightMagenta": "#ef8c3c",
                "brightCyan": "#8fd8ef",
                "brightWhite": "#ffffff"
            },
            "effects": {
                "glowColor": "#6fc3df",
                "scanlineColor": "#6fc3df08",
                "particleColor": "#6fc3df"
            }
        }),
        _ => {
            return Err(format!("Theme not found: {}", name));
        }
    };

    Ok(theme)
}