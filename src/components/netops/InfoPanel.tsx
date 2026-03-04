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
  wifiauth:    { title: "WiFi Auth Monitor", desc: "Monitors WiFi authentication events from macOS system logs. Shows failures, deauthentications, and association events.", usage: "Optionally enter time window in hours (1-24). Click Run to scan logs.", example: "" },
  traffic:     { title: "Traffic Anomalies", desc: "Captures network connections via lsof and flags suspicious patterns — high ports, IRC, BitTorrent, many connections to same IP.", usage: "No input needed — click Run to scan.", example: "" },
  rogueap:     { title: "Rogue AP Detect", desc: "WiFi scan compared against stored baseline. Flags unknown and potentially spoofed (evil twin) access points.", usage: "Select Scan or Save Baseline. Save first to establish trusted APs.", example: "" },
  logs:        { title: "Log Viewer", desc: "Queries macOS system logs for security-relevant events. Filter by category: security, network, firewall, auth, or all.", usage: "Select filter category from dropdown. Click Run.", example: "" },
  threatfeed:  { title: "Threat Intel", desc: "Checks an IP against public threat feeds (Shodan InternetDB, Spamhaus, abuse.ch). Shows open ports, vulns, and threat score.", usage: "Enter an IP address", example: "8.8.8.8" },
  secscore:    { title: "Security Score", desc: "Comprehensive local security assessment. Checks firewall, FileVault, SIP, Gatekeeper, auto-updates, and more.", usage: "No input needed — click Run to assess.", example: "" },
  incidents:   { title: "Incidents", desc: "Local incident management. Create, list, and update security incidents with severity and status tracking.", usage: "Empty = list all. 'new:severity:title:desc' to create. 'update:ID:status:note' to update.", example: "new:high:Suspicious login:Multiple failed SSH attempts" },
  servicescan: { title: "Service Scan", desc: "Enhanced port scan with TCP banner grabbing. Connects to open ports and reads service banners to identify software and versions.", usage: "Enter hostname or IP. Optionally specify ports (comma-separated) in extra field.", example: "scanme.nmap.org" },
  subenum:     { title: "Subdomain Enum", desc: "DNS brute force with ~90 common subdomain wordlist (www, mail, api, dev, staging, admin, cdn, vpn, etc.).", usage: "Enter a domain name", example: "example.com" },
  dirbust:     { title: "Dir Brute Force", desc: "HTTP path enumeration with ~80 built-in paths. Checks for admin panels, config files, backups, API docs, and sensitive files.", usage: "Enter a URL", example: "https://example.com" },
  webfinger:   { title: "Web Fingerprint", desc: "Technology detection from HTTP headers, cookies, and HTML body. Identifies servers, frameworks, CDNs, CMS, and JS libraries.", usage: "Enter a URL", example: "https://github.com" },
  wafdetect:   { title: "WAF Detection", desc: "Detects Web Application Firewalls by comparing normal vs attack-pattern responses. Checks header signatures and status codes.", usage: "Enter a URL", example: "https://cloudflare.com" },
  webvuln:     { title: "Vuln Scanner", desc: "Nikto-lite web vulnerability scanner. Checks for exposed files, directory listing, missing security headers, CORS issues, and more.", usage: "Enter a URL", example: "https://example.com" },
  hashid:      { title: "Hash Identifier", desc: "Identifies hash type from input string. Supports MD5, SHA-1/256/512, bcrypt, NTLM, MySQL, Unix crypt, Argon2, JWT, and Base64.", usage: "Enter a hash string", example: "5d41402abc4b2a76b9719d911017c592" },
  cipherscan:  { title: "Cipher Scan", desc: "Enumerates supported TLS protocol versions and cipher suites. Grades configuration A-F based on protocol and cipher strength.", usage: "Enter a domain name", example: "github.com" },
  handshake:   { title: "WPA Handshake", desc: "Analyzes the current WiFi connection's WPA/WPA2/WPA3 4-way handshake status, security parameters, and recent EAPOL events from system logs.", usage: "No input needed — click Run to analyze current connection.", example: "" },
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
