import { createSignal, createMemo, createEffect, For, Show } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  BharatLinkStore,
  TransferHistoryEntry,
  TransferRequest,
  TransferProgress,
} from "./types";

// ─── Chat item union ────────────────────────────────────────────────
type ChatItem =
  | { kind: "history"; entry: TransferHistoryEntry; ts: number }
  | { kind: "request"; entry: TransferRequest; ts: number }
  | { kind: "active"; entry: TransferProgress; ts: number };

interface Props {
  store: BharatLinkStore;
}

export default function TransferPanel(props: Props) {
  const [textInput, setTextInput] = createSignal("");
  let messagesRef: HTMLDivElement | undefined;
  let wasNearBottom = true;

  // ─── Selected peer name ───────────────────────────────────────────
  const selectedPeerName = () => {
    const id = props.store.selectedPeer();
    if (!id) return null;
    const peer = props.store.peers().find((p) => p.node_id === id);
    return peer?.nickname || peer?.node_id_short || id.slice(0, 12);
  };

  // ─── Unified chat timeline ────────────────────────────────────────
  const chatItems = createMemo<ChatItem[]>(() => {
    const peerId = props.store.selectedPeer();
    const items: ChatItem[] = [];

    // History entries for this peer
    const hist = peerId
      ? props.store.history().filter((h) => h.peer_id === peerId)
      : [];
    for (const entry of hist) {
      items.push({ kind: "history", entry, ts: entry.timestamp });
    }

    // Pending incoming requests for this peer
    const requests = peerId
      ? props.store.pendingRequests().filter((r) => r.from_peer === peerId)
      : props.store.pendingRequests();
    for (const entry of requests) {
      items.push({ kind: "request", entry, ts: entry.timestamp });
    }

    // Active transfers (no peer_id available — show all)
    for (const entry of props.store.activeTransfers()) {
      items.push({ kind: "active", entry, ts: Date.now() });
    }

    // Sort ascending (oldest first, newest at bottom)
    items.sort((a, b) => a.ts - b.ts);
    return items;
  });

  // ─── Auto-scroll ──────────────────────────────────────────────────
  const isNearBottom = () => {
    if (!messagesRef) return true;
    const { scrollTop, scrollHeight, clientHeight } = messagesRef;
    return scrollHeight - scrollTop - clientHeight < 100;
  };

  createEffect(() => {
    chatItems(); // track dependency
    wasNearBottom = isNearBottom();
    setTimeout(() => {
      if (messagesRef && wasNearBottom) {
        messagesRef.scrollTop = messagesRef.scrollHeight;
      }
    }, 0);
  });

  // ─── Actions ──────────────────────────────────────────────────────
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
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      e.stopPropagation();
      handleSendText();
    }
  };

  // ─── Formatters ───────────────────────────────────────────────────
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

  const formatChatTime = (ts: number): string => {
    const d = new Date(ts);
    const now = new Date();
    const isToday = d.toDateString() === now.toDateString();
    if (isToday) {
      return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    }
    return (
      d.toLocaleDateString([], { month: "short", day: "numeric" }) +
      " " +
      d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
    );
  };

  // ─── Render helpers ───────────────────────────────────────────────

  const renderHistoryItem = (entry: TransferHistoryEntry) => {
    const isSent = entry.direction === "send";
    const isText = entry.transfer_type === "text";
    const isFailed = entry.status === "failed";
    const isCancelled = entry.status === "cancelled";

    if (isText) {
      return (
        <div
          class="blnk-chat-row"
          classList={{
            "blnk-chat-row-sent": isSent,
            "blnk-chat-row-received": !isSent,
          }}
        >
          <div
            class="blnk-chat-bubble"
            classList={{
              "blnk-chat-sent": isSent,
              "blnk-chat-received": !isSent,
            }}
          >
            <div class="blnk-chat-bubble-text">{entry.text_content || ""}</div>
            <Show when={isFailed}>
              <span class="blnk-chat-status-badge blnk-chat-status-failed">
                Failed
              </span>
            </Show>
          </div>
          <span class="blnk-chat-time">{formatChatTime(entry.timestamp)}</span>
        </div>
      );
    }

    // File transfer
    return (
      <div
        class="blnk-chat-row"
        classList={{
          "blnk-chat-row-sent": isSent,
          "blnk-chat-row-received": !isSent,
        }}
      >
        <div
          class="blnk-chat-file-card"
          classList={{
            "blnk-chat-file-sent": isSent,
            "blnk-chat-file-received": !isSent,
            "blnk-chat-file-failed": isFailed || isCancelled,
          }}
        >
          <div class="blnk-chat-file-header">
            <span class="blnk-chat-file-icon">
              {isSent ? "\u2191" : "\u2193"}
            </span>
            <span class="blnk-chat-file-name">
              {entry.filename || "File"}
            </span>
          </div>
          <div class="blnk-chat-file-meta">
            {formatBytes(entry.file_size)} · {entry.status}
          </div>
          <Show when={entry.save_path && !isFailed}>
            <div class="blnk-chat-file-save">
              Saved: {entry.save_path?.split("/").pop() || entry.save_path}
            </div>
          </Show>
          <Show when={isFailed && entry.save_path?.startsWith("Error:")}>
            <div class="blnk-chat-file-error">
              {entry.save_path}
            </div>
          </Show>
        </div>
        <span class="blnk-chat-time">{formatChatTime(entry.timestamp)}</span>
      </div>
    );
  };

  const renderRequestItem = (req: TransferRequest) => {
    return (
      <div class="blnk-chat-row blnk-chat-row-received">
        <div class="blnk-chat-request-card">
          <div class="blnk-chat-request-label">INCOMING REQUEST</div>
          <div class="blnk-chat-request-from">
            {req.from_nickname || req.from_peer.slice(0, 12)}
          </div>
          <div class="blnk-chat-request-info">
            {req.transfer_type === "file"
              ? `${req.filename || "File"} (${formatBytes(req.file_size)})`
              : `Text: ${req.text_preview || "..."}`}
          </div>
          <div class="blnk-chat-request-actions">
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
        <span class="blnk-chat-time">{formatChatTime(req.timestamp)}</span>
      </div>
    );
  };

  const renderActiveTransfer = (t: TransferProgress) => {
    const isSent = t.direction === "send";
    return (
      <div
        class="blnk-chat-row"
        classList={{
          "blnk-chat-row-sent": isSent,
          "blnk-chat-row-received": !isSent,
        }}
      >
        <div class="blnk-chat-transfer-card">
          <div class="blnk-chat-transfer-info">
            <span>
              {isSent ? "\u2191" : "\u2193"} {t.filename || "Text"}
            </span>
            <button
              class="blnk-btn-icon blnk-btn-cancel"
              onClick={() => props.store.cancelTransfer(t.transfer_id)}
              title="Cancel"
            >
              &#x2715;
            </button>
          </div>
          <div class="blnk-progress-bar">
            <div
              class="blnk-progress-fill"
              style={{ width: `${t.percent}%` }}
            />
          </div>
          <div class="blnk-chat-transfer-stats">
            <span>
              {formatBytes(t.bytes_transferred)} / {formatBytes(t.total_bytes)}
            </span>
            <span>
              {t.percent.toFixed(0)}% · {formatSpeed(t.speed_bps)}
            </span>
          </div>
        </div>
      </div>
    );
  };

  const renderChatItem = (item: ChatItem) => {
    switch (item.kind) {
      case "history":
        return renderHistoryItem(item.entry);
      case "request":
        return renderRequestItem(item.entry);
      case "active":
        return renderActiveTransfer(item.entry);
    }
  };

  // ─── Component ────────────────────────────────────────────────────
  return (
    <div class="blnk-panel blnk-chat-panel">
      {/* Header */}
      <div class="blnk-panel-header">
        <span class="blnk-panel-icon">{"\uD83D\uDCAC"}</span>
        <span>CHAT</span>
        <Show when={selectedPeerName()}>
          <span class="blnk-chat-peer-label">
            · {selectedPeerName()}
          </span>
        </Show>
      </div>

      {/* Messages area */}
      <div class="blnk-chat-messages" ref={messagesRef}>
        <Show
          when={props.store.selectedPeer()}
          fallback={
            <div class="blnk-chat-empty">
              Select a peer to start chatting
            </div>
          }
        >
          <Show
            when={chatItems().length > 0}
            fallback={
              <div class="blnk-chat-empty">
                No messages yet. Say hello!
              </div>
            }
          >
            <For each={chatItems()}>{(item) => renderChatItem(item)}</For>
          </Show>
        </Show>
      </div>

      {/* Input bar */}
      <Show when={props.store.selectedPeer()}>
        <div class="blnk-chat-input-bar">
          <button
            class="blnk-chat-attach-btn"
            onClick={handleSendFile}
            disabled={props.store.loading()}
            title="Attach file"
          >
            {"\uD83D\uDCCE"}
          </button>
          <input
            type="text"
            class="blnk-chat-input"
            placeholder="Type a message..."
            value={textInput()}
            onInput={(e) => setTextInput(e.currentTarget.value)}
            onKeyDown={handleKeyDown}
            disabled={props.store.loading()}
          />
          <button
            class="blnk-chat-send-btn"
            onClick={handleSendText}
            disabled={!textInput().trim() || props.store.loading()}
            title="Send"
          >
            {"\u27A4"}
          </button>
        </div>
      </Show>
    </div>
  );
}
