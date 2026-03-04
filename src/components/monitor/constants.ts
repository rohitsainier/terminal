import type { DashboardMode, WebcamInfo } from "./types";

export const MODE_CONFIG: Record<
  DashboardMode,
  { icon: string; label: string; key: string; color: string }
> = {
  INTEL: { icon: "📡", label: "INTEL", key: "1", color: "#00ff41" },
  CYBER: { icon: "🔒", label: "CYBER", key: "2", color: "#ff4444" },
  SAT: { icon: "🛰", label: "SAT", key: "3", color: "#4488ff" },
  FLIGHTS: { icon: "🛫", label: "FLIGHTS", key: "4", color: "#00d4ff" },
  CAMS: { icon: "📷", label: "CAMS", key: "5", color: "#ffaa44" },
  WEATHER: { icon: "🌤️", label: "WEATHER", key: "6", color: "#f59e0b" },
  QUAKE: { icon: "🌋", label: "QUAKE", key: "7", color: "#ef4444" },
};

export const SAT_GROUPS = [
  { id: "stations", label: "STATIONS", desc: "ISS, Tiangong, etc." },
  { id: "starlink", label: "STARLINK", desc: "SpaceX constellation" },
  { id: "gps", label: "GPS", desc: "Navigation satellites" },
  { id: "weather", label: "WEATHER", desc: "Meteorological sats" },
  { id: "oneweb", label: "ONEWEB", desc: "OneWeb constellation" },
  { id: "iridium", label: "IRIDIUM", desc: "Iridium NEXT" },
  { id: "geo", label: "GEO", desc: "Geostationary orbit" },
  { id: "science", label: "SCIENCE", desc: "Scientific missions" },
];

export const LIVE_STREAMS = [
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

export const WEBCAMS: WebcamInfo[] = [
  { id: "nyc-ts", city: "New York", country: "US", lat: 40.758, lng: -73.985, label: "Times Square", url: "https://www.youtube-nocookie.com/embed/rnXIjl_Rzy4" },
  { id: "tokyo-shibuya", city: "Tokyo", country: "JP", lat: 35.659, lng: 139.700, label: "Shibuya Crossing", url: "https://www.youtube-nocookie.com/embed/3dfVK7ld38Ys" },
  { id: "london-eye", city: "London", country: "UK", lat: 51.503, lng: -0.119, label: "London Eye", url: "https://www.youtube-nocookie.com/embed/5XQZt2r8n9o" },
  { id: "sydney-opera", city: "Sydney", country: "AU", lat: -33.856, lng: 151.215, label: "Sydney Opera House", url: "https://www.youtube-nocookie.com/embed/1aXqj8ZyHkA" },
  { id: "paris-eiffel", city: "Paris", country: "FR", lat: 48.858, lng: 2.294, label: "Eiffel Tower", url: "https://www.youtube-nocookie.com/embed/2XjvQZyHkA" },
];

export const GLOBE_CITIES = [
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

export const ATTACK_TYPES = [
  "DDoS","SQL_INJ","XSS","BRUTE_FORCE","PORT_SCAN",
  "MALWARE_C2","PHISHING","ZERO_DAY","RANSOMWARE",
  "APT","DNS_TUNNEL","EXFIL","RCE","SSRF",
];

export const SAT_ICON_MAP: Record<string, { icon: string; color: string; label: string }> = {
  stations:  { icon: "🛰", color: "#ff4466", label: "Station" },
  starlink:  { icon: "⛓",  color: "#88aaff", label: "Starlink" },
  gps:       { icon: "📍", color: "#ffcc44", label: "GPS" },
  weather:   { icon: "🌤", color: "#44ddaa", label: "Weather" },
  oneweb:    { icon: "◈",  color: "#aa88ff", label: "OneWeb" },
  iridium:   { icon: "✦",  color: "#44ccff", label: "Iridium" },
  geo:       { icon: "⊛",  color: "#ff8844", label: "GEO" },
  science:   { icon: "🔭", color: "#ff66aa", label: "Science" },
};
