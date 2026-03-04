import { Show, For } from "solid-js";
import type { NetopsTool, NetopsStore } from "./types";

interface Props {
  store: NetopsStore;
}

const TOOL_HELP: Record<NetopsTool, { title: string; desc: string; usage: string; example: string }> = {
  ping:        { title: "HTTP Ping",        desc: "Measures round-trip latency to a host using HTTP HEAD requests.", usage: "Enter a hostname or IP address", example: "google.com" },
  portscan:    { title: "Port Scanner",     desc: "Scans TCP ports on a target host using connect() with 2-second timeout.", usage: "Enter hostname/IP. Optionally specify ports (comma-separated) in the extra field.", example: "192.168.1.1" },
  dns:         { title: "DNS Lookup",       desc: "Resolves DNS records for a hostname using the dig command.", usage: "Enter hostname. Select record type (A, AAAA, MX, CNAME, TXT).", example: "example.com" },
  whois:       { title: "WHOIS Lookup",     desc: "Retrieves domain registration info including registrar, dates, and name servers.", usage: "Enter a domain name", example: "github.com" },
  wifi:        { title: "WiFi Scanner",     desc: "Lists nearby WiFi networks with SSID, signal strength, channel, and security type.", usage: "No input needed — click Run to scan.", example: "" },
  httpheaders: { title: "HTTP Headers",     desc: "Fetches HTTP response headers and checks for security headers (CSP, HSTS, X-Frame, etc.).", usage: "Enter a URL or hostname", example: "https://github.com" },
  ssl:         { title: "SSL/TLS Inspector", desc: "Inspects the TLS certificate chain, validity dates, SANs, and protocol version.", usage: "Enter a domain name", example: "github.com" },
  geoip:       { title: "IP Geolocation",   desc: "Geolocates an IP address showing country, city, ISP, org, and coordinates.", usage: "Enter an IP address", example: "8.8.8.8" },
  arp:         { title: "ARP Table",        desc: "Shows devices on your local network with IP, MAC address, and interface.", usage: "No input needed — click Run to scan.", example: "" },
  subnet:      { title: "Subnet Calculator", desc: "Computes network range, broadcast, host count, netmask, and wildcard from CIDR notation.", usage: "Enter a CIDR notation", example: "192.168.1.0/24" },
  reversedns:  { title: "Reverse DNS",     desc: "Resolves an IP address to its hostname(s) using PTR records.", usage: "Enter an IP address", example: "8.8.8.8" },
  traceroute:  { title: "Traceroute",       desc: "Traces the network path to a target, showing each hop with latency.", usage: "Enter a hostname or IP", example: "google.com" },
};

export default function InfoPanel(props: Props) {
  const help = () => TOOL_HELP[props.store.activeTool()];

  return (
    <div class="nops-panel-col nops-right">
      <div class="nops-panel">
        <div class="nops-panel-hdr">TOOL INFO</div>
        <div class="nops-help-section">
          <div class="nops-help-title">{help().title}</div>
          <p class="nops-help-desc">{help().desc}</p>
          <div class="nops-help-usage">
            <span class="nops-help-label">USAGE</span>
            <span>{help().usage}</span>
          </div>
          <Show when={help().example}>
            <div class="nops-help-usage">
              <span class="nops-help-label">EXAMPLE</span>
              <span class="nops-help-example">{help().example}</span>
            </div>
          </Show>
        </div>
      </div>

      <div class="nops-panel nops-panel-grow">
        <div class="nops-panel-hdr">SCAN HISTORY</div>
        <div class="nops-history-list">
          <Show when={props.store.history().length === 0}>
            <div class="nops-history-empty">No scans yet</div>
          </Show>
          <For each={props.store.history().slice(0, 20)}>
            {(entry) => (
              <div class="nops-history-row">
                <span class={`nops-history-dot ${entry.success ? "success" : "fail"}`} />
                <span class="nops-history-tool">{entry.tool.toUpperCase()}</span>
                <span class="nops-history-target">{entry.target || "—"}</span>
                <span class="nops-history-time">
                  {new Date(entry.timestamp).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" })}
                </span>
              </div>
            )}
          </For>
        </div>
      </div>
    </div>
  );
}
