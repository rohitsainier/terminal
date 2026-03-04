// ═══════════════════════════════════════════════════════════════════
//  FLUX CYBER COMMAND — Monitor Backend
//  TLE Satellites · Live Flights · ISS · News · System · Activity
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
pub struct SatTLE {
    pub name: String,
    pub line1: String,
    pub line2: String,
    pub group: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightInfo {
    pub icao24: String,
    pub callsign: String,
    pub origin_country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub velocity: f64,
    pub heading: f64,
    pub on_ground: bool,
    pub vertical_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ISSPos {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub velocity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub title: String,
    pub source: String,
    pub timestamp: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysStats {
    pub os: String,
    pub hostname: String,
    pub uptime_secs: u64,
    pub cpu_count: usize,
    pub memory_total_mb: u64,
    pub memory_used_mb: u64,
    pub local_ip: String,
    pub public_ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub lat: f64,
    pub lon: f64,
    pub label: String,
    pub event_type: String,
    pub intensity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherPoint {
    pub city: String,
    pub country: String,
    pub lat: f64,
    pub lng: f64,
    pub temperature: f64,
    pub humidity: u16,
    pub wind_speed: f64,
    pub weather_code: u16,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuakeEvent {
    pub id: String,
    pub place: String,
    pub lat: f64,
    pub lng: f64,
    pub magnitude: f64,
    pub depth: f64,
    pub time: u64,
    pub url: String,
    pub tsunami: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoPrice {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_24h: f64,
    pub market_cap: f64,
    pub volume_24h: f64,
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

// Each cache is a function returning a &'static Mutex<...>
// Uses OnceLock (stable since Rust 1.70, Tauri v2 requires 1.77+)

fn tle_cache() -> &'static Mutex<HashMap<String, CacheEntry<Vec<SatTLE>>>> {
    static C: OnceLock<Mutex<HashMap<String, CacheEntry<Vec<SatTLE>>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

fn flight_cache() -> &'static Mutex<Option<CacheEntry<Vec<FlightInfo>>>> {
    static C: OnceLock<Mutex<Option<CacheEntry<Vec<FlightInfo>>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(None))
}

fn iss_cache() -> &'static Mutex<Option<CacheEntry<ISSPos>>> {
    static C: OnceLock<Mutex<Option<CacheEntry<ISSPos>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(None))
}

fn news_cache() -> &'static Mutex<Option<CacheEntry<Vec<NewsItem>>>> {
    static C: OnceLock<Mutex<Option<CacheEntry<Vec<NewsItem>>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(None))
}

fn ip_cache() -> &'static Mutex<Option<CacheEntry<String>>> {
    static C: OnceLock<Mutex<Option<CacheEntry<String>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(None))
}

fn weather_cache() -> &'static Mutex<Option<CacheEntry<Vec<WeatherPoint>>>> {
    static C: OnceLock<Mutex<Option<CacheEntry<Vec<WeatherPoint>>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(None))
}

fn quake_cache() -> &'static Mutex<Option<CacheEntry<Vec<QuakeEvent>>>> {
    static C: OnceLock<Mutex<Option<CacheEntry<Vec<QuakeEvent>>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(None))
}

fn crypto_cache() -> &'static Mutex<Option<CacheEntry<Vec<CryptoPrice>>>> {
    static C: OnceLock<Mutex<Option<CacheEntry<Vec<CryptoPrice>>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(None))
}

