import { For } from "solid-js";
import type { NetopsTool, NetopsStore } from "./types";

interface Props {
  store: NetopsStore;
}

const TOOLS: { id: NetopsTool; icon: string; label: string; desc: string }[] = [
  { id: "ping",        icon: "📡", label: "PING",         desc: "HTTP latency test" },
  { id: "portscan",    icon: "🔍", label: "PORT SCAN",    desc: "TCP port probe" },
  { id: "dns",         icon: "🌐", label: "DNS LOOKUP",   desc: "DNS record query" },
  { id: "whois",       icon: "📋", label: "WHOIS",        desc: "Domain registration" },
  { id: "wifi",        icon: "📶", label: "WiFi SCAN",    desc: "Nearby networks" },
  { id: "wifiauth",   icon: "🔐", label: "WiFi AUTH",    desc: "Auth failure monitor" },
  { id: "httpheaders", icon: "🔒", label: "HTTP HEADERS", desc: "Response headers" },
  { id: "ssl",         icon: "🛡️", label: "SSL/TLS",      desc: "Certificate inspect" },
  { id: "geoip",       icon: "📍", label: "GEO IP",       desc: "IP geolocation" },
  { id: "arp",         icon: "🖥️", label: "ARP TABLE",    desc: "Local devices" },
  { id: "subnet",      icon: "🧮", label: "SUBNET CALC",  desc: "CIDR calculator" },
  { id: "reversedns",  icon: "↩️", label: "REVERSE DNS",  desc: "IP to hostname" },
  { id: "traceroute",  icon: "🗺️", label: "TRACEROUTE",   desc: "Route tracing" },
  { id: "traffic",     icon: "🚨", label: "TRAFFIC",      desc: "Anomaly detection" },
  { id: "rogueap",     icon: "👻", label: "ROGUE AP",     desc: "AP baseline check" },
  { id: "logs",        icon: "📜", label: "LOG VIEWER",   desc: "System log aggregation" },
  { id: "threatfeed",  icon: "☠️", label: "THREAT INTEL",  desc: "IP threat check" },
  { id: "secscore",    icon: "🏆", label: "SEC SCORE",    desc: "Security assessment" },
  { id: "incidents",   icon: "🔔", label: "INCIDENTS",    desc: "Incident tracking" },
  { id: "servicescan", icon: "🔬", label: "SVC SCAN",     desc: "Banner grabbing" },
  { id: "subenum",     icon: "🌍", label: "SUBDOMAINS",   desc: "DNS brute force" },
  { id: "dirbust",     icon: "📂", label: "DIR BRUTE",    desc: "Path enumeration" },
  { id: "webfinger",   icon: "🕵️", label: "FINGERPRINT",  desc: "Tech detection" },
  { id: "wafdetect",   icon: "🧱", label: "WAF DETECT",   desc: "Firewall detection" },
  { id: "webvuln",     icon: "🐛", label: "VULN SCAN",    desc: "Web vuln check" },
  { id: "hashid",      icon: "🔑", label: "HASH ID",      desc: "Hash identifier" },
  { id: "cipherscan",  icon: "🔐", label: "CIPHER SCAN",  desc: "TLS cipher enum" },
  { id: "handshake",   icon: "🤝", label: "HANDSHAKE",    desc: "WPA 4-way analysis" },
  { id: "pcapview",    icon: "📦", label: "PCAP VIEW",    desc: "Open .pcap/.cap files" },
];

export default function ToolPanel(props: Props) {
  function selectTool(id: NetopsTool) {
    props.store.setActiveTool(id);
    props.store.setResult(null);
    props.store.setError("");
  }

  return (
    <div class="nops-panel-col nops-left">
      <div class="nops-panel">
        <div class="nops-panel-hdr">TOOLS</div>
      </div>
      <div class="nops-tool-list">
        <For each={TOOLS}>
          {(tool) => (
            <div
              class={`nops-tool-row ${props.store.activeTool() === tool.id ? "active" : ""}`}
              onClick={() => selectTool(tool.id)}
            >
              <span class="nops-tool-icon">{tool.icon}</span>
              <div class="nops-tool-info">
                <span class="nops-tool-label">{tool.label}</span>
                <span class="nops-tool-desc">{tool.desc}</span>
              </div>
            </div>
          )}
        </For>
      </div>
    </div>
  );
}
