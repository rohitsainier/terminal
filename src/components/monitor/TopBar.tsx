import type { MonitorStore } from "./types";

interface TopBarProps {
  store: MonitorStore;
  onClose: () => void;
}

export default function TopBar(props: TopBarProps) {
  const { store } = props;

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
        <span class="fcmd-packets">PKT: {store.packetCount().toLocaleString()}</span>
        <span class="fcmd-close" onClick={() => props.onClose()}>✕</span>
      </div>
    </header>
  );
}
