use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Snippet {
    pub id: String,
    pub name: String,
    pub command: String,
    pub icon: String,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<String>,
}

pub struct SnippetManager {
    snippets: Vec<Snippet>,
    file_path: PathBuf,
}

impl SnippetManager {
    pub fn new() -> Self {
        let file_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("flux-terminal")
            .join("snippets.json");

        let mut manager = Self {
            snippets: Vec::new(),
            file_path,
        };

        manager.load();
        manager
    }

    pub fn load(&mut self) {
        if self.file_path.exists() {
            match std::fs::read_to_string(&self.file_path) {
                Ok(content) => {
                    self.snippets = serde_json::from_str(&content).unwrap_or_else(|_| {
                        Self::default_snippets()
                    });
                }
                Err(_) => {
                    self.snippets = Self::default_snippets();
                }
            }
        } else {
            self.snippets = Self::default_snippets();
            let _ = self.save();
        }
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir: {}", e))?;
        }

        let content = serde_json::to_string_pretty(&self.snippets)
            .map_err(|e| format!("Serialize error: {}", e))?;

        std::fs::write(&self.file_path, content)
            .map_err(|e| format!("Write error: {}", e))?;

        Ok(())
    }

    pub fn list(&self) -> Vec<Snippet> {
        self.snippets.clone()
    }

    pub fn search(&self, query: &str) -> Vec<Snippet> {
        let q = query.to_lowercase();
        self.snippets
            .iter()
            .filter(|s| {
                s.name.to_lowercase().contains(&q)
                    || s.command.to_lowercase().contains(&q)
                    || s.tags.iter().any(|t| t.to_lowercase().contains(&q))
                    || s.category
                        .as_ref()
                        .map(|c| c.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    pub fn add(&mut self, snippet: Snippet) -> Result<(), String> {
        // Check for duplicate ID
        if self.snippets.iter().any(|s| s.id == snippet.id) {
            return Err(format!("Snippet with id '{}' already exists", snippet.id));
        }

        self.snippets.push(snippet);
        self.save()
    }

    pub fn update(&mut self, snippet: Snippet) -> Result<(), String> {
        let pos = self
            .snippets
            .iter()
            .position(|s| s.id == snippet.id)
            .ok_or_else(|| format!("Snippet not found: {}", snippet.id))?;

        self.snippets[pos] = snippet;
        self.save()
    }

    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        let len_before = self.snippets.len();
        self.snippets.retain(|s| s.id != id);

        if self.snippets.len() == len_before {
            return Err(format!("Snippet not found: {}", id));
        }

        self.save()
    }

    pub fn get(&self, id: &str) -> Option<Snippet> {
        self.snippets.iter().find(|s| s.id == id).cloned()
    }

    pub fn get_by_category(&self, category: &str) -> Vec<Snippet> {
        self.snippets
            .iter()
            .filter(|s| {
                s.category
                    .as_ref()
                    .map(|c| c.to_lowercase() == category.to_lowercase())
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .snippets
            .iter()
            .filter_map(|s| s.category.clone())
            .collect();

        categories.sort();
        categories.dedup();
        categories
    }

    pub fn import_from_json(&mut self, json_str: &str) -> Result<usize, String> {
        let imported: Vec<Snippet> = serde_json::from_str(json_str)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        let count = imported.len();
        for snippet in imported {
            if !self.snippets.iter().any(|s| s.id == snippet.id) {
                self.snippets.push(snippet);
            }
        }

        self.save()?;
        Ok(count)
    }

    pub fn export_to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.snippets)
            .map_err(|e| format!("Serialize error: {}", e))
    }

    fn default_snippets() -> Vec<Snippet> {
        vec![
            Snippet {
                id: "default-1".into(),
                name: "Docker Cleanup".into(),
                command: "docker system prune -af --volumes".into(),
                icon: "🐳".into(),
                tags: vec!["docker".into(), "cleanup".into()],
                category: Some("Docker".into()),
                description: Some("Remove all unused containers, images, networks and volumes".into()),
                created_at: None,
            },
            Snippet {
                id: "default-2".into(),
                name: "Kill Port".into(),
                command: "lsof -ti:$PORT | xargs kill -9".into(),
                icon: "💀".into(),
                tags: vec!["network".into(), "port".into(), "kill".into()],
                category: Some("Network".into()),
                description: Some("Kill process running on a specific port. Replace $PORT with port number".into()),
                created_at: None,
            },
            Snippet {
                id: "default-3".into(),
                name: "Git Undo Last Commit".into(),
                command: "git reset --soft HEAD~1".into(),
                icon: "⏪".into(),
                tags: vec!["git".into(), "undo".into()],
                category: Some("Git".into()),
                description: Some("Undo last commit but keep changes staged".into()),
                created_at: None,
            },
            Snippet {
                id: "default-4".into(),
                name: "Find Large Files".into(),
                command: "find . -size +100M -exec ls -lh {} \\;".into(),
                icon: "🔍".into(),
                tags: vec!["files".into(), "disk".into(), "search".into()],
                category: Some("Files".into()),
                description: Some("Find all files larger than 100MB in current directory".into()),
                created_at: None,
            },
            Snippet {
                id: "default-5".into(),
                name: "System Info".into(),
                command: "uname -a && echo '---' && df -h && echo '---' && free -h 2>/dev/null || vm_stat".into(),
                icon: "💻".into(),
                tags: vec!["system".into(), "info".into()],
                category: Some("System".into()),
                description: Some("Show OS, disk usage, and memory info".into()),
                created_at: None,
            },
            Snippet {
                id: "default-6".into(),
                name: "Git Status + Log".into(),
                command: "git status && echo '---' && git log --oneline -10".into(),
                icon: "📊".into(),
                tags: vec!["git".into(), "status".into()],
                category: Some("Git".into()),
                description: Some("Show git status and last 10 commits".into()),
                created_at: None,
            },
            Snippet {
                id: "default-7".into(),
                name: "NPM Fresh Install".into(),
                command: "rm -rf node_modules package-lock.json && npm install".into(),
                icon: "📦".into(),
                tags: vec!["npm".into(), "node".into(), "clean".into()],
                category: Some("Node.js".into()),
                description: Some("Delete node_modules and lock file, then reinstall".into()),
                created_at: None,
            },
            Snippet {
                id: "default-8".into(),
                name: "SSH Key Generate".into(),
                command: "ssh-keygen -t ed25519 -C \"$EMAIL\"".into(),
                icon: "🔑".into(),
                tags: vec!["ssh".into(), "security".into(), "key".into()],
                category: Some("Security".into()),
                description: Some("Generate a new ED25519 SSH key pair".into()),
                created_at: None,
            },
            Snippet {
                id: "default-9".into(),
                name: "Disk Usage Summary".into(),
                command: "du -sh */ 2>/dev/null | sort -rh | head -20".into(),
                icon: "💾".into(),
                tags: vec!["disk".into(), "space".into(), "usage".into()],
                category: Some("System".into()),
                description: Some("Show top 20 directories by size".into()),
                created_at: None,
            },
            Snippet {
                id: "default-10".into(),
                name: "Docker Running Containers".into(),
                command: "docker ps --format 'table {{.Names}}\\t{{.Status}}\\t{{.Ports}}'".into(),
                icon: "🐋".into(),
                tags: vec!["docker".into(), "containers".into()],
                category: Some("Docker".into()),
                description: Some("List running containers in a clean table format".into()),
                created_at: None,
            },
        ]
    }
}