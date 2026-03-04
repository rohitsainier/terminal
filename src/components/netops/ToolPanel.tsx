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
  { id: "httpheaders", icon: "🔒", label: "HTTP HEADERS", desc: "Response headers" },
  { id: "ssl",         icon: "🛡️", label: "SSL/TLS",      desc: "Certificate inspect" },
  { id: "geoip",       icon: "📍", label: "GEO IP",       desc: "IP geolocation" },
  { id: "arp",         icon: "🖥️", label: "ARP TABLE",    desc: "Local devices" },
  { id: "subnet",      icon: "🧮", label: "SUBNET CALC",  desc: "CIDR calculator" },
  { id: "reversedns",  icon: "↩️", label: "REVERSE DNS",  desc: "IP to hostname" },
  { id: "traceroute",  icon: "🗺️", label: "TRACEROUTE",   desc: "Route tracing" },
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
