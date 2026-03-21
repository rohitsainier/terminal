import { Show } from "solid-js";
import type { BharatLinkStore } from "./types";

interface Props {
  store: BharatLinkStore;
  onClose: () => void;
}

export default function TopBar(props: Props) {
  const deviceName = () => props.store.settings()?.device_name || "";

  return (
    <div class="blnk-topbar">
      <div class="blnk-topbar-left">
        <span class="blnk-logo">FLUX</span>
        <span class="blnk-badge">BHARATLINK</span>
        <div
          class="blnk-status-dot"
          classList={{ "blnk-status-online": props.store.isRunning() }}
        />
        <span class="blnk-status-label">
          {props.store.isRunning() ? "ONLINE" : "OFFLINE"}
        </span>
        <Show when={deviceName()}>
          <span class="blnk-device-name">{deviceName()}</span>
        </Show>
      </div>

      <div class="blnk-topbar-center">
        <span class="blnk-status-text">{props.store.statusText()}</span>
      </div>

      <div class="blnk-topbar-right">
        {!props.store.isRunning() ? (
          <button
            class="blnk-btn blnk-btn-start"
            onClick={() => props.store.startNode()}
            disabled={props.store.loading()}
          >
            START NODE
          </button>
        ) : (
          <button
            class="blnk-btn blnk-btn-stop"
            onClick={() => props.store.stopNode()}
            disabled={props.store.loading()}
          >
            STOP
          </button>
        )}
        <span class="blnk-utc">{props.store.utc()}</span>
        <span class="blnk-close" onClick={() => props.onClose()}>
          ✕
        </span>
      </div>
    </div>
  );
}
