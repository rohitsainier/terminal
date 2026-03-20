import { createSignal, For, Show } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import type { BharatLinkStore } from "./types";

interface Props {
  store: BharatLinkStore;
}

export default function TransferPanel(props: Props) {
  const [textInput, setTextInput] = createSignal("");
  const [showTextInput, setShowTextInput] = createSignal(false);

  const selectedPeerName = () => {
    const id = props.store.selectedPeer();
    if (!id) return null;
    const peer = props.store.peers().find((p) => p.node_id === id);
    return peer?.nickname || peer?.node_id_short || id.slice(0, 12);
  };

  const handleSendFile = async () => {
    const peerId = props.store.selectedPeer();
    if (!peerId) return;
    const selected = await open({
      multiple: false,
      directory: false,
    });
    if (selected) {
      await props.store.sendFile(peerId, selected as string);
    }
  };

  const handleSendText = async () => {
    const peerId = props.store.selectedPeer();
    const text = textInput().trim();
    if (!peerId || !text) return;
    await props.store.sendText(peerId, text);
    setTextInput("");
    setShowTextInput(false);
  };

  const formatBytes = (bytes: number | null) => {
    if (!bytes) return "0 B";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)} MB`;
    return `${(bytes / 1073741824).toFixed(2)} GB`;
  };

  const formatSpeed = (bps: number) => {
    if (bps < 1024) return `${bps} B/s`;
    if (bps < 1048576) return `${(bps / 1024).toFixed(1)} KB/s`;
    return `${(bps / 1048576).toFixed(1)} MB/s`;
  };

  const formatTime = (ts: number) => {
    const d = new Date(ts);
    return d.toLocaleTimeString();
  };

  const recentHistory = () => props.store.history().slice(0, 10);

  return (
    <div class="blnk-panel blnk-transfer-panel">
      <div class="blnk-panel-header">
        <span class="blnk-panel-icon">⇄</span>
        <span>TRANSFERS</span>
      </div>

      <div class="blnk-panel-body">
        {/* Send controls */}
        <Show when={props.store.selectedPeer()}>
          <div class="blnk-send-section">
            <div class="blnk-send-target">
              TO: <span class="blnk-send-target-name">{selectedPeerName()}</span>
            </div>
            <div class="blnk-send-actions">
              <button
                class="blnk-btn blnk-btn-send"
                onClick={handleSendFile}
                disabled={props.store.loading()}
              >
                SEND FILE
              </button>
              <button
                class="blnk-btn blnk-btn-send"
                onClick={() => setShowTextInput(!showTextInput())}
                disabled={props.store.loading()}
              >
                SEND TEXT
              </button>
            </div>
            <Show when={showTextInput()}>
              <div class="blnk-text-input-area">
                <textarea
                  class="blnk-textarea"
                  placeholder="Type message to send..."
                  value={textInput()}
                  onInput={(e) => setTextInput(e.currentTarget.value)}
                  rows={3}
                />
                <button
                  class="blnk-btn blnk-btn-send"
                  onClick={handleSendText}
                  disabled={!textInput().trim()}
                >
                  SEND
                </button>
              </div>
            </Show>
          </div>
        </Show>

        <Show when={!props.store.selectedPeer()}>
          <div class="blnk-empty">Select a peer to send files or text</div>
        </Show>

        {/* Incoming requests */}
        <Show when={props.store.pendingRequests().length > 0}>
          <div class="blnk-section-label">INCOMING REQUESTS</div>
          <For each={props.store.pendingRequests()}>
            {(req) => (
              <div class="blnk-request-row">
                <div class="blnk-request-info">
                  <span class="blnk-request-from">
                    {req.from_nickname || req.from_peer.slice(0, 12)}
                  </span>
                  <span class="blnk-request-type">
                    {req.transfer_type === "file"
                      ? `File: ${req.filename} (${formatBytes(req.file_size)})`
                      : `Text: ${req.text_preview || "..."}`}
                  </span>
                </div>
                <div class="blnk-request-actions">
                  <button
                    class="blnk-btn blnk-btn-accept"
                    onClick={() => props.store.acceptTransfer(req.id)}
                  >
                    ACCEPT
                  </button>
                  <button
                    class="blnk-btn blnk-btn-reject"
                    onClick={() => props.store.rejectTransfer(req.id)}
                  >
                    REJECT
                  </button>
                </div>
              </div>
            )}
          </For>
        </Show>

        {/* Active transfers */}
        <Show when={props.store.activeTransfers().length > 0}>
          <div class="blnk-section-label">ACTIVE TRANSFERS</div>
          <For each={props.store.activeTransfers()}>
            {(t) => (
              <div class="blnk-transfer-row">
                <div class="blnk-transfer-info">
                  <span class="blnk-transfer-name">
                    {t.direction === "send" ? "↑" : "↓"}{" "}
                    {t.filename || "Text"}
                  </span>
                  <span class="blnk-transfer-speed">
                    {formatBytes(t.bytes_transferred)} / {formatBytes(t.total_bytes)}{" "}
                    · {formatSpeed(t.speed_bps)}
                  </span>
                </div>
                <div class="blnk-progress-bar">
                  <div
                    class="blnk-progress-fill"
                    style={{ width: `${t.percent}%` }}
                  />
                </div>
                <span class="blnk-transfer-percent">
                  {t.percent.toFixed(0)}%
                </span>
                <button
                  class="blnk-btn-icon blnk-btn-cancel"
                  onClick={() => props.store.cancelTransfer(t.transfer_id)}
                  title="Cancel"
                >
                  ✕
                </button>
              </div>
            )}
          </For>
        </Show>

        {/* Recent history */}
        <div class="blnk-section-label">RECENT</div>
        <Show
          when={recentHistory().length > 0}
          fallback={<div class="blnk-empty">No transfer history</div>}
        >
          <For each={recentHistory()}>
            {(entry) => (
              <div
                class="blnk-history-row"
                classList={{
                  "blnk-history-failed": entry.status === "failed",
                  "blnk-history-cancelled": entry.status === "cancelled",
                }}
              >
                <span class="blnk-history-icon">
                  {entry.direction === "send" ? "↑" : "↓"}
                </span>
                <div class="blnk-history-info">
                  <span class="blnk-history-name">
                    {entry.transfer_type === "file"
                      ? entry.filename || "File"
                      : "Text message"}
                  </span>
                  <span class="blnk-history-meta">
                    {entry.peer_nickname || entry.peer_id.slice(0, 12)} ·{" "}
                    {formatTime(entry.timestamp)} ·{" "}
                    {entry.status}
                  </span>
                  <Show when={entry.transfer_type === "text" && entry.text_content}>
                    <span class="blnk-history-text-preview">
                      {entry.text_content}
                    </span>
                  </Show>
                  <Show when={entry.save_path}>
                    <span class="blnk-history-save-path" title={entry.save_path || ""}>
                      Saved: {entry.save_path?.split("/").pop() || entry.save_path}
                    </span>
                  </Show>
                </div>
                <Show when={entry.file_size}>
                  <span class="blnk-history-size">
                    {formatBytes(entry.file_size!)}
                  </span>
                </Show>
              </div>
            )}
          </For>
        </Show>
      </div>
    </div>
  );
}
