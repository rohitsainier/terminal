import { createEffect } from "solid-js";
import { open } from "@tauri-apps/plugin-shell";
import type { DashboardMode, MonitorStore } from "./types";
import { GLOBE_CITIES, WEBCAMS } from "./constants";
import {
  generateArcs, altitudeColor, getSatVisual, computeOrbitPath,
} from "./utils";

export interface GlobeTimers {
  satProp: number | undefined;
  autoRotateResume: number | undefined;
}

export function pauseGlobeRotation(globeInstance: any, timers: GlobeTimers) {
  if (!globeInstance) return;
  globeInstance.controls().autoRotate = false;
  clearTimeout(timers.autoRotateResume);
  timers.autoRotateResume = window.setTimeout(() => {
    if (globeInstance) {
      globeInstance.controls().autoRotate = true;
      globeInstance.controls().autoRotateSpeed = 0.2;
    }
  }, 15000);
}

function createMarkerEl(
  d: any,
  globeInstance: any,
  timers: GlobeTimers,
  store: MonitorStore,
): HTMLDivElement {
  const el = document.createElement("div");

  switch (d._type) {
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

    case "sat": {
      const vis = getSatVisual(d.name || "", d.group || "");
      const sizeClass = vis.size === "lg" ? "lg" : vis.size === "md" ? "md" : "sm";
      el.className = `fcmd-marker fcmd-marker-sat ${sizeClass}${vis.glow ? " glow" : ""}`;
      el.style.setProperty("--sat-color", vis.color);

      if (sizeClass === "sm" && !vis.glow) {
        el.innerHTML = `<span class="fcmd-sat-icon-s" style="color:${vis.color}">${vis.icon}</span>`;
      } else {
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

    case "cam": {
      el.className = "fcmd-marker fcmd-marker-cam";
      el.innerHTML = `
        <span class="fcmd-cam-pin">📷</span>
        <span class="fcmd-cam-lbl">${d.city || d.label || ""}</span>
      `;
      el.title = `${d.city || ""} — ${d.label || ""}\n${d.country || ""}`;
      break;
    }

    case "weather": {
      el.className = "fcmd-marker fcmd-weather-marker";
      const temp = d.temperature ?? 0;
      const tempColor = temp < 0 ? "#60a5fa" : temp < 15 ? "#94a3b8" : temp < 30 ? "#fbbf24" : "#ef4444";
      el.innerHTML = `
        <span class="fcmd-weather-icon">${d.icon || "🌡️"}</span>
        <span class="fcmd-weather-temp" style="color: ${tempColor}">${Math.round(temp)}°</span>
        <span class="fcmd-weather-city">${d.city || ""}</span>
      `;
      el.title = `${d.city}, ${d.country}\n${d.description}\nTemp: ${temp}°C | Humidity: ${d.humidity}% | Wind: ${d.wind_speed} km/h`;
      break;
    }

    case "quake": {
      el.className = "fcmd-marker fcmd-quake-marker";
      const mag = d.magnitude ?? 0;
      const size = Math.max(16, mag * 6);
      const magColor = mag < 3 ? "#22c55e" : mag < 5 ? "#eab308" : mag < 7 ? "#f97316" : "#ef4444";
      const timeAgo = (() => {
        const mins = Math.floor((Date.now() - (d.time || 0)) / 60000);
        if (mins < 60) return `${mins}m ago`;
        const hrs = Math.floor(mins / 60);
        return `${hrs}h ago`;
      })();
      el.innerHTML = `
        <div class="fcmd-quake-circle" style="
          width: ${size}px; height: ${size}px;
          background: ${magColor}33;
          border: 2px solid ${magColor};
          box-shadow: 0 0 ${mag * 3}px ${magColor}66;
        ">
          <span class="fcmd-quake-mag-label" style="color: ${magColor}">${mag.toFixed(1)}</span>
        </div>
        <span class="fcmd-quake-place">${(d.place || "").split(",").pop()?.trim() || ""}</span>
      `;
      el.title = `${d.place}\nM${mag.toFixed(1)} | Depth: ${d.depth?.toFixed(1)}km | ${timeAgo}${d.tsunami ? " | ⚠️ TSUNAMI WARNING" : ""}`;
      if (d.url) {
        el.addEventListener("dblclick", (e) => {
          e.stopPropagation();
          open(d.url);
        });
      }
      break;
    }

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

  el.addEventListener("click", (e) => {
    e.stopPropagation();
    e.preventDefault();
    pauseGlobeRotation(globeInstance, timers);
    handleMarkerClick(d, globeInstance, store);
  });

  return el;
}

function handleMarkerClick(d: any, globeInstance: any, store: MonitorStore) {
  const m = store.mode();
  if (m === "SAT" && d._type === "sat") {
    store.setSelectedSat(d);
    if (globeInstance) {
      globeInstance.pointOfView({ lat: d.lat, lng: d.lng, altitude: 1.5 }, 800);
    }
  } else if (m === "FLIGHTS" && d._type === "flight") {
    store.setSelectedFlight(d);
    if (globeInstance) {
      globeInstance.pointOfView({ lat: d.latitude ?? d.lat, lng: d.longitude ?? d.lng, altitude: 1.2 }, 800);
    }
  } else if (m === "CAMS" && d._type === "cam") {
    store.setActiveWebcam(d);
    if (globeInstance) {
      globeInstance.pointOfView({ lat: d.lat, lng: d.lng, altitude: 1.0 }, 800);
    }
  }
}

// ─── Globe Init ───────────────────────────────────────

export function initGlobe(
  Globe: any,
  container: HTMLDivElement,
  store: MonitorStore,
  timers: GlobeTimers,
): any {
  const w = container.clientWidth;
  const h = container.clientHeight;
  if (w === 0 || h === 0) return null;

  const globeInstance = Globe()
    .globeImageUrl("//unpkg.com/three-globe/example/img/earth-night.jpg")
    .bumpImageUrl("//unpkg.com/three-globe/example/img/earth-topology.png")
    .backgroundColor("rgba(0,0,0,0)")
    .showGlobe(true)
    .showAtmosphere(true)
    .atmosphereColor("#00c8ff")
    .atmosphereAltitude(0.15)
    .pointsData(GLOBE_CITIES)
    .pointLat("lat")
    .pointLng("lng")
    .pointColor((d: any) => d.color || "#00c8ff")
    .pointAltitude((d: any) => d.alt || 0.01)
    .pointRadius((d: any) => d.size || d.radius || 0.15)
    .pointLabel((d: any) => `<div class="fcmd-globe-tooltip">${d.name || d.label || ""}</div>`)
    .onPointClick((p: any) => {
      pauseGlobeRotation(globeInstance, timers);
      handleMarkerClick(p, globeInstance, store);
    })
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
    .ringsData([])
    .ringLat("lat")
    .ringLng("lng")
    .ringColor(() => (t: number) => `rgba(0,200,255,${1 - t})`)
    .ringMaxRadius(3)
    .ringPropagationSpeed(2)
    .ringRepeatPeriod(1200)
    .labelsData(
      GLOBE_CITIES.filter((c) => c.size >= 0.25).map((c) => ({
        lat: c.lat, lng: c.lng,
        text: c.name.toUpperCase(),
        size: c.size,
      }))
    )
    .labelLat("lat")
    .labelLng("lng")
    .labelText("text")
    .labelColor(() => "rgba(140,200,255,0.35)")
    .labelSize((d: any) => d.size * 1.6)
    .labelDotRadius((d: any) => d.size * 0.5)
    .labelDotOrientation(() => "right" as any)
    .labelResolution(2)
    .pathsData([])
    .pathPointLat("lat")
    .pathPointLng("lng")
    .pathColor(() => "rgba(255,68,68,0.35)")
    .pathStroke(1.2)
    .pathDashLength(0.01)
    .pathDashGap(0.004)
    .pathDashAnimateTime(100000)
    .htmlElementsData([])
    .htmlLat((d: any) => d.lat ?? d.latitude ?? 0)
    .htmlLng((d: any) => d.lng ?? d.longitude ?? 0)
    .htmlAltitude((d: any) => d._htmlAlt ?? 0)
    .htmlElement((d: any) => createMarkerEl(d, globeInstance, timers, store))
    .width(w)
    .height(h)(container);

  const controls = globeInstance.controls();
  controls.autoRotate = true;
  controls.autoRotateSpeed = 0.2;
  controls.enableZoom = true;
  controls.minDistance = 140;
  controls.maxDistance = 650;
  controls.enableDamping = true;
  controls.dampingFactor = 0.12;

  const onInteract = () => pauseGlobeRotation(globeInstance, timers);
  container.addEventListener("pointerdown", onInteract);
  container.addEventListener("wheel", onInteract, { passive: true });
  container.addEventListener("touchstart", onInteract, { passive: true });

  globeInstance.pointOfView({ lat: 20, lng: 10, altitude: 2.2 }, 1500);

  const ro = new ResizeObserver((entries) => {
    for (const e of entries) {
      if (globeInstance && e.contentRect.width > 0) {
        globeInstance.width(e.contentRect.width).height(e.contentRect.height);
      }
    }
  });
  ro.observe(container);

  const arcsRefresh = setInterval(() => {
    if (globeInstance && (store.mode() === "INTEL" || store.mode() === "CYBER")) {
      globeInstance.arcsData(generateArcs(12, store.mode() === "CYBER"));
    }
  }, 7000);

  store.setGlobeReady(true);

  (container as any).__cleanup = () => {
    clearInterval(arcsRefresh);
    clearTimeout(timers.autoRotateResume);
    container.removeEventListener("pointerdown", onInteract);
    container.removeEventListener("wheel", onInteract);
    container.removeEventListener("touchstart", onInteract);
    ro.disconnect();
  };

  return globeInstance;
}

export function destroyGlobe(globeInstance: any, container: HTMLDivElement | undefined) {
  if ((container as any)?.__cleanup) (container as any).__cleanup();
  if (globeInstance) {
    try {
      const canvas = container?.querySelector("canvas");
      if (canvas) {
        const gl = canvas.getContext("webgl2") || canvas.getContext("webgl");
        gl?.getExtension("WEBGL_lose_context")?.loseContext();
      }
    } catch {}
  }
  if (container) {
    while (container.firstChild) container.removeChild(container.firstChild);
  }
}

// ─── Mode Configuration ───────────────────────────────

export function configureGlobeINTEL(g: any) {
  if (!g) return;
  g.globeImageUrl("//unpkg.com/three-globe/example/img/earth-night.jpg")
    .atmosphereColor("#3388ff")
    .pointsData(GLOBE_CITIES)
    .pointColor(() => "#55aaff")
    .pointAltitude(() => 0.01)
    .pointRadius((d: any) => d.size || 0.15)
    .arcsData(generateArcs(12))
    .labelsData(GLOBE_CITIES.filter((c) => c.size >= 0.22).map((c) => ({
      lat: c.lat, lng: c.lng, text: c.name.toUpperCase(), size: c.size,
    })))
    .labelColor(() => "rgba(180,210,255,0.55)")
    .labelSize((d: any) => (d.size || 0.2) * 2.2)
    .labelDotRadius((d: any) => (d.size || 0.2) * 0.6)
    .labelDotOrientation(() => "right" as any)
    .pathsData([])
    .ringsData([])
    .htmlElementsData([]);
  g.controls().autoRotateSpeed = 0.2;
}

export function configureGlobeCYBER(g: any, store: MonitorStore) {
  if (!g) return;
  g.globeImageUrl("//unpkg.com/three-globe/example/img/earth-night.jpg")
    .atmosphereColor("#ff2244")
    .pointsData(GLOBE_CITIES.map((c) => ({ ...c, color: "#ff5577" })))
    .pointColor((d: any) => d.color || "#ff5577")
    .pointAltitude(() => 0.01)
    .pointRadius((d: any) => d.size || 0.15)
    .arcsData(generateArcs(18, true))
    .labelsData(GLOBE_CITIES.filter((c) => c.size >= 0.22).map((c) => ({
      lat: c.lat, lng: c.lng, text: c.name.toUpperCase(), size: c.size,
    })))
    .labelColor(() => "rgba(255,150,170,0.55)")
    .labelSize((d: any) => (d.size || 0.2) * 2.2)
    .labelDotRadius((d: any) => (d.size || 0.2) * 0.6)
    .pathsData([])
    .ringsData(store.activity().slice(0, 6).map((a) => ({ lat: a.lat, lng: a.lon })))
    .ringColor(() => (t: number) => `rgba(255,68,68,${1 - t})`)
    .htmlElementsData([]);
  g.controls().autoRotateSpeed = 0.1;
}

export function configureGlobeSAT(g: any, store: MonitorStore, timers: GlobeTimers) {
  if (!g) return;
  g.globeImageUrl("//unpkg.com/three-globe/example/img/earth-night.jpg")
    .atmosphereColor("#4488ff")
    .arcsData([])
    .labelsData([])
    .pathsData([])
    .ringsData([])
    .htmlElementsData([]);
  g.controls().autoRotateSpeed = 0.25;
  store.fetchSatellites(store.satGroup());
  if (timers.satProp) clearInterval(timers.satProp);
  timers.satProp = window.setInterval(() => store.propagateAllSats(), 2000);
}

export function configureGlobeFLIGHTS(g: any, store: MonitorStore) {
  if (!g) return;
  g.globeImageUrl("//unpkg.com/three-globe/example/img/earth-blue-marble.jpg")
    .atmosphereColor("#55bbff")
    .arcsData([])
    .labelsData(GLOBE_CITIES.filter((c) => c.size >= 0.25).map((c) => ({
      lat: c.lat, lng: c.lng, text: c.name.toUpperCase(), size: c.size,
    })))
    .labelColor(() => "rgba(180,220,255,0.4)")
    .labelSize((d: any) => (d.size || 0.2) * 2)
    .labelDotRadius(() => 0)
    .pathsData([])
    .ringsData([])
    .pointsData([])
    .htmlElementsData([]);
  g.controls().autoRotateSpeed = 0.15;
  store.fetchFlights();
}

export function configureGlobeCAMS(g: any) {
  if (!g) return;
  const camMarkers = WEBCAMS.map((w) => ({
    ...w,
    _type: "cam" as const,
    _htmlAlt: 0.01,
  }));
  g.globeImageUrl("//unpkg.com/three-globe/example/img/earth-blue-marble.jpg")
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
  g.controls().autoRotateSpeed = 0.1;
}

export function configureGlobeWEATHER(g: any, store: MonitorStore) {
  if (!g) return;
  const w = store.weather();
  const markers = w.map((wp) => ({
    ...wp,
    _type: "weather" as const,
    _htmlAlt: 0.01,
  }));
  g.globeImageUrl("//unpkg.com/three-globe/example/img/earth-blue-marble.jpg")
    .atmosphereColor("#f59e0b")
    .pointsData([])
    .arcsData([])
    .labelsData([])
    .pathsData([])
    .ringsData([])
    .htmlElementsData(markers);
  g.controls().autoRotateSpeed = 0.15;
}

export function configureGlobeQUAKE(g: any, store: MonitorStore) {
  if (!g) return;
  const q = store.quakes();
  const markers = q.map((qk) => ({
    ...qk,
    _type: "quake" as const,
    _htmlAlt: 0.01,
  }));
  const rings = q.map((qk) => ({
    lat: qk.lat,
    lng: qk.lng,
    maxR: Math.max(1, qk.magnitude * 0.8),
    propagationSpeed: 2,
    repeatPeriod: Math.max(800, 3000 - qk.magnitude * 300),
  }));
  g.globeImageUrl("//unpkg.com/three-globe/example/img/earth-night.jpg")
    .atmosphereColor("#ef4444")
    .pointsData([])
    .arcsData([])
    .labelsData([])
    .pathsData([])
    .ringsData(rings)
    .ringLat((d: any) => d.lat)
    .ringLng((d: any) => d.lng)
    .ringColor(() => (t: number) => `rgba(239,68,68,${1 - t})`)
    .ringMaxRadius((d: any) => d.maxR)
    .ringPropagationSpeed((d: any) => d.propagationSpeed)
    .ringRepeatPeriod((d: any) => d.repeatPeriod)
    .htmlElementsData(markers);
  g.controls().autoRotateSpeed = 0.1;
}

// ─── Mode Switch ──────────────────────────────────────

export function switchMode(
  globeInstance: any,
  store: MonitorStore,
  timers: GlobeTimers,
  newMode: DashboardMode,
) {
  const prev = store.mode();
  if (prev === newMode) return;
  store.setMode(newMode);

  if (timers.satProp) { clearInterval(timers.satProp); timers.satProp = undefined; }
  store.setSelectedSat(null);
  store.setSelectedFlight(null);
  store.setActiveWebcam(null);

  if (!globeInstance) return;

  switch (newMode) {
    case "INTEL": configureGlobeINTEL(globeInstance); break;
    case "CYBER": configureGlobeCYBER(globeInstance, store); break;
    case "SAT": configureGlobeSAT(globeInstance, store, timers); break;
    case "FLIGHTS": configureGlobeFLIGHTS(globeInstance, store); break;
    case "CAMS": configureGlobeCAMS(globeInstance); break;
    case "WEATHER": configureGlobeWEATHER(globeInstance, store); break;
    case "QUAKE": configureGlobeQUAKE(globeInstance, store); break;
  }
}

// ─── Reactive Globe Effects ───────────────────────────

export function setupGlobeEffects(
  getGlobe: () => any,
  store: MonitorStore,
) {
  // Weather markers reactive update
  createEffect(() => {
    if (store.mode() !== "WEATHER" || !getGlobe()) return;
    const w = store.weather();
    if (!w.length) return;
    const markers = w.map((wp) => ({
      ...wp,
      _type: "weather" as const,
      _htmlAlt: 0.01,
    }));
    getGlobe().htmlElementsData(markers);
  });

  // Quake markers reactive update
  createEffect(() => {
    if (store.mode() !== "QUAKE" || !getGlobe()) return;
    const q = store.quakes();
    if (!q.length) return;
    const markers = q.map((qk) => ({
      ...qk,
      _type: "quake" as const,
      _htmlAlt: 0.01,
    }));
    const rings = q.map((qk) => ({
      lat: qk.lat,
      lng: qk.lng,
      maxR: Math.max(1, qk.magnitude * 0.8),
      propagationSpeed: 2,
      repeatPeriod: Math.max(800, 3000 - qk.magnitude * 300),
    }));
    getGlobe().htmlElementsData(markers);
    getGlobe().ringsData(rings);
  });

  // ISS on globe (INTEL / CYBER)
  createEffect(() => {
    const m = store.mode();
    const issData = store.iss();
    if (!getGlobe() || (m !== "INTEL" && m !== "CYBER")) return;

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
      getGlobe().pointsData([
        ...GLOBE_CITIES.map((c) => ({ ...c, color: cityColor, alt: 0.01 })),
        issPoint,
      ]);
      getGlobe().ringsData([
        { lat: issData.latitude, lng: issData.longitude },
        ...store.activity().slice(0, 4).map((a) => ({ lat: a.lat, lng: a.lon })),
      ]);
      getGlobe().htmlElementsData([
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

  // Satellites on globe
  createEffect(() => {
    if (store.mode() !== "SAT" || !getGlobe()) return;
    const positions = store.satPositions();
    if (!positions.length) return;

    const points = positions.map((s) => {
      const vis = getSatVisual(s.name, s.group);
      return {
        lat: s.lat, lng: s.lng, alt: s.alt,
        name: s.name, altKm: s.altKm, group: s.group,
        size: s.name.includes("ISS") ? 0.4 : 0.04,
        radius: s.name.includes("ISS") ? 0.4 : 0.04,
        color: vis.color,
        _type: "sat",
      };
    });
    getGlobe().pointsData(points);
    getGlobe().pointAltitude((d: any) => d.alt || 0.01);
    getGlobe().pointRadius((d: any) => d.radius || 0.04);

    const notableKeywords = [
      "ISS", "ZARYA", "TIANGONG", "HUBBLE", "JWST", "JAMES WEBB",
      "GOES", "NOAA", "LANDSAT", "TERRA", "AQUA", "TDRS",
      "METEOSAT", "FENGYUN",
    ];
    const notable = positions.filter((s) =>
      notableKeywords.some((kw) => s.name.toUpperCase().includes(kw))
    );
    const firstFew = positions
      .filter((s) => !notable.find((n) => n.name === s.name))
      .slice(0, Math.min(30, positions.length));

    const selSat = store.selectedSat();
    const markerData: any[] = [];

    for (const s of notable) {
      markerData.push({
        lat: s.lat, lng: s.lng, _type: "sat", _htmlAlt: s.alt + 0.01,
        name: s.name, altKm: s.altKm, group: s.group, alt: s.alt,
      });
    }
    for (const s of firstFew) {
      markerData.push({
        lat: s.lat, lng: s.lng, _type: "sat", _htmlAlt: s.alt + 0.005,
        name: s.name, altKm: s.altKm, group: s.group, alt: s.alt,
      });
    }
    if (selSat && !markerData.find((m) => m.name === selSat.name)) {
      markerData.push({
        lat: selSat.lat, lng: selSat.lng, _type: "sat", _htmlAlt: selSat.alt + 0.015,
        name: selSat.name, altKm: selSat.altKm, group: selSat.group, alt: selSat.alt,
      });
    }

    getGlobe().htmlElementsData(markerData);
    getGlobe().labelsData([]);

    const satForOrbit = selSat || positions.find((s) => s.name.toUpperCase().includes("ISS"));
    if (satForOrbit) {
      const tle = store.satTLEs().find((t) => t.name === satForOrbit.name);
      if (tle) {
        const vis = getSatVisual(satForOrbit.name, satForOrbit.group);
        const orbit = computeOrbitPath(tle, 150);
        getGlobe().pathsData(orbit.length > 2 ? [orbit] : []);
        getGlobe().pathColor(() => `${vis.color}44`);
        getGlobe().pathStroke(1.5);
      }
    }
  });

  // Flights on globe
  createEffect(() => {
    if (store.mode() !== "FLIGHTS" || !getGlobe()) return;
    const f = store.flights();
    if (!f.length) return;

    const flightMarkers = f.slice(0, 200).map((fl) => ({
      lat: fl.latitude, lng: fl.longitude,
      _type: "flight" as const,
      _htmlAlt: Math.min(fl.altitude / 600000, 0.15),
      _color: altitudeColor(fl.altitude),
      callsign: fl.callsign, icao24: fl.icao24,
      origin_country: fl.origin_country,
      altitude: fl.altitude, velocity: fl.velocity,
      heading: fl.heading, vertical_rate: fl.vertical_rate,
      latitude: fl.latitude, longitude: fl.longitude,
    }));

    getGlobe().htmlElementsData(flightMarkers);
    getGlobe().pointsData([]);
    getGlobe().labelsData([]);

    const sel = store.selectedFlight();
    if (sel) {
      getGlobe().ringsData([{ lat: sel.latitude, lng: sel.longitude }]);
      getGlobe().ringColor(() => (t: number) => `rgba(0,212,255,${1 - t})`);
    } else {
      getGlobe().ringsData([]);
    }
  });

  // Satellite group change
  createEffect(() => {
    const group = store.satGroup();
    if (store.mode() === "SAT") {
      store.fetchSatellites(group);
    }
  });
}
