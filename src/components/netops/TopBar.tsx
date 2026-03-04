import type { NetopsStore } from "./types";

interface Props {
  store: NetopsStore;
  onClose: () => void;
}

export default function TopBar(props: Props) {
  return (
    <header class="nops-topbar">
      <div class="nops-topbar-left">
        <span class="nops-logo">FLUX</span>
        <span class="nops-logo-sub">NETOPS</span>
        <span class="nops-version">v1.0</span>
        <span class="nops-live-badge">LIVE</span>
      </div>

      <div class="nops-topbar-center">
        <span
          class="nops-status-text"
          data-status={props.store.loading() ? "scanning" : props.store.error() ? "error" : props.store.result() ? "complete" : "ready"}
        >
          {props.store.statusText()}
        </span>
      </div>

      <div class="nops-topbar-right">
        <span class="nops-clock">{props.store.utc()} UTC</span>
        <button class="nops-close-btn" onClick={props.onClose} title="Close (Esc)">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    </header>
  );
}
