import { Show, For, Switch, Match } from "solid-js";
import type { NetopsStore, NetopsTool } from "./types";

interface Props {
  store: NetopsStore;
}

const TOOL_PLACEHOLDERS: Record<NetopsTool, string> = {
  ping: "google.com",
  portscan: "192.168.1.1",
  dns: "example.com",
  whois: "github.com",
  wifi: "",
  httpheaders: "https://github.com",
  ssl: "github.com",
  geoip: "8.8.8.8",
  arp: "",
  subnet: "192.168.1.0/24",
  reversedns: "8.8.8.8",
  traceroute: "google.com",
  wifiauth: "",
  traffic: "",
  rogueap: "",
  logs: "",
  threatfeed: "8.8.8.8",
  secscore: "",
  incidents: "",
};

const TOOL_LABELS: Record<NetopsTool, string> = {
  ping: "Host",
  portscan: "Host",
  dns: "Hostname",
  whois: "Domain",
  wifi: "",
  httpheaders: "URL",
  ssl: "Domain",
  geoip: "IP Address",
  arp: "",
  subnet: "CIDR",
  reversedns: "IP Address",
  traceroute: "Target",
  wifiauth: "",
  traffic: "",
  rogueap: "",
  logs: "",
  threatfeed: "IP Address",
  secscore: "",
  incidents: "",
};

const DNS_TYPES = ["A", "AAAA", "MX", "CNAME", "TXT", "NS", "SOA"];
const LOG_FILTERS = ["all", "security", "network", "firewall", "auth"];
const ROGUEAP_MODES = ["scan", "save"];
const NO_TARGET_TOOLS: NetopsTool[] = ["wifi", "arp", "wifiauth", "traffic", "rogueap", "logs", "secscore", "incidents"];

