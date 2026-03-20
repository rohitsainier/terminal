import { createSignal, For, Show } from "solid-js";
import type { BharatLinkStore } from "./types";

interface Props {
  store: BharatLinkStore;
}

export default function PeerPanel(props: Props) {
  const [addInput, setAddInput] = createSignal("");
  const [nicknameInput, setNicknameInput] = createSignal("");
  const [showAddForm, setShowAddForm] = createSignal(false);

  const localPeers = () =>
    props.store.peers().filter((p) => !p.is_trusted);
  const trustedPeers = () =>
    props.store.peers().filter((p) => p.is_trusted);

  const onlinePeers = () => props.store.peers().filter((p) => p.is_connected).length;

  const handleAdd = async () => {
    const id = addInput().trim();
    if (!id) return;
    await props.store.addPeer(id, nicknameInput().trim() || undefined);
    setAddInput("");
    setNicknameInput("");
    setShowAddForm(false);
  };

  const formatLastSeen = (ts: number | null, isConnected: boolean) => {
    if (isConnected) return "online";
    if (!ts) return "offline";
    const diff = Math.floor((Date.now() - ts) / 1000);
    if (diff < 0) return "just now";
    if (diff < 60) return `${diff}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return "offline";
  };

  return (
    <div class="blnk-panel blnk-peer-panel">
      <div class="blnk-panel-header">
        <span class="blnk-panel-icon">📡</span>
        <span>PEERS</span>
        <Show when={props.store.isRunning()}>
          <span class="blnk-peer-count">
            {onlinePeers()} online
          </span>
        </Show>
        <button
          class="blnk-btn-icon"
          onClick={() => props.store.refreshPeers()}
          title="Refresh peers"
        >
          ↻
        </button>
      </div>

      <div class="blnk-panel-body">
        {/* Trusted peers */}
        <Show when={trustedPeers().length > 0}>
          <div class="blnk-peer-section-label">TRUSTED</div>
          <For each={trustedPeers()}>
            {(peer) => (
              <div
                class="blnk-peer-row"
                classList={{
                  "blnk-peer-selected":
                    props.store.selectedPeer() === peer.node_id,
                  "blnk-peer-online": peer.is_connected,
                  "blnk-peer-offline": !peer.is_connected,
                }}
                onClick={() => props.store.setSelectedPeer(peer.node_id)}
              >
                <span
                  class="blnk-peer-status-dot"
                  classList={{
                    "blnk-dot-online": peer.is_connected,
                    "blnk-dot-offline": !peer.is_connected,
                  }}
                />
                <div class="blnk-peer-info">
                  <span class="blnk-peer-name">
                    {peer.nickname || peer.node_id_short}
                  </span>
                  <span
                    class="blnk-peer-meta"
                    classList={{
                      "blnk-meta-online": peer.is_connected,
                    }}
                  >
                    {formatLastSeen(peer.last_seen, peer.is_connected)}
                  </span>
                </div>
                <button
                  class="blnk-btn-icon blnk-btn-untrust"
                  onClick={(e) => {
                    e.stopPropagation();
                    props.store.untrustPeer(peer.node_id);
                  }}
                  title="Remove trust"
                >
                  ×
                </button>
              </div>
            )}
          </For>
        </Show>

        {/* Discovered peers */}
        <div class="blnk-peer-section-label">DISCOVERED</div>
        <Show
          when={localPeers().length > 0}
          fallback={
            <div class="blnk-empty">
              {props.store.isRunning()
                ? "Scanning for peers..."
                : "Start node to discover peers"}
            </div>
          }
        >
          <For each={localPeers()}>
            {(peer) => (
              <div
                class="blnk-peer-row"
                classList={{
                  "blnk-peer-selected":
                    props.store.selectedPeer() === peer.node_id,
                  "blnk-peer-online": peer.is_connected,
                  "blnk-peer-offline": !peer.is_connected,
                }}
                onClick={() => props.store.setSelectedPeer(peer.node_id)}
              >
                <span
                  class="blnk-peer-status-dot"
                  classList={{
                    "blnk-dot-online": peer.is_connected,
                    "blnk-dot-offline": !peer.is_connected,
                  }}
                />
                <div class="blnk-peer-info">
                  <span class="blnk-peer-name">
                    {peer.nickname || peer.node_id_short}
                  </span>
                  <span
                    class="blnk-peer-meta"
                    classList={{
                      "blnk-meta-online": peer.is_connected,
                    }}
                  >
                    {formatLastSeen(peer.last_seen, peer.is_connected)}
                  </span>
                </div>
                <button
                  class="blnk-btn-icon"
                  onClick={(e) => {
                    e.stopPropagation();
                    const name = prompt("Nickname for this peer:");
                    if (name)
                      props.store.trustPeer(peer.node_id, name);
                  }}
                  title="Trust peer"
                >
                  ♡
                </button>
              </div>
            )}
          </For>
        </Show>
      </div>

      {/* Add peer form */}
      <div class="blnk-panel-footer">
        <Show
          when={showAddForm()}
          fallback={
            <button
              class="blnk-btn blnk-btn-add"
              onClick={() => setShowAddForm(true)}
              disabled={!props.store.isRunning()}
            >
              + ADD PEER
            </button>
          }
        >
          <div class="blnk-add-form">
            <input
              class="blnk-input"
              placeholder="Node ID..."
              value={addInput()}
              onInput={(e) => setAddInput(e.currentTarget.value)}
            />
            <input
              class="blnk-input"
              placeholder="Nickname (optional)"
              value={nicknameInput()}
              onInput={(e) => setNicknameInput(e.currentTarget.value)}
            />
            <div class="blnk-add-actions">
              <button class="blnk-btn" onClick={handleAdd}>
                ADD
              </button>
              <button
                class="blnk-btn blnk-btn-dim"
                onClick={() => setShowAddForm(false)}
              >
                CANCEL
              </button>
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
}
