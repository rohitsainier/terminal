import { Show } from "solid-js";
import type { MonitorStore, NetThroughput } from "./types";

interface TopBarProps {
  store: MonitorStore;
  onClose: () => void;
}

function formatBps(bps: number): string {
  if (bps < 1024) return `${bps.toFixed(0)} B/s`;
  if (bps < 1048576) return `${(bps / 1024).toFixed(1)} KB/s`;
  if (bps < 1073741824) return `${(bps / 1048576).toFixed(1)} MB/s`;
  return `${(bps / 1073741824).toFixed(2)} GB/s`;
}

export default function TopBar(props: TopBarProps) {
  const { store } = props;

  const speedtestTooltip = () => {
    const s = store.speedtest();
    if (!s) return "Click to run speed test";
    const date = new Date(s.timestamp);
    return `Download: ${s.download_mbps.toFixed(1)} Mbps\nUpload: ${s.upload_mbps.toFixed(1)} Mbps\nPing: ${s.ping_ms.toFixed(0)} ms\nServer: ${s.server}\nTested: ${date.toLocaleTimeString()}`;
  };

  return (
    <header class="fcmd-topbar">
      <div class="fcmd-topbar-section">
        <span class="fcmd-logo">⚡ FLUX COMMAND</span>
        <span class="fcmd-live-badge"><span class="fcmd-live-dot" />LIVE</span>
        <span class="fcmd-threat-badge" style={{ color: store.threatLevel().color }}>
          THREAT: {store.threatLevel().text}
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
                {tz.l === "UTC" ? store.utc() : store.tzTime(tz.o)}
              </span>
            </div>
          ))}
        </div>
        <span
          class={`fcmd-net-monitor ${store.netMonitorEnabled() ? "active" : ""}`}
          onClick={() => store.setNetMonitorEnabled((v) => !v)}
          title={store.netMonitorEnabled() ? "Click to disable network monitoring" : "Click to enable network monitoring"}
        >
          <span class="fcmd-net-label">NET</span>
          <Show when={store.netMonitorEnabled() && store.netThroughput()}>
            <span class="fcmd-net-down">↓{formatBps(store.netThroughput()!.download_bytes_sec)}</span>
            <span class="fcmd-net-up">↑{formatBps(store.netThroughput()!.upload_bytes_sec)}</span>
          </Show>
          <Show when={store.netMonitorEnabled() && !store.netThroughput()}>
            <span class="fcmd-net-waiting">...</span>
          </Show>
          <Show when={!store.netMonitorEnabled()}>
            <span class="fcmd-net-off">OFF</span>
          </Show>
        </span>

        <span
          class={`fcmd-speedtest ${store.speedtestLoading() ? "loading" : ""}`}
          onClick={() => store.runSpeedtest()}
          title={speedtestTooltip()}
        >
          <Show when={store.speedtestLoading()}>
            <span class="fcmd-speedtest-label">⚡ TESTING...</span>
          </Show>
          <Show when={!store.speedtestLoading() && store.speedtest()}>
            <span class="fcmd-speedtest-label">⚡</span>
            <span class="fcmd-speedtest-down">↓{store.speedtest()!.download_mbps.toFixed(1)}</span>
            <span class="fcmd-speedtest-up">↑{store.speedtest()!.upload_mbps.toFixed(1)}</span>
            <span class="fcmd-speedtest-ping">•{store.speedtest()!.ping_ms.toFixed(0)}ms</span>
          </Show>
          <Show when={!store.speedtestLoading() && !store.speedtest()}>
            <span class="fcmd-speedtest-label">⚡ SPEED</span>
          </Show>
        </span>

        <span class="fcmd-close" onClick={() => props.onClose()}>✕</span>
      </div>
    </header>
  );
}