export default function ResultPanel(props: Props) {
  const tool = () => props.store.activeTool();
  const needsTarget = () => !NO_TARGET_TOOLS.includes(tool());
  const needsExtra = () => tool() === "dns" || tool() === "portscan" || tool() === "wifiauth" || tool() === "logs" || tool() === "rogueap";

  function handleSubmit(e: Event) {
    e.preventDefault();
    props.store.runTool();
  }

  return (
    <div class="nops-center">
      {/* Input bar */}
      <form class="nops-input-bar" onSubmit={handleSubmit}>
        <Show when={needsTarget()}>
          <input
            class="nops-input"
            type="text"
            placeholder={TOOL_PLACEHOLDERS[tool()]}
            value={props.store.target()}
            onInput={(e) => props.store.setTarget(e.currentTarget.value)}
            autofocus
          />
        </Show>

        <Show when={tool() === "dns"}>
          <select
            class="nops-select"
            value={props.store.extraParam() || "A"}
            onChange={(e) => props.store.setExtraParam(e.currentTarget.value)}
          >
            <For each={DNS_TYPES}>
              {(t) => <option value={t}>{t}</option>}
            </For>
          </select>
        </Show>

        <Show when={tool() === "portscan"}>
          <input
            class="nops-input nops-input-small"
            type="text"
            placeholder="Ports (e.g. 22,80,443)"
            value={props.store.extraParam()}
            onInput={(e) => props.store.setExtraParam(e.currentTarget.value)}
          />
        </Show>

        <Show when={tool() === "wifiauth"}>
          <input
            class="nops-input nops-input-small"
            type="number"
            min="1"
            max="24"
            placeholder="Hours (1-24)"
            value={props.store.extraParam()}
            onInput={(e) => props.store.setExtraParam(e.currentTarget.value)}
          />
        </Show>

        <Show when={tool() === "logs"}>
          <select
            class="nops-select"
            value={props.store.extraParam() || "all"}
            onChange={(e) => props.store.setExtraParam(e.currentTarget.value)}
          >
            <For each={LOG_FILTERS}>
              {(f) => <option value={f}>{f.toUpperCase()}</option>}
            </For>
          </select>
        </Show>

        <Show when={tool() === "rogueap"}>
          <select
            class="nops-select"
            value={props.store.extraParam() || "scan"}
            onChange={(e) => props.store.setExtraParam(e.currentTarget.value)}
          >
            <For each={ROGUEAP_MODES}>
              {(m) => <option value={m}>{m === "save" ? "SAVE BASELINE" : "SCAN"}</option>}
            </For>
          </select>
        </Show>

        <button
          class={`nops-run-btn ${props.store.loading() ? "loading" : ""}`}
          type="submit"
          disabled={props.store.loading()}
        >
          {props.store.loading() ? "SCANNING..." : "RUN"}
        </button>
      </form>

      {/* Error */}
      <Show when={props.store.error()}>
        <div class="nops-error">{props.store.error()}</div>
      </Show>

      {/* Results */}
      <div class="nops-results">
        <Show when={props.store.result()}>
          {(res) => (
            <Switch>
              <Match when={res().kind === "ping"}>
                {(() => {
                  const d = res() as { kind: "ping"; data: import("./types").PingResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">PING {d.data.host}</div>
                      <div class="nops-kv">
                        <span>STATUS</span>
                        <span class={d.data.status === "ok" ? "nops-status-open" : "nops-status-closed"}>{d.data.status.toUpperCase()}</span>
                      </div>
                      <div class="nops-kv">
                        <span>LATENCY</span>
                        <span>{d.data.latency_ms.toFixed(1)} ms</span>
                      </div>
                      <Show when={props.store.pingHistory().length > 1}>
                        <div class="nops-panel-hdr" style="margin-top: 12px">LATENCY HISTORY</div>
                        <div class="nops-sparkline">
                          <For each={props.store.pingHistory()}>
                            {(p) => {
                              const max = Math.max(...props.store.pingHistory().map((h) => h.latency_ms), 1);
                              const height = Math.max(2, (p.latency_ms / max) * 36);
                              return <div class="nops-spark-bar" style={`height: ${height}px`} title={`${p.latency_ms.toFixed(1)}ms`} />;
                            }}
                          </For>
                        </div>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "portscan"}>
                {(() => {
                  const d = res() as { kind: "portscan"; data: import("./types").PortScanResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">PORT SCAN — {d.data.host} ({d.data.scan_duration_ms}ms)</div>
                      <table class="nops-table">
                        <thead><tr><th>PORT</th><th>SERVICE</th><th>STATUS</th></tr></thead>
                        <tbody>
                          <For each={d.data.ports}>
                            {(p) => (
                              <tr>
                                <td>{p.port}</td>
                                <td>{p.service}</td>
                                <td class={`nops-status-${p.status}`}>{p.status.toUpperCase()}</td>
                              </tr>
                            )}
                          </For>
                        </tbody>
                      </table>
                      <div class="nops-result-summary">
                        {d.data.ports.filter((p) => p.status === "open").length} open / {d.data.ports.length} scanned
                      </div>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "dns"}>
                {(() => {
                  const d = res() as { kind: "dns"; data: import("./types").DnsLookupResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">DNS — {d.data.hostname} ({d.data.query_time_ms}ms)</div>
                      <Show when={d.data.records.length > 0} fallback={<div class="nops-no-data">No records found</div>}>
                        <table class="nops-table">
                          <thead><tr><th>TYPE</th><th>NAME</th><th>VALUE</th><th>TTL</th></tr></thead>
                          <tbody>
                            <For each={d.data.records}>
                              {(r) => (
                                <tr>
                                  <td class="nops-status-open">{r.record_type}</td>
                                  <td>{r.name}</td>
                                  <td>{r.value}</td>
                                  <td>{r.ttl ?? "—"}</td>
                                </tr>
                              )}
                            </For>
                          </tbody>
                        </table>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "whois"}>
                {(() => {
                  const d = res() as { kind: "whois"; data: import("./types").WhoisResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">WHOIS — {d.data.domain}</div>
                      <div class="nops-kv"><span>REGISTRAR</span><span>{d.data.registrar || "—"}</span></div>
                      <div class="nops-kv"><span>CREATED</span><span>{d.data.creation_date || "—"}</span></div>
                      <div class="nops-kv"><span>EXPIRES</span><span>{d.data.expiry_date || "—"}</span></div>
                      <Show when={d.data.name_servers.length > 0}>
                        <div class="nops-panel-hdr" style="margin-top: 8px">NAME SERVERS</div>
                        <For each={d.data.name_servers}>
                          {(ns) => <div class="nops-list-item">{ns}</div>}
                        </For>
                      </Show>
                      <Show when={d.data.status.length > 0}>
                        <div class="nops-panel-hdr" style="margin-top: 8px">STATUS</div>
                        <For each={d.data.status}>
                          {(s) => <div class="nops-list-item">{s}</div>}
                        </For>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "wifi"}>
                {(() => {
                  const d = res() as { kind: "wifi"; data: import("./types").WifiNetwork[] };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">WiFi NETWORKS ({d.data.length} found)</div>
                      <Show when={d.data.length > 0} fallback={<div class="nops-no-data">No networks found</div>}>
                        <table class="nops-table">
                          <thead><tr><th>SSID</th><th>SIGNAL</th><th>CH</th><th>SECURITY</th></tr></thead>
                          <tbody>
                            <For each={d.data}>
                              {(n) => (
                                <tr>
                                  <td>{n.ssid || "(hidden)"}</td>
                                  <td class={n.rssi > -50 ? "nops-status-open" : n.rssi > -70 ? "nops-status-filtered" : "nops-status-closed"}>
                                    {n.rssi} dBm {signalBars(n.rssi)}
                                  </td>
                                  <td>{n.channel}</td>
                                  <td>{n.security}</td>
                                </tr>
                              )}
                            </For>
                          </tbody>
                        </table>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "httpheaders"}>
                {(() => {
                  const d = res() as { kind: "httpheaders"; data: import("./types").HttpHeaderResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">HTTP HEADERS — {d.data.status_code} ({d.data.response_time_ms}ms)</div>

                      <div class="nops-panel-hdr" style="margin-top: 8px">SECURITY HEADERS</div>
                      <For each={d.data.security_headers}>
                        {(h) => (
                          <div class="nops-kv">
                            <span>{h.name}</span>
                            <span class={`nops-badge nops-badge-${h.rating}`}>
                              {h.present ? "PRESENT" : "MISSING"}
                            </span>
                          </div>
                        )}
                      </For>

                      <div class="nops-panel-hdr" style="margin-top: 12px">ALL HEADERS</div>
                      <For each={d.data.headers}>
                        {([k, v]) => (
                          <div class="nops-kv">
                            <span>{k}</span>
                            <span class="nops-header-val">{v}</span>
                          </div>
                        )}
                      </For>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "ssl"}>
                {(() => {
                  const d = res() as { kind: "ssl"; data: import("./types").SslCertInfo };
                  const daysClass = () =>
                    d.data.days_remaining > 30 ? "nops-status-open" :
                    d.data.days_remaining > 7 ? "nops-status-filtered" : "nops-status-closed";
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">SSL/TLS — {d.data.domain}</div>
                      <div class="nops-kv"><span>SUBJECT</span><span>{d.data.subject || "—"}</span></div>
                      <div class="nops-kv"><span>ISSUER</span><span>{d.data.issuer || "—"}</span></div>
                      <div class="nops-kv"><span>VALID FROM</span><span>{d.data.valid_from || "—"}</span></div>
                      <div class="nops-kv"><span>VALID TO</span><span>{d.data.valid_to || "—"}</span></div>
                      <div class="nops-kv">
                        <span>DAYS LEFT</span>
                        <span class={daysClass()}>{d.data.days_remaining >= 0 ? d.data.days_remaining : "EXPIRED"}</span>
                      </div>
                      <div class="nops-kv"><span>SERIAL</span><span>{d.data.serial || "—"}</span></div>
                      <div class="nops-kv"><span>PROTOCOL</span><span>{d.data.protocol || "—"}</span></div>
                      <Show when={d.data.sans.length > 0}>
                        <div class="nops-panel-hdr" style="margin-top: 8px">SUBJECT ALT NAMES</div>
                        <For each={d.data.sans}>
                          {(san) => <div class="nops-list-item">{san}</div>}
                        </For>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "geoip"}>
                {(() => {
                  const d = res() as { kind: "geoip"; data: import("./types").GeoIpResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">GEO IP — {d.data.ip}</div>
                      <div class="nops-kv"><span>COUNTRY</span><span>{d.data.country} ({d.data.country_code})</span></div>
                      <div class="nops-kv"><span>REGION</span><span>{d.data.region}</span></div>
                      <div class="nops-kv"><span>CITY</span><span>{d.data.city}</span></div>
                      <div class="nops-kv"><span>COORDS</span><span>{d.data.lat.toFixed(4)}, {d.data.lon.toFixed(4)}</span></div>
                      <div class="nops-kv"><span>ISP</span><span>{d.data.isp}</span></div>
                      <div class="nops-kv"><span>ORG</span><span>{d.data.org}</span></div>
                      <div class="nops-kv"><span>TIMEZONE</span><span>{d.data.timezone}</span></div>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "arp"}>
                {(() => {
                  const d = res() as { kind: "arp"; data: import("./types").ArpEntry[] };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">ARP TABLE ({d.data.length} devices)</div>
                      <Show when={d.data.length > 0} fallback={<div class="nops-no-data">No ARP entries</div>}>
                        <table class="nops-table">
                          <thead><tr><th>IP</th><th>MAC</th><th>HOST</th><th>IFACE</th></tr></thead>
                          <tbody>
                            <For each={d.data}>
                              {(e) => (
                                <tr>
                                  <td>{e.ip}</td>
                                  <td>{e.mac}</td>
                                  <td>{e.hostname}</td>
                                  <td>{e.interface_name}</td>
                                </tr>
                              )}
                            </For>
                          </tbody>
                        </table>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "subnet"}>
                {(() => {
                  const d = res() as { kind: "subnet"; data: import("./types").SubnetCalcResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">SUBNET — {d.data.cidr}</div>
                      <div class="nops-kv"><span>NETWORK</span><span>{d.data.network}</span></div>
                      <div class="nops-kv"><span>BROADCAST</span><span>{d.data.broadcast}</span></div>
                      <div class="nops-kv"><span>FIRST HOST</span><span>{d.data.first_host}</span></div>
                      <div class="nops-kv"><span>LAST HOST</span><span>{d.data.last_host}</span></div>
                      <div class="nops-kv"><span>NETMASK</span><span>{d.data.netmask}</span></div>
                      <div class="nops-kv"><span>WILDCARD</span><span>{d.data.wildcard}</span></div>
                      <div class="nops-kv"><span>HOST COUNT</span><span class="nops-status-open">{d.data.host_count.toLocaleString()}</span></div>
                      <div class="nops-kv"><span>PREFIX</span><span>/{d.data.prefix_len}</span></div>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "reversedns"}>
                {(() => {
                  const d = res() as { kind: "reversedns"; data: import("./types").ReverseDnsResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">REVERSE DNS — {d.data.ip} ({d.data.query_time_ms}ms)</div>
                      <Show when={d.data.hostnames.length > 0} fallback={<div class="nops-no-data">No PTR records found</div>}>
                        <For each={d.data.hostnames}>
                          {(h) => <div class="nops-list-item nops-status-open">{h}</div>}
                        </For>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "traceroute"}>
                {(() => {
                  const d = res() as { kind: "traceroute"; data: import("./types").TracerouteResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">TRACEROUTE — {d.data.target}</div>
                      <table class="nops-table">
                        <thead><tr><th>#</th><th>HOST</th><th>IP</th><th>RTT</th></tr></thead>
                        <tbody>
                          <For each={d.data.hops}>
                            {(hop) => (
                              <tr>
                                <td>{hop.hop}</td>
                                <td>{hop.hostname}</td>
                                <td class={hop.ip === "*" ? "nops-status-closed" : ""}>{hop.ip}</td>
                                <td>
                                  {hop.rtt_ms.length > 0
                                    ? hop.rtt_ms.map((r) => `${r.toFixed(1)}ms`).join(" / ")
                                    : "* * *"}
                                </td>
                              </tr>
                            )}
                          </For>
                        </tbody>
                      </table>
                      <div class="nops-result-summary">
                        {d.data.hops.length} hops — {d.data.completed ? "COMPLETE" : "PARTIAL"}
                      </div>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "wifiauth"}>
                {(() => {
                  const d = res() as { kind: "wifiauth"; data: import("./types").WifiAuthMonitorResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">WiFi AUTH MONITOR — Last {d.data.time_window_hours}h ({d.data.query_time_ms}ms)</div>
                      <div class="nops-kv">
                        <span>TOTAL EVENTS</span>
                        <span>{d.data.total_events}</span>
                      </div>
                      <div class="nops-kv">
                        <span>FAILURES / DEAUTH / TIMEOUT</span>
                        <span class={d.data.total_failures > 0 ? "nops-status-closed" : "nops-status-open"}>
                          {d.data.total_failures}
                        </span>
                      </div>
                      <Show when={d.data.events.length > 0} fallback={<div class="nops-no-data">No auth events found in this time window</div>}>
                        <table class="nops-table" style="margin-top: 8px">
                          <thead><tr><th>TIME</th><th>TYPE</th><th>MESSAGE</th></tr></thead>
                          <tbody>
                            <For each={d.data.events}>
                              {(ev) => (
                                <tr>
                                  <td style="white-space: nowrap; font-size: 0.75rem">{ev.timestamp}</td>
                                  <td class={
                                    ev.event_type === "failure" || ev.event_type === "deauth" ? "nops-status-closed" :
                                    ev.event_type === "timeout" ? "nops-status-filtered" :
                                    ev.event_type === "success" ? "nops-status-open" : ""
                                  }>
                                    {ev.event_type.toUpperCase()}
                                  </td>
                                  <td style="font-size: 0.75rem; word-break: break-all">{ev.message}</td>
                                </tr>
                              )}
                            </For>
                          </tbody>
                        </table>
                      </Show>
                      <div class="nops-result-summary">
                        {d.data.events.length} events shown (max 500)
                      </div>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "traffic"}>
                {(() => {
                  const d = res() as { kind: "traffic"; data: import("./types").TrafficAnomalyResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">TRAFFIC ANOMALIES ({d.data.scan_time_ms}ms)</div>
                      <div class="nops-kv">
                        <span>TOTAL CONNECTIONS</span>
                        <span>{d.data.total_connections}</span>
                      </div>
                      <div class="nops-kv">
                        <span>SUSPICIOUS</span>
                        <span class={d.data.suspicious_count > 0 ? "nops-status-closed" : "nops-status-open"}>
                          {d.data.suspicious_count}
                        </span>
                      </div>
                      <Show when={d.data.connections.length > 0} fallback={<div class="nops-no-data">No suspicious connections detected</div>}>
                        <table class="nops-table" style="margin-top: 8px">
                          <thead><tr><th>PROCESS</th><th>FOREIGN</th><th>PORT</th><th>THREAT</th><th>REASON</th></tr></thead>
                          <tbody>
                            <For each={d.data.connections}>
                              {(c) => (
                                <tr>
                                  <td>{c.process} ({c.pid})</td>
                                  <td>{c.foreign_addr}</td>
                                  <td>{c.port}</td>
                                  <td>
                                    <span class={`nops-badge nops-badge-${c.threat_level === "critical" || c.threat_level === "high" ? "missing" : c.threat_level === "medium" ? "warning" : "good"}`}>
                                      {c.threat_level.toUpperCase()}
                                    </span>
                                  </td>
                                  <td style="font-size: 9px">{c.reason}</td>
                                </tr>
                              )}
                            </For>
                          </tbody>
                        </table>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "rogueap"}>
                {(() => {
                  const d = res() as { kind: "rogueap"; data: import("./types").RogueApResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">ROGUE AP SCAN ({d.data.networks.length} networks)</div>
                      <div class="nops-kv">
                        <span>BASELINE</span>
                        <span class={d.data.baseline_exists ? "nops-status-open" : "nops-status-filtered"}>
                          {d.data.baseline_exists ? "EXISTS" : "NOT SET"}
                        </span>
                      </div>
                      <div class="nops-kv"><span>TRUSTED</span><span class="nops-status-open">{d.data.known_count}</span></div>
                      <div class="nops-kv"><span>UNKNOWN</span><span class={d.data.unknown_count > 0 ? "nops-status-filtered" : ""}>{d.data.unknown_count}</span></div>
                      <div class="nops-kv">
                        <span>SPOOFED</span>
                        <span class={d.data.spoofed_count > 0 ? "nops-status-closed" : "nops-status-open"}>{d.data.spoofed_count}</span>
                      </div>
                      <Show when={d.data.networks.length > 0}>
                        <table class="nops-table" style="margin-top: 8px">
                          <thead><tr><th>SSID</th><th>BSSID</th><th>SIGNAL</th><th>CH</th><th>STATUS</th></tr></thead>
                          <tbody>
                            <For each={d.data.networks}>
                              {(n) => (
                                <tr>
                                  <td>{n.ssid || "(hidden)"}</td>
                                  <td style="font-size: 9px">{n.bssid}</td>
                                  <td>{n.rssi} dBm</td>
                                  <td>{n.channel}</td>
                                  <td>
                                    <span class={`nops-badge nops-badge-${n.status === "trusted" ? "good" : n.status === "evil_twin" ? "missing" : "warning"}`}>
                                      {n.status.toUpperCase()}
                                    </span>
                                  </td>
                                </tr>
                              )}
                            </For>
                          </tbody>
                        </table>
                      </Show>
                      <Show when={!d.data.baseline_exists}>
                        <div class="nops-result-summary">
                          No baseline set. Select "SAVE BASELINE" to save current networks as trusted.
                        </div>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "logs"}>
                {(() => {
                  const d = res() as { kind: "logs"; data: import("./types").SystemLogsResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">SYSTEM LOGS — {d.data.filter.toUpperCase()} ({d.data.query_time_ms}ms)</div>
                      <div class="nops-kv"><span>ENTRIES</span><span>{d.data.total_count}</span></div>
                      <Show when={d.data.entries.length > 0} fallback={<div class="nops-no-data">No log entries found</div>}>
                        <div class="nops-log-list" style="margin-top: 8px">
                          <For each={d.data.entries}>
                            {(entry) => (
                              <div class={`nops-log-entry nops-log-${entry.level.toLowerCase()}`}>
                                <span class="nops-log-time">{entry.timestamp}</span>
                                <span class={`nops-log-level nops-log-level-${entry.level.toLowerCase()}`}>{entry.level}</span>
                                <span class="nops-log-proc">{entry.process}</span>
                                <span class="nops-log-msg">{entry.message}</span>
                              </div>
                            )}
                          </For>
                        </div>
                      </Show>
                      <div class="nops-result-summary">{d.data.entries.length} entries shown (max 200)</div>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "threatfeed"}>
                {(() => {
                  const d = res() as { kind: "threatfeed"; data: import("./types").ThreatCheckResult };
                  const scoreClass = () =>
                    d.data.threat_score >= 70 ? "nops-score-critical" :
                    d.data.threat_score >= 40 ? "nops-score-warn" : "nops-score-safe";
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">THREAT INTEL — {d.data.indicator} ({d.data.query_time_ms}ms)</div>
                      <div class="nops-score-gauge">
                        <div class={`nops-score-num ${scoreClass()}`}>{d.data.threat_score}</div>
                        <div class="nops-score-label">THREAT SCORE</div>
                      </div>
                      <div class="nops-kv">
                        <span>THREAT STATUS</span>
                        <span class={d.data.is_threat ? "nops-status-closed" : "nops-status-open"}>
                          {d.data.is_threat ? "THREAT DETECTED" : "CLEAN"}
                        </span>
                      </div>
                      <Show when={d.data.sources.length > 0}>
                        <div class="nops-panel-hdr" style="margin-top: 8px">SOURCES</div>
                        <For each={d.data.sources}>
                          {(s) => (
                            <div class="nops-kv">
                              <span>{s.name}</span>
                              <span class={`nops-badge nops-badge-${s.listed ? "missing" : "good"}`}>
                                {s.listed ? "LISTED" : "CLEAR"}
                              </span>
                            </div>
                          )}
                        </For>
                      </Show>
                      <Show when={d.data.open_ports.length > 0}>
                        <div class="nops-panel-hdr" style="margin-top: 8px">OPEN PORTS</div>
                        <div class="nops-list-item">{d.data.open_ports.join(", ")}</div>
                      </Show>
                      <Show when={d.data.vulns.length > 0}>
                        <div class="nops-panel-hdr" style="margin-top: 8px">VULNERABILITIES</div>
                        <For each={d.data.vulns}>
                          {(v) => <div class="nops-list-item nops-status-closed">{v}</div>}
                        </For>
                      </Show>
                      <Show when={d.data.hostnames.length > 0}>
                        <div class="nops-panel-hdr" style="margin-top: 8px">HOSTNAMES</div>
                        <For each={d.data.hostnames}>
                          {(h) => <div class="nops-list-item">{h}</div>}
                        </For>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "secscore"}>
                {(() => {
                  const d = res() as { kind: "secscore"; data: import("./types").SecurityScoreResult };
                  const gradeClass = () => {
                    const g = d.data.grade;
                    if (g === "A") return "nops-grade-a";
                    if (g === "B") return "nops-grade-b";
                    if (g === "C") return "nops-grade-c";
                    if (g === "D") return "nops-grade-d";
                    return "nops-grade-f";
                  };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">SECURITY SCORE</div>
                      <div class="nops-score-gauge">
                        <div class={`nops-score-num ${gradeClass()}`}>{d.data.overall_score}</div>
                        <div class={`nops-grade-letter ${gradeClass()}`}>{d.data.grade}</div>
                        <div class="nops-score-label">OVERALL SCORE</div>
                      </div>
                      <div class="nops-kv"><span>PASSED</span><span class="nops-status-open">{d.data.passed}</span></div>
                      <div class="nops-kv"><span>WARNINGS</span><span class="nops-status-filtered">{d.data.warned}</span></div>
                      <div class="nops-kv"><span>FAILED</span><span class="nops-status-closed">{d.data.failed}</span></div>
                      <div class="nops-panel-hdr" style="margin-top: 10px">CHECKS</div>
                      <For each={d.data.checks}>
                        {(c) => (
                          <div class="nops-kv">
                            <span>{c.name}</span>
                            <span class={`nops-badge nops-badge-${c.status === "pass" ? "good" : c.status === "warn" ? "warning" : c.status === "fail" ? "missing" : "good"}`}>
                              {c.status.toUpperCase()}
                            </span>
                          </div>
                        )}
                      </For>
                      <Show when={d.data.checks.some(c => c.detail)}>
                        <div class="nops-panel-hdr" style="margin-top: 10px">DETAILS</div>
                        <For each={d.data.checks.filter(c => c.detail)}>
                          {(c) => (
                            <div class="nops-log-entry">
                              <span class="nops-log-proc">{c.name}</span>
                              <span class="nops-log-msg">{c.detail}</span>
                            </div>
                          )}
                        </For>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "incidents"}>
                {(() => {
                  const d = res() as { kind: "incidents"; data: import("./types").SecurityIncident[] };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">INCIDENTS ({d.data.length})</div>
                      <div class="nops-incident-summary">
                        <span>{d.data.filter(i => i.status === "open").length} open</span>
                        <span class="nops-status-sep">|</span>
                        <span>{d.data.filter(i => i.status === "investigating").length} investigating</span>
                        <span class="nops-status-sep">|</span>
                        <span>{d.data.filter(i => i.status === "resolved" || i.status === "closed").length} resolved</span>
                      </div>
                      <Show when={d.data.length === 0}>
                        <div class="nops-no-data">No incidents. Use target "new:severity:title:description" to create one.</div>
                      </Show>
                      <For each={d.data}>
                        {(inc) => (
                          <div class="nops-incident-card">
                            <div class="nops-incident-header">
                              <span class={`nops-badge nops-badge-${inc.severity === "critical" || inc.severity === "high" ? "missing" : inc.severity === "medium" ? "warning" : "good"}`}>
                                {inc.severity.toUpperCase()}
                              </span>
                              <span class={`nops-badge nops-badge-${inc.status === "open" ? "missing" : inc.status === "investigating" || inc.status === "mitigating" ? "warning" : "good"}`}>
                                {inc.status.toUpperCase()}
                              </span>
                              <span class="nops-incident-id">{inc.id.substring(0, 8)}</span>
                            </div>
                            <div class="nops-incident-title">{inc.title}</div>
                            <Show when={inc.description}>
                              <div class="nops-incident-desc">{inc.description}</div>
                            </Show>
                            <div class="nops-incident-meta">
                              Created: {new Date(inc.created_at).toLocaleString()} | Notes: {inc.notes.length}
                            </div>
                          </div>
                        )}
                      </For>
                    </div>
                  );
                })()}
              </Match>
            </Switch>
          )}
        </Show>

        <Show when={!props.store.result() && !props.store.error() && !props.store.loading()}>
          <div class="nops-placeholder">
            <div class="nops-placeholder-icon">⚡</div>
            <div class="nops-placeholder-text">Select a tool and enter a target to begin</div>
            <div class="nops-placeholder-hint">Press ⌘+Enter or click RUN</div>
          </div>
        </Show>
      </div>
    </div>
  );
}

function signalBars(rssi: number): string {
  if (rssi > -50) return "▂▄▆█";
  if (rssi > -60) return "▂▄▆░";
  if (rssi > -70) return "▂▄░░";
  if (rssi > -80) return "▂░░░";
  return "░░░░";
}
