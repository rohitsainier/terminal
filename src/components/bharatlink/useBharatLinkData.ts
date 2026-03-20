// ═══════════════════════════════════════════════════════════════════════════
//  useBharatLinkData — SolidJS hook for BharatLink P2P state management
// ═══════════════════════════════════════════════════════════════════════════

import { createSignal, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  BharatLinkStore,
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
        setPeers((prev) => prev.filter((p) => p.node_id !== e.payload));
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
        // Add to history
        setHistory((prev) => [e.payload, ...prev].slice(0, 500));
      })
    );

    unlisteners.push(
      await listen<NodeInfo>("bharatlink-node-status", (e) => {
        setNodeInfo(e.payload);
        setIsRunning(e.payload.is_running);
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

  const sendText = async (peerId: string, text: string) => {
    setError(null);
    try {
      await invoke("bharatlink_send_text", { peerId, text });
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
    sendText,
    acceptTransfer,
    rejectTransfer,
    cancelTransfer,
    refreshHistory,
    clearHistory,
    getSettings,
    updateSettings,
  };
}
