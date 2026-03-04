import {
  twoline2satrec,
  propagate,
  gstime,
  eciToGeodetic,
  degreesLat,
  degreesLong,
} from "satellite.js";
import type { SatTLE, SatPos, ThreatEvent } from "./types";
import { GLOBE_CITIES, ATTACK_TYPES, SAT_ICON_MAP } from "./constants";

export function randomIP(): string {
  return `${Math.floor(Math.random() * 223) + 1}.${Math.floor(Math.random() * 256)}.${Math.floor(Math.random() * 256)}.${Math.floor(Math.random() * 256)}`;
}

export function generateThreats(count: number): ThreatEvent[] {
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

export function generateArcs(count: number, threatMode = false) {
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

export function propagateTLE(tle: SatTLE, time?: Date): SatPos | null {
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

export function computeOrbitPath(tle: SatTLE, points = 120): { lat: number; lng: number }[] {
  try {
    const satrec = twoline2satrec(tle.line1, tle.line2);
    const periodMin = (2 * Math.PI) / satrec.no;
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

export function altitudeColor(altMeters: number): string {
  const altKm = altMeters / 1000;
  if (altKm < 3) return "#00ff41";
  if (altKm < 6) return "#44ff88";
  if (altKm < 9) return "#88ffcc";
  if (altKm < 11) return "#00d4ff";
  return "#4488ff";
}

export function severityColor(s: string) {
  switch (s) {
    case "critical": return "#ff0040";
    case "high": return "#ff4444";
    case "medium": return "#ffaa00";
    default: return "#00ff41";
  }
}

export function getSatVisual(name: string, group: string) {
  const upper = name.toUpperCase();

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

  const groupVisual = SAT_ICON_MAP[group] || { icon: "•", color: "#6688aa" };
  return { ...groupVisual, glow: false, size: "sm" as const };
}

export function tzTime(offset: number) {
  const d = new Date();
  d.setMinutes(d.getMinutes() + d.getTimezoneOffset() + offset * 60);
  return d.toTimeString().slice(0, 5);
}

export function formatUptime(secs: number) {
  const d = Math.floor(secs / 86400);
  const h = Math.floor((secs % 86400) / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return d > 0 ? `${d}d ${h}h ${m}m` : `${h}h ${m}m`;
}
