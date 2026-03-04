import type { Accessor, Setter } from "solid-js";

// ═══ Tool identifiers ═══

export type NetopsTool =
  | "ping" | "portscan" | "dns" | "whois" | "wifi" | "wifiauth"
  | "httpheaders" | "ssl" | "geoip" | "arp"
  | "subnet" | "reversedns" | "traceroute"
  | "traffic" | "rogueap" | "logs" | "threatfeed" | "secscore" | "incidents"
  | "servicescan" | "subenum" | "dirbust" | "webfinger" | "wafdetect" | "webvuln" | "hashid" | "cipherscan"
  | "handshake";

// ═══ Result types (mirror Rust structs) ═══

export interface PingResult {
  host: string;
  latency_ms: number;
  status: string;
  timestamp: number;
}

export interface PortEntry {
  port: number;
  status: string;
  service: string;
}

export interface PortScanResult {
  host: string;
  ports: PortEntry[];
  scan_duration_ms: number;
}

export interface DnsRecord {
  record_type: string;
  name: string;
  value: string;
  ttl: number | null;
}

export interface DnsLookupResult {
  hostname: string;
  records: DnsRecord[];
  query_time_ms: number;
  server: string;
}

export interface WhoisResult {
  domain: string;
  registrar: string;
  creation_date: string;
  expiry_date: string;
  name_servers: string[];
  status: string[];
  raw: string;
}

export interface WifiNetwork {
  ssid: string;
  bssid: string;
  rssi: number;
  channel: number;
  security: string;
}

export interface SecurityHeader {
  name: string;
  present: boolean;
  value: string;
  rating: string;
}

export interface HttpHeaderResult {
  url: string;
  status_code: number;
  headers: [string, string][];
  security_headers: SecurityHeader[];
  response_time_ms: number;
}

export interface CertChainEntry {
  subject: string;
  issuer: string;
}

export interface SslCertInfo {
  domain: string;
  issuer: string;
  subject: string;
  valid_from: string;
  valid_to: string;
  days_remaining: number;
  serial: string;
  sans: string[];
  chain: CertChainEntry[];
  protocol: string;
}

export interface GeoIpResult {
  ip: string;
  country: string;
  country_code: string;
  region: string;
  city: string;
  lat: number;
  lon: number;
  isp: string;
  org: string;
  timezone: string;
}

export interface ArpEntry {
  ip: string;
  mac: string;
  interface_name: string;
  hostname: string;
}

export interface SubnetCalcResult {
  cidr: string;
  network: string;
  broadcast: string;
  first_host: string;
  last_host: string;
  netmask: string;
  wildcard: string;
  host_count: number;
  prefix_len: number;
}

export interface ReverseDnsResult {
  ip: string;
  hostnames: string[];
  query_time_ms: number;
}

export interface TracerouteHop {
  hop: number;
  ip: string;
  hostname: string;
  rtt_ms: number[];
}

export interface TracerouteResult {
  target: string;
  hops: TracerouteHop[];
  completed: boolean;
}

export interface WifiAuthEvent {
  timestamp: string;
  event_type: string;
  message: string;
}

export interface WifiAuthMonitorResult {
  time_window_hours: number;
  events: WifiAuthEvent[];
  total_failures: number;
  total_events: number;
  query_time_ms: number;
}

// ═══ Traffic Anomaly Detection ═══

export interface SuspiciousConnection {
  process: string;
  pid: string;
  protocol: string;
  local_addr: string;
  foreign_addr: string;
  port: number;
  reason: string;
  threat_level: string;
}

export interface TrafficAnomalyResult {
  connections: SuspiciousConnection[];
  total_connections: number;
  suspicious_count: number;
  scan_time_ms: number;
}

// ═══ Rogue AP Detection ═══

export interface ApStatus {
  ssid: string;
  bssid: string;
  rssi: number;
  channel: number;
  security: string;
  status: string;
  reason: string;
}

export interface RogueApResult {
  known_count: number;
  unknown_count: number;
  spoofed_count: number;
  networks: ApStatus[];
  baseline_exists: boolean;
}

// ═══ Log Aggregation ═══

export interface LogEntry {
  timestamp: string;
  level: string;
  subsystem: string;
  process: string;
  message: string;
}

export interface SystemLogsResult {
  filter: string;
  entries: LogEntry[];
  total_count: number;
  query_time_ms: number;
}

// ═══ Threat Intelligence ═══

export interface ThreatSource {
  name: string;
  listed: boolean;
  category: string;
  details: string;
}

export interface ThreatCheckResult {
  indicator: string;
  threat_score: number;
  is_threat: boolean;
  sources: ThreatSource[];
  open_ports: number[];
  vulns: string[];
  hostnames: string[];
  query_time_ms: number;
}

// ═══ Security Score ═══

export interface SecurityCheck {
  name: string;
  category: string;
  status: string;
  detail: string;
  weight: number;
}

export interface SecurityScoreResult {
  overall_score: number;
  grade: string;
  checks: SecurityCheck[];
  passed: number;
  failed: number;
  warned: number;
  total: number;
}

// ═══ Incident Tracking ═══

export interface IncidentNote {
  timestamp: string;
  content: string;
}

export interface SecurityIncident {
  id: string;
  created_at: string;
  updated_at: string;
  severity: string;
  title: string;
  description: string;
  status: string;
  notes: IncidentNote[];
}

// ═══ Kali-style tool types ═══

export interface ServiceEntry {
  port: number;
  service: string;
  version: string;
  banner: string;
  status: string;
}

