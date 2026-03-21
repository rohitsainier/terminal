import { createSignal, createMemo, createEffect, onMount, onCleanup, For, Show } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
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

// ─── Image file detection ───────────────────────────────────────────
const IMAGE_EXTS = new Set(["png", "jpg", "jpeg", "gif", "bmp", "webp", "svg", "ico", "tiff"]);
const isImageFile = (filename: string | null): boolean => {
  if (!filename) return false;
  const ext = filename.split(".").pop()?.toLowerCase() || "";
  return IMAGE_EXTS.has(ext);
};

// ─── URL detection for link previews ────────────────────────────────
const URL_REGEX = /https?:\/\/[^\s<>"{}|\\^`[\]]+/g;
const extractUrls = (text: string): string[] => {
  return text.match(URL_REGEX) || [];
};

const getDomain = (url: string): string => {
  try {
    return new URL(url).hostname;
  } catch {
    return url;
  }
};

interface Props {
  store: BharatLinkStore;
}

export default function TransferPanel(props: Props) {
  const [textInput, setTextInput] = createSignal("");
  const [isDragging, setIsDragging] = createSignal(false);
  let messagesRef: HTMLDivElement | undefined;
  let wasNearBottom = true;
  let dragCounter = 0;
  let typingTimeout: ReturnType<typeof setTimeout> | undefined;
  let lastTypingSent = 0;

  // ─── Tauri drag-drop listener for native file paths ─────────────
  let unlistenDrop: UnlistenFn | undefined;

  onMount(async () => {
    unlistenDrop = await listen<{ paths: string[] }>("tauri://drag-drop", (e) => {
      setIsDragging(false);
      dragCounter = 0;
      const peerId = props.store.selectedPeer();
      if (!peerId || !e.payload.paths?.length) return;
      if (e.payload.paths.length === 1) {
        props.store.sendFile(peerId, e.payload.paths[0]);
      } else {
        props.store.sendFiles(peerId, e.payload.paths);
      }
    });
  });

  onCleanup(() => {
    unlistenDrop?.();
    if (typingTimeout) clearTimeout(typingTimeout);
  });

  const handleDragEnter = (e: DragEvent) => {
    e.preventDefault();
    dragCounter++;
    setIsDragging(true);
  };

  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    dragCounter--;
    if (dragCounter <= 0) {
      setIsDragging(false);
      dragCounter = 0;
    }
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
  };

  // ─── Selected peer name ───────────────────────────────────────────
  const selectedPeerName = () => {
    const id = props.store.selectedPeer();
    if (!id) return null;
    const peer = props.store.peers().find((p) => p.node_id === id);
    return peer?.nickname || peer?.node_id_short || id.slice(0, 12);
  };

  // ─── Typing indicator for selected peer ───────────────────────────
  const isSelectedPeerTyping = () => {
    const peerId = props.store.selectedPeer();
    if (!peerId) return false;
    return props.store.typingPeers().has(peerId);
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

    // Sort ascending (oldest first, newest at bottom)
    // NOTE: Active transfers are rendered separately below the list
    // to avoid full list re-renders on every progress event
    items.sort((a, b) => a.ts - b.ts);
    return items;
  });

  // ─── Auto-scroll ──────────────────────────────────────────────────
  const scrollToBottom = () => {
    if (messagesRef) {
      messagesRef.scrollTop = messagesRef.scrollHeight;
    }
  };

  // Track previous item count to detect new messages
  let prevItemCount = 0;
  let prevActiveCount = 0;

  // Auto-scroll when new chat items arrive
  createEffect(() => {
    const items = chatItems();
    const count = items.length;
    if (count > prevItemCount) {
      // New message arrived — scroll to bottom
      setTimeout(scrollToBottom, 0);
    }
    prevItemCount = count;
  });

  // Scroll once when a new active transfer appears (not on every tick)
  createEffect(() => {
    const count = props.store.activeTransfers().length;
    if (count > prevActiveCount) {
      setTimeout(scrollToBottom, 0);
    }
    prevActiveCount = count;
  });

  // ─── Actions ──────────────────────────────────────────────────────
  const handleSendFile = async () => {
    const peerId = props.store.selectedPeer();
    if (!peerId) return;
    const selected = await open({
      multiple: true,
      directory: false,
    });
    if (selected) {
      if (Array.isArray(selected)) {
        if (selected.length === 1) {
          await props.store.sendFile(peerId, selected[0]);
        } else if (selected.length > 1) {
          await props.store.sendFiles(peerId, selected);
        }
      } else {
        await props.store.sendFile(peerId, selected as string);
      }
    }
  };

  const handleSendFolder = async () => {
    const peerId = props.store.selectedPeer();
    if (!peerId) return;
    const selected = await open({
      multiple: false,
      directory: true,
    });
    if (selected) {
      try {
        const files = await invoke<string[]>("bharatlink_list_dir_files", { dirPath: selected as string });
        if (files.length > 0) {
          await props.store.sendFiles(peerId, files);
        }
      } catch (e) {
        console.error("Failed to list dir files:", e);
      }
    }
  };

  const handleSendScreenshot = async () => {
    const peerId = props.store.selectedPeer();
    if (!peerId) return;
    await props.store.captureAndSendScreenshot(peerId);
  };

  const handleSendClipboard = async () => {
    const peerId = props.store.selectedPeer();
    if (!peerId) return;
    try {
      const text = await navigator.clipboard.readText();
      if (text) {
        await props.store.sendClipboard(peerId, text);
      }
    } catch (e) {
      console.error("Clipboard read failed:", e);
    }
  };

  const handleSendText = async () => {
    const peerId = props.store.selectedPeer();
    const text = textInput().trim();
    if (!peerId || !text) return;
    // Send stop_typing signal
    props.store.sendSignal(peerId, "stop_typing");
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

  const handleTextInput = (e: InputEvent) => {
    const target = e.currentTarget as HTMLInputElement;
    setTextInput(target.value);

    // Send typing signal (throttled to once per 2 seconds)
    const peerId = props.store.selectedPeer();
    if (peerId && target.value.trim()) {
      const now = Date.now();
      if (now - lastTypingSent > 2000) {
        lastTypingSent = now;
        props.store.sendSignal(peerId, "typing");
      }

      // Clear previous timeout, set new one for stop_typing
      if (typingTimeout) clearTimeout(typingTimeout);
      typingTimeout = setTimeout(() => {
        if (peerId) props.store.sendSignal(peerId, "stop_typing");
      }, 3000);
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

  /** Render text with inline link previews */
  const renderTextContent = (text: string) => {
    const urls = extractUrls(text);
    if (urls.length === 0) {
      return <div class="blnk-chat-bubble-text">{text}</div>;
    }

    // Split text around URLs and render inline
    const parts: (string | { url: string })[] = [];
    let remaining = text;
    for (const url of urls) {
      const idx = remaining.indexOf(url);
      if (idx > 0) parts.push(remaining.slice(0, idx));
      parts.push({ url });
      remaining = remaining.slice(idx + url.length);
    }
    if (remaining) parts.push(remaining);

    return (
      <>
        <div class="blnk-chat-bubble-text">
          {parts.map((p) =>
            typeof p === "string" ? (
              p
            ) : (
              <a
                class="blnk-chat-link"
                href={p.url}
                target="_blank"
                rel="noopener noreferrer"
              >
                {p.url}
              </a>
            )
          )}
        </div>
        {/* Link preview cards */}
        {urls.slice(0, 2).map((url) => (
          <div class="blnk-link-preview">
            <span class="blnk-link-preview-icon">{"\uD83D\uDD17"}</span>
            <div class="blnk-link-preview-content">
              <span class="blnk-link-preview-domain">{getDomain(url)}</span>
              <span class="blnk-link-preview-url">{url.length > 60 ? url.slice(0, 60) + "..." : url}</span>
            </div>
          </div>
        ))}
      </>
    );
  };

  const renderHistoryItem = (entry: TransferHistoryEntry) => {
    const isSent = entry.direction === "send";
    const isText = entry.transfer_type === "text" || entry.transfer_type === "clipboard";
    const isClipboard = entry.transfer_type === "clipboard";
    const isFailed = entry.status === "failed";
    const isDelivered = isSent && props.store.deliveredMessages().has(entry.id);

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
              "blnk-chat-clipboard": isClipboard,
            }}
          >
            <Show when={isClipboard}>
              <span class="blnk-clipboard-badge">{"\uD83D\uDCCB"} Clipboard</span>
            </Show>
            {renderTextContent(entry.text_content || "")}
            <Show when={isFailed}>
              <span class="blnk-chat-status-badge blnk-chat-status-failed">
                Failed
              </span>
            </Show>
          </div>
          <div class="blnk-chat-time-row">
            <span class="blnk-chat-time">{formatChatTime(entry.timestamp)}</span>
            <Show when={isSent}>
              <span class="blnk-chat-receipt" classList={{ "blnk-receipt-delivered": isDelivered }}>
                {isDelivered ? "\u2713\u2713" : "\u2713"}
              </span>
            </Show>
          </div>
        </div>
      );
    }

    // File transfer
    const isImage = isImageFile(entry.filename);
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
            "blnk-chat-file-failed": isFailed || entry.status === "cancelled",
          }}
        >
          {/* Image preview for received images with save_path */}
          <Show when={isImage && entry.save_path && !isFailed}>
            <div class="blnk-chat-image-preview">
              <img
                src={convertFileSrc(entry.save_path!)}
                alt={entry.filename || "Image"}
                class="blnk-chat-image"
                loading="lazy"
                onError={(e) => {
                  // Hide broken image, show file card instead
                  (e.currentTarget as HTMLImageElement).style.display = "none";
                }}
              />
            </div>
          </Show>
          <div class="blnk-chat-file-header">
            <span class="blnk-chat-file-icon">
              {isImage ? "\uD83D\uDDBC\uFE0F" : isSent ? "\u2191" : "\u2193"}
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
        <div class="blnk-chat-time-row">
          <span class="blnk-chat-time">{formatChatTime(entry.timestamp)}</span>
          <Show when={isSent}>
            <span class="blnk-chat-receipt" classList={{ "blnk-receipt-delivered": isDelivered }}>
              {isDelivered ? "\u2713\u2713" : "\u2713"}
            </span>
          </Show>
        </div>
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
        class="blnk-chat-row blnk-chat-row-active"
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
    <div
      class="blnk-panel blnk-chat-panel"
      onDragEnter={handleDragEnter}
      onDragLeave={handleDragLeave}
      onDragOver={handleDragOver}
    >
      {/* Drag & Drop overlay */}
      <Show when={isDragging() && props.store.selectedPeer()}>
        <div class="blnk-drop-zone">
          <div class="blnk-drop-zone-content">
            <span class="blnk-drop-zone-icon">{"\uD83D\uDCC1"}</span>
            <span class="blnk-drop-zone-text">Drop files to send</span>
          </div>
        </div>
      </Show>

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
            when={chatItems().length > 0 || props.store.activeTransfers().length > 0}
            fallback={
              <div class="blnk-chat-empty">
                No messages yet. Say hello!
              </div>
            }
          >
            <For each={chatItems()}>{(item) => renderChatItem(item)}</For>
            {/* Active transfers rendered separately to avoid full list re-render on progress ticks */}
            <For each={props.store.activeTransfers()}>
              {(t) => renderActiveTransfer(t)}
            </For>
          </Show>
        </Show>

        {/* Typing indicator */}
        <Show when={isSelectedPeerTyping()}>
          <div class="blnk-chat-row blnk-chat-row-received">
            <div class="blnk-typing-indicator">
              <span class="blnk-typing-dot" />
              <span class="blnk-typing-dot" />
              <span class="blnk-typing-dot" />
            </div>
          </div>
        </Show>
      </div>

      {/* Input bar */}
      <Show when={props.store.selectedPeer()}>
        <div class="blnk-chat-input-bar">
          <button
            class="blnk-chat-attach-btn"
            onClick={handleSendFile}
            disabled={props.store.loading()}
            title="Attach files"
          >
            {"\uD83D\uDCCE"}
          </button>
          <button
            class="blnk-chat-attach-btn"
            onClick={handleSendFolder}
            disabled={props.store.loading()}
            title="Send folder"
          >
            {"\uD83D\uDCC1"}
          </button>
          <button
            class="blnk-chat-attach-btn"
            onClick={handleSendScreenshot}
            disabled={props.store.loading()}
            title="Send screenshot"
          >
            {"\uD83D\uDCF7"}
          </button>
          <button
            class="blnk-chat-attach-btn"
            onClick={handleSendClipboard}
            disabled={props.store.loading()}
            title="Send clipboard"
          >
            {"\uD83D\uDCCB"}
          </button>
          <input
            type="text"
            class="blnk-chat-input"
            placeholder="Type a message..."
            value={textInput()}
            onInput={handleTextInput}
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
