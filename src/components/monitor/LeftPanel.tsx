import { Show, For } from "solid-js";
import type { MonitorStore } from "./types";
import { SAT_GROUPS, WEBCAMS } from "./constants";
import { severityColor, altitudeColor, getSatVisual } from "./utils";

interface LeftPanelProps {
  store: MonitorStore;
  onFocusGlobe: (lat: number, lng: number, alt?: number) => void;
  onPauseRotation: () => void;
}

export default function LeftPanel(props: LeftPanelProps) {
  const { store } = props;

  return (
    <aside class="fcmd-panel-col fcmd-left">

      {/* System Panel — always visible */}
      <section class="fcmd-panel">
        <h3 class="fcmd-panel-hdr">⊞ SYSTEM</h3>
        <Show when={store.stats()} fallback={<div class="fcmd-dim">Loading...</div>}>
          <div class="fcmd-kv"><span>HOST</span><span>{store.stats()!.hostname}</span></div>
          <div class="fcmd-kv"><span>OS</span><span>{store.stats()!.os}</span></div>
          <div class="fcmd-kv"><span>CPU</span><span>{store.stats()!.cpu_count} cores</span></div>
          <div class="fcmd-kv"><span>RAM</span><span>{store.stats()!.memory_used_mb}/{store.stats()!.memory_total_mb} MB</span></div>
          <div class="fcmd-progress-bar">
            <div class="fcmd-progress-fill" style={{
              width: `${store.memPercent()}%`,
              background: store.memPercent() > 80 ? "#ff4444" : "#00ff41",
            }} />
          </div>
          <div class="fcmd-kv"><span>UPTIME</span><span>{store.formatUptime(store.stats()!.uptime_secs)}</span></div>
          <div class="fcmd-kv"><span>LAN</span><span class="fcmd-mono">{store.stats()!.local_ip}</span></div>
          <div class="fcmd-kv"><span>WAN</span><span class="fcmd-mono">{store.publicIp()}</span></div>
        </Show>
      </section>

      {/* ISS Panel — always visible */}
      <section class="fcmd-panel">
        <h3 class="fcmd-panel-hdr">🛰 ISS TRACKER</h3>
        <Show when={store.iss()} fallback={<div class="fcmd-dim">Acquiring...</div>}>
          <div class="fcmd-kv"><span>LAT</span><span class="fcmd-val-cyan">{store.iss()!.latitude.toFixed(4)}°</span></div>
          <div class="fcmd-kv"><span>LON</span><span class="fcmd-val-cyan">{store.iss()!.longitude.toFixed(4)}°</span></div>
          <div class="fcmd-kv"><span>ALT</span><span>{store.iss()!.altitude.toFixed(0)} km</span></div>
          <div class="fcmd-kv"><span>VEL</span><span>{store.iss()!.velocity.toFixed(0)} km/h</span></div>
        </Show>
      </section>

      {/* INTEL / CYBER: Threat Feed */}
      <Show when={store.mode() === "INTEL" || store.mode() === "CYBER"}>
        <section class="fcmd-panel fcmd-panel-grow">
          <h3 class="fcmd-panel-hdr">⚠ THREAT FEED</h3>
          <div class="fcmd-threat-list">
            <For each={store.threats().slice(0, 10)}>
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
      <Show when={store.mode() === "SAT"}>
        <section class="fcmd-panel">
          <h3 class="fcmd-panel-hdr">📡 SAT GROUP</h3>
          <div class="fcmd-sat-groups">
            <For each={SAT_GROUPS}>
              {(g) => (
                <button
                  class={`fcmd-sat-group-btn ${store.satGroup() === g.id ? "active" : ""}`}
                  onClick={() => store.setSatGroup(g.id)}
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
          <Show when={store.satLoading()}>
            <div class="fcmd-dim">Fetching TLE data...</div>
          </Show>
          <Show when={!store.satLoading()}>
            <div class="fcmd-kv"><span>GROUP</span><span>{store.satGroup().toUpperCase()}</span></div>
            <div class="fcmd-kv"><span>TLEs</span><span>{store.satTLEs().length}</span></div>
            <div class="fcmd-kv"><span>TRACKED</span><span class="fcmd-val-cyan">{store.satPositions().length}</span></div>
            <div class="fcmd-kv"><span>SOURCE</span><span class="fcmd-dim">CelesTrak</span></div>
          </Show>
        </section>
        <Show when={store.selectedSat()}>
          <section class="fcmd-panel">
            <h3 class="fcmd-panel-hdr">🎯 SELECTED</h3>
            <div class="fcmd-kv"><span>NAME</span><span class="fcmd-val-cyan">{store.selectedSat()!.name}</span></div>
            <div class="fcmd-kv"><span>LAT</span><span>{store.selectedSat()!.lat.toFixed(3)}°</span></div>
            <div class="fcmd-kv"><span>LON</span><span>{store.selectedSat()!.lng.toFixed(3)}°</span></div>
            <div class="fcmd-kv"><span>ALT</span><span>{store.selectedSat()!.altKm.toFixed(0)} km</span></div>
            <button class="fcmd-btn" onClick={() => store.setSelectedSat(null)}>DESELECT</button>
          </section>
        </Show>
        <section class="fcmd-panel fcmd-panel-grow">
          <h3 class="fcmd-panel-hdr">🗒 SAT LIST</h3>
          <div class="fcmd-scroll-list">
            <For each={store.satPositions().slice(0, 50)}>
              {(s) => {
                const vis = getSatVisual(s.name, s.group);
                return (
                  <div
                    class={`fcmd-list-row ${store.selectedSat()?.name === s.name ? "selected" : ""}`}
                    onClick={() => {
                      store.setSelectedSat(s);
                      props.onPauseRotation();
                      props.onFocusGlobe(s.lat, s.lng, 1.5);
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
      <Show when={store.mode() === "FLIGHTS"}>
        <section class="fcmd-panel">
          <h3 class="fcmd-panel-hdr">📊 FLIGHT DATA</h3>
          <Show when={store.flightLoading()}>
            <div class="fcmd-dim">Contacting OpenSky Network...</div>
          </Show>
          <Show when={!store.flightLoading()}>
            <div class="fcmd-kv"><span>AIRBORNE</span><span class="fcmd-val-cyan">{store.flights().length}</span></div>
            <div class="fcmd-kv"><span>SOURCE</span><span class="fcmd-dim">OpenSky Network</span></div>
            <div class="fcmd-kv">
              <span>TOP ORIGIN</span>
              <span>{(() => {
                const counts: Record<string, number> = {};
                store.flights().forEach((f) => { counts[f.origin_country] = (counts[f.origin_country] || 0) + 1; });
                return Object.entries(counts).sort((a, b) => b[1] - a[1])?.[0]?.[0] || "—";
              })()}</span>
            </div>
          </Show>
          <button class="fcmd-btn" onClick={() => store.fetchFlights()} style={{ "margin-top": "6px" }}>
            ↻ REFRESH
          </button>
        </section>
        <Show when={store.selectedFlight()}>
          <section class="fcmd-panel">
            <h3 class="fcmd-panel-hdr">🎯 SELECTED</h3>
            <div class="fcmd-kv"><span>CALL</span><span class="fcmd-val-cyan">{store.selectedFlight()!.callsign || "N/A"}</span></div>
            <div class="fcmd-kv"><span>ICAO</span><span>{store.selectedFlight()!.icao24}</span></div>
            <div class="fcmd-kv"><span>ORIGIN</span><span>{store.selectedFlight()!.origin_country}</span></div>
            <div class="fcmd-kv"><span>ALT</span><span>{(store.selectedFlight()!.altitude / 0.3048).toFixed(0)} ft</span></div>
            <div class="fcmd-kv"><span>SPD</span><span>{(store.selectedFlight()!.velocity * 1.944).toFixed(0)} kts</span></div>
            <div class="fcmd-kv"><span>HDG</span><span>{store.selectedFlight()!.heading.toFixed(0)}°</span></div>
            <div class="fcmd-kv"><span>V/S</span><span>{(store.selectedFlight()!.vertical_rate * 196.85).toFixed(0)} fpm</span></div>
            <button class="fcmd-btn" onClick={() => store.setSelectedFlight(null)}>DESELECT</button>
          </section>
        </Show>
        <section class="fcmd-panel fcmd-panel-grow">
          <h3 class="fcmd-panel-hdr">✈ FLIGHT LIST</h3>
          <div class="fcmd-scroll-list">
            <For each={store.flights().slice(0, 80)}>
              {(f) => (
                <div
                  class={`fcmd-list-row ${store.selectedFlight()?.icao24 === f.icao24 ? "selected" : ""}`}
                  onClick={() => {
                    store.setSelectedFlight(f);
                    props.onFocusGlobe(f.latitude, f.longitude, 1.2);
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
      <Show when={store.mode() === "CAMS"}>
        <section class="fcmd-panel fcmd-panel-grow">
          <h3 class="fcmd-panel-hdr">📷 WEBCAMS ({WEBCAMS.length})</h3>
          <div class="fcmd-scroll-list">
            <For each={WEBCAMS}>
              {(cam) => (
                <div
                  class={`fcmd-list-row ${store.activeWebcam()?.id === cam.id ? "selected" : ""}`}
                  onClick={() => {
                    store.setActiveWebcam(cam);
                    props.onFocusGlobe(cam.lat, cam.lng, 1.0);
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

      {/* WEATHER mode: city weather list */}
      <Show when={store.mode() === "WEATHER"}>
        <section class="fcmd-panel fcmd-panel-grow">
          <h3 class="fcmd-panel-hdr">🌤️ WEATHER ({store.weather().length})</h3>
          <Show when={store.weather().length > 0} fallback={<div class="fcmd-dim" style={{ padding: "12px" }}>Fetching weather...</div>}>
            <div class="fcmd-scroll-list">
              <For each={store.weather()}>
                {(wp) => {
                  const tempColor = () => wp.temperature < 0 ? "#60a5fa" : wp.temperature < 15 ? "#94a3b8" : wp.temperature < 30 ? "#fbbf24" : "#ef4444";
                  return (
                    <div
                      class="fcmd-list-row fcmd-weather-row"
                      onClick={() => props.onFocusGlobe(wp.lat, wp.lng, 1.2)}
                    >
                      <span class="fcmd-weather-row-icon">{wp.icon}</span>
                      <div class="fcmd-weather-row-info">
                        <span class="fcmd-weather-row-city">{wp.city}, {wp.country}</span>
                        <span class="fcmd-weather-row-desc">{wp.description}</span>
                      </div>
                      <div class="fcmd-weather-row-data">
                        <span class="fcmd-weather-row-temp" style={{ color: tempColor() }}>{Math.round(wp.temperature)}°C</span>
                        <span class="fcmd-weather-row-meta">💧{wp.humidity}% 💨{wp.wind_speed.toFixed(0)}</span>
                      </div>
                    </div>
                  );
                }}
              </For>
            </div>
          </Show>
        </section>
      </Show>

      {/* QUAKE mode: earthquake list */}
      <Show when={store.mode() === "QUAKE"}>
        <section class="fcmd-panel fcmd-panel-grow">
          <h3 class="fcmd-panel-hdr">🌋 EARTHQUAKES ({store.quakes().length})</h3>
          <Show when={store.quakes().length > 0} fallback={<div class="fcmd-dim" style={{ padding: "12px" }}>Fetching seismic data...</div>}>
            <div class="fcmd-scroll-list">
              <For each={store.quakes()}>
                {(qk) => {
                  const magColor = () => qk.magnitude < 3 ? "#22c55e" : qk.magnitude < 5 ? "#eab308" : qk.magnitude < 7 ? "#f97316" : "#ef4444";
                  const timeAgo = () => {
                    const mins = Math.floor((Date.now() - qk.time) / 60000);
                    if (mins < 60) return `${mins}m`;
                    return `${Math.floor(mins / 60)}h`;
                  };
                  return (
                    <div
                      class="fcmd-list-row fcmd-quake-row"
                      onClick={() => props.onFocusGlobe(qk.lat, qk.lng, 1.0)}
                    >
                      <span class="fcmd-quake-mag-badge" style={{ background: magColor(), color: "#000" }}>
                        {qk.magnitude.toFixed(1)}
                      </span>
                      <div class="fcmd-quake-row-info">
                        <span class="fcmd-quake-row-place">{qk.place}</span>
                        <span class="fcmd-quake-row-meta">
                          {qk.depth.toFixed(0)}km deep · {timeAgo()} ago
                          {qk.tsunami ? " · ⚠️ TSUNAMI" : ""}
                        </span>
                      </div>
                    </div>
                  );
                }}
              </For>
            </div>
          </Show>
        </section>
      </Show>
    </aside>
  );
}