/// Helper: lock a mutex, converting PoisonError to String
fn lock_cache<T>(m: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, String> {
    m.lock().map_err(|e| format!("cache lock: {}", e))
}

// ═══════════════════════════════════════════════════════════════════
//  HTTP CLIENT
// ═══════════════════════════════════════════════════════════════════

fn http_client(timeout_secs: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent("FluxTerminal/2.0")
        .build()
        .map_err(|e| format!("http client: {}", e))
}

// ═══════════════════════════════════════════════════════════════════
//  TLE GROUP MAPPING
// ═══════════════════════════════════════════════════════════════════

fn tle_group_url(group: &str) -> String {
    let celestrak_id = match group {
        "stations" => "stations",
        "starlink" => "starlink",
        "gps" => "gps-ops",
        "weather" => "weather",
        "oneweb" => "oneweb",
        "iridium" => "iridium-NEXT",
        "geo" => "geo",
        "science" => "science",
        other => other,
    };
    format!(
        "https://celestrak.org/NORAD/elements/gp.php?GROUP={}&FORMAT=tle",
        celestrak_id
    )
}

/// Parse CelesTrak 3-line TLE format
fn parse_tle_text(text: &str, group: &str) -> Vec<SatTLE> {
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    let mut result = Vec::new();
    let mut i = 0;

    while i + 2 < lines.len() {
        let name_line = lines[i];
        let line1 = lines[i + 1];
        let line2 = lines[i + 2];

        if line1.starts_with('1') && line2.starts_with('2') {
            result.push(SatTLE {
                name: name_line.trim_start_matches("0 ").to_string(),
                line1: line1.to_string(),
                line2: line2.to_string(),
                group: group.to_string(),
            });
            i += 3;
        } else {
            // Misaligned — skip one line and retry
            i += 1;
        }
    }

    result
}

// ═══════════════════════════════════════════════════════════════════
//  SIMPLE RSS / XML HELPERS
// ═══════════════════════════════════════════════════════════════════

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);

    let start_pos = xml.find(&open)?;
    let gt = xml[start_pos..].find('>')?;
    let content_start = start_pos + gt + 1;
    let end_pos = xml[content_start..].find(&close)?;

    Some(xml[content_start..content_start + end_pos].to_string())
}

/// Extract link from an RSS/Atom item.
/// RSS: `<link>https://...</link>`
/// Atom: `<link href="https://..." .../>` or `<link href="https://..."></link>`
fn extract_item_link(item_xml: &str) -> String {
    // Try Atom-style href attribute first
    if let Some(link_start) = item_xml.find("<link") {
        let rest = &item_xml[link_start..];
        if let Some(href_pos) = rest.find("href=") {
            let after_href = &rest[href_pos + 5..];
            let quote = after_href.chars().next().unwrap_or('"');
            if quote == '"' || quote == '\'' {
                let inner = &after_href[1..];
                if let Some(end) = inner.find(quote) {
                    let url = inner[..end].trim().to_string();
                    if url.starts_with("http") {
                        return url;
                    }
                }
            }
        }
    }
    // Fall back to RSS-style <link>text</link>
    extract_xml_tag(item_xml, "link")
        .map(|s| s.trim().to_string())
        .filter(|s| s.starts_with("http"))
        .unwrap_or_default()
}

fn clean_html(s: &str) -> String {
    let mut out = s.to_string();
    // Strip CDATA wrappers
    out = out.replace("<![CDATA[", "").replace("]]>", "");
    // Strip common HTML tags
    let tags = ["<p>", "</p>", "<b>", "</b>", "<i>", "</i>", "<br>", "<br/>"];
    for t in tags {
        out = out.replace(t, "");
    }
    // Strip remaining tags (rough)
    while let (Some(open), Some(_close)) = (out.find('<'), out.find('>')) {
        if open < out.len() {
            let close = out[open..].find('>').unwrap_or(0) + open;
            if close > open && close < out.len() {
                out = format!("{}{}", &out[..open], &out[close + 1..]);
            } else {
                break;
            }
        } else {
            break;
        }
    }
    // Decode common entities
    out = out
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'");
    out.trim().to_string()
}

async fn fetch_rss_feed(
    client: &reqwest::Client,
    url: &str,
    source: &str,
    max_items: usize,
) -> Vec<NewsItem> {
    let resp = match client.get(url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return Vec::new(),
    };
    let text = match resp.text().await {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };

    let mut items = Vec::new();
    let mut pos = 0;

    // Handle both <item> (RSS) and <entry> (Atom)
    let (open_tag, close_tag) = if text.contains("<entry") {
        ("<entry", "</entry>")
    } else {
        ("<item", "</item>")
    };

    while let Some(item_start) = text[pos..].find(open_tag) {
        let abs_start = pos + item_start;
        let Some(item_end) = text[abs_start..].find(close_tag) else {
            break;
        };
        let item_xml = &text[abs_start..abs_start + item_end];

        let title = extract_xml_tag(item_xml, "title")
            .map(|t| clean_html(&t))
            .unwrap_or_default();

        let pub_date = extract_xml_tag(item_xml, "pubDate")
            .or_else(|| extract_xml_tag(item_xml, "published"))
            .or_else(|| extract_xml_tag(item_xml, "updated"))
            .unwrap_or_default();

        let timestamp = if pub_date.len() > 22 {
            pub_date[..22].trim().to_string()
        } else {
            pub_date.trim().to_string()
        };

        let url = extract_item_link(item_xml);

        if !title.is_empty() && title.len() > 5 {
            items.push(NewsItem {
                title,
                source: source.to_string(),
                timestamp,
                url,
            });
        }

        pos = abs_start + item_end + close_tag.len();
        if items.len() >= max_items {
            break;
        }
    }

    items
}

