// ═══════════════════════════════════════════════════════════════════════════
//  BharatLink Types — mirrors Rust types from bharatlink.rs
// ═══════════════════════════════════════════════════════════════════════════

import { Accessor, Setter } from "solid-js";

// ─── Node & Peer ─────────────────────────────────────────────────────────

export interface NodeInfo {
  node_id: string;
  node_id_short: string;
  is_running: boolean;
  relay_url: string | null;
  local_addrs: string[];
  discovered_peers: number;
}

export interface PeerInfo {
  node_id: string;
  node_id_short: string;
  nickname: string | null;
  is_trusted: boolean;
  is_connected: boolean;
  last_seen: number | null;
}

// ─── Transfer ────────────────────────────────────────────────────────────

export type TransferType = "file" | "text";
export type TransferStatus =
  | "pending"
  | "in_progress"
  | "complete"
  | "completed"
  | "failed"
  | "cancelled"
  | "rejected";
export type TransferDirection = "send" | "receive";

export interface TransferRequest {
  id: string;
  from_peer: string;
  from_nickname: string | null;
  transfer_type: TransferType;
  filename: string | null;
  file_size: number | null;
  text_preview: string | null;
  timestamp: number;
}

export interface TransferProgress {
  transfer_id: string;
  direction: TransferDirection;
  filename: string | null;
  bytes_transferred: number;
  total_bytes: number;
  percent: number;
  speed_bps: number;
  status: TransferStatus;
  error: string | null;
}

export interface TransferHistoryEntry {
  id: string;
  direction: TransferDirection;
  peer_id: string;
  peer_nickname: string | null;
  transfer_type: TransferType;
  filename: string | null;
  file_size: number | null;
  text_content: string | null;
  status: TransferStatus;
  timestamp: number;
  duration_ms: number | null;
  save_path: string | null;
}

// ─── Settings ────────────────────────────────────────────────────────────

export interface BharatLinkSettings {
  auto_start: boolean;
  accept_from_trusted_only: boolean;
  auto_accept_text: boolean;
  auto_accept_from_trusted: boolean;
  download_dir: string;
  device_name: string;
  max_concurrent_transfers: number;
}

// ─── View ────────────────────────────────────────────────────────────────

export type BharatLinkView = "peers" | "send" | "receive" | "history" | "settings";

// ─── Store (returned by useBharatLinkData) ───────────────────────────────

export interface BharatLinkStore {
  // Node state
  nodeInfo: Accessor<NodeInfo | null>;
  isRunning: Accessor<boolean>;

  // Peers
  peers: Accessor<PeerInfo[]>;
  selectedPeer: Accessor<string | null>;
  setSelectedPeer: Setter<string | null>;

  // Transfers
  activeTransfers: Accessor<TransferProgress[]>;
  pendingRequests: Accessor<TransferRequest[]>;
  history: Accessor<TransferHistoryEntry[]>;

  // Settings
  settings: Accessor<BharatLinkSettings | null>;

  // UI
  loading: Accessor<boolean>;
  error: Accessor<string | null>;
  utc: Accessor<string>;
  statusText: () => string;

  // Actions
  startNode: () => Promise<void>;
  stopNode: () => Promise<void>;
  refreshPeers: () => Promise<void>;
  addPeer: (nodeId: string, nickname?: string) => Promise<void>;
  trustPeer: (nodeId: string, nickname: string) => Promise<void>;
  untrustPeer: (nodeId: string) => Promise<void>;
  sendFile: (peerId: string, filePath: string) => Promise<void>;
  sendText: (peerId: string, text: string) => Promise<void>;
  acceptTransfer: (requestId: string) => Promise<void>;
  rejectTransfer: (requestId: string) => Promise<void>;
  cancelTransfer: (transferId: string) => Promise<void>;
  refreshHistory: () => Promise<void>;
  clearHistory: () => Promise<void>;
  getSettings: () => Promise<void>;
  updateSettings: (settings: BharatLinkSettings) => Promise<void>;
}
