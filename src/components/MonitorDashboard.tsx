// ═══════════════════════════════════════════════════════════════════
//  FLUX CYBER COMMAND v2 — Global Intelligence Monitoring System
//  Modes: INTEL · CYBER · SAT · FLIGHTS · CAMS
//  3D Globe · Live Satellites · Real-Time Flights · Webcams
// ═══════════════════════════════════════════════════════════════════

import {
  createSignal,
  createEffect,
  onMount,
  onCleanup,
  Show,
  For,
  batch,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import {
  twoline2satrec,
  propagate,
  gstime,
  eciToGeodetic,
  degreesLat,
  degreesLong,
} from "satellite.js";

/* ═══════════════════════════════════════════════════════════════
   TYPES
   ═══════════════════════════════════════════════════════════════ */

interface ISSPos {
  latitude: number;
  longitude: number;
  altitude: number;
  velocity: number;
}
interface NewsItem {
  title: string;
  source: string;
  timestamp: string;
  url: string;
}
interface Activity {
  lat: number;
  lon: number;
  label: string;
  event_type: string;
  intensity: number;
}
interface SysStats {
  os: string;
  hostname: string;
  uptime_secs: number;
  cpu_count: number;
  memory_total_mb: number;
  memory_used_mb: number;
  local_ip: string;
  public_ip: string | null;
}
interface SatTLE {
  name: string;
  line1: string;
  line2: string;
  group: string;
}
interface SatPos {
  lat: number;
  lng: number;
  alt: number;
  altKm: number;
  name: string;
  group: string;
  velocity?: number;
}
interface FlightInfo {
  icao24: string;
  callsign: string;
  origin_country: string;
  latitude: number;
  longitude: number;
  altitude: number;
  velocity: number;
  heading: number;
  on_ground: boolean;
  vertical_rate: number;
}
interface WebcamInfo {
  id: string;
  city: string;
  country: string;
  lat: number;
  lng: number;
  label: string;
  url: string;
}
interface ThreatEvent {
  id: string;
  src_ip: string;
  target: string;
  attack_type: string;
  severity: "low" | "medium" | "high" | "critical";
  time: string;
}
interface Props {
  onClose: () => void;
}

type DashboardMode = "INTEL" | "CYBER" | "SAT" | "FLIGHTS" | "CAMS";

/* ═══════════════════════════════════════════════════════════════
   CONSTANTS
   ═══════════════════════════════════════════════════════════════ */

const MODE_CONFIG: Record<
  DashboardMode,
  { icon: string; label: string; key: string; color: string }
> = {
  INTEL: { icon: "📡", label: "INTEL", key: "1", color: "#00ff41" },
  CYBER: { icon: "🔒", label: "CYBER", key: "2", color: "#ff4444" },
  SAT: { icon: "🛰", label: "SAT", key: "3", color: "#4488ff" },
  FLIGHTS: { icon: "🛫", label: "FLIGHTS", key: "4", color: "#00d4ff" },
  CAMS: { icon: "📷", label: "CAMS", key: "5", color: "#ffaa44" },
};

const SAT_GROUPS = [
  { id: "stations", label: "STATIONS", desc: "ISS, Tiangong, etc." },
  { id: "starlink", label: "STARLINK", desc: "SpaceX constellation" },
  { id: "gps", label: "GPS", desc: "Navigation satellites" },
  { id: "weather", label: "WEATHER", desc: "Meteorological sats" },
  { id: "oneweb", label: "ONEWEB", desc: "OneWeb constellation" },
  { id: "iridium", label: "IRIDIUM", desc: "Iridium NEXT" },
  { id: "geo", label: "GEO", desc: "Geostationary orbit" },
  { id: "science", label: "SCIENCE", desc: "Scientific missions" },
];

const LIVE_STREAMS = [
  {
    id: "aje",
    label: "AL JAZEERA",
    tag: "AJE",
    url: "https://www.youtube-nocookie.com/embed/gCNeDWCI0vo",
    accent: "#f4a100",
  },
  {
    id: "firstpost",
    label: "First Post",
    tag: "First Post",
    url: "https://www.youtube-nocookie.com/embed/QoMZg-vPesE",
    accent: "#0055a4",
  },
  {
    id: "dw",
    label: "DW NEWS",
    tag: "DW",
    url: "https://www.youtube-nocookie.com/embed/LuKwFajn37U",
    accent: "#e3001b",
  },
  {
    id: "wion",
    label: "WION",
    tag: "WION",
    url: "https://www.youtube-nocookie.com/embed/vfszY1JYbMc",
    accent: "#e3001b",
  },
];

// Public webcams — Update video IDs if streams go offline
const WEBCAMS: WebcamInfo[] = [
  { id: "nyc-ts", city: "New York", country: "US", lat: 40.758, lng: -73.985, label: "Times Square", url: "https://www.youtube-nocookie.com/embed/rnXIjl_Rzy4" },
  { id: "tokyo-shibuya", city: "Tokyo", country: "JP", lat: 35.659, lng: 139.700, label: "Shibuya Crossing", url: "https://www.youtube-nocookie.com/embed/3dfVK7ld38Ys" },
  { id: "london-eye", city: "London", country: "UK", lat: 51.503, lng: -0.119, label: "London Eye", url: "https://www.youtube-nocookie.com/embed/5XQZt2r8n9o" },
  { id: "sydney-opera", city: "Sydney", country: "AU", lat: -33.856, lng: 151.215, label: "Sydney Opera House", url: "https://www.youtube-nocookie.com/embed/1aXqj8ZyHkA" },
  { id: "paris-eiffel", city: "Paris", country: "FR", lat: 48.858, lng: 2.294, label: "Eiffel Tower", url: "https://www.youtube-nocookie.com/embed/2XjvQZyHkA" },
  
];

const GLOBE_CITIES = [
  { lat: 40.71, lng: -74.0, name: "New York", size: 0.3 },
  { lat: 51.51, lng: -0.13, name: "London", size: 0.3 },
  { lat: 35.68, lng: 139.69, name: "Tokyo", size: 0.3 },
  { lat: 22.32, lng: 114.17, name: "Hong Kong", size: 0.25 },
  { lat: 37.77, lng: -122.42, name: "San Francisco", size: 0.22 },
  { lat: -33.87, lng: 151.21, name: "Sydney", size: 0.25 },
  { lat: 55.76, lng: 37.62, name: "Moscow", size: 0.25 },
  { lat: 1.35, lng: 103.82, name: "Singapore", size: 0.22 },
  { lat: 48.86, lng: 2.35, name: "Paris", size: 0.25 },
  { lat: 52.52, lng: 13.41, name: "Berlin", size: 0.22 },
  { lat: 19.08, lng: 72.88, name: "Mumbai", size: 0.25 },
  { lat: -23.55, lng: -46.63, name: "São Paulo", size: 0.25 },
  { lat: 39.9, lng: 116.4, name: "Beijing", size: 0.3 },
  { lat: 37.57, lng: 126.98, name: "Seoul", size: 0.25 },
  { lat: 25.2, lng: 55.27, name: "Dubai", size: 0.22 },
  { lat: 30.04, lng: 31.24, name: "Cairo", size: 0.22 },
  { lat: 33.94, lng: -118.24, name: "Los Angeles", size: 0.25 },
  { lat: 28.61, lng: 77.21, name: "Delhi", size: 0.25 },
  { lat: 31.23, lng: 121.47, name: "Shanghai", size: 0.28 },
  { lat: 13.76, lng: 100.5, name: "Bangkok", size: 0.22 },
  { lat: -34.6, lng: -58.38, name: "Buenos Aires", size: 0.22 },
  { lat: -6.21, lng: 106.85, name: "Jakarta", size: 0.22 },
  { lat: 41.01, lng: 29.0, name: "Istanbul", size: 0.22 },
  { lat: 38.9, lng: -77.04, name: "Washington DC", size: 0.2 },
  { lat: 59.33, lng: 18.07, name: "Stockholm", size: 0.18 },
  { lat: 60.17, lng: 24.94, name: "Helsinki", size: 0.15 },
  { lat: 64.13, lng: -21.9, name: "Reykjavik", size: 0.14 },
];

const ATTACK_TYPES = [
  "DDoS","SQL_INJ","XSS","BRUTE_FORCE","PORT_SCAN",
  "MALWARE_C2","PHISHING","ZERO_DAY","RANSOMWARE",
  "APT","DNS_TUNNEL","EXFIL","RCE","SSRF",
];

/* ═══════════════════════════════════════════════════════════════
   HELPERS
   ═══════════════════════════════════════════════════════════════ */

function randomIP(): string {
  return `${Math.floor(Math.random() * 223) + 1}.${Math.floor(Math.random() * 256)}.${Math.floor(Math.random() * 256)}.${Math.floor(Math.random() * 256)}`;
}

function generateThreats(count: number): ThreatEvent[] {
  const severities: ThreatEvent["severity"][] = ["low","medium","high","critical"];
  return Array.from({ length: count }, () => ({
    id: Math.random().toString(36).substring(2, 10).toUpperCase(),
    src_ip: randomIP(),
    target: GLOBE_CITIES[Math.floor(Math.random() * GLOBE_CITIES.length)].name,
    attack_type: ATTACK_TYPES[Math.floor(Math.random() * ATTACK_TYPES.length)],
    severity: severities[Math.floor(Math.random() * severities.length)],
    time: new Date().toISOString().slice(11, 19),
  }));
}

function generateArcs(count: number, threatMode = false) {
  return Array.from({ length: count }, () => {
    const from = GLOBE_CITIES[Math.floor(Math.random() * GLOBE_CITIES.length)];
    let to = from;
    while (to === from) to = GLOBE_CITIES[Math.floor(Math.random() * GLOBE_CITIES.length)];
    const isThreat = threatMode ? Math.random() > 0.3 : Math.random() > 0.8;
    return {
      startLat: from.lat, startLng: from.lng,
      endLat: to.lat, endLng: to.lng,
      color: isThreat
        ? ["rgba(255,68,68,0.9)", "rgba(255,68,68,0)"]
        : ["rgba(0,255,65,0.6)", "rgba(0,255,65,0)"],
      stroke: isThreat ? 0.6 : 0.2,
    };
  });
}

/** Propagate a TLE to current time → geodetic coords */
function propagateTLE(
  tle: SatTLE,
  time?: Date
): SatPos | null {
  try {
    const satrec = twoline2satrec(tle.line1, tle.line2);
    const now = time || new Date();
    const result = propagate(satrec, now);
    if (!result || typeof result.position === "boolean" || !result.position) return null;
    const gmst = gstime(now);
    const geo = eciToGeodetic(result.position, gmst);

    const lat = degreesLat(geo.latitude);
    const lng = degreesLong(geo.longitude);
    const altKm = geo.height;

    if (isNaN(lat) || isNaN(lng) || isNaN(altKm)) return null;

    return {
      lat,
      lng,
      alt: Math.min(Math.sqrt(altKm / 6371) * 0.35, 0.6),
      altKm,
      name: tle.name,
      group: tle.group,
    };
  } catch {
    return null;
  }
}

/** Compute one full orbit path for a satellite */
function computeOrbitPath(tle: SatTLE, points = 120): { lat: number; lng: number }[] {
  try {
    const satrec = twoline2satrec(tle.line1, tle.line2);
    const periodMin = (2 * Math.PI) / satrec.no; // minutes
    const path: { lat: number; lng: number }[] = [];
    const now = Date.now();

    for (let i = 0; i < points; i++) {
      const t = (i / points) * periodMin;
      const time = new Date(now + t * 60000);
      const result = propagate(satrec, time);
      if (!result || typeof result.position === "boolean" || !result.position) continue;
      const gmst = gstime(time);
      const geo = eciToGeodetic(result.position, gmst);
      const lat = degreesLat(geo.latitude);
      const lng = degreesLong(geo.longitude);
      if (!isNaN(lat) && !isNaN(lng)) {
        path.push({ lat, lng });
      }
    }
    return path;
  } catch {
    return [];
  }
}

function altitudeColor(altMeters: number): string {
  const altKm = altMeters / 1000;
  if (altKm < 3) return "#00ff41";
  if (altKm < 6) return "#44ff88";
  if (altKm < 9) return "#88ffcc";
  if (altKm < 11) return "#00d4ff";
  return "#4488ff";
}

function severityColor(s: string) {
  switch (s) {
    case "critical": return "#ff0040";
    case "high": return "#ff4444";
    case "medium": return "#ffaa00";
    default: return "#00ff41";
  }
}

/* ═══════════════════════════════════════════════════════════════
   COMPONENT
   ═══════════════════════════════════════════════════════════════ */

export default function MonitorDashboard(props: Props) {
  let globeContainerRef!: HTMLDivElement;
  let globeInstance: any = null;

  // ─── Signals: Core ─────────────────────
  const [mode, setMode] = createSignal<DashboardMode>("INTEL");
  const [utc, setUtc] = createSignal("");
  const [tickerOffset, setTickerOffset] = createSignal(0);
  const [packetCount, setPacketCount] = createSignal(0);
  const [globeReady, setGlobeReady] = createSignal(false);
  const [streamMuted, setStreamMuted] = createSignal(true);
  const [activeStream, setActiveStream] = createSignal(0);
  const [showStream, setShowStream] = createSignal(false);
  const [showModeMenu, setShowModeMenu] = createSignal(false);

  // ─── Signals: Data ────────────────────
  const [iss, setISS] = createSignal<ISSPos | null>(null);
  const [news, setNews] = createSignal<NewsItem[]>([]);
  const [stats, setStats] = createSignal<SysStats | null>(null);
  const [activity, setActivity] = createSignal<Activity[]>([]);
  const [publicIp, setPublicIp] = createSignal("...");
  const [threats, setThreats] = createSignal<ThreatEvent[]>([]);

  // ─── Signals: Satellites ──────────────
  const [satGroup, setSatGroup] = createSignal("stations");
  const [satTLEs, setSatTLEs] = createSignal<SatTLE[]>([]);
  const [satPositions, setSatPositions] = createSignal<SatPos[]>([]);
  const [satLoading, setSatLoading] = createSignal(false);
  const [selectedSat, setSelectedSat] = createSignal<SatPos | null>(null);

  // ─── Signals: Flights ─────────────────
  const [flights, setFlights] = createSignal<FlightInfo[]>([]);
  const [flightLoading, setFlightLoading] = createSignal(false);
  const [selectedFlight, setSelectedFlight] = createSignal<FlightInfo | null>(null);

  // ─── Signals: Webcams ─────────────────
  const [activeWebcam, setActiveWebcam] = createSignal<WebcamInfo | null>(null);

  // ─── Timer refs ───────────────────────
  let satPropTimer: number | undefined;
  let autoRotateResumeTimer: number | undefined;

  /* ═══════════════════════════════════════════════════════════════
     LIFECYCLE
     ═══════════════════════════════════════════════════════════════ */

  onMount(async () => {
    // Clocks
    setUtc(new Date().toISOString().slice(11, 19));
    const clockTimer = setInterval(() => setUtc(new Date().toISOString().slice(11, 19)), 1000);
    const tickerTimer = setInterval(() => setTickerOffset((o) => o + 1), 40);
    const packetTimer = setInterval(() => setPacketCount((c) => c + Math.floor(Math.random() * 120 + 10)), 100);

    // Threats
    setThreats(generateThreats(8));
    const threatTimer = setInterval(() => {
      setThreats((prev) => [...generateThreats(1), ...prev].slice(0, 14));
    }, 5000);

    // Data
    fetchCoreData();
    const dataTimer = setInterval(fetchCoreData, 20000);
    const issTimer = setInterval(fetchISS, 5000);

    // Globe
    try {
      const { default: Globe } = await import("globe.gl");
      initGlobe(Globe);
    } catch (err) {
      console.error("[FLUX] Globe init failed:", err);
    }

    // Keyboard
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (showModeMenu()) { setShowModeMenu(false); return; }
        props.onClose();
      }
      if (e.key === "1") { switchMode("INTEL"); setShowModeMenu(false); }
      if (e.key === "2") { switchMode("CYBER"); setShowModeMenu(false); }
      if (e.key === "3") { switchMode("SAT"); setShowModeMenu(false); }
      if (e.key === "4") { switchMode("FLIGHTS"); setShowModeMenu(false); }
      if (e.key === "5") { switchMode("CAMS"); setShowModeMenu(false); }
      if (e.key === "m" || e.key === "M") setStreamMuted((m) => !m);
    };
    window.addEventListener("keydown", onKey);

    onCleanup(() => {
      clearInterval(clockTimer);
      clearInterval(tickerTimer);
      clearInterval(packetTimer);
      clearInterval(threatTimer);
      clearInterval(dataTimer);
      clearInterval(issTimer);
      if (satPropTimer) clearInterval(satPropTimer);
      clearTimeout(autoRotateResumeTimer);
      window.removeEventListener("keydown", onKey);
      destroyGlobe();
    });
  });

  /* ═══════════════════════════════════════════════════════════════
     DATA FETCHING
     ═══════════════════════════════════════════════════════════════ */

  async function fetchCoreData() {
    fetchISS();
    try { setNews(await invoke("monitor_news")); } catch {}
    try { setStats(await invoke("monitor_system_stats")); } catch {}
    try { setActivity(await invoke("monitor_activity")); } catch {}
    try { setPublicIp(await invoke("monitor_public_ip")); } catch {}
  }

  async function fetchISS() {
    try { setISS(await invoke("monitor_iss_position")); } catch {}
  }

  async function fetchSatellites(group: string) {
    setSatLoading(true);
    try {
      const tles: SatTLE[] = await invoke("monitor_fetch_tle", { group });
      setSatTLEs(tles);
      propagateAllSats(tles);
    } catch (err) {
      console.error("[SAT] Fetch failed:", err);
    }
    setSatLoading(false);
  }

  function propagateAllSats(tles?: SatTLE[]) {
    const data = tles || satTLEs();
    const positions = data
      .map((tle) => propagateTLE(tle))
      .filter(Boolean) as SatPos[];
    setSatPositions(positions);
  }

  async function fetchFlights() {
    setFlightLoading(true);
    try {
      const data: FlightInfo[] = await invoke("monitor_flights");
      setFlights(data);
    } catch (err) {
      console.error("[FLIGHTS] Fetch failed:", err);
    }
    setFlightLoading(false);
  }

    /* ═══════════════════════════════════════════════════════════════
     GLOBE INTERACTION — Pause spin on touch, resume after idle
     ═══════════════════════════════════════════════════════════════ */

  function pauseGlobeRotation() {
    if (!globeInstance) return;
    globeInstance.controls().autoRotate = false;
    clearTimeout(autoRotateResumeTimer);
    autoRotateResumeTimer = window.setTimeout(() => {
      if (globeInstance) {
        globeInstance.controls().autoRotate = true;
        globeInstance.controls().autoRotateSpeed = 0.2;
      }
    }, 15000); // resume after 15s idle
  }

    /* ═══════════════════════════════════════════════════════════════
     SATELLITE GROUP → ICON + COLOR MAPPING
     ═══════════════════════════════════════════════════════════════ */

  const SAT_ICON_MAP: Record<string, { icon: string; color: string; label: string }> = {
    stations:  { icon: "🛰", color: "#ff4466", label: "Station" },
    starlink:  { icon: "⛓",  color: "#88aaff", label: "Starlink" },
    gps:       { icon: "📍", color: "#ffcc44", label: "GPS" },
    weather:   { icon: "🌤", color: "#44ddaa", label: "Weather" },
    oneweb:    { icon: "◈",  color: "#aa88ff", label: "OneWeb" },
    iridium:   { icon: "✦",  color: "#44ccff", label: "Iridium" },
    geo:       { icon: "⊛",  color: "#ff8844", label: "GEO" },
    science:   { icon: "🔭", color: "#ff66aa", label: "Science" },
  };

  function getSatVisual(name: string, group: string) {
    const upper = name.toUpperCase();

    // Special named satellites override group defaults
    if (upper.includes("ISS") || upper.includes("ZARYA") || upper.includes("UNITY"))
      return { icon: "🛰", color: "#ff4466", glow: true, size: "lg" };
    if (upper.includes("TIANGONG"))
      return { icon: "🛰", color: "#ff8844", glow: true, size: "lg" };
    if (upper.includes("HUBBLE"))
      return { icon: "🔭", color: "#cc66ff", glow: true, size: "lg" };
    if (upper.includes("JAMES WEBB") || upper.includes("JWST"))
      return { icon: "🔭", color: "#ffaa33", glow: true, size: "lg" };
    if (upper.includes("GOES"))
      return { icon: "🌤", color: "#44ddaa", glow: true, size: "md" };
    if (upper.includes("NOAA"))
      return { icon: "🌤", color: "#33bbaa", glow: true, size: "md" };
    if (upper.includes("LANDSAT") || upper.includes("TERRA") || upper.includes("AQUA"))
      return { icon: "🌍", color: "#44aaff", glow: true, size: "md" };
    if (upper.includes("TDRS"))
      return { icon: "📡", color: "#88aaff", glow: false, size: "md" };
    if (upper.includes("GPS"))
      return { icon: "📍", color: "#ffcc44", glow: false, size: "sm" };
    if (upper.includes("COSMOS") || upper.includes("GLONASS"))
      return { icon: "📍", color: "#ffaa66", glow: false, size: "sm" };
    if (upper.includes("GALILEO"))
      return { icon: "📍", color: "#66bbff", glow: false, size: "sm" };
    if (upper.includes("BEIDOU"))
      return { icon: "📍", color: "#ff8866", glow: false, size: "sm" };
    if (upper.includes("STARLINK"))
      return { icon: "⛓", color: "#88aaff", glow: false, size: "sm" };
    if (upper.includes("ONEWEB"))
      return { icon: "◈", color: "#aa88ff", glow: false, size: "sm" };
    if (upper.includes("IRIDIUM"))
      return { icon: "✦", color: "#44ccff", glow: false, size: "sm" };
    if (upper.includes("INTELSAT") || upper.includes("ASTRA"))
      return { icon: "📡", color: "#ff8844", glow: false, size: "sm" };
    if (upper.includes("METEOSAT") || upper.includes("METEOR") || upper.includes("FENGYUN"))
      return { icon: "☁", color: "#55ccaa", glow: false, size: "sm" };
    if (upper.includes("MOLNIYA") || upper.includes("MERIDIAN"))
      return { icon: "⊛", color: "#ff6644", glow: false, size: "sm" };

    // Fallback to group default
    const groupVisual = SAT_ICON_MAP[group] || { icon: "•", color: "#6688aa" };
    return { ...groupVisual, glow: false, size: "sm" as const };
  }

  /* ═══════════════════════════════════════════════════════════════
     MARKER ELEMENT FACTORY
     ═══════════════════════════════════════════════════════════════ */

  function createMarkerEl(d: any): HTMLDivElement {
    const el = document.createElement("div");

    switch (d._type) {

      /* ─── FLIGHT ─── */
      case "flight": {
        el.className = "fcmd-marker fcmd-marker-flight";
        const heading = (d.heading ?? 0) - 90;
        const altFt = d.altitude ? (d.altitude / 0.3048).toFixed(0) : "?";
        const spdKts = d.velocity ? (d.velocity * 1.944).toFixed(0) : "?";
        const color = d._color || "#00d4ff";

        el.innerHTML = `
          <span class="fcmd-marker-plane" style="
            transform: rotate(${heading}deg);
            color: ${color};
            filter: drop-shadow(0 0 4px ${color});
          ">✈</span>
        `;
        el.title = [
          d.callsign || d.icao24 || "Unknown",
          `Country: ${d.origin_country || "?"}`,
          `Alt: ${altFt} ft`,
          `Speed: ${spdKts} kts`,
          `Heading: ${d.heading?.toFixed(0) ?? "?"}°`,
          `V/S: ${d.vertical_rate ? (d.vertical_rate * 196.85).toFixed(0) + " fpm" : "?"}`,
        ].join("\n");
        break;
      }

      /* ─── SATELLITE ─── */
      case "sat": {
        const vis = getSatVisual(d.name || "", d.group || "");
        const sizeClass = vis.size === "lg" ? "lg" : vis.size === "md" ? "md" : "sm";

        el.className = `fcmd-marker fcmd-marker-sat ${sizeClass}${vis.glow ? " glow" : ""}`;
        el.style.setProperty("--sat-color", vis.color);

        if (sizeClass === "sm" && !vis.glow) {
          // Small dot only — no label
          el.innerHTML = `<span class="fcmd-sat-icon-s" style="color:${vis.color}">${vis.icon}</span>`;
        } else {
          // Icon + label
          el.innerHTML = `
            <span class="fcmd-sat-icon-l" style="color:${vis.color}">${vis.icon}</span>
            <span class="fcmd-sat-lbl" style="
              color: ${vis.color};
              border-color: ${vis.color}33;
              background: ${vis.color}11;
            ">${d.name || "?"}</span>
          `;
        }

        el.title = [
          d.name || "Unknown Satellite",
          `Group: ${(d.group || "?").toUpperCase()}`,
          `Alt: ${d.altKm ? d.altKm.toFixed(0) + " km" : "?"}`,
          `Lat: ${d.lat?.toFixed(3) ?? "?"}°`,
          `Lon: ${d.lng?.toFixed(3) ?? "?"}°`,
        ].join("\n");
        break;
      }

      /* ─── WEBCAM ─── */
      case "cam": {
        el.className = "fcmd-marker fcmd-marker-cam";
        el.innerHTML = `
          <span class="fcmd-cam-pin">📷</span>
          <span class="fcmd-cam-lbl">${d.city || d.label || ""}</span>
        `;
        el.title = `${d.city || ""} — ${d.label || ""}\n${d.country || ""}`;
        break;
      }

      /* ─── ISS BEACON (INTEL mode) ─── */
      case "iss": {
        el.className = "fcmd-marker fcmd-marker-iss";
        el.innerHTML = `
          <span class="fcmd-iss-ring"></span>
          <span class="fcmd-iss-icon">🛰</span>
          <span class="fcmd-iss-tag">ISS</span>
        `;
        el.title = "International Space Station";
        break;
      }

      default: {
        el.className = "fcmd-marker";
        el.innerHTML = "•";
      }
    }

    // Click handler — pause rotation + select
    el.addEventListener("click", (e) => {
      e.stopPropagation();
      e.preventDefault();
      pauseGlobeRotation();
      handleMarkerClick(d);
    });

    return el;
  }

  function handleMarkerClick(d: any) {
    const m = mode();
    if (m === "SAT" && d._type === "sat") {
      setSelectedSat(d);
      if (globeInstance) {
        globeInstance.pointOfView(
          { lat: d.lat, lng: d.lng, altitude: 1.5 },
          800
        );
      }
    } else if (m === "FLIGHTS" && d._type === "flight") {
      setSelectedFlight(d);
      if (globeInstance) {
        globeInstance.pointOfView(
          { lat: d.latitude ?? d.lat, lng: d.longitude ?? d.lng, altitude: 1.2 },
          800
        );
      }
    } else if (m === "CAMS" && d._type === "cam") {
      setActiveWebcam(d);
      if (globeInstance) {
        globeInstance.pointOfView(
          { lat: d.lat, lng: d.lng, altitude: 1.0 },
          800
        );
      }
    }
  }

  /* ═══════════════════════════════════════════════════════════════
     GLOBE INIT
     ═══════════════════════════════════════════════════════════════ */

    function initGlobe(Globe: any) {
    if (!globeContainerRef) return;
    const w = globeContainerRef.clientWidth;
    const h = globeContainerRef.clientHeight;
    if (w === 0 || h === 0) return;

    globeInstance = Globe()
      .globeImageUrl(
        "//unpkg.com/three-globe/example/img/earth-night.jpg"
      )
      .bumpImageUrl(
        "//unpkg.com/three-globe/example/img/earth-topology.png"
      )
      .backgroundColor("rgba(0,0,0,0)")
      .showGlobe(true)
      .showAtmosphere(true)
      .atmosphereColor("#00c8ff")
      .atmosphereAltitude(0.15)

      // — Point layer (cities / bulk sat dots) —
      .pointsData(GLOBE_CITIES)
      .pointLat("lat")
      .pointLng("lng")
      .pointColor((d: any) => d.color || "#00c8ff")
      .pointAltitude((d: any) => d.alt || 0.01)
      .pointRadius((d: any) => d.size || d.radius || 0.15)
      .pointLabel(
        (d: any) =>
          `<div class="fcmd-globe-tooltip">${d.name || d.label || ""}</div>`
      )
      .onPointClick((p: any) => {
        pauseGlobeRotation();
        handleMarkerClick(p);
      })

      // — Arcs —
      .arcsData(generateArcs(12))
      .arcStartLat("startLat")
      .arcStartLng("startLng")
      .arcEndLat("endLat")
      .arcEndLng("endLng")
      .arcColor("color")
      .arcDashLength(() => 0.4)
      .arcDashGap(() => 0.15)
      .arcDashAnimateTime(() => 1200 + Math.random() * 2500)
      .arcStroke("stroke")

      // — Rings —
      .ringsData([])
      .ringLat("lat")
      .ringLng("lng")
      .ringColor(
        () => (t: number) => `rgba(0,200,255,${1 - t})`
      )
      .ringMaxRadius(3)
      .ringPropagationSpeed(2)
      .ringRepeatPeriod(1200)

      // — Labels —
      .labelsData(
        GLOBE_CITIES.filter((c) => c.size >= 0.25).map(
          (c) => ({
            lat: c.lat,
            lng: c.lng,
            text: c.name.toUpperCase(),
            size: c.size,
          })
        )
      )
      .labelLat("lat")
      .labelLng("lng")
      .labelText("text")
      .labelColor(() => "rgba(140,200,255,0.35)")
      .labelSize((d: any) => d.size * 1.6)
      .labelDotRadius((d: any) => d.size * 0.5)
      .labelDotOrientation(() => "right" as any)
      .labelResolution(2)

      // — Paths (orbit tracks) —
      .pathsData([])
      .pathPointLat("lat")
      .pathPointLng("lng")
      .pathColor(() => "rgba(255,68,68,0.35)")
      .pathStroke(1.2)
      .pathDashLength(0.01)
      .pathDashGap(0.004)
      .pathDashAnimateTime(100000)

      // ★ HTML ELEMENTS layer — custom DOM markers (flights, sats, cams)
      .htmlElementsData([])
      .htmlLat((d: any) => d.lat ?? d.latitude ?? 0)
      .htmlLng((d: any) => d.lng ?? d.longitude ?? 0)
      .htmlAltitude((d: any) => d._htmlAlt ?? 0)
      .htmlElement((d: any) => createMarkerEl(d))

      // — Dimensions —
      .width(w)
      .height(h)(globeContainerRef);

    // ★ Camera controls — reduced auto-rotate speed
    const controls = globeInstance.controls();
    controls.autoRotate = true;
    controls.autoRotateSpeed = 0.2; // ★ slower
    controls.enableZoom = true;
    controls.minDistance = 140;
    controls.maxDistance = 650;
    controls.enableDamping = true;
    controls.dampingFactor = 0.12;

    // ★ INTERACTION HANDLERS — pause rotation on user input
    const onInteract = () => pauseGlobeRotation();
    globeContainerRef.addEventListener("pointerdown", onInteract);
    globeContainerRef.addEventListener("wheel", onInteract, {
      passive: true,
    });
    globeContainerRef.addEventListener("touchstart", onInteract, {
      passive: true,
    });

    // Initial viewpoint
    globeInstance.pointOfView(
      { lat: 20, lng: 10, altitude: 2.2 },
      1500
    );

    // Resize observer
    const ro = new ResizeObserver((entries) => {
      for (const e of entries) {
        if (globeInstance && e.contentRect.width > 0) {
          globeInstance
            .width(e.contentRect.width)
            .height(e.contentRect.height);
        }
      }
    });
    ro.observe(globeContainerRef);

    // Periodic arc refresh
    const arcsRefresh = setInterval(() => {
      if (
        globeInstance &&
        (mode() === "INTEL" || mode() === "CYBER")
      ) {
        globeInstance.arcsData(
          generateArcs(12, mode() === "CYBER")
        );
      }
    }, 7000);

    setGlobeReady(true);

    (globeContainerRef as any).__cleanup = () => {
      clearInterval(arcsRefresh);
      clearTimeout(autoRotateResumeTimer);
      globeContainerRef.removeEventListener("pointerdown", onInteract);
      globeContainerRef.removeEventListener("wheel", onInteract);
      globeContainerRef.removeEventListener("touchstart", onInteract);
      ro.disconnect();
    };
  }

  function destroyGlobe() {
    if ((globeContainerRef as any)?.__cleanup) (globeContainerRef as any).__cleanup();
    if (globeInstance) {
      try {
        const canvas = globeContainerRef?.querySelector("canvas");
        if (canvas) {
          const gl = canvas.getContext("webgl2") || canvas.getContext("webgl");
          gl?.getExtension("WEBGL_lose_context")?.loseContext();
        }
      } catch {}
      globeInstance = null;
    }
    if (globeContainerRef) {
      while (globeContainerRef.firstChild) globeContainerRef.removeChild(globeContainerRef.firstChild);
    }
  }

  /* ═══════════════════════════════════════════════════════════════
     MODE SWITCHING
     ═══════════════════════════════════════════════════════════════ */

  function switchMode(newMode: DashboardMode) {
    const prev = mode();
    if (prev === newMode) return;
    setMode(newMode);

    // Cleanup previous mode
    if (satPropTimer) { clearInterval(satPropTimer); satPropTimer = undefined; }
    setSelectedSat(null);
    setSelectedFlight(null);
    setActiveWebcam(null);

    if (!globeInstance) return;

    switch (newMode) {
      case "INTEL":
        configureGlobeINTEL();
        break;
      case "CYBER":
        configureGlobeCYBER();
        break;
      case "SAT":
        configureGlobeSAT();
        break;
      case "FLIGHTS":
        configureGlobeFLIGHTS();
        break;
      case "CAMS":
        configureGlobeCAMS();
        break;
    }
  }

  function configureGlobeINTEL() {
    if (!globeInstance) return;
    globeInstance
      .globeImageUrl("//unpkg.com/three-globe/example/img/earth-night.jpg")
      .atmosphereColor("#3388ff")    // ★ blue-white atmosphere
      .pointsData(GLOBE_CITIES)
      .pointColor(() => "#55aaff")   // ★ brighter city dots
      .pointAltitude(() => 0.01)
      .pointRadius((d: any) => d.size || 0.15)
      .arcsData(generateArcs(12))
      .labelsData(
        GLOBE_CITIES.filter((c) => c.size >= 0.22).map((c) => ({
          lat: c.lat, lng: c.lng,
          text: c.name.toUpperCase(),
          size: c.size,
        }))
      )
      .labelColor(() => "rgba(180,210,255,0.55)")   // ★ much brighter labels
      .labelSize((d: any) => (d.size || 0.2) * 2.2) // ★ bigger
      .labelDotRadius((d: any) => (d.size || 0.2) * 0.6)
      .labelDotOrientation(() => "right" as any)
      .pathsData([])
      .ringsData([])
      .htmlElementsData([]);
    globeInstance.controls().autoRotateSpeed = 0.2;
  }

  function configureGlobeCYBER() {
    if (!globeInstance) return;
    globeInstance
      .globeImageUrl("//unpkg.com/three-globe/example/img/earth-night.jpg")
      .atmosphereColor("#ff2244")
      .pointsData(GLOBE_CITIES.map((c) => ({ ...c, color: "#ff5577" })))
      .pointColor((d: any) => d.color || "#ff5577")
      .pointAltitude(() => 0.01)
      .pointRadius((d: any) => d.size || 0.15)
      .arcsData(generateArcs(18, true))
      .labelsData(
        GLOBE_CITIES.filter((c) => c.size >= 0.22).map((c) => ({
          lat: c.lat, lng: c.lng,
          text: c.name.toUpperCase(),
          size: c.size,
        }))
      )
      .labelColor(() => "rgba(255,150,170,0.55)")   // ★ visible red-ish labels
      .labelSize((d: any) => (d.size || 0.2) * 2.2)
      .labelDotRadius((d: any) => (d.size || 0.2) * 0.6)
      .pathsData([])
      .ringsData(
        activity().slice(0, 6).map((a) => ({ lat: a.lat, lng: a.lon }))
      )
      .ringColor(() => (t: number) => `rgba(255,68,68,${1 - t})`)
      .htmlElementsData([]);
    globeInstance.controls().autoRotateSpeed = 0.1;
  }

  function configureGlobeSAT() {
    if (!globeInstance) return;
    globeInstance
      .globeImageUrl("//unpkg.com/three-globe/example/img/earth-night.jpg")
      .atmosphereColor("#4488ff")
      .arcsData([])
      .labelsData([])
      .pathsData([])
      .ringsData([])
      .htmlElementsData([]);
    globeInstance.controls().autoRotateSpeed = 0.25;
    fetchSatellites(satGroup());
    if (satPropTimer) clearInterval(satPropTimer);
    satPropTimer = window.setInterval(() => propagateAllSats(), 2000);
  }

  function configureGlobeFLIGHTS() {
    if (!globeInstance) return;
    globeInstance
      .globeImageUrl("//unpkg.com/three-globe/example/img/earth-blue-marble.jpg")
      .atmosphereColor("#55bbff")
      .arcsData([])
      .labelsData(
        GLOBE_CITIES.filter((c) => c.size >= 0.25).map((c) => ({
          lat: c.lat, lng: c.lng,
          text: c.name.toUpperCase(),
          size: c.size,
        }))
      )
      .labelColor(() => "rgba(180,220,255,0.4)")  // ★ visible city names on blue marble
      .labelSize((d: any) => (d.size || 0.2) * 2)
      .labelDotRadius(() => 0)
      .pathsData([])
      .ringsData([])
      .pointsData([])
      .htmlElementsData([]);
    globeInstance.controls().autoRotateSpeed = 0.15;
    fetchFlights();
  }

  function configureGlobeCAMS() {
    if (!globeInstance) return;
    const camMarkers = WEBCAMS.map((w) => ({
      ...w,
      _type: "cam" as const,
      _htmlAlt: 0.01,
    }));
    globeInstance
      .globeImageUrl("//unpkg.com/three-globe/example/img/earth-blue-marble.jpg")
      .atmosphereColor("#ff9944")
      .pointsData([])
      .arcsData([])
      .labelsData([])
      .pathsData([])
      .ringsData(WEBCAMS.map((c) => ({ lat: c.lat, lng: c.lng })))
      .ringColor(() => (t: number) => `rgba(255,170,68,${1 - t})`)
      .ringMaxRadius(2)
      .ringRepeatPeriod(2200)
      .htmlElementsData(camMarkers);
    globeInstance.controls().autoRotateSpeed = 0.1;
  }

  /* ═══════════════════════════════════════════════════════════════
     REACTIVE GLOBE UPDATES
     ═══════════════════════════════════════════════════════════════ */

  // ISS on globe (INTEL / CYBER modes)
    // ★ ISS on globe (INTEL / CYBER modes)
  createEffect(() => {
    const m = mode();
    const issData = iss();
    if (!globeInstance || (m !== "INTEL" && m !== "CYBER")) return;

    if (issData) {
      const cityColor = m === "CYBER" ? "#ff4466" : "#00c8ff";
      const issPoint = {
        lat: issData.latitude,
        lng: issData.longitude,
        name: "ISS",
        size: 0.5,
        color: "#ff4444",
        alt: 0.06,
      };
      globeInstance.pointsData([
        ...GLOBE_CITIES.map((c) => ({
          ...c,
          color: cityColor,
          alt: 0.01,
        })),
        issPoint,
      ]);
      globeInstance.ringsData([
        { lat: issData.latitude, lng: issData.longitude },
        ...activity()
          .slice(0, 4)
          .map((a) => ({ lat: a.lat, lng: a.lon })),
      ]);

      // ★ HTML marker for ISS label
      globeInstance.htmlElementsData([
        {
          lat: issData.latitude,
          lng: issData.longitude,
          _type: "iss",
          _htmlAlt: 0.07,
          name: "ISS",
        },
      ]);
    }
  });

  // ★ Satellites on globe — uses htmlElementsData for icons
    // ★ Satellites on globe
  createEffect(() => {
    if (mode() !== "SAT" || !globeInstance) return;
    const positions = satPositions();
    if (!positions.length) return;

    const currentGroup = satGroup();
    const groupVisual = SAT_ICON_MAP[currentGroup] || { color: "#6688aa" };

    // Point layer for bulk dots
    const points = positions.map((s) => {
      const vis = getSatVisual(s.name, s.group);
      return {
        lat: s.lat,
        lng: s.lng,
        alt: s.alt,
        name: s.name,
        altKm: s.altKm,
        group: s.group,
        size: s.name.includes("ISS") ? 0.4 : 0.04,
        radius: s.name.includes("ISS") ? 0.4 : 0.04,
        color: vis.color,
        _type: "sat",
      };
    });
    globeInstance.pointsData(points);
    globeInstance.pointAltitude((d: any) => d.alt || 0.01);
    globeInstance.pointRadius((d: any) => d.radius || 0.04);

    // HTML markers — notable sats get full icon + label
    const notableKeywords = [
      "ISS", "ZARYA", "TIANGONG", "HUBBLE", "JWST", "JAMES WEBB",
      "GOES", "NOAA", "LANDSAT", "TERRA", "AQUA", "TDRS",
      "METEOSAT", "FENGYUN",
    ];

    const notable = positions.filter((s) =>
      notableKeywords.some((kw) => s.name.toUpperCase().includes(kw))
    );

    // Also include first few of each group for visual variety
    const firstFew = positions
      .filter((s) => !notable.find((n) => n.name === s.name))
      .slice(0, Math.min(30, positions.length));

    const selSat = selectedSat();
    const markerData: any[] = [];

    // Notable sats — full markers
    for (const s of notable) {
      markerData.push({
        lat: s.lat,
        lng: s.lng,
        _type: "sat",
        _htmlAlt: s.alt + 0.01,
        name: s.name,
        altKm: s.altKm,
        group: s.group,
        alt: s.alt,
      });
    }

    // Representative sample — small markers
    for (const s of firstFew) {
      markerData.push({
        lat: s.lat,
        lng: s.lng,
        _type: "sat",
        _htmlAlt: s.alt + 0.005,
        name: s.name,
        altKm: s.altKm,
        group: s.group,
        alt: s.alt,
      });
    }

    // Selected satellite always gets a marker
    if (selSat && !markerData.find((m) => m.name === selSat.name)) {
      markerData.push({
        lat: selSat.lat,
        lng: selSat.lng,
        _type: "sat",
        _htmlAlt: selSat.alt + 0.015,
        name: selSat.name,
        altKm: selSat.altKm,
        group: selSat.group,
        alt: selSat.alt,
      });
    }

    globeInstance.htmlElementsData(markerData);
    globeInstance.labelsData([]);

    // Orbit path
    const satForOrbit =
      selSat || positions.find((s) => s.name.toUpperCase().includes("ISS"));
    if (satForOrbit) {
      const tle = satTLEs().find((t) => t.name === satForOrbit.name);
      if (tle) {
        const vis = getSatVisual(satForOrbit.name, satForOrbit.group);
        const orbit = computeOrbitPath(tle, 150);
        globeInstance.pathsData(orbit.length > 2 ? [orbit] : []);
        globeInstance.pathColor(() =>
          `${vis.color}44`
        );
        globeInstance.pathStroke(1.5);
      }
    }
  });

  // ★ Flights on globe — uses htmlElementsData for airplane icons
  createEffect(() => {
    if (mode() !== "FLIGHTS" || !globeInstance) return;
    const f = flights();
    if (!f.length) return;

    // ★ Use HTML markers with airplane icons (cap at 200 for perf)
    const flightMarkers = f.slice(0, 200).map((fl) => ({
      lat: fl.latitude,
      lng: fl.longitude,
      _type: "flight" as const,
      _htmlAlt: Math.min(fl.altitude / 600000, 0.15),
      _color: altitudeColor(fl.altitude),
      callsign: fl.callsign,
      icao24: fl.icao24,
      origin_country: fl.origin_country,
      altitude: fl.altitude,
      velocity: fl.velocity,
      heading: fl.heading,
      vertical_rate: fl.vertical_rate,
      latitude: fl.latitude,
      longitude: fl.longitude,
    }));

    globeInstance.htmlElementsData(flightMarkers);

    // Points layer OFF for flights (using HTML markers)
    globeInstance.pointsData([]);
    globeInstance.labelsData([]);

    // Ring on selected
    const sel = selectedFlight();
    if (sel) {
      globeInstance.ringsData([
        { lat: sel.latitude, lng: sel.longitude },
      ]);
      globeInstance.ringColor(
        () => (t: number) => `rgba(0,212,255,${1 - t})`
      );
    } else {
      globeInstance.ringsData([]);
    }
  });

  // When satellite group changes
  createEffect(() => {
    const group = satGroup();
    if (mode() === "SAT") {
      fetchSatellites(group);
    }
  });

  /* ═══════════════════════════════════════════════════════════════
     HELPERS
     ═══════════════════════════════════════════════════════════════ */

  function tzTime(offset: number) {
    const d = new Date();
    d.setMinutes(d.getMinutes() + d.getTimezoneOffset() + offset * 60);
    return d.toTimeString().slice(0, 5);
  }

  function formatUptime(secs: number) {
    const d = Math.floor(secs / 86400);
    const h = Math.floor((secs % 86400) / 3600);
    const m = Math.floor((secs % 3600) / 60);
    return d > 0 ? `${d}d ${h}h ${m}m` : `${h}h ${m}m`;
  }

  function memPercent() {
    const s = stats();
    return s ? Math.round((s.memory_used_mb / s.memory_total_mb) * 100) : 0;
  }

  function threatLevel() {
    const t = threats();
    const c = t.filter((x) => x.severity === "critical").length;
    const h = t.filter((x) => x.severity === "high").length;
    if (c > 1) return { text: "CRITICAL", color: "#ff0040" };
    if (c > 0 || h > 2) return { text: "HIGH", color: "#ff4444" };
    if (h > 0) return { text: "ELEVATED", color: "#ffaa00" };
    return { text: "NORMAL", color: "#00ff41" };
  }

  const tickerText = () => {
    const items = news();
    if (!items.length) return "  ▸▸▸  FLUX CYBER COMMAND — ENCRYPTED CHANNEL ACTIVE — MONITORING ALL SECTORS  ◈  ";
    return items.map((n) => `  ▸ ${n.title.toUpperCase()}  [${n.source}]  `).join("  ◈  ");
  };

  const streamUrl = () => {
    const s = LIVE_STREAMS[activeStream()];
    return `${s.url}?autoplay=1&mute=${streamMuted() ? 1 : 0}&controls=0&modestbranding=1&rel=0`;
  };

  const webcamUrl = () => {
    const cam = activeWebcam();
    if (!cam) return "";
    return `${cam.url}?autoplay=1&mute=1&controls=0&modestbranding=1&rel=0`;
  };

  /* ═══════════════════════════════════════════════════════════════
     JSX
     ═══════════════════════════════════════════════════════════════ */

  return (
    <div class="fcmd-overlay" onClick={() => props.onClose()}>
      <div class="fcmd-dashboard" onClick={(e) => e.stopPropagation()}>
        <div class="fcmd-scanlines" />
        <div class="fcmd-vignette" />

        {/* ═══ TOP BAR ═══════════════════════════════════════════ */}
        <header class="fcmd-topbar">
          <div class="fcmd-topbar-section">
            <span class="fcmd-logo">⚡ FLUX COMMAND</span>
            <span class="fcmd-live-badge"><span class="fcmd-live-dot" />LIVE</span>
            <span class="fcmd-threat-badge" style={{ color: threatLevel().color }}>
              THREAT: {threatLevel().text}
            </span>
          </div>

          <div class="fcmd-topbar-section">
            <div class="fcmd-clocks">
              {[
                { l: "UTC", o: 0 }, { l: "NYC", o: -5 },
                { l: "LON", o: 0 }, { l: "TYO", o: 9 },
              ].map((tz) => (
                <div class="fcmd-clock">
                  <span class="fcmd-clock-label">{tz.l}</span>
                  <span class="fcmd-clock-time">
                    {tz.l === "UTC" ? utc() : tzTime(tz.o)}
                  </span>
                </div>
              ))}
            </div>
            <span class="fcmd-packets">PKT: {packetCount().toLocaleString()}</span>
            <span class="fcmd-close" onClick={() => props.onClose()}>✕</span>
          </div>
        </header>

        {/* ═══ MAIN CONTENT ══════════════════════════════════════ */}
        <main class="fcmd-main">

          {/* ─── LEFT PANEL ─────────────────────────────────────── */}
          <aside class="fcmd-panel-col fcmd-left">

            {/* System Panel — always visible */}
            <section class="fcmd-panel">
              <h3 class="fcmd-panel-hdr">⊞ SYSTEM</h3>
              <Show when={stats()} fallback={<div class="fcmd-dim">Loading...</div>}>
                <div class="fcmd-kv"><span>HOST</span><span>{stats()!.hostname}</span></div>
                <div class="fcmd-kv"><span>OS</span><span>{stats()!.os}</span></div>
                <div class="fcmd-kv"><span>CPU</span><span>{stats()!.cpu_count} cores</span></div>
                <div class="fcmd-kv"><span>RAM</span><span>{stats()!.memory_used_mb}/{stats()!.memory_total_mb} MB</span></div>
                <div class="fcmd-progress-bar">
                  <div class="fcmd-progress-fill" style={{
                    width: `${memPercent()}%`,
                    background: memPercent() > 80 ? "#ff4444" : "#00ff41",
                  }} />
                </div>
                <div class="fcmd-kv"><span>UPTIME</span><span>{formatUptime(stats()!.uptime_secs)}</span></div>
                <div class="fcmd-kv"><span>LAN</span><span class="fcmd-mono">{stats()!.local_ip}</span></div>
                <div class="fcmd-kv"><span>WAN</span><span class="fcmd-mono">{publicIp()}</span></div>
              </Show>
            </section>

            {/* ISS Panel — always visible */}
            <section class="fcmd-panel">
              <h3 class="fcmd-panel-hdr">🛰 ISS TRACKER</h3>
              <Show when={iss()} fallback={<div class="fcmd-dim">Acquiring...</div>}>
                <div class="fcmd-kv"><span>LAT</span><span class="fcmd-val-cyan">{iss()!.latitude.toFixed(4)}°</span></div>
                <div class="fcmd-kv"><span>LON</span><span class="fcmd-val-cyan">{iss()!.longitude.toFixed(4)}°</span></div>
                <div class="fcmd-kv"><span>ALT</span><span>{iss()!.altitude.toFixed(0)} km</span></div>
                <div class="fcmd-kv"><span>VEL</span><span>{iss()!.velocity.toFixed(0)} km/h</span></div>
              </Show>
            </section>

            {/* ── Mode-Specific Left Content ── */}

            {/* INTEL / CYBER: Threat Feed */}
            <Show when={mode() === "INTEL" || mode() === "CYBER"}>
              <section class="fcmd-panel fcmd-panel-grow">
                <h3 class="fcmd-panel-hdr">⚠ THREAT FEED</h3>
                <div class="fcmd-threat-list">
                  <For each={threats().slice(0, 10)}>
                    {(t) => (
                      <div class="fcmd-threat-row">
                        <span class="fcmd-threat-sev" style={{ background: severityColor(t.severity) }} />
                        <span class="fcmd-threat-type">{t.attack_type}</span>
                        <span class="fcmd-threat-detail">
                          <span class="fcmd-dim">{t.src_ip}</span>
                          <span class="fcmd-threat-arrow">→</span>
                          <span>{t.target}</span>
                        </span>
                        <span class="fcmd-threat-time">{t.time}</span>
                      </div>
                    )}
                  </For>
                </div>
              </section>
            </Show>

            {/* SAT: Satellite Groups & Info */}
            <Show when={mode() === "SAT"}>
              <section class="fcmd-panel">
                <h3 class="fcmd-panel-hdr">📡 SAT GROUP</h3>
                <div class="fcmd-sat-groups">
                  <For each={SAT_GROUPS}>
                    {(g) => (
                      <button
                        class={`fcmd-sat-group-btn ${satGroup() === g.id ? "active" : ""}`}
                        onClick={() => setSatGroup(g.id)}
                      >
                        <span>{g.label}</span>
                        <span class="fcmd-dim">{g.desc}</span>
                      </button>
                    )}
                  </For>
                </div>
              </section>
              <section class="fcmd-panel">
                <h3 class="fcmd-panel-hdr">📊 STATUS</h3>
                <Show when={satLoading()}>
                  <div class="fcmd-dim">Fetching TLE data...</div>
                </Show>
                <Show when={!satLoading()}>
                  <div class="fcmd-kv"><span>GROUP</span><span>{satGroup().toUpperCase()}</span></div>
                  <div class="fcmd-kv"><span>TLEs</span><span>{satTLEs().length}</span></div>
                  <div class="fcmd-kv"><span>TRACKED</span><span class="fcmd-val-cyan">{satPositions().length}</span></div>
                  <div class="fcmd-kv"><span>SOURCE</span><span class="fcmd-dim">CelesTrak</span></div>
                </Show>
              </section>
              <Show when={selectedSat()}>
                <section class="fcmd-panel">
                  <h3 class="fcmd-panel-hdr">🎯 SELECTED</h3>
                  <div class="fcmd-kv"><span>NAME</span><span class="fcmd-val-cyan">{selectedSat()!.name}</span></div>
                  <div class="fcmd-kv"><span>LAT</span><span>{selectedSat()!.lat.toFixed(3)}°</span></div>
                  <div class="fcmd-kv"><span>LON</span><span>{selectedSat()!.lng.toFixed(3)}°</span></div>
                  <div class="fcmd-kv"><span>ALT</span><span>{selectedSat()!.altKm.toFixed(0)} km</span></div>
                  <button class="fcmd-btn" onClick={() => setSelectedSat(null)}>DESELECT</button>
                </section>
              </Show>
               <section class="fcmd-panel fcmd-panel-grow">
                <h3 class="fcmd-panel-hdr">🗒 SAT LIST</h3>
                <div class="fcmd-scroll-list">
                  <For each={satPositions().slice(0, 50)}>
                    {(s) => {
                      const vis = getSatVisual(s.name, s.group);
                      return (
                        <div
                          class={`fcmd-list-row ${selectedSat()?.name === s.name ? "selected" : ""}`}
                          onClick={() => {
                            setSelectedSat(s);
                            pauseGlobeRotation();
                            if (globeInstance) {
                              globeInstance.pointOfView(
                                { lat: s.lat, lng: s.lng, altitude: 1.5 },
                                800
                              );
                            }
                          }}
                        >
                          <span class="fcmd-list-icon" style={{ color: vis.color }}>
                            {vis.icon}
                          </span>
                          <span class="fcmd-list-name">{s.name}</span>
                          <span class="fcmd-list-val">{s.altKm.toFixed(0)}km</span>
                        </div>
                      );
                    }}
                  </For>
                </div>
              </section>
            </Show>

            {/* FLIGHTS: Flight Stats & List */}
            <Show when={mode() === "FLIGHTS"}>
              <section class="fcmd-panel">
                <h3 class="fcmd-panel-hdr">📊 FLIGHT DATA</h3>
                <Show when={flightLoading()}>
                  <div class="fcmd-dim">Contacting OpenSky Network...</div>
                </Show>
                <Show when={!flightLoading()}>
                  <div class="fcmd-kv"><span>AIRBORNE</span><span class="fcmd-val-cyan">{flights().length}</span></div>
                  <div class="fcmd-kv"><span>SOURCE</span><span class="fcmd-dim">OpenSky Network</span></div>
                  <div class="fcmd-kv">
                    <span>TOP ORIGIN</span>
                    <span>{(() => {
                      const counts: Record<string, number> = {};
                      flights().forEach((f) => { counts[f.origin_country] = (counts[f.origin_country] || 0) + 1; });
                      return Object.entries(counts).sort((a, b) => b[1] - a[1])?.[0]?.[0] || "—";
                    })()}</span>
                  </div>
                </Show>
                <button class="fcmd-btn" onClick={fetchFlights} style={{ "margin-top": "6px" }}>
                  ↻ REFRESH
                </button>
              </section>
              <Show when={selectedFlight()}>
                <section class="fcmd-panel">
                  <h3 class="fcmd-panel-hdr">🎯 SELECTED</h3>
                  <div class="fcmd-kv"><span>CALL</span><span class="fcmd-val-cyan">{selectedFlight()!.callsign || "N/A"}</span></div>
                  <div class="fcmd-kv"><span>ICAO</span><span>{selectedFlight()!.icao24}</span></div>
                  <div class="fcmd-kv"><span>ORIGIN</span><span>{selectedFlight()!.origin_country}</span></div>
                  <div class="fcmd-kv"><span>ALT</span><span>{(selectedFlight()!.altitude / 0.3048).toFixed(0)} ft</span></div>
                  <div class="fcmd-kv"><span>SPD</span><span>{(selectedFlight()!.velocity * 1.944).toFixed(0)} kts</span></div>
                  <div class="fcmd-kv"><span>HDG</span><span>{selectedFlight()!.heading.toFixed(0)}°</span></div>
                  <div class="fcmd-kv"><span>V/S</span><span>{(selectedFlight()!.vertical_rate * 196.85).toFixed(0)} fpm</span></div>
                  <button class="fcmd-btn" onClick={() => setSelectedFlight(null)}>DESELECT</button>
                </section>
              </Show>
              <section class="fcmd-panel fcmd-panel-grow">
                <h3 class="fcmd-panel-hdr">✈ FLIGHT LIST</h3>
                <div class="fcmd-scroll-list">
                  <For each={flights().slice(0, 80)}>
                    {(f) => (
                      <div
                        class={`fcmd-list-row ${selectedFlight()?.icao24 === f.icao24 ? "selected" : ""}`}
                        onClick={() => {
                          setSelectedFlight(f);
                          if (globeInstance) {
                            globeInstance.pointOfView({ lat: f.latitude, lng: f.longitude, altitude: 1.2 }, 600);
                          }
                        }}
                      >
                        <span class="fcmd-list-dot" style={{ background: altitudeColor(f.altitude) }} />
                        <span class="fcmd-list-name">{f.callsign || f.icao24}</span>
                        <span class="fcmd-list-val">{f.origin_country}</span>
                      </div>
                    )}
                  </For>
                </div>
              </section>
            </Show>

            {/* CAMS: Webcam List */}
            <Show when={mode() === "CAMS"}>
              <section class="fcmd-panel fcmd-panel-grow">
                <h3 class="fcmd-panel-hdr">📷 WEBCAMS ({WEBCAMS.length})</h3>
                <div class="fcmd-scroll-list">
                  <For each={WEBCAMS}>
                    {(cam) => (
                      <div
                        class={`fcmd-list-row ${activeWebcam()?.id === cam.id ? "selected" : ""}`}
                        onClick={() => {
                          setActiveWebcam(cam);
                          if (globeInstance) {
                            globeInstance.pointOfView({ lat: cam.lat, lng: cam.lng, altitude: 1.0 }, 600);
                          }
                        }}
                      >
                        <span class="fcmd-list-dot" style={{ background: "#ffaa44" }} />
                        <span class="fcmd-list-name">{cam.city}</span>
                        <span class="fcmd-list-val">{cam.country}</span>
                      </div>
                    )}
                  </For>
                </div>
              </section>
            </Show>
          </aside>

          {/* ─── CENTER: 3D GLOBE ──────────────────────────────── */}
          <div class="fcmd-globe-wrap" onClick={() => showModeMenu() && setShowModeMenu(false)}>
            <div ref={globeContainerRef!} class="fcmd-globe-container" />

            {/* Loading overlay */}
            <Show when={!globeReady()}>
              <div class="fcmd-globe-loading">
                <div class="fcmd-spinner" />
                <span>INITIALIZING 3D GLOBE...</span>
              </div>
            </Show>

            {/* Globe HUD — top-left */}
            <div class="fcmd-globe-hud fcmd-hud-tl">
              <span>{MODE_CONFIG[mode()].icon} {MODE_CONFIG[mode()].label} MODE</span>
              <Show when={mode() === "SAT"}>
                <span class="fcmd-dim">TRACKING {satPositions().length} OBJECTS</span>
              </Show>
              <Show when={mode() === "FLIGHTS"}>
                <span class="fcmd-dim">{flights().length} AIRCRAFT</span>
              </Show>
              <Show when={mode() === "CAMS"}>
                <span class="fcmd-dim">{WEBCAMS.length} CAMERAS</span>
              </Show>
              <Show when={mode() === "CYBER"}>
                <span class="fcmd-dim" style={{ color: "#ff4444" }}>THREAT MONITORING ACTIVE</span>
              </Show>
            </div>

            {/* Mode selector — bottom-left of globe */}
            <div class="fcmd-mode-selector" onClick={(e) => e.stopPropagation()}>
              <button
                class="fcmd-mode-selector-btn"
                onClick={() => setShowModeMenu((v) => !v)}
                title="Switch mode"
              >
                <span>{MODE_CONFIG[mode()].icon}</span>
              </button>
              <Show when={showModeMenu()}>
                <div class="fcmd-mode-menu">
                  <For each={Object.entries(MODE_CONFIG)}>
                    {([key, cfg]) => (
                      <button
                        class={`fcmd-mode-menu-item ${mode() === key ? "active" : ""}`}
                        onClick={() => {
                          switchMode(key as DashboardMode);
                          setShowModeMenu(false);
                        }}
                        style={{ "--mode-color": cfg.color }}
                      >
                        <span class="fcmd-mode-menu-icon">{cfg.icon}</span>
                        <span class="fcmd-mode-menu-label">{cfg.label}</span>
                        <span class="fcmd-mode-menu-key">{cfg.key}</span>
                      </button>
                    )}
                  </For>
                </div>
              </Show>
            </div>

            {/* Globe HUD — bottom-right */}
            <div class="fcmd-globe-hud fcmd-hud-br">
              <Show when={iss()}>
                <span class="fcmd-dim">ISS: {iss()!.latitude.toFixed(1)}°, {iss()!.longitude.toFixed(1)}°</span>
              </Show>
              <span class="fcmd-dim">SRC: {
                mode() === "SAT" ? "CELESTRAK" :
                mode() === "FLIGHTS" ? "OPENSKY-NET" :
                mode() === "CAMS" ? "CURATED" : "FLUX-NET"
              }</span>
            </div>

            {/* Altitude Legend (Flights mode) */}
            <Show when={mode() === "FLIGHTS"}>
              <div class="fcmd-alt-legend">
                <span class="fcmd-alt-title">ALT (ft)</span>
                <div class="fcmd-alt-item"><span style={{ background: "#00ff41" }} />{"<10K"}</div>
                <div class="fcmd-alt-item"><span style={{ background: "#44ff88" }} />10-20K</div>
                <div class="fcmd-alt-item"><span style={{ background: "#88ffcc" }} />20-30K</div>
                <div class="fcmd-alt-item"><span style={{ background: "#00d4ff" }} />30-36K</div>
                <div class="fcmd-alt-item"><span style={{ background: "#4488ff" }} />36K+</div>
              </div>
            </Show>

            {/* Floating Webcam Player (CAMS mode, over globe) */}
            <Show when={mode() === "CAMS" && activeWebcam()}>
              <div class="fcmd-floating-cam">
                <div class="fcmd-floating-cam-header">
                  <span class="fcmd-live-dot" />
                  <span>{activeWebcam()!.city} — {activeWebcam()!.label}</span>
                  <button class="fcmd-floating-cam-close" onClick={() => setActiveWebcam(null)}>✕</button>
                </div>
                <div class="fcmd-floating-cam-viewport">
                  <iframe
                    src={webcamUrl()}
                    title={activeWebcam()!.label}
                    allow="accelerometer; autoplay; encrypted-media; gyroscope"
                    allowfullscreen
                    class="fcmd-cam-iframe"
                  />
                </div>
              </div>
            </Show>

            {/* Floating Live News Stream (bottom-right of globe) */}
            <Show when={showStream()}>
              <div class="fcmd-floating-stream">
                <div class="fcmd-floating-stream-header">
                  <span class="fcmd-live-dot" />
                  <span>LIVE — {LIVE_STREAMS[activeStream()].label}</span>
                  <span class="fcmd-floating-stream-actions">
                    <span class="fcmd-mute-btn" onClick={() => setStreamMuted((m) => !m)} title="Toggle audio (M)">
                      {streamMuted() ? "🔇" : "🔊"}
                    </span>
                    <button class="fcmd-floating-stream-close" onClick={() => setShowStream(false)}>✕</button>
                  </span>
                </div>
                <div class="fcmd-floating-stream-tabs">
                  <For each={LIVE_STREAMS}>
                    {(s, i) => (
                      <button
                        class={`fcmd-stream-tab ${activeStream() === i() ? "active" : ""}`}
                        onClick={() => setActiveStream(i())}
                        style={{ "--tab-accent": s.accent }}
                      >
                        <span class="fcmd-stream-tab-dot" />{s.tag}
                      </button>
                    )}
                  </For>
                </div>
                <div class="fcmd-floating-stream-viewport">
                  <iframe
                    src={streamUrl()}
                    title={LIVE_STREAMS[activeStream()].label}
                    allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture"
                    allowfullscreen
                    class="fcmd-stream-iframe"
                  />
                </div>
              </div>
            </Show>

            {/* Stream toggle icon (bottom-right of globe) */}
            <Show when={!showStream()}>
              <button
                class="fcmd-stream-toggle-icon"
                onClick={() => setShowStream(true)}
                title="Open live news stream"
              >
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <rect x="2" y="7" width="20" height="15" rx="2" ry="2" />
                  <polyline points="17 2 12 7 7 2" />
                </svg>
              </button>
            </Show>
          </div>

          {/* ─── RIGHT PANEL ───────────────────────────────────── */}
          <aside class="fcmd-panel-col fcmd-right">


            {/* SAT mode: right panel info */}
            <Show when={mode() === "SAT"}>
              <section class="fcmd-panel fcmd-stream-panel">
                <h3 class="fcmd-panel-hdr">ℹ SATELLITE INTEL</h3>
                <div class="fcmd-sat-info-box">
                  <p>Tracking <strong>{satPositions().length}</strong> objects in the <strong>{satGroup().toUpperCase()}</strong> group.</p>
                  <p class="fcmd-dim">Positions computed via SGP4 propagation from NORAD TLE data (CelesTrak). Updated every 2s.</p>
                    <div class="fcmd-sat-legend">
                    <div class="fcmd-legend-row"><span style={{ color: "#ff4466" }}>🛰</span> Space Stations (ISS, Tiangong)</div>
                    <div class="fcmd-legend-row"><span style={{ color: "#cc66ff" }}>🔭</span> Science (Hubble, JWST)</div>
                    <div class="fcmd-legend-row"><span style={{ color: "#44ddaa" }}>🌤</span> Weather (GOES, NOAA)</div>
                    <div class="fcmd-legend-row"><span style={{ color: "#ffcc44" }}>📍</span> Navigation (GPS, GLONASS)</div>
                    <div class="fcmd-legend-row"><span style={{ color: "#88aaff" }}>⛓</span> Starlink</div>
                    <div class="fcmd-legend-row"><span style={{ color: "#44ccff" }}>✦</span> Iridium</div>
                    <div class="fcmd-legend-row"><span style={{ color: "#aa88ff" }}>◈</span> OneWeb</div>
                    <div class="fcmd-legend-row"><span style={{ color: "#ff8844" }}>⊛</span> Geostationary</div>
                    <div class="fcmd-legend-row"><span style={{ color: "#44aaff" }}>🌍</span> Earth Observation</div>
                    <div class="fcmd-legend-row"><span style={{ color: "#88aaff" }}>📡</span> Communications</div>
                </div>
                </div>
              </section>
            </Show>

            {/* FLIGHTS mode: right panel info */}
            <Show when={mode() === "FLIGHTS"}>
              <section class="fcmd-panel fcmd-stream-panel">
                <h3 class="fcmd-panel-hdr">ℹ FLIGHT INTEL</h3>
                <div class="fcmd-sat-info-box">
                  <p>Showing <strong>{flights().length}</strong> airborne aircraft worldwide.</p>
                  <p class="fcmd-dim">Live data from OpenSky Network ADS-B receivers. Click an aircraft on the globe or list for details.</p>
                  <Show when={flights().length > 0}>
                    <div style={{ "margin-top": "8px" }}>
                      <div class="fcmd-kv"><span>AVG ALT</span><span>
                        {(flights().reduce((s, f) => s + f.altitude, 0) / flights().length / 0.3048).toFixed(0)} ft
                      </span></div>
                      <div class="fcmd-kv"><span>AVG SPD</span><span>
                        {(flights().reduce((s, f) => s + f.velocity, 0) / flights().length * 1.944).toFixed(0)} kts
                      </span></div>
                      <div class="fcmd-kv"><span>COUNTRIES</span><span>
                        {new Set(flights().map((f) => f.origin_country)).size}
                      </span></div>
                    </div>
                  </Show>
                </div>
              </section>
            </Show>

            {/* News Feed — always visible */}
            <section class="fcmd-panel fcmd-panel-grow">
              <h3 class="fcmd-panel-hdr">📡 GLOBAL FEED</h3>
              <div class="fcmd-news-list">
                <For each={news()}>
                  {(item, i) => (
                    <div
                      class={`fcmd-news-item${item.url ? " fcmd-news-clickable" : ""}`}
                      onClick={() => { if (item.url) open(item.url); }}
                      title={item.url || undefined}
                    >
                      <span class="fcmd-news-idx">{String(i() + 1).padStart(2, "0")}</span>
                      <div class="fcmd-news-body">
                        <div class="fcmd-news-title">{item.title}</div>
                        <div class="fcmd-news-meta">
                          {item.source} · {item.timestamp}
                          <Show when={!!item.url}>
                            <span class="fcmd-news-link">↗</span>
                          </Show>
                        </div>
                      </div>
                    </div>
                  )}
                </For>
                <Show when={news().length === 0}>
                  <div class="fcmd-dim" style={{ padding: "12px" }}>Decrypting feeds...</div>
                </Show>
              </div>
            </section>
          </aside>
        </main>

        {/* ═══ BOTTOM TICKER ═════════════════════════════════════ */}
        <footer class="fcmd-ticker">
          <div class="fcmd-ticker-label">INTEL</div>
          <div class="fcmd-ticker-track">
            <div
              class="fcmd-ticker-text"
              style={{ transform: `translateX(-${tickerOffset() % (tickerText().length * 7.5)}px)` }}
            >
              {tickerText()}{tickerText()}{tickerText()}
            </div>
          </div>
        </footer>
      </div>
    </div>
  );
}