// ═══════════════════════════════════════════════════════════════════
//  NETWORK HELPERS
// ═══════════════════════════════════════════════════════════════════

fn get_local_ip() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("8.8.8.8:80")?;
            s.local_addr()
        })
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

// ═══════════════════════════════════════════════════════════════════
//  ACTIVITY DATA (SIMULATED)
// ═══════════════════════════════════════════════════════════════════

struct CityInfo {
    lat: f64,
    lon: f64,
    name: &'static str,
}

const ACTIVITY_CITIES: &[CityInfo] = &[
    CityInfo { lat: 40.71, lon: -74.0, name: "New York" },
    CityInfo { lat: 51.51, lon: -0.13, name: "London" },
    CityInfo { lat: 35.68, lon: 139.69, name: "Tokyo" },
    CityInfo { lat: 22.32, lon: 114.17, name: "Hong Kong" },
    CityInfo { lat: 37.77, lon: -122.42, name: "San Francisco" },
    CityInfo { lat: -33.87, lon: 151.21, name: "Sydney" },
    CityInfo { lat: 55.76, lon: 37.62, name: "Moscow" },
    CityInfo { lat: 1.35, lon: 103.82, name: "Singapore" },
    CityInfo { lat: 48.86, lon: 2.35, name: "Paris" },
    CityInfo { lat: 52.52, lon: 13.41, name: "Berlin" },
    CityInfo { lat: 19.08, lon: 72.88, name: "Mumbai" },
    CityInfo { lat: -23.55, lon: -46.63, name: "São Paulo" },
    CityInfo { lat: 39.9, lon: 116.4, name: "Beijing" },
    CityInfo { lat: 37.57, lon: 126.98, name: "Seoul" },
    CityInfo { lat: 25.2, lon: 55.27, name: "Dubai" },
    CityInfo { lat: 30.04, lon: 31.24, name: "Cairo" },
    CityInfo { lat: -1.29, lon: 36.82, name: "Nairobi" },
    CityInfo { lat: 41.01, lon: 29.0, name: "Istanbul" },
    CityInfo { lat: 13.76, lon: 100.5, name: "Bangkok" },
    CityInfo { lat: -6.21, lon: 106.85, name: "Jakarta" },
    CityInfo { lat: 28.61, lon: 77.21, name: "Delhi" },
    CityInfo { lat: 31.23, lon: 121.47, name: "Shanghai" },
    CityInfo { lat: 33.94, lon: -118.24, name: "Los Angeles" },
    CityInfo { lat: 43.65, lon: -79.38, name: "Toronto" },
    CityInfo { lat: -34.6, lon: -58.38, name: "Buenos Aires" },
    CityInfo { lat: 59.33, lon: 18.07, name: "Stockholm" },
    CityInfo { lat: 38.9, lon: -77.04, name: "Washington DC" },
    CityInfo { lat: 35.69, lon: 51.39, name: "Tehran" },
    CityInfo { lat: 6.52, lon: 3.38, name: "Lagos" },
    CityInfo { lat: -33.93, lon: 18.42, name: "Cape Town" },
];

const EVENT_TYPES: &[&str] = &[
    "DATA_BURST",
    "NET_SCAN",
    "AUTH_SPIKE",
    "TRAFFIC_PEAK",
    "SYS_ALERT",
    "LINK_EST",
    "SYNC_PULSE",
    "API_FLOOD",
    "KEY_EXCHANGE",
    "BEACON",
    "HANDSHAKE",
    "PROBE",
];

