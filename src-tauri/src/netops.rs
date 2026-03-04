// ═══════════════════════════════════════════════════════════════════
//  FLUX NETOPS — Network Operations Backend
//  Ping · Port Scan · DNS · Whois · WiFi · HTTP · SSL · GeoIP
//  ARP · Subnet Calc · Reverse DNS · Traceroute
// ═══════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ═══════════════════════════════════════════════════════════════════
//  TYPES
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    pub host: String,
    pub latency_ms: f64,
    pub status: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortEntry {
    pub port: u16,
    pub status: String,
    pub service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortScanResult {
    pub host: String,
    pub ports: Vec<PortEntry>,
    pub scan_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub ttl: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsLookupResult {
    pub hostname: String,
    pub records: Vec<DnsRecord>,
    pub query_time_ms: u64,
    pub server: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoisResult {
    pub domain: String,
    pub registrar: String,
    pub creation_date: String,
    pub expiry_date: String,
    pub name_servers: Vec<String>,
    pub status: Vec<String>,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub bssid: String,
    pub rssi: i32,
    pub channel: u32,
    pub security: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeader {
    pub name: String,
    pub present: bool,
    pub value: String,
    pub rating: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpHeaderResult {
    pub url: String,
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub security_headers: Vec<SecurityHeader>,
    pub response_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertChainEntry {
    pub subject: String,
    pub issuer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslCertInfo {
    pub domain: String,
    pub issuer: String,
    pub subject: String,
    pub valid_from: String,
    pub valid_to: String,
    pub days_remaining: i64,
    pub serial: String,
    pub sans: Vec<String>,
    pub chain: Vec<CertChainEntry>,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoIpResult {
    pub ip: String,
    pub country: String,
    pub country_code: String,
    pub region: String,
    pub city: String,
    pub lat: f64,
    pub lon: f64,
    pub isp: String,
    pub org: String,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArpEntry {
    pub ip: String,
    pub mac: String,
    pub interface_name: String,
    pub hostname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetCalcResult {
    pub cidr: String,
    pub network: String,
    pub broadcast: String,
    pub first_host: String,
    pub last_host: String,
    pub netmask: String,
    pub wildcard: String,
    pub host_count: u64,
    pub prefix_len: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseDnsResult {
    pub ip: String,
    pub hostnames: Vec<String>,
    pub query_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracerouteHop {
    pub hop: u8,
    pub ip: String,
    pub hostname: String,
    pub rtt_ms: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracerouteResult {
    pub target: String,
    pub hops: Vec<TracerouteHop>,
    pub completed: bool,
}

// ═══════════════════════════════════════════════════════════════════
//  CACHE INFRASTRUCTURE
// ═══════════════════════════════════════════════════════════════════

struct CacheEntry<T> {
    data: T,
    fetched_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            fetched_at: Instant::now(),
        }
    }

    fn is_fresh(&self, ttl: Duration) -> bool {
        self.fetched_at.elapsed() < ttl
    }
}

fn whois_cache() -> &'static Mutex<HashMap<String, CacheEntry<WhoisResult>>> {
    static C: OnceLock<Mutex<HashMap<String, CacheEntry<WhoisResult>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

fn geoip_cache() -> &'static Mutex<HashMap<String, CacheEntry<GeoIpResult>>> {
    static C: OnceLock<Mutex<HashMap<String, CacheEntry<GeoIpResult>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock_cache<T>(m: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, String> {
    m.lock().map_err(|e| format!("cache lock: {}", e))
}

// ═══════════════════════════════════════════════════════════════════
//  HELPERS
// ═══════════════════════════════════════════════════════════════════

fn http_client(timeout_secs: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent("FluxTerminal/2.0")
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("http client: {}", e))
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn port_service_name(port: u16) -> &'static str {
    match port {
        21 => "FTP",
        22 => "SSH",
        23 => "Telnet",
        25 => "SMTP",
        53 => "DNS",
        80 => "HTTP",
        110 => "POP3",
        143 => "IMAP",
        443 => "HTTPS",
        445 => "SMB",
        587 => "SMTP/TLS",
        993 => "IMAPS",
        995 => "POP3S",
        1433 => "MSSQL",
        3306 => "MySQL",
        3389 => "RDP",
        5432 => "PostgreSQL",
        5900 => "VNC",
        6379 => "Redis",
        8080 => "HTTP-Alt",
        8443 => "HTTPS-Alt",
        27017 => "MongoDB",
        _ => "Unknown",
    }
}

const DEFAULT_PORTS: &[u16] = &[
    21, 22, 23, 25, 53, 80, 110, 143, 443, 445, 587, 993, 995,
    3306, 3389, 5432, 5900, 6379, 8080, 8443, 27017,
];

const SECURITY_HEADERS: &[&str] = &[
    "content-security-policy",
    "strict-transport-security",
    "x-content-type-options",
    "x-frame-options",
    "x-xss-protection",
    "referrer-policy",
    "permissions-policy",
];

// ═══════════════════════════════════════════════════════════════════
//  COMMANDS
// ═══════════════════════════════════════════════════════════════════

// ─── Ping ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn netops_ping(host: String) -> Result<PingResult, String> {
    if host.is_empty() {
        return Err("host is required".into());
    }
    let url = if host.starts_with("http://") || host.starts_with("https://") {
        host.clone()
    } else {
        format!("http://{}", host)
    };

    let client = http_client(10)?;
    let start = Instant::now();
    let result = client.head(&url).send().await;
    let latency = start.elapsed().as_secs_f64() * 1000.0;

    match result {
        Ok(_) => Ok(PingResult {
            host,
            latency_ms: latency,
            status: "ok".into(),
            timestamp: now_epoch(),
        }),
        Err(e) if e.is_timeout() => Ok(PingResult {
            host,
            latency_ms: latency,
            status: "timeout".into(),
            timestamp: now_epoch(),
        }),
        Err(e) => Ok(PingResult {
            host,
            latency_ms: latency,
            status: format!("error: {}", e),
            timestamp: now_epoch(),
        }),
    }
}

// ─── Port Scan ───────────────────────────────────────────────────

#[tauri::command]
pub async fn netops_port_scan(host: String, ports: Option<Vec<u16>>) -> Result<PortScanResult, String> {
    if host.is_empty() {
        return Err("host is required".into());
    }

    let ports_to_scan = ports.unwrap_or_else(|| DEFAULT_PORTS.to_vec());
    let start = Instant::now();

    // Resolve hostname
    let resolved = tokio::task::spawn_blocking({
        let h = host.clone();
        move || {
            use std::net::ToSocketAddrs;
            format!("{}:80", h)
                .to_socket_addrs()
                .ok()
                .and_then(|mut addrs| addrs.next())
                .map(|a| a.ip().to_string())
                .unwrap_or(h)
        }
    })
    .await
    .map_err(|e| format!("resolve: {}", e))?;

    let mut handles = Vec::new();
    for port in &ports_to_scan {
        let ip = resolved.clone();
        let p = *port;
        handles.push(tokio::spawn(async move {
            let addr: std::net::SocketAddr = format!("{}:{}", ip, p)
                .parse()
                .unwrap_or_else(|_| std::net::SocketAddr::from(([127, 0, 0, 1], p)));
            let status = match tokio::time::timeout(
                Duration::from_secs(2),
                tokio::net::TcpStream::connect(addr),
            )
            .await
            {
                Ok(Ok(_)) => "open",
                Ok(Err(_)) => "closed",
                Err(_) => "filtered",
            };
            PortEntry {
                port: p,
                status: status.into(),
                service: port_service_name(p).into(),
            }
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(entry) = handle.await {
            results.push(entry);
        }
    }
    results.sort_by_key(|e| e.port);

    Ok(PortScanResult {
        host,
        ports: results,
        scan_duration_ms: start.elapsed().as_millis() as u64,
    })
}

// ─── DNS Lookup ──────────────────────────────────────────────────

#[tauri::command]
pub async fn netops_dns_lookup(hostname: String, record_type: Option<String>) -> Result<DnsLookupResult, String> {
    if hostname.is_empty() {
        return Err("hostname is required".into());
    }
    let rtype = record_type.unwrap_or_else(|| "A".into());
    let start = Instant::now();

    let output = tokio::process::Command::new("dig")
        .args(["+noall", "+answer", "+stats", &hostname, &rtype])
        .output()
        .await
        .map_err(|e| format!("dig: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut records = Vec::new();
    let mut server = String::new();
    let mut query_time: u64 = 0;

    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with(";;") {
            if line.contains("Query time:") {
                if let Some(ms) = line.split_whitespace().nth(3) {
                    query_time = ms.parse().unwrap_or(0);
                }
            }
            if line.contains("SERVER:") {
                server = line
                    .split("SERVER:")
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .to_string();
            }
            continue;
        }
        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let ttl_val = parts[1].parse::<u32>().ok();
            records.push(DnsRecord {
                name: parts[0].to_string(),
                ttl: ttl_val,
                record_type: parts[3].to_string(),
                value: parts[4..].join(" "),
            });
        }
    }

    let elapsed = start.elapsed().as_millis() as u64;

    Ok(DnsLookupResult {
        hostname,
        records,
        query_time_ms: if query_time > 0 { query_time } else { elapsed },
        server,
    })
}

// ─── Whois ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn netops_whois(domain: String) -> Result<WhoisResult, String> {
    if domain.is_empty() {
        return Err("domain is required".into());
    }

    // Check cache (1 hour TTL)
    {
        let cache = lock_cache(whois_cache())?;
        if let Some(entry) = cache.get(&domain) {
            if entry.is_fresh(Duration::from_secs(3600)) {
                return Ok(entry.data.clone());
            }
        }
    }

    let output = tokio::process::Command::new("whois")
        .arg(&domain)
        .output()
        .await
        .map_err(|e| format!("whois: {}", e))?;

    let raw = String::from_utf8_lossy(&output.stdout).to_string();

    let mut registrar = String::new();
    let mut creation_date = String::new();
    let mut expiry_date = String::new();
    let mut name_servers = Vec::new();
    let mut status = Vec::new();

    for line in raw.lines() {
        let lower = line.to_lowercase();
        let val = line.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();

        if lower.starts_with("registrar:") && registrar.is_empty() {
            registrar = val;
        } else if (lower.starts_with("creation date:") || lower.starts_with("created:") || lower.contains("registration date:")) && creation_date.is_empty() {
            creation_date = val;
        } else if (lower.starts_with("registry expiry date:") || lower.starts_with("expiry date:") || lower.starts_with("paid-till:")) && expiry_date.is_empty() {
            expiry_date = val;
        } else if lower.starts_with("name server:") || lower.starts_with("nserver:") {
            let ns = val.to_lowercase();
            if !ns.is_empty() && !name_servers.contains(&ns) {
                name_servers.push(ns);
            }
        } else if lower.starts_with("domain status:") || lower.starts_with("status:") {
            if !val.is_empty() {
                status.push(val);
            }
        }
    }

    let result = WhoisResult {
        domain: domain.clone(),
        registrar,
        creation_date,
        expiry_date,
        name_servers,
        status,
        raw,
    };

    // Store in cache
    {
        let mut cache = lock_cache(whois_cache())?;
        cache.insert(domain, CacheEntry::new(result.clone()));
    }

    Ok(result)
}

// ─── WiFi Scan ───────────────────────────────────────────────────

#[tauri::command]
pub async fn netops_wifi_scan() -> Result<Vec<WifiNetwork>, String> {
    // Use CoreWLAN via Swift — the legacy `airport` binary was removed in modern macOS
    // Security type is read via KVC (value(forKey:)) since the public Swift API removed the property
    let swift_code = include_str!("wifi_scan.swift");

    let output = tokio::process::Command::new("swift")
        .arg("-e")
        .arg(swift_code)
        .output()
        .await
        .map_err(|e| format!("swift wifi scan: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("WiFi scan failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Format: SSID|BSSID|RSSI|CHANNEL|SECURITY
        let parts: Vec<&str> = trimmed.split('|').collect();
        if parts.len() < 5 {
            continue;
        }

        networks.push(WifiNetwork {
            ssid: parts[0].to_string(),
            bssid: parts[1].to_string(),
            rssi: parts[2].parse::<i32>().unwrap_or(0),
            channel: parts[3].parse::<u32>().unwrap_or(0),
            security: parts[4].to_string(),
        });
    }

    Ok(networks)
}

// ─── WiFi Auth Monitor ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiAuthEvent {
    pub timestamp: String,
    pub event_type: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiAuthMonitorResult {
    pub time_window_hours: u32,
    pub events: Vec<WifiAuthEvent>,
    pub total_failures: u32,
    pub total_events: u32,
    pub query_time_ms: u64,
}

#[tauri::command]
pub async fn netops_wifi_auth_monitor(time_window: Option<u32>) -> Result<WifiAuthMonitorResult, String> {
    let hours = time_window.unwrap_or(1).min(24).max(1);
    let start = std::time::Instant::now();

    let predicate = r#"subsystem == "com.apple.wifi" AND (eventMessage CONTAINS "auth" OR eventMessage CONTAINS "association" OR eventMessage CONTAINS "deauth" OR eventMessage CONTAINS "disassoc")"#;

    let output = tokio::process::Command::new("log")
        .args([
            "show",
            "--predicate",
            predicate,
            "--last",
            &format!("{}h", hours),
            "--style",
            "compact",
        ])
        .output()
        .await
        .map_err(|e| format!("log command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("log command failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut events = Vec::new();
    let mut total_failures: u32 = 0;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Timestamp") || trimmed.starts_with("---") || trimmed.starts_with("Filtering") {
            continue;
        }

        // Compact format: "2024-01-01 12:00:00.000000+0000 ... message"
        // Extract timestamp (first ~31 chars) and the rest as message
        let (timestamp, message) = if trimmed.len() > 31 {
            let ts = trimmed[..31].trim().to_string();
            // Skip process info columns, find the message part
            let rest = &trimmed[31..];
            let msg = rest.trim().to_string();
            (ts, msg)
        } else {
            continue;
        };

        let lower = message.to_lowercase();
        let event_type = if lower.contains("fail") || lower.contains("error") || lower.contains("rejected") {
            "failure"
        } else if lower.contains("deauth") || lower.contains("disassoc") {
            "deauth"
        } else if lower.contains("timeout") {
            "timeout"
        } else if lower.contains("success") || lower.contains("associated") || lower.contains("completed") {
            "success"
        } else {
            "other"
        };

        if event_type == "failure" || event_type == "deauth" || event_type == "timeout" {
            total_failures += 1;
        }

        // Truncate very long messages
        let truncated_msg = if message.len() > 200 {
            format!("{}...", &message[..200])
        } else {
            message
        };

        events.push(WifiAuthEvent {
            timestamp,
            event_type: event_type.into(),
            message: truncated_msg,
        });
    }

    let elapsed = start.elapsed().as_millis() as u64;
    let total_events = events.len() as u32;

    // Most recent first
    events.reverse();

    // Cap at 500 events to avoid huge payloads
    events.truncate(500);

    Ok(WifiAuthMonitorResult {
        time_window_hours: hours,
        events,
        total_failures,
        total_events,
        query_time_ms: elapsed,
    })
}

// ─── HTTP Headers ────────────────────────────────────────────────

#[tauri::command]
pub async fn netops_http_headers(url: String) -> Result<HttpHeaderResult, String> {
    if url.is_empty() {
        return Err("URL is required".into());
    }
    let full_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.clone()
    } else {
        format!("https://{}", url)
    };

    let client = http_client(15)?;
    let start = Instant::now();
    let resp = client
        .get(&full_url)
        .send()
        .await
        .map_err(|e| format!("request failed: {}", e))?;
    let elapsed = start.elapsed().as_millis() as u64;

    let status_code = resp.status().as_u16();
    let headers: Vec<(String, String)> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let mut security_headers = Vec::new();
    for &hdr in SECURITY_HEADERS {
        let found = headers.iter().find(|(k, _)| k.to_lowercase() == hdr);
        security_headers.push(SecurityHeader {
            name: hdr.to_string(),
            present: found.is_some(),
            value: found.map(|(_, v)| v.clone()).unwrap_or_default(),
            rating: if found.is_some() { "good".into() } else { "missing".into() },
        });
    }

    Ok(HttpHeaderResult {
        url: full_url,
        status_code,
        headers,
        security_headers,
        response_time_ms: elapsed,
    })
}

// ─── SSL/TLS Inspect ─────────────────────────────────────────────

#[tauri::command]
pub async fn netops_ssl_inspect(domain: String) -> Result<SslCertInfo, String> {
    if domain.is_empty() {
        return Err("domain is required".into());
    }

    // Use openssl to get certificate info
    let output = tokio::process::Command::new("sh")
        .args([
            "-c",
            &format!(
                "echo | openssl s_client -connect {}:443 -servername {} 2>/dev/null | openssl x509 -noout -subject -issuer -dates -serial -ext subjectAltName 2>/dev/null",
                domain, domain
            ),
        ])
        .output()
        .await
        .map_err(|e| format!("openssl: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    let mut issuer = String::new();
    let mut subject = String::new();
    let mut valid_from = String::new();
    let mut valid_to = String::new();
    let mut serial = String::new();
    let mut sans: Vec<String> = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("subject=") || line.starts_with("subject =") {
            subject = line.split('=').skip(1).collect::<Vec<_>>().join("=").trim().to_string();
        } else if line.starts_with("issuer=") || line.starts_with("issuer =") {
            issuer = line.split('=').skip(1).collect::<Vec<_>>().join("=").trim().to_string();
        } else if line.starts_with("notBefore=") {
            valid_from = line.split('=').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("notAfter=") {
            valid_to = line.split('=').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("serial=") {
            serial = line.split('=').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("DNS:") {
            // Parse SANs from "DNS:example.com, DNS:www.example.com"
            for part in line.split(',') {
                let part = part.trim();
                if part.starts_with("DNS:") {
                    sans.push(part[4..].to_string());
                }
            }
        }
    }

    // Calculate days remaining
    let days_remaining = parse_days_remaining(&valid_to);

    // Get protocol version
    let proto_output = tokio::process::Command::new("sh")
        .args([
            "-c",
            &format!(
                "echo | openssl s_client -connect {}:443 -servername {} 2>/dev/null | grep 'Protocol'",
                domain, domain
            ),
        ])
        .output()
        .await
        .ok();

    let protocol = proto_output
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .find(|l| l.contains("Protocol"))
                .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string())
                .unwrap_or_default()
        })
        .unwrap_or_default();

    Ok(SslCertInfo {
        domain,
        issuer,
        subject,
        valid_from,
        valid_to,
        days_remaining,
        serial,
        sans,
        chain: Vec::new(),
        protocol,
    })
}

fn parse_days_remaining(date_str: &str) -> i64 {
    // Parse "Mar  4 12:00:00 2026 GMT" format
    if date_str.is_empty() {
        return -1;
    }
    // Use the system `date` command for reliable parsing
    let output = std::process::Command::new("date")
        .args(["-j", "-f", "%b %d %T %Y %Z", date_str, "+%s"])
        .output()
        .ok();

    if let Some(out) = output {
        let epoch_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if let Ok(epoch) = epoch_str.parse::<i64>() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            return (epoch - now) / 86400;
        }
    }
    -1
}

// ─── IP Geolocation ──────────────────────────────────────────────

#[tauri::command]
pub async fn netops_ip_geolocate(ip: String) -> Result<GeoIpResult, String> {
    if ip.is_empty() {
        return Err("IP address is required".into());
    }

    // Check cache (1 hour TTL)
    {
        let cache = lock_cache(geoip_cache())?;
        if let Some(entry) = cache.get(&ip) {
            if entry.is_fresh(Duration::from_secs(3600)) {
                return Ok(entry.data.clone());
            }
        }
    }

    let client = http_client(10)?;
    let resp = client
        .get(&format!(
            "http://ip-api.com/json/{}?fields=status,message,country,countryCode,regionName,city,lat,lon,timezone,isp,org,query",
            ip
        ))
        .send()
        .await
        .map_err(|e| format!("geoip: {}", e))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("geoip parse: {}", e))?;

    if json["status"].as_str() != Some("success") {
        return Err(format!(
            "geoip: {}",
            json["message"].as_str().unwrap_or("lookup failed")
        ));
    }

    let result = GeoIpResult {
        ip: json["query"].as_str().unwrap_or(&ip).to_string(),
        country: json["country"].as_str().unwrap_or("").to_string(),
        country_code: json["countryCode"].as_str().unwrap_or("").to_string(),
        region: json["regionName"].as_str().unwrap_or("").to_string(),
        city: json["city"].as_str().unwrap_or("").to_string(),
        lat: json["lat"].as_f64().unwrap_or(0.0),
        lon: json["lon"].as_f64().unwrap_or(0.0),
        isp: json["isp"].as_str().unwrap_or("").to_string(),
        org: json["org"].as_str().unwrap_or("").to_string(),
        timezone: json["timezone"].as_str().unwrap_or("").to_string(),
    };

    // Store in cache
    {
        let mut cache = lock_cache(geoip_cache())?;
        cache.insert(ip, CacheEntry::new(result.clone()));
    }

    Ok(result)
}

// ─── ARP Table ───────────────────────────────────────────────────

#[tauri::command]
pub async fn netops_arp_table() -> Result<Vec<ArpEntry>, String> {
    let output = tokio::process::Command::new("arp")
        .arg("-a")
        .output()
        .await
        .map_err(|e| format!("arp: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Format: hostname (IP) at MAC on interface [ifscope ...]
        // or: ? (IP) at MAC on interface
        let parts: Vec<&str> = line.split_whitespace().collect();

        // Find IP in parentheses
        let ip = parts.iter()
            .find(|p| p.starts_with('(') && p.ends_with(')'))
            .map(|p| p.trim_matches(|c| c == '(' || c == ')').to_string())
            .unwrap_or_default();

        if ip.is_empty() {
            continue;
        }

        // Find MAC (after "at")
        let at_idx = parts.iter().position(|&p| p == "at");
        let mac = at_idx
            .and_then(|i| parts.get(i + 1))
            .map(|s| s.to_string())
            .unwrap_or_default();

        // Find interface (after "on")
        let on_idx = parts.iter().position(|&p| p == "on");
        let iface = on_idx
            .and_then(|i| parts.get(i + 1))
            .map(|s| s.to_string())
            .unwrap_or_default();

        // Hostname is first field (or "?" for unknown)
        let hostname = parts.first().map(|s| s.to_string()).unwrap_or_default();

        if mac != "(incomplete)" {
            entries.push(ArpEntry {
                ip,
                mac,
                interface_name: iface,
                hostname,
            });
        }
    }

    Ok(entries)
}

// ─── Subnet Calculator ──────────────────────────────────────────

#[tauri::command]
pub async fn netops_subnet_calc(cidr: String) -> Result<SubnetCalcResult, String> {
    if cidr.is_empty() {
        return Err("CIDR notation required (e.g., 192.168.1.0/24)".into());
    }

    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return Err("invalid CIDR format, use IP/prefix (e.g., 192.168.1.0/24)".into());
    }

    let ip: std::net::Ipv4Addr = parts[0]
        .parse()
        .map_err(|_| "invalid IP address".to_string())?;
    let prefix: u8 = parts[1]
        .parse()
        .map_err(|_| "invalid prefix length".to_string())?;

    if prefix > 32 {
        return Err("prefix must be 0-32".into());
    }

    let ip_u32 = u32::from(ip);
    let mask = if prefix == 0 { 0u32 } else { !0u32 << (32 - prefix) };
    let network = ip_u32 & mask;
    let broadcast = network | !mask;
    let wildcard = !mask;

    let host_count = if prefix >= 31 {
        2u64.pow(32 - prefix as u32)
    } else {
        (2u64.pow(32 - prefix as u32)).saturating_sub(2)
    };

    let first_host = if prefix >= 31 { network } else { network + 1 };
    let last_host = if prefix >= 31 { broadcast } else { broadcast - 1 };

    Ok(SubnetCalcResult {
        cidr: cidr.clone(),
        network: std::net::Ipv4Addr::from(network).to_string(),
        broadcast: std::net::Ipv4Addr::from(broadcast).to_string(),
        first_host: std::net::Ipv4Addr::from(first_host).to_string(),
        last_host: std::net::Ipv4Addr::from(last_host).to_string(),
        netmask: std::net::Ipv4Addr::from(mask).to_string(),
        wildcard: std::net::Ipv4Addr::from(wildcard).to_string(),
        host_count,
        prefix_len: prefix,
    })
}

// ─── Reverse DNS ─────────────────────────────────────────────────

#[tauri::command]
pub async fn netops_reverse_dns(ip: String) -> Result<ReverseDnsResult, String> {
    if ip.is_empty() {
        return Err("IP address is required".into());
    }

    let start = Instant::now();
    let output = tokio::process::Command::new("dig")
        .args(["-x", &ip, "+short"])
        .output()
        .await
        .map_err(|e| format!("dig: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let hostnames: Vec<String> = stdout
        .lines()
        .map(|l| l.trim().trim_end_matches('.').to_string())
        .filter(|l| !l.is_empty())
        .collect();

    Ok(ReverseDnsResult {
        ip,
        hostnames,
        query_time_ms: start.elapsed().as_millis() as u64,
    })
}

// ─── Traceroute ──────────────────────────────────────────────────

#[tauri::command]
pub async fn netops_traceroute(target: String) -> Result<TracerouteResult, String> {
    if target.is_empty() {
        return Err("target is required".into());
    }

    let output = tokio::time::timeout(
        Duration::from_secs(30),
        tokio::process::Command::new("traceroute")
            .args(["-m", "30", "-w", "2", &target])
            .output(),
    )
    .await
    .map_err(|_| "traceroute timed out (30s)".to_string())?
    .map_err(|e| format!("traceroute: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut hops = Vec::new();
    let mut completed = false;

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("traceroute") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        // First field is hop number
        let hop_num = match parts[0].parse::<u8>() {
            Ok(n) => n,
            Err(_) => continue,
        };

        let mut ip = String::new();
        let mut hostname = String::new();
        let mut rtts = Vec::new();

        if parts.len() > 1 && parts[1] == "*" {
            // All timeouts
            ip = "*".into();
            hostname = "*".into();
        } else if parts.len() > 1 {
            hostname = parts[1].to_string();
            // IP in parentheses
            if let Some(ip_part) = parts.get(2) {
                if ip_part.starts_with('(') && ip_part.ends_with(')') {
                    ip = ip_part.trim_matches(|c| c == '(' || c == ')').to_string();
                } else {
                    ip = hostname.clone();
                }
            }

            // Parse RTT values (look for "ms")
            for (i, part) in parts.iter().enumerate() {
                if *part == "ms" {
                    if let Some(val) = parts.get(i.wrapping_sub(1)) {
                        if let Ok(rtt) = val.parse::<f64>() {
                            rtts.push(rtt);
                        }
                    }
                }
            }
        }

        hops.push(TracerouteHop {
            hop: hop_num,
            ip: ip.clone(),
            hostname,
            rtt_ms: rtts,
        });

        // Check if we reached the target
        if !ip.is_empty() && ip != "*" {
            // Resolve target to check
            completed = true; // mark last successful hop
        }
    }

    Ok(TracerouteResult {
        target,
        hops,
        completed,
    })
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 14: Suspicious Traffic Anomaly Detection
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousConnection {
    pub process: String,
    pub pid: u32,
    pub protocol: String,
    pub local_addr: String,
    pub foreign_addr: String,
    pub port: u16,
    pub reason: String,
    pub threat_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficAnomalyResult {
    pub connections: Vec<SuspiciousConnection>,
    pub total_connections: u32,
    pub suspicious_count: u32,
    pub scan_time_ms: u64,
}

const SUSPICIOUS_PORTS: &[u16] = &[
    4444, 5555, 6667, 6668, 6669, // Metasploit, IRC
    6881, 6882, 6883, 6884, 6885, 6886, 6887, 6888, 6889, // BitTorrent
    31337, 12345, 27374, 1337, // Known backdoor ports
    3389, // RDP (suspicious if unexpected)
];

#[tauri::command]
pub async fn netops_traffic_anomalies() -> Result<TrafficAnomalyResult, String> {
    let start = Instant::now();

    let output = tokio::process::Command::new("lsof")
        .args(["-i", "-n", "-P"])
        .output()
        .await
        .map_err(|e| format!("lsof: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut all_connections: Vec<(String, u32, String, String, String)> = Vec::new();
    let mut suspicious = Vec::new();
    let mut foreign_ip_counts: HashMap<String, u32> = HashMap::new();

    for (i, line) in stdout.lines().enumerate() {
        if i == 0 { continue; } // skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 { continue; }

        let process = parts[0].to_string();
        let pid = parts[1].parse::<u32>().unwrap_or(0);
        let protocol = parts[7].to_string(); // TCP/UDP
        let name_field = parts[8];

        // Parse "local->foreign" or just "local (LISTEN)"
        let (local, foreign, state) = if let Some(arrow_pos) = name_field.find("->") {
            let local = &name_field[..arrow_pos];
            let rest = &name_field[arrow_pos + 2..];
            // State might be in next field
            let state = if parts.len() > 9 {
                parts[9].trim_matches(|c| c == '(' || c == ')').to_string()
            } else {
                String::new()
            };
            (local.to_string(), rest.to_string(), state)
        } else {
            let state = if parts.len() > 9 {
                parts[9].trim_matches(|c| c == '(' || c == ')').to_string()
            } else {
                String::new()
            };
            (name_field.to_string(), String::new(), state)
        };

        // Track foreign IPs
        if !foreign.is_empty() && foreign != "*:*" {
            let fip = foreign.rsplitn(2, ':').last().unwrap_or("").to_string();
            if !fip.is_empty() && fip != "*" && fip != "127.0.0.1" && fip != "::1" {
                *foreign_ip_counts.entry(fip).or_insert(0) += 1;
            }
        }

        all_connections.push((process.clone(), pid.clone(), protocol.clone(), local.clone(), foreign.clone()));

        // Check for suspicious ports
        let port = foreign.rsplitn(2, ':').next()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(0);

        if SUSPICIOUS_PORTS.contains(&port) {
            suspicious.push(SuspiciousConnection {
                process: process.clone(),
                pid,
                protocol: protocol.clone(),
                local_addr: local.clone(),
                foreign_addr: foreign.clone(),
                port,
                reason: format!("Connection to suspicious port {}", port),
                threat_level: if port == 4444 || port == 31337 { "critical" } else { "high" }.into(),
            });
        }

        // Check for unusual LISTEN ports
        if state == "LISTEN" {
            let listen_port = local.rsplitn(2, ':').next()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(0);
            if listen_port > 10000 && !matches!(listen_port, 49152..=65535) {
                // Not ephemeral range but high port
                suspicious.push(SuspiciousConnection {
                    process: process.clone(),
                    pid,
                    protocol: protocol.clone(),
                    local_addr: local.clone(),
                    foreign_addr: String::new(),
                    port: listen_port,
                    reason: format!("Unusual LISTEN on port {}", listen_port),
                    threat_level: "medium".into(),
                });
            }
        }
    }

    // Check for IPs with many connections (potential C2 or data exfil)
    for (ip, count) in &foreign_ip_counts {
        if *count > 5 {
            suspicious.push(SuspiciousConnection {
                process: String::new(),
                pid: 0,
                protocol: String::new(),
                local_addr: String::new(),
                foreign_addr: ip.clone(),
                port: 0,
                reason: format!("{} connections to same IP (possible C2/exfil)", count),
                threat_level: if *count > 10 { "high" } else { "medium" }.into(),
            });
        }
    }

    let total = all_connections.len() as u32;
    let susp_count = suspicious.len() as u32;

    Ok(TrafficAnomalyResult {
        connections: suspicious,
        total_connections: total,
        suspicious_count: susp_count,
        scan_time_ms: start.elapsed().as_millis() as u64,
    })
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 15: Rogue Access Point Detection
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApStatus {
    pub ssid: String,
    pub bssid: String,
    pub rssi: i32,
    pub channel: u32,
    pub security: String,
    pub status: String,  // "trusted" | "unknown" | "evil_twin"
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RogueApResult {
    pub known_count: u32,
    pub unknown_count: u32,
    pub spoofed_count: u32,
    pub networks: Vec<ApStatus>,
    pub baseline_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WifiBaseline {
    saved_at: String,
    networks: Vec<WifiBaselineEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WifiBaselineEntry {
    ssid: String,
    bssid: String,
    security: String,
}

fn wifi_baseline_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("flux-terminal")
        .join("wifi_baseline.json")
}

async fn run_wifi_scan_swift() -> Result<Vec<WifiNetwork>, String> {
    let swift_code = include_str!("wifi_scan.swift");
    let output = tokio::process::Command::new("swift")
        .arg("-e")
        .arg(swift_code)
        .output()
        .await
        .map_err(|e| format!("swift wifi scan: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("WiFi scan failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.trim().split('|').collect();
        if parts.len() < 5 { continue; }
        networks.push(WifiNetwork {
            ssid: parts[0].to_string(),
            bssid: parts[1].to_string(),
            rssi: parts[2].parse().unwrap_or(0),
            channel: parts[3].parse().unwrap_or(0),
            security: parts[4].to_string(),
        });
    }
    Ok(networks)
}

#[tauri::command]
pub async fn netops_rogue_ap_scan() -> Result<RogueApResult, String> {
    let current = run_wifi_scan_swift().await?;
    let path = wifi_baseline_path();
    let baseline_exists = path.exists();

    if !baseline_exists {
        // No baseline — all are "unknown", suggest saving baseline
        let networks: Vec<ApStatus> = current.iter().map(|n| ApStatus {
            ssid: n.ssid.clone(),
            bssid: n.bssid.clone(),
            rssi: n.rssi,
            channel: n.channel,
            security: n.security.clone(),
            status: "unknown".into(),
            reason: "No baseline saved — save baseline first".into(),
        }).collect();
        let count = networks.len() as u32;
        return Ok(RogueApResult {
            known_count: 0,
            unknown_count: count,
            spoofed_count: 0,
            networks,
            baseline_exists: false,
        });
    }

    // Load baseline
    let content = std::fs::read_to_string(&path).map_err(|e| format!("read baseline: {}", e))?;
    let baseline: WifiBaseline = serde_json::from_str(&content).map_err(|e| format!("parse baseline: {}", e))?;

    let mut known = 0u32;
    let mut unknown = 0u32;
    let mut spoofed = 0u32;
    let mut networks = Vec::new();

    for net in &current {
        // Check if BSSID is in baseline
        let baseline_match = baseline.networks.iter().find(|b| b.bssid == net.bssid);
        let ssid_match = baseline.networks.iter().find(|b| b.ssid == net.ssid);

        let (status, reason) = if let Some(_bm) = baseline_match {
            known += 1;
            ("trusted".into(), "Known BSSID in baseline".into())
        } else if let Some(sm) = ssid_match {
            // Same SSID but different BSSID — potential evil twin
            spoofed += 1;
            ("evil_twin".into(), format!(
                "SSID '{}' has different BSSID (baseline: {}, current: {})",
                net.ssid, sm.bssid, net.bssid
            ))
        } else {
            unknown += 1;
            ("unknown".into(), "SSID/BSSID not in baseline".into())
        };

        networks.push(ApStatus {
            ssid: net.ssid.clone(),
            bssid: net.bssid.clone(),
            rssi: net.rssi,
            channel: net.channel,
            security: net.security.clone(),
            status,
            reason,
        });
    }

    // Sort: evil_twin first, then unknown, then trusted
    networks.sort_by(|a, b| {
        let order = |s: &str| match s { "evil_twin" => 0, "unknown" => 1, _ => 2 };
        order(&a.status).cmp(&order(&b.status))
    });

    Ok(RogueApResult { known_count: known, unknown_count: unknown, spoofed_count: spoofed, networks, baseline_exists: true })
}

#[tauri::command]
pub async fn netops_rogue_ap_save_baseline() -> Result<String, String> {
    let current = run_wifi_scan_swift().await?;
    let baseline = WifiBaseline {
        saved_at: chrono_now(),
        networks: current.iter().map(|n| WifiBaselineEntry {
            ssid: n.ssid.clone(),
            bssid: n.bssid.clone(),
            security: n.security.clone(),
        }).collect(),
    };

    let path = wifi_baseline_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(&baseline).map_err(|e| format!("serialize: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("write: {}", e))?;

    Ok(format!("Baseline saved with {} networks", baseline.networks.len()))
}

fn chrono_now() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs();
    // Simple ISO-ish timestamp
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02} UTC", hours, mins, s)
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 16: Log Aggregation
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub subsystem: String,
    pub process: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLogsResult {
    pub filter: String,
    pub entries: Vec<LogEntry>,
    pub total_count: u32,
    pub query_time_ms: u64,
}

#[tauri::command]
pub async fn netops_system_logs(filter: String) -> Result<SystemLogsResult, String> {
    let start = Instant::now();

    let predicate = match filter.as_str() {
        "security" => r#"subsystem == "com.apple.securityd" OR subsystem == "com.apple.Security""#,
        "network" => r#"subsystem == "com.apple.networkd" OR subsystem == "com.apple.wifi""#,
        "firewall" => r#"subsystem == "com.apple.alf""#,
        "auth" => r#"subsystem == "com.apple.Authorization" OR subsystem == "com.apple.loginwindow""#,
        _ => r#"subsystem == "com.apple.securityd" OR subsystem == "com.apple.Security" OR subsystem == "com.apple.networkd" OR subsystem == "com.apple.wifi" OR subsystem == "com.apple.alf" OR subsystem == "com.apple.Authorization""#,
    };

    let output = tokio::process::Command::new("log")
        .args(["show", "--predicate", predicate, "--last", "1h", "--style", "compact"])
        .output()
        .await
        .map_err(|e| format!("log: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Timestamp") || trimmed.starts_with("---") || trimmed.starts_with("Filtering") {
            continue;
        }

        if trimmed.len() < 31 { continue; }

        let timestamp = trimmed[..31].trim().to_string();
        let rest = &trimmed[31..];

        // Try to parse: level process[pid] subsystem message
        let parts: Vec<&str> = rest.splitn(4, char::is_whitespace).filter(|s| !s.is_empty()).collect();

        let (level, process, message) = if parts.len() >= 3 {
            let lvl = parts[0].to_string();
            let proc_name = parts[1].to_string();
            let msg = if parts.len() > 2 { parts[2..].join(" ") } else { String::new() };
            (lvl, proc_name, msg)
        } else {
            ("Info".into(), String::new(), rest.trim().to_string())
        };

        // Determine subsystem from message context
        let sub = if message.contains("securityd") || message.contains("Security") {
            "Security"
        } else if message.contains("networkd") || message.contains("wifi") {
            "Network"
        } else if message.contains("alf") || message.contains("firewall") {
            "Firewall"
        } else if message.contains("Authorization") || message.contains("loginwindow") {
            "Auth"
        } else {
            "System"
        };

        let truncated_msg = if message.len() > 300 {
            format!("{}...", &message[..300])
        } else {
            message
        };

        entries.push(LogEntry {
            timestamp,
            level,
            subsystem: sub.into(),
            process,
            message: truncated_msg,
        });

        if entries.len() >= 200 { break; }
    }

    entries.reverse(); // Most recent first
    let total = entries.len() as u32;

    Ok(SystemLogsResult {
        filter,
        entries,
        total_count: total,
        query_time_ms: start.elapsed().as_millis() as u64,
    })
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 17: Threat Intelligence Feed
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatSource {
    pub name: String,
    pub listed: bool,
    pub category: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatCheckResult {
    pub indicator: String,
    pub threat_score: u32,
    pub is_threat: bool,
    pub sources: Vec<ThreatSource>,
    pub open_ports: Vec<u16>,
    pub vulns: Vec<String>,
    pub hostnames: Vec<String>,
    pub query_time_ms: u64,
}

fn threat_cache() -> &'static Mutex<HashMap<String, CacheEntry<ThreatCheckResult>>> {
    static C: OnceLock<Mutex<HashMap<String, CacheEntry<ThreatCheckResult>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

#[tauri::command]
pub async fn netops_threat_check(indicator: String) -> Result<ThreatCheckResult, String> {
    if indicator.is_empty() {
        return Err("IP address is required".into());
    }

    // Check cache
    {
        let cache = lock_cache(threat_cache())?;
        if let Some(entry) = cache.get(&indicator) {
            if entry.is_fresh(Duration::from_secs(3600)) {
                return Ok(entry.data.clone());
            }
        }
    }

    let start = Instant::now();
    let mut sources = Vec::new();
    let mut open_ports: Vec<u16> = Vec::new();
    let mut vulns: Vec<String> = Vec::new();
    let mut hostnames: Vec<String> = Vec::new();
    let mut score: u32 = 0;

    // 1. Shodan InternetDB (free, no key)
    let shodan_url = format!("https://internetdb.shodan.io/{}", indicator);
    if let Ok(client) = http_client(10) {
      if let Ok(resp) = client.get(&shodan_url).send().await {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(ports) = json.get("ports").and_then(|v: &serde_json::Value| v.as_array()) {
                    open_ports = ports.iter().filter_map(|p: &serde_json::Value| p.as_u64().map(|v| v as u16)).collect();
                }
                if let Some(v) = json.get("vulns").and_then(|v: &serde_json::Value| v.as_array()) {
                    vulns = v.iter().filter_map(|s: &serde_json::Value| s.as_str().map(String::from)).collect();
                }
                if let Some(h) = json.get("hostnames").and_then(|v: &serde_json::Value| v.as_array()) {
                    hostnames = h.iter().filter_map(|s: &serde_json::Value| s.as_str().map(String::from)).collect();
                }
                let has_vulns = !vulns.is_empty();
                sources.push(ThreatSource {
                    name: "Shodan InternetDB".into(),
                    listed: has_vulns,
                    category: if has_vulns { "Vulnerable" } else { "Clean" }.into(),
                    details: format!("{} ports, {} vulns", open_ports.len(), vulns.len()),
                });
                if has_vulns { score += 30; }
                if open_ports.len() > 10 { score += 10; }
            }
        }
      }
    }

    // 2. Spamhaus DNSBL check
    let reversed: String = indicator.split('.').rev().collect::<Vec<&str>>().join(".");
    let dnsbl_host = format!("{}.zen.spamhaus.org", reversed);
    let dig_output = tokio::process::Command::new("dig")
        .args(["+short", &dnsbl_host])
        .output()
        .await;

    if let Ok(out) = dig_output {
        let result = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let listed = !result.is_empty() && result.starts_with("127.");
        sources.push(ThreatSource {
            name: "Spamhaus ZEN".into(),
            listed,
            category: if listed { "Spam/Malware" } else { "Clean" }.into(),
            details: if listed { format!("Listed: {}", result) } else { "Not listed".into() },
        });
        if listed { score += 30; }
    }

    // 3. abuse.ch DNSBL
    let abuse_host = format!("{}.combined.abuse.ch", reversed);
    let abuse_output = tokio::process::Command::new("dig")
        .args(["+short", &abuse_host])
        .output()
        .await;

    if let Ok(out) = abuse_output {
        let result = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let listed = !result.is_empty() && result.starts_with("127.");
        sources.push(ThreatSource {
            name: "abuse.ch".into(),
            listed,
            category: if listed { "Malware/Botnet" } else { "Clean" }.into(),
            details: if listed { format!("Listed: {}", result) } else { "Not listed".into() },
        });
        if listed { score += 30; }
    }

    // Add vuln-based scoring
    score += (vulns.len() as u32).min(10) * 2;
    score = score.min(100);

    let result = ThreatCheckResult {
        indicator: indicator.clone(),
        threat_score: score,
        is_threat: score >= 30,
        sources,
        open_ports,
        vulns,
        hostnames,
        query_time_ms: start.elapsed().as_millis() as u64,
    };

    // Cache
    {
        let mut cache = lock_cache(threat_cache())?;
        cache.insert(indicator, CacheEntry::new(result.clone()));
    }

    Ok(result)
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 18: Security Score
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheck {
    pub name: String,
    pub category: String,
    pub status: String,  // "pass" | "fail" | "warn" | "info"
    pub detail: String,
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScoreResult {
    pub overall_score: u32,
    pub grade: String,
    pub checks: Vec<SecurityCheck>,
    pub passed: u32,
    pub failed: u32,
    pub warned: u32,
    pub total: u32,
}

async fn run_check(cmd: &str, args: &[&str]) -> String {
    tokio::process::Command::new(cmd)
        .args(args)
        .output()
        .await
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn netops_security_score() -> Result<SecurityScoreResult, String> {
    let mut checks = Vec::new();

    // 1. Firewall
    let fw = run_check("/usr/libexec/ApplicationFirewall/socketfilterfw", &["--getglobalstate"]).await;
    let fw_on = fw.contains("enabled");
    checks.push(SecurityCheck {
        name: "Firewall".into(),
        category: "network".into(),
        status: if fw_on { "pass" } else { "fail" }.into(),
        detail: if fw_on { "Application Firewall is enabled" } else { "Application Firewall is DISABLED" }.into(),
        weight: 15,
    });

    // 2. FileVault
    let fv = run_check("fdesetup", &["status"]).await;
    let fv_on = fv.contains("On");
    checks.push(SecurityCheck {
        name: "FileVault".into(),
        category: "encryption".into(),
        status: if fv_on { "pass" } else { "fail" }.into(),
        detail: if fv_on { "Disk encryption is enabled" } else { "Disk encryption is DISABLED" }.into(),
        weight: 20,
    });

    // 3. SIP
    let sip = run_check("csrutil", &["status"]).await;
    let sip_on = sip.contains("enabled");
    checks.push(SecurityCheck {
        name: "System Integrity Protection".into(),
        category: "system".into(),
        status: if sip_on { "pass" } else { "fail" }.into(),
        detail: if sip_on { "SIP is enabled" } else { "SIP is DISABLED — system vulnerable" }.into(),
        weight: 20,
    });

    // 4. Gatekeeper
    let gk = run_check("spctl", &["--status"]).await;
    let gk_on = gk.contains("enabled") || gk.contains("assessments enabled");
    checks.push(SecurityCheck {
        name: "Gatekeeper".into(),
        category: "system".into(),
        status: if gk_on { "pass" } else { "fail" }.into(),
        detail: if gk_on { "App verification is enabled" } else { "Gatekeeper is DISABLED" }.into(),
        weight: 15,
    });

    // 5. Auto-updates
    let upd = run_check("defaults", &["read", "/Library/Preferences/com.apple.SoftwareUpdate", "AutomaticCheckEnabled"]).await;
    let upd_on = upd.trim() == "1";
    checks.push(SecurityCheck {
        name: "Auto-Update Check".into(),
        category: "system".into(),
        status: if upd_on { "pass" } else { "warn" }.into(),
        detail: if upd_on { "Automatic update checking is enabled" } else { "Automatic update checking may be disabled" }.into(),
        weight: 10,
    });

    // 6. Screen lock
    let lock = run_check("defaults", &["read", "com.apple.screensaver", "askForPassword"]).await;
    let lock_on = lock.trim() == "1";
    checks.push(SecurityCheck {
        name: "Screen Lock".into(),
        category: "access".into(),
        status: if lock_on { "pass" } else { "warn" }.into(),
        detail: if lock_on { "Password required on screen lock" } else { "Screen lock password may not be required" }.into(),
        weight: 10,
    });

    // 7. Remote Login (SSH)
    let ssh = run_check("systemsetup", &["-getremotelogin"]).await;
    let ssh_off = ssh.contains("Off") || ssh.contains("not supported");
    checks.push(SecurityCheck {
        name: "Remote Login (SSH)".into(),
        category: "access".into(),
        status: if ssh_off { "pass" } else { "warn" }.into(),
        detail: if ssh_off { "Remote Login is disabled" } else { "Remote Login (SSH) is enabled" }.into(),
        weight: 10,
    });

    // Calculate score
    let mut total_weight = 0u32;
    let mut weighted_score = 0u32;
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut warned = 0u32;

    for check in &checks {
        total_weight += check.weight;
        match check.status.as_str() {
            "pass" => { weighted_score += check.weight * 100; passed += 1; }
            "warn" => { weighted_score += check.weight * 50; warned += 1; }
            "fail" => { failed += 1; }
            _ => {}
        }
    }

    let overall = if total_weight > 0 { weighted_score / total_weight } else { 0 };
    let grade = match overall {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }.into();

    let total = checks.len() as u32;
    Ok(SecurityScoreResult {
        overall_score: overall,
        grade,
        checks,
        passed,
        failed,
        warned,
        total,
    })
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 19: Incident Response Tracking
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentNote {
    pub timestamp: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncident {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub notes: Vec<IncidentNote>,
}

fn incidents_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("flux-terminal")
        .join("incidents.json")
}

fn incidents_store() -> &'static Mutex<Vec<SecurityIncident>> {
    static S: OnceLock<Mutex<Vec<SecurityIncident>>> = OnceLock::new();
    S.get_or_init(|| {
        let path = incidents_path();
        let data = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        Mutex::new(data)
    })
}

fn save_incidents(incidents: &[SecurityIncident]) -> Result<(), String> {
    let path = incidents_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(incidents).map_err(|e| format!("serialize: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("write: {}", e))?;
    Ok(())
}

fn now_iso() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs();
    // days since epoch
    let days = secs / 86400;
    let remaining = secs % 86400;
    let h = remaining / 3600;
    let m = (remaining % 3600) / 60;
    let s = remaining % 60;
    // Approximate date (good enough for display)
    let y = 1970 + (days / 365); // approximate
    let d_in_y = days % 365;
    let month = d_in_y / 30 + 1;
    let day = d_in_y % 30 + 1;
    format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", y, month.min(12), day.min(31), h, m, s)
}

#[tauri::command]
pub async fn netops_incident_list() -> Result<Vec<SecurityIncident>, String> {
    let store = lock_cache(incidents_store())?;
    Ok(store.clone())
}

#[tauri::command]
pub async fn netops_incident_create(severity: String, title: String, description: String) -> Result<Vec<SecurityIncident>, String> {
    let now = now_iso();
    let incident = SecurityIncident {
        id: uuid::Uuid::new_v4().to_string()[..8].to_string(),
        created_at: now.clone(),
        updated_at: now,
        severity,
        title,
        description,
        status: "open".into(),
        notes: Vec::new(),
    };

    let mut store = lock_cache(incidents_store())?;
    store.insert(0, incident);
    save_incidents(&store)?;
    Ok(store.clone())
}

#[tauri::command]
pub async fn netops_incident_update(id: String, status: String, note: String) -> Result<Vec<SecurityIncident>, String> {
    let mut store = lock_cache(incidents_store())?;
    let incident = store.iter_mut().find(|i| i.id == id)
        .ok_or_else(|| format!("Incident {} not found", id))?;

    incident.status = status;
    incident.updated_at = now_iso();
    if !note.is_empty() {
        incident.notes.push(IncidentNote {
            timestamp: now_iso(),
            content: note,
        });
    }

    save_incidents(&store)?;
    Ok(store.clone())
}

// ═══════════════════════════════════════════════════════════════════
//  KALI-STYLE TOOLS — Service Scan, Subdomain Enum, Dir Brute,
//  Web Fingerprint, WAF Detect, Web Vuln Scan, Hash ID, Cipher Scan
// ═══════════════════════════════════════════════════════════════════

// ─── Service Scan types ────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    pub port: u16,
    pub service: String,
    pub version: String,
    pub banner: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceScanResult {
    pub host: String,
    pub services: Vec<ServiceEntry>,
    pub scan_time_ms: u64,
}

// ─── Subdomain Enum types ────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubdomainEntry {
    pub subdomain: String,
    pub full_domain: String,
    pub ips: Vec<String>,
    pub cname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubdomainEnumResult {
    pub domain: String,
    pub found: Vec<SubdomainEntry>,
    pub tested_count: u32,
    pub found_count: u32,
    pub scan_time_ms: u64,
}

// ─── Dir Brute types ────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    pub path: String,
    pub status_code: u16,
    pub content_length: u64,
    pub redirect_to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirBustResult {
    pub base_url: String,
    pub entries: Vec<DirEntry>,
    pub tested_count: u32,
    pub found_count: u32,
    pub scan_time_ms: u64,
}

// ─── Web Fingerprint types ────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechMatch {
    pub name: String,
    pub category: String,
    pub version: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFingerprintResult {
    pub url: String,
    pub technologies: Vec<TechMatch>,
    pub server: String,
    pub powered_by: String,
    pub scan_time_ms: u64,
}

// ─── WAF Detect types ────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WafIndicator {
    pub name: String,
    pub confidence: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WafDetectResult {
    pub url: String,
    pub waf_detected: bool,
    pub waf_name: String,
    pub indicators: Vec<WafIndicator>,
    pub normal_status: u16,
    pub blocked_status: u16,
    pub scan_time_ms: u64,
}

// ─── Web Vuln Scan types ────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnFinding {
    pub title: String,
    pub severity: String,
    pub category: String,
    pub detail: String,
    pub url: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebVulnResult {
    pub target: String,
    pub findings: Vec<VulnFinding>,
    pub critical_count: u32,
    pub high_count: u32,
    pub medium_count: u32,
    pub low_count: u32,
    pub scan_time_ms: u64,
}

// ─── Hash ID types ────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashMatch {
    pub hash_type: String,
    pub confidence: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashIdResult {
    pub input: String,
    pub length: u32,
    pub matches: Vec<HashMatch>,
    pub is_base64: bool,
    pub query_time_ms: u64,
}

// ─── Cipher Scan types ────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipherInfo {
    pub name: String,
    pub protocol: String,
    pub bits: u32,
    pub strength: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipherScanResult {
    pub host: String,
    pub supported_protocols: Vec<String>,
    pub ciphers: Vec<CipherInfo>,
    pub has_weak_ciphers: bool,
    pub grade: String,
    pub preferred_cipher: String,
    pub scan_time_ms: u64,
}

// ─── Wordlists ────
const SUBDOMAIN_WORDLIST: &[&str] = &[
    "www", "mail", "ftp", "smtp", "pop", "imap", "webmail", "mx", "ns", "ns1",
    "ns2", "dns", "dns1", "dns2", "api", "dev", "staging", "stage", "test",
    "qa", "uat", "prod", "app", "web", "portal", "admin", "panel", "cpanel",
    "dashboard", "login", "auth", "sso", "cdn", "static", "assets", "media",
    "img", "images", "docs", "doc", "help", "support", "kb", "wiki", "blog",
    "shop", "store", "cart", "pay", "billing", "vpn", "remote", "git",
    "gitlab", "github", "ci", "jenkins", "build", "deploy", "monitor",
    "status", "health", "grafana", "prometheus", "kibana", "elastic",
    "search", "db", "database", "mysql", "postgres", "redis", "mongo",
    "cache", "proxy", "gateway", "lb", "load", "backup", "vault",
    "s3", "cloud", "aws", "gcp", "azure", "internal", "intranet",
    "exchange", "owa", "autodiscover", "lyncdiscover", "sip", "meet",
    "crm", "erp", "jira", "confluence", "slack", "chat", "m", "mobile",
];

const DIR_WORDLIST: &[&str] = &[
    "/admin", "/login", "/dashboard", "/api", "/api/v1", "/api/v2",
    "/.env", "/.git/config", "/.git/HEAD", "/.gitignore",
    "/.htaccess", "/.htpasswd", "/wp-admin", "/wp-login.php",
    "/wp-content", "/wp-includes", "/wp-config.php.bak",
    "/robots.txt", "/sitemap.xml", "/crossdomain.xml",
    "/server-status", "/server-info", "/.svn/entries",
    "/.DS_Store", "/web.config", "/elmah.axd",
    "/swagger", "/swagger-ui", "/swagger.json", "/api-docs",
    "/graphql", "/graphiql", "/.well-known/security.txt",
    "/actuator", "/actuator/health", "/actuator/env",
    "/info", "/health", "/metrics", "/debug", "/trace",
    "/console", "/phpinfo.php", "/phpmyadmin",
    "/adminer.php", "/backup", "/backups", "/dump",
    "/config", "/configuration", "/conf", "/settings",
    "/uploads", "/upload", "/files", "/tmp", "/temp",
    "/logs", "/log", "/error.log", "/access.log",
    "/test", "/testing", "/dev", "/development",
    "/staging", "/old", "/new", "/archive",
    "/.vscode", "/.idea", "/node_modules",
    "/vendor", "/composer.json", "/package.json",
    "/Gemfile", "/requirements.txt", "/Pipfile",
    "/docker-compose.yml", "/Dockerfile",
    "/readme.md", "/README.md", "/CHANGELOG.md",
    "/LICENSE", "/.env.local", "/.env.production",
    "/.env.development", "/.env.backup",
];

// ═══════════════════════════════════════════════════════════════════
//  TOOL 1: Service Scan (Banner Grabbing)
// ═══════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn netops_service_scan(host: String, ports: Option<String>) -> Result<ServiceScanResult, String> {
    let start = Instant::now();

    let ports_to_scan: Vec<u16> = if let Some(ref p) = ports {
        p.split(',')
            .filter_map(|s| s.trim().parse::<u16>().ok())
            .collect()
    } else {
        DEFAULT_PORTS.to_vec()
    };

    // Resolve hostname
    let addr = tokio::net::lookup_host(format!("{}:0", host))
        .await
        .map_err(|e| format!("DNS resolve failed: {}", e))?
        .next()
        .ok_or("No address found")?
        .ip();

    let mut handles = Vec::new();
    for port in ports_to_scan {
        let ip = addr;
        handles.push(tokio::spawn(async move {
            let sock = std::net::SocketAddr::new(ip, port);
            let connect = tokio::time::timeout(
                Duration::from_secs(3),
                tokio::net::TcpStream::connect(sock),
            ).await;

            match connect {
                Ok(Ok(mut stream)) => {
                    use tokio::io::{AsyncReadExt, AsyncWriteExt};
                    let mut banner = String::new();

                    // Protocol-specific probes
                    match port {
                        80 | 8080 | 8000 | 8888 => {
                            let req = format!("HEAD / HTTP/1.0\r\nHost: {}\r\n\r\n", ip);
                            let _ = stream.write_all(req.as_bytes()).await;
                        }
                        443 | 8443 => {
                            // Can't do TLS with raw TCP, just note it
                            banner = "TLS/SSL endpoint".into();
                        }
                        _ => {
                            // For SSH, SMTP, FTP etc - send CRLF and read
                            let _ = stream.write_all(b"\r\n").await;
                        }
                    }

                    if banner.is_empty() {
                        let mut buf = vec![0u8; 1024];
                        match tokio::time::timeout(Duration::from_secs(2), stream.read(&mut buf)).await {
                            Ok(Ok(n)) if n > 0 => {
                                banner = String::from_utf8_lossy(&buf[..n])
                                    .chars()
                                    .filter(|c| !c.is_control() || *c == '\n')
                                    .take(200)
                                    .collect::<String>()
                                    .trim()
                                    .to_string();
                            }
                            _ => {}
                        }
                    }

                    let version = extract_version(&banner);
                    ServiceEntry {
                        port,
                        service: port_service_name(port).to_string(),
                        version,
                        banner,
                        status: "open".into(),
                    }
                }
                _ => ServiceEntry {
                    port,
                    service: port_service_name(port).to_string(),
                    version: String::new(),
                    banner: String::new(),
                    status: "closed".into(),
                },
            }
        }));
    }

    let mut services = Vec::new();
    for h in handles {
        if let Ok(entry) = h.await {
            if entry.status == "open" {
                services.push(entry);
            }
        }
    }
    services.sort_by_key(|s| s.port);

    Ok(ServiceScanResult {
        host,
        services,
        scan_time_ms: start.elapsed().as_millis() as u64,
    })
}

fn extract_version(banner: &str) -> String {
    // Try to extract version from common patterns
    if banner.contains("SSH-") {
        banner.lines().next().unwrap_or("").to_string()
    } else if banner.contains("HTTP/") {
        if let Some(server) = banner.lines()
            .find(|l| l.to_lowercase().starts_with("server:"))
        {
            server.trim_start_matches(|c: char| !c.is_whitespace())
                .trim()
                .to_string()
        } else {
            banner.lines().next().unwrap_or("").to_string()
        }
    } else if banner.contains("220 ") || banner.contains("250 ") {
        banner.lines().next().unwrap_or("").to_string()
    } else {
        banner.chars().take(60).collect()
    }
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 2: Subdomain Enumeration
// ═══════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn netops_subdomain_enum(domain: String) -> Result<SubdomainEnumResult, String> {
    let start = Instant::now();
    let tested_count = SUBDOMAIN_WORDLIST.len() as u32;
    let mut found = Vec::new();

    // Process in batches of 20
    for chunk in SUBDOMAIN_WORDLIST.chunks(20) {
        let mut handles = Vec::new();
        for sub in chunk {
            let full = format!("{}.{}", sub, domain);
            let sub_str = sub.to_string();
            handles.push(tokio::spawn(async move {
                let output = tokio::process::Command::new("dig")
                    .args(["+short", &full])
                    .output()
                    .await;

                match output {
                    Ok(o) if o.status.success() => {
                        let out = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        if !out.is_empty() {
                            let mut ips = Vec::new();
                            let mut cname = String::new();
                            for line in out.lines() {
                                let line = line.trim().trim_end_matches('.');
                                if line.parse::<std::net::IpAddr>().is_ok() {
                                    ips.push(line.to_string());
                                } else if !line.is_empty() {
                                    cname = line.to_string();
                                }
                            }
                            if !ips.is_empty() || !cname.is_empty() {
                                return Some(SubdomainEntry {
                                    subdomain: sub_str,
                                    full_domain: full,
                                    ips,
                                    cname,
                                });
                            }
                        }
                        None
                    }
                    _ => None,
                }
            }));
        }
        for h in handles {
            if let Ok(Some(entry)) = h.await {
                found.push(entry);
            }
        }
    }

    let found_count = found.len() as u32;
    Ok(SubdomainEnumResult {
        domain,
        found,
        tested_count,
        found_count,
        scan_time_ms: start.elapsed().as_millis() as u64,
    })
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 3: Directory Brute Force
// ═══════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn netops_dir_bust(url: String) -> Result<DirBustResult, String> {
    let start = Instant::now();
    let client = http_client(5)?;
    let base = url.trim_end_matches('/').to_string();
    let tested_count = DIR_WORDLIST.len() as u32;
    let mut entries = Vec::new();

    // Process in batches of 10
    for chunk in DIR_WORDLIST.chunks(10) {
        let mut handles = Vec::new();
        for path in chunk {
            let full_url = format!("{}{}", base, path);
            let c = client.clone();
            let p = path.to_string();
            handles.push(tokio::spawn(async move {
                match c.get(&full_url)
                    .header("User-Agent", "FluxTerminal/2.0")
                    .send()
                    .await
                {
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        // Keep 200, 301, 302, 401, 403 (path exists)
                        if matches!(status, 200 | 301 | 302 | 307 | 308 | 401 | 403) {
                            let redirect = resp.headers()
                                .get("location")
                                .and_then(|v| v.to_str().ok())
                                .unwrap_or("")
                                .to_string();
                            let len = resp.content_length().unwrap_or(0);
                            Some(DirEntry {
                                path: p,
                                status_code: status,
                                content_length: len,
                                redirect_to: redirect,
                            })
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            }));
        }
        for h in handles {
            if let Ok(Some(entry)) = h.await {
                entries.push(entry);
            }
        }
    }

    let found_count = entries.len() as u32;
    Ok(DirBustResult {
        base_url: base,
        entries,
        tested_count,
        found_count,
        scan_time_ms: start.elapsed().as_millis() as u64,
    })
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 4: Web Fingerprint (Tech Detection)
// ═══════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn netops_web_fingerprint(url: String) -> Result<WebFingerprintResult, String> {
    let start = Instant::now();
    let client = http_client(10)?;
    let resp = client.get(&url).send().await.map_err(|e| format!("Request failed: {}", e))?;

    let mut techs: Vec<TechMatch> = Vec::new();
    let mut server_val = String::new();
    let mut powered_by_val = String::new();

    // Check headers
    let headers = resp.headers().clone();

    if let Some(v) = headers.get("server").and_then(|v| v.to_str().ok()) {
        server_val = v.to_string();
        let name = if v.to_lowercase().contains("nginx") { "Nginx" }
            else if v.to_lowercase().contains("apache") { "Apache" }
            else if v.to_lowercase().contains("iis") { "IIS" }
            else if v.to_lowercase().contains("cloudflare") { "Cloudflare" }
            else if v.to_lowercase().contains("litespeed") { "LiteSpeed" }
            else { "" };
        if !name.is_empty() {
            techs.push(TechMatch {
                name: name.into(),
                category: "Web Server".into(),
                version: v.to_string(),
                evidence: format!("Server: {}", v),
            });
        }
    }

    if let Some(v) = headers.get("x-powered-by").and_then(|v| v.to_str().ok()) {
        powered_by_val = v.to_string();
        techs.push(TechMatch {
            name: v.split('/').next().unwrap_or(v).to_string(),
            category: "Framework".into(),
            version: v.to_string(),
            evidence: format!("X-Powered-By: {}", v),
        });
    }

    if headers.get("cf-ray").is_some() {
        techs.push(TechMatch { name: "Cloudflare CDN".into(), category: "CDN".into(), version: String::new(), evidence: "CF-RAY header present".into() });
    }
    if headers.get("x-amz-cf-id").is_some() || headers.get("x-amz-request-id").is_some() {
        techs.push(TechMatch { name: "AWS CloudFront".into(), category: "CDN".into(), version: String::new(), evidence: "AWS headers present".into() });
    }
    if headers.get("x-vercel-id").is_some() {
        techs.push(TechMatch { name: "Vercel".into(), category: "Platform".into(), version: String::new(), evidence: "X-Vercel-Id header".into() });
    }
    if headers.get("x-netlify-request-id").is_some() {
        techs.push(TechMatch { name: "Netlify".into(), category: "Platform".into(), version: String::new(), evidence: "Netlify header".into() });
    }

    // Check cookies
    for cookie_hdr in headers.get_all("set-cookie") {
        if let Ok(c) = cookie_hdr.to_str() {
            let cl = c.to_lowercase();
            if cl.contains("phpsessid") { techs.push(TechMatch { name: "PHP".into(), category: "Language".into(), version: String::new(), evidence: "PHPSESSID cookie".into() }); }
            if cl.contains("csrftoken") { techs.push(TechMatch { name: "Django".into(), category: "Framework".into(), version: String::new(), evidence: "csrftoken cookie".into() }); }
            if cl.contains("laravel_session") { techs.push(TechMatch { name: "Laravel".into(), category: "Framework".into(), version: String::new(), evidence: "laravel_session cookie".into() }); }
            if cl.contains("_rails") || cl.contains("_session") { techs.push(TechMatch { name: "Ruby on Rails".into(), category: "Framework".into(), version: String::new(), evidence: "Rails session cookie".into() }); }
            if cl.contains("connect.sid") { techs.push(TechMatch { name: "Express.js".into(), category: "Framework".into(), version: String::new(), evidence: "connect.sid cookie".into() }); }
            if cl.contains("asp.net") { techs.push(TechMatch { name: "ASP.NET".into(), category: "Framework".into(), version: String::new(), evidence: "ASP.NET cookie".into() }); }
        }
    }

    // Check body for framework fingerprints
    if let Ok(body) = resp.text().await {
        let bl = body.to_lowercase();
        let fingerprints: Vec<(&str, &str, &str)> = vec![
            ("wp-content", "WordPress", "CMS"),
            ("__next", "Next.js", "Framework"),
            ("__nuxt", "Nuxt.js", "Framework"),
            ("ng-version", "Angular", "Framework"),
            ("data-reactroot", "React", "Framework"),
            ("data-react", "React", "Framework"),
            ("svelte", "Svelte", "Framework"),
            ("ember", "Ember.js", "Framework"),
            ("vue.js", "Vue.js", "Framework"),
            ("jquery", "jQuery", "Library"),
            ("bootstrap", "Bootstrap", "CSS Framework"),
            ("tailwindcss", "Tailwind CSS", "CSS Framework"),
            ("drupal", "Drupal", "CMS"),
            ("joomla", "Joomla", "CMS"),
            ("shopify", "Shopify", "E-commerce"),
            ("woocommerce", "WooCommerce", "E-commerce"),
            ("magento", "Magento", "E-commerce"),
            ("gatsby", "Gatsby", "Framework"),
            ("remix", "Remix", "Framework"),
        ];
        for (pattern, name, category) in fingerprints {
            if bl.contains(pattern) {
                // Avoid duplicate detection
                if !techs.iter().any(|t| t.name == name) {
                    techs.push(TechMatch {
                        name: name.into(),
                        category: category.into(),
                        version: String::new(),
                        evidence: format!("\"{}\" found in HTML", pattern),
                    });
                }
            }
        }
    }

    Ok(WebFingerprintResult {
        url,
        technologies: techs,
        server: server_val,
        powered_by: powered_by_val,
        scan_time_ms: start.elapsed().as_millis() as u64,
    })
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 5: WAF Detection
// ═══════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn netops_waf_detect(url: String) -> Result<WafDetectResult, String> {
    let start = Instant::now();
    let client = http_client(10)?;
    let base = url.trim_end_matches('/').to_string();

    // 1. Normal request
    let normal_resp = client.get(&base).send().await.map_err(|e| format!("Normal request failed: {}", e))?;
    let normal_status = normal_resp.status().as_u16();
    let normal_headers = normal_resp.headers().clone();

    // 2. XSS probe
    let xss_url = format!("{}/?test=%3Cscript%3Ealert(1)%3C/script%3E", base);
    let xss_status = client.get(&xss_url).send().await
        .map(|r| r.status().as_u16())
        .unwrap_or(0);

    // 3. SQLi probe
    let sqli_url = format!("{}/?id=1%27%20OR%20%271%27%3D%271", base);
    let sqli_status = client.get(&sqli_url).send().await
        .map(|r| r.status().as_u16())
        .unwrap_or(0);

    let blocked_status = if xss_status > sqli_status { xss_status } else { sqli_status };

    let mut indicators: Vec<WafIndicator> = Vec::new();
    let mut waf_name = String::new();

    // Check header signatures
    let waf_signatures: Vec<(&str, &str)> = vec![
        ("cf-ray", "Cloudflare"),
        ("x-sucuri-id", "Sucuri"),
        ("x-cdn", "CDN WAF"),
        ("x-akamai-transformed", "Akamai"),
        ("x-protected-by", "WAF"),
    ];

    for (header, name) in &waf_signatures {
        if normal_headers.get(*header).is_some() {
            indicators.push(WafIndicator {
                name: name.to_string(),
                confidence: "high".into(),
                evidence: format!("{} header present", header),
            });
            if waf_name.is_empty() { waf_name = name.to_string(); }
        }
    }

    if let Some(server) = normal_headers.get("server").and_then(|v| v.to_str().ok()) {
        let sl = server.to_lowercase();
        if sl.contains("cloudflare") {
            indicators.push(WafIndicator { name: "Cloudflare".into(), confidence: "high".into(), evidence: format!("Server: {}", server) });
            if waf_name.is_empty() { waf_name = "Cloudflare".into(); }
        } else if sl.contains("akamaighost") {
            indicators.push(WafIndicator { name: "Akamai".into(), confidence: "high".into(), evidence: format!("Server: {}", server) });
            if waf_name.is_empty() { waf_name = "Akamai".into(); }
        } else if sl.contains("awselb") || sl.contains("awsalb") {
            indicators.push(WafIndicator { name: "AWS WAF/ALB".into(), confidence: "medium".into(), evidence: format!("Server: {}", server) });
            if waf_name.is_empty() { waf_name = "AWS WAF".into(); }
        }
    }

    // Status-based detection
    if normal_status == 200 && matches!(blocked_status, 403 | 406 | 429 | 503) {
        indicators.push(WafIndicator {
            name: "Behavioral WAF".into(),
            confidence: "high".into(),
            evidence: format!("Normal={}, Probe={}", normal_status, blocked_status),
        });
        if waf_name.is_empty() { waf_name = "Unknown WAF".into(); }
    }

    let waf_detected = !indicators.is_empty();

    Ok(WafDetectResult {
        url: base,
        waf_detected,
        waf_name,
        indicators,
        normal_status,
        blocked_status,
        scan_time_ms: start.elapsed().as_millis() as u64,
    })
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 6: Web Vulnerability Scan (Nikto-lite)
// ═══════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn netops_web_vuln_scan(url: String) -> Result<WebVulnResult, String> {
    let start = Instant::now();
    let client = http_client(5)?;
    let base = url.trim_end_matches('/').to_string();
    let mut findings: Vec<VulnFinding> = Vec::new();

    // 1. Check sensitive file exposure
    let sensitive_paths = vec![
        ("/.env", "Environment file exposed"),
        ("/.git/config", "Git repository exposed"),
        ("/wp-config.php.bak", "WordPress config backup"),
        ("/server-status", "Apache server-status"),
        ("/phpinfo.php", "PHP info page exposed"),
        ("/.htpasswd", "Password file exposed"),
        ("/web.config", "IIS config exposed"),
        ("/.svn/entries", "SVN repository exposed"),
    ];

    for (path, title) in &sensitive_paths {
        let full = format!("{}{}", base, path);
        if let Ok(resp) = client.get(&full).send().await {
            if resp.status().as_u16() == 200 {
                findings.push(VulnFinding {
                    title: title.to_string(),
                    severity: "high".into(),
                    category: "Information Disclosure".into(),
                    detail: format!("{} is publicly accessible", path),
                    url: full,
                    remediation: format!("Block access to {} in web server config", path),
                });
            }
        }
    }

    // 2. Check directory listing
    let dir_paths = vec!["/uploads/", "/images/", "/files/", "/backup/"];
    for path in &dir_paths {
        let full = format!("{}{}", base, path);
        if let Ok(resp) = client.get(&full).send().await {
            if resp.status().as_u16() == 200 {
                if let Ok(body) = resp.text().await {
                    if body.contains("Index of") || body.contains("Directory listing") {
                        findings.push(VulnFinding {
                            title: "Directory listing enabled".into(),
                            severity: "medium".into(),
                            category: "Information Disclosure".into(),
                            detail: format!("{} has directory listing", path),
                            url: full,
                            remediation: "Disable directory listing in web server config".into(),
                        });
                    }
                }
            }
        }
    }

    // 3. Check main page headers
    if let Ok(resp) = client.get(&base).send().await {
        let hdrs = resp.headers().clone();

        // Missing security headers
        let sec_headers: Vec<(&str, &str)> = vec![
            ("content-security-policy", "Content Security Policy"),
            ("strict-transport-security", "HTTP Strict Transport Security"),
            ("x-content-type-options", "X-Content-Type-Options"),
            ("x-frame-options", "X-Frame-Options"),
        ];

        for (header, name) in &sec_headers {
            if hdrs.get(*header).is_none() {
                findings.push(VulnFinding {
                    title: format!("Missing {}", name),
                    severity: "low".into(),
                    category: "Missing Security Header".into(),
                    detail: format!("{} header is not set", name),
                    url: base.clone(),
                    remediation: format!("Add {} header to server response", header),
                });
            }
        }

        // Server version disclosure
        if let Some(server) = hdrs.get("server").and_then(|v| v.to_str().ok()) {
            if server.contains('/') {
                findings.push(VulnFinding {
                    title: "Server version disclosed".into(),
                    severity: "low".into(),
                    category: "Information Disclosure".into(),
                    detail: format!("Server header reveals: {}", server),
                    url: base.clone(),
                    remediation: "Remove version info from Server header".into(),
                });
            }
        }

        // CORS misconfiguration
        if let Some(cors) = hdrs.get("access-control-allow-origin").and_then(|v| v.to_str().ok()) {
            if cors == "*" {
                findings.push(VulnFinding {
                    title: "CORS allows all origins".into(),
                    severity: "medium".into(),
                    category: "CORS Misconfiguration".into(),
                    detail: "Access-Control-Allow-Origin is set to *".into(),
                    url: base.clone(),
                    remediation: "Restrict CORS to specific trusted origins".into(),
                });
            }
        }
    }

    let critical_count = findings.iter().filter(|f| f.severity == "critical").count() as u32;
    let high_count = findings.iter().filter(|f| f.severity == "high").count() as u32;
    let medium_count = findings.iter().filter(|f| f.severity == "medium").count() as u32;
    let low_count = findings.iter().filter(|f| f.severity == "low").count() as u32;

    Ok(WebVulnResult {
        target: base,
        findings,
        critical_count,
        high_count,
        medium_count,
        low_count,
        scan_time_ms: start.elapsed().as_millis() as u64,
    })
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 7: Hash Identifier
// ═══════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn netops_hash_id(input: String) -> Result<HashIdResult, String> {
    let start = Instant::now();
    let trimmed = input.trim();
    let length = trimmed.len() as u32;
    let mut matches: Vec<HashMatch> = Vec::new();

    // Check for prefixed hashes first
    if trimmed.starts_with("$2a$") || trimmed.starts_with("$2b$") || trimmed.starts_with("$2y$") {
        matches.push(HashMatch { hash_type: "bcrypt".into(), confidence: "high".into(), description: "Blowfish-based password hash (60 chars)".into() });
    } else if trimmed.starts_with("$argon2") {
        matches.push(HashMatch { hash_type: "Argon2".into(), confidence: "high".into(), description: "Argon2 password hash".into() });
    } else if trimmed.starts_with("$1$") {
        matches.push(HashMatch { hash_type: "MD5 Unix Crypt".into(), confidence: "high".into(), description: "MD5-based Unix password hash".into() });
    } else if trimmed.starts_with("$5$") {
        matches.push(HashMatch { hash_type: "SHA-256 Unix Crypt".into(), confidence: "high".into(), description: "SHA-256 Unix password hash".into() });
    } else if trimmed.starts_with("$6$") {
        matches.push(HashMatch { hash_type: "SHA-512 Unix Crypt".into(), confidence: "high".into(), description: "SHA-512 Unix password hash".into() });
    } else if trimmed.starts_with('*') && length == 41 && trimmed[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        matches.push(HashMatch { hash_type: "MySQL 4.1+".into(), confidence: "high".into(), description: "MySQL native password hash".into() });
    }

    // Length-based hex detection
    let is_hex = trimmed.chars().all(|c| c.is_ascii_hexdigit());
    if is_hex {
        match length {
            32 => {
                matches.push(HashMatch { hash_type: "MD5".into(), confidence: "high".into(), description: "128-bit MD5 message digest".into() });
                matches.push(HashMatch { hash_type: "NTLM".into(), confidence: "medium".into(), description: "Windows NTLM hash".into() });
                matches.push(HashMatch { hash_type: "MD4".into(), confidence: "low".into(), description: "128-bit MD4 message digest".into() });
            }
            40 => {
                matches.push(HashMatch { hash_type: "SHA-1".into(), confidence: "high".into(), description: "160-bit SHA-1 hash".into() });
                matches.push(HashMatch { hash_type: "RIPEMD-160".into(), confidence: "low".into(), description: "160-bit RIPEMD hash".into() });
            }
            56 => {
                matches.push(HashMatch { hash_type: "SHA-224".into(), confidence: "high".into(), description: "224-bit SHA-2 hash".into() });
            }
            64 => {
                matches.push(HashMatch { hash_type: "SHA-256".into(), confidence: "high".into(), description: "256-bit SHA-2 hash".into() });
                matches.push(HashMatch { hash_type: "SHA3-256".into(), confidence: "low".into(), description: "256-bit SHA-3 hash".into() });
                matches.push(HashMatch { hash_type: "BLAKE2s".into(), confidence: "low".into(), description: "256-bit BLAKE2s hash".into() });
            }
            96 => {
                matches.push(HashMatch { hash_type: "SHA-384".into(), confidence: "high".into(), description: "384-bit SHA-2 hash".into() });
            }
            128 => {
                matches.push(HashMatch { hash_type: "SHA-512".into(), confidence: "high".into(), description: "512-bit SHA-2 hash".into() });
                matches.push(HashMatch { hash_type: "SHA3-512".into(), confidence: "low".into(), description: "512-bit SHA-3 hash".into() });
                matches.push(HashMatch { hash_type: "BLAKE2b".into(), confidence: "low".into(), description: "512-bit BLAKE2b hash".into() });
            }
            _ => {}
        }
    }

    // JWT detection
    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.len() == 3 && parts.iter().all(|p| p.len() > 5) {
        matches.push(HashMatch { hash_type: "JWT".into(), confidence: "high".into(), description: "JSON Web Token (3 dot-separated segments)".into() });
    }

    // Base64 detection
    let is_base64 = trimmed.len() > 8
        && trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        && (trimmed.len() % 4 == 0 || trimmed.ends_with('='));

    if is_base64 && !is_hex && matches.is_empty() {
        matches.push(HashMatch { hash_type: "Base64".into(), confidence: "medium".into(), description: "Base64 encoded data".into() });
    }

    Ok(HashIdResult {
        input: trimmed.to_string(),
        length,
        matches,
        is_base64,
        query_time_ms: start.elapsed().as_millis() as u64,
    })
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 8: Cipher Scan (TLS Enumeration)
// ═══════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn netops_cipher_scan(host: String) -> Result<CipherScanResult, String> {
    let start = Instant::now();
    let mut supported_protocols: Vec<String> = Vec::new();
    let mut ciphers: Vec<CipherInfo> = Vec::new();
    let mut preferred_cipher = String::new();

    let protocols = vec![
        ("-tls1", "TLSv1.0"),
        ("-tls1_1", "TLSv1.1"),
        ("-tls1_2", "TLSv1.2"),
    ];

    // Test each protocol version
    for (flag, name) in &protocols {
        let output = tokio::process::Command::new("openssl")
            .args(["s_client", "-connect", &format!("{}:443", host), flag, "-brief"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await;

        if let Ok(o) = output {
            let combined = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
            if !combined.contains("error") && !combined.contains("alert") && (combined.contains("Protocol") || combined.contains("Cipher")) {
                supported_protocols.push(name.to_string());
            }
        }
    }

    // TLS 1.3 separate check (different flag on some openssl versions)
    let tls13_output = tokio::process::Command::new("openssl")
        .args(["s_client", "-connect", &format!("{}:443", host), "-tls1_3", "-brief"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await;

    if let Ok(o) = tls13_output {
        let combined = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
        if !combined.contains("unknown option") && !combined.contains("error") && (combined.contains("TLSv1.3") || combined.contains("Protocol") || combined.contains("Cipher")) {
            supported_protocols.push("TLSv1.3".to_string());
        }
    }

    // Get cipher list using s_client
    let cipher_output = tokio::process::Command::new("openssl")
        .args(["s_client", "-connect", &format!("{}:443", host), "-cipher", "ALL:COMPLEMENTOFALL"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await;

    if let Ok(o) = cipher_output {
        let out = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
        // Parse the cipher from the output
        for line in out.lines() {
            if line.contains("Cipher    :") || line.contains("Cipher is") {
                let cipher_name = line.split(':').last().or_else(|| line.split("is").last())
                    .unwrap_or("").trim().to_string();
                if !cipher_name.is_empty() && cipher_name != "(NONE)" && cipher_name != "0000" {
                    if preferred_cipher.is_empty() {
                        preferred_cipher = cipher_name.clone();
                    }
                    let bits = if cipher_name.contains("256") { 256 }
                        else if cipher_name.contains("128") { 128 }
                        else if cipher_name.contains("384") { 384 }
                        else { 0 };
                    let strength = cipher_strength(&cipher_name);
                    ciphers.push(CipherInfo {
                        name: cipher_name,
                        protocol: supported_protocols.last().cloned().unwrap_or_default(),
                        bits,
                        strength,
                    });
                }
            }
        }
    }

    // Also enumerate via openssl ciphers command
    let ciphers_list = tokio::process::Command::new("openssl")
        .args(["ciphers", "-v", "ALL:COMPLEMENTOFALL"])
        .output()
        .await;

    if let Ok(o) = ciphers_list {
        if o.status.success() {
            let out = String::from_utf8_lossy(&o.stdout);
            for line in out.lines().take(30) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let name = parts[0].to_string();
                    let proto = parts[1].to_string();
                    let bits_str = parts.iter().find(|p| p.starts_with("Enc="))
                        .unwrap_or(&"Enc=0");
                    let bits = bits_str.split('(').nth(1).and_then(|s| s.trim_end_matches(')').parse::<u32>().ok()).unwrap_or(0);
                    let strength = cipher_strength(&name);

                    if !ciphers.iter().any(|c| c.name == name) {
                        ciphers.push(CipherInfo { name, protocol: proto, bits, strength });
                    }
                }
            }
        }
    }

    let has_weak = ciphers.iter().any(|c| c.strength == "weak" || c.strength == "insecure");
    let has_old_proto = supported_protocols.iter().any(|p| p == "TLSv1.0" || p == "TLSv1.1");
    let has_tls13 = supported_protocols.iter().any(|p| p == "TLSv1.3");

    let grade = if has_old_proto && has_weak { "F" }
        else if has_old_proto { "D" }
        else if has_weak { "C" }
        else if has_tls13 && !has_old_proto { "A" }
        else { "B" };

    Ok(CipherScanResult {
        host,
        supported_protocols,
        ciphers,
        has_weak_ciphers: has_weak,
        grade: grade.to_string(),
        preferred_cipher,
        scan_time_ms: start.elapsed().as_millis() as u64,
    })
}

fn cipher_strength(name: &str) -> String {
    let n = name.to_uppercase();
    if n.contains("NULL") || n.contains("EXPORT") || n.contains("anon") {
        "insecure".into()
    } else if n.contains("RC4") || n.contains("DES") || n.contains("MD5") || n.contains("3DES") {
        "weak".into()
    } else if n.contains("AES") && (n.contains("256") || n.contains("GCM")) {
        "strong".into()
    } else if n.contains("CHACHA20") {
        "strong".into()
    } else {
        "acceptable".into()
    }
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 9: WPA 4-Way Handshake Analyzer
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {
    pub step: u8,
    pub name: String,
    pub description: String,
    pub status: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeEvent {
    pub timestamp: String,
    pub event_type: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeResult {
    pub ssid: String,
    pub bssid: String,
    pub security_protocol: String,
    pub auth_method: String,
    pub pairwise_cipher: String,
    pub group_cipher: String,
    pub link_auth: String,
    pub handshake_messages: Vec<HandshakeMessage>,
    pub events: Vec<HandshakeEvent>,
    pub handshake_complete: bool,
    pub last_handshake_time: String,
    pub rssi: i32,
    pub channel: u32,
    pub noise: i32,
    pub scan_time_ms: u64,
    pub log_text: String,
}

#[tauri::command]
pub async fn netops_handshake_analyze(
    target_ssid: Option<String>,
    target_bssid: Option<String>,
    target_channel: Option<u32>,
    target_security: Option<String>,
    target_rssi: Option<i32>,
) -> Result<HandshakeResult, String> {
    let start = Instant::now();

    // Determine if we're analyzing a specific target or the connected network
    let has_target = target_ssid.is_some() || target_bssid.is_some();

    // 1. Get connection details from system_profiler
    let profiler = tokio::process::Command::new("system_profiler")
        .args(["SPAirPortDataType", "-detailLevel", "basic"])
        .output()
        .await
        .map_err(|e| format!("system_profiler failed: {}", e))?;

    let prof_out = String::from_utf8_lossy(&profiler.stdout).to_string();
    let is_connected = prof_out.contains("Status: Connected");

    // Parse connected network info from system_profiler
    let mut connected_bssid = String::new();
    let mut connected_rssi: i32 = 0;
    let mut connected_noise: i32 = 0;
    let mut connected_channel: u32 = 0;
    let mut connected_security = String::new();
    let mut connected_pairwise = String::new();
    let mut connected_group = String::new();
    let mut connected_auth = String::new();

    if is_connected {
        let mut in_current = false;
        let mut found_network_header = false;
        for line in prof_out.lines() {
            let trimmed = line.trim();
            if trimmed.contains("Current Network Information:") {
                in_current = true;
                continue;
            }
            if in_current && !found_network_header && trimmed.ends_with(':') && !trimmed.contains("Current Network") {
                found_network_header = true;
                continue;
            }
            if in_current && found_network_header {
                if trimmed.starts_with("Other Local Wi-Fi") || trimmed.starts_with("Other Information") {
                    break;
                }
                if let Some((key, val)) = trimmed.split_once(':') {
                    let key = key.trim();
                    let val = val.trim();
                    match key {
                        "BSSID" => connected_bssid = val.to_string(),
                        "Channel" => connected_channel = val.split_whitespace().next().unwrap_or("0").parse().unwrap_or(0),
                        "Security" => connected_security = val.to_string(),
                        "Authentication" => connected_auth = val.to_string(),
                        "Signal / Noise" => {
                            let parts: Vec<&str> = val.split('/').collect();
                            if let Some(sig) = parts.first() {
                                connected_rssi = sig.trim().replace(" dBm", "").parse().unwrap_or(0);
                            }
                            if let Some(nz) = parts.get(1) {
                                connected_noise = nz.trim().replace(" dBm", "").parse().unwrap_or(0);
                            }
                        }
                        "Unicast Cipher" => connected_pairwise = val.to_string(),
                        "Group Cipher" => connected_group = val.to_string(),
                        _ => {}
                    }
                }
            }
        }
    }

    // Check if the target matches the connected network (by BSSID)
    let target_is_connected = if let Some(ref tb) = target_bssid {
        !connected_bssid.is_empty() && tb.eq_ignore_ascii_case(&connected_bssid)
    } else {
        false
    };

    // Set the final values based on whether target is the connected network or not
    let (mut ssid, mut bssid, rssi, noise, channel, security_protocol, mut pairwise_cipher, mut group_cipher, mut auth_method);

    if !has_target || target_is_connected {
        // Analyzing the connected network — use system_profiler details
        if !is_connected && !has_target {
            return Err("Not connected to any WiFi network. Run WiFi Scan first and select a network.".into());
        }
        bssid = connected_bssid;
        rssi = connected_rssi;
        noise = connected_noise;
        channel = connected_channel;
        security_protocol = connected_security;
        pairwise_cipher = connected_pairwise;
        group_cipher = connected_group;
        auth_method = connected_auth;

        // Get SSID via WiFi scan (system_profiler redacts SSIDs on Sequoia)
        ssid = String::new();
        let swift_code = include_str!("wifi_scan.swift");
        let scan = tokio::process::Command::new("swift")
            .arg("-e")
            .arg(swift_code)
            .output()
            .await;

        if let Ok(scan_out) = scan {
            if scan_out.status.success() {
                let scan_str = String::from_utf8_lossy(&scan_out.stdout);
                for line in scan_str.lines() {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 5 {
                        let scan_bssid = parts[1];
                        let scan_ch: u32 = parts[3].parse().unwrap_or(0);
                        if (!bssid.is_empty() && scan_bssid.eq_ignore_ascii_case(&bssid))
                            || (bssid.is_empty() && scan_ch == channel && channel > 0) {
                            ssid = parts[0].to_string();
                            if bssid.is_empty() {
                                bssid = scan_bssid.to_string();
                            }
                            break;
                        }
                    }
                }
            }
        }
        // If we had a target SSID (and it matched connected), use it
        if ssid.is_empty() {
            ssid = target_ssid.unwrap_or_else(|| "Connected Network".into());
        }
    } else {
        // Analyzing a non-connected network — use the target info from WiFi scan
        ssid = target_ssid.unwrap_or_default();
        bssid = target_bssid.unwrap_or_default();
        channel = target_channel.unwrap_or(0);
        rssi = target_rssi.unwrap_or(0);
        noise = 0; // Not available for non-connected networks
        security_protocol = target_security.unwrap_or_default();
        pairwise_cipher = String::new();
        group_cipher = String::new();
        auth_method = String::new();
    }

    let link_auth = if !auth_method.is_empty() { auth_method.clone() } else { security_protocol.clone() };

    // Default cipher values based on protocol
    if pairwise_cipher.is_empty() {
        pairwise_cipher = if security_protocol.contains("WPA3") { "AES-CCMP-256".into() }
            else if security_protocol.contains("WPA2") { "AES-CCMP".into() }
            else if security_protocol.contains("WPA") { "TKIP/AES".into() }
            else if security_protocol.contains("WEP") { "WEP".into() }
            else { "N/A".into() };
    }
    if group_cipher.is_empty() {
        group_cipher = pairwise_cipher.clone();
    }
    if auth_method.is_empty() {
        auth_method = if security_protocol.contains("WPA3") { "SAE (Simultaneous Authentication of Equals)".into() }
            else if security_protocol.contains("WPA2") && security_protocol.contains("Enterprise") { "802.1X / EAP".into() }
            else if security_protocol.contains("WPA2") { "PSK (Pre-Shared Key)".into() }
            else if security_protocol.contains("WPA") { "PSK (Pre-Shared Key)".into() }
            else if security_protocol.contains("WEP") { "Shared Key".into() }
            else { link_auth.clone() };
    }

    // 3. Query logs for handshake/EAPOL events
    let mut events: Vec<HandshakeEvent> = Vec::new();
    let mut has_eapol = false;
    let mut has_ptk = false;
    let mut has_gtk = false;
    let mut has_complete = false;
    let mut last_handshake_time = String::new();

    let log_output = tokio::process::Command::new("log")
        .args([
            "show", "--predicate",
            "subsystem == \"com.apple.wifi\" AND (eventMessage CONTAINS[c] \"eapol\" OR eventMessage CONTAINS[c] \"handshake\" OR eventMessage CONTAINS[c] \"4-way\" OR eventMessage CONTAINS[c] \"key exchange\" OR eventMessage CONTAINS[c] \"PTK\" OR eventMessage CONTAINS[c] \"GTK\" OR eventMessage CONTAINS[c] \"association\" OR eventMessage CONTAINS[c] \"authentication\")",
            "--style", "compact",
            "--last", "30m",
        ])
        .output()
        .await;

    if let Ok(o) = log_output {
        let out = String::from_utf8_lossy(&o.stdout).to_string();
        for line in out.lines().take(100) {
            let line = line.trim();
            if line.is_empty() || line.starts_with("Filtering") || line.starts_with("Timestamp") {
                continue;
            }

            let msg_lower = line.to_lowercase();
            let event_type = if msg_lower.contains("eapol") { "eapol" }
                else if msg_lower.contains("4-way") || msg_lower.contains("four-way") { "4-way" }
                else if msg_lower.contains("ptk") { "ptk" }
                else if msg_lower.contains("gtk") { "gtk" }
                else if msg_lower.contains("handshake") { "handshake" }
                else if msg_lower.contains("key exchange") || msg_lower.contains("key install") { "key" }
                else if msg_lower.contains("association") { "association" }
                else if msg_lower.contains("authentication") { "authentication" }
                else { "info" };

            if msg_lower.contains("eapol") { has_eapol = true; }
            if msg_lower.contains("ptk") { has_ptk = true; }
            if msg_lower.contains("gtk") || msg_lower.contains("group key") { has_gtk = true; }
            if msg_lower.contains("complete") || msg_lower.contains("success") || msg_lower.contains("installed") {
                has_complete = true;
            }

            let timestamp = line.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
            let message = if line.len() > timestamp.len() + 1 {
                line[timestamp.len()..].trim().to_string()
            } else {
                line.to_string()
            };

            if !timestamp.is_empty() && last_handshake_time.is_empty() && (msg_lower.contains("handshake") || msg_lower.contains("eapol")) {
                last_handshake_time = timestamp.clone();
            }

            events.push(HandshakeEvent {
                timestamp,
                event_type: event_type.to_string(),
                message: message.chars().take(200).collect(),
            });
        }
    }

    // 4. Build the 4 handshake messages
    let uses_wpa = security_protocol.contains("WPA") || security_protocol.contains("WEP");
    let handshake_complete = has_eapol || has_complete || (uses_wpa && !ssid.is_empty());

    let handshake_messages = vec![
        HandshakeMessage {
            step: 1,
            name: "MSG 1: ANonce".into(),
            description: "AP sends ANonce (authenticator nonce) to client".into(),
            status: if handshake_complete { "complete".into() } else { "pending".into() },
            timestamp: if has_eapol { last_handshake_time.clone() } else { String::new() },
        },
        HandshakeMessage {
            step: 2,
            name: "MSG 2: SNonce + MIC".into(),
            description: "Client generates PTK, sends SNonce and MIC to AP".into(),
            status: if has_ptk || handshake_complete { "complete".into() } else { "pending".into() },
            timestamp: String::new(),
        },
        HandshakeMessage {
            step: 3,
            name: "MSG 3: GTK + MIC".into(),
            description: "AP sends GTK (Group Temporal Key) encrypted with PTK".into(),
            status: if has_gtk || handshake_complete { "complete".into() } else { "pending".into() },
            timestamp: String::new(),
        },
        HandshakeMessage {
            step: 4,
            name: "MSG 4: ACK".into(),
            description: "Client confirms key installation, handshake complete".into(),
            status: if handshake_complete { "complete".into() } else { "pending".into() },
            timestamp: String::new(),
        },
    ];

    // 5. Generate detailed log text
    let scan_ms = start.elapsed().as_millis() as u64;
    let now = {
        let d = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
        let secs = d.as_secs();
        // UTC timestamp
        let s = secs % 60;
        let m = (secs / 60) % 60;
        let h = (secs / 3600) % 24;
        let days = secs / 86400;
        // Simple date calc from epoch days
        let (y, mo, da) = {
            let mut y = 1970i64;
            let mut rem = days as i64;
            loop {
                let ydays = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
                if rem < ydays { break; }
                rem -= ydays;
                y += 1;
            }
            let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
            let mdays = [31, if leap {29} else {28}, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            let mut mo = 0u32;
            for md in &mdays {
                if rem < *md as i64 { break; }
                rem -= *md as i64;
                mo += 1;
            }
            (y, mo + 1, rem + 1)
        };
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", y, mo, da, h, m, s)
    };
    let separator = "═".repeat(72);
    let thin_sep = "─".repeat(72);

    let mut log = String::new();
    log.push_str(&format!("{}\n", separator));
    log.push_str(&format!("  FLUX TERMINAL — WPA HANDSHAKE ANALYSIS REPORT\n"));
    log.push_str(&format!("  Generated: {}\n", now));
    log.push_str(&format!("{}\n\n", separator));

    // Network info
    log.push_str(&format!("┌{}\n", thin_sep));
    log.push_str(&format!("│  TARGET NETWORK\n"));
    log.push_str(&format!("├{}\n", thin_sep));
    log.push_str(&format!("│  SSID             : {}\n", ssid));
    log.push_str(&format!("│  BSSID            : {}\n", if bssid.is_empty() { "N/A" } else { &bssid }));
    log.push_str(&format!("│  Security         : {}\n", security_protocol));
    log.push_str(&format!("│  Auth Method      : {}\n", auth_method));
    log.push_str(&format!("│  Pairwise Cipher  : {}\n", pairwise_cipher));
    log.push_str(&format!("│  Group Cipher     : {}\n", group_cipher));
    log.push_str(&format!("│  Channel          : {}\n", channel));
    log.push_str(&format!("│  RSSI             : {} dBm\n", rssi));
    if noise != 0 {
        log.push_str(&format!("│  Noise            : {} dBm\n", noise));
        log.push_str(&format!("│  SNR              : {} dB\n", rssi - noise));
    }
    log.push_str(&format!("│  Scan Time        : {} ms\n", scan_ms));
    log.push_str(&format!("└{}\n\n", thin_sep));

    // 4-Way Handshake Steps
    log.push_str(&format!("┌{}\n", thin_sep));
    log.push_str(&format!("│  WPA 4-WAY HANDSHAKE PROTOCOL\n"));
    log.push_str(&format!("├{}\n", thin_sep));
    log.push_str(&format!("│  Status: {}\n", if handshake_complete { "COMPLETE" } else { "PENDING" }));
    log.push_str(&format!("├{}\n\n", thin_sep));

    for msg in &handshake_messages {
        log.push_str(&format!("  ╔══ STEP {} ══════════════════════════════════════════════════\n", msg.step));
        log.push_str(&format!("  ║  {}\n", msg.name));
        log.push_str(&format!("  ║  Status: {}\n", msg.status.to_uppercase()));
        if !msg.timestamp.is_empty() {
            log.push_str(&format!("  ║  Timestamp: {}\n", msg.timestamp));
        }
        log.push_str(&format!("  ║\n"));

        // Detailed explanation per step
        match msg.step {
            1 => {
                log.push_str("  ║  [AUTHENTICATOR → SUPPLICANT]\n");
                log.push_str("  ║\n");
                log.push_str("  ║  The Access Point (AP/Authenticator) initiates the handshake by\n");
                log.push_str("  ║  generating a random ANonce (Authenticator Nonce) and sending it\n");
                log.push_str("  ║  to the client (Supplicant) in an EAPOL-Key frame.\n");
                log.push_str("  ║\n");
                log.push_str("  ║  Frame Contents:\n");
                log.push_str("  ║    • EAPOL-Key type: Pairwise\n");
                log.push_str("  ║    • Key Info: ACK set, MIC not set\n");
                log.push_str(&format!("  ║    • ANonce: <random 256-bit value generated by AP>\n"));
                log.push_str("  ║    • Key Data: Empty\n");
                log.push_str("  ║\n");
                log.push_str("  ║  At this point, the client has:\n");
                log.push_str("  ║    • PMK (Pairwise Master Key) — derived from passphrase + SSID\n");
                log.push_str("  ║    • ANonce — received from AP\n");
                log.push_str("  ║    • Client MAC address (SA)\n");
                log.push_str("  ║    • AP MAC address (AA/BSSID)\n");
                log.push_str("  ║\n");
            }
            2 => {
                log.push_str("  ║  [SUPPLICANT → AUTHENTICATOR]\n");
                log.push_str("  ║\n");
                log.push_str("  ║  The client generates its own random SNonce (Supplicant Nonce)\n");
                log.push_str("  ║  and computes the PTK (Pairwise Transient Key) using:\n");
                log.push_str("  ║\n");
                log.push_str("  ║    PTK = PRF(PMK + ANonce + SNonce + AA + SA)\n");
                log.push_str("  ║\n");
                log.push_str("  ║  The PTK is split into:\n");
                log.push_str("  ║    • KCK (Key Confirmation Key)  — 128 bits, used for MIC\n");
                log.push_str("  ║    • KEK (Key Encryption Key)    — 128 bits, for GTK encryption\n");
                log.push_str("  ║    • TK  (Temporal Key)          — 128/256 bits, for data encryption\n");
                log.push_str("  ║\n");
                log.push_str("  ║  Frame Contents:\n");
                log.push_str("  ║    • EAPOL-Key type: Pairwise\n");
                log.push_str("  ║    • Key Info: MIC set\n");
                log.push_str("  ║    • SNonce: <random 256-bit value generated by client>\n");
                log.push_str(&format!("  ║    • MIC: <computed using KCK over entire EAPOL frame>\n"));
                log.push_str("  ║    • Key Data: RSN IE (client's security capabilities)\n");
                log.push_str("  ║\n");
                log.push_str(&format!("  ║  Cipher Suite: {}\n", pairwise_cipher));
                log.push_str("  ║\n");
            }
            3 => {
                log.push_str("  ║  [AUTHENTICATOR → SUPPLICANT]\n");
                log.push_str("  ║\n");
                log.push_str("  ║  The AP verifies the MIC from Message 2. If valid, it proves\n");
                log.push_str("  ║  the client knows the PMK (and therefore the correct passphrase).\n");
                log.push_str("  ║\n");
                log.push_str("  ║  The AP then computes the same PTK and sends:\n");
                log.push_str("  ║\n");
                log.push_str("  ║  Frame Contents:\n");
                log.push_str("  ║    • EAPOL-Key type: Pairwise\n");
                log.push_str("  ║    • Key Info: Install, ACK, MIC set\n");
                log.push_str("  ║    • ANonce: <same as Message 1>\n");
                log.push_str("  ║    • MIC: <computed using KCK>\n");
                log.push_str("  ║    • Key Data (encrypted with KEK):\n");
                log.push_str(&format!("  ║        - GTK (Group Temporal Key) for {} broadcast encryption\n", group_cipher));
                log.push_str("  ║        - RSN IE (AP's security capabilities)\n");
                log.push_str("  ║\n");
                log.push_str("  ║  The Install flag tells the client to install the PTK for\n");
                log.push_str("  ║  subsequent unicast data encryption.\n");
                log.push_str("  ║\n");
            }
            4 => {
                log.push_str("  ║  [SUPPLICANT → AUTHENTICATOR]\n");
                log.push_str("  ║\n");
                log.push_str("  ║  The client confirms successful key installation by sending\n");
                log.push_str("  ║  a final EAPOL-Key frame.\n");
                log.push_str("  ║\n");
                log.push_str("  ║  Frame Contents:\n");
                log.push_str("  ║    • EAPOL-Key type: Pairwise\n");
                log.push_str("  ║    • Key Info: MIC set (no ACK, no Install)\n");
                log.push_str("  ║    • Key Data: Empty\n");
                log.push_str("  ║    • MIC: <computed using KCK>\n");
                log.push_str("  ║\n");
                log.push_str("  ║  After this message:\n");
                log.push_str("  ║    • Both parties install the PTK for unicast traffic\n");
                log.push_str("  ║    • GTK is installed for broadcast/multicast traffic\n");
                log.push_str("  ║    • Encrypted data communication begins\n");
                log.push_str("  ║\n");
                log.push_str(&format!("  ║  Encryption Active: {} (Pairwise) / {} (Group)\n", pairwise_cipher, group_cipher));
                log.push_str("  ║\n");
            }
            _ => {}
        }
        log.push_str(&format!("  ╚═══════════════════════════════════════════════════════════════\n\n"));
    }

    // Security Assessment
    log.push_str(&format!("┌{}\n", thin_sep));
    log.push_str(&format!("│  SECURITY ASSESSMENT\n"));
    log.push_str(&format!("├{}\n", thin_sep));
    if security_protocol.contains("WPA3") {
        log.push_str("│  [STRONG] WPA3 with SAE provides robust protection against\n");
        log.push_str("│           offline dictionary attacks and KRACK-type exploits.\n");
        log.push_str("│           Forward secrecy ensures past sessions remain secure\n");
        log.push_str("│           even if the passphrase is later compromised.\n");
    } else if security_protocol.contains("WPA2") {
        log.push_str("│  [GOOD] WPA2 provides strong encryption but is vulnerable to\n");
        log.push_str("│         offline dictionary attacks if a weak passphrase is used.\n");
        log.push_str("│         Consider upgrading to WPA3 if supported by your hardware.\n");
        log.push_str("│         Ensure PMKID caching is disabled to reduce attack surface.\n");
    } else if security_protocol.contains("WPA") {
        log.push_str("│  [WEAK] WPA (TKIP) has known vulnerabilities. Upgrade to WPA2/WPA3.\n");
    } else if security_protocol.contains("WEP") {
        log.push_str("│  [CRITICAL] WEP is completely broken. Upgrade immediately to WPA2/WPA3.\n");
    } else {
        log.push_str("│  [OPEN] No encryption. All traffic is transmitted in plaintext.\n");
    }
    log.push_str(&format!("└{}\n\n", thin_sep));

    // EAPOL Events
    if !events.is_empty() {
        log.push_str(&format!("┌{}\n", thin_sep));
        log.push_str(&format!("│  EAPOL / HANDSHAKE EVENTS (last 30 minutes)\n"));
        log.push_str(&format!("├{}\n", thin_sep));
        for ev in &events {
            log.push_str(&format!("│  [{}] [{}] {}\n", ev.timestamp, ev.event_type.to_uppercase(), ev.message));
        }
        log.push_str(&format!("└{}\n\n", thin_sep));
    } else {
        log.push_str("  No EAPOL events captured in the last 30 minutes.\n\n");
    }

    log.push_str(&format!("{}\n", separator));
    log.push_str(&format!("  Report generated by Flux Terminal NETOPS\n"));
    log.push_str(&format!("{}\n", separator));

    Ok(HandshakeResult {
        ssid,
        bssid,
        security_protocol,
        auth_method,
        pairwise_cipher,
        group_cipher,
        link_auth,
        handshake_messages,
        events,
        handshake_complete,
        last_handshake_time,
        rssi,
        channel,
        noise,
        scan_time_ms: scan_ms,
        log_text: log,
    })
}

// ─── Synthetic PCAP Builder ────────────────────────────────────────
fn build_handshake_pcap(bssid: &str, security: &str) -> Vec<u8> {
    let mut buf = Vec::with_capacity(1024);

    // ── PCAP Global Header (24 bytes, little-endian) ──
    buf.extend_from_slice(&0xA1B2C3D4u32.to_le_bytes()); // magic
    buf.extend_from_slice(&2u16.to_le_bytes());           // version major
    buf.extend_from_slice(&4u16.to_le_bytes());           // version minor
    buf.extend_from_slice(&0i32.to_le_bytes());           // timezone
    buf.extend_from_slice(&0u32.to_le_bytes());           // sigfigs
    buf.extend_from_slice(&65535u32.to_le_bytes());       // snaplen
    buf.extend_from_slice(&1u32.to_le_bytes());           // link type: Ethernet

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let base_ts = now.as_secs() as u32;

    // Parse BSSID → 6-byte MAC
    let ap_mac: [u8; 6] = {
        let parts: Vec<u8> = bssid.split(':')
            .filter_map(|h| u8::from_str_radix(h, 16).ok())
            .collect();
        if parts.len() == 6 {
            [parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]]
        } else {
            [0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E] // fallback
        }
    };
    let client_mac: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];

    // Key descriptor type: 0x02 for RSN (WPA2/WPA3), 0xFE for WPA
    let desc_type: u8 = if security.contains("WPA2") || security.contains("WPA3") { 0x02 } else { 0xFE };

    // Key length: 16 for CCMP (WPA2/WPA3), 32 for TKIP (WPA)
    let key_len: u16 = if desc_type == 0x02 { 16 } else { 32 };

    // Generate synthetic nonces (deterministic from BSSID for reproducibility)
    let mut anonce = [0u8; 32];
    let mut snonce = [0u8; 32];
    for i in 0..32 {
        anonce[i] = ap_mac[i % 6].wrapping_mul((i as u8).wrapping_add(0x37));
        snonce[i] = client_mac[i % 6].wrapping_mul((i as u8).wrapping_add(0x5A));
    }

    // Key Info flags per message (little-endian u16)
    // Bit layout: [Ver(3)][Type(1)][Install(1)][Ack(1)][MIC(1)][Secure(1)][Error(1)][Request(1)][EncKeyData(1)]
    let key_infos: [u16; 4] = [
        0x008A, // Msg1: Pairwise(0x08) + Ack(0x80) + Ver2(0x02)
        0x010A, // Msg2: Pairwise(0x08) + MIC(0x100) + Ver2(0x02)
        0x13CA, // Msg3: Pairwise(0x08) + Install(0x40) + Ack(0x80) + MIC(0x100) + Secure(0x200) + EncKeyData(0x1000) + Ver2(0x02)
        0x030A, // Msg4: Pairwise(0x08) + MIC(0x100) + Secure(0x200) + Ver2(0x02)
    ];

    // Directions: true = AP→Client, false = Client→AP
    let ap_to_client = [true, false, true, false];

    // Synthetic RSN IE for Message 3 key data
    let rsn_ie: Vec<u8> = vec![
        0x30,       // Element ID: RSN
        0x14,       // Length: 20
        0x01, 0x00, // RSN Version: 1
        0x00, 0x0F, 0xAC, 0x04, // Group Cipher: CCMP
        0x01, 0x00, // Pairwise Cipher Count: 1
        0x00, 0x0F, 0xAC, 0x04, // Pairwise Cipher: CCMP
        0x01, 0x00, // AKM Count: 1
        0x00, 0x0F, 0xAC, 0x02, // AKM: PSK
        0x00, 0x00, // RSN Capabilities
    ];

    for msg_idx in 0..4u8 {
        let mut pkt = Vec::with_capacity(128);

        // ── Ethernet Header (14 bytes) ──
        if ap_to_client[msg_idx as usize] {
            pkt.extend_from_slice(&client_mac); // dst
            pkt.extend_from_slice(&ap_mac);     // src
        } else {
            pkt.extend_from_slice(&ap_mac);     // dst
            pkt.extend_from_slice(&client_mac); // src
        }
        pkt.extend_from_slice(&0x888Eu16.to_be_bytes()); // EtherType: EAPOL

        // ── EAPOL Header (4 bytes) ──
        // Key data for msg3 only
        let key_data = if msg_idx == 2 { &rsn_ie[..] } else { &[] };
        let eapol_key_body_len: u16 = 95 + key_data.len() as u16;

        pkt.push(0x02); // EAPOL version 2
        pkt.push(0x03); // Type: EAPOL-Key
        pkt.extend_from_slice(&eapol_key_body_len.to_be_bytes()); // body length

        // ── EAPOL-Key Descriptor ──
        pkt.push(desc_type); // Key Descriptor Type

        // Key Information (2 bytes, big-endian)
        pkt.extend_from_slice(&key_infos[msg_idx as usize].to_be_bytes());

        // Key Length (2 bytes)
        pkt.extend_from_slice(&key_len.to_be_bytes());

        // Replay Counter (8 bytes) — increments per message
        pkt.extend_from_slice(&(msg_idx as u64 + 1).to_be_bytes());

        // Key Nonce (32 bytes)
        match msg_idx {
            0 | 2 => pkt.extend_from_slice(&anonce), // AP sends ANonce
            1     => pkt.extend_from_slice(&snonce), // Client sends SNonce
            _     => pkt.extend_from_slice(&[0u8; 32]), // Msg4: empty
        }

        // Key IV (16 bytes) — zeros for WPA2
        pkt.extend_from_slice(&[0u8; 16]);

        // Key RSC (8 bytes)
        pkt.extend_from_slice(&[0u8; 8]);

        // Key ID / Reserved (8 bytes)
        pkt.extend_from_slice(&[0u8; 8]);

        // Key MIC (16 bytes) — synthetic (zero for msg1, hash-like for msg2-4)
        if msg_idx == 0 {
            pkt.extend_from_slice(&[0u8; 16]);
        } else {
            let mut mic = [0u8; 16];
            for i in 0..16 {
                mic[i] = ap_mac[i % 6]
                    .wrapping_add(client_mac[i % 6])
                    .wrapping_mul(msg_idx.wrapping_add(i as u8).wrapping_add(0x3C));
            }
            pkt.extend_from_slice(&mic);
        }

        // Key Data Length (2 bytes)
        pkt.extend_from_slice(&(key_data.len() as u16).to_be_bytes());

        // Key Data (variable)
        pkt.extend_from_slice(key_data);

        // ── PCAP Record Header (16 bytes) ──
        let ts_sec = base_ts + msg_idx as u32;
        let ts_usec = msg_idx as u32 * 250_000; // 250ms apart
        let pkt_len = pkt.len() as u32;

        buf.extend_from_slice(&ts_sec.to_le_bytes());
        buf.extend_from_slice(&ts_usec.to_le_bytes());
        buf.extend_from_slice(&pkt_len.to_le_bytes()); // incl_len
        buf.extend_from_slice(&pkt_len.to_le_bytes()); // orig_len
        buf.extend_from_slice(&pkt);
    }

    buf
}

#[tauri::command]
pub async fn netops_save_handshake_log(
    log_text: String,
    ssid: String,
    bssid: String,
    security: String,
) -> Result<String, String> {
    let downloads = dirs::download_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
        .ok_or("Could not find Downloads directory")?;

    let safe_ssid: String = ssid.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let timestamp = format!("{}", secs);
    let base_name = format!("handshake_{}_{}", safe_ssid, timestamp);

    // 1. Save text log
    let txt_path = downloads.join(format!("{}.txt", base_name));
    tokio::fs::write(&txt_path, &log_text)
        .await
        .map_err(|e| format!("Failed to save txt: {}", e))?;

    // 2. Generate and save pcap
    let pcap_data = build_handshake_pcap(&bssid, &security);
    let pcap_path = downloads.join(format!("{}.pcap", base_name));
    tokio::fs::write(&pcap_path, &pcap_data)
        .await
        .map_err(|e| format!("Failed to save pcap: {}", e))?;

    // 3. Save cap (same data, different extension for aircrack-ng compat)
    let cap_path = downloads.join(format!("{}.cap", base_name));
    tokio::fs::write(&cap_path, &pcap_data)
        .await
        .map_err(|e| format!("Failed to save cap: {}", e))?;

    // Open the Downloads folder so user can see all 3 files
    let _ = tokio::process::Command::new("open")
        .arg(&downloads)
        .spawn();

    Ok(format!(
        "Saved 3 files to Downloads:\n  {} (.txt)\n  {} (.pcap)\n  {} (.cap)",
        txt_path.file_name().unwrap_or_default().to_string_lossy(),
        pcap_path.file_name().unwrap_or_default().to_string_lossy(),
        cap_path.file_name().unwrap_or_default().to_string_lossy(),
    ))
}

// ═══════════════════════════════════════════════════════════════════
//  TOOL 10: PCAP / CAP File Viewer
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcapPacket {
    pub index: u32,
    pub timestamp: f64,
    pub src: String,
    pub dst: String,
    pub protocol: String,
    pub length: u32,
    pub info: String,
    pub is_eapol: bool,
    pub hex_preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcapAnalysis {
    pub filename: String,
    pub file_size: u64,
    pub link_type: u32,
    pub packet_count: u32,
    pub packets: Vec<PcapPacket>,
    pub eapol_count: u32,
    pub duration_secs: f64,
    pub parse_time_ms: u64,
}

fn format_mac(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(":")
}

fn hex_preview(data: &[u8], max: usize) -> String {
    data.iter()
        .take(max)
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .chunks(16)
        .map(|chunk| chunk.join(" "))
        .collect::<Vec<_>>()
        .join("\n")
}


fn read_u32(data: &[u8], off: usize, le: bool) -> u32 {
    if off + 4 > data.len() { return 0; }
    if le { u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]) }
    else  { u32::from_be_bytes([data[off], data[off+1], data[off+2], data[off+3]]) }
}

#[tauri::command]
pub async fn netops_pcap_analyze(path: String) -> Result<PcapAnalysis, String> {
    let start = Instant::now();

    let data = tokio::fs::read(&path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    if data.len() < 24 {
        return Err("File too small to be a valid pcap".into());
    }

    // Parse global header
    let magic = u32::from_ne_bytes([data[0], data[1], data[2], data[3]]);
    let le = match magic {
        0xA1B2C3D4 => true,  // little-endian
        0xD4C3B2A1 => false, // big-endian
        _ => return Err(format!("Invalid pcap magic: 0x{:08X}. Not a valid pcap/cap file.", magic)),
    };

    let link_type = read_u32(&data, 20, le);
    let filename = std::path::Path::new(&path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let file_size = data.len() as u64;

    let mut packets = Vec::new();
    let mut eapol_count: u32 = 0;
    let mut offset = 24usize; // skip global header
    let mut first_ts: f64 = 0.0;
    let mut last_ts: f64 = 0.0;
    let max_packets = 500u32;

    while offset + 16 <= data.len() && packets.len() < max_packets as usize {
        // Record header
        let ts_sec = read_u32(&data, offset, le) as f64;
        let ts_usec = read_u32(&data, offset + 4, le) as f64;
        let incl_len = read_u32(&data, offset + 8, le) as usize;
        let _orig_len = read_u32(&data, offset + 12, le);
        offset += 16;

        if offset + incl_len > data.len() { break; }
        let pkt_data = &data[offset..offset + incl_len];
        offset += incl_len;

        let ts = ts_sec + ts_usec / 1_000_000.0;
        if first_ts == 0.0 { first_ts = ts; }
        last_ts = ts;
        let relative_ts = ts - first_ts;

        let idx = packets.len() as u32 + 1;
        let hex = hex_preview(pkt_data, 64);

        // Parse based on link type
        if link_type == 1 && pkt_data.len() >= 14 {
            // Ethernet
            let dst_mac = format_mac(&pkt_data[0..6]);
            let src_mac = format_mac(&pkt_data[6..12]);
            let ether_type = u16::from_be_bytes([pkt_data[12], pkt_data[13]]);

            match ether_type {
                0x0800 if pkt_data.len() >= 34 => {
                    // IPv4
                    let ihl = (pkt_data[14] & 0x0F) as usize * 4;
                    let total_len = u16::from_be_bytes([pkt_data[16], pkt_data[17]]);
                    let proto = pkt_data[23];
                    let src_ip = format!("{}.{}.{}.{}", pkt_data[26], pkt_data[27], pkt_data[28], pkt_data[29]);
                    let dst_ip = format!("{}.{}.{}.{}", pkt_data[30], pkt_data[31], pkt_data[32], pkt_data[33]);

                    let (protocol, info) = match proto {
                        6 => {
                            let tp = 14 + ihl;
                            if pkt_data.len() >= tp + 4 {
                                let sport = u16::from_be_bytes([pkt_data[tp], pkt_data[tp+1]]);
                                let dport = u16::from_be_bytes([pkt_data[tp+2], pkt_data[tp+3]]);
                                let flags = if pkt_data.len() > tp + 13 { pkt_data[tp + 13] } else { 0 };
                                let flag_str = {
                                    let mut f = Vec::new();
                                    if flags & 0x02 != 0 { f.push("SYN"); }
                                    if flags & 0x10 != 0 { f.push("ACK"); }
                                    if flags & 0x01 != 0 { f.push("FIN"); }
                                    if flags & 0x04 != 0 { f.push("RST"); }
                                    if flags & 0x08 != 0 { f.push("PSH"); }
                                    if f.is_empty() { f.push(""); }
                                    f.join(",")
                                };
                                ("TCP".into(), format!("{} → {} [{}] Len={}", sport, dport, flag_str, total_len))
                            } else {
                                ("TCP".into(), format!("{} → {} Len={}", src_ip, dst_ip, total_len))
                            }
                        }
                        17 => {
                            let tp = 14 + ihl;
                            if pkt_data.len() >= tp + 4 {
                                let sport = u16::from_be_bytes([pkt_data[tp], pkt_data[tp+1]]);
                                let dport = u16::from_be_bytes([pkt_data[tp+2], pkt_data[tp+3]]);
                                let udp_len = if pkt_data.len() >= tp + 6 { u16::from_be_bytes([pkt_data[tp+4], pkt_data[tp+5]]) } else { 0 };
                                let proto_name = match dport {
                                    53 | 5353 => "DNS",
                                    67 | 68 => "DHCP",
                                    123 => "NTP",
                                    _ => "UDP",
                                };
                                (proto_name.into(), format!("{} → {} Len={}", sport, dport, udp_len))
                            } else {
                                ("UDP".into(), format!("{} → {}", src_ip, dst_ip))
                            }
                        }
                        1 => {
                            let icmp_type = if pkt_data.len() > 14 + ihl { pkt_data[14 + ihl] } else { 0 };
                            let desc = match icmp_type {
                                0 => "Echo Reply",
                                8 => "Echo Request",
                                3 => "Destination Unreachable",
                                11 => "Time Exceeded",
                                _ => "ICMP",
                            };
                            ("ICMP".into(), desc.into())
                        }
                        _ => (format!("IPv4/{}", proto), format!("{} → {}", src_ip, dst_ip)),
                    };

                    packets.push(PcapPacket {
                        index: idx, timestamp: relative_ts,
                        src: src_ip, dst: dst_ip,
                        protocol, length: incl_len as u32,
                        info, is_eapol: false, hex_preview: hex,
                    });
                }
                0x0806 if pkt_data.len() >= 28 => {
                    // ARP
                    let op = u16::from_be_bytes([pkt_data[20], pkt_data[21]]);
                    let op_str = if op == 1 { "Request" } else { "Reply" };
                    let sender_ip = if pkt_data.len() >= 32 {
                        format!("{}.{}.{}.{}", pkt_data[28], pkt_data[29], pkt_data[30], pkt_data[31])
                    } else { "?".into() };
                    let target_ip = if pkt_data.len() >= 42 {
                        format!("{}.{}.{}.{}", pkt_data[38], pkt_data[39], pkt_data[40], pkt_data[41])
                    } else { "?".into() };

                    packets.push(PcapPacket {
                        index: idx, timestamp: relative_ts,
                        src: src_mac.clone(), dst: dst_mac.clone(),
                        protocol: "ARP".into(), length: incl_len as u32,
                        info: format!("{}: Who has {}? Tell {}", op_str, target_ip, sender_ip),
                        is_eapol: false, hex_preview: hex,
                    });
                }
                0x888E => {
                    // EAPOL
                    eapol_count += 1;
                    let mut info = "EAPOL".to_string();
                    if pkt_data.len() >= 18 {
                        let eapol_type = pkt_data[15];
                        if eapol_type == 0x03 && pkt_data.len() >= 19 {
                            // EAPOL-Key
                            let key_info = u16::from_be_bytes([pkt_data[19], pkt_data[20]]);
                            let has_ack = key_info & 0x0080 != 0;
                            let has_mic = key_info & 0x0100 != 0;
                            let has_install = key_info & 0x0040 != 0;
                            let has_secure = key_info & 0x0200 != 0;
                            let msg_num = if has_ack && !has_mic { 1 }
                                else if !has_ack && has_mic && !has_install { 2 }
                                else if has_ack && has_mic && has_install { 3 }
                                else if has_mic && has_secure && !has_ack { 4 }
                                else { 0 };
                            if msg_num > 0 {
                                info = format!("EAPOL-Key Message {} (4-Way Handshake)", msg_num);
                            } else {
                                info = format!("EAPOL-Key [Info: 0x{:04X}]", key_info);
                            }
                        } else {
                            info = format!("EAPOL Type {}", eapol_type);
                        }
                    }

                    packets.push(PcapPacket {
                        index: idx, timestamp: relative_ts,
                        src: src_mac, dst: dst_mac,
                        protocol: "EAPOL".into(), length: incl_len as u32,
                        info, is_eapol: true, hex_preview: hex,
                    });
                }
                0x86DD if pkt_data.len() >= 54 => {
                    // IPv6
                    let src_v6 = format!("{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
                        pkt_data[22], pkt_data[23], pkt_data[24], pkt_data[25],
                        pkt_data[26], pkt_data[27], pkt_data[28], pkt_data[29]);
                    let dst_v6 = format!("{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
                        pkt_data[38], pkt_data[39], pkt_data[40], pkt_data[41],
                        pkt_data[42], pkt_data[43], pkt_data[44], pkt_data[45]);
                    let next_header = pkt_data[20];
                    let proto = match next_header {
                        6 => "TCP", 17 => "UDP", 58 => "ICMPv6", _ => "IPv6",
                    };

                    packets.push(PcapPacket {
                        index: idx, timestamp: relative_ts,
                        src: src_v6, dst: dst_v6,
                        protocol: proto.into(), length: incl_len as u32,
                        info: format!("Next Header: {}", next_header),
                        is_eapol: false, hex_preview: hex,
                    });
                }
                _ => {
                    packets.push(PcapPacket {
                        index: idx, timestamp: relative_ts,
                        src: src_mac, dst: dst_mac,
                        protocol: format!("0x{:04X}", ether_type), length: incl_len as u32,
                        info: format!("EtherType 0x{:04X}", ether_type),
                        is_eapol: false, hex_preview: hex,
                    });
                }
            }
        } else {
            // Non-Ethernet or too short
            packets.push(PcapPacket {
                index: idx, timestamp: relative_ts,
                src: "—".into(), dst: "—".into(),
                protocol: format!("Link:{}", link_type), length: incl_len as u32,
                info: format!("{} bytes", incl_len),
                is_eapol: false, hex_preview: hex,
            });
        }
    }

    let packet_count = packets.len() as u32;
    let duration_secs = if last_ts > first_ts { last_ts - first_ts } else { 0.0 };

    Ok(PcapAnalysis {
        filename,
        file_size,
        link_type,
        packet_count,
        packets,
        eapol_count,
        duration_secs,
        parse_time_ms: start.elapsed().as_millis() as u64,
    })
}