export interface ServiceScanResult {
  host: string;
  services: ServiceEntry[];
  scan_time_ms: number;
}

export interface SubdomainEntry {
  subdomain: string;
  full_domain: string;
  ips: string[];
  cname: string;
}

export interface SubdomainEnumResult {
  domain: string;
  found: SubdomainEntry[];
  tested_count: number;
  found_count: number;
  scan_time_ms: number;
}

export interface DirEntry {
  path: string;
  status_code: number;
  content_length: number;
  redirect_to: string;
}

export interface DirBustResult {
  base_url: string;
  entries: DirEntry[];
  tested_count: number;
  found_count: number;
  scan_time_ms: number;
}

export interface TechMatch {
  name: string;
  category: string;
  version: string;
  evidence: string;
}

export interface WebFingerprintResult {
  url: string;
  technologies: TechMatch[];
  server: string;
  powered_by: string;
  scan_time_ms: number;
}

export interface WafIndicator {
  name: string;
  confidence: string;
  evidence: string;
}

export interface WafDetectResult {
  url: string;
  waf_detected: boolean;
  waf_name: string;
  indicators: WafIndicator[];
  normal_status: number;
  blocked_status: number;
  scan_time_ms: number;
}

export interface VulnFinding {
  title: string;
  severity: string;
  category: string;
  detail: string;
  url: string;
  remediation: string;
}

export interface WebVulnResult {
  target: string;
  findings: VulnFinding[];
  critical_count: number;
  high_count: number;
  medium_count: number;
  low_count: number;
  scan_time_ms: number;
}

export interface HashMatch {
  hash_type: string;
  confidence: string;
  description: string;
}

export interface HashIdResult {
  input: string;
  length: number;
  matches: HashMatch[];
  is_base64: boolean;
  query_time_ms: number;
}

export interface CipherInfo {
  name: string;
  protocol: string;
  bits: number;
  strength: string;
}

export interface CipherScanResult {
  host: string;
  supported_protocols: string[];
  ciphers: CipherInfo[];
  has_weak_ciphers: boolean;
  grade: string;
  preferred_cipher: string;
  scan_time_ms: number;
}

// ═══ WPA Handshake types ═══

export interface HandshakeMessage {
  step: number;
  name: string;
  description: string;
  status: string;
  timestamp: string;
}

export interface HandshakeEvent {
  timestamp: string;
  event_type: string;
  message: string;
}

export interface HandshakeResult {
  ssid: string;
  bssid: string;
  security_protocol: string;
  auth_method: string;
  pairwise_cipher: string;
  group_cipher: string;
  link_auth: string;
  handshake_messages: HandshakeMessage[];
  events: HandshakeEvent[];
  handshake_complete: boolean;
  last_handshake_time: string;
  rssi: number;
  channel: number;
  noise: number;
  scan_time_ms: number;
}

// ═══ Discriminated union for results ═══

export type ToolResult =
  | { kind: "ping"; data: PingResult }
  | { kind: "portscan"; data: PortScanResult }
  | { kind: "dns"; data: DnsLookupResult }
  | { kind: "whois"; data: WhoisResult }
  | { kind: "wifi"; data: WifiNetwork[] }
  | { kind: "httpheaders"; data: HttpHeaderResult }
  | { kind: "ssl"; data: SslCertInfo }
  | { kind: "geoip"; data: GeoIpResult }
  | { kind: "arp"; data: ArpEntry[] }
  | { kind: "subnet"; data: SubnetCalcResult }
  | { kind: "reversedns"; data: ReverseDnsResult }
  | { kind: "traceroute"; data: TracerouteResult }
  | { kind: "wifiauth"; data: WifiAuthMonitorResult }
  | { kind: "traffic"; data: TrafficAnomalyResult }
  | { kind: "rogueap"; data: RogueApResult }
  | { kind: "logs"; data: SystemLogsResult }
  | { kind: "threatfeed"; data: ThreatCheckResult }
  | { kind: "secscore"; data: SecurityScoreResult }
  | { kind: "incidents"; data: SecurityIncident[] }
  | { kind: "servicescan"; data: ServiceScanResult }
  | { kind: "subenum"; data: SubdomainEnumResult }
  | { kind: "dirbust"; data: DirBustResult }
  | { kind: "webfinger"; data: WebFingerprintResult }
  | { kind: "wafdetect"; data: WafDetectResult }
  | { kind: "webvuln"; data: WebVulnResult }
  | { kind: "hashid"; data: HashIdResult }
  | { kind: "cipherscan"; data: CipherScanResult }
  | { kind: "handshake"; data: HandshakeResult };

export interface HistoryEntry {
  tool: NetopsTool;
  target: string;
  timestamp: number;
  success: boolean;
}

// ═══ Store interface ═══

export interface NetopsStore {
  activeTool: Accessor<NetopsTool>;
  setActiveTool: Setter<NetopsTool>;
  utc: Accessor<string>;
  setUtc: Setter<string>;
  target: Accessor<string>;
  setTarget: Setter<string>;
  extraParam: Accessor<string>;
  setExtraParam: Setter<string>;
  result: Accessor<ToolResult | null>;
  setResult: Setter<ToolResult | null>;
  loading: Accessor<boolean>;
  setLoading: Setter<boolean>;
  error: Accessor<string>;
  setError: Setter<string>;
  pingHistory: Accessor<PingResult[]>;
  setPingHistory: Setter<PingResult[]>;
  history: Accessor<HistoryEntry[]>;
  setHistory: Setter<HistoryEntry[]>;
  runTool: () => Promise<void>;
  statusText: () => string;
}
