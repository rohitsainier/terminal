import { createSignal, Show } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import type { BharatLinkStore } from "./types";

interface Props {
  store: BharatLinkStore;
}

export default function InfoPanel(props: Props) {
  const [showAbout, setShowAbout] = createSignal(true);

  const copyNodeId = () => {
    const id = props.store.nodeInfo()?.node_id;
    if (id) navigator.clipboard.writeText(id);
  };

  const toggleSetting = (key: "auto_accept_text" | "accept_from_trusted_only" | "auto_accept_from_trusted") => {
    const current = props.store.settings();
    if (!current) return;
    const updated = { ...current, [key]: !current[key] };
    props.store.updateSettings(updated);
  };

  const changeDownloadDir = async () => {
    const current = props.store.settings();
    if (!current) return;
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: current.download_dir,
    });
    if (selected) {
      const updated = { ...current, download_dir: selected as string };
      props.store.updateSettings(updated);
    }
  };

  const totalSent = () =>
    props.store
      .history()
      .filter((h) => h.direction === "send" && h.status === "complete").length;

  const totalReceived = () =>
    props.store
      .history()
      .filter((h) => h.direction === "receive" && h.status === "complete").length;

  const totalBytes = () => {
    const bytes = props.store
      .history()
      .filter((h) => h.status === "complete")
      .reduce((sum, h) => sum + (h.file_size || 0), 0);
    if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)} MB`;
    return `${(bytes / 1073741824).toFixed(2)} GB`;
  };

  return (
    <div class="blnk-panel blnk-info-panel">
      {/* ─── About Section ─── */}
      <div class="blnk-panel-header blnk-about-header" onClick={() => setShowAbout(!showAbout())}>
        <span class="blnk-panel-icon">?</span>
        <span>ABOUT</span>
        <span class="blnk-collapse-icon">{showAbout() ? "▾" : "▸"}</span>
      </div>

      <Show when={showAbout()}>
        <div class="blnk-about-body">
          <div class="blnk-about-title">BharatLink</div>
          <div class="blnk-about-tagline">Sovereign P2P File & Text Sharing</div>

          <div class="blnk-about-desc">
            Indian AirDrop — share files and text directly between devices with no servers, no accounts, no cloud. Pure peer-to-peer over QUIC with end-to-end encryption.
          </div>

          <div class="blnk-about-section-label">FEATURES</div>
          <div class="blnk-about-features">
            <div class="blnk-about-feature">
              <span class="blnk-about-feat-icon">&#x1F4E1;</span>
              <div>
                <span class="blnk-about-feat-title">Auto Discovery</span>
                <span class="blnk-about-feat-desc">mDNS finds peers on your local network automatically</span>
              </div>
            </div>
            <div class="blnk-about-feature">
              <span class="blnk-about-feat-icon">&#x1F310;</span>
              <div>
                <span class="blnk-about-feat-title">Works Anywhere</span>
                <span class="blnk-about-feat-desc">NAT hole-punching + relay fallback for cross-network transfers</span>
              </div>
            </div>
            <div class="blnk-about-feature">
              <span class="blnk-about-feat-icon">&#x1F512;</span>
              <div>
                <span class="blnk-about-feat-title">E2E Encrypted</span>
                <span class="blnk-about-feat-desc">QUIC + TLS 1.3 — all data encrypted in transit</span>
              </div>
            </div>
            <div class="blnk-about-feature">
              <span class="blnk-about-feat-icon">&#x1F4C4;</span>
              <div>
                <span class="blnk-about-feat-title">File Transfer</span>
                <span class="blnk-about-feat-desc">Chunked, BLAKE3-verified, resumable file transfers</span>
              </div>
            </div>
            <div class="blnk-about-feature">
              <span class="blnk-about-feat-icon">&#x1F4AC;</span>
              <div>
                <span class="blnk-about-feat-title">Text Sharing</span>
                <span class="blnk-about-feat-desc">Instant text/snippet sharing between peers</span>
              </div>
            </div>
            <div class="blnk-about-feature">
              <span class="blnk-about-feat-icon">&#x2705;</span>
              <div>
                <span class="blnk-about-feat-title">Trusted Peers</span>
                <span class="blnk-about-feat-desc">Save peers with nicknames for quick access</span>
              </div>
            </div>
          </div>

          <div class="blnk-about-section-label">KEYBOARD SHORTCUTS</div>
          <div class="blnk-about-shortcuts">
            <div class="blnk-about-shortcut">
              <kbd>Cmd/Ctrl + Shift + B</kbd>
              <span>Open / close BharatLink</span>
            </div>
            <div class="blnk-about-shortcut">
              <kbd>Escape</kbd>
              <span>Close dashboard</span>
            </div>
          </div>

          <div class="blnk-about-section-label">HOW TO USE</div>
          <div class="blnk-about-steps">
            <div class="blnk-about-step">
              <span class="blnk-about-step-num">1</span>
              <span>Click <strong>START NODE</strong> to go online</span>
            </div>
            <div class="blnk-about-step">
              <span class="blnk-about-step-num">2</span>
              <span>Share your <strong>Endpoint ID</strong> with your friend</span>
            </div>
            <div class="blnk-about-step">
              <span class="blnk-about-step-num">3</span>
              <span>Add their ID via <strong>+ ADD PEER</strong></span>
            </div>
            <div class="blnk-about-step">
              <span class="blnk-about-step-num">4</span>
              <span>Select the peer and click <strong>SEND FILE</strong> or <strong>SEND TEXT</strong></span>
            </div>
            <div class="blnk-about-step">
              <span class="blnk-about-step-num">5</span>
              <span>Receiver sees the request and clicks <strong>ACCEPT</strong></span>
            </div>
          </div>

          <div class="blnk-about-tech">
            Powered by iroh QUIC + mDNS
          </div>
        </div>
      </Show>

      {/* ─── Node Info Section ─── */}
      <div class="blnk-panel-header">
        <span class="blnk-panel-icon">&#x2139;</span>
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
                <div
                  class="blnk-setting-row blnk-setting-toggle"
                  onClick={() => toggleSetting("auto_accept_text")}
                >
                  <span>Auto-accept text</span>
                  <span class={s().auto_accept_text ? "blnk-on" : "blnk-off"}>
                    {s().auto_accept_text ? "ON" : "OFF"}
                  </span>
                </div>
                <div
                  class="blnk-setting-row blnk-setting-toggle"
                  onClick={() => toggleSetting("accept_from_trusted_only")}
                >
                  <span>Trusted only</span>
                  <span class={s().accept_from_trusted_only ? "blnk-on" : "blnk-off"}>
                    {s().accept_from_trusted_only ? "ON" : "OFF"}
                  </span>
                </div>
                <div
                  class="blnk-setting-row blnk-setting-toggle"
                  onClick={() => toggleSetting("auto_accept_from_trusted")}
                >
                  <span>Auto-accept from trusted</span>
                  <span class={s().auto_accept_from_trusted ? "blnk-on" : "blnk-off"}>
                    {s().auto_accept_from_trusted ? "ON" : "OFF"}
                  </span>
                </div>
                <div
                  class="blnk-setting-row blnk-setting-toggle"
                  onClick={changeDownloadDir}
                >
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
