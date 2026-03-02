use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub cwd: String,
    pub shell: String,
    pub created_at: String,
    pub is_active: bool,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub command_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandHistoryEntry {
    pub command: String,
    pub timestamp: String,
    pub cwd: String,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
    pub session_id: String,
}

pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, SessionInfo>>>,
    history: Arc<Mutex<Vec<CommandHistoryEntry>>>,
    max_history: usize,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            history: Arc::new(Mutex::new(Vec::new())),
            max_history: 10000,
        }
    }

    pub fn register_session(&self, id: &str, shell: Option<String>) -> Result<SessionInfo, String> {
        let shell = shell.unwrap_or_else(|| {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into())
        });
        let cwd = std::env::var("HOME").unwrap_or_else(|_| "~".into());

        let info = SessionInfo {
            id: id.to_string(),
            title: "flux".into(),
            cwd,
            shell,
            created_at: now_iso(),
            is_active: true,
            pid: None,
            exit_code: None,
            command_count: 0,
        };

        self.sessions
            .lock()
            .map_err(|_| "Lock error".to_string())?
            .insert(id.to_string(), info.clone());

        Ok(info)
    }

    pub fn update_session(
        &self,
        id: &str,
        title: Option<String>,
        cwd: Option<String>,
        pid: Option<u32>,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|_| "Lock error".to_string())?;
        let session = sessions
            .get_mut(id)
            .ok_or_else(|| format!("Session not found: {}", id))?;

        if let Some(t) = title {
            session.title = t;
        }
        if let Some(c) = cwd {
            session.cwd = c;
        }
        if let Some(p) = pid {
            session.pid = Some(p);
        }
        Ok(())
    }

    pub fn close_session(&self, id: &str, exit_code: Option<i32>) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|_| "Lock error".to_string())?;
        if let Some(session) = sessions.get_mut(id) {
            session.is_active = false;
            session.exit_code = exit_code;
        }
        sessions.remove(id);
        Ok(())
    }

    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions
            .lock()
            .map(|s| s.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Fixed: clone session_id before moving entry
    pub fn add_to_history(&self, entry: CommandHistoryEntry) {
        let session_id = entry.session_id.clone();

        if let Ok(mut history) = self.history.lock() {
            history.push(entry);
            if history.len() > self.max_history {
                let drain_count = history.len() - self.max_history;
                history.drain(0..drain_count);
            }
        }

        if !session_id.is_empty() {
            if let Ok(mut sessions) = self.sessions.lock() {
                if let Some(session) = sessions.get_mut(&session_id) {
                    session.command_count += 1;
                }
            }
        }
    }

    pub fn search_history(&self, query: &str, limit: usize) -> Vec<CommandHistoryEntry> {
        let q = query.to_lowercase();
        self.history
            .lock()
            .map(|h| {
                h.iter()
                    .rev()
                    .filter(|e| e.command.to_lowercase().contains(&q))
                    .take(limit)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn recent_history(&self, limit: usize) -> Vec<CommandHistoryEntry> {
        self.history
            .lock()
            .map(|h| h.iter().rev().take(limit).cloned().collect())
            .unwrap_or_default()
    }

    pub fn unique_commands(&self, limit: usize) -> Vec<String> {
        self.history
            .lock()
            .map(|h| {
                let mut seen = std::collections::HashSet::new();
                h.iter()
                    .rev()
                    .filter_map(|e| {
                        let cmd = e.command.trim().to_string();
                        if !cmd.is_empty() && seen.insert(cmd.clone()) {
                            Some(cmd)
                        } else {
                            None
                        }
                    })
                    .take(limit)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn most_used_commands(&self, limit: usize) -> Vec<(String, usize)> {
        self.history
            .lock()
            .map(|h| {
                let mut counts: HashMap<String, usize> = HashMap::new();
                for entry in h.iter() {
                    let cmd = entry.command.trim().to_string();
                    if !cmd.is_empty() {
                        *counts.entry(cmd).or_insert(0) += 1;
                    }
                }
                let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
                sorted.sort_by(|a, b| b.1.cmp(&a.1));
                sorted.truncate(limit);
                sorted
            })
            .unwrap_or_default()
    }

    pub fn clear_history(&self) {
        if let Ok(mut history) = self.history.lock() {
            history.clear();
        }
    }

    pub fn save_history(&self) -> Result<(), String> {
        let history_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("flux-terminal")
            .join("history.json");

        if let Some(parent) = history_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir: {}", e))?;
        }

        let history = self.history.lock().map_err(|_| "Lock error".to_string())?;
        let content =
            serde_json::to_string(&*history).map_err(|e| format!("Serialize error: {}", e))?;
        std::fs::write(&history_path, content).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }

    pub fn load_history(&self) -> Result<usize, String> {
        let history_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("flux-terminal")
            .join("history.json");

        if !history_path.exists() {
            return Ok(0);
        }

        let content =
            std::fs::read_to_string(&history_path).map_err(|e| format!("Read error: {}", e))?;
        let loaded: Vec<CommandHistoryEntry> =
            serde_json::from_str(&content).map_err(|e| format!("Parse error: {}", e))?;
        let count = loaded.len();

        let mut history = self.history.lock().map_err(|_| "Lock error".to_string())?;
        *history = loaded;
        Ok(count)
    }
}

pub fn list_available_shells() -> Vec<String> {
    let mut shells = Vec::new();
    #[cfg(unix)]
    {
        if let Ok(content) = std::fs::read_to_string("/etc/shells") {
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty()
                    && !line.starts_with('#')
                    && std::path::Path::new(line).exists()
                {
                    shells.push(line.to_string());
                }
            }
        }
        if shells.is_empty() {
            let common = [
                "/bin/zsh",
                "/bin/bash",
                "/bin/sh",
                "/bin/fish",
                "/usr/local/bin/fish",
                "/opt/homebrew/bin/fish",
                "/usr/local/bin/zsh",
                "/usr/local/bin/bash",
            ];
            for shell in common {
                if std::path::Path::new(shell).exists() {
                    shells.push(shell.to_string());
                }
            }
        }
    }
    #[cfg(windows)]
    {
        shells.push("cmd.exe".into());
        shells.push("powershell.exe".into());
        if let Ok(output) = std::process::Command::new("where").arg("pwsh").output() {
            if output.status.success() {
                shells.push("pwsh.exe".into());
            }
        }
        let git_bash = "C:\\Program Files\\Git\\bin\\bash.exe";
        if std::path::Path::new(git_bash).exists() {
            shells.push(git_bash.into());
        }
        if let Ok(output) = std::process::Command::new("where").arg("wsl").output() {
            if output.status.success() {
                shells.push("wsl.exe".into());
            }
        }
    }
    shells
}

fn now_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cwd() {
        assert_eq!(
            detect_cwd_from_output("user@host:~/projects$"),
            Some("~/projects".into())
        );
        assert_eq!(
            detect_cwd_from_output("/Users/john/projects ❯"),
            Some("/Users/john/projects".into())
        );
    }

    #[test]
    fn test_detect_shell() {
        let shell = detect_default_shell();
        assert!(!shell.is_empty());
    }

    #[test]
    fn test_list_shells() {
        let shells = list_available_shells();
        assert!(!shells.is_empty());
    }

    #[test]
    fn test_session_manager_history() {
        let sm = SessionManager::new();
        sm.register_session("test-1", None).unwrap();

        sm.add_to_history(CommandHistoryEntry {
            command: "ls -la".into(),
            timestamp: "123".into(),
            cwd: "~".into(),
            exit_code: Some(0),
            duration_ms: None,
            session_id: "test-1".into(),
        });

        let results = sm.search_history("ls", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].command, "ls -la");

        let session = sm.get_session("test-1").unwrap();
        assert_eq!(session.command_count, 1);
    }
}