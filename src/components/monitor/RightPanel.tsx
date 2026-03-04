import { Show, For } from "solid-js";
import { open } from "@tauri-apps/plugin-shell";
import type { MonitorStore } from "./types";

interface RightPanelProps {
  store: MonitorStore;
}

export default function RightPanel(props: RightPanelProps) {
  const { store } = props;

  return (
    <aside class="fcmd-panel-col fcmd-right">

      {/* WEATHER mode: right panel summary */}
      <Show when={store.mode() === "WEATHER"}>
        <section class="fcmd-panel fcmd-stream-panel">
          <h3 class="fcmd-panel-hdr">🌡️ WEATHER SUMMARY</h3>
          <div class="fcmd-sat-info-box">
            <Show when={store.weather().length > 0} fallback={<p class="fcmd-dim">Loading weather data...</p>}>
              <div class="fcmd-kv"><span>CITIES</span><span>{store.weather().length}</span></div>
              <div class="fcmd-kv"><span>HOTTEST</span><span style={{ color: "#ef4444" }}>
                {(() => { const w = store.weather().reduce((a, b) => a.temperature > b.temperature ? a : b); return `${w.city} ${Math.round(w.temperature)}°C`; })()}
              </span></div>
              <div class="fcmd-kv"><span>COLDEST</span><span style={{ color: "#60a5fa" }}>
                {(() => { const w = store.weather().reduce((a, b) => a.temperature < b.temperature ? a : b); return `${w.city} ${Math.round(w.temperature)}°C`; })()}
              </span></div>
              <div class="fcmd-kv"><span>AVG TEMP</span><span>
                {Math.round(store.weather().reduce((s, w) => s + w.temperature, 0) / store.weather().length)}°C
              </span></div>
              <div class="fcmd-kv"><span>AVG HUMIDITY</span><span>
                {Math.round(store.weather().reduce((s, w) => s + w.humidity, 0) / store.weather().length)}%
              </span></div>
              <p class="fcmd-dim" style={{ "margin-top": "8px" }}>Data from Open-Meteo API. Updated every 10 minutes.</p>
            </Show>
          </div>
        </section>
      </Show>

      {/* QUAKE mode: right panel summary */}
      <Show when={store.mode() === "QUAKE"}>
        <section class="fcmd-panel fcmd-stream-panel">
          <h3 class="fcmd-panel-hdr">🌋 SEISMIC SUMMARY</h3>
          <div class="fcmd-sat-info-box">
            <Show when={store.quakes().length > 0} fallback={<p class="fcmd-dim">Loading seismic data...</p>}>
              <div class="fcmd-kv"><span>EVENTS (24H)</span><span>{store.quakes().length}</span></div>
              <div class="fcmd-kv"><span>STRONGEST</span><span style={{ color: "#ef4444" }}>
                M{store.quakes()[0].magnitude.toFixed(1)} — {store.quakes()[0].place.split(",").pop()?.trim()}
              </span></div>
              <div class="fcmd-kv"><span>AVG MAG</span><span>
                M{(store.quakes().reduce((s, q) => s + q.magnitude, 0) / store.quakes().length).toFixed(1)}
              </span></div>
              <div class="fcmd-kv"><span>AVG DEPTH</span><span>
                {(store.quakes().reduce((s, q) => s + q.depth, 0) / store.quakes().length).toFixed(0)} km
              </span></div>
              <div class="fcmd-kv"><span>TSUNAMI ALERTS</span><span style={{ color: store.quakes().some(q => q.tsunami) ? "#ef4444" : "inherit" }}>
                {store.quakes().filter(q => q.tsunami).length}
              </span></div>
              <p class="fcmd-dim" style={{ "margin-top": "8px" }}>Data from USGS Earthquake Hazards. M2.5+ events, updated every 5 min.</p>
            </Show>
          </div>
        </section>
      </Show>

      {/* SAT mode: right panel info */}
      <Show when={store.mode() === "SAT"}>
        <section class="fcmd-panel fcmd-stream-panel">
          <h3 class="fcmd-panel-hdr">ℹ SATELLITE INTEL</h3>
          <div class="fcmd-sat-info-box">
            <p>Tracking <strong>{store.satPositions().length}</strong> objects in the <strong>{store.satGroup().toUpperCase()}</strong> group.</p>
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
      <Show when={store.mode() === "FLIGHTS"}>
        <section class="fcmd-panel fcmd-stream-panel">
          <h3 class="fcmd-panel-hdr">ℹ FLIGHT INTEL</h3>
          <div class="fcmd-sat-info-box">
            <p>Showing <strong>{store.flights().length}</strong> airborne aircraft worldwide.</p>
            <p class="fcmd-dim">Live data from OpenSky Network ADS-B receivers. Click an aircraft on the globe or list for details.</p>
            <Show when={store.flights().length > 0}>
              <div style={{ "margin-top": "8px" }}>
                <div class="fcmd-kv"><span>AVG ALT</span><span>
                  {(store.flights().reduce((s, f) => s + f.altitude, 0) / store.flights().length / 0.3048).toFixed(0)} ft
                </span></div>
                <div class="fcmd-kv"><span>AVG SPD</span><span>
                  {(store.flights().reduce((s, f) => s + f.velocity, 0) / store.flights().length * 1.944).toFixed(0)} kts
                </span></div>
                <div class="fcmd-kv"><span>COUNTRIES</span><span>
                  {new Set(store.flights().map((f) => f.origin_country)).size}
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
          <For each={store.news()}>
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
          <Show when={store.news().length === 0}>
            <div class="fcmd-dim" style={{ padding: "12px" }}>Decrypting feeds...</div>
          </Show>
        </div>
      </section>
    </aside>
  );
}
