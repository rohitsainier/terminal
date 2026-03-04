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

fn scan_wifi_networks() -> Result<Vec<WifiNetwork>, String> {
    // Synchronous helper that calls the swift script
    // We'll call this from async context with spawn_blocking or just inline the command
    Ok(Vec::new()) // placeholder, actual scanning done in the async commands
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
