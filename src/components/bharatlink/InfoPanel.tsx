import { Show } from "solid-js";
import type { BharatLinkStore } from "./types";

interface Props {
  store: BharatLinkStore;
}

export default function InfoPanel(props: Props) {
  const copyNodeId = () => {
    const id = props.store.nodeInfo()?.node_id;
    if (id) navigator.clipboard.writeText(id);
  };

  const totalSent = () =>
    props.store
      .history()
      .filter((h) => h.direction === "send" && h.status === "completed").length;

  const totalReceived = () =>
    props.store
      .history()
      .filter((h) => h.direction === "receive" && h.status === "completed").length;

  const totalBytes = () => {
    const bytes = props.store
      .history()
      .filter((h) => h.status === "completed")
      .reduce((sum, h) => sum + (h.file_size || 0), 0);
    if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)} MB`;
    return `${(bytes / 1073741824).toFixed(2)} GB`;
  };

  return (
    <div class="blnk-panel blnk-info-panel">
      <div class="blnk-panel-header">
        <span class="blnk-panel-icon">ℹ</span>
        <span>NODE INFO</span>
      </div>

      <div class="blnk-panel-body">
        <Show
          when={props.store.nodeInfo()}
          fallback={<div class="blnk-empty">Node not started</div>}
        >
          {(info) => (
            <>
              {/* Node ID */}
              <div class="blnk-info-section">
                <div class="blnk-info-label">ENDPOINT ID</div>
                <div class="blnk-info-value blnk-info-copyable" onClick={copyNodeId}>
                  {info().node_id_short}
                  <span class="blnk-copy-hint">click to copy full ID</span>
                </div>
              </div>

              {/* Relay */}
              <Show when={info().relay_url}>
                <div class="blnk-info-section">
                  <div class="blnk-info-label">RELAY</div>
                  <div class="blnk-info-value">{info().relay_url}</div>
                </div>
              </Show>

              {/* Local addresses */}
              <Show when={info().local_addrs.length > 0}>
                <div class="blnk-info-section">
                  <div class="blnk-info-label">LOCAL ADDRESSES</div>
                  {info().local_addrs.map((addr) => (
                    <div class="blnk-info-value blnk-info-addr">{addr}</div>
                  ))}
                </div>
              </Show>
            </>
          )}
        </Show>

        {/* Stats */}
        <div class="blnk-info-section">
          <div class="blnk-info-label">STATISTICS</div>
          <div class="blnk-stats-grid">
            <div class="blnk-stat">
              <span class="blnk-stat-value">{totalSent()}</span>
              <span class="blnk-stat-label">Sent</span>
            </div>
            <div class="blnk-stat">
              <span class="blnk-stat-value">{totalReceived()}</span>
              <span class="blnk-stat-label">Received</span>
            </div>
            <div class="blnk-stat">
              <span class="blnk-stat-value">{totalBytes()}</span>
              <span class="blnk-stat-label">Total</span>
            </div>
          </div>
        </div>

        {/* Quick actions */}
        <div class="blnk-info-section">
          <div class="blnk-info-label">ACTIONS</div>
          <button
            class="blnk-btn blnk-btn-full"
            onClick={() => props.store.refreshHistory()}
          >
            REFRESH HISTORY
          </button>
          <button
            class="blnk-btn blnk-btn-full blnk-btn-dim"
            onClick={() => props.store.clearHistory()}
          >
            CLEAR HISTORY
          </button>
        </div>

        {/* Settings summary */}
        <Show when={props.store.settings()}>
          {(s) => (
            <div class="blnk-info-section">
              <div class="blnk-info-label">SETTINGS</div>
              <div class="blnk-settings-list">
                <div class="blnk-setting-row">
                  <span>Auto-accept text</span>
                  <span class={s().auto_accept_text ? "blnk-on" : "blnk-off"}>
                    {s().auto_accept_text ? "ON" : "OFF"}
                  </span>
                </div>
                <div class="blnk-setting-row">
                  <span>Trusted only</span>
                  <span class={s().accept_from_trusted_only ? "blnk-on" : "blnk-off"}>
                    {s().accept_from_trusted_only ? "ON" : "OFF"}
                  </span>
                </div>
                <div class="blnk-setting-row">
                  <span>Download dir</span>
                  <span class="blnk-setting-dir">{s().download_dir}</span>
                </div>
              </div>
            </div>
          )}
        </Show>

        {/* Error display */}
        <Show when={props.store.error()}>
          <div class="blnk-error-box">
            {props.store.error()}
          </div>
        </Show>
      </div>
    </div>
  );
}
