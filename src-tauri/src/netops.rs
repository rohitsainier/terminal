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
    let output = tokio::process::Command::new(
        "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport",
    )
    .arg("-s")
    .output()
    .await
    .map_err(|e| format!("airport scan: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();

    for (i, line) in stdout.lines().enumerate() {
        if i == 0 {
            continue; // skip header
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // airport output is fixed-width: SSID takes first 33 chars, then BSSID, RSSI, CHANNEL, HT, CC, SECURITY
        // But SSID can have spaces, so we parse from the right side
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }

        // Find BSSID (MAC format xx:xx:xx:xx:xx:xx) to anchor parsing
        let bssid_idx = parts.iter().position(|p| p.matches(':').count() == 5);
        if let Some(bi) = bssid_idx {
            let ssid = if bi > 0 {
                parts[..bi].join(" ")
            } else {
                "(hidden)".into()
            };
            let bssid = parts[bi].to_string();
            let rssi = parts.get(bi + 1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
            let channel = parts.get(bi + 2).and_then(|s| {
                // Channel can be like "1" or "1,+1" — take first number
                s.split(',').next().and_then(|c| c.parse::<u32>().ok())
            }).unwrap_or(0);
            // Security is everything after HT and CC columns
            let security = if parts.len() > bi + 5 {
                parts[bi + 5..].join(" ")
            } else {
                "Unknown".into()
            };

            networks.push(WifiNetwork {
                ssid,
                bssid,
                rssi,
                channel,
                security,
            });
        }
    }

    networks.sort_by(|a, b| b.rssi.cmp(&a.rssi));
    Ok(networks)
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
