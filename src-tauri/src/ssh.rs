use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SSHConnection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: SSHAuthMethod,
    pub jump_host: Option<String>,
    pub local_forwards: Vec<PortForward>,
    pub remote_forwards: Vec<PortForward>,
    pub startup_command: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub group: Option<String>,
    pub last_connected: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SSHAuthMethod {
    Key { path: String, passphrase: Option<String> },
    Password,
    Agent,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortForward {
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
}

pub struct SSHManager {
    connections: Vec<SSHConnection>,
    file_path: PathBuf,
}

impl SSHManager {
    pub fn new() -> Self {
        let file_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("flux-terminal")
            .join("ssh_connections.json");

        let mut manager = Self {
            connections: Vec::new(),
            file_path,
        };

        manager.load();

        // Also try to import from ~/.ssh/config
        if manager.connections.is_empty() {
            let _ = manager.import_from_ssh_config();
        }

        manager
    }

    pub fn load(&mut self) {
        if self.file_path.exists() {
            match std::fs::read_to_string(&self.file_path) {
                Ok(content) => {
                    self.connections = serde_json::from_str(&content).unwrap_or_default();
                }
                Err(_) => {
                    self.connections = Vec::new();
                }
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir: {}", e))?;
        }

        let content = serde_json::to_string_pretty(&self.connections)
            .map_err(|e| format!("Serialize error: {}", e))?;

        std::fs::write(&self.file_path, content)
            .map_err(|e| format!("Write error: {}", e))?;

        Ok(())
    }

    pub fn list(&self) -> Vec<SSHConnection> {
        self.connections.clone()
    }

    pub fn list_grouped(&self) -> HashMap<String, Vec<SSHConnection>> {
        let mut groups: HashMap<String, Vec<SSHConnection>> = HashMap::new();

        for conn in &self.connections {
            let group = conn.group.clone().unwrap_or_else(|| "Ungrouped".into());
            groups.entry(group).or_default().push(conn.clone());
        }

        groups
    }

    pub fn get(&self, id: &str) -> Option<SSHConnection> {
        self.connections.iter().find(|c| c.id == id).cloned()
    }

    pub fn add(&mut self, connection: SSHConnection) -> Result<(), String> {
        if self.connections.iter().any(|c| c.id == connection.id) {
            return Err(format!("Connection with id '{}' already exists", connection.id));
        }

        self.connections.push(connection);
        self.save()
    }

    pub fn update(&mut self, connection: SSHConnection) -> Result<(), String> {
        let pos = self
            .connections
            .iter()
            .position(|c| c.id == connection.id)
            .ok_or_else(|| format!("Connection not found: {}", connection.id))?;

        self.connections[pos] = connection;
        self.save()
    }

    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        let len_before = self.connections.len();
        self.connections.retain(|c| c.id != id);

        if self.connections.len() == len_before {
            return Err(format!("Connection not found: {}", id));
        }

        self.save()
    }

    /// Build the SSH command string for a connection
    pub fn build_ssh_command(&self, id: &str) -> Result<String, String> {
        let conn = self
            .get(id)
            .ok_or_else(|| format!("Connection not found: {}", id))?;

        let mut parts: Vec<String> = vec!["ssh".into()];

        // Port
        if conn.port != 22 {
            parts.push("-p".into());
            parts.push(conn.port.to_string());
        }

        // Auth method
        match &conn.auth_method {
            SSHAuthMethod::Key { path, .. } => {
                parts.push("-i".into());
                parts.push(path.clone());
            }
            SSHAuthMethod::Agent => {
                // Uses SSH agent, no extra flags needed
            }
            SSHAuthMethod::Password => {
                // Will prompt for password
            }
        }

        // Jump host (ProxyJump)
        if let Some(jump) = &conn.jump_host {
            parts.push("-J".into());
            parts.push(jump.clone());
        }

        // Local port forwards
        for fwd in &conn.local_forwards {
            parts.push("-L".into());
            parts.push(format!(
                "{}:{}:{}",
                fwd.local_port, fwd.remote_host, fwd.remote_port
            ));
        }

        // Remote port forwards
        for fwd in &conn.remote_forwards {
            parts.push("-R".into());
            parts.push(format!(
                "{}:{}:{}",
                fwd.remote_port, fwd.remote_host, fwd.local_port
            ));
        }

        // User@Host
        parts.push(format!("{}@{}", conn.username, conn.host));

        // Startup command
        if let Some(cmd) = &conn.startup_command {
            parts.push(format!("'{}'", cmd));
        }

        Ok(parts.join(" "))
    }

    /// Import connections from ~/.ssh/config
    pub fn import_from_ssh_config(&mut self) -> Result<usize, String> {
        let ssh_config_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join(".ssh")
            .join("config");

        if !ssh_config_path.exists() {
            return Ok(0);
        }

        let content = std::fs::read_to_string(&ssh_config_path)
            .map_err(|e| format!("Failed to read SSH config: {}", e))?;

        let mut count = 0;
        let mut current_host: Option<String> = None;
        let mut current_hostname: Option<String> = None;
        let mut current_user: Option<String> = None;
        let mut current_port: u16 = 22;
        let mut current_identity: Option<String> = None;

        let flush = |host: &Option<String>,
                     hostname: &Option<String>,
                     user: &Option<String>,
                     port: u16,
                     identity: &Option<String>,
                     connections: &mut Vec<SSHConnection>|
         -> bool {
            if let (Some(h), Some(hn)) = (host, hostname) {
                // Skip wildcard entries
                if h.contains('*') || h.contains('?') {
                    return false;
                }

                let id = format!("ssh-config-{}", h.to_lowercase().replace(' ', "-"));

                // Skip if already exists
                if connections.iter().any(|c| c.id == id) {
                    return false;
                }

                let auth = if let Some(key_path) = identity {
                    SSHAuthMethod::Key {
                        path: key_path.clone(),
                        passphrase: None,
                    }
                } else {
                    SSHAuthMethod::Agent
                };

                connections.push(SSHConnection {
                    id,
                    name: h.clone(),
                    host: hn.clone(),
                    port,
                    username: user.clone().unwrap_or_else(|| whoami().unwrap_or_else(|| "root".into())),
                    auth_method: auth,
                    jump_host: None,
                    local_forwards: Vec::new(),
                    remote_forwards: Vec::new(),
                    startup_command: None,
                    color: None,
                    icon: Some("🔗".into()),
                    group: Some("Imported".into()),
                    last_connected: None,
                });

                return true;
            }
            false
        };

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
            if parts.len() < 2 {
                continue;
            }

            let key = parts[0].to_lowercase();
            let value = parts[1].trim().to_string();

            match key.as_str() {
                "host" => {
                    // Flush previous entry
                    if flush(
                        &current_host,
                        &current_hostname,
                        &current_user,
                        current_port,
                        &current_identity,
                        &mut self.connections,
                    ) {
                        count += 1;
                    }

                    current_host = Some(value);
                    current_hostname = None;
                    current_user = None;
                    current_port = 22;
                    current_identity = None;
                }
                "hostname" => {
                    current_hostname = Some(value);
                }
                "user" => {
                    current_user = Some(value);
                }
                "port" => {
                    current_port = value.parse().unwrap_or(22);
                }
                "identityfile" => {
                    // Expand ~ to home dir
                    let expanded = if value.starts_with("~/") {
                        dirs::home_dir()
                            .map(|h| h.join(&value[2..]).to_string_lossy().to_string())
                            .unwrap_or(value)
                    } else {
                        value
                    };
                    current_identity = Some(expanded);
                }
                _ => {}
            }
        }

        // Flush last entry
        if flush(
            &current_host,
            &current_hostname,
            &current_user,
            current_port,
            &current_identity,
            &mut self.connections,
        ) {
            count += 1;
        }

        if count > 0 {
            self.save()?;
        }

        Ok(count)
    }
}