fn pseudo_random(seed: u64, index: usize) -> usize {
    // Simple LCG-style pseudo-random
    let val = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
        .wrapping_add(index as u64);
    (val >> 16) as usize
}

fn generate_activity() -> Vec<Activity> {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let count = 8 + (seed as usize % 5); // 8-12 items

    (0..count)
        .map(|i| {
            let city_idx = pseudo_random(seed, i * 3) % ACTIVITY_CITIES.len();
            let event_idx = pseudo_random(seed, i * 7 + 1) % EVENT_TYPES.len();
            let intensity_raw = pseudo_random(seed, i * 11 + 2) % 100;

            let city = &ACTIVITY_CITIES[city_idx];

            Activity {
                lat: city.lat + (pseudo_random(seed, i * 13) % 100) as f64 * 0.01 - 0.5,
                lon: city.lon + (pseudo_random(seed, i * 17) % 100) as f64 * 0.01 - 0.5,
                label: city.name.to_string(),
                event_type: EVENT_TYPES[event_idx].to_string(),
                intensity: (intensity_raw as f64) / 100.0,
            }
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════
//  TAURI COMMANDS
// ═══════════════════════════════════════════════════════════════════

// ─── 1. Fetch TLE Data ──────────────────────────────────────────

#[tauri::command]
pub async fn monitor_fetch_tle(group: String) -> Result<Vec<SatTLE>, String> {
    // Check cache (TTL: 2 hours)
    {
        let cache = lock_cache(tle_cache())?;
        if let Some(entry) = cache.get(&group) {
            if entry.is_fresh(Duration::from_secs(7200)) {
                return Ok(entry.data.clone());
            }
        }
    }

    let client = http_client(30)?;
    let url = tle_group_url(&group);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("TLE fetch failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "CelesTrak returned HTTP {}",
            response.status()
        ));
    }

    let text = response.text().await.map_err(|e| e.to_string())?;
    let tles = parse_tle_text(&text, &group);

    if tles.is_empty() {
        return Err(format!("No TLE data parsed for group '{}'", group));
    }

    // Store in cache
    {
        let mut cache = lock_cache(tle_cache())?;
        cache.insert(group, CacheEntry::new(tles.clone()));
    }

    Ok(tles)
}

// ─── 2. Fetch Live Flights ──────────────────────────────────────

#[tauri::command]
pub async fn monitor_flights() -> Result<Vec<FlightInfo>, String> {
    // Check cache (TTL: 15 seconds)
    {
        let cache = lock_cache(flight_cache())?;
        if let Some(ref entry) = *cache {
            if entry.is_fresh(Duration::from_secs(15)) {
                return Ok(entry.data.clone());
            }
        }
    }

    let client = http_client(25)?;

    let response = client
        .get("https://opensky-network.org/api/states/all")
        .send()
        .await
        .map_err(|e| format!("OpenSky fetch failed: {}", e))?;

    if !response.status().is_success() {
        // If rate-limited, return cached data if any
        let cache = lock_cache(flight_cache())?;
        if let Some(ref entry) = *cache {
            return Ok(entry.data.clone());
        }
        return Err(format!("OpenSky HTTP {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("JSON parse: {}", e))?;

    let states = json["states"]
        .as_array()
        .ok_or("No 'states' array in OpenSky response")?;

    let mut flights = Vec::with_capacity(500);

    for state in states.iter().take(800) {
        let arr = match state.as_array() {
            Some(a) if a.len() >= 14 => a,
            _ => continue,
        };

        let on_ground = arr[8].as_bool().unwrap_or(true);
        // OpenSky: index 6 = latitude, index 5 = longitude
        let lat = match arr[6].as_f64() {
            Some(v) if v != 0.0 => v,
            _ => continue,
        };
        let lon = match arr[5].as_f64() {
            Some(v) if v != 0.0 => v,
            _ => continue,
        };

        // Skip aircraft on the ground
        if on_ground {
            continue;
        }

        flights.push(FlightInfo {
            icao24: arr[0].as_str().unwrap_or("").to_string(),
            callsign: arr[1]
                .as_str()
                .unwrap_or("")
                .trim()
                .to_string(),
            origin_country: arr[2].as_str().unwrap_or("").to_string(),
            latitude: lat,
            longitude: lon,
            // Prefer geometric altitude (13), fallback to barometric (7)
            altitude: arr[13]
                .as_f64()
                .or_else(|| arr[7].as_f64())
                .unwrap_or(0.0),
            velocity: arr[9].as_f64().unwrap_or(0.0),
            heading: arr[10].as_f64().unwrap_or(0.0),
            on_ground,
            vertical_rate: arr[11].as_f64().unwrap_or(0.0),
        });

        if flights.len() >= 500 {
            break;
        }
    }

    // Store in cache
    {
        let mut cache = lock_cache(flight_cache())?;
        *cache = Some(CacheEntry::new(flights.clone()));
    }

    Ok(flights)
}

// ─── 3. ISS Position ────────────────────────────────────────────

#[tauri::command]
pub async fn monitor_iss_position() -> Result<ISSPos, String> {
    // Check cache (TTL: 5 seconds)
    {
        let cache = lock_cache(iss_cache())?;
        if let Some(ref entry) = *cache {
            if entry.is_fresh(Duration::from_secs(5)) {
                return Ok(entry.data.clone());
            }
        }
    }

    let client = http_client(10)?;

    // Primary: wheretheiss.at (has altitude + velocity)
    let pos = match client
        .get("https://api.wheretheiss.at/v1/satellites/25544")
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let json: serde_json::Value =
                resp.json().await.map_err(|e| e.to_string())?;
            ISSPos {
                latitude: json["latitude"].as_f64().unwrap_or(0.0),
                longitude: json["longitude"].as_f64().unwrap_or(0.0),
                altitude: json["altitude"].as_f64().unwrap_or(408.0),
                velocity: json["velocity"].as_f64().unwrap_or(27600.0),
            }
        }
        _ => {
            // Fallback: open-notify
            let resp = client
                .get("http://api.open-notify.org/iss-now.json")
                .send()
                .await
                .map_err(|e| format!("ISS fallback failed: {}", e))?;

            let json: serde_json::Value =
                resp.json().await.map_err(|e| e.to_string())?;

            let lat = json["iss_position"]["latitude"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| json["iss_position"]["latitude"].as_f64())
                .unwrap_or(0.0);
            let lon = json["iss_position"]["longitude"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| json["iss_position"]["longitude"].as_f64())
                .unwrap_or(0.0);

            ISSPos {
                latitude: lat,
                longitude: lon,
                altitude: 408.0,
                velocity: 27600.0,
            }
        }
    };

    // Validate we got something real
    if pos.latitude == 0.0 && pos.longitude == 0.0 {
        // Return cached if available, even if stale
        let cache = lock_cache(iss_cache())?;
        if let Some(ref entry) = *cache {
            return Ok(entry.data.clone());
        }
        return Err("ISS position unavailable".to_string());
    }

    // Update cache
    {
        let mut cache = lock_cache(iss_cache())?;
        *cache = Some(CacheEntry::new(pos.clone()));
    }

    Ok(pos)
}

// ─── 4. News ────────────────────────────────────────────────────

#[tauri::command]
pub async fn monitor_news() -> Result<Vec<NewsItem>, String> {
    // Check cache (TTL: 60 seconds)
    {
        let cache = lock_cache(news_cache())?;
        if let Some(ref entry) = *cache {
            if entry.is_fresh(Duration::from_secs(60)) {
                return Ok(entry.data.clone());
            }
        }
    }

    let client = http_client(15)?;

    let feeds: Vec<(&str, &str)> = vec![
        ("https://feeds.bbci.co.uk/news/world/rss.xml", "BBC"),
        (
            "https://rss.nytimes.com/services/xml/rss/nyt/World.xml",
            "NYT",
        ),
        (
            "https://www.aljazeera.com/xml/rss/all.xml",
            "Al Jazeera",
        ),
        ("http://rss.cnn.com/rss/edition_world.rss", "CNN"),
        ("https://feeds.npr.org/1001/rss.xml", "NPR"),
    ];

    let mut all_news: Vec<NewsItem> = Vec::new();

    for (url, source) in &feeds {
        let items = fetch_rss_feed(&client, url, source, 6).await;
        all_news.extend(items);
    }

    // Interleave sources for variety (round-robin by source)
    let mut grouped: HashMap<String, Vec<NewsItem>> = HashMap::new();
    for item in all_news {
        grouped
            .entry(item.source.clone())
            .or_default()
            .push(item);
    }
    let mut interleaved: Vec<NewsItem> = Vec::new();
    let max_per_source = grouped
        .values()
        .map(|v| v.len())
        .max()
        .unwrap_or(0);
    for i in 0..max_per_source {
        for items in grouped.values() {
            if i < items.len() {
                interleaved.push(items[i].clone());
            }
        }
    }
    interleaved.truncate(20);

    // Update cache
    {
        let mut cache = lock_cache(news_cache())?;
        *cache = Some(CacheEntry::new(interleaved.clone()));
    }

    Ok(interleaved)
}

// ─── 5. System Stats ────────────────────────────────────────────

#[tauri::command]
pub async fn monitor_system_stats() -> Result<SysStats, String> {
    use sysinfo::System;

    let mut sys = System::new();
    sys.refresh_memory();

    let total_memory_mb = sys.total_memory() / 1_048_576;
    let used_memory_mb = sys.used_memory() / 1_048_576;

    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let hostname = System::host_name().unwrap_or_else(|| "unknown".into());
    let uptime = System::uptime();

    let os_info = System::long_os_version()
        .or_else(|| System::os_version())
        .unwrap_or_else(|| std::env::consts::OS.to_string());

    let local_ip = get_local_ip();

    Ok(SysStats {
        os: os_info,
        hostname,
        uptime_secs: uptime,
        cpu_count,
        memory_total_mb: total_memory_mb,
        memory_used_mb: used_memory_mb,
        local_ip,
        public_ip: None, // fetched separately via monitor_public_ip
    })
}

// ─── 6. Public IP ───────────────────────────────────────────────

#[tauri::command]
pub async fn monitor_public_ip() -> Result<String, String> {
    // Check cache (TTL: 5 minutes)
    {
        let cache = lock_cache(ip_cache())?;
        if let Some(ref entry) = *cache {
            if entry.is_fresh(Duration::from_secs(300)) {
                return Ok(entry.data.clone());
            }
        }
    }

    let client = http_client(8)?;

    // Try multiple services in order
    let services = [
        "https://api.ipify.org",
        "https://checkip.amazonaws.com",
        "https://ifconfig.me/ip",
        "https://icanhazip.com",
    ];

    let mut last_err = String::from("all IP services failed");

    for url in &services {
        match client.get(*url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(text) = resp.text().await {
                    let ip = text.trim().to_string();
                    if !ip.is_empty() && ip.len() < 50 {
                        // Update cache
                        let mut cache = lock_cache(ip_cache())?;
                        *cache = Some(CacheEntry::new(ip.clone()));
                        return Ok(ip);
                    }
                }
            }
            Ok(resp) => {
                last_err = format!("{} returned {}", url, resp.status());
            }
            Err(e) => {
                last_err = format!("{}: {}", url, e);
            }
        }
    }

    // Return cached even if stale
    {
        let cache = lock_cache(ip_cache())?;
        if let Some(ref entry) = *cache {
            return Ok(entry.data.clone());
        }
    }

    Err(last_err)
}

// ─── 7. Activity ────────────────────────────────────────────────

#[tauri::command]
pub async fn monitor_activity() -> Result<Vec<Activity>, String> {
    Ok(generate_activity())
}

// ─── 8. Weather ─────────────────────────────────────────────────

const WEATHER_CITIES: &[(&str, &str, f64, f64)] = &[
    ("New York", "US", 40.71, -74.01),
    ("London", "UK", 51.51, -0.13),
    ("Tokyo", "JP", 35.68, 139.69),
    ("Sydney", "AU", -33.87, 151.21),
    ("Dubai", "AE", 25.20, 55.27),
    ("Mumbai", "IN", 19.08, 72.88),
    ("São Paulo", "BR", -23.55, -46.63),
    ("Paris", "FR", 48.86, 2.35),
    ("Berlin", "DE", 52.52, 13.41),
    ("Moscow", "RU", 55.76, 37.62),
    ("Beijing", "CN", 39.90, 116.40),
    ("Singapore", "SG", 1.35, 103.82),
    ("Cairo", "EG", 30.04, 31.24),
    ("Lagos", "NG", 6.52, 3.38),
    ("Toronto", "CA", 43.65, -79.38),
    ("Mexico City", "MX", 19.43, -99.13),
    ("Seoul", "KR", 37.57, 126.98),
    ("Istanbul", "TR", 41.01, 28.98),
    ("Bangkok", "TH", 13.76, 100.50),
    ("Nairobi", "KE", -1.29, 36.82),
    ("Buenos Aires", "AR", -34.60, -58.38),
    ("Johannesburg", "ZA", -26.20, 28.04),
    ("Jakarta", "ID", -6.21, 106.85),
    ("Hong Kong", "HK", 22.32, 114.17),
    ("Los Angeles", "US", 34.05, -118.24),
];

fn wmo_to_description(code: u16) -> (&'static str, &'static str) {
    match code {
        0 => ("Clear sky", "☀️"),
        1 => ("Mainly clear", "🌤️"),
        2 => ("Partly cloudy", "⛅"),
        3 => ("Overcast", "☁️"),
        45 | 48 => ("Fog", "🌫️"),
        51 | 53 | 55 => ("Drizzle", "🌦️"),
        56 | 57 => ("Freezing drizzle", "🌧️"),
        61 | 63 | 65 => ("Rain", "🌧️"),
        66 | 67 => ("Freezing rain", "🌧️"),
        71 | 73 | 75 => ("Snowfall", "❄️"),
        77 => ("Snow grains", "❄️"),
        80 | 81 | 82 => ("Rain showers", "🌦️"),
        85 | 86 => ("Snow showers", "🌨️"),
        95 => ("Thunderstorm", "⛈️"),
        96 | 99 => ("Thunderstorm with hail", "⛈️"),
        _ => ("Unknown", "🌡️"),
    }
}

#[tauri::command]
pub async fn monitor_weather() -> Result<Vec<WeatherPoint>, String> {
    // Check cache (TTL: 10 minutes)
    {
        let cache = lock_cache(weather_cache())?;
        if let Some(ref entry) = *cache {
            if entry.is_fresh(Duration::from_secs(600)) {
                return Ok(entry.data.clone());
            }
        }
    }

    let client = http_client(15)?;

    // Build comma-separated lat/lng lists for batch request
    let lats: Vec<String> = WEATHER_CITIES.iter().map(|c| c.2.to_string()).collect();
    let lngs: Vec<String> = WEATHER_CITIES.iter().map(|c| c.3.to_string()).collect();

    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m",
        lats.join(","),
        lngs.join(",")
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("weather fetch: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("weather API returned {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("weather parse: {}", e))?;

    // Open-Meteo returns an array of objects when multiple coordinates are queried
    let results: Vec<WeatherPoint> = if let Some(arr) = body.as_array() {
        arr.iter()
            .enumerate()
            .filter_map(|(i, item)| {
                let current = item.get("current")?;
                let temp = current.get("temperature_2m")?.as_f64()?;
                let humidity = current.get("relative_humidity_2m")?.as_u64()? as u16;
                let wind = current.get("wind_speed_10m")?.as_f64()?;
                let code = current.get("weather_code")?.as_u64()? as u16;
                let (desc, icon) = wmo_to_description(code);
                let city = WEATHER_CITIES.get(i)?;
                Some(WeatherPoint {
                    city: city.0.to_string(),
                    country: city.1.to_string(),
                    lat: city.2,
                    lng: city.3,
                    temperature: temp,
                    humidity,
                    wind_speed: wind,
                    weather_code: code,
                    description: desc.to_string(),
                    icon: icon.to_string(),
                })
            })
            .collect()
    } else if body.get("current").is_some() {
        // Single location fallback (shouldn't happen with 25 cities, but handle gracefully)
        let current = body.get("current").unwrap();
        let temp = current.get("temperature_2m").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let humidity = current.get("relative_humidity_2m").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
        let wind = current.get("wind_speed_10m").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let code = current.get("weather_code").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
        let (desc, icon) = wmo_to_description(code);
        let city = WEATHER_CITIES[0];
        vec![WeatherPoint {
            city: city.0.to_string(),
            country: city.1.to_string(),
            lat: city.2,
            lng: city.3,
            temperature: temp,
            humidity,
            wind_speed: wind,
            weather_code: code,
            description: desc.to_string(),
            icon: icon.to_string(),
        }]
    } else {
        return Err("unexpected weather API response format".to_string());
    };

    // Store in cache
    {
        let mut cache = lock_cache(weather_cache())?;
        *cache = Some(CacheEntry::new(results.clone()));
    }

    Ok(results)
}

// ─── 9. Earthquakes ─────────────────────────────────────────────

#[tauri::command]
pub async fn monitor_quakes() -> Result<Vec<QuakeEvent>, String> {
    // Check cache (TTL: 5 minutes)
    {
        let cache = lock_cache(quake_cache())?;
        if let Some(ref entry) = *cache {
            if entry.is_fresh(Duration::from_secs(300)) {
                return Ok(entry.data.clone());
            }
        }
    }

    let client = http_client(15)?;

    let resp = client
        .get("https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/2.5_day.geojson")
        .send()
        .await
        .map_err(|e| format!("quake fetch: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("USGS API returned {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("quake parse: {}", e))?;

    let features = body
        .get("features")
        .and_then(|f| f.as_array())
        .ok_or_else(|| "no features in USGS response".to_string())?;

    let mut quakes: Vec<QuakeEvent> = features
        .iter()
        .filter_map(|f| {
            let props = f.get("properties")?;
            let geom = f.get("geometry")?;
            let coords = geom.get("coordinates")?.as_array()?;

            let lng = coords.first()?.as_f64()?;
            let lat = coords.get(1)?.as_f64()?;
            let depth = coords.get(2)?.as_f64().unwrap_or(0.0);

            let mag = props.get("mag")?.as_f64()?;
            let place = props
                .get("place")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let time = props.get("time")?.as_u64()?;
            let url = props
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let tsunami = props
                .get("tsunami")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 0;
            let id = f
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            Some(QuakeEvent {
                id,
                place,
                lat,
                lng,
                magnitude: mag,
                depth,
                time,
                url,
                tsunami,
            })
        })
        .collect();

    // Sort by magnitude descending
    quakes.sort_by(|a, b| b.magnitude.partial_cmp(&a.magnitude).unwrap_or(std::cmp::Ordering::Equal));

    // Store in cache
    {
        let mut cache = lock_cache(quake_cache())?;
        *cache = Some(CacheEntry::new(quakes.clone()));
    }

    Ok(quakes)
}

// ─── 10. Crypto Prices ──────────────────────────────────────────

#[tauri::command]
pub async fn monitor_crypto() -> Result<Vec<CryptoPrice>, String> {
    // Check cache (TTL: 2 minutes)
    {
        let cache = lock_cache(crypto_cache())?;
        if let Some(ref entry) = *cache {
            if entry.is_fresh(Duration::from_secs(120)) {
                return Ok(entry.data.clone());
            }
        }
    }

    let client = http_client(10)?;

    let url = "https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&ids=bitcoin,ethereum,solana,dogecoin,cardano,ripple,polkadot,chainlink&order=market_cap_desc&per_page=8&page=1&sparkline=false&price_change_percentage=24h";

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("crypto fetch: {}", e))?;

    if !resp.status().is_success() {
        // Return cached even if stale on API failure
        let cache = lock_cache(crypto_cache())?;
        if let Some(ref entry) = *cache {
            return Ok(entry.data.clone());
        }
        return Err(format!("CoinGecko API returned {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("crypto parse: {}", e))?;

    let coins = body
        .as_array()
        .ok_or_else(|| "unexpected crypto API response".to_string())?;

    let prices: Vec<CryptoPrice> = coins
        .iter()
        .filter_map(|c| {
            Some(CryptoPrice {
                id: c.get("id")?.as_str()?.to_string(),
                symbol: c.get("symbol")?.as_str()?.to_uppercase(),
                name: c.get("name")?.as_str()?.to_string(),
                price: c.get("current_price")?.as_f64()?,
                change_24h: c
                    .get("price_change_percentage_24h")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                market_cap: c.get("market_cap").and_then(|v| v.as_f64()).unwrap_or(0.0),
                volume_24h: c
                    .get("total_volume")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            })
        })
        .collect();

    // Store in cache
    {
        let mut cache = lock_cache(crypto_cache())?;
        *cache = Some(CacheEntry::new(prices.clone()));
    }

    Ok(prices)
}