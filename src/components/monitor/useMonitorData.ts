import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type {
  ISSPos, NewsItem, Activity, WeatherPoint, QuakeEvent, CryptoPrice,
  SysStats, SatTLE, SatPos, FlightInfo, WebcamInfo, ThreatEvent,
  DashboardMode, MonitorStore,
} from "./types";
import { LIVE_STREAMS } from "./constants";
import { propagateTLE, formatUptime, tzTime } from "./utils";

export function useMonitorData(): MonitorStore {
  // Core
  const [mode, setMode] = createSignal<DashboardMode>("INTEL");
  const [utc, setUtc] = createSignal("");
  const [tickerOffset, setTickerOffset] = createSignal(0);
  const [packetCount, setPacketCount] = createSignal(0);
  const [globeReady, setGlobeReady] = createSignal(false);
  const [streamMuted, setStreamMuted] = createSignal(true);
  const [activeStream, setActiveStream] = createSignal(0);
  const [showStream, setShowStream] = createSignal(false);
  const [showModeMenu, setShowModeMenu] = createSignal(false);

  // Data
  const [iss, setISS] = createSignal<ISSPos | null>(null);
  const [news, setNews] = createSignal<NewsItem[]>([]);
  const [stats, setStats] = createSignal<SysStats | null>(null);
  const [activity, setActivity] = createSignal<Activity[]>([]);
  const [weather, setWeather] = createSignal<WeatherPoint[]>([]);
  const [quakes, setQuakes] = createSignal<QuakeEvent[]>([]);
  const [crypto, setCrypto] = createSignal<CryptoPrice[]>([]);
  const [publicIp, setPublicIp] = createSignal("...");
  const [threats, setThreats] = createSignal<ThreatEvent[]>([]);

  // Satellites
  const [satGroup, setSatGroup] = createSignal("stations");
  const [satTLEs, setSatTLEs] = createSignal<SatTLE[]>([]);
  const [satPositions, setSatPositions] = createSignal<SatPos[]>([]);
  const [satLoading, setSatLoading] = createSignal(false);
  const [selectedSat, setSelectedSat] = createSignal<SatPos | null>(null);

  // Flights
  const [flights, setFlights] = createSignal<FlightInfo[]>([]);
  const [flightLoading, setFlightLoading] = createSignal(false);
  const [selectedFlight, setSelectedFlight] = createSignal<FlightInfo | null>(null);

  // Webcams
  const [activeWebcam, setActiveWebcam] = createSignal<WebcamInfo | null>(null);

  // Fetch functions
  async function fetchISS() {
    try { setISS(await invoke("monitor_iss_position")); } catch {}
  }

  async function fetchCoreData() {
    fetchISS();
    try { setNews(await invoke("monitor_news")); } catch {}
    try { setStats(await invoke("monitor_system_stats")); } catch {}
    try { setActivity(await invoke("monitor_activity")); } catch {}
    try { setPublicIp(await invoke("monitor_public_ip")); } catch {}
    try { setWeather(await invoke("monitor_weather")); } catch {}
    try { setQuakes(await invoke("monitor_quakes")); } catch {}
    try { setCrypto(await invoke("monitor_crypto")); } catch {}
  }

  async function fetchCrypto() {
    try { setCrypto(await invoke("monitor_crypto")); } catch {}
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

  // Derived helpers
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

  return {
    mode, setMode, utc, setUtc, tickerOffset, setTickerOffset,
    packetCount, setPacketCount, globeReady, setGlobeReady,
    streamMuted, setStreamMuted, activeStream, setActiveStream,
    showStream, setShowStream, showModeMenu, setShowModeMenu,
    iss, setISS, news, setNews, stats, setStats,
    activity, setActivity, weather, setWeather, quakes, setQuakes,
    crypto, setCrypto, publicIp, setPublicIp, threats, setThreats,
    satGroup, setSatGroup, satTLEs, setSatTLEs,
    satPositions, setSatPositions, satLoading, setSatLoading,
    selectedSat, setSelectedSat,
    flights, setFlights, flightLoading, setFlightLoading,
    selectedFlight, setSelectedFlight,
    activeWebcam, setActiveWebcam,
    fetchCoreData, fetchCrypto, fetchISS,
    fetchSatellites, fetchFlights, propagateAllSats,
    memPercent, threatLevel, tickerText, streamUrl, webcamUrl,
    formatUptime, tzTime,
  };
}
