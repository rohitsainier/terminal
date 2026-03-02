use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ISSPosition {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub velocity: f64,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewsItem {
    pub title: String,
    pub source: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemStats {
    pub os: String,
    pub hostname: String,
    pub uptime_secs: u64,
    pub cpu_count: usize,
    pub memory_total_mb: u64,
    pub memory_used_mb: u64,
    pub disk_total_gb: u64,
    pub disk_used_gb: u64,
    pub local_ip: String,
    pub public_ip: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActivityEvent {
    pub lat: f64,
    pub lon: f64,
    pub label: String,
    pub event_type: String,
    pub intensity: f64,
}

pub struct MonitorEngine {
    client: Client,
}

impl MonitorEngine {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    /// Fetch ISS position from free API
    pub async fn fetch_iss_position(&self) -> Result<ISSPosition, String> {
        let resp = self
            .client
            .get("https://api.wheretheiss.at/v1/satellites/25544")
            .send()
            .await
            .map_err(|e| format!("ISS API error: {}", e))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("ISS parse error: {}", e))?;

        Ok(ISSPosition {
            latitude: data["latitude"].as_f64().unwrap_or(0.0),
            longitude: data["longitude"].as_f64().unwrap_or(0.0),
            altitude: data["altitude"].as_f64().unwrap_or(408.0),
            velocity: data["velocity"].as_f64().unwrap_or(27600.0),
            timestamp: data["timestamp"].as_u64().unwrap_or(0),
        })
    }

    /// Fetch news headlines from free APIs
    pub async fn fetch_news(&self) -> Result<Vec<NewsItem>, String> {
        // Try multiple free sources
        let sources = vec![
            ("https://hacker-news.firebaseio.com/v0/topstories.json", "hacker_news"),
            ("https://www.reddit.com/r/worldnews/hot.json?limit=10", "reddit"),
        ];

        // Try Hacker News first
        if let Ok(items) = self.fetch_hackernews().await {
            if !items.is_empty() {
                return Ok(items);
            }
        }

        // Fallback to Reddit
        if let Ok(items) = self.fetch_reddit_news().await {
            if !items.is_empty() {
                return Ok(items);
            }
        }

        // Final fallback: simulated headlines
        Ok(vec![
            NewsItem { title: "Global AI Summit Announces New Safety Framework".into(), source: "Reuters".into(), timestamp: "2m ago".into() },
            NewsItem { title: "SpaceX Launches 40 Starlink Satellites".into(), source: "SpaceNews".into(), timestamp: "15m ago".into() },
            NewsItem { title: "Quantum Computing Breakthrough Achieved".into(), source: "Nature".into(), timestamp: "1h ago".into() },
            NewsItem { title: "Cybersecurity Alert: Zero-Day Exploit Discovered".into(), source: "CISA".into(), timestamp: "2h ago".into() },
            NewsItem { title: "Global Markets Rally on Economic Data".into(), source: "Bloomberg".into(), timestamp: "3h ago".into() },
            NewsItem { title: "Open Source Project Reaches 100K Stars".into(), source: "GitHub".into(), timestamp: "4h ago".into() },
            NewsItem { title: "New Undersea Cable Connects Continents".into(), source: "TeleGeography".into(), timestamp: "5h ago".into() },
            NewsItem { title: "Climate Satellite Captures High-Res Data".into(), source: "NASA".into(), timestamp: "6h ago".into() },
        ])
    }

    async fn fetch_hackernews(&self) -> Result<Vec<NewsItem>, String> {
        let resp = self
            .client
            .get("https://hacker-news.firebaseio.com/v0/topstories.json")
            .send()
            .await
            .map_err(|e| format!("{}", e))?;

        let ids: Vec<u64> = resp.json().await.map_err(|e| format!("{}", e))?;

        let mut items = Vec::new();
        for id in ids.iter().take(12) {
            if let Ok(resp) = self
                .client
                .get(format!(
                    "https://hacker-news.firebaseio.com/v0/item/{}.json",
                    id
                ))
                .send()
                .await
            {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    if let Some(title) = data["title"].as_str() {
                        items.push(NewsItem {
                            title: title.to_string(),
                            source: "Hacker News".into(),
                            timestamp: format!("#{}", id),
                        });
                    }
                }
            }
        }

        Ok(items)
    }

    async fn fetch_reddit_news(&self) -> Result<Vec<NewsItem>, String> {
        let resp = self
            .client
            .get("https://www.reddit.com/r/worldnews/hot.json?limit=10")
            .header("User-Agent", "FluxTerminal/0.1")
            .send()
            .await
            .map_err(|e| format!("{}", e))?;

        let data: serde_json::Value = resp.json().await.map_err(|e| format!("{}", e))?;

        let items = data["data"]["children"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|child| {
                        let d = &child["data"];
                        let title = d["title"].as_str()?;
                        Some(NewsItem {
                            title: title.to_string(),
                            source: "Reddit/WorldNews".into(),
                            timestamp: d["created_utc"]
                                .as_f64()
                                .map(|t| format_relative_time(t as u64))
                                .unwrap_or_else(|| "now".into()),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(items)
    }

    /// Get system statistics
    pub fn get_system_stats(&self) -> SystemStats {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".into());

        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        // Get local IP
        let local_ip = get_local_ip().unwrap_or_else(|| "127.0.0.1".into());

        SystemStats {
            os: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
            hostname,
            uptime_secs: get_uptime(),
            cpu_count,
            memory_total_mb: get_total_memory_mb(),
            memory_used_mb: get_used_memory_mb(),
            disk_total_gb: 0,
            disk_used_gb: 0,
            local_ip,
            public_ip: None,
        }
    }

    /// Fetch public IP
    pub async fn fetch_public_ip(&self) -> Result<String, String> {
        let resp = self
            .client
            .get("https://api.ipify.org?format=json")
            .send()
            .await
            .map_err(|e| format!("{}", e))?;

        let data: serde_json::Value = resp.json().await.map_err(|e| format!("{}", e))?;

        data["ip"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No IP".into())
    }

    /// Generate simulated global activity events
    pub fn generate_activity(&self) -> Vec<ActivityEvent> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let cities = vec![
            (40.7, -74.0, "New York"),
            (51.5, -0.1, "London"),
            (35.7, 139.7, "Tokyo"),
            (22.3, 114.2, "Hong Kong"),
            (37.8, -122.4, "San Francisco"),
            (-33.9, 151.2, "Sydney"),
            (55.8, 37.6, "Moscow"),
            (1.3, 103.8, "Singapore"),
            (48.9, 2.4, "Paris"),
            (52.5, 13.4, "Berlin"),
            (19.1, 72.9, "Mumbai"),
            (-23.6, -46.6, "São Paulo"),
            (39.9, 116.4, "Beijing"),
            (37.6, 127.0, "Seoul"),
            (25.0, 55.3, "Dubai"),
            (30.0, 31.2, "Cairo"),
            (-1.3, 36.8, "Nairobi"),
            (33.9, -118.2, "Los Angeles"),
            (41.9, -87.6, "Chicago"),
            (49.3, -123.1, "Vancouver"),
            (59.3, 18.1, "Stockholm"),
            (35.7, 51.4, "Tehran"),
            (-34.6, -58.4, "Buenos Aires"),
            (13.8, 100.5, "Bangkok"),
            (14.6, 121.0, "Manila"),
            (28.6, 77.2, "Delhi"),
            (31.2, 121.5, "Shanghai"),
            (43.7, -79.4, "Toronto"),
            (64.1, -21.9, "Reykjavik"),
            (-6.2, 106.8, "Jakarta"),
        ];

        let event_types = ["NETWORK", "TRAFFIC", "SIGNAL", "DATA_XFER", "SCAN", "PING", "ALERT"];

        cities
            .iter()
            .enumerate()
            .map(|(i, (lat, lon, name))| {
                let pseudo_rand = ((seed + i as u64 * 7919) % 100) as f64 / 100.0;
                let evt_idx = ((seed / (i as u64 + 1)) % event_types.len() as u64) as usize;
                ActivityEvent {
                    lat: *lat,
                    lon: *lon,
                    label: name.to_string(),
                    event_type: event_types[evt_idx].to_string(),
                    intensity: 0.3 + pseudo_rand * 0.7,
                }
            })
            .collect()
    }
}

fn format_relative_time(unix_ts: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let diff = now.saturating_sub(unix_ts);
    if diff < 60 {
        format!("{}s ago", diff)
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip().to_string())
}

fn get_uptime() -> u64 {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("sysctl").arg("-n").arg("kern.boottime").output() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Some(sec_str) = s.split("sec = ").nth(1) {
                if let Some(sec) = sec_str.split(',').next() {
                    if let Ok(boot) = sec.trim().parse::<u64>() {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        return now.saturating_sub(boot);
                    }
                }
            }
        }
    }
    0
}

fn get_total_memory_mb() -> u64 {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("sysctl").arg("-n").arg("hw.memsize").output() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Ok(bytes) = s.trim().parse::<u64>() {
                return bytes / 1024 / 1024;
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        if let Ok(val) = kb.parse::<u64>() {
                            return val / 1024;
                        }
                    }
                }
            }
        }
    }
    0
}

fn get_used_memory_mb() -> u64 {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("vm_stat").output() {
            let s = String::from_utf8_lossy(&output.stdout);
            let mut active = 0u64;
            let mut wired = 0u64;
            let mut compressed = 0u64;
            for line in s.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    let val = parts[1].trim().trim_end_matches('.').parse::<u64>().unwrap_or(0);
                    let label = parts[0].trim();
                    if label.contains("Pages active") { active = val; }
                    if label.contains("Pages wired") { wired = val; }
                    if label.contains("Pages occupied by compressor") { compressed = val; }
                }
            }
            return (active + wired + compressed) * 4096 / 1024 / 1024;
        }
    }
    0
}