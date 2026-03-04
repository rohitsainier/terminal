import type { Accessor, Setter } from "solid-js";

export interface ISSPos {
  latitude: number;
  longitude: number;
  altitude: number;
  velocity: number;
}
export interface NewsItem {
  title: string;
  source: string;
  timestamp: string;
  url: string;
}
export interface Activity {
  lat: number;
  lon: number;
  label: string;
  event_type: string;
  intensity: number;
}
export interface WeatherPoint {
  city: string;
  country: string;
  lat: number;
  lng: number;
  temperature: number;
  humidity: number;
  wind_speed: number;
  weather_code: number;
  description: string;
  icon: string;
}
export interface QuakeEvent {
  id: string;
  place: string;
  lat: number;
  lng: number;
  magnitude: number;
  depth: number;
  time: number;
  url: string;
  tsunami: boolean;
}
export interface CryptoPrice {
  id: string;
  symbol: string;
  name: string;
  price: number;
  change_24h: number;
  market_cap: number;
  volume_24h: number;
}
export interface SysStats {
  os: string;
  hostname: string;
  uptime_secs: number;
  cpu_count: number;
  memory_total_mb: number;
  memory_used_mb: number;
  local_ip: string;
  public_ip: string | null;
}
export interface SatTLE {
  name: string;
  line1: string;
  line2: string;
  group: string;
}
export interface SatPos {
  lat: number;
  lng: number;
  alt: number;
  altKm: number;
  name: string;
  group: string;
  velocity?: number;
}
export interface FlightInfo {
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
export interface WebcamInfo {
  id: string;
  city: string;
  country: string;
  lat: number;
  lng: number;
  label: string;
  url: string;
}
export interface ThreatEvent {
  id: string;
  src_ip: string;
  target: string;
  attack_type: string;
  severity: "low" | "medium" | "high" | "critical";
  time: string;
}

export type DashboardMode = "INTEL" | "CYBER" | "SAT" | "FLIGHTS" | "CAMS" | "WEATHER" | "QUAKE";

export interface MonitorStore {
  // Core
  mode: Accessor<DashboardMode>;
  setMode: Setter<DashboardMode>;
  utc: Accessor<string>;
  setUtc: Setter<string>;
  tickerOffset: Accessor<number>;
  setTickerOffset: Setter<number>;
  packetCount: Accessor<number>;
  setPacketCount: Setter<number>;
  globeReady: Accessor<boolean>;
  setGlobeReady: Setter<boolean>;
  streamMuted: Accessor<boolean>;
  setStreamMuted: Setter<boolean>;
  activeStream: Accessor<number>;
  setActiveStream: Setter<number>;
  showStream: Accessor<boolean>;
  setShowStream: Setter<boolean>;
  showModeMenu: Accessor<boolean>;
  setShowModeMenu: Setter<boolean>;

  // Data
  iss: Accessor<ISSPos | null>;
  setISS: Setter<ISSPos | null>;
  news: Accessor<NewsItem[]>;
  setNews: Setter<NewsItem[]>;
  stats: Accessor<SysStats | null>;
  setStats: Setter<SysStats | null>;
  activity: Accessor<Activity[]>;
  setActivity: Setter<Activity[]>;
  weather: Accessor<WeatherPoint[]>;
  setWeather: Setter<WeatherPoint[]>;
  quakes: Accessor<QuakeEvent[]>;
  setQuakes: Setter<QuakeEvent[]>;
  crypto: Accessor<CryptoPrice[]>;
  setCrypto: Setter<CryptoPrice[]>;
  publicIp: Accessor<string>;
  setPublicIp: Setter<string>;
  threats: Accessor<ThreatEvent[]>;
  setThreats: Setter<ThreatEvent[]>;

  // Satellites
  satGroup: Accessor<string>;
  setSatGroup: Setter<string>;
  satTLEs: Accessor<SatTLE[]>;
  setSatTLEs: Setter<SatTLE[]>;
  satPositions: Accessor<SatPos[]>;
  setSatPositions: Setter<SatPos[]>;
  satLoading: Accessor<boolean>;
  setSatLoading: Setter<boolean>;
  selectedSat: Accessor<SatPos | null>;
  setSelectedSat: Setter<SatPos | null>;

  // Flights
  flights: Accessor<FlightInfo[]>;
  setFlights: Setter<FlightInfo[]>;
  flightLoading: Accessor<boolean>;
  setFlightLoading: Setter<boolean>;
  selectedFlight: Accessor<FlightInfo | null>;
  setSelectedFlight: Setter<FlightInfo | null>;

  // Webcams
  activeWebcam: Accessor<WebcamInfo | null>;
  setActiveWebcam: Setter<WebcamInfo | null>;

  // Fetch functions
  fetchCoreData: () => void;
  fetchCrypto: () => void;
  fetchISS: () => void;
  fetchSatellites: (group: string) => void;
  fetchFlights: () => void;
  propagateAllSats: (tles?: SatTLE[]) => void;

  // Derived helpers
  memPercent: () => number;
  threatLevel: () => { text: string; color: string };
  tickerText: () => string;
  streamUrl: () => string;
  webcamUrl: () => string;
  formatUptime: (secs: number) => string;
  tzTime: (offset: number) => string;
}