/// Get current system username
fn whoami() -> Option<String> {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .ok()
}

/// Check if a host is reachable (quick ping)
pub fn check_host_reachable(host: &str, port: u16, timeout_ms: u64) -> bool {
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;

    let addr_str = format!("{}:{}", host, port);
    let timeout = Duration::from_millis(timeout_ms);

    match addr_str.to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                TcpStream::connect_timeout(&addr, timeout).is_ok()
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// Parse known_hosts file to get previously connected hosts
pub fn get_known_hosts() -> Vec<String> {
    let known_hosts_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".ssh")
        .join("known_hosts");

    if !known_hosts_path.exists() {
        return Vec::new();
    }

    match std::fs::read_to_string(&known_hosts_path) {
        Ok(content) => {
            let mut hosts: Vec<String> = content
                .lines()
                .filter(|line| !line.starts_with('#') && !line.is_empty())
                .filter_map(|line| {
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if parts.is_empty() {
                        return None;
                    }
                    // Handle hashed hosts
                    let host = parts[0];
                    if host.starts_with('|') {
                        return None; // Hashed, can't read
                    }
                    // Handle [host]:port format
                    let clean = host
                        .split(',')
                        .next()
                        .unwrap_or(host)
                        .trim_start_matches('[')
                        .split(']')
                        .next()
                        .unwrap_or(host);
                    Some(clean.to_string())
                })
                .collect();

            hosts.sort();
            hosts.dedup();
            hosts
        }
        Err(_) => Vec::new(),
    }
}