import { Show, For } from "solid-js";
import type { DashboardMode, MonitorStore } from "./types";
import { MODE_CONFIG, LIVE_STREAMS, WEBCAMS } from "./constants";

interface GlobeOverlaysProps {
  store: MonitorStore;
  onModeSwitch: (mode: DashboardMode) => void;
}

export default function GlobeOverlays(props: GlobeOverlaysProps) {
  const { store } = props;

  return (
    <>
      {/* Globe HUD — top-left */}
      <div class="fcmd-globe-hud fcmd-hud-tl">
        <span>{MODE_CONFIG[store.mode()].icon} {MODE_CONFIG[store.mode()].label} MODE</span>
        <Show when={store.mode() === "SAT"}>
          <span class="fcmd-dim">TRACKING {store.satPositions().length} OBJECTS</span>
        </Show>
        <Show when={store.mode() === "FLIGHTS"}>
          <span class="fcmd-dim">{store.flights().length} AIRCRAFT</span>
        </Show>
        <Show when={store.mode() === "CAMS"}>
          <span class="fcmd-dim">{WEBCAMS.length} CAMERAS</span>
        </Show>
        <Show when={store.mode() === "CYBER"}>
          <span class="fcmd-dim" style={{ color: "#ff4444" }}>THREAT MONITORING ACTIVE</span>
        </Show>
        <Show when={store.mode() === "WEATHER"}>
          <span class="fcmd-dim">{store.weather().length} CITIES</span>
        </Show>
        <Show when={store.mode() === "QUAKE"}>
          <span class="fcmd-dim" style={{ color: "#ef4444" }}>{store.quakes().length} EARTHQUAKES (24H)</span>
        </Show>
      </div>

      {/* Mode selector — bottom-left of globe */}
      <div class="fcmd-mode-selector" onClick={(e) => e.stopPropagation()}>
        <button
          class="fcmd-mode-selector-btn"
          onClick={() => store.setShowModeMenu((v) => !v)}
          title="Switch mode"
        >
          <span>{MODE_CONFIG[store.mode()].icon}</span>
        </button>
        <Show when={store.showModeMenu()}>
          <div class="fcmd-mode-menu">
            <For each={Object.entries(MODE_CONFIG)}>
              {([key, cfg]) => (
                <button
                  class={`fcmd-mode-menu-item ${store.mode() === key ? "active" : ""}`}
                  onClick={() => {
                    props.onModeSwitch(key as DashboardMode);
                    store.setShowModeMenu(false);
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
        <Show when={store.iss()}>
          <span class="fcmd-dim">ISS: {store.iss()!.latitude.toFixed(1)}°, {store.iss()!.longitude.toFixed(1)}°</span>
        </Show>
        <span class="fcmd-dim">SRC: {
          store.mode() === "SAT" ? "CELESTRAK" :
          store.mode() === "FLIGHTS" ? "OPENSKY-NET" :
          store.mode() === "CAMS" ? "CURATED" :
          store.mode() === "WEATHER" ? "OPEN-METEO" :
          store.mode() === "QUAKE" ? "USGS" : "FLUX-NET"
        }</span>
      </div>

      {/* Altitude Legend (Flights mode) */}
      <Show when={store.mode() === "FLIGHTS"}>
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
      <Show when={store.mode() === "CAMS" && store.activeWebcam()}>
        <div class="fcmd-floating-cam">
          <div class="fcmd-floating-cam-header">
            <span class="fcmd-live-dot" />
            <span>{store.activeWebcam()!.city} — {store.activeWebcam()!.label}</span>
            <button class="fcmd-floating-cam-close" onClick={() => store.setActiveWebcam(null)}>✕</button>
          </div>
          <div class="fcmd-floating-cam-viewport">
            <iframe
              src={store.webcamUrl()}
              title={store.activeWebcam()!.label}
              allow="accelerometer; autoplay; encrypted-media; gyroscope"
              allowfullscreen
              class="fcmd-cam-iframe"
            />
          </div>
        </div>
      </Show>

      {/* Floating Live News Stream (bottom-right of globe) */}
      <Show when={store.showStream()}>
        <div class="fcmd-floating-stream">
          <div class="fcmd-floating-stream-header">
            <span class="fcmd-live-dot" />
            <span>LIVE — {LIVE_STREAMS[store.activeStream()].label}</span>
            <span class="fcmd-floating-stream-actions">
              <span class="fcmd-mute-btn" onClick={() => store.setStreamMuted((m) => !m)} title="Toggle audio (M)">
                {store.streamMuted() ? "🔇" : "🔊"}
              </span>
              <button class="fcmd-floating-stream-close" onClick={() => store.setShowStream(false)}>✕</button>
            </span>
          </div>
          <div class="fcmd-floating-stream-tabs">
            <For each={LIVE_STREAMS}>
              {(s, i) => (
                <button
                  class={`fcmd-stream-tab ${store.activeStream() === i() ? "active" : ""}`}
                  onClick={() => store.setActiveStream(i())}
                  style={{ "--tab-accent": s.accent }}
                >
                  <span class="fcmd-stream-tab-dot" />{s.tag}
                </button>
              )}
            </For>
          </div>
          <div class="fcmd-floating-stream-viewport">
            <iframe
              src={store.streamUrl()}
              title={LIVE_STREAMS[store.activeStream()].label}
              allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture"
              allowfullscreen
              class="fcmd-stream-iframe"
            />
          </div>
        </div>
      </Show>

      {/* Stream toggle icon (bottom-right of globe) */}
      <Show when={!store.showStream()}>
        <button
          class="fcmd-stream-toggle-icon"
          onClick={() => store.setShowStream(true)}
          title="Open live news stream"
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="2" y="7" width="20" height="15" rx="2" ry="2" />
            <polyline points="17 2 12 7 7 2" />
          </svg>
        </button>
      </Show>
    </>
  );
}
