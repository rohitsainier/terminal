// ═══════════════════════════════════════════════════════════════════════════
//  useBharatLinkData — SolidJS hook for BharatLink P2P state management
// ═══════════════════════════════════════════════════════════════════════════

import { createSignal, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  BharatLinkStore,
  BharatLinkSignal,
  NodeInfo,
  PeerInfo,
  TransferProgress,
  TransferRequest,
  TransferHistoryEntry,
  BharatLinkSettings,
} from "./types";

export function useBharatLinkData(): BharatLinkStore {
  // ─── Signals ─────────────────────────────────────────────────────────
  const [nodeInfo, setNodeInfo] = createSignal<NodeInfo | null>(null);
  const [isRunning, setIsRunning] = createSignal(false);
  const [peers, setPeers] = createSignal<PeerInfo[]>([]);
  const [selectedPeer, setSelectedPeer] = createSignal<string | null>(null);
  const [activeTransfers, setActiveTransfers] = createSignal<TransferProgress[]>([]);
  const [pendingRequests, setPendingRequests] = createSignal<TransferRequest[]>([]);
  const [history, setHistory] = createSignal<TransferHistoryEntry[]>([]);
  const [settings, setSettings] = createSignal<BharatLinkSettings | null>(null);
  const [deliveredMessages, setDeliveredMessages] = createSignal<Set<string>>(new Set());
  const [typingPeers, setTypingPeers] = createSignal<Set<string>>(new Set());
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [utc, setUtc] = createSignal("");

  // ─── UTC Clock ───────────────────────────────────────────────────────
  const updateUtc = () => {
    const now = new Date();
    setUtc(
      now.toISOString().replace("T", " ").substring(0, 19) + " UTC"
    );
  };
  updateUtc();
  const clockInterval = setInterval(updateUtc, 1000);

  // ─── Event Listeners ────────────────────────────────────────────────
  const unlisteners: UnlistenFn[] = [];

  const setupListeners = async () => {
    unlisteners.push(
      await listen<PeerInfo>("bharatlink-peer-discovered", (e) => {
        setPeers((prev) => {
          const existing = prev.findIndex((p) => p.node_id === e.payload.node_id);
          if (existing >= 0) {
            const updated = [...prev];
            updated[existing] = e.payload;
            return updated;
          }
          return [...prev, e.payload];
        });
      })
    );

    unlisteners.push(
      await listen<string>("bharatlink-peer-lost", (e) => {
        // Mark peer as offline instead of removing (so trusted peers stay visible)
        setPeers((prev) =>
          prev.map((p) =>
            p.node_id === e.payload ? { ...p, is_connected: false } : p
          )
        );
      })
    );

    unlisteners.push(
      await listen<TransferRequest>("bharatlink-incoming-request", (e) => {
        setPendingRequests((prev) => [...prev, e.payload]);
      })
    );

    unlisteners.push(
      await listen<TransferProgress>("bharatlink-transfer-progress", (e) => {
        setActiveTransfers((prev) => {
          const idx = prev.findIndex(
            (t) => t.transfer_id === e.payload.transfer_id
          );
          if (idx >= 0) {
            const updated = [...prev];
            updated[idx] = e.payload;
            return updated;
          }
          return [...prev, e.payload];
        });
      })
    );

    unlisteners.push(
      await listen<TransferHistoryEntry>("bharatlink-transfer-complete", (e) => {
        // Remove from active transfers
        setActiveTransfers((prev) =>
          prev.filter((t) => t.transfer_id !== e.payload.id)
        );
        // Remove from pending if it was incoming
        setPendingRequests((prev) =>
          prev.filter((r) => r.id !== e.payload.id)
        );
        // Add to history (append at end so newest is last — matches chat scroll)
        setHistory((prev) => [...prev, e.payload].slice(-500));
      })
    );

    unlisteners.push(
      await listen<NodeInfo>("bharatlink-node-status", (e) => {
        setNodeInfo(e.payload);
        setIsRunning(e.payload.is_running);
      })
    );

    // Signal listener (read receipts, typing indicators)
    unlisteners.push(
      await listen<BharatLinkSignal>("bharatlink-signal", (e) => {
        const signal = e.payload;
        if (signal.signal_type === "delivered") {
          // Mark ALL sent messages to this peer as delivered
          const peerSentIds = history()
            .filter((h) => h.peer_id === signal.from_peer && h.direction === "send")
            .map((h) => h.id);
          setDeliveredMessages((prev) => {
            const next = new Set(prev);
            for (const id of peerSentIds) next.add(id);
            return next;
          });
        } else if (signal.signal_type === "typing") {
          setTypingPeers((prev) => {
            const next = new Set(prev);
            next.add(signal.from_peer);
            return next;
          });
          // Auto-clear typing after 4 seconds
          setTimeout(() => {
            setTypingPeers((prev) => {
              const next = new Set(prev);
              next.delete(signal.from_peer);
              return next;
            });
          }, 4000);
        } else if (signal.signal_type === "stop_typing") {
          setTypingPeers((prev) => {
            const next = new Set(prev);
            next.delete(signal.from_peer);
            return next;
          });
        }
      })
    );
  };

  setupListeners();

  onCleanup(() => {
    clearInterval(clockInterval);
    unlisteners.forEach((fn) => fn());
  });

  // ─── Actions ─────────────────────────────────────────────────────────
  const startNode = async () => {
    setLoading(true);
    setError(null);
    try {
      const info = await invoke<NodeInfo>("bharatlink_start");
      setNodeInfo(info);
      setIsRunning(true);
      await refreshPeers();
      await getSettings();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const stopNode = async () => {
    setLoading(true);
    try {
      await invoke("bharatlink_stop");
      setNodeInfo(null);
      setIsRunning(false);
      setPeers([]);
      setActiveTransfers([]);
      setPendingRequests([]);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const refreshPeers = async () => {
    try {
      const list = await invoke<PeerInfo[]>("bharatlink_get_peers");
      setPeers(list);
    } catch (e) {
      setError(String(e));
    }
  };

  const addPeer = async (nodeId: string, nickname?: string) => {
    try {
      await invoke("bharatlink_add_peer", {
        nodeId,
        nickname: nickname || null,
      });
      await refreshPeers();
    } catch (e) {
      setError(String(e));
    }
  };

  const trustPeer = async (nodeId: string, nickname: string) => {
    try {
      await invoke("bharatlink_trust_peer", { nodeId, nickname });
      await refreshPeers();
    } catch (e) {
      setError(String(e));
    }
  };

  const untrustPeer = async (nodeId: string) => {
    try {
      await invoke("bharatlink_untrust_peer", { nodeId });
      await refreshPeers();
    } catch (e) {
      setError(String(e));
    }
  };

  const sendFile = async (peerId: string, filePath: string) => {
    setError(null);
    try {
      await invoke("bharatlink_send_file", { peerId, filePath });
    } catch (e) {
      setError(String(e));
    }
  };

  const sendFiles = async (peerId: string, filePaths: string[]) => {
    setError(null);
    try {
      await invoke("bharatlink_send_files", { peerId, filePaths });
    } catch (e) {
      setError(String(e));
    }
  };

  const sendText = async (peerId: string, text: string) => {
    setError(null);
    try {
      await invoke("bharatlink_send_text", { peerId, text });
    } catch (e) {
      setError(String(e));
    }
  };

  const sendClipboard = async (peerId: string, text: string) => {
    setError(null);
    try {
      await invoke("bharatlink_send_clipboard", { peerId, clipboardText: text });
    } catch (e) {
      setError(String(e));
    }
  };

  const captureAndSendScreenshot = async (peerId: string) => {
    setError(null);
    try {
      await invoke("bharatlink_capture_screenshot", { peerId });
    } catch (e) {
      setError(String(e));
    }
  };

  const acceptTransfer = async (requestId: string) => {
    try {
      await invoke("bharatlink_accept_transfer", { requestId });
      setPendingRequests((prev) => prev.filter((r) => r.id !== requestId));
    } catch (e) {
      setError(String(e));
    }
  };

  const rejectTransfer = async (requestId: string) => {
    try {
      await invoke("bharatlink_reject_transfer", { requestId });
      setPendingRequests((prev) => prev.filter((r) => r.id !== requestId));
    } catch (e) {
      setError(String(e));
    }
  };

  const cancelTransfer = async (transferId: string) => {
    try {
      await invoke("bharatlink_cancel_transfer", { transferId });
      setActiveTransfers((prev) =>
        prev.filter((t) => t.transfer_id !== transferId)
      );
    } catch (e) {
      setError(String(e));
    }
  };

  const refreshHistory = async () => {
    try {
      const h = await invoke<TransferHistoryEntry[]>("bharatlink_get_history");
      // Sort by timestamp ascending (oldest first) for chat scroll
      h.sort((a, b) => a.timestamp - b.timestamp);
      setHistory(h);
    } catch (e) {
      setError(String(e));
    }
  };

  const clearHistory = async () => {
    try {
      await invoke("bharatlink_clear_history");
      setHistory([]);
    } catch (e) {
      setError(String(e));
    }
  };

  const getSettings = async () => {
    try {
      const s = await invoke<BharatLinkSettings>("bharatlink_get_settings");
      setSettings(s);
    } catch (e) {
      setError(String(e));
    }
  };

  const updateSettings = async (s: BharatLinkSettings) => {
    try {
      await invoke("bharatlink_update_settings", { settings: s });
      setSettings(s);
    } catch (e) {
      setError(String(e));
    }
  };

  const sendSignal = async (peerId: string, signalType: string, messageId?: string) => {
    try {
      await invoke("bharatlink_send_signal", {
        peerId,
        signalType,
        messageId: messageId || null,
      });
    } catch {
      // Signals are fire-and-forget, ignore errors
    }
  };

  const statusText = () => {
    if (loading()) return "PROCESSING...";
    if (error()) return `ERROR: ${error()}`;
    if (!isRunning()) return "NODE OFFLINE";
    const n = nodeInfo();
    if (!n) return "INITIALIZING...";
    const peerCount = peers().length;
    const activeCount = activeTransfers().length;
    let status = `NODE ONLINE · ${peerCount} peer${peerCount !== 1 ? "s" : ""}`;
    if (activeCount > 0) {
      status += ` · ${activeCount} active transfer${activeCount !== 1 ? "s" : ""}`;
    }
    return status;
  };

  // ─── Return Store ────────────────────────────────────────────────────
  return {
    nodeInfo,
    isRunning,
    peers,
    selectedPeer,
    setSelectedPeer,
    activeTransfers,
    pendingRequests,
    history,
    deliveredMessages,
    typingPeers,
    settings,
    loading,
    error,
    utc,
    statusText,
    startNode,
    stopNode,
    refreshPeers,
    addPeer,
    trustPeer,
    untrustPeer,
    sendFile,
    sendFiles,
    sendText,
    sendClipboard,
    captureAndSendScreenshot,
    acceptTransfer,
    rejectTransfer,
    cancelTransfer,
    refreshHistory,
    clearHistory,
    getSettings,
    updateSettings,
    sendSignal,
  };
}